//! Functions to interact with the API directly (typically, you would use the client
//! wrappers found in the [crate::clients] module)
//!
//! In the end, all client and API functions boil down to the `X_with_args` functions. For example,
//! calling `client.read_items()` will invoke the `read_items_with_args` function and pass in the
//! client's stored configurations (the http client, token, and base URL).
//!
//! Calling the stateless functions in this module (for example, `read_items`) will instantiate an
//! HTTP client each time. That is convenient for one-off usages, but the client wrappers give you
//! a convenient way to only do that work once.
use crate::errors::{api_error, Error, Kind, Result};
use crate::models::{FeedItem, InputItem};
use crate::{api_token, env_or_default_url, normalize_item_time, X_AUTH_TOKEN_HEADER};
use serde::{Deserialize, Serialize};
use serde_json::from_str as json_from_str;
use std::time::Duration;
use tokio::time::sleep;

pub trait YupdatesV0 {
    /// Add items to a feed (using a feed-specific API token)
    ///
    /// You can send up to 10 at a time. See [YupdatesV0::new_items_all] for chunked example.
    /// Sending zero items is legal (you might want to verify the token is authorized for this
    /// call, or you might want to get the matching `feed_id` returned without adding an item).
    fn new_items(&self, items: &[InputItem]) -> Result<NewInputItemsResponse>;

    /// Add an arbitrary number of items to a feed (using a feed-specific API token)
    ///
    /// This sends all of the input items in batches, up to 10 at a time. It pauses for N ms
    /// between each call (to preemptively avoid throttling). Must be 5 or more ms.
    ///
    /// Returns feed ID
    fn new_items_all(&self, items: &[InputItem], sleep_ms: u64) -> Result<String>;

    /// Tests configuration and authentication. If this is Ok, the call worked and your API token
    /// configuration is valid. There may be permissions errors for other operations, but it was
    /// a working credential for some operations.
    fn ping(&self) -> Result<PingResponse>;

    /// Convenience for: ping() == Ok
    /// If you need error logging, use [YupdatesV0::ping] instead.
    fn ping_bool(&self) -> bool;

    /// Read items from a feed. Gets up to ten most recent items. The content is not returned (but
    /// that is an option, see [YupdatesV0::read_items_with_options]).
    fn read_items<S>(&self, feed_id: S) -> Result<Vec<FeedItem>>
    where
        S: AsRef<str>;

    /// Read items from a feed, with options. See [ReadOptions].
    fn read_items_with_options<S>(
        &self,
        feed_id: S,
        options: &ReadOptions,
    ) -> Result<Vec<FeedItem>>
    where
        S: AsRef<str>;
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// ping(): GET $base_url/ping/
// ─────────────────────────────────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct PingResponse {
    pub code: u16,
    pub message: String,
}

/// See [YupdatesV0::ping]
pub async fn ping() -> Result<PingResponse> {
    let base_url = env_or_default_url()?;
    let token = api_token()?;
    let http_client = reqwest::Client::new();
    ping_with_args(&http_client, base_url, token).await
}

/// See [YupdatesV0::ping_bool]
pub async fn ping_bool() -> bool {
    ping().await.is_ok()
}

pub async fn ping_with_args<S>(
    http_client: &reqwest::Client,
    base_url: S,
    token: S,
) -> Result<PingResponse>
where
    S: AsRef<str>,
{
    let full_url = format!("{}ping/", base_url.as_ref());
    let (code, text) = api_get(http_client, &full_url, token.as_ref()).await?;
    if code == 200 {
        Ok(json_from_str(&text)?)
    } else {
        // Including other 2XX/3XX in this category for now, they are unexpected
        Err(api_error(code, &text))
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// new_items(): POST $base_url/items/
// ─────────────────────────────────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct NewInputItemsResponse {
    pub code: u16,
    pub feed_id: String,
    pub message: String,
}

/// See [YupdatesV0::new_items]
pub async fn new_items(items: &[InputItem]) -> Result<NewInputItemsResponse> {
    let base_url = env_or_default_url()?;
    let token = api_token()?;
    let http_client = reqwest::Client::new();
    new_items_with_args(items, &http_client, base_url, token).await
}

pub async fn new_items_with_args<S>(
    items: &[InputItem],
    http_client: &reqwest::Client,
    base_url: S,
    token: S,
) -> Result<NewInputItemsResponse>
where
    S: AsRef<str>,
{
    if items.len() > 10 {
        return Err(Error {
            kind: Kind::IllegalParameter(format!(
                "too many items ({}). See chunking example (new_items_all) to send 10 at a time.",
                items.len()
            )),
        });
    }
    let data = NewItemsBody {
        items: items.to_vec(),
    };
    let full_url = format!("{}items/", base_url.as_ref());
    let (code, text) = api_post(http_client, &full_url, token.as_ref(), &data).await?;
    if code == 200 {
        Ok(json_from_str(&text)?)
    } else {
        // Including other 2XX/3XX in this category for now, they are unexpected
        Err(api_error(code, &text))
    }
}

/// See [YupdatesV0::new_items_all]
pub async fn new_items_all(items: &[InputItem], sleep_ms: u64) -> Result<String> {
    let base_url = env_or_default_url()?;
    let token = api_token()?;
    let http_client = reqwest::Client::new();
    new_items_all_with_args(items, sleep_ms, &http_client, base_url, token).await
}

pub async fn new_items_all_with_args<S>(
    items: &[InputItem],
    sleep_ms: u64,
    http_client: &reqwest::Client,
    base_url: S,
    token: S,
) -> Result<String>
where
    S: AsRef<str>,
{
    if sleep_ms < 5 {
        return Err(Error {
            kind: Kind::IllegalParameter(format!("sleep_ms ({}) must be 5 or more", sleep_ms)),
        });
    }
    let sleep_duration = Duration::from_millis(sleep_ms);

    let base_url = base_url.as_ref();
    let token = token.as_ref();

    let mut feed_id = None;
    let mut chunks = items.chunks(10).peekable();
    while let Some(chunk) = chunks.next() {
        let response = new_items_with_args(chunk, http_client, base_url, token).await?;
        if feed_id.is_none() {
            feed_id = Some(response.feed_id);
        }
        if chunks.peek().is_some() {
            sleep(sleep_duration).await;
        }
    }

    match feed_id {
        None => Err(Error {
            kind: Kind::IllegalResult("new items API success(es) without a feed ID".to_string()),
        }),
        Some(fid) => Ok(fid),
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct NewItemsBody {
    items: Vec<InputItem>,
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// read_items(): GET $base_url/feeds/$feed_id
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// Extra options for reading items.
///
/// If you don't supply `item_time_after` or `item_time_before`, the latest items are queried.
/// You cannot supply `item_time_after` and `item_time_before` at the same time.
///
/// An item time is a unix epoch millisecond with an optional 5 digit suffix. In practice, you
/// would only use the suffix form if you got that as the item time string from the service.
/// Examples: 1234, 1661564013555, "1661564013555", "1661564013555.00003", "123456.789"
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ReadOptions {
    /// The number of items to return, must be 1 <= N <= 50. Default is 10. May not be more than
    /// 10 if `include_item_content` is true.
    pub max_items: usize,

    /// If true, populate each FeedItem with the full item content.
    pub include_item_content: bool,

    /// Only return items that come after this item time (non-inclusive).
    pub item_time_after: Option<String>,

    /// Only return items that come before this item time (non-inclusive).
    pub item_time_before: Option<String>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            max_items: 10,
            include_item_content: false,
            item_time_after: None,
            item_time_before: None,
        }
    }
}

/// See [YupdatesV0::read_items]
pub async fn read_items<S>(feed_id: S, read_options: Option<&ReadOptions>) -> Result<Vec<FeedItem>>
where
    S: AsRef<str>,
{
    let base_url = env_or_default_url()?;
    let token = api_token()?;
    let http_client = reqwest::Client::new();
    read_items_with_args(
        feed_id.as_ref(),
        read_options,
        &http_client,
        &base_url,
        &token,
    )
    .await
}

pub async fn read_items_with_args<S>(
    feed_id: S,
    read_options: Option<&ReadOptions>,
    http_client: &reqwest::Client,
    base_url: S,
    token: S,
) -> Result<Vec<FeedItem>>
where
    S: AsRef<str>,
{
    let feed_id_str = feed_id.as_ref().trim();
    if feed_id_str.len() != 45 {
        return Err(Error {
            kind: Kind::IllegalParameter(format!(
                "`feed_id` is expected to be 45 characters ('{}')",
                feed_id.as_ref()
            )),
        });
    }

    let validated = match read_options.as_ref() {
        None => ReadOptions {
            ..Default::default()
        },
        Some(given) => validate_read_options(given)?,
    };

    let mut query = vec![
        ("max_items", validated.max_items.to_string()),
        (
            "include_item_content",
            validated.include_item_content.to_string(),
        ),
    ];
    if let Some(item_time_after) = validated.item_time_after {
        query.push(("item_time_after", item_time_after));
    }
    if let Some(item_time_before) = validated.item_time_before {
        query.push(("item_time_before", item_time_before));
    }

    let url = format!("{}feeds/{}/", base_url.as_ref(), feed_id_str);
    let (code, text) = api_get_with_query(http_client, &url, &query, token.as_ref()).await?;
    let response: ReadFeedItemsResponse = if code == 200 {
        json_from_str(&text)?
    } else {
        // Including other 2XX/3XX in this category for now, they are unexpected
        return Err(api_error(code, &text));
    };

    Ok(response.feed_items)
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct ReadFeedItemsResponse {
    pub code: u16,
    pub feed_items: Vec<FeedItem>,
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// IMPL
// ─────────────────────────────────────────────────────────────────────────────────────────────────

async fn api_get(
    http_client: &reqwest::Client,
    full_url: &str,
    token: &str,
) -> Result<(u16, String)> {
    let res = http_client
        .get(full_url)
        .header(X_AUTH_TOKEN_HEADER, token)
        .send()
        .await?;
    let code = res.status().as_u16();
    let text = res.text().await?;
    Ok((code, text))
}

async fn api_get_with_query<T>(
    http_client: &reqwest::Client,
    url: &str,
    query: &T,
    token: &str,
) -> Result<(u16, String)>
where
    T: Serialize + ?Sized,
{
    let res = http_client
        .get(url)
        .header(X_AUTH_TOKEN_HEADER, token)
        .query(query)
        .send()
        .await?;
    let code = res.status().as_u16();
    let text = res.text().await?;
    Ok((code, text))
}

async fn api_post<T>(
    http_client: &reqwest::Client,
    full_url: &str,
    token: &str,
    data: &T,
) -> Result<(u16, String)>
where
    T: Serialize + ?Sized,
{
    let res = http_client
        .post(full_url)
        .header(X_AUTH_TOKEN_HEADER, token)
        .json(data)
        .send()
        .await?;
    let code = res.status().as_u16();
    let text = res.text().await?;
    Ok((code, text))
}

fn validate_read_options(given: &ReadOptions) -> Result<ReadOptions> {
    if given.include_item_content && ((given.max_items < 1) || (given.max_items > 10)) {
        return Err(Error {
            kind: Kind::IllegalParameter(format!(
                "`max_items` must be 1 to 10 when `include_item_content` is true, received {}",
                given.max_items
            )),
        });
    }
    if (given.max_items < 1) || (given.max_items > 50) {
        return Err(Error {
            kind: Kind::IllegalParameter(format!(
                "`max_items` must be 1 to 50, received {}",
                given.max_items
            )),
        });
    }
    if given.item_time_after.is_some() && given.item_time_before.is_some() {
        return Err(Error {
            kind: Kind::IllegalParameter(
                "cannot simultaneously query with `item_time_after` and `item_time_before`"
                    .to_string(),
            ),
        });
    }
    let item_time_after = match &given.item_time_after {
        None => None,
        Some(it) => Some(normalize_item_time(it)?),
    };
    let item_time_before = match &given.item_time_before {
        None => None,
        Some(it) => Some(normalize_item_time(it)?),
    };
    Ok(ReadOptions {
        max_items: given.max_items,
        include_item_content: given.include_item_content,
        item_time_after,
        item_time_before,
    })
}

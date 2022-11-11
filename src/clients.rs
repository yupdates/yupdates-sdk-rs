//! Wrappers that allow you set up the configurations once and then make many calls
//!
//! There is an `async` client and a synchronous version of it which hides the need for you to
//! set up an async runtime. See the top-level documentation of this library for examples of each.
//!
//! If you want control over the `base_url`, `token`, or `http_client`, you can instantiate the
//! [AsyncYupdatesClient] and [sync::SyncYupdatesClient] structs directly.
//!
//! The HTTP client can be configured with many options, see the Reqwest library's documentation
//! for [ClientBuilder](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html), and be
//! sure to adjust the documentation version to match the right version of this dependency (see
//! this library's `Cargo.toml`).
use crate::api::{
    new_items_all_with_args, new_items_with_args, ping_with_args, read_items_with_args,
    NewInputItemsResponse, PingResponse, ReadOptions,
};
use crate::errors::Result;
use crate::models::{FeedItem, InputItem};
use crate::{api_token, env_or_default_url};

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// ASYNC CLIENT
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// Create an [AsyncYupdatesClient] instance using the default configuration sources.
pub fn new_async_client() -> Result<AsyncYupdatesClient> {
    let base_url = env_or_default_url()?;
    let http_client = reqwest::Client::new();
    let token = api_token()?;
    Ok(AsyncYupdatesClient {
        base_url,
        http_client,
        token,
    })
}

/// Create an [AsyncYupdatesClient] instance using the default configuration sources and
/// a custom [reqwest::Client]
pub fn new_async_client_with_http_client(
    http_client: reqwest::Client,
) -> Result<AsyncYupdatesClient> {
    let base_url = env_or_default_url()?;
    let token = api_token()?;
    Ok(AsyncYupdatesClient {
        base_url,
        http_client,
        token,
    })
}

/// Wraps everything needed to make async calls to the API
///
/// Instantiate this struct directly if you want total control. See [new_async_client] impl for
/// the default values.
pub struct AsyncYupdatesClient {
    pub base_url: String,
    pub http_client: reqwest::Client,
    pub token: String,
}

// Rust does not support async traits, but here we "implement" `crate::api::YupdatesV0`
impl AsyncYupdatesClient {
    /// See [crate::api::YupdatesV0::new_items]
    pub async fn new_items(&self, items: &[InputItem]) -> Result<NewInputItemsResponse> {
        new_items_with_args(items, &self.http_client, &self.base_url, &self.token).await
    }

    /// See [crate::api::YupdatesV0::new_items_all]
    pub async fn new_items_all(&self, items: &[InputItem], sleep_ms: u64) -> Result<String> {
        new_items_all_with_args(
            items,
            sleep_ms,
            &self.http_client,
            &self.base_url,
            &self.token,
        )
        .await
    }

    /// See [crate::api::YupdatesV0::ping]
    pub async fn ping(&self) -> Result<PingResponse> {
        ping_with_args(&self.http_client, &self.base_url, &self.token).await
    }

    /// See [crate::api::YupdatesV0::ping_bool]
    pub async fn ping_bool(&self) -> bool {
        self.ping().await.is_ok()
    }

    /// See [crate::api::YupdatesV0::read_items]
    pub async fn read_items<S>(&self, feed_id: S) -> Result<Vec<FeedItem>>
    where
        S: AsRef<str>,
    {
        read_items_with_args(
            feed_id.as_ref(),
            None,
            &self.http_client,
            &self.base_url,
            &self.token,
        )
        .await
    }

    /// See [crate::api::YupdatesV0::read_items_with_options]
    pub async fn read_items_with_options<S>(
        &self,
        feed_id: S,
        options: &ReadOptions,
    ) -> Result<Vec<FeedItem>>
    where
        S: AsRef<str>,
    {
        read_items_with_args(
            feed_id.as_ref(),
            Some(options),
            &self.http_client,
            &self.base_url,
            &self.token,
        )
        .await
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// SYNC CLIENT
// ─────────────────────────────────────────────────────────────────────────────────────────────────

// In the future, we would like this to be optional: #[cfg(feature = "sync_client")]
/// Alternative client that sets up and hides a [tokio::runtime::Runtime](https://docs.rs/tokio/latest/tokio/runtime/index.html)
pub mod sync {
    use crate::api::{NewInputItemsResponse, PingResponse, ReadOptions, YupdatesV0};
    use crate::clients::{new_async_client, AsyncYupdatesClient};
    use crate::errors::{Error, Result};
    use crate::models::{FeedItem, InputItem};
    use crate::Kind;
    use tokio::runtime::Runtime;

    /// Wraps everything needed to make sync calls to the API, encapsulating a Tokio runtime.
    ///
    /// This allows you to make one-off CLIs more easily. You can list just `yupdates` as a
    /// dependency and write code like `new_sync_client()?.ping()`.
    pub struct SyncYupdatesClient {
        pub client: AsyncYupdatesClient,
        pub rt: Runtime,
    }

    /// Create a [SyncYupdatesClient] instance using the default configuration sources.
    pub fn new_sync_client() -> Result<SyncYupdatesClient> {
        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(Error {
                    kind: Kind::Config(format!("Could not create Tokio runtime: {}", e)),
                })
            }
        };
        Ok(SyncYupdatesClient {
            client: new_async_client()?,
            rt,
        })
    }

    impl YupdatesV0 for SyncYupdatesClient {
        fn new_items(&self, items: &[InputItem]) -> Result<NewInputItemsResponse> {
            self.rt.block_on(self.client.new_items(items))
        }

        fn new_items_all(&self, items: &[InputItem], sleep_ms: u64) -> Result<String> {
            self.rt.block_on(self.client.new_items_all(items, sleep_ms))
        }

        fn ping(&self) -> Result<PingResponse> {
            self.rt.block_on(self.client.ping())
        }

        fn ping_bool(&self) -> bool {
            self.rt.block_on(self.client.ping_bool())
        }

        fn read_items<S>(&self, feed_id: S) -> Result<Vec<FeedItem>>
        where
            S: AsRef<str>,
        {
            self.rt.block_on(self.client.read_items(feed_id))
        }

        fn read_items_with_options<S>(
            &self,
            feed_id: S,
            options: &ReadOptions,
        ) -> Result<Vec<FeedItem>>
        where
            S: AsRef<str>,
        {
            self.rt
                .block_on(self.client.read_items_with_options(feed_id, options))
        }
    }
}

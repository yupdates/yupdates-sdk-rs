extern crate core;

use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
use std::env::VarError;
use yupdates::clients::AsyncYupdatesClient;
use yupdates::env_or_default_url;
use yupdates::errors::{Error, Kind, Result};
use yupdates::models::{AssociatedFile, InputItem};

mod test_input_items;
mod test_read_items;

pub const YUPDATES_TEST_FEED_SPECIFIC_TOKEN: &str = "YUPDATES_TEST_FEED_SPECIFIC_TOKEN";
pub const YUPDATES_TEST_RO_TOKEN: &str = "YUPDATES_TEST_RO_TOKEN";

pub fn test_clients() -> Result<(AsyncYupdatesClient, AsyncYupdatesClient)> {
    let (read_only_token, feed_token) = test_tokens()?;
    let base_url = env_or_default_url()?;
    let ro_client = AsyncYupdatesClient {
        base_url: base_url.clone(),
        http_client: Default::default(),
        token: read_only_token,
    };
    let feed_client = AsyncYupdatesClient {
        base_url,
        http_client: Default::default(),
        token: feed_token,
    };
    Ok((ro_client, feed_client))
}

pub fn test_tokens() -> Result<(String, String)> {
    let read_only = one_env("read-only API test token", YUPDATES_TEST_RO_TOKEN)?;
    let feed_specific = one_env(
        "feed-specific API test token",
        YUPDATES_TEST_FEED_SPECIFIC_TOKEN,
    )?;
    Ok((read_only, feed_specific))
}

pub fn random_ascii_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn random_test_items(num: usize) -> (Vec<InputItem>, Vec<String>) {
    let mut input_items = Vec::new();
    let mut suffixes = Vec::new();
    for i in 0..num {
        let suffix = random_ascii_string(10);
        let associated_files = if i % 2 == 0 {
            None
        } else {
            Some(vec![AssociatedFile {
                url: format!("https://www.example.com/file-{}", suffix),
                length: 1234,
                type_str: "audio/mpeg".to_string(),
            }])
        };
        let input_item = InputItem {
            title: format!("title-{}", suffix),
            content: format!("content-{}", suffix),
            canonical_url: format!("https://www.example.com/{}", suffix),
            associated_files,
        };
        suffixes.push(suffix);
        input_items.push(input_item);
    }
    suffixes.reverse();
    (input_items, suffixes)
}

fn one_env(description: &str, config: &str) -> Result<String> {
    match env::var(config) {
        Ok(s) => Ok(s),
        Err(e) => {
            let err = match e {
                VarError::NotPresent => {
                    format!("The {} is missing, set {}", description, config)
                }
                VarError::NotUnicode(_) => {
                    format!("{} is not valid unicode", config)
                }
            };
            Err(Error {
                kind: Kind::Config(err),
            })
        }
    }
}

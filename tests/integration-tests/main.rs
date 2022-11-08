use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
use std::env::VarError;
use yupdates::clients::AsyncYupdatesClient;
use yupdates::env_or_default_url;
use yupdates::errors::{Error, Kind, Result};

mod input_items;

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

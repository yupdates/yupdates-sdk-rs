//! # Yupdates Rust SDK
//!
//! The Yupdates Rust SDK lets you easily use the Yupdates API from your own software and scripts.
//!
//! The code is [hosted on GitHub](https://github.com/yupdates/yupdates-sdk-rs). Also see the
//! [Yupdates Python SDK](https://github.com/yupdates/yupdates-sdk-py).
//!
//! The [api] module provides a low-level functions that wrap calls to the HTTP+JSON API,
//! serializing and deserializing the requests and responses.
//!
//! The [clients] module provides an `async` client that is more convenient, and [clients::sync]
//! provides a synchronous version of the client that hides any need to set up an async runtime.
//!
//! The following examples require setting the `YUPDATES_API_TOKEN` environment variable.
//!
//! Synchronous client example:
//! ```rust
//! use yupdates::api::YupdatesV0;
//! use yupdates::clients::sync::new_sync_client;
//! use yupdates::errors::Error;
//!
//! fn main() -> Result<(), Error> {
//!     let feed_id = "02fb24a4478462a4491067224b66d9a8b2338ddca2737";
//!     let yup = new_sync_client()?;
//!     for item in yup.read_items(feed_id)? {
//!         println!("Title: {}", item.title);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! The following asynchronous client example requires adding `tokio` to your `Cargo.toml`.
//! For example: `tokio = { version = "1", features = ["macros"] }`
//! ```rust
//! use yupdates::clients::new_async_client;
//! use yupdates::errors::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     let feed_id = "02fb24a4478462a4491067224b66d9a8b2338ddca2737";
//!     let yup = new_async_client()?;
//!     for item in yup.read_items(feed_id).await? {
//!         println!("Title: {}", item.title);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! See the [README](https://github.com/yupdates/yupdates-sdk-rs/blob/main/README.md).
//! The SDK is distributed under the MIT license, see [LICENSE](https://github.com/yupdates/yupdates-sdk-rs/blob/main/LICENSE).

pub mod api;
pub mod clients;
pub mod errors;
pub mod models;

use crate::errors::{Error, Kind, Result};

use std::env;
use std::env::VarError;

/// The HTTP header we need on every API call
pub const X_AUTH_TOKEN_HEADER: &str = "X-Auth-Token";
/// Environment variable to consult for the API token (you can bypass this by passing the token
/// directly to certain functions)
pub const YUPDATES_API_TOKEN: &str = "YUPDATES_API_TOKEN";
/// Environment variable to consult for the base API URL. It's not usually needed: you would
/// typically only use this to exercise against an alternative API endpoint, or if you wanted
/// to downgrade API versions in the future (right now, there is only `/api/v0/`).
pub const YUPDATES_API_URL: &str = "YUPDATES_API_URL";
/// The default base URL
pub const YUPDATES_DEFAULT_API_URL: &str = "https://feeds.yupdates.com/api/v0/";

/// Retrieve the API URL from the environment or use the default.
///
/// You can override by bypassing the default setup methods. You can instantiate your own
/// `AsyncYupdatesClient` or use the functions in the `api` module directly.
pub fn env_or_default_url() -> Result<String> {
    match env::var(YUPDATES_API_URL) {
        Ok(s) => {
            if s.ends_with('/') {
                Ok(s)
            } else {
                Ok(format!("{}/", s))
            }
        }
        Err(e) => match e {
            VarError::NotPresent => Ok(YUPDATES_DEFAULT_API_URL.to_string()),
            VarError::NotUnicode(_) => Err(Error {
                kind: Kind::Config(format!("{} is not valid unicode", YUPDATES_API_URL)),
            }),
        },
    }
}

/// Retrieve the API token from the environment.
///
/// This is the default source; you can override by bypassing the default setup methods. You can
/// instantiate your own `AsyncYupdatesClient` or use the functions in the `api` module directly.
pub fn api_token() -> Result<String> {
    match env::var(YUPDATES_API_TOKEN) {
        Ok(s) => Ok(s),
        Err(e) => {
            let err = match e {
                VarError::NotPresent => {
                    format!("API token is missing, set {}", YUPDATES_API_TOKEN)
                }
                VarError::NotUnicode(_) => {
                    format!("{} is not valid unicode", YUPDATES_API_TOKEN)
                }
            };
            Err(Error {
                kind: Kind::Config(err),
            })
        }
    }
}

/// Accept many forms of item time, validate it, and return a normalized version.
///
/// An item time is a unix ms from 0 to 9_999_999_999_999. It has an optional 5 digit suffix.
/// Valid inputs: "1234", "1661564013555", "1661564013555.00003", "123456.789"
pub fn normalize_item_time<S>(item_time: S) -> Result<String>
where
    S: AsRef<str>,
{
    let it = item_time.as_ref();
    let parts = it.split('.').collect::<Vec<&str>>();
    let (base_str, slot_str) = match parts.len() {
        1 => (it, "0"),
        2 => (parts[0], parts[1]),
        _ => {
            return Err(Error {
                kind: Kind::Deserialization(format!("invalid item time: '{}'", it)),
            });
        }
    };
    let base_ms = parse_bounded_int(base_str, "base ms", 9_999_999_999_999)?;
    let slot = parse_bounded_int(slot_str, "suffix", 99_999)?;
    Ok(format!("{:0>13}.{:0>5}", base_ms, slot))
}

/// This is [normalize_item_time] for when you are using integer timestamps.
pub fn normalize_item_time_ms(item_time_ms: u64) -> Result<String> {
    normalize_item_time(item_time_ms.to_string())
}

fn parse_bounded_int(int_str: &str, name: &str, upper_bound: u64) -> Result<u64> {
    let parsed = int_str.parse::<u64>().map_err(|_| Error {
        kind: Kind::IllegalParameter(format!("invalid u64: '{}'", int_str)),
    })?;
    if parsed > upper_bound {
        return Err(Error {
            kind: Kind::IllegalParameter(format!(
                "item time {} may not be larger than {}: '{}'",
                name, upper_bound, parsed
            )),
        });
    }
    Ok(parsed)
}

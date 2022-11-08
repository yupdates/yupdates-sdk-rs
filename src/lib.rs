pub mod api;
pub mod clients;
pub mod errors;

use crate::errors::{Error, Kind, Result};

use std::env;
use std::env::VarError;

pub const YUPDATES_API_TOKEN: &str = "YUPDATES_API_TOKEN";
pub const YUPDATES_API_URL: &str = "YUPDATES_API_URL";
pub const YUPDATES_DEFAULT_API_URL: &str = "https://feeds.yupdates.com/api/v0/";
pub const X_AUTH_TOKEN_HEADER: &str = "X-Auth-Token";

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
        Err(e) => {
            match e {
                VarError::NotPresent => {
                    Ok(YUPDATES_DEFAULT_API_URL.to_string())
                }
                VarError::NotUnicode(_) => {
                    Err(Error {
                        kind: Kind::Config(format!("{} is not valid unicode", YUPDATES_API_URL)),
                    })
                }
            }
        }
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

pub mod api;
pub mod clients;
pub mod errors;

use crate::errors::{Error, Kind, Result};

use std::env;
use std::env::VarError;

pub const YUPDATES_API_TOKEN: &str = "YUPDATES_API_TOKEN";
pub const YUPDATES_API_URL: &str = "YUPDATES_API_URL";
pub const X_AUTH_TOKEN_HEADER: &str = "X-Auth-Token";

/// During the preview, there is no default URL yet and the environment variable is required.
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
            let err = match e {
                VarError::NotPresent => {
                    format!(
                        "sorry, during the preview you need to set {}",
                        YUPDATES_API_URL
                    )
                }
                VarError::NotUnicode(_) => {
                    format!("{} is not valid unicode", YUPDATES_API_URL)
                }
            };
            Err(Error {
                kind: Kind::Config(err),
            })
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
                    format!("{} is not valid unicode", YUPDATES_API_URL)
                }
            };
            Err(Error {
                kind: Kind::Config(err),
            })
        }
    }
}

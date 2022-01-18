use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_json::from_str as json_from_str;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ApiErrorData {
    pub code: Option<u16>,
    pub error: Option<String>,
    pub error_detail: Option<String>,
}

#[derive(Debug)]
pub struct Error {
    pub kind: Kind,
}

#[derive(Debug)]
pub enum Kind {
    Config(String),
    Deserialization(String),
    DetailedHttpCode(u16, String),
    HttpCode(u16),
    Reqwest(ReqwestError),
}

pub fn api_error(code: u16, text: &str) -> Error {
    match json_from_str::<ApiErrorData>(text) {
        Ok(data) => {
            let msg = msg_from_api_error_data(&data);
            Error {
                kind: Kind::DetailedHttpCode(code, msg),
            }
        }
        Err(_) => Error {
            kind: Kind::HttpCode(code),
        },
    }
}

pub fn msg_from_api_error_data(data: &ApiErrorData) -> String {
    let err = data
        .error
        .as_ref()
        .map_or(String::from(""), |s| s.to_string());
    match &data.error_detail {
        None => err,
        Some(s) => {
            if err.is_empty() {
                s.to_string()
            } else {
                format!("{} | {}", err, s)
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error {
            kind: Kind::Reqwest(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error {
            kind: Kind::Deserialization(e.to_string()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match &self.kind {
            Kind::Config(s) => {
                format!("Configuration issue: {}", s)
            }
            Kind::DetailedHttpCode(code, s) => {
                format!("HTTP {}: {}", code, s)
            }
            Kind::HttpCode(code) => {
                format!("HTTP {}", code)
            }
            Kind::Deserialization(s) => {
                format!("Problem deserializing the response: {}", s)
            }
            Kind::Reqwest(e) => {
                format!("Problem with API call: {}", e)
            }
        };
        write!(f, "{}", msg)
    }
}

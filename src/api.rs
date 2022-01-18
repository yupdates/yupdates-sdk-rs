use crate::errors::{api_error, Result};
use crate::{api_token, env_or_default_url, X_AUTH_TOKEN_HEADER};

use serde::{Deserialize, Serialize};
use serde_json::from_str as json_from_str;

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// TRAIT
// ─────────────────────────────────────────────────────────────────────────────────────────────────

pub trait YupdatesV0 {
    /// Tests configuration and authentication
    fn ping(&self) -> Result<PingResponse>;
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// PING
// ─────────────────────────────────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct PingResponse {
    pub code: u16,
    pub message: String,
}

pub async fn ping() -> Result<PingResponse> {
    let base_url = env_or_default_url()?;
    let token = api_token()?;
    ping_with_args(base_url, token).await
}

pub async fn ping_with_args<S>(base_url: S, token: S) -> Result<PingResponse>
where
    S: AsRef<str>,
{
    let full_url = format!("{}ping/", base_url.as_ref());
    let client = reqwest::Client::new();
    let res = client
        .get(&full_url)
        .header(X_AUTH_TOKEN_HEADER, token.as_ref())
        .send()
        .await?;
    let code = res.status().as_u16();
    let text = res.text().await?;
    if code == 200 {
        Ok(json_from_str(&text)?)
    } else {
        // Including other 2XX/3XX in this category for now, they are unexpected
        Err(api_error(code, &text))
    }
}

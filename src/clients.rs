use crate::api::{ping_with_args, PingResponse};
use crate::errors::Result;
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

/// Rust does not support async traits, but we 'implement' [YupdatesV0]
impl AsyncYupdatesClient {
    pub async fn ping(&self) -> Result<PingResponse> {
        ping_with_args(&self.http_client, &self.base_url, &self.token).await
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// SYNC CLIENT
// ─────────────────────────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "sync_client")]
pub mod sync {
    use crate::api::{PingResponse, YupdatesV0};
    use crate::clients::{new_async_client, AsyncYupdatesClient};
    use crate::errors::{Error, Result};
    use crate::Kind;
    use tokio::runtime::Runtime;

    /// Wraps everything needed to make sync calls to the API, encapsulating a Tokio runtime.
    ///
    /// This allows you to make one-off CLIs pretty easily. You can list just `yupdates` as a
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
        fn ping(&self) -> Result<PingResponse> {
            self.rt.block_on(self.client.ping())
        }
    }
}

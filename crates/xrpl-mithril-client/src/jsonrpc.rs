//! JSON-RPC client for connecting to XRPL nodes over HTTP.
//!
//! Uses `reqwest` with `rustls` for TLS — no OpenSSL dependency.
//!
//! # Examples
//!
//! ```no_run
//! use xrpl_mithril_client::{JsonRpcClient, Client};
//! use xrpl_mithril_models::requests::server::FeeRequest;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = JsonRpcClient::new("https://s1.ripple.com:51234")?;
//! let resp = client.request(FeeRequest {}).await?;
//! println!("Open ledger fee: {} drops", resp.drops.open_ledger_fee);
//! # Ok(())
//! # }
//! ```

use serde::Deserialize;
use xrpl_mithril_models::requests::XrplRequest;

use crate::client::Client;
use crate::error::ClientError;

/// JSON-RPC client for the XRP Ledger.
///
/// Wraps an HTTP client (`reqwest::Client`) and sends requests as JSON-RPC
/// POST payloads. Connection pooling and TLS are handled by `reqwest`.
#[derive(Debug, Clone)]
pub struct JsonRpcClient {
    http: reqwest::Client,
    url: reqwest::Url,
}

impl JsonRpcClient {
    /// Creates a new JSON-RPC client pointing to the given URL.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::InvalidUrl`] if the URL cannot be parsed.
    pub fn new(url: &str) -> Result<Self, ClientError> {
        let url = reqwest::Url::parse(url)
            .map_err(|e| ClientError::InvalidUrl(e.to_string()))?;
        Ok(Self {
            http: reqwest::Client::new(),
            url,
        })
    }

    /// Creates a client with a custom `reqwest::Client` configuration.
    ///
    /// This allows setting timeouts, custom headers, proxy settings, etc.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::InvalidUrl`] if the URL cannot be parsed.
    pub fn with_http_client(http: reqwest::Client, url: &str) -> Result<Self, ClientError> {
        let url = reqwest::Url::parse(url)
            .map_err(|e| ClientError::InvalidUrl(e.to_string()))?;
        Ok(Self { http, url })
    }

    /// Returns the URL this client connects to.
    #[must_use]
    pub fn url(&self) -> &reqwest::Url {
        &self.url
    }
}

/// The XRPL JSON-RPC response envelope.
///
/// The server wraps the actual result inside a `result` field.
#[derive(Debug, Deserialize)]
struct RpcEnvelope {
    result: serde_json::Value,
}

impl Client for JsonRpcClient {
    async fn request<R: XrplRequest + Send + Sync>(
        &self,
        request: R,
    ) -> Result<R::Response, ClientError> {
        // Serialize the request params
        let params = serde_json::to_value(&request)?;

        // Build the JSON-RPC envelope
        // XRPL uses: {"method": "...", "params": [{...}]}
        let envelope = serde_json::json!({
            "method": request.method(),
            "params": [params],
        });

        tracing::debug!(method = request.method(), "sending JSON-RPC request");

        // Send the HTTP POST
        let http_response = self
            .http
            .post(self.url.clone())
            .json(&envelope)
            .send()
            .await?;

        // Parse the response body
        let body: RpcEnvelope = http_response.json().await?;
        let result = body.result;

        // Check for RPC-level errors
        // XRPL error responses look like:
        // {"result": {"status": "error", "error": "...", "error_code": N, "error_message": "..."}}
        if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
            if status == "error" {
                let error = result
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let code = result.get("error_code").and_then(|v| v.as_i64()).map(|c| c as i32);
                let message = result
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        error.as_deref().unwrap_or("unknown error")
                    })
                    .to_string();

                return Err(ClientError::RpcError {
                    code,
                    message,
                    error,
                });
            }
        }

        // Deserialize the result into the typed response
        let response: R::Response = serde_json::from_value(result)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_valid_url() {
        let client = JsonRpcClient::new("https://s1.ripple.com:51234");
        assert!(client.is_ok());
    }

    #[test]
    fn new_with_invalid_url() {
        let client = JsonRpcClient::new("not a url");
        assert!(matches!(client, Err(ClientError::InvalidUrl(_))));
    }
}

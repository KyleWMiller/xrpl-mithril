//! Error types for the XRPL client.

use std::time::Duration;

/// Errors that can occur when communicating with an XRPL node.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ClientError {
    /// JSON-RPC error response from the server.
    #[error("RPC error: {message}")]
    RpcError {
        /// Numeric error code from the server, if present.
        code: Option<i32>,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error name (e.g., `"actNotFound"`).
        error: Option<String>,
    },

    /// HTTP transport error (connection refused, timeout, TLS, etc.).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// WebSocket transport error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// WebSocket connection closed unexpectedly.
    #[error("WebSocket connection closed: {reason}")]
    ConnectionClosed {
        /// The reason the connection was closed.
        reason: String,
    },

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// The server response was missing expected fields.
    #[error("unexpected response format: {0}")]
    UnexpectedResponse(String),

    /// Request timed out.
    #[error("request timed out after {0:?}")]
    Timeout(Duration),

    /// Invalid URL provided.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// A subscription stream ended unexpectedly.
    #[error("subscription stream ended")]
    SubscriptionEnded,
}

//! Transport-agnostic XRPL client trait.
//!
//! Both [`JsonRpcClient`](crate::JsonRpcClient) and
//! [`WebSocketClient`](crate::WebSocketClient) implement this trait,
//! allowing higher-level code (autofill, submit) to be generic over
//! the transport.

use std::future::Future;

use xrpl_mithril_models::requests::XrplRequest;

use crate::ClientError;

/// A transport-agnostic XRPL client.
///
/// Implementors handle the details of serializing requests and parsing
/// responses over their respective transports (HTTP JSON-RPC, WebSocket).
///
/// # Examples
///
/// ```ignore
/// use xrpl_mithril_client::Client;
/// use xrpl_mithril_models::requests::server::FeeRequest;
///
/// async fn get_fee(client: &impl Client) -> Result<String, xrpl_mithril_client::ClientError> {
///     let resp = client.request(FeeRequest {}).await?;
///     Ok(resp.drops.open_ledger_fee)
/// }
/// ```
pub trait Client: Send + Sync {
    /// Send a typed request and receive a typed response.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError`] on transport errors, JSON parsing errors,
    /// or RPC-level error responses from the server.
    fn request<R: XrplRequest + Send + Sync>(
        &self,
        request: R,
    ) -> impl Future<Output = Result<R::Response, ClientError>> + Send;
}

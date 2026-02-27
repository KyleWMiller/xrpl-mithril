//! Typed subscription streams for real-time XRPL events.
//!
//! These types represent messages pushed by the server on a WebSocket
//! connection after subscribing to event streams.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio::sync::mpsc;

/// A stream of subscription messages from the XRPL server.
///
/// Created by [`WebSocketClient::subscribe`](crate::WebSocketClient::subscribe).
/// Implements [`futures::Stream`] for use with `.next().await`.
///
/// # Examples
///
/// ```ignore
/// use futures::StreamExt;
///
/// let mut stream = ws_client.subscribe(subscribe_request).await?;
/// while let Some(msg) = stream.next().await {
///     println!("received: {:?}", msg);
/// }
/// ```
pub struct SubscriptionStream {
    receiver: mpsc::UnboundedReceiver<serde_json::Value>,
}

impl SubscriptionStream {
    pub(crate) fn new(receiver: mpsc::UnboundedReceiver<serde_json::Value>) -> Self {
        Self { receiver }
    }
}

impl Stream for SubscriptionStream {
    type Item = serde_json::Value;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

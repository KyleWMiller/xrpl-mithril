//! WebSocket client for connecting to XRPL nodes.
//!
//! Supports both request-response and subscription patterns over a persistent
//! WebSocket connection.
//!
//! # Architecture
//!
//! The client spawns a background `tokio` task that manages the WebSocket
//! connection. Requests are sent via an internal channel, and responses are
//! dispatched back to the caller via `oneshot` channels. Subscription messages
//! (those without an `id` field) are routed to subscription streams.
//!
//! # Examples
//!
//! ```no_run
//! use xrpl_mithril_client::{WebSocketClient, Client};
//! use xrpl_mithril_models::requests::server::ServerInfoRequest;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = WebSocketClient::connect("wss://s1.ripple.com:443").await?;
//! let resp = client.request(ServerInfoRequest {}).await?;
//! println!("Server: {:?}", resp.info.build_version);
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use xrpl_mithril_models::requests::XrplRequest;

use crate::client::Client;
use crate::error::ClientError;
use crate::subscription::SubscriptionStream;

/// A command sent from the client API to the background WebSocket task.
enum WsCommand {
    /// Send a request and expect a response.
    Request {
        payload: serde_json::Value,
        response_tx: oneshot::Sender<Result<serde_json::Value, ClientError>>,
    },
    /// Register a subscription stream.
    Subscribe {
        stream_tx: mpsc::UnboundedSender<serde_json::Value>,
    },
}

/// WebSocket client for the XRP Ledger.
///
/// Maintains a persistent WebSocket connection with a background task that
/// handles message routing. Supports both request-response (via `id` tracking)
/// and subscription streams.
pub struct WebSocketClient {
    command_tx: mpsc::UnboundedSender<WsCommand>,
    next_id: AtomicU64,
    connected: Arc<AtomicBool>,
    _task: tokio::task::JoinHandle<()>,
}

impl std::fmt::Debug for WebSocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketClient")
            .field("connected", &self.connected.load(Ordering::Relaxed))
            .finish()
    }
}

impl WebSocketClient {
    /// Connect to an XRPL WebSocket endpoint.
    ///
    /// Spawns a background task that manages the connection and routes
    /// messages between the API and the WebSocket.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::WebSocket`] if the connection fails.
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        let (ws_stream, _response) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| ClientError::WebSocket(e.to_string()))?;

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let connected = Arc::new(AtomicBool::new(true));
        let connected_clone = Arc::clone(&connected);

        let task = tokio::spawn(Self::run_loop(ws_stream, command_rx, connected_clone));

        Ok(Self {
            command_tx,
            next_id: AtomicU64::new(1),
            connected,
            _task: task,
        })
    }

    /// Subscribe to receive raw subscription messages.
    ///
    /// Returns a [`SubscriptionStream`] that yields `serde_json::Value`
    /// messages pushed by the server (ledger closes, transactions, etc.).
    ///
    /// You must also send a [`SubscribeRequest`](xrpl_mithril_models::requests::subscription::SubscribeRequest)
    /// via [`Client::request`] to tell the server what to subscribe to.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::ConnectionClosed`] if the WebSocket is disconnected.
    pub fn subscribe_stream(&self) -> Result<SubscriptionStream, ClientError> {
        let (stream_tx, stream_rx) = mpsc::unbounded_channel();
        self.command_tx
            .send(WsCommand::Subscribe { stream_tx })
            .map_err(|_| ClientError::ConnectionClosed {
                reason: "background task ended".into(),
            })?;
        Ok(SubscriptionStream::new(stream_rx))
    }

    /// Check if the WebSocket connection is still alive.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// The background event loop that manages the WebSocket connection.
    async fn run_loop(
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        mut command_rx: mpsc::UnboundedReceiver<WsCommand>,
        connected: Arc<AtomicBool>,
    ) {
        let (mut ws_sink, mut ws_source) = ws_stream.split();

        // Pending request-response pairs, keyed by request ID
        let mut pending: HashMap<u64, oneshot::Sender<Result<serde_json::Value, ClientError>>> =
            HashMap::new();

        // Subscription stream senders
        let mut subscribers: Vec<mpsc::UnboundedSender<serde_json::Value>> = Vec::new();

        loop {
            tokio::select! {
                // Handle commands from the client API
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        WsCommand::Request { payload, response_tx } => {
                            let id = payload.get("id")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            pending.insert(id, response_tx);

                            let msg = Message::Text(payload.to_string().into());
                            if let Err(e) = ws_sink.send(msg).await {
                                if let Some(tx) = pending.remove(&id) {
                                    let _ = tx.send(Err(ClientError::WebSocket(e.to_string())));
                                }
                            }
                        }
                        WsCommand::Subscribe { stream_tx } => {
                            subscribers.push(stream_tx);
                        }
                    }
                }

                // Handle messages from the WebSocket
                Some(msg_result) = ws_source.next() => {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                                // Check if this is a response to a pending request
                                if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                                    if let Some(tx) = pending.remove(&id) {
                                        // Extract the result, checking for errors
                                        let result = extract_result(&value);
                                        let _ = tx.send(result);
                                    }
                                } else {
                                    // No id — this is a subscription message
                                    // Remove closed subscribers
                                    subscribers.retain(|tx| {
                                        tx.send(value.clone()).is_ok()
                                    });
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            connected.store(false, Ordering::Relaxed);
                            break;
                        }
                        Ok(Message::Ping(data)) => {
                            let _ = ws_sink.send(Message::Pong(data)).await;
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "WebSocket error");
                            connected.store(false, Ordering::Relaxed);
                            break;
                        }
                        _ => {}
                    }
                }

                else => break,
            }
        }

        // Clean up: notify all pending requests that the connection closed
        for (_id, tx) in pending {
            let _ = tx.send(Err(ClientError::ConnectionClosed {
                reason: "WebSocket connection closed".into(),
            }));
        }
        connected.store(false, Ordering::Relaxed);
    }
}

/// Extract the result from a WebSocket response, checking for errors.
fn extract_result(value: &serde_json::Value) -> Result<serde_json::Value, ClientError> {
    // Check for error status in the result
    if let Some(result) = value.get("result") {
        if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
            if status == "error" {
                let error = result.get("error").and_then(|v| v.as_str()).map(String::from);
                let code = result
                    .get("error_code")
                    .and_then(|v| v.as_i64())
                    .map(|c| c as i32);
                let message = result
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| error.as_deref().unwrap_or("unknown error"))
                    .to_string();
                return Err(ClientError::RpcError {
                    code,
                    message,
                    error,
                });
            }
        }
        return Ok(result.clone());
    }

    // Some responses don't have a "result" wrapper
    Ok(value.clone())
}

impl Client for WebSocketClient {
    async fn request<R: XrplRequest + Send + Sync>(
        &self,
        request: R,
    ) -> Result<R::Response, ClientError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        // Serialize the request with an id field
        let mut params = serde_json::to_value(&request)?;
        if let Some(map) = params.as_object_mut() {
            map.insert("id".into(), serde_json::Value::Number(id.into()));
            map.insert(
                "command".into(),
                serde_json::Value::String(request.method().into()),
            );
        }

        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(WsCommand::Request {
                payload: params,
                response_tx,
            })
            .map_err(|_| ClientError::ConnectionClosed {
                reason: "background task ended".into(),
            })?;

        let result = response_rx.await.map_err(|_| ClientError::ConnectionClosed {
            reason: "response channel dropped".into(),
        })??;

        let response: R::Response = serde_json::from_value(result)?;
        Ok(response)
    }
}

//! Network clients for connecting to XRP Ledger nodes.
//!
//! This crate provides:
//! - JSON-RPC client over HTTP (`reqwest` + `rustls`)
//! - WebSocket client with subscription support (`tokio-tungstenite`)
//! - Transport-agnostic [`Client`] trait for generic code
//! - Typed subscription streams for real-time ledger events

#![forbid(unsafe_code)]

pub mod client;
pub mod error;
pub mod jsonrpc;
pub mod subscription;
pub mod websocket;

pub use client::Client;
pub use error::ClientError;
pub use jsonrpc::JsonRpcClient;
pub use websocket::WebSocketClient;

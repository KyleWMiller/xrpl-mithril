//! Network clients for connecting to XRP Ledger nodes.
//!
//! This crate provides:
//! - JSON-RPC client over HTTP (`reqwest` + `rustls`)
//! - WebSocket client with subscription support (`tokio-tungstenite`)
//! - Typed request/response models
//! - Retry logic with exponential backoff

#![forbid(unsafe_code)]

// Modules will be added in Phase 3
// pub mod jsonrpc;
// pub mod websocket;
// pub mod subscription;
// pub mod retry;
// pub mod error;

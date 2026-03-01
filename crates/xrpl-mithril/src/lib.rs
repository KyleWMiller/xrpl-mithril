//! xrpl-mithril — A next-generation, pure Rust SDK for the XRP Ledger.
//!
//! This is the facade crate that re-exports all sub-crates for convenience.
//! Users can depend on just `xrpl-mithril` to get everything, or pick individual
//! crates for a smaller dependency footprint.
//!
//! # Crate Organization
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | [`xrpl_types`] | Core protocol types (amounts, accounts, hashes) |
//! | [`xrpl_codec`] | Binary serialization/deserialization |
//! | [`xrpl_models`] | Transaction types, ledger objects, request/response types |
//! | [`xrpl_wallet`] | Key generation, signing, address management |
//! | [`xrpl_client`] | JSON-RPC and WebSocket clients |
//! | [`xrpl_tx`] | Transaction building, autofill, and submission |

#![forbid(unsafe_code)]

pub use xrpl_types;
pub use xrpl_codec;
pub use xrpl_models;
pub use xrpl_wallet;
pub use xrpl_client;
pub use xrpl_tx;

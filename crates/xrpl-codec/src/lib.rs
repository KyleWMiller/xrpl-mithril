//! Binary serialization and deserialization for the XRPL protocol.
//!
//! This crate implements the canonical binary encoding used by the XRP Ledger
//! for transactions, ledger objects, and other protocol messages.
//!
//! # Architecture
//!
//! - [`definitions`] — Field definitions loaded from `definitions.json`
//! - [`field_code`] — Field ID encoding (1–3 byte headers from type_code/field_code)
//! - [`error`] — Codec error types
//! - `serializer` — Canonical binary serialization
//! - `deserializer` — Binary to typed objects
//! - `signing` — Signing-specific serialization with hash prefixes

#![forbid(unsafe_code)]

pub mod definitions;
pub mod error;
pub mod field_code;

pub mod serializer;

pub mod deserializer;
pub mod signing;

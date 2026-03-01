#![doc(html_logo_url = "https://raw.githubusercontent.com/KyleWMiller/xrpl-mithril/main/assets/logo.png")]
//! Binary serialization and deserialization for the XRPL protocol.
//!
//! This crate implements the canonical binary encoding used by the XRP Ledger
//! for transactions, ledger objects, and other protocol messages.
//!
//! # Architecture
//!
//! - [`definitions`] — Field definitions loaded from `definitions.json`
//! - [`field_code`] — Field ID encoding (1-3 byte headers from type_code/field_code)
//! - [`error`] — Codec error types
//! - [`serializer`] — Canonical binary serialization
//! - [`deserializer`] — Binary to typed objects
//! - [`signing`] — Signing-specific serialization with hash prefixes
//!
//! # Examples
//!
//! Serialize a transaction to binary and deserialize it back:
//!
//! ```
//! use serde_json::json;
//! use xrpl_codec::serializer::serialize_json_object;
//! use xrpl_codec::deserializer::deserialize_object;
//!
//! // Build a minimal Payment transaction as JSON
//! let tx = json!({
//!     "TransactionType": "Payment",
//!     "Flags": 0u32,
//!     "Sequence": 1u32,
//!     "Fee": "12",
//!     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!     "Amount": "1000000"
//! });
//!
//! // Serialize to canonical binary
//! let map = tx.as_object().expect("json object");
//! let mut buf = Vec::new();
//! serialize_json_object(map, &mut buf, false)?;
//! assert!(!buf.is_empty());
//!
//! // Deserialize back to JSON
//! let decoded = deserialize_object(&buf)?;
//! assert_eq!(decoded.get("TransactionType").and_then(|v| v.as_str()), Some("Payment"));
//! assert_eq!(decoded.get("Fee").and_then(|v| v.as_str()), Some("12"));
//! assert_eq!(decoded.get("Amount").and_then(|v| v.as_str()), Some("1000000"));
//! assert_eq!(
//!     decoded.get("Account").and_then(|v| v.as_str()),
//!     Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
//! );
//! # Ok::<(), xrpl_codec::error::CodecError>(())
//! ```
//!
//! Compute a transaction's signing hash and transaction ID:
//!
//! ```
//! use serde_json::json;
//! use xrpl_codec::signing::{signing_hash, transaction_id_hex};
//!
//! let tx = json!({
//!     "TransactionType": "Payment",
//!     "Sequence": 1u32,
//!     "Fee": "12",
//!     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!     "Amount": "1000000",
//!     "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
//!     "TxnSignature": "DEADBEEFCAFE"
//! });
//! let map = tx.as_object().expect("json object");
//!
//! // Signing hash (for the signer to sign over)
//! let hash = signing_hash(map)?;
//! assert_eq!(hash.len(), 32);
//!
//! // Transaction ID (includes the signature in the hash)
//! let tx_id = transaction_id_hex(map)?;
//! assert_eq!(tx_id.len(), 64); // 32 bytes as uppercase hex
//! assert!(tx_id.chars().all(|c| c.is_ascii_hexdigit()));
//! # Ok::<(), xrpl_codec::error::CodecError>(())
//! ```

#![forbid(unsafe_code)]

pub mod definitions;
pub mod error;
pub mod field_code;

pub mod serializer;

pub mod deserializer;
pub mod signing;

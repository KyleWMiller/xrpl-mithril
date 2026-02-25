//! Custom serde helpers for XRPL JSON representations.
//!
//! The XRPL JSON format has several quirks:
//! - XRP amounts are strings of drops (e.g., `"1000000"` for 1 XRP)
//! - Issued currency amounts are objects with `value`, `currency`, `issuer` fields
//! - MPT amounts have their own object format
//! - Hashes are uppercase hex-encoded strings
//! - AccountIDs are base58check-encoded strings (classic addresses, Ripple alphabet)
//! - Blobs are uppercase hex-encoded strings
//! - Currency codes are 3-char ASCII (standard) or 40-char hex (non-standard)
//!
//! Custom `Serialize`/`Deserialize` impls are on each type directly.
//! This module provides reusable helpers for `#[serde(with = "...")]` patterns.

/// Serde helper for fields that serialize as uppercase hex strings.
///
/// Use with `#[serde(with = "crate::serde_helpers::hex_upper")]` on `Vec<u8>` fields.
pub mod hex_upper {
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serialize bytes as an uppercase hex string.
    ///
    /// # Errors
    ///
    /// Returns a serializer error if the serializer fails.
    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(bytes))
    }

    /// Deserialize bytes from a hex string (accepts upper or lowercase).
    ///
    /// # Errors
    ///
    /// Returns a deserializer error if the hex string is invalid.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

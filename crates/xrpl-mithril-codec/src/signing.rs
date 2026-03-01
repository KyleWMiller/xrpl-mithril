//! Signing-specific serialization and transaction hashing.
//!
//! This module provides hash prefixes, SHA-512/Half, and functions for
//! computing signing data and transaction IDs per the XRPL protocol.
//!
//! # Hash Prefixes
//!
//! XRPL uses 4-byte hash prefixes prepended to data before hashing to ensure
//! different contexts produce different hashes even with identical payloads:
//!
//! - `STX\0` — Single-key signing
//! - `SMT\0` — Multi-signing (appends signer's 20-byte account ID)
//! - `TXN\0` — Transaction ID computation (includes signatures)
//!
//! # SHA-512/Half
//!
//! XRPL uses SHA-512 truncated to the first 32 bytes for all protocol hashing.

use sha2::{Digest, Sha512};

use crate::error::CodecError;
use crate::serializer;

/// Hash prefix for single-key transaction signing: `STX\0` = `0x53545800`
pub const HASH_PREFIX_TRANSACTION_SIGN: [u8; 4] = [0x53, 0x54, 0x58, 0x00];

/// Hash prefix for multi-signing: `SMT\0` = `0x534D5400`
pub const HASH_PREFIX_TRANSACTION_MULTI_SIGN: [u8; 4] = [0x53, 0x4D, 0x54, 0x00];

/// Hash prefix for transaction ID computation: `TXN\0` = `0x54584E00`
pub const HASH_PREFIX_TRANSACTION_ID: [u8; 4] = [0x54, 0x58, 0x4E, 0x00];

/// Hash prefix for ledger objects: `SND\0` = `0x534E4400`
pub const HASH_PREFIX_INNER_NODE: [u8; 4] = [0x53, 0x4E, 0x44, 0x00];

/// Hash prefix for ledger header: `LWR\0` = `0x4C575200`
pub const HASH_PREFIX_LEDGER: [u8; 4] = [0x4C, 0x57, 0x52, 0x00];

/// Compute SHA-512/Half: SHA-512 of the input, truncated to the first 32 bytes.
///
/// This is the standard hash function used throughout the XRPL protocol.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_codec::signing::sha512_half;
///
/// let hash = sha512_half(b"");
/// assert_eq!(hash.len(), 32);
/// // SHA-512("") starts with cf83e1357eefb8bd...
/// assert_eq!(hash[0], 0xCF);
/// assert_eq!(hash[1], 0x83);
///
/// // Different inputs produce different hashes
/// let hash2 = sha512_half(b"hello");
/// assert_ne!(hash, hash2);
/// ```
#[must_use]
pub fn sha512_half(data: &[u8]) -> [u8; 32] {
    let full = Sha512::digest(data);
    let mut result = [0u8; 32];
    result.copy_from_slice(&full[..32]);
    result
}

/// Produce the data that a single signer signs for a transaction.
///
/// This is: `HASH_PREFIX_TRANSACTION_SIGN` + canonical binary serialization
/// (with `for_signing=true`, which excludes `TxnSignature` and `Signers`).
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use xrpl_mithril_codec::signing::signing_data;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000"
/// });
/// let map = tx.as_object().expect("json object");
/// let data = signing_data(map)?;
///
/// // First 4 bytes are the STX\0 hash prefix
/// assert_eq!(&data[..4], b"STX\0");
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if serialization fails.
pub fn signing_data(
    tx: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<u8>, CodecError> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
    serializer::serialize_json_object(tx, &mut buf, true)?;
    Ok(buf)
}

/// Compute the signing hash for single-key signing.
///
/// This is `SHA-512/Half(signing_data(tx))` -- the hash that the signer
/// signs with their private key.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use xrpl_mithril_codec::signing::signing_hash;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000"
/// });
/// let map = tx.as_object().expect("json object");
/// let hash = signing_hash(map)?;
///
/// assert_eq!(hash.len(), 32);
///
/// // The hash is deterministic
/// let hash2 = signing_hash(map)?;
/// assert_eq!(hash, hash2);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if serialization fails.
pub fn signing_hash(
    tx: &serde_json::Map<String, serde_json::Value>,
) -> Result<[u8; 32], CodecError> {
    let data = signing_data(tx)?;
    Ok(sha512_half(&data))
}

/// Produce the data that a multi-signer signs for a transaction.
///
/// This is: `HASH_PREFIX_TRANSACTION_MULTI_SIGN` + canonical binary serialization
/// (with `for_signing=true`) + raw 20-byte signer account ID.
///
/// The signer account ID is appended so that the same transaction produces
/// different signing hashes for each multi-signer.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use xrpl_mithril_codec::signing::multi_signing_data;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000"
/// });
/// let map = tx.as_object().expect("json object");
/// let signer_id = [0xAAu8; 20];
/// let data = multi_signing_data(map, &signer_id)?;
///
/// // First 4 bytes are the SMT\0 hash prefix
/// assert_eq!(&data[..4], b"SMT\0");
/// // Last 20 bytes are the signer account ID
/// assert_eq!(&data[data.len() - 20..], &signer_id);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if serialization fails or the account ID is invalid.
pub fn multi_signing_data(
    tx: &serde_json::Map<String, serde_json::Value>,
    signer_account_id: &[u8; 20],
) -> Result<Vec<u8>, CodecError> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&HASH_PREFIX_TRANSACTION_MULTI_SIGN);
    serializer::serialize_json_object(tx, &mut buf, true)?;
    buf.extend_from_slice(signer_account_id);
    Ok(buf)
}

/// Compute the multi-signing hash for a specific signer.
///
/// This is `SHA-512/Half(multi_signing_data(tx, signer_account_id))`.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use xrpl_mithril_codec::signing::multi_signing_hash;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000"
/// });
/// let map = tx.as_object().expect("json object");
///
/// // Different signers produce different hashes for the same transaction
/// let signer_a = [0xAAu8; 20];
/// let signer_b = [0xBBu8; 20];
/// let hash_a = multi_signing_hash(map, &signer_a)?;
/// let hash_b = multi_signing_hash(map, &signer_b)?;
/// assert_ne!(hash_a, hash_b);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if serialization fails.
pub fn multi_signing_hash(
    tx: &serde_json::Map<String, serde_json::Value>,
    signer_account_id: &[u8; 20],
) -> Result<[u8; 32], CodecError> {
    let data = multi_signing_data(tx, signer_account_id)?;
    Ok(sha512_half(&data))
}

/// Compute the transaction ID (hash) for a fully-signed transaction.
///
/// This is: `SHA-512/Half(HASH_PREFIX_TRANSACTION_ID + canonical_binary)`.
///
/// The canonical binary includes ALL fields (including `TxnSignature` and `Signers`),
/// produced with `for_signing=false`.
///
/// The resulting 32-byte hash is the transaction ID as shown on the ledger.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use xrpl_mithril_codec::signing::transaction_id;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000",
///     "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
///     "TxnSignature": "DEADBEEFCAFE"
/// });
/// let map = tx.as_object().expect("json object");
/// let id = transaction_id(map)?;
///
/// assert_eq!(id.len(), 32);
///
/// // The transaction ID is deterministic
/// let id2 = transaction_id(map)?;
/// assert_eq!(id, id2);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if serialization fails.
pub fn transaction_id(
    tx: &serde_json::Map<String, serde_json::Value>,
) -> Result<[u8; 32], CodecError> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&HASH_PREFIX_TRANSACTION_ID);
    serializer::serialize_json_object(tx, &mut buf, false)?;
    Ok(sha512_half(&buf))
}

/// Compute the transaction ID and return it as an uppercase hex string.
///
/// This is a convenience wrapper around [`transaction_id`].
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use xrpl_mithril_codec::signing::transaction_id_hex;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000",
///     "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
///     "TxnSignature": "DEADBEEFCAFE"
/// });
/// let map = tx.as_object().expect("json object");
/// let hex_id = transaction_id_hex(map)?;
///
/// // 32 bytes encoded as 64 uppercase hex characters
/// assert_eq!(hex_id.len(), 64);
/// assert!(hex_id.chars().all(|c| c.is_ascii_hexdigit()));
/// assert_eq!(hex_id, hex_id.to_uppercase());
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if serialization fails.
pub fn transaction_id_hex(
    tx: &serde_json::Map<String, serde_json::Value>,
) -> Result<String, CodecError> {
    let hash = transaction_id(tx)?;
    Ok(hex::encode_upper(hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hash_prefix_values() {
        // Verify the ASCII encoding of hash prefixes
        assert_eq!(&HASH_PREFIX_TRANSACTION_SIGN, b"STX\0");
        assert_eq!(&HASH_PREFIX_TRANSACTION_MULTI_SIGN, b"SMT\0");
        assert_eq!(&HASH_PREFIX_TRANSACTION_ID, b"TXN\0");
        assert_eq!(&HASH_PREFIX_INNER_NODE, b"SND\0");
        assert_eq!(&HASH_PREFIX_LEDGER, b"LWR\0");
    }

    #[test]
    fn sha512_half_known_vector() {
        // SHA-512 of empty string is known; verify truncation
        let hash = sha512_half(b"");
        // SHA-512("") starts with cf83e1357eefb8bd...
        assert_eq!(hash[0], 0xCF);
        assert_eq!(hash[1], 0x83);
        assert_eq!(hash[2], 0xE1);
        assert_eq!(hash[3], 0x35);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn signing_data_has_prefix() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        });
        let map = tx.as_object().unwrap();
        let data = signing_data(map).expect("signing_data");

        // First 4 bytes should be STX\0
        assert_eq!(&data[..4], b"STX\0");
    }

    #[test]
    fn signing_data_excludes_signature() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "TxnSignature": "DEADBEEF"
        });
        let map = tx.as_object().unwrap();
        let data = signing_data(map).expect("signing_data");

        // Both with-sig and without-sig should produce identical signing data
        // since TxnSignature is excluded when for_signing=true
        let tx_no_sig = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        });
        let data_no_sig = signing_data(tx_no_sig.as_object().unwrap()).expect("no_sig");

        // Both should produce the same signing data (TxnSignature excluded)
        assert_eq!(data, data_no_sig);
    }

    #[test]
    fn transaction_id_deterministic() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
            "TxnSignature": "DEADBEEFCAFE"
        });
        let map = tx.as_object().unwrap();

        let id1 = transaction_id(map).expect("tx_id");
        let id2 = transaction_id(map).expect("tx_id");

        assert_eq!(id1, id2, "transaction ID must be deterministic");
        assert_eq!(id1.len(), 32);
    }

    #[test]
    fn transaction_id_includes_signature() {
        // Transaction ID computation includes the signature (for_signing=false)
        let tx_with_sig = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "TxnSignature": "DEADBEEFCAFE"
        });
        let tx_without_sig = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        });

        let id_with = transaction_id(tx_with_sig.as_object().unwrap()).expect("with");
        let id_without = transaction_id(tx_without_sig.as_object().unwrap()).expect("without");

        assert_ne!(id_with, id_without, "signature must affect transaction ID");
    }

    #[test]
    fn transaction_id_hex_format() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        });
        let hex_id = transaction_id_hex(tx.as_object().unwrap()).expect("hex_id");

        assert_eq!(hex_id.len(), 64);
        assert!(hex_id.chars().all(|c| c.is_ascii_hexdigit()));
        // Should be uppercase
        assert_eq!(hex_id, hex_id.to_uppercase());
    }

    #[test]
    fn multi_signing_differs_by_signer() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        });
        let map = tx.as_object().unwrap();

        let signer_a = [0xAAu8; 20];
        let signer_b = [0xBBu8; 20];

        let hash_a = multi_signing_hash(map, &signer_a).expect("hash_a");
        let hash_b = multi_signing_hash(map, &signer_b).expect("hash_b");

        assert_ne!(hash_a, hash_b, "different signers must produce different hashes");
    }

    #[test]
    fn multi_signing_data_has_prefix_and_suffix() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        });
        let map = tx.as_object().unwrap();

        let signer = [0x42u8; 20];
        let data = multi_signing_data(map, &signer).expect("multi_data");

        // First 4 bytes: SMT\0
        assert_eq!(&data[..4], b"SMT\0");

        // Last 20 bytes: signer account ID
        assert_eq!(&data[data.len() - 20..], &signer);
    }
}

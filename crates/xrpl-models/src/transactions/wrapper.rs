//! Typed transaction wrappers that enforce the build-sign-submit workflow.
//!
//! These types implement ADR-004: separate `UnsignedTransaction` and
//! `TypedSignedTransaction` types so that submitting an unsigned transaction
//! is a compile error, and signing an already-signed transaction is impossible.
//!
//! # Workflow
//!
//! ```text
//! Transaction (typed) ──> UnsignedTransaction<T> ──> TypedSignedTransaction<T>
//!   (construct)              (autofill)                 (submit)
//! ```

use serde::Serialize;

use super::TransactionCommon;

/// A transaction type that can be signed.
///
/// Implemented for the [`Transaction`](super::Transaction) enum and
/// potentially for individual transaction structs.
pub trait Signable: Serialize + Clone {
    /// Returns a reference to the common transaction fields.
    fn common(&self) -> &TransactionCommon;

    /// Returns a mutable reference to the common fields (for autofill).
    fn common_mut(&mut self) -> &mut TransactionCommon;

    /// Returns the `TransactionType` name (e.g., `"Payment"`).
    fn transaction_type_name(&self) -> &str;
}

/// An unsigned transaction ready to be autofilled and signed.
///
/// Wraps a typed transaction and provides mutable access to common fields
/// for autofill. Convert to a JSON map for the codec/signer via
/// [`to_json_map`](Self::to_json_map).
///
/// # Examples
///
/// ```ignore
/// use xrpl_models::transactions::{Transaction, wrapper::UnsignedTransaction};
///
/// let tx = Transaction::Payment { common, fields };
/// let unsigned = UnsignedTransaction::new(tx);
/// // Autofill fee, sequence, etc.
/// // Then sign to produce a TypedSignedTransaction
/// ```
#[derive(Debug, Clone)]
pub struct UnsignedTransaction<T: Signable> {
    inner: T,
}

impl<T: Signable> UnsignedTransaction<T> {
    /// Creates a new unsigned transaction.
    pub fn new(tx: T) -> Self {
        Self { inner: tx }
    }

    /// Access the inner typed transaction.
    #[must_use]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Consume the wrapper and return the inner transaction.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Access the common transaction fields.
    #[must_use]
    pub fn common(&self) -> &TransactionCommon {
        self.inner.common()
    }

    /// Mutably access the common transaction fields (for autofill).
    pub fn common_mut(&mut self) -> &mut TransactionCommon {
        self.inner.common_mut()
    }

    /// Convert to a JSON map for use with the codec and signer.
    ///
    /// Serializes the transaction to JSON and strips any existing signature
    /// fields, ensuring the map is in the correct form for signing.
    ///
    /// # Errors
    ///
    /// Returns a JSON serialization error if the transaction cannot be serialized.
    pub fn to_json_map(
        &self,
    ) -> Result<serde_json::Map<String, serde_json::Value>, serde_json::Error> {
        let value = serde_json::to_value(&self.inner)?;
        let mut map = value
            .as_object()
            .cloned()
            .unwrap_or_default();
        // Strip any pre-existing signature fields
        map.remove("TxnSignature");
        map.remove("Signers");
        Ok(map)
    }
}

/// A signed transaction ready for submission to the network.
///
/// Contains both the original typed transaction and the signing artifacts
/// (JSON with signature fields, binary blob, transaction hash).
///
/// Created by the signing functions in `xrpl-tx`.
#[derive(Debug, Clone)]
pub struct TypedSignedTransaction<T: Signable> {
    /// The original transaction (without signature fields).
    inner: T,
    /// The complete signed JSON (includes SigningPubKey, TxnSignature).
    tx_json: serde_json::Map<String, serde_json::Value>,
    /// Hex-encoded binary blob ready for submission.
    tx_blob: String,
    /// Transaction hash/ID (uppercase hex, 64 chars).
    hash: String,
}

impl<T: Signable> TypedSignedTransaction<T> {
    /// Creates a new signed transaction from its components.
    ///
    /// This is an internal constructor — users should use the signing
    /// functions in `xrpl-tx` instead.
    pub fn new(
        inner: T,
        tx_json: serde_json::Map<String, serde_json::Value>,
        tx_blob: String,
        hash: String,
    ) -> Self {
        Self {
            inner,
            tx_json,
            tx_blob,
            hash,
        }
    }

    /// Access the inner typed transaction.
    #[must_use]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// The signed JSON representation (includes signature fields).
    #[must_use]
    pub fn tx_json(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.tx_json
    }

    /// Hex-encoded binary blob for submission via the `submit` RPC method.
    #[must_use]
    pub fn tx_blob(&self) -> &str {
        &self.tx_blob
    }

    /// Transaction hash/ID (uppercase hex, 64 characters).
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

// Implement Signable for the Transaction enum
impl Signable for super::Transaction {
    fn common(&self) -> &TransactionCommon {
        self.common()
    }

    fn common_mut(&mut self) -> &mut TransactionCommon {
        self.common_mut()
    }

    fn transaction_type_name(&self) -> &str {
        self.transaction_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::Transaction;
    use serde_json::json;

    #[test]
    fn unsigned_transaction_round_trip() {
        let tx_json = json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Fee": "12",
            "Sequence": 1,
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000"
        });
        let tx: Transaction = serde_json::from_value(tx_json).expect("deser");

        let unsigned = UnsignedTransaction::new(tx);
        assert_eq!(unsigned.common().sequence, 1);
        assert_eq!(unsigned.inner().transaction_type(), "Payment");

        let map = unsigned.to_json_map().expect("to_json_map");
        assert_eq!(
            map.get("TransactionType").and_then(|v| v.as_str()),
            Some("Payment")
        );
        assert!(!map.contains_key("TxnSignature"));
        assert!(!map.contains_key("Signers"));
    }

    #[test]
    fn common_mut_allows_modification() {
        let tx_json = json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Fee": "12",
            "Sequence": 1,
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000"
        });
        let tx: Transaction = serde_json::from_value(tx_json).expect("deser");
        let mut unsigned = UnsignedTransaction::new(tx);

        unsigned.common_mut().sequence = 42;
        unsigned.common_mut().last_ledger_sequence = Some(100);

        assert_eq!(unsigned.common().sequence, 42);
        assert_eq!(unsigned.common().last_ledger_sequence, Some(100));
    }
}

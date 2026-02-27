//! Transaction submission and lookup request types.

use serde::Serialize;
use xrpl_types::Hash256;

use super::{LedgerSpecifier, XrplRequest};
use crate::responses::transaction::{SubmitResponse, TransactionEntryResponse, TxResponse};

/// Submit a signed transaction blob to the network.
#[derive(Debug, Clone, Serialize)]
pub struct SubmitRequest {
    /// The hex-encoded signed transaction binary.
    pub tx_blob: String,
    /// If true, the server does not retry or relay the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_hard: Option<bool>,
}

impl XrplRequest for SubmitRequest {
    type Response = SubmitResponse;
    fn method(&self) -> &'static str {
        "submit"
    }
}

/// Submit a multi-signed transaction as JSON.
#[derive(Debug, Clone, Serialize)]
pub struct SubmitMultisignedRequest {
    /// The transaction JSON with the Signers array.
    pub tx_json: serde_json::Value,
    /// If true, the server does not retry or relay the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_hard: Option<bool>,
}

impl XrplRequest for SubmitMultisignedRequest {
    type Response = SubmitResponse;
    fn method(&self) -> &'static str {
        "submit_multisigned"
    }
}

/// Look up a transaction by its hash.
#[derive(Debug, Clone, Serialize)]
pub struct TxRequest {
    /// The transaction hash (hex).
    pub transaction: String,
    /// If true, return the transaction as a binary blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Minimum ledger sequence to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ledger: Option<u32>,
    /// Maximum ledger sequence to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ledger: Option<u32>,
}

impl XrplRequest for TxRequest {
    type Response = TxResponse;
    fn method(&self) -> &'static str {
        "tx"
    }
}

/// Look up a transaction in a specific ledger.
#[derive(Debug, Clone, Serialize)]
pub struct TransactionEntryRequest {
    /// The transaction hash.
    pub tx_hash: Hash256,
    /// Which ledger to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for TransactionEntryRequest {
    type Response = TransactionEntryResponse;
    fn method(&self) -> &'static str {
        "transaction_entry"
    }
}

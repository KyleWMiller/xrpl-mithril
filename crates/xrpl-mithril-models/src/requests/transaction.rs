//! Transaction submission and lookup request types.

use serde::Serialize;
use xrpl_mithril_types::Hash256;

use super::{LedgerSpecifier, XrplRequest};
use crate::responses::transaction::{SubmitResponse, TransactionEntryResponse, TxResponse};

/// Submit a signed transaction blob to the network.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::requests::SubmitRequest;
///
/// let request = SubmitRequest {
///     tx_blob: "1200002200000000240000000361D4838D7EA4C6800000000000000000000000000055534400000000004B4E9C06F24296074F7BC48F92A97916C6DC5EA968400000000000000A732103AC651208BDA639C37B9C8C561EA9CF5E3A4C5B88B50CB4C7F208E1B9F4B21643744630440220781B399C4C038E2E160866B1F89001AA0434AC88C44F2E83D25A4B6019B96B4E0220155DC1621DC1AABA0AA1C4AFEE4AE82D37B4F0B0E0C4F15A32F05C7FBC54B6C38114F36B40EBB5004A18AAAB1A6D03FC0A40C1F1AF8314B5F762798A53D543A014CAF8B297CFF8F2F937E8".to_string(),
///     fail_hard: Some(false),
/// };
///
/// let json = serde_json::to_value(&request).unwrap();
/// assert!(json["tx_blob"].as_str().unwrap().len() > 0);
/// ```
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
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::requests::SubmitMultisignedRequest;
///
/// let request = SubmitMultisignedRequest {
///     tx_json: serde_json::json!({"TransactionType": "Payment"}),
///     fail_hard: None,
/// };
/// ```
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
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::requests::TxRequest;
///
/// let request = TxRequest {
///     transaction: "E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7".to_string(),
///     binary: None,
///     min_ledger: None,
///     max_ledger: None,
/// };
///
/// let json = serde_json::to_value(&request).unwrap();
/// assert!(json["transaction"].as_str().unwrap().len() == 64);
/// ```
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
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::requests::TransactionEntryRequest;
/// use xrpl_mithril_types::Hash256;
///
/// let request = TransactionEntryRequest {
///     tx_hash: Hash256::from_hex("E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7").unwrap(),
///     ledger_index: Some(xrpl_mithril_models::requests::LedgerSpecifier::Index(12345)),
/// };
/// ```
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

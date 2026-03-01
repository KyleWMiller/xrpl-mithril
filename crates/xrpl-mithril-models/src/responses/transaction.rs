//! Transaction submission and lookup response types.

use serde::Deserialize;
use xrpl_mithril_types::Hash256;

/// Response from the `submit` and `submit_multisigned` methods.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::responses::transaction::SubmitResponse;
///
/// let json = serde_json::json!({
///     "engine_result": "tesSUCCESS",
///     "engine_result_code": 0,
///     "engine_result_message": "The transaction was applied."
/// });
///
/// let response: SubmitResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.engine_result, "tesSUCCESS");
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitResponse {
    /// The preliminary engine result code (e.g., "tesSUCCESS", "terQUEUED").
    pub engine_result: String,
    /// Numeric engine result code.
    pub engine_result_code: Option<i32>,
    /// Human-readable result message.
    pub engine_result_message: String,
    /// The transaction blob that was submitted.
    pub tx_blob: Option<String>,
    /// The transaction JSON.
    pub tx_json: Option<serde_json::Value>,
    /// The transaction hash.
    #[serde(default)]
    pub hash: Option<String>,
}

/// Response from the `tx` method.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::responses::transaction::TxResponse;
///
/// let json = serde_json::json!({
///     "hash": "E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7",
///     "ledger_index": 12345,
///     "validated": true
/// });
///
/// let response: TxResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.validated, Some(true));
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct TxResponse {
    /// The transaction hash.
    pub hash: Option<Hash256>,
    /// Ledger index where the transaction was included.
    pub ledger_index: Option<u32>,
    /// Whether this transaction was validated.
    pub validated: Option<bool>,
    /// Transaction metadata (contains result code, affected nodes, etc.).
    pub meta: Option<serde_json::Value>,
    /// The transaction data (when not using binary mode).
    #[serde(flatten)]
    pub tx_data: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `transaction_entry` method.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionEntryResponse {
    /// The transaction data.
    pub tx_json: serde_json::Value,
    /// Transaction metadata.
    pub metadata: serde_json::Value,
    /// Ledger index.
    pub ledger_index: u32,
    /// Ledger hash.
    pub ledger_hash: Option<Hash256>,
}

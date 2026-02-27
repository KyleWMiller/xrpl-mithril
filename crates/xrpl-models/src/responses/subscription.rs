//! Subscription response types.

use serde::Deserialize;

/// Response from the `subscribe` method.
///
/// The initial response may contain the current state of subscribed streams.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeResponse {
    /// Additional data depends on what was subscribed.
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `unsubscribe` method.
#[derive(Debug, Clone, Deserialize)]
pub struct UnsubscribeResponse {
    /// Additional fields (usually empty on success).
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

/// A message received from a subscription stream.
///
/// These are not direct RPC responses but asynchronous messages pushed by
/// the server on a WebSocket connection.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum SubscriptionMessage {
    /// A new ledger was validated.
    #[serde(rename = "ledgerClosed")]
    LedgerClosed(LedgerClosedMessage),
    /// A transaction was processed.
    #[serde(rename = "transaction")]
    Transaction(TransactionMessage),
    /// Server status update.
    #[serde(rename = "serverStatus")]
    ServerStatus(ServerStatusMessage),
}

/// Ledger closed subscription message.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerClosedMessage {
    /// Ledger sequence number.
    pub ledger_index: u32,
    /// Ledger hash.
    pub ledger_hash: String,
    /// Close time in Ripple epoch seconds.
    pub ledger_time: Option<u32>,
    /// Number of transactions in this ledger.
    pub txn_count: Option<u32>,
    /// Reserve base in drops.
    pub reserve_base: Option<u32>,
    /// Reserve increment in drops.
    pub reserve_inc: Option<u32>,
    /// Fee base.
    pub fee_base: Option<u32>,
    /// Fee reference.
    pub fee_ref: Option<u32>,
    /// Whether this is from a validated ledger.
    pub validated_ledgers: Option<String>,
}

/// Transaction subscription message.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionMessage {
    /// The transaction data.
    pub transaction: serde_json::Value,
    /// Transaction metadata (affected nodes, result code).
    pub meta: serde_json::Value,
    /// Whether this transaction was validated.
    pub validated: Option<bool>,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Engine result code.
    pub engine_result: Option<String>,
    /// Engine result code (numeric).
    pub engine_result_code: Option<i32>,
    /// Engine result message.
    pub engine_result_message: Option<String>,
}

/// Server status subscription message.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerStatusMessage {
    /// Server state.
    pub server_status: Option<String>,
    /// Load factor.
    pub load_factor: Option<f64>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

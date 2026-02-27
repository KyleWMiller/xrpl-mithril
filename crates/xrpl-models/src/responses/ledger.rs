//! Ledger information response types.

use serde::Deserialize;
use xrpl_types::Hash256;

use crate::requests::Marker;

/// Response from the `ledger` method.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerResponse {
    /// The ledger data.
    pub ledger: LedgerData,
    /// Ledger hash.
    pub ledger_hash: Option<Hash256>,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Queue data (if requested).
    pub queue_data: Option<Vec<serde_json::Value>>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// Ledger summary data.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerData {
    /// Ledger hash.
    pub ledger_hash: Option<String>,
    /// Ledger index/sequence.
    pub ledger_index: Option<String>,
    /// Whether this ledger is closed.
    pub closed: Option<bool>,
    /// The close time in Ripple epoch seconds.
    pub close_time: Option<u32>,
    /// Human-readable close time.
    pub close_time_human: Option<String>,
    /// Parent ledger hash.
    pub parent_hash: Option<String>,
    /// Total XRP in drops.
    pub total_coins: Option<String>,
    /// Transaction hash tree root.
    pub transaction_hash: Option<String>,
    /// Account state hash tree root.
    pub account_hash: Option<String>,
    /// Transactions in this ledger (if expanded).
    pub transactions: Option<Vec<serde_json::Value>>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `ledger_closed` method.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerClosedResponse {
    /// Hash of the most recently closed ledger.
    pub ledger_hash: Hash256,
    /// Sequence number of the most recently closed ledger.
    pub ledger_index: u32,
}

/// Response from the `ledger_current` method.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerCurrentResponse {
    /// Sequence number of the current in-progress ledger.
    pub ledger_current_index: u32,
}

/// Response from the `ledger_data` method.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerDataResponse {
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Ledger hash.
    pub ledger_hash: Option<Hash256>,
    /// Ledger entries.
    pub state: Vec<serde_json::Value>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// Response from the `ledger_entry` method.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerEntryResponse {
    /// The ledger entry data.
    pub node: Option<serde_json::Value>,
    /// Binary representation (if requested).
    pub node_binary: Option<String>,
    /// The ledger entry index.
    pub index: Option<Hash256>,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

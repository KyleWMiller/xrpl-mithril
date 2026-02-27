//! Ledger information request types.

use serde::Serialize;
use xrpl_types::Hash256;

use super::{LedgerSpecifier, Marker, XrplRequest};
use crate::responses::ledger::{
    LedgerClosedResponse, LedgerCurrentResponse, LedgerDataResponse, LedgerEntryResponse,
    LedgerResponse,
};

/// Request information about a ledger.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerRequest {
    /// Which ledger to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Ledger hash to look up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<Hash256>,
    /// If true, return full transaction data instead of hashes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<bool>,
    /// If true, expand transaction data as JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<bool>,
    /// If true, include the owner_funds field in offers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_funds: Option<bool>,
    /// If true, return binary data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// If true, include the ledger queue info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<bool>,
}

impl XrplRequest for LedgerRequest {
    type Response = LedgerResponse;
    fn method(&self) -> &'static str {
        "ledger"
    }
}

/// Request the sequence number of the most recently closed ledger.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerClosedRequest {}

impl XrplRequest for LedgerClosedRequest {
    type Response = LedgerClosedResponse;
    fn method(&self) -> &'static str {
        "ledger_closed"
    }
}

/// Request the sequence number of the current in-progress ledger.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerCurrentRequest {}

impl XrplRequest for LedgerCurrentRequest {
    type Response = LedgerCurrentResponse;
    fn method(&self) -> &'static str {
        "ledger_current"
    }
}

/// Request raw ledger entries from a ledger.
#[derive(Debug, Clone, Serialize)]
pub struct LedgerDataRequest {
    /// Which ledger to query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// If true, return binary data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for LedgerDataRequest {
    type Response = LedgerDataResponse;
    fn method(&self) -> &'static str {
        "ledger_data"
    }
}

/// Request a specific ledger entry by its ID or type-specific keys.
#[derive(Debug, Clone, Serialize)]
pub struct LedgerEntryRequest {
    /// The ledger entry index (hash) to look up directly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Look up an AccountRoot by account address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_root: Option<String>,
    /// Look up an Offer by owner + sequence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offer: Option<serde_json::Value>,
    /// Look up a RippleState (trust line).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ripple_state: Option<serde_json::Value>,
    /// Look up a Check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
    /// Look up an Escrow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escrow: Option<serde_json::Value>,
    /// Look up a PayChannel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_channel: Option<String>,
    /// Look up a DepositPreauth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_preauth: Option<serde_json::Value>,
    /// Look up a Ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<serde_json::Value>,
    /// Which ledger to query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// If true, return binary data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
}

impl XrplRequest for LedgerEntryRequest {
    type Response = LedgerEntryResponse;
    fn method(&self) -> &'static str {
        "ledger_entry"
    }
}

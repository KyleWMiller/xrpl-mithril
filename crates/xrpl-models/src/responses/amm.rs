//! AMM-related response types.

use serde::Deserialize;

/// Response from the `amm_info` method.
#[derive(Debug, Clone, Deserialize)]
pub struct AmmInfoResponse {
    /// The AMM data.
    pub amm: AmmData,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// AMM instance data.
#[derive(Debug, Clone, Deserialize)]
pub struct AmmData {
    /// The AMM account.
    pub account: Option<String>,
    /// Amount of the first asset in the pool.
    pub amount: serde_json::Value,
    /// Amount of the second asset in the pool.
    pub amount2: serde_json::Value,
    /// The LP token balance.
    pub lp_token: Option<serde_json::Value>,
    /// The current trading fee (in basis points, 0-1000).
    pub trading_fee: Option<u16>,
    /// Auction slot data.
    pub auction_slot: Option<serde_json::Value>,
    /// Vote slot data.
    pub vote_slots: Option<Vec<serde_json::Value>>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

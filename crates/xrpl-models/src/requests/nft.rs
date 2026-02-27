//! NFT-related request types.

use serde::Serialize;
use xrpl_types::Hash256;

use super::{LedgerSpecifier, Marker, XrplRequest};
use crate::responses::nft::{
    NftBuyOffersResponse, NftHistoryResponse, NftInfoResponse, NftSellOffersResponse,
};

/// Request information about a specific NFT.
#[derive(Debug, Clone, Serialize)]
pub struct NftInfoRequest {
    /// The NFToken ID to query.
    pub nft_id: Hash256,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for NftInfoRequest {
    type Response = NftInfoResponse;
    fn method(&self) -> &'static str {
        "nft_info"
    }
}

/// Request outstanding buy offers for an NFT.
#[derive(Debug, Clone, Serialize)]
pub struct NftBuyOffersRequest {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for NftBuyOffersRequest {
    type Response = NftBuyOffersResponse;
    fn method(&self) -> &'static str {
        "nft_buy_offers"
    }
}

/// Request outstanding sell offers for an NFT.
#[derive(Debug, Clone, Serialize)]
pub struct NftSellOffersRequest {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for NftSellOffersRequest {
    type Response = NftSellOffersResponse;
    fn method(&self) -> &'static str {
        "nft_sell_offers"
    }
}

/// Request the transaction history for an NFT.
#[derive(Debug, Clone, Serialize)]
pub struct NftHistoryRequest {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Minimum ledger index to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index_min: Option<i64>,
    /// Maximum ledger index to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index_max: Option<i64>,
    /// If true, return results as binary blobs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// If true, return oldest results first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward: Option<bool>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for NftHistoryRequest {
    type Response = NftHistoryResponse;
    fn method(&self) -> &'static str {
        "nft_history"
    }
}

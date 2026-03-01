//! NFT-related request types.

use serde::Serialize;
use xrpl_types::Hash256;

use super::{LedgerSpecifier, Marker, XrplRequest};
use crate::responses::nft::{
    NftBuyOffersResponse, NftHistoryResponse, NftInfoResponse, NftSellOffersResponse,
};

/// Request information about a specific NFT.
///
/// # Examples
///
/// ```
/// use xrpl_models::requests::NftInfoRequest;
/// use xrpl_types::Hash256;
///
/// let request = NftInfoRequest {
///     nft_id: Hash256::from_hex("000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65").unwrap(),
///     ledger_index: None,
/// };
/// ```
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
///
/// # Examples
///
/// ```
/// use xrpl_models::requests::NftBuyOffersRequest;
/// use xrpl_types::Hash256;
///
/// let request = NftBuyOffersRequest {
///     nft_id: Hash256::from_hex("000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65").unwrap(),
///     ledger_index: None,
///     limit: None,
///     marker: None,
/// };
/// ```
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
///
/// # Examples
///
/// ```
/// use xrpl_models::requests::NftSellOffersRequest;
/// use xrpl_types::Hash256;
///
/// let request = NftSellOffersRequest {
///     nft_id: Hash256::from_hex("000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65").unwrap(),
///     ledger_index: None,
///     limit: None,
///     marker: None,
/// };
/// ```
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
///
/// # Examples
///
/// ```
/// use xrpl_models::requests::NftHistoryRequest;
/// use xrpl_types::Hash256;
///
/// let request = NftHistoryRequest {
///     nft_id: Hash256::from_hex("000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65").unwrap(),
///     ledger_index: None,
///     ledger_index_min: None,
///     ledger_index_max: None,
///     binary: None,
///     forward: None,
///     limit: None,
///     marker: None,
/// };
/// ```
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

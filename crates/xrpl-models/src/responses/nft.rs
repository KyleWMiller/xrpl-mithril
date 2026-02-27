//! NFT-related response types.

use serde::Deserialize;
use xrpl_types::{AccountId, Amount, Hash256};

use crate::requests::Marker;

/// Response from the `nft_info` method.
#[derive(Debug, Clone, Deserialize)]
pub struct NftInfoResponse {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// The account that owns this NFT.
    pub owner: Option<AccountId>,
    /// Whether the NFT is burned.
    pub is_burned: Option<bool>,
    /// Flags.
    pub flags: Option<u32>,
    /// Transfer fee in basis points.
    pub transfer_fee: Option<u16>,
    /// The issuer.
    pub issuer: Option<AccountId>,
    /// The taxon.
    pub nft_taxon: Option<u32>,
    /// Serial number.
    pub nft_serial: Option<u32>,
    /// URI (hex-encoded).
    pub uri: Option<String>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// Response from the `nft_buy_offers` method.
#[derive(Debug, Clone, Deserialize)]
pub struct NftBuyOffersResponse {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Outstanding buy offers.
    pub offers: Vec<NftOffer>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// Response from the `nft_sell_offers` method.
#[derive(Debug, Clone, Deserialize)]
pub struct NftSellOffersResponse {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Outstanding sell offers.
    pub offers: Vec<NftOffer>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// An NFT offer entry.
#[derive(Debug, Clone, Deserialize)]
pub struct NftOffer {
    /// Amount offered/requested.
    pub amount: Amount,
    /// Flags.
    pub flags: u32,
    /// Offer index.
    pub nft_offer_index: String,
    /// Owner of the offer.
    pub owner: AccountId,
    /// Destination (if specified).
    pub destination: Option<AccountId>,
    /// Expiration.
    pub expiration: Option<u32>,
}

/// Response from the `nft_history` method.
#[derive(Debug, Clone, Deserialize)]
pub struct NftHistoryResponse {
    /// The NFToken ID.
    pub nft_id: Hash256,
    /// Transaction history.
    pub transactions: Vec<serde_json::Value>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

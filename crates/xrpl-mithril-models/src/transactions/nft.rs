//! NFToken transaction types.
//!
//! This module defines the transaction types for creating, managing, and trading
//! non-fungible tokens (NFTs) on the XRP Ledger:
//!
//! - [`NFTokenMint`] — Mint a new NFToken
//! - [`NFTokenBurn`] — Destroy an existing NFToken
//! - [`NFTokenCreateOffer`] — Create a buy or sell offer for an NFToken
//! - [`NFTokenAcceptOffer`] — Accept an existing buy or sell offer
//! - [`NFTokenCancelOffer`] — Cancel one or more outstanding offers
//! - [`NFTokenModify`] — Modify a mutable (DynamicNFT) token's metadata

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Blob, Hash256};

// ---------------------------------------------------------------------------
// NFTokenMint — TransactionType = 25
// ---------------------------------------------------------------------------

/// An NFTokenMint transaction (TransactionType = 25).
///
/// Mints a new NFToken and adds it to the `NFTokenPage` objects of the minting
/// account (or the `issuer` if specified). The token is assigned a unique
/// `NFTokenID` derived from the taxon, sequence, flags, and issuer.
///
/// The `transfer_fee` sets a royalty percentage (in basis points, 0-50000)
/// that the issuer collects on secondary sales when the `tfTransferable` flag
/// is set.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::nft::NFTokenMint;
///
/// let json = serde_json::json!({
///     "NFTokenTaxon": 0,
///     "TransferFee": 5000,
///     "URI": "68747470733A2F2F6578616D706C652E636F6D2F6E6674"
/// });
/// let mint: NFTokenMint = serde_json::from_value(json).unwrap();
/// assert_eq!(mint.nftoken_taxon, 0);
/// assert_eq!(mint.transfer_fee, Some(5000));
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/nftokenmint>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenMint {
    /// The taxon for this NFToken.
    ///
    /// An issuer may mint multiple tokens with the same taxon to create a
    /// logical collection. The taxon is scrambled in the final `NFTokenID` to
    /// prevent enumeration.
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,

    /// The issuer of the NFToken.
    ///
    /// If present, the transaction sender is minting on behalf of this issuer.
    /// The sender must be authorized (via `MintAccount` in the issuer's
    /// `AccountRoot`) to do so.
    #[serde(rename = "Issuer", default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<AccountId>,

    /// The transfer fee for secondary sales, in basis points (1/100th of a
    /// percent).
    ///
    /// Valid range: 0 to 50000 (0% to 50%). Only meaningful when the
    /// `tfTransferable` flag (0x00000008) is set.
    #[serde(rename = "TransferFee", default, skip_serializing_if = "Option::is_none")]
    pub transfer_fee: Option<u16>,

    /// An arbitrary URI pointing to metadata or content for this NFToken.
    ///
    /// Must not exceed 256 bytes. Conventionally a link to an IPFS CID or
    /// HTTPS URL containing the token's metadata JSON.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,
}

// ---------------------------------------------------------------------------
// NFTokenBurn — TransactionType = 26
// ---------------------------------------------------------------------------

/// An NFTokenBurn transaction (TransactionType = 26).
///
/// Permanently destroys an NFToken, removing it from the ledger. The token
/// must be owned by the sender, or the sender must be the token's issuer
/// (if the `lsfBurnable` flag was set at mint time).
///
/// If an `owner` is specified, the sender (who must be the issuer) is burning
/// a token held in another account's NFToken pages.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/nftokenburn>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenBurn {
    /// The ID of the NFToken to burn.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Hash256,

    /// The account that owns the NFToken to burn.
    ///
    /// Required when the issuer is burning a token held by another account
    /// (only possible if `lsfBurnable` was set at mint time).
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// NFTokenCreateOffer — TransactionType = 27
// ---------------------------------------------------------------------------

/// An NFTokenCreateOffer transaction (TransactionType = 27).
///
/// Creates either a buy offer or a sell offer for an NFToken:
///
/// - **Sell offer** (`tfSellNFToken` / 0x00000001): The sender owns the token
///   and is willing to sell it for at least `amount`.
/// - **Buy offer** (no sell flag): The sender wants to buy the token identified
///   by `nftoken_id` and is willing to pay up to `amount`.
///
/// An optional `destination` restricts who can accept the offer. An optional
/// `expiration` sets a time limit.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/nftokencreateoffer>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenCreateOffer {
    /// The ID of the NFToken this offer is for.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Hash256,

    /// The amount the offer creator is willing to pay or accept.
    ///
    /// For a sell offer, this is the minimum acceptable price. For a buy
    /// offer, this is the maximum price the buyer is willing to pay.
    /// Must be zero for a sell offer of a free transfer.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The owner of the NFToken (required for buy offers).
    ///
    /// For buy offers, identifies the current owner. Not used for sell offers
    /// since the sender must own the token.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,

    /// An account that is the only allowed acceptor of this offer.
    ///
    /// If set, no other account can accept the offer.
    #[serde(rename = "Destination", default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<AccountId>,

    /// Time after which the offer is no longer valid, in seconds since the
    /// Ripple Epoch (2000-01-01T00:00:00Z).
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
}

// ---------------------------------------------------------------------------
// NFTokenAcceptOffer — TransactionType = 29
// ---------------------------------------------------------------------------

/// An NFTokenAcceptOffer transaction (TransactionType = 29).
///
/// Accepts an existing buy or sell offer for an NFToken. This transaction can
/// operate in three modes:
///
/// 1. **Accept a sell offer**: Provide `nftoken_sell_offer` only. The sender
///    pays the sell price and receives the token.
/// 2. **Accept a buy offer**: Provide `nftoken_buy_offer` only. The sender
///    (who must own the token) receives the buy price.
/// 3. **Brokered mode**: Provide both `nftoken_sell_offer` and
///    `nftoken_buy_offer`. The broker matches a buyer and seller, optionally
///    taking a fee via `nftoken_broker_fee`. The buy amount must be >= the
///    sell amount plus the broker fee.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/nftokenacceptoffer>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenAcceptOffer {
    /// The ID of an existing sell offer (`NFTokenOffer` ledger object) to
    /// accept.
    #[serde(
        rename = "NFTokenSellOffer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nftoken_sell_offer: Option<Hash256>,

    /// The ID of an existing buy offer (`NFTokenOffer` ledger object) to
    /// accept.
    #[serde(
        rename = "NFTokenBuyOffer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nftoken_buy_offer: Option<Hash256>,

    /// The fee the broker keeps from the brokered sale.
    ///
    /// Only valid in brokered mode (both sell and buy offers provided). Must
    /// be positive and less than or equal to the difference between the buy
    /// and sell amounts.
    #[serde(
        rename = "NFTokenBrokerFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nftoken_broker_fee: Option<Amount>,
}

// ---------------------------------------------------------------------------
// NFTokenCancelOffer — TransactionType = 28
// ---------------------------------------------------------------------------

/// An NFTokenCancelOffer transaction (TransactionType = 28).
///
/// Cancels one or more existing NFToken offers. The sender must be either:
///
/// - The creator of the offer, OR
/// - The account specified as the `Destination` of the offer, OR
/// - Any account, if the offer has expired.
///
/// The `nftoken_offers` array may contain up to 32 offer IDs per transaction.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/nftokencanceloffer>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenCancelOffer {
    /// Array of `NFTokenOffer` ledger object IDs to cancel.
    ///
    /// Must contain at least one and at most 32 entries. The IDs are the
    /// hashes of the `NFTokenOffer` ledger entries created by
    /// [`NFTokenCreateOffer`] transactions.
    #[serde(rename = "NFTokenOffers")]
    pub nftoken_offers: Vec<Hash256>,
}

// ---------------------------------------------------------------------------
// NFTokenModify — TransactionType = 61
// ---------------------------------------------------------------------------

/// An NFTokenModify transaction (TransactionType = 61).
///
/// Modifies the metadata of a mutable NFToken (DynamicNFT). Only tokens
/// minted with the `lsfMutable` flag can be modified. The transaction must be
/// submitted by the token's issuer.
///
/// Requires the **DynamicNFT** amendment (activated June 2025).
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/nftokenmodify>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenModify {
    /// The ID of the NFToken to modify.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Hash256,

    /// The account that owns the NFToken, if different from the issuer.
    ///
    /// Required when the token is held by an account other than the issuer.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,

    /// The new URI for this NFToken's metadata.
    ///
    /// Replaces the existing URI. Must not exceed 256 bytes. Pass an empty
    /// blob to clear the URI.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,
}

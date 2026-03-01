//! DEX offer transaction types.
//!
//! This module defines the [`OfferCreate`] and [`OfferCancel`] transactions
//! for interacting with the XRPL's built-in decentralized exchange.

use serde::{Deserialize, Serialize};
use xrpl_types::Amount;

// ---------------------------------------------------------------------------
// OfferCreate — TransactionType = 7
// ---------------------------------------------------------------------------

/// An OfferCreate transaction (TransactionType = 7).
///
/// Places an offer on the XRPL decentralized exchange. The offer specifies an
/// exchange between two assets: the account is willing to pay `taker_gets` in
/// order to receive `taker_pays`.
///
/// If a matching offer already exists on the order book, the new offer will
/// consume it (partial or full fill). Any remainder becomes a passive offer
/// on the book unless `tfImmediateOrCancel` or `tfFillOrKill` flags are set.
///
/// If `offer_sequence` is provided, the transaction also cancels the existing
/// offer with that sequence number from the same account before placing the
/// new one.
///
/// # Examples
///
/// ```
/// use xrpl_models::transactions::offer::OfferCreate;
///
/// let json = serde_json::json!({
///     "TakerPays": "5000000",
///     "TakerGets": "1000000"
/// });
/// let offer: OfferCreate = serde_json::from_value(json).unwrap();
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/offercreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferCreate {
    /// The amount and type of currency the offer taker must pay.
    ///
    /// From the perspective of the offer creator, this is what they want to
    /// receive in exchange for `taker_gets`.
    #[serde(rename = "TakerPays")]
    pub taker_pays: Amount,

    /// The amount and type of currency the offer taker receives.
    ///
    /// From the perspective of the offer creator, this is what they are
    /// willing to give up in exchange for `taker_pays`.
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,

    /// Time after which the offer is no longer active, in seconds since the
    /// Ripple Epoch (2000-01-01T00:00:00Z).
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    /// The sequence number of a previous offer to cancel when this one is
    /// placed. This allows atomic replacement of an existing offer.
    #[serde(rename = "OfferSequence", default, skip_serializing_if = "Option::is_none")]
    pub offer_sequence: Option<u32>,
}

// ---------------------------------------------------------------------------
// OfferCancel — TransactionType = 8
// ---------------------------------------------------------------------------

/// An OfferCancel transaction (TransactionType = 8).
///
/// Removes an existing offer from the XRPL decentralized exchange. The offer
/// is identified by the sequence number of the [`OfferCreate`] transaction
/// that created it. Only the account that created the offer can cancel it.
///
/// # Examples
///
/// ```
/// use xrpl_models::transactions::offer::OfferCancel;
///
/// let json = serde_json::json!({
///     "OfferSequence": 7
/// });
/// let cancel: OfferCancel = serde_json::from_value(json).unwrap();
/// assert_eq!(cancel.offer_sequence, 7);
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/offercancel>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferCancel {
    /// The sequence number of the [`OfferCreate`] transaction that created
    /// the offer to cancel.
    #[serde(rename = "OfferSequence")]
    pub offer_sequence: u32,
}

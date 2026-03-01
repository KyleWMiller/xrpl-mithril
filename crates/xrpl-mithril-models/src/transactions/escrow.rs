//! Escrow transaction types.
//!
//! Escrows hold XRP, issued tokens, or MPTs until conditions are met.
//! An escrow locks funds and releases them when a crypto-condition is
//! fulfilled, a time threshold passes, or both. If neither condition is met
//! before the optional cancel-after time, anyone can cancel the escrow and
//! return the funds to the creator.

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Blob};

// ---------------------------------------------------------------------------
// EscrowCreate — TransactionType = 1
// ---------------------------------------------------------------------------

/// An EscrowCreate transaction (TransactionType = 1).
///
/// Sequesters XRP, an issued-currency amount, or an MPT amount in an escrow
/// ledger entry. The funds are held until the escrow is finished
/// ([`EscrowFinish`]) or cancelled ([`EscrowCancel`]).
///
/// At least one of `finish_after` or `condition` must be provided so the
/// escrow can eventually be finished. If `cancel_after` is provided, it must
/// be after `finish_after`.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::escrow::EscrowCreate;
///
/// let json = serde_json::json!({
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "10000000",
///     "FinishAfter": 533257958,
///     "CancelAfter": 533344358
/// });
/// let escrow: EscrowCreate = serde_json::from_value(json).unwrap();
/// assert_eq!(escrow.finish_after, Some(533257958));
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/escrowcreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscrowCreate {
    /// The address of the account that will receive the escrowed funds when
    /// the escrow is finished.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The amount to escrow.
    ///
    /// This can be an XRP amount (in drops), an issued-currency amount, or an
    /// MPT amount. The full value is debited from the sender's balance when the
    /// EscrowCreate transaction is processed.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The time, in seconds since the Ripple Epoch, after which the escrow
    /// can be finished.
    ///
    /// The escrow cannot be finished before this time. If omitted, the escrow
    /// can be finished at any time provided the `condition` (if any) is
    /// fulfilled.
    #[serde(rename = "FinishAfter", default, skip_serializing_if = "Option::is_none")]
    pub finish_after: Option<u32>,

    /// The time, in seconds since the Ripple Epoch, after which the escrow
    /// can be cancelled.
    ///
    /// If this time passes without the escrow being finished, anyone may
    /// submit an [`EscrowCancel`] transaction to return the funds to the
    /// creator. Must be after `finish_after` if both are specified.
    #[serde(rename = "CancelAfter", default, skip_serializing_if = "Option::is_none")]
    pub cancel_after: Option<u32>,

    /// A PREIMAGE-SHA-256 crypto-condition that must be fulfilled to finish
    /// the escrow.
    ///
    /// Encoded as a DER-encoded condition per the
    /// [Crypto-Conditions spec (RFC draft)](https://tools.ietf.org/html/draft-thomas-crypto-conditions-04).
    /// If omitted, the escrow can be finished solely based on `finish_after`.
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Blob>,

    /// Arbitrary tag that identifies the reason for the escrow to the
    /// destination, or a hosted recipient to pay.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
}

// ---------------------------------------------------------------------------
// EscrowFinish — TransactionType = 2
// ---------------------------------------------------------------------------

/// An EscrowFinish transaction (TransactionType = 2).
///
/// Delivers the escrowed funds to the destination. If the escrow has a
/// crypto-condition, the matching `fulfillment` must be provided. If the
/// escrow has a `finish_after` time, this transaction must be submitted
/// after that time.
///
/// Anyone may submit an EscrowFinish, not just the escrow creator or
/// destination, as long as the conditions are met.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::escrow::EscrowFinish;
///
/// let json = serde_json::json!({
///     "Owner": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "OfferSequence": 7
/// });
/// let finish: EscrowFinish = serde_json::from_value(json).unwrap();
/// assert_eq!(finish.offer_sequence, 7);
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/escrowfinish>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscrowFinish {
    /// The address of the account that created the escrow.
    #[serde(rename = "Owner")]
    pub owner: AccountId,

    /// The sequence number of the [`EscrowCreate`] transaction that created
    /// the escrow to finish.
    #[serde(rename = "OfferSequence")]
    pub offer_sequence: u32,

    /// The crypto-condition from the original [`EscrowCreate`].
    ///
    /// Must match the condition stored in the escrow ledger object. Required
    /// if the escrow was created with a condition.
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Blob>,

    /// The PREIMAGE-SHA-256 fulfillment that satisfies the escrow's
    /// crypto-condition.
    ///
    /// Must be the preimage whose SHA-256 hash matches the condition.
    /// Required if `condition` is provided.
    #[serde(rename = "Fulfillment", default, skip_serializing_if = "Option::is_none")]
    pub fulfillment: Option<Blob>,
}

// ---------------------------------------------------------------------------
// EscrowCancel — TransactionType = 4
// ---------------------------------------------------------------------------

/// An EscrowCancel transaction (TransactionType = 4).
///
/// Returns escrowed funds to the original sender. This is only valid if the
/// escrow's `cancel_after` time has passed. Anyone may submit an
/// EscrowCancel once the cancellation time has elapsed.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::escrow::EscrowCancel;
///
/// let json = serde_json::json!({
///     "Owner": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "OfferSequence": 7
/// });
/// let cancel: EscrowCancel = serde_json::from_value(json).unwrap();
/// assert_eq!(cancel.offer_sequence, 7);
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/escrowcancel>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscrowCancel {
    /// The address of the account that created the escrow.
    #[serde(rename = "Owner")]
    pub owner: AccountId,

    /// The sequence number of the [`EscrowCreate`] transaction that created
    /// the escrow to cancel.
    #[serde(rename = "OfferSequence")]
    pub offer_sequence: u32,
}

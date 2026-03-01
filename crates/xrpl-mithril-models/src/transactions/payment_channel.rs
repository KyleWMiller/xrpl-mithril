//! Payment channel transaction types.
//!
//! This module defines the payment channel lifecycle transactions:
//! [`PaymentChannelCreate`], [`PaymentChannelFund`], and
//! [`PaymentChannelClaim`].
//!
//! Payment channels enable fast, off-ledger XRP payments between two parties.
//! The source opens a channel, the destination claims from it, and either
//! party can close it.

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Blob, Hash256};

// ---------------------------------------------------------------------------
// PaymentChannelCreate — TransactionType = 13
// ---------------------------------------------------------------------------

/// A PaymentChannelCreate transaction (TransactionType = 13).
///
/// Creates a unidirectional payment channel from the sender to the
/// `destination`. The channel is funded with `amount` XRP and uses
/// `public_key` for claim verification.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelcreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentChannelCreate {
    /// The account that may claim XRP from this channel.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The total amount of XRP, in drops, to set aside in this channel.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The number of seconds the source must wait before closing the channel
    /// if it has unclaimed XRP.
    #[serde(rename = "SettleDelay")]
    pub settle_delay: u32,

    /// The 33-byte public key (in hexadecimal) used to verify claim
    /// signatures for this channel. This must match the key pair used by
    /// the destination to sign claims.
    #[serde(rename = "PublicKey")]
    pub public_key: Blob,

    /// The time, in seconds since the Ripple Epoch, when this channel expires.
    /// Any transaction that would set the channel's expiration to a time after
    /// this value is rejected. If omitted, the channel has no fixed expiration.
    #[serde(rename = "CancelAfter", default, skip_serializing_if = "Option::is_none")]
    pub cancel_after: Option<u32>,

    /// Arbitrary tag that identifies the reason for the channel to the
    /// destination, or a hosted recipient to pay.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
}

// ---------------------------------------------------------------------------
// PaymentChannelFund — TransactionType = 14
// ---------------------------------------------------------------------------

/// A PaymentChannelFund transaction (TransactionType = 14).
///
/// Adds additional XRP to an open payment channel, and optionally sets a new
/// `expiration` time. Only the source address of the channel can use this
/// transaction.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelfund>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentChannelFund {
    /// The unique ID of the payment channel to fund, as a 256-bit hex string.
    #[serde(rename = "Channel")]
    pub channel: Hash256,

    /// The amount of XRP, in drops, to add to the channel.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// New expiration time for this channel, in seconds since the Ripple
    /// Epoch. This replaces any previously set expiration. The channel is
    /// closed if a new validated ledger's close time is after this value.
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
}

// ---------------------------------------------------------------------------
// PaymentChannelClaim — TransactionType = 15
// ---------------------------------------------------------------------------

/// A PaymentChannelClaim transaction (TransactionType = 15).
///
/// Claims XRP from a payment channel, adjusts the channel's expiration, or
/// both. Can be submitted by the source or destination of the channel.
///
/// The `signature` and `public_key` fields are required when claiming XRP
/// (i.e., when `balance` is provided). The signature authorizes updating the
/// channel balance.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelclaim>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentChannelClaim {
    /// The unique ID of the payment channel.
    #[serde(rename = "Channel")]
    pub channel: Hash256,

    /// Total amount of XRP, in drops, delivered by this channel after
    /// processing this claim. Required to deliver XRP. Must be greater than
    /// the amount already delivered.
    #[serde(rename = "Balance", default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<Amount>,

    /// The amount of XRP, in drops, authorized by the `signature`. This must
    /// match the amount in the claim signature.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,

    /// The signature of the claim, in hexadecimal. This is signed by the
    /// key pair associated with the channel's `PublicKey`.
    #[serde(rename = "Signature", default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Blob>,

    /// The public key used to verify the claim `signature`, in hexadecimal.
    /// Must match the `PublicKey` stored in the payment channel ledger object.
    #[serde(rename = "PublicKey", default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<Blob>,
}

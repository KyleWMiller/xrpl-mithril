//! The PayChannel ledger entry type.
//!
//! A [`PayChannel`] object represents a payment channel that holds XRP
//! for asynchronous off-ledger payments.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/paychannel>

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Amount, Blob, Hash256};

/// A PayChannel ledger entry.
///
/// Represents an open payment channel for off-ledger XRP transfers.
/// The sender deposits XRP, and the receiver can claim verified amounts
/// without on-ledger transactions for each individual payment.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/paychannel>
///
/// # Examples
///
/// ```
/// use xrpl_models::ledger::PayChannel;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "PayChannel",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "10000000",
///     "Balance": "5000000",
///     "SettleDelay": 86400,
///     "PublicKey": "0330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020",
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 50,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: PayChannel = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.settle_delay, 86400);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayChannel {
    /// The ledger entry type identifier. Always `"PayChannel"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The source address that owns this payment channel.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The destination address for this payment channel. Only this
    /// account can receive XRP from the channel.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// Total XRP, in drops, that has been allocated to this channel.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// Total XRP, in drops, already paid out by the channel. The
    /// difference between `amount` and `balance` is the XRP that can
    /// still be claimed.
    #[serde(rename = "Balance")]
    pub balance: Amount,

    /// Number of seconds the source address must wait before closing
    /// the channel if it has unclaimed XRP.
    #[serde(rename = "SettleDelay")]
    pub settle_delay: u32,

    /// The public key of the key pair the source uses to sign claims
    /// against this channel.
    #[serde(rename = "PublicKey")]
    pub public_key: Blob,

    /// The mutable expiration time for this payment channel, in seconds
    /// since the Ripple Epoch.
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    /// The immutable expiration time for this payment channel, in seconds
    /// since the Ripple Epoch.
    #[serde(rename = "CancelAfter", default, skip_serializing_if = "Option::is_none")]
    pub cancel_after: Option<u32>,

    /// An arbitrary tag to further specify the source for this payment channel.
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,

    /// An arbitrary tag to further specify the destination for this payment channel.
    #[serde(rename = "DestinationTag", default, skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,

    /// A hint indicating which page of the source account's owner
    /// directory links to this object.
    #[serde(rename = "OwnerNode", default, skip_serializing_if = "Option::is_none")]
    pub owner_node: Option<String>,

    /// The identifying hash of the transaction that most recently modified
    /// this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,

    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,

    /// The unique ID (hash) of this ledger entry.
    #[serde(rename = "index")]
    pub index: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_pay_channel() {
        let json = json!({
            "LedgerEntryType": "PayChannel",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "10000000",
            "Balance": "5000000",
            "SettleDelay": 86400,
            "PublicKey": "0330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020",
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 50,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: PayChannel = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "PayChannel");
        assert_eq!(entry.settle_delay, 86400);
    }
}

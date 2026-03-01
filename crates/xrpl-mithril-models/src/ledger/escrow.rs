//! The Escrow ledger entry type.
//!
//! An [`Escrow`] object represents a held payment of XRP, issued currency,
//! or MPT that is waiting for conditions to be met before it can be finished
//! or cancelled.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/escrow>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Blob, Hash256};

/// An Escrow ledger entry.
///
/// Represents held funds that can only be released when specific conditions
/// are met (time-based, crypto-condition, or both).
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/escrow>
///
/// # Examples
///
/// Deserialize a time-based escrow from JSON:
///
/// ```
/// use xrpl_mithril_models::ledger::Escrow;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "Escrow",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000",
///     "FinishAfter": 700000000,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 10,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: Escrow = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.finish_after, Some(700000000));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Escrow {
    /// The ledger entry type identifier. Always `"Escrow"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The address of the owner (sender) of this held payment.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The destination address where the held payment will be sent.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The amount of currency to be delivered by the held payment.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// A PREIMAGE-SHA-256 crypto-condition in hexadecimal. The escrow
    /// can only be finished if this condition is fulfilled.
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Blob>,

    /// The held payment can be finished after this time. In seconds
    /// since the Ripple Epoch.
    #[serde(rename = "FinishAfter", default, skip_serializing_if = "Option::is_none")]
    pub finish_after: Option<u32>,

    /// The held payment can be cancelled after this time. In seconds
    /// since the Ripple Epoch.
    #[serde(rename = "CancelAfter", default, skip_serializing_if = "Option::is_none")]
    pub cancel_after: Option<u32>,

    /// An arbitrary tag to further specify the source for this held payment.
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,

    /// An arbitrary tag to further specify the destination for this held payment.
    #[serde(rename = "DestinationTag", default, skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,

    /// A hint indicating which page of the sender's owner directory links
    /// to this object.
    #[serde(rename = "OwnerNode", default, skip_serializing_if = "Option::is_none")]
    pub owner_node: Option<String>,

    /// A hint indicating which page of the destination's owner directory
    /// links to this object.
    #[serde(rename = "DestinationNode", default, skip_serializing_if = "Option::is_none")]
    pub destination_node: Option<String>,

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
    fn deserialize_escrow() {
        let json = json!({
            "LedgerEntryType": "Escrow",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "FinishAfter": 700000000,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 10,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Escrow = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "Escrow");
        assert_eq!(entry.finish_after, Some(700000000));
    }
}

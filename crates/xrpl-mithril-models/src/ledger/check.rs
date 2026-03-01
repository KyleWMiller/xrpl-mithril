//! The Check ledger entry type.
//!
//! A [`Check`] object represents a check that can be cashed by its
//! destination for a specified maximum amount.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/check>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Hash256};

/// A Check ledger entry.
///
/// Represents a deferred payment (like a paper check) that the destination
/// can cash for up to the specified `send_max` amount.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/check>
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::ledger::Check;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "Check",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "SendMax": "1000000",
///     "Sequence": 5,
///     "Flags": 0,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 20,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: Check = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.sequence, 5);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    /// The ledger entry type identifier. Always `"Check"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The address of the account that created the check.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The intended recipient of the check.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The maximum amount of currency the check can be cashed for.
    #[serde(rename = "SendMax")]
    pub send_max: Amount,

    /// The sequence number of the CheckCreate transaction that created
    /// this check.
    #[serde(rename = "Sequence")]
    pub sequence: u32,

    /// A bit-map of boolean flags for this check.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// An arbitrary tag to further specify the destination for this check.
    #[serde(rename = "DestinationTag", default, skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,

    /// An arbitrary tag to further specify the source for this check.
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,

    /// The time after which this check is no longer valid, in seconds
    /// since the Ripple Epoch.
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    /// Arbitrary 256-bit hash provided by the creator as a specific
    /// reason or identifier for this check.
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,

    /// A hint indicating which page of the sender's owner directory
    /// links to this object.
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
    fn deserialize_check() {
        let json = json!({
            "LedgerEntryType": "Check",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "SendMax": "1000000",
            "Sequence": 5,
            "Flags": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 20,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Check = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "Check");
        assert_eq!(entry.sequence, 5);
    }
}

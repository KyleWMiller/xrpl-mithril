//! The DID ledger entry type.
//!
//! A [`Did`] object represents a Decentralized Identifier (DID) associated
//! with an account on the XRP Ledger.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/did>

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Blob, Hash256};

/// A DID ledger entry.
///
/// Holds the Decentralized Identifier data for an account, including
/// the DID document, a URI reference, and arbitrary additional data.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/did>
///
/// # Examples
///
/// ```
/// use xrpl_models::ledger::Did;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "DID",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Flags": 0,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 10,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: Did = serde_json::from_value(json).unwrap();
/// assert!(entry.did_document.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Did {
    /// The ledger entry type identifier. Always `"DID"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The account that controls this DID.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The W3C DID document associated with this DID.
    #[serde(rename = "DIDDocument", default, skip_serializing_if = "Option::is_none")]
    pub did_document: Option<Blob>,

    /// A URI that points to additional DID-related data.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,

    /// Arbitrary additional data associated with this DID.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Blob>,

    /// A bit-map of boolean flags for this DID.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// A hint indicating which page of the owner's directory links to this
    /// object.
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
    fn deserialize_did() {
        let json = json!({
            "LedgerEntryType": "DID",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Flags": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 10,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Did = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "DID");
        assert_eq!(entry.flags, 0);
        assert!(entry.did_document.is_none());
    }
}

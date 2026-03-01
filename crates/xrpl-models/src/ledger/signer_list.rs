//! The SignerList ledger entry type.
//!
//! A [`SignerList`] object represents a list of parties that, as a group,
//! are authorized to sign a transaction in place of an individual account.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/signerlist>

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Hash256};

/// A single entry in a [`SignerList`], representing one authorized signer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerEntry {
    /// The address of an account whose signature contributes to the
    /// multi-signature.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The weight of a signature from this signer. A multi-signature is
    /// only valid if the sum of weights of the signatures provided meets
    /// or exceeds the signer list's `SignerQuorum` value.
    #[serde(rename = "SignerWeight")]
    pub signer_weight: u16,

    /// An optional 256-bit hash that can be used to identify this signer
    /// in an external system.
    #[serde(rename = "WalletLocator", default, skip_serializing_if = "Option::is_none")]
    pub wallet_locator: Option<Hash256>,
}

impl crate::serde_helpers::StArrayElement for SignerEntry {
    const WRAPPER_KEY: &'static str = "SignerEntry";
}

/// A SignerList ledger entry.
///
/// Defines the multi-signing configuration for an account: which accounts
/// can sign and how much weight each signature carries.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/signerlist>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerList {
    /// The ledger entry type identifier. Always `"SignerList"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// A bit-map of boolean flags for this signer list.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// A hint indicating which page of the owner directory links to this
    /// object.
    #[serde(rename = "OwnerNode", default, skip_serializing_if = "Option::is_none")]
    pub owner_node: Option<String>,

    /// A target number for signer weights. To produce a valid signature
    /// for this account, the total weight of the signatures provided must
    /// meet or exceed this value.
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,

    /// The list of authorized signers and their weights.
    #[serde(rename = "SignerEntries")]
    pub signer_entries: crate::serde_helpers::StArray<SignerEntry>,

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
    fn deserialize_signer_list() {
        let json = json!({
            "LedgerEntryType": "SignerList",
            "Flags": 0,
            "SignerQuorum": 3,
            "SignerEntries": [
                {
                    "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                    "SignerWeight": 2
                },
                {
                    "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
                    "SignerWeight": 1
                }
            ],
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 10,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: SignerList = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "SignerList");
        assert_eq!(entry.signer_quorum, 3);
        assert_eq!(entry.signer_entries.len(), 2);
        assert_eq!(entry.signer_entries[0].signer_weight, 2);
    }
}

//! The DepositPreauth ledger entry type.
//!
//! A [`DepositPreauth`] object tracks a preauthorization from one account
//! to another. The preauthorized account can send payments directly to the
//! authorizing account even if the authorizing account has DepositAuth enabled.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/depositpreauth>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Hash256};

/// A DepositPreauth ledger entry.
///
/// Grants preauthorization for one account to send payments to another,
/// bypassing deposit authorization requirements.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/depositpreauth>
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::ledger::DepositPreauth;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "DepositPreauth",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Authorize": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Flags": 0,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 10,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: DepositPreauth = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.flags, 0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositPreauth {
    /// The ledger entry type identifier. Always `"DepositPreauth"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The account that granted the preauthorization (the account that
    /// has DepositAuth enabled).
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The account that is preauthorized to deliver payments.
    #[serde(rename = "Authorize")]
    pub authorize: AccountId,

    /// A bit-map of boolean flags for this entry.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// A hint indicating which page of the owner's directory links to
    /// this object.
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
    fn deserialize_deposit_preauth() {
        let json = json!({
            "LedgerEntryType": "DepositPreauth",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Authorize": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Flags": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 10,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: DepositPreauth = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "DepositPreauth");
        assert_eq!(entry.flags, 0);
    }
}

//! The DirectoryNode ledger entry type.
//!
//! A [`DirectoryNode`] object links to other objects in the ledger.
//! There are two types of directories:
//! - Owner directories list objects owned by an account (offers, trust lines, etc.)
//! - Offer directories list offers available in the DEX at a specific exchange rate
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/directorynode>

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Hash160, Hash256};

/// A DirectoryNode ledger entry.
///
/// Provides a linked-list structure for organizing other ledger entries.
/// Owner directories track objects belonging to an account. Offer
/// directories organize DEX offers by exchange rate.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/directorynode>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryNode {
    /// The ledger entry type identifier. Always `"DirectoryNode"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// A bit-map of boolean flags for this directory.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// The ID of root object for this directory.
    #[serde(rename = "RootIndex")]
    pub root_index: Hash256,

    /// The contents of this directory: an array of IDs of other objects.
    #[serde(rename = "Indexes")]
    pub indexes: Vec<Hash256>,

    /// If this directory consists of multiple pages, this links to the
    /// next page in sequence.
    #[serde(rename = "IndexNext", default, skip_serializing_if = "Option::is_none")]
    pub index_next: Option<String>,

    /// If this directory consists of multiple pages, this links to the
    /// previous page in sequence.
    #[serde(rename = "IndexPrevious", default, skip_serializing_if = "Option::is_none")]
    pub index_previous: Option<String>,

    /// The address of the account that owns the objects in this directory.
    /// Only present in owner directories.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,

    /// The currency code of the TakerPays amount from the offers in this
    /// directory. Only present in offer directories.
    #[serde(
        rename = "TakerPaysCurrency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_currency: Option<Hash160>,

    /// The issuer of the TakerPays amount from the offers in this directory.
    /// Only present in offer directories.
    #[serde(
        rename = "TakerPaysIssuer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_issuer: Option<Hash160>,

    /// The currency code of the TakerGets amount from the offers in this
    /// directory. Only present in offer directories.
    #[serde(
        rename = "TakerGetsCurrency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_currency: Option<Hash160>,

    /// The issuer of the TakerGets amount from the offers in this directory.
    /// Only present in offer directories.
    #[serde(
        rename = "TakerGetsIssuer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_issuer: Option<Hash160>,

    /// The ID of the NFToken in a buy/sell offer directory. Only present
    /// in NFToken offer directories.
    #[serde(rename = "NFTokenID", default, skip_serializing_if = "Option::is_none")]
    pub nftoken_id: Option<Hash256>,

    /// The unique ID (hash) of this ledger entry.
    #[serde(rename = "index")]
    pub index: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_owner_directory() {
        let json = json!({
            "LedgerEntryType": "DirectoryNode",
            "Flags": 0,
            "RootIndex": "0000000000000000000000000000000000000000000000000000000000000000",
            "Indexes": [
                "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
            ],
            "Owner": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: DirectoryNode = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "DirectoryNode");
        assert_eq!(entry.indexes.len(), 1);
        assert!(entry.owner.is_some());
    }
}

//! The NFTokenPage ledger entry type.
//!
//! An [`NftTokenPage`] object contains a collection of NFTokens owned
//! by the same account. Accounts can have multiple pages linked together.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/nftokenpage>

use serde::{Deserialize, Serialize};
use xrpl_types::{Blob, Hash256};

/// A single NFToken within an [`NftTokenPage`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NfToken {
    /// The unique identifier for this NFToken.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Hash256,

    /// The URI associated with this NFToken, pointing to data or metadata.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,
}

impl crate::serde_helpers::StArrayElement for NfToken {
    const WRAPPER_KEY: &'static str = "NFToken";
}

/// An NFTokenPage ledger entry.
///
/// Contains up to 32 NFTokens owned by a single account. Multiple pages
/// are linked together to support accounts with more than 32 NFTokens.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/nftokenpage>
///
/// # Examples
///
/// ```
/// use xrpl_models::ledger::NftTokenPage;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "NFTokenPage",
///     "NFTokens": [
///         {
///             "NFTokenID": "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
///         }
///     ],
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: NftTokenPage = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.nftokens.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftTokenPage {
    /// The ledger entry type identifier. Always `"NFTokenPage"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The collection of NFTokens contained in this page.
    #[serde(rename = "NFTokens")]
    pub nftokens: crate::serde_helpers::StArray<NfToken>,

    /// The locator of the previous page, if this is not the first page.
    #[serde(
        rename = "PreviousPageMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_page_min: Option<Hash256>,

    /// The locator of the next page, if this is not the last page.
    #[serde(rename = "NextPageMin", default, skip_serializing_if = "Option::is_none")]
    pub next_page_min: Option<Hash256>,

    /// The identifying hash of the transaction that most recently modified
    /// this object, if any.
    #[serde(rename = "PreviousTxnID", default, skip_serializing_if = "Option::is_none")]
    pub previous_txn_id: Option<Hash256>,

    /// The index of the ledger that contains the transaction that most
    /// recently modified this object, if any.
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,

    /// The unique ID (hash) of this ledger entry.
    #[serde(rename = "index")]
    pub index: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_nft_page() {
        let json = json!({
            "LedgerEntryType": "NFTokenPage",
            "NFTokens": [
                {
                    "NFTokenID": "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65"
                },
                {
                    "NFTokenID": "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D66",
                    "URI": "68747470733A2F2F6578616D706C652E636F6D"
                }
            ],
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: NftTokenPage = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "NFTokenPage");
        assert_eq!(entry.nftokens.len(), 2);
        assert!(entry.nftokens[0].uri.is_none());
        assert!(entry.nftokens[1].uri.is_some());
    }
}

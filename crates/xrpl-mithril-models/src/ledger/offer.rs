//! The Offer ledger entry type.
//!
//! An [`Offer`] object represents an order on the XRPL decentralized exchange.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/offer>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Hash256};

/// An Offer (DEX order) ledger entry.
///
/// Represents an outstanding order on the XRPL's built-in decentralized
/// exchange. The offer expresses a willingness to exchange `taker_gets`
/// for `taker_pays`.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/offer>
///
/// # Examples
///
/// Deserialize a DEX offer from JSON:
///
/// ```
/// use xrpl_mithril_models::ledger::Offer;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "Offer",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Sequence": 7,
///     "Flags": 0,
///     "TakerPays": "5000000",
///     "TakerGets": "1000000",
///     "BookDirectory": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 50,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: Offer = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.sequence, 7);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    /// The ledger entry type identifier. Always `"Offer"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The address of the account that owns this offer.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The sequence number of the OfferCreate transaction that created
    /// this offer.
    #[serde(rename = "Sequence")]
    pub sequence: u32,

    /// A bit-map of boolean flags for this offer.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// The remaining amount and type of currency requested by the offer
    /// creator. The taker of the offer must pay this amount.
    #[serde(rename = "TakerPays")]
    pub taker_pays: Amount,

    /// The remaining amount and type of currency being provided by the
    /// offer creator. The taker of the offer receives this amount.
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,

    /// The ID of the Offer Directory that links to this offer.
    #[serde(rename = "BookDirectory")]
    pub book_directory: Hash256,

    /// A hint indicating which page of the offer directory links to this
    /// object.
    #[serde(rename = "BookNode", default, skip_serializing_if = "Option::is_none")]
    pub book_node: Option<String>,

    /// A hint indicating which page of the owner directory links to this
    /// object.
    #[serde(rename = "OwnerNode", default, skip_serializing_if = "Option::is_none")]
    pub owner_node: Option<String>,

    /// The time after which this offer is considered unfunded and can be
    /// removed. In seconds since the Ripple Epoch.
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

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
    fn deserialize_offer() {
        let json = json!({
            "LedgerEntryType": "Offer",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Sequence": 7,
            "Flags": 0,
            "TakerPays": "5000000",
            "TakerGets": "1000000",
            "BookDirectory": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 50,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Offer = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "Offer");
        assert_eq!(entry.sequence, 7);
        assert_eq!(entry.flags, 0);
    }
}

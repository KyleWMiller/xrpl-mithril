//! MPT-related ledger entry types.
//!
//! This module defines two ledger entry types for Multi-Purpose Tokens:
//! - [`MptIssuance`]: Defines the properties of a token issuance.
//! - [`MpToken`]: Tracks an individual holder's balance of an MPT.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptokenissuance>
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptoken>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Blob, Hash256, MptIssuanceId};

/// An MPTIssuance ledger entry.
///
/// Defines the properties and metadata of a Multi-Purpose Token issuance,
/// including the issuer, maximum supply, transfer fee, and metadata.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptokenissuance>
///
/// # Examples
///
/// Deserialize an MPT issuance from JSON:
///
/// ```
/// use xrpl_mithril_models::ledger::MptIssuance;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "MPTokenIssuance",
///     "Issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Flags": 0,
///     "Sequence": 1,
///     "MaximumAmount": "1000000",
///     "OutstandingAmount": "500000",
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 10,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: MptIssuance = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.maximum_amount, Some("1000000".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MptIssuance {
    /// The ledger entry type identifier. Always `"MPTokenIssuance"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The address of the account that created this token issuance.
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,

    /// The number of decimal places used in the token's amounts.
    #[serde(rename = "AssetScale", default, skip_serializing_if = "Option::is_none")]
    pub asset_scale: Option<u8>,

    /// The maximum amount of this token that can ever be issued, as a
    /// string integer.
    #[serde(rename = "MaximumAmount", default, skip_serializing_if = "Option::is_none")]
    pub maximum_amount: Option<String>,

    /// The total amount of this token currently in circulation, as a
    /// string integer.
    #[serde(rename = "OutstandingAmount", default, skip_serializing_if = "Option::is_none")]
    pub outstanding_amount: Option<String>,

    /// The fee to charge for transferring this token between accounts,
    /// in units of 1/10,000.
    #[serde(rename = "TransferFee", default, skip_serializing_if = "Option::is_none")]
    pub transfer_fee: Option<u16>,

    /// Arbitrary metadata about this token issuance.
    #[serde(rename = "MPTokenMetadata", default, skip_serializing_if = "Option::is_none")]
    pub mptoken_metadata: Option<Blob>,

    /// A bit-map of boolean flags for this token issuance.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// A hint indicating which page of the issuer's owner directory links
    /// to this object.
    #[serde(rename = "OwnerNode", default, skip_serializing_if = "Option::is_none")]
    pub owner_node: Option<String>,

    /// The sequence number of the transaction that created this issuance.
    #[serde(rename = "Sequence")]
    pub sequence: u32,

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

/// An MPToken ledger entry.
///
/// Tracks an individual account's balance of a specific Multi-Purpose Token.
/// Each holder of an MPT has their own MPToken ledger entry.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptoken>
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::ledger::MpToken;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "MPToken",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "MPTokenIssuanceID": "00000001AABBCCDD00000001AABBCCDD00000001AABBCCDD",
///     "MPTAmount": "1000",
///     "Flags": 0,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 20,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: MpToken = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.mpt_amount, Some("1000".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MpToken {
    /// The ledger entry type identifier. Always `"MPToken"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The address of the account that holds this token balance.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The identifier of the MPTIssuance this token belongs to.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: MptIssuanceId,

    /// The amount of this token held by the account, as a string integer.
    #[serde(rename = "MPTAmount", default, skip_serializing_if = "Option::is_none")]
    pub mpt_amount: Option<String>,

    /// A bit-map of boolean flags for this token balance.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// A hint indicating which page of the holder's owner directory links
    /// to this object.
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
    fn deserialize_mpt_issuance() {
        let json = json!({
            "LedgerEntryType": "MPTokenIssuance",
            "Issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Flags": 0,
            "Sequence": 1,
            "MaximumAmount": "1000000",
            "OutstandingAmount": "500000",
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 10,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: MptIssuance = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "MPTokenIssuance");
        assert_eq!(entry.sequence, 1);
        assert_eq!(entry.maximum_amount, Some("1000000".to_string()));
    }

    #[test]
    fn deserialize_mp_token() {
        let json = json!({
            "LedgerEntryType": "MPToken",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "MPTokenIssuanceID": "00000001AABBCCDD00000001AABBCCDD00000001AABBCCDD",
            "MPTAmount": "1000",
            "Flags": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 20,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: MpToken = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "MPToken");
        assert_eq!(entry.mpt_amount, Some("1000".to_string()));
    }
}

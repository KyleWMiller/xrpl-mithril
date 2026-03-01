//! The AccountRoot ledger entry type.
//!
//! An [`AccountRoot`] object describes a single account, its settings, and
//! XRP balance. Every funded account on the XRPL has exactly one AccountRoot.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/accountroot>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Amount, Blob, Hash128, Hash256};

/// An AccountRoot ledger entry.
///
/// Represents a single account on the XRP Ledger. Contains the account's
/// XRP balance, sequence number, settings flags, and optional configuration.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/accountroot>
///
/// # Examples
///
/// Deserialize from JSON returned by the XRPL server:
///
/// ```
/// use xrpl_mithril_models::ledger::AccountRoot;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "AccountRoot",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Balance": "10000000000",
///     "Sequence": 1,
///     "Flags": 0,
///     "OwnerCount": 0,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 1,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: AccountRoot = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.sequence, 1);
/// assert_eq!(entry.owner_count, 0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountRoot {
    /// The ledger entry type identifier. Always `"AccountRoot"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The identifying (classic) address of this account.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The account's current XRP balance in drops, represented as a string.
    #[serde(rename = "Balance")]
    pub balance: Amount,

    /// The sequence number of the next valid transaction for this account.
    #[serde(rename = "Sequence")]
    pub sequence: u32,

    /// A bit-map of boolean flags for this account.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// The number of objects this account owns in the ledger, which
    /// contributes to its owner reserve.
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,

    /// The identifying hash of the transaction that most recently modified
    /// this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,

    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,

    /// The identifying hash of the transaction most recently sent by this
    /// account. Enabled by the `asfAccountTxnID` flag.
    #[serde(rename = "AccountTxnID", default, skip_serializing_if = "Option::is_none")]
    pub account_txn_id: Option<Hash256>,

    /// The domain that owns this account, as a hex string representing the
    /// ASCII domain in lowercase.
    #[serde(rename = "Domain", default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<Blob>,

    /// The md5 hash of an email address. Clients can display a Gravatar image.
    #[serde(rename = "EmailHash", default, skip_serializing_if = "Option::is_none")]
    pub email_hash: Option<Hash128>,

    /// A public key that may be used to send encrypted messages to this account.
    #[serde(rename = "MessageKey", default, skip_serializing_if = "Option::is_none")]
    pub message_key: Option<Blob>,

    /// The address of a key pair that can authorize transactions for this
    /// account instead of the master key.
    #[serde(rename = "RegularKey", default, skip_serializing_if = "Option::is_none")]
    pub regular_key: Option<AccountId>,

    /// How many tickets this account owns in the ledger.
    #[serde(rename = "TicketCount", default, skip_serializing_if = "Option::is_none")]
    pub ticket_count: Option<u32>,

    /// A transfer fee to charge other users for sending currency issued by
    /// this account (in billionths of a unit).
    #[serde(rename = "TransferRate", default, skip_serializing_if = "Option::is_none")]
    pub transfer_rate: Option<u32>,

    /// Another account that can mint NFTokens on behalf of this account.
    #[serde(rename = "NFTokenMinter", default, skip_serializing_if = "Option::is_none")]
    pub nftoken_minter: Option<AccountId>,

    /// The total number of NFTokens minted by and on behalf of this account.
    #[serde(rename = "MintedNFTokens", default, skip_serializing_if = "Option::is_none")]
    pub minted_nftokens: Option<u32>,

    /// The total number of NFTokens burned by and on behalf of this account.
    #[serde(rename = "BurnedNFTokens", default, skip_serializing_if = "Option::is_none")]
    pub burned_nftokens: Option<u32>,

    /// The unique ID (hash) of this ledger entry.
    #[serde(rename = "index")]
    pub index: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_account_root() {
        let json = json!({
            "LedgerEntryType": "AccountRoot",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Balance": "10000000000",
            "Sequence": 1,
            "Flags": 0,
            "OwnerCount": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 1,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: AccountRoot = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "AccountRoot");
        assert_eq!(entry.sequence, 1);
        assert_eq!(entry.flags, 0);
        assert_eq!(entry.owner_count, 0);
    }

    #[test]
    fn round_trip_account_root() {
        let json = json!({
            "LedgerEntryType": "AccountRoot",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Balance": "10000000000",
            "Sequence": 1,
            "Flags": 0,
            "OwnerCount": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 1,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: AccountRoot = serde_json::from_value(json).expect("should deserialize");
        let serialized = serde_json::to_value(&entry).expect("should serialize");
        let map = serialized.as_object().expect("should be object");
        assert_eq!(
            map.get("LedgerEntryType").and_then(|v| v.as_str()),
            Some("AccountRoot")
        );
        assert_eq!(
            map.get("Account").and_then(|v| v.as_str()),
            Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
        );
    }
}

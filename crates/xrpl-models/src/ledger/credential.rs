//! The Credential ledger entry type.
//!
//! A [`Credential`] object represents an on-ledger attestation that an
//! issuer has made about a subject account.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/credential>

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Blob, Hash256};

/// A Credential ledger entry.
///
/// Represents a credential issued by one account (the issuer) attesting
/// something about another account (the subject). Credentials can have
/// an expiration and an optional URI pointing to off-ledger data.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/credential>
///
/// # Examples
///
/// ```
/// use xrpl_models::ledger::Credential;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "Credential",
///     "Subject": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "CredentialType": "4B5943",
///     "Flags": 0,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 20,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: Credential = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.flags, 0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    /// The ledger entry type identifier. Always `"Credential"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The account that the credential is about (the subject).
    #[serde(rename = "Subject")]
    pub subject: AccountId,

    /// The account that issued the credential.
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,

    /// A value indicating the type of credential (e.g., KYC, accreditation).
    /// Represented as hex-encoded data.
    #[serde(rename = "CredentialType")]
    pub credential_type: Blob,

    /// The time after which this credential is no longer valid, in seconds
    /// since the Ripple Epoch.
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    /// A URI pointing to additional data about the credential.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,

    /// A bit-map of boolean flags for this credential.
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
    fn deserialize_credential() {
        let json = json!({
            "LedgerEntryType": "Credential",
            "Subject": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "CredentialType": "4B5943",
            "Flags": 0,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 20,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Credential = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "Credential");
        assert_eq!(entry.flags, 0);
    }
}

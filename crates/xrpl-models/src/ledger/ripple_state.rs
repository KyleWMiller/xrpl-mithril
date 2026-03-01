//! The RippleState ledger entry type.
//!
//! A [`RippleState`] object represents a trust line between two accounts.
//! It tracks the balance of a particular issued currency between them.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/ripplestate>

use serde::{Deserialize, Serialize};
use xrpl_types::{Hash256, IssuedAmount};

/// A RippleState (trust line) ledger entry.
///
/// Connects two accounts for a single currency. The `Balance` reflects
/// the net position: positive means the low account holds tokens issued
/// by the high account, negative means the reverse.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/ripplestate>
///
/// # Examples
///
/// Deserialize a trust line from JSON:
///
/// ```
/// use xrpl_models::ledger::RippleState;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "RippleState",
///     "Balance": {
///         "value": "10",
///         "currency": "USD",
///         "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji"
///     },
///     "LowLimit": {
///         "value": "0",
///         "currency": "USD",
///         "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
///     },
///     "HighLimit": {
///         "value": "100",
///         "currency": "USD",
///         "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
///     },
///     "Flags": 65536,
///     "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
///     "PreviousTxnLgrSeq": 100,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: RippleState = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.flags, 65536);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RippleState {
    /// The ledger entry type identifier. Always `"RippleState"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The balance of the trust line, from the perspective of the low account.
    /// A positive balance means the low account holds tokens; negative means
    /// the high account holds tokens.
    #[serde(rename = "Balance")]
    pub balance: IssuedAmount,

    /// The limit and account address of the low account on this trust line.
    /// The `value` field is the maximum amount the low account is willing to owe.
    #[serde(rename = "LowLimit")]
    pub low_limit: IssuedAmount,

    /// The limit and account address of the high account on this trust line.
    /// The `value` field is the maximum amount the high account is willing to owe.
    #[serde(rename = "HighLimit")]
    pub high_limit: IssuedAmount,

    /// A bit-map of boolean flags for this trust line.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// A hint indicating which page of the low account's owner directory
    /// links to this object.
    #[serde(rename = "LowNode", default, skip_serializing_if = "Option::is_none")]
    pub low_node: Option<String>,

    /// A hint indicating which page of the high account's owner directory
    /// links to this object.
    #[serde(rename = "HighNode", default, skip_serializing_if = "Option::is_none")]
    pub high_node: Option<String>,

    /// The inbound quality setting of the low account (fee for incoming
    /// payments, in billionths).
    #[serde(rename = "LowQualityIn", default, skip_serializing_if = "Option::is_none")]
    pub low_quality_in: Option<u32>,

    /// The outbound quality setting of the low account.
    #[serde(rename = "LowQualityOut", default, skip_serializing_if = "Option::is_none")]
    pub low_quality_out: Option<u32>,

    /// The inbound quality setting of the high account.
    #[serde(rename = "HighQualityIn", default, skip_serializing_if = "Option::is_none")]
    pub high_quality_in: Option<u32>,

    /// The outbound quality setting of the high account.
    #[serde(rename = "HighQualityOut", default, skip_serializing_if = "Option::is_none")]
    pub high_quality_out: Option<u32>,

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
    fn deserialize_ripple_state() {
        let json = json!({
            "LedgerEntryType": "RippleState",
            "Balance": {
                "value": "10",
                "currency": "USD",
                "issuer": "rrrrrrrrrrrrrrrrrrrrBZbvji"
            },
            "LowLimit": {
                "value": "0",
                "currency": "USD",
                "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
            },
            "HighLimit": {
                "value": "100",
                "currency": "USD",
                "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
            },
            "Flags": 65536,
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 100,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: RippleState = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "RippleState");
        assert_eq!(entry.flags, 65536);
    }
}

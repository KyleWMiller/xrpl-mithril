//! The FeeSettings ledger entry type.
//!
//! The [`FeeSettings`] object is a singleton that contains the current
//! base transaction cost and reserve requirements as determined by
//! fee voting.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/feesettings>

use serde::{Deserialize, Serialize};
use xrpl_types::Hash256;

/// The FeeSettings singleton ledger entry.
///
/// Contains the current fee schedule for the network. There is exactly
/// one FeeSettings object in every ledger version.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/feesettings>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeSettings {
    /// The ledger entry type identifier. Always `"FeeSettings"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The transaction cost of the "reference transaction" in drops of XRP,
    /// as a string integer.
    #[serde(rename = "BaseFee")]
    pub base_fee: String,

    /// The `BaseFee` translated into "fee units" (used internally by the
    /// transaction cost calculation).
    #[serde(rename = "ReferenceFeeUnits")]
    pub reference_fee_units: u32,

    /// The base reserve for an account in the XRP Ledger, in drops.
    #[serde(rename = "ReserveBase")]
    pub reserve_base: u32,

    /// The incremental owner reserve for each object an account owns, in drops.
    #[serde(rename = "ReserveIncrement")]
    pub reserve_increment: u32,

    /// A bit-map of boolean flags for this entry.
    #[serde(rename = "Flags")]
    pub flags: u32,

    /// The unique ID (hash) of this ledger entry.
    #[serde(rename = "index")]
    pub index: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_fee_settings() {
        let json = json!({
            "LedgerEntryType": "FeeSettings",
            "BaseFee": "A",
            "ReferenceFeeUnits": 10,
            "ReserveBase": 10000000,
            "ReserveIncrement": 2000000,
            "Flags": 0,
            "index": "4BC50C9B0D8515BE09A7627D12897A15FB64CBF90C71FFE5E1E4B3742C55E982"
        });

        let entry: FeeSettings = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "FeeSettings");
        assert_eq!(entry.base_fee, "A");
        assert_eq!(entry.reserve_base, 10000000);
        assert_eq!(entry.reserve_increment, 2000000);
    }
}

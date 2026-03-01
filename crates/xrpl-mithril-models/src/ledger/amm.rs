//! The AMM ledger entry type.
//!
//! An [`Amm`] object represents an Automated Market Maker (AMM) instance
//! that holds a pool of two assets and provides liquidity for exchanges.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/amm>

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::{AccountId, Hash256, IssuedAmount, Issue};

use crate::serde_helpers::{StArray, StArrayElement};

/// A single vote entry in an AMM's trading fee voting slots.
///
/// Represents one LP token holder's vote for the trading fee. Up to 8
/// vote entries are tracked per AMM. Votes are weighted by the voter's
/// LP token balance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteEntry {
    /// The account that cast this vote.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The trading fee this account voted for, in units of 1/100,000.
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,

    /// The voter's LP token balance weight at the time of the vote.
    #[serde(rename = "VoteWeight")]
    pub vote_weight: u32,
}

impl StArrayElement for VoteEntry {
    const WRAPPER_KEY: &'static str = "VoteEntry";
}

/// An AMM (Automated Market Maker) ledger entry.
///
/// Represents an AMM instance with a pool of two assets, an LP token,
/// and optional auction/voting state.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/amm>
///
/// # Examples
///
/// Deserialize an AMM entry from JSON:
///
/// ```
/// use xrpl_mithril_models::ledger::Amm;
///
/// let json = serde_json::json!({
///     "LedgerEntryType": "AMM",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Asset": {"currency": "XRP"},
///     "Asset2": {"currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"},
///     "LPTokenBalance": {
///         "value": "1000",
///         "currency": "USD",
///         "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
///     },
///     "TradingFee": 500,
///     "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
/// });
///
/// let entry: Amm = serde_json::from_value(json).unwrap();
/// assert_eq!(entry.trading_fee, 500);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amm {
    /// The ledger entry type identifier. Always `"AMM"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The address of the special account that holds this AMM's assets.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The definition for one of the two assets this AMM holds.
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The definition for the other asset this AMM holds.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,

    /// Details of the current auction slot holder, if any. Uses
    /// `serde_json::Value` to represent the complex nested structure
    /// without full typing.
    #[serde(rename = "AuctionSlot", default, skip_serializing_if = "Option::is_none")]
    pub auction_slot: Option<serde_json::Value>,

    /// The total outstanding balance of liquidity provider tokens from
    /// this AMM instance.
    #[serde(rename = "LPTokenBalance")]
    pub lp_token_balance: IssuedAmount,

    /// The percentage fee to be charged for trades against this AMM
    /// instance, in units of 1/100,000 (a value of 1 = 0.001%).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,

    /// The current votes for the trading fee, represented as a list of
    /// [`VoteEntry`] objects (up to 8). Each entry records one LP token
    /// holder's fee vote and their voting weight.
    #[serde(rename = "VoteSlots", default, skip_serializing_if = "Option::is_none")]
    pub vote_slots: Option<StArray<VoteEntry>>,

    /// A hint indicating which page of the AMM account's owner directory
    /// links to this object.
    #[serde(rename = "OwnerNode", default, skip_serializing_if = "Option::is_none")]
    pub owner_node: Option<String>,

    /// The unique ID (hash) of this ledger entry.
    #[serde(rename = "index")]
    pub index: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_amm_with_vote_slots() {
        let json = json!({
            "LedgerEntryType": "AMM",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Asset": {"currency": "XRP"},
            "Asset2": {"currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"},
            "LPTokenBalance": {
                "value": "1000",
                "currency": "USD",
                "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
            },
            "TradingFee": 500,
            "VoteSlots": [
                {"VoteEntry": {
                    "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                    "TradingFee": 600,
                    "VoteWeight": 50000
                }},
                {"VoteEntry": {
                    "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
                    "TradingFee": 400,
                    "VoteWeight": 30000
                }}
            ],
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Amm = serde_json::from_value(json).expect("should deserialize");
        let slots = entry.vote_slots.expect("should have vote slots");
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].trading_fee, 600);
        assert_eq!(slots[0].vote_weight, 50000);
        assert_eq!(slots[1].trading_fee, 400);
    }

    #[test]
    fn deserialize_amm() {
        let json = json!({
            "LedgerEntryType": "AMM",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Asset": {"currency": "XRP"},
            "Asset2": {"currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"},
            "LPTokenBalance": {
                "value": "1000",
                "currency": "USD",
                "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
            },
            "TradingFee": 500,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Amm = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "AMM");
        assert_eq!(entry.trading_fee, 500);
    }
}

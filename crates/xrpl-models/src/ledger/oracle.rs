//! The Oracle ledger entry type.
//!
//! An [`Oracle`] object represents a price oracle instance that provides
//! on-ledger price data for asset pairs.
//!
//! # XRPL Documentation
//!
//! <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/oracle>

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Blob, CurrencyCode, Hash256};

/// A single price data entry within an [`Oracle`].
///
/// Represents the price of a base asset in terms of a quote asset,
/// expressed as `AssetPrice * 10^(-Scale)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceData {
    /// The base asset for this price entry.
    #[serde(rename = "BaseAsset")]
    pub base_asset: CurrencyCode,

    /// The quote asset for this price entry.
    #[serde(rename = "QuoteAsset")]
    pub quote_asset: CurrencyCode,

    /// The price of the base asset in terms of the quote asset, represented
    /// as a string integer. The actual price is `AssetPrice * 10^(-Scale)`.
    #[serde(rename = "AssetPrice", default, skip_serializing_if = "Option::is_none")]
    pub asset_price: Option<String>,

    /// The scaling factor for the asset price. The actual price is
    /// `AssetPrice * 10^(-Scale)`.
    #[serde(rename = "Scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,
}

/// An Oracle ledger entry.
///
/// Represents a price oracle that publishes exchange rate data on-ledger.
/// Each oracle can track multiple asset pairs via the `PriceDataSeries`.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/oracle>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Oracle {
    /// The ledger entry type identifier. Always `"Oracle"`.
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,

    /// The account that controls and updates this oracle.
    #[serde(rename = "Owner")]
    pub owner: AccountId,

    /// An arbitrary string identifying the oracle provider (e.g., a company name).
    #[serde(rename = "Provider")]
    pub provider: Blob,

    /// An arbitrary string describing the type of asset class being tracked.
    #[serde(rename = "AssetClass")]
    pub asset_class: Blob,

    /// The time of the last update, in seconds since the Ripple Epoch.
    #[serde(rename = "LastUpdateTime")]
    pub last_update_time: u32,

    /// The array of price data entries for this oracle.
    #[serde(rename = "PriceDataSeries")]
    pub price_data_series: Vec<PriceData>,

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
    fn deserialize_oracle() {
        let json = json!({
            "LedgerEntryType": "Oracle",
            "Owner": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Provider": "70726F7669646572",
            "AssetClass": "63757272656E6379",
            "LastUpdateTime": 700000000,
            "PriceDataSeries": [
                {
                    "BaseAsset": "USD",
                    "QuoteAsset": "EUR",
                    "AssetPrice": "12345",
                    "Scale": 5
                }
            ],
            "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000",
            "PreviousTxnLgrSeq": 100,
            "index": "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
        });

        let entry: Oracle = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(entry.ledger_entry_type, "Oracle");
        assert_eq!(entry.last_update_time, 700000000);
        assert_eq!(entry.price_data_series.len(), 1);
        assert_eq!(entry.price_data_series[0].asset_price, Some("12345".to_string()));
    }
}

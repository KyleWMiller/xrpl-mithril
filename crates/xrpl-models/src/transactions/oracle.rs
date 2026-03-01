//! Price Oracle transaction types.
//!
//! The Price Oracle amendment enables on-ledger price feeds. Oracle providers
//! publish asset price data directly on the XRP Ledger, where it can be
//! consumed by other on-ledger features (e.g., AMM, lending).
//!
//! Two transactions manage oracle lifecycle:
//! - [`OracleSet`] — Creates or updates an oracle with price data.
//! - [`OracleDelete`] — Removes an oracle from the ledger.
//!
//! Price data is stored as a series of [`PriceData`] entries, each containing
//! a base/quote asset pair and a scaled integer price.

use serde::{Deserialize, Serialize};
use xrpl_types::Blob;

// Re-export PriceData from the canonical ledger definition.
// The same type is used in both OracleSet transactions and Oracle ledger entries.
pub use crate::ledger::oracle::PriceData;

// ---------------------------------------------------------------------------
// OracleSet — TransactionType = 51
// ---------------------------------------------------------------------------

/// An OracleSet transaction (TransactionType = 51).
///
/// Creates a new oracle or updates an existing one owned by the sending
/// account. Each account can own multiple oracles, distinguished by
/// `oracle_document_id`.
///
/// When updating, only the provided fields are changed; omitted optional
/// fields retain their previous values. To update price data, include the
/// full `price_data_series` — entries not included are removed from the oracle.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/oracleset>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleSet {
    /// A unique identifier for this oracle within the account's oracles.
    ///
    /// An account can own multiple oracle objects, each with a different
    /// `oracle_document_id`. This value is used to derive the oracle's
    /// ledger entry ID.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,

    /// The provider name or identifier.
    ///
    /// An opaque blob describing who is providing the price data
    /// (e.g., a hex-encoded string like "chainlink" or "band"). Required
    /// when creating a new oracle; optional when updating.
    #[serde(rename = "Provider", default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<Blob>,

    /// The asset class this oracle covers.
    ///
    /// An opaque blob describing the category of assets (e.g., "currency",
    /// "commodity"). Required when creating a new oracle; optional when
    /// updating.
    #[serde(rename = "AssetClass", default, skip_serializing_if = "Option::is_none")]
    pub asset_class: Option<Blob>,

    /// The time of the last price update, in seconds since the Ripple Epoch
    /// (2000-01-01T00:00:00Z).
    ///
    /// Must be within 300 seconds of the last validated ledger close time.
    #[serde(rename = "LastUpdateTime", default, skip_serializing_if = "Option::is_none")]
    pub last_update_time: Option<u32>,

    /// The array of price data entries.
    ///
    /// Each entry contains a base/quote asset pair and a scaled price.
    /// Maximum 10 entries per oracle. When updating, this replaces the
    /// entire price data series.
    #[serde(rename = "PriceDataSeries", default, skip_serializing_if = "Option::is_none")]
    pub price_data_series: Option<crate::serde_helpers::StArray<PriceData>>,
}

// ---------------------------------------------------------------------------
// OracleDelete — TransactionType = 52
// ---------------------------------------------------------------------------

/// An OracleDelete transaction (TransactionType = 52).
///
/// Removes an oracle object from the ledger. Only the oracle owner (the
/// `Account` in [`TransactionCommon`](super::TransactionCommon)) can delete it.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/oracledelete>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleDelete {
    /// The identifier of the oracle to delete, matching the value used in
    /// the original [`OracleSet`].
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
}

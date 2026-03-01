//! Price Oracle request types.

use serde::Serialize;

use super::{LedgerSpecifier, XrplRequest};
use crate::responses::oracle::GetAggregatePriceResponse;

/// An oracle identifier for aggregate price queries.
#[derive(Debug, Clone, Serialize)]
pub struct OracleIdentifier {
    /// The oracle provider's account.
    pub account: String,
    /// The oracle document ID.
    pub oracle_document_id: u32,
}

/// Request an aggregate price from multiple oracles.
///
/// # Examples
///
/// ```
/// use xrpl_models::requests::{GetAggregatePriceRequest, OracleIdentifier};
///
/// let request = GetAggregatePriceRequest {
///     base_asset: "USD".to_string(),
///     quote_asset: "EUR".to_string(),
///     oracles: vec![OracleIdentifier {
///         account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///         oracle_document_id: 1,
///     }],
///     trim: None,
///     trim_threshold: None,
///     ledger_index: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct GetAggregatePriceRequest {
    /// The base asset currency code.
    pub base_asset: String,
    /// The quote asset currency code.
    pub quote_asset: String,
    /// Oracles to query.
    pub oracles: Vec<OracleIdentifier>,
    /// Percentage of outliers to trim (0-25).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim: Option<u8>,
    /// Threshold percentage for filtering old data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim_threshold: Option<u8>,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for GetAggregatePriceRequest {
    type Response = GetAggregatePriceResponse;
    fn method(&self) -> &'static str {
        "get_aggregate_price"
    }
}

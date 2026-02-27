//! Oracle-related response types.

use serde::Deserialize;

/// Response from the `get_aggregate_price` method.
#[derive(Debug, Clone, Deserialize)]
pub struct GetAggregatePriceResponse {
    /// The median price.
    pub entire_set: Option<PriceStats>,
    /// Price stats after trimming.
    pub trimmed_set: Option<PriceStats>,
    /// Time of the most recent data point.
    pub time: Option<u32>,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// Price statistics from oracle aggregation.
#[derive(Debug, Clone, Deserialize)]
pub struct PriceStats {
    /// The mean price.
    pub mean: Option<String>,
    /// The median price.
    pub median: Option<String>,
    /// The size of the data set.
    pub size: Option<u32>,
    /// Standard deviation.
    pub standard_deviation: Option<String>,
}

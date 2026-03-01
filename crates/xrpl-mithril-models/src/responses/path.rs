//! Path finding and order book response types.

use serde::Deserialize;
use xrpl_mithril_types::{AccountId, Amount};

/// Response from the `book_offers` method.
#[derive(Debug, Clone, Deserialize)]
pub struct BookOffersResponse {
    /// Order book offers.
    pub offers: Vec<BookOffer>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// An offer in a book_offers response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BookOffer {
    /// The account placing the offer.
    pub account: AccountId,
    /// Sequence number of the offer.
    pub sequence: u32,
    /// Offer flags.
    pub flags: u32,
    /// Amount the offer creator is willing to pay.
    pub taker_gets: Amount,
    /// Amount the offer creator wants to receive.
    pub taker_pays: Amount,
    /// Quality (exchange rate).
    pub quality: Option<String>,
    /// The offer creator's funded amount available.
    #[serde(default)]
    pub owner_funds: Option<String>,
    /// Expiration time.
    pub expiration: Option<u32>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `deposit_authorized` method.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::responses::path::DepositAuthorizedResponse;
///
/// let json = serde_json::json!({
///     "deposit_authorized": true,
///     "source_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "destination_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
/// });
///
/// let response: DepositAuthorizedResponse = serde_json::from_value(json).unwrap();
/// assert!(response.deposit_authorized);
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct DepositAuthorizedResponse {
    /// Whether the deposit is authorized.
    pub deposit_authorized: bool,
    /// The source account.
    pub source_account: AccountId,
    /// The destination account.
    pub destination_account: AccountId,
    /// Ledger index.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// Response from the `ripple_path_find` method.
#[derive(Debug, Clone, Deserialize)]
pub struct RipplePathFindResponse {
    /// Possible payment paths.
    pub alternatives: Vec<PathAlternative>,
    /// The destination account.
    pub destination_account: AccountId,
    /// The destination amount requested.
    pub destination_amount: Amount,
}

/// A payment path alternative.
#[derive(Debug, Clone, Deserialize)]
pub struct PathAlternative {
    /// The source amount required for this path.
    pub source_amount: Amount,
    /// The payment paths (arrays of path steps).
    pub paths_computed: Vec<Vec<serde_json::Value>>,
}

/// Response from the `path_find` method.
#[derive(Debug, Clone, Deserialize)]
pub struct PathFindResponse {
    /// Additional fields depend on the subcommand.
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

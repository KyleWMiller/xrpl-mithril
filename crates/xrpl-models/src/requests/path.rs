//! Path finding and order book request types.

use serde::Serialize;
use xrpl_types::{AccountId, Amount, Issue};

use super::{LedgerSpecifier, Marker, XrplRequest};
use crate::responses::path::{
    BookOffersResponse, DepositAuthorizedResponse, PathFindResponse, RipplePathFindResponse,
};

/// Request current offers for a specific order book.
#[derive(Debug, Clone, Serialize)]
pub struct BookOffersRequest {
    /// The asset the offers are buying.
    pub taker_gets: Issue,
    /// The asset the offers are selling.
    pub taker_pays: Issue,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Address of a prospective taker (affects funded amounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker: Option<AccountId>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for BookOffersRequest {
    type Response = BookOffersResponse;
    fn method(&self) -> &'static str {
        "book_offers"
    }
}

/// Check whether a deposit from one account to another would be authorized.
#[derive(Debug, Clone, Serialize)]
pub struct DepositAuthorizedRequest {
    /// The account that would send a deposit.
    pub source_account: AccountId,
    /// The account that would receive the deposit.
    pub destination_account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for DepositAuthorizedRequest {
    type Response = DepositAuthorizedResponse;
    fn method(&self) -> &'static str {
        "deposit_authorized"
    }
}

/// Find a payment path between two accounts (one-time lookup).
#[derive(Debug, Clone, Serialize)]
pub struct RipplePathFindRequest {
    /// The account that would send the payment.
    pub source_account: AccountId,
    /// The account that would receive the payment.
    pub destination_account: AccountId,
    /// The amount the destination would receive.
    pub destination_amount: Amount,
    /// Currencies the source can spend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_currencies: Option<Vec<Issue>>,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for RipplePathFindRequest {
    type Response = RipplePathFindResponse;
    fn method(&self) -> &'static str {
        "ripple_path_find"
    }
}

/// WebSocket-only: create, close, or check a persistent path-finding request.
#[derive(Debug, Clone, Serialize)]
pub struct PathFindRequest {
    /// The subcommand: `"create"`, `"close"`, or `"status"`.
    pub subcommand: String,
    /// The sending account (for `"create"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_account: Option<AccountId>,
    /// The receiving account (for `"create"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_account: Option<AccountId>,
    /// The amount the destination would receive (for `"create"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_amount: Option<Amount>,
    /// Currencies the source can spend (for `"create"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_currencies: Option<Vec<Issue>>,
}

impl XrplRequest for PathFindRequest {
    type Response = PathFindResponse;
    fn method(&self) -> &'static str {
        "path_find"
    }
}

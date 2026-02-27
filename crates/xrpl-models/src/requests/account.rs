//! Account-related request types.

use serde::Serialize;
use xrpl_types::AccountId;

use super::{LedgerSpecifier, Marker, XrplRequest};
use crate::responses::account::{
    AccountChannelsResponse, AccountCurrenciesResponse, AccountInfoResponse, AccountLinesResponse,
    AccountNftsResponse, AccountObjectsResponse, AccountOffersResponse, AccountTxResponse,
    GatewayBalancesResponse, NorippleCheckResponse,
};

/// Request information about an account.
#[derive(Debug, Clone, Serialize)]
pub struct AccountInfoRequest {
    /// The account to query.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// If true, include information about queued transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<bool>,
    /// If true, include the account's signer lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_lists: Option<bool>,
}

impl XrplRequest for AccountInfoRequest {
    type Response = AccountInfoResponse;
    fn method(&self) -> &'static str {
        "account_info"
    }
}

/// Request trust lines for an account.
#[derive(Debug, Clone, Serialize)]
pub struct AccountLinesRequest {
    /// The account to query.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Filter by peer account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<AccountId>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for AccountLinesRequest {
    type Response = AccountLinesResponse;
    fn method(&self) -> &'static str {
        "account_lines"
    }
}

/// Request payment channels where an account is the source or destination.
#[derive(Debug, Clone, Serialize)]
pub struct AccountChannelsRequest {
    /// The account to query.
    pub account: AccountId,
    /// Filter by destination account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_account: Option<AccountId>,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for AccountChannelsRequest {
    type Response = AccountChannelsResponse;
    fn method(&self) -> &'static str {
        "account_channels"
    }
}

/// Request the currencies an account can send or receive.
#[derive(Debug, Clone, Serialize)]
pub struct AccountCurrenciesRequest {
    /// The account to query.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for AccountCurrenciesRequest {
    type Response = AccountCurrenciesResponse;
    fn method(&self) -> &'static str {
        "account_currencies"
    }
}

/// Request NFTs owned by an account.
#[derive(Debug, Clone, Serialize)]
pub struct AccountNftsRequest {
    /// The account to query.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for AccountNftsRequest {
    type Response = AccountNftsResponse;
    fn method(&self) -> &'static str {
        "account_nfts"
    }
}

/// Request objects owned by an account (offers, trust lines, etc.).
#[derive(Debug, Clone, Serialize)]
pub struct AccountObjectsRequest {
    /// The account to query.
    pub account: AccountId,
    /// Filter by ledger entry type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for AccountObjectsRequest {
    type Response = AccountObjectsResponse;
    fn method(&self) -> &'static str {
        "account_objects"
    }
}

/// Request current offers placed by an account.
#[derive(Debug, Clone, Serialize)]
pub struct AccountOffersRequest {
    /// The account to query.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for AccountOffersRequest {
    type Response = AccountOffersResponse;
    fn method(&self) -> &'static str {
        "account_offers"
    }
}

/// Request an account's transaction history.
#[derive(Debug, Clone, Serialize)]
pub struct AccountTxRequest {
    /// The account to query.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Minimum ledger index to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index_min: Option<i64>,
    /// Maximum ledger index to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index_max: Option<i64>,
    /// If true, return results as binary blobs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// If true, return oldest results first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward: Option<bool>,
    /// Maximum number of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,
}

impl XrplRequest for AccountTxRequest {
    type Response = AccountTxResponse;
    fn method(&self) -> &'static str {
        "account_tx"
    }
}

/// Request total balances issued by an account (gateway).
#[derive(Debug, Clone, Serialize)]
pub struct GatewayBalancesRequest {
    /// The issuing account.
    pub account: AccountId,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Addresses to exclude from obligations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotwallet: Option<Vec<AccountId>>,
}

impl XrplRequest for GatewayBalancesRequest {
    type Response = GatewayBalancesResponse;
    fn method(&self) -> &'static str {
        "gateway_balances"
    }
}

/// Check an account's default ripple settings.
#[derive(Debug, Clone, Serialize)]
pub struct NorippleCheckRequest {
    /// The account to check.
    pub account: AccountId,
    /// The role of the account: `"gateway"` or `"user"`.
    pub role: String,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
    /// Maximum number of trust line problems to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// If true, include an array of suggested transactions to fix problems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<bool>,
}

impl XrplRequest for NorippleCheckRequest {
    type Response = NorippleCheckResponse;
    fn method(&self) -> &'static str {
        "noripple_check"
    }
}

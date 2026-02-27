//! WebSocket subscription request types.

use serde::Serialize;
use xrpl_types::{AccountId, Issue};

use super::XrplRequest;
use crate::responses::subscription::{SubscribeResponse, UnsubscribeResponse};

/// Subscribe to one or more event streams (WebSocket only).
#[derive(Debug, Clone, Default, Serialize)]
pub struct SubscribeRequest {
    /// Named streams to subscribe to (e.g., `"ledger"`, `"transactions"`,
    /// `"transactions_proposed"`, `"server"`, `"peer_status"`,
    /// `"consensus"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streams: Option<Vec<String>>,
    /// Accounts to watch for transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<AccountId>>,
    /// Accounts to watch (including proposed/unvalidated transactions).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_proposed: Option<Vec<AccountId>>,
    /// Order books to watch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub books: Option<Vec<BookSubscription>>,
}

/// An order book subscription entry.
#[derive(Debug, Clone, Serialize)]
pub struct BookSubscription {
    /// The asset being bought.
    pub taker_gets: Issue,
    /// The asset being sold.
    pub taker_pays: Issue,
    /// Address of a prospective taker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker: Option<AccountId>,
    /// If true, return the current order book snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,
    /// If true, subscribe to the reverse book too.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub both: Option<bool>,
}

impl XrplRequest for SubscribeRequest {
    type Response = SubscribeResponse;
    fn method(&self) -> &'static str {
        "subscribe"
    }
}

/// Unsubscribe from previously subscribed streams (WebSocket only).
#[derive(Debug, Clone, Default, Serialize)]
pub struct UnsubscribeRequest {
    /// Named streams to unsubscribe from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streams: Option<Vec<String>>,
    /// Accounts to stop watching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<AccountId>>,
    /// Proposed-account subscriptions to cancel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_proposed: Option<Vec<AccountId>>,
    /// Order book subscriptions to cancel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub books: Option<Vec<BookSubscription>>,
}

impl XrplRequest for UnsubscribeRequest {
    type Response = UnsubscribeResponse;
    fn method(&self) -> &'static str {
        "unsubscribe"
    }
}

//! Payment-related transaction types.
//!
//! This module defines the [`Payment`] transaction and the Check pseudo-payment
//! family: [`CheckCreate`], [`CheckCash`], and [`CheckCancel`].
//!
//! Payment paths are modeled via [`PathStep`], which can reference an account,
//! a currency, an issuer, or any combination thereof.

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::currency::CurrencyCode;
use xrpl_mithril_types::{AccountId, Amount, Hash256};

// ---------------------------------------------------------------------------
// PathStep — building block for payment paths
// ---------------------------------------------------------------------------

/// A single step in a payment path.
///
/// Each step can specify an account, a currency, an issuer, or a combination.
/// The XRPL payment engine uses paths to find intermediate order books when
/// the source and destination assets differ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathStep {
    /// An intermediary account in the path.
    #[serde(rename = "account", default, skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountId>,

    /// The currency to ripple through at this step.
    #[serde(rename = "currency", default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<CurrencyCode>,

    /// The issuer for the currency at this step.
    #[serde(rename = "issuer", default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// Payment — TransactionType = 0
// ---------------------------------------------------------------------------

/// A Payment transaction (TransactionType = 0).
///
/// Sends value from one account to another. The `amount` field specifies the
/// amount to deliver to the destination. For cross-currency payments, the
/// engine may debit up to `send_max` from the source account.
///
/// Partial payments (flag `tfPartialPayment` / 0x00020000) allow delivery of
/// less than `amount` as long as at least `deliver_min` is received.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::payment::Payment;
///
/// let json = serde_json::json!({
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000"
/// });
/// let payment: Payment = serde_json::from_value(json).unwrap();
/// assert_eq!(payment.destination.to_string(), "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/payment>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payment {
    /// The address of the account receiving the payment.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The amount of currency to deliver to the destination.
    ///
    /// For non-XRP amounts this includes the currency code and issuer.
    /// For partial payments this is the *maximum* amount to deliver.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// Maximum amount of source currency this transaction is allowed to cost.
    ///
    /// Required for cross-currency or cross-issuer payments. Ignored for
    /// direct XRP-to-XRP payments.
    #[serde(rename = "SendMax", default, skip_serializing_if = "Option::is_none")]
    pub send_max: Option<Amount>,

    /// Minimum amount of destination currency the transaction must deliver.
    ///
    /// Only valid for partial payments (`tfPartialPayment`). If the payment
    /// engine cannot deliver at least this amount, the transaction fails.
    #[serde(rename = "DeliverMin", default, skip_serializing_if = "Option::is_none")]
    pub deliver_min: Option<Amount>,

    /// Arbitrary tag that identifies the reason for the payment to the
    /// destination, or a hosted recipient to pay.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,

    /// Arbitrary 256-bit hash representing a specific reason or identifier
    /// for this payment.
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,

    /// Array of payment paths for cross-currency payments.
    ///
    /// Each inner `Vec<PathStep>` is a single path; the outer `Vec` holds
    /// up to 6 alternative paths. The payment engine chooses the cheapest.
    #[serde(rename = "Paths", default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<Vec<PathStep>>>,
}

// ---------------------------------------------------------------------------
// CheckCreate — TransactionType = 16
// ---------------------------------------------------------------------------

/// A CheckCreate transaction (TransactionType = 16).
///
/// Creates a Check object in the ledger, which is a deferred payment that
/// the destination can cash later (up to `send_max`).
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::payment::CheckCreate;
///
/// let json = serde_json::json!({
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "SendMax": "10000000"
/// });
/// let check: CheckCreate = serde_json::from_value(json).unwrap();
/// assert_eq!(check.destination.to_string(), "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/checkcreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckCreate {
    /// The address of the account that can cash the Check.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// Maximum amount the source is willing to pay, including transfer fees.
    #[serde(rename = "SendMax")]
    pub send_max: Amount,

    /// Arbitrary tag that identifies the reason for the Check to the
    /// destination, or a hosted recipient.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,

    /// Time after which the Check is no longer valid, in seconds since the
    /// Ripple Epoch (2000-01-01T00:00:00Z).
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    /// Arbitrary 256-bit hash representing a specific reason or identifier
    /// for this Check.
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,
}

// ---------------------------------------------------------------------------
// CheckCash — TransactionType = 17
// ---------------------------------------------------------------------------

/// A CheckCash transaction (TransactionType = 17).
///
/// Redeems a Check object. The destination of the Check submits this
/// transaction. Exactly one of `amount` or `deliver_min` must be provided:
///
/// - `amount`: Cash the Check for exactly this value.
/// - `deliver_min`: Cash the Check for at least this value (flexible amount).
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::payment::CheckCash;
///
/// let json = serde_json::json!({
///     "CheckID": "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0",
///     "Amount": "5000000"
/// });
/// let cash: CheckCash = serde_json::from_value(json).unwrap();
/// assert!(cash.amount.is_some());
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/checkcash>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckCash {
    /// The ID of the Check ledger object to cash (from a previous
    /// [`CheckCreate`] transaction).
    #[serde(rename = "CheckID")]
    pub check_id: Hash256,

    /// Redeem the Check for exactly this amount.
    ///
    /// Mutually exclusive with `deliver_min`.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,

    /// Redeem the Check for at least this amount, and for as much as possible.
    ///
    /// Mutually exclusive with `amount`.
    #[serde(rename = "DeliverMin", default, skip_serializing_if = "Option::is_none")]
    pub deliver_min: Option<Amount>,
}

// ---------------------------------------------------------------------------
// CheckCancel — TransactionType = 18
// ---------------------------------------------------------------------------

/// A CheckCancel transaction (TransactionType = 18).
///
/// Cancels an unredeemed Check, removing it from the ledger. The Check can
/// be cancelled by the source, the destination, or anyone if the Check has
/// expired.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::transactions::payment::CheckCancel;
///
/// let json = serde_json::json!({
///     "CheckID": "49647F0D748DC3FE26BDACBC57F251AADEFFF391403EC9BF87C97F67E9977FB0"
/// });
/// let cancel: CheckCancel = serde_json::from_value(json).unwrap();
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/checkcancel>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckCancel {
    /// The ID of the Check ledger object to cancel.
    #[serde(rename = "CheckID")]
    pub check_id: Hash256,
}

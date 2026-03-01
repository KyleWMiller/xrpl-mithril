//! Trust line transaction types.
//!
//! This module defines the [`TrustSet`] transaction, which creates or modifies
//! a trust line between two accounts for an issued currency.

use serde::{Deserialize, Serialize};
use xrpl_types::amount::IssuedAmount;

// ---------------------------------------------------------------------------
// TrustSet — TransactionType = 20
// ---------------------------------------------------------------------------

/// A TrustSet transaction (TransactionType = 20).
///
/// Creates or modifies a trust line linking two accounts. The trust line's
/// currency and issuer are derived from `limit_amount`. The `limit_amount`
/// value is the maximum balance the account is willing to hold in that
/// currency from that issuer.
///
/// Optional `quality_in` and `quality_out` fields allow the account to value
/// incoming and outgoing balances on this trust line at a ratio relative to
/// face value (useful for transfer-fee accounting).
///
/// # Flags
///
/// | Flag                | Value       | Description                                   |
/// |---------------------|-------------|-----------------------------------------------|
/// | `tfSetfAuth`        | 0x00010000  | Authorize the other party to hold tokens.     |
/// | `tfSetNoRipple`     | 0x00020000  | Disable rippling on this trust line.          |
/// | `tfClearNoRipple`   | 0x00040000  | Re-enable rippling on this trust line.        |
/// | `tfSetFreeze`       | 0x00100000  | Freeze the trust line.                        |
/// | `tfClearFreeze`     | 0x00200000  | Unfreeze the trust line.                      |
///
/// # Examples
///
/// ```
/// use xrpl_models::transactions::trust_set::TrustSet;
///
/// let json = serde_json::json!({
///     "LimitAmount": {
///         "value": "1000000",
///         "currency": "USD",
///         "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
///     }
/// });
/// let trust: TrustSet = serde_json::from_value(json).unwrap();
/// assert_eq!(trust.limit_amount.currency.to_string(), "USD");
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/trustset>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustSet {
    /// The trust line limit: specifies the currency, issuer, and maximum
    /// amount the account is willing to owe the issuer.
    ///
    /// The `value` field of the [`IssuedAmount`] sets the limit. The
    /// `currency` and `issuer` fields identify the trust line. Setting the
    /// limit to zero with no outstanding balance removes the trust line.
    #[serde(rename = "LimitAmount")]
    pub limit_amount: IssuedAmount,

    /// Incoming quality ratio for balances on this trust line, as the
    /// ratio `quality_in / 1,000,000,000`.
    ///
    /// For example, a value of `500_000_000` means the account values
    /// incoming balances at 50% of face value. A value of `0` is
    /// equivalent to the default (`1_000_000_000`).
    #[serde(rename = "QualityIn", default, skip_serializing_if = "Option::is_none")]
    pub quality_in: Option<u32>,

    /// Outgoing quality ratio for balances on this trust line, as the
    /// ratio `quality_out / 1,000,000,000`.
    ///
    /// For example, a value of `500_000_000` means the account values
    /// outgoing balances at 50% of face value. A value of `0` is
    /// equivalent to the default (`1_000_000_000`).
    #[serde(rename = "QualityOut", default, skip_serializing_if = "Option::is_none")]
    pub quality_out: Option<u32>,
}

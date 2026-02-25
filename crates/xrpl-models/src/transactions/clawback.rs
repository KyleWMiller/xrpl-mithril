//! Clawback transaction types.
//!
//! This module defines the [`Clawback`] transaction, which allows an issuer to
//! reclaim issued tokens from a holder's trust line.

use serde::{Deserialize, Serialize};
use xrpl_types::Amount;

// ---------------------------------------------------------------------------
// Clawback — TransactionType = 30
// ---------------------------------------------------------------------------

/// A Clawback transaction (TransactionType = 30).
///
/// Allows a token issuer to claw back (reclaim) tokens they have issued from
/// a holder's trust line. The issuer must have enabled the `asfAllowClawback`
/// flag on their account via [`AccountSet`](super::account::AccountSet)
/// before issuing any tokens.
///
/// The `amount` field specifies both the token to claw back and the quantity.
/// The `issuer` field within the amount must be the holder (not the issuing
/// account, which is the transaction sender).
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/clawback>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clawback {
    /// The amount of currency to claw back.
    ///
    /// The `issuer` sub-field identifies the holder from whom the tokens are
    /// clawed back. The transaction sender must be the original token issuer.
    #[serde(rename = "Amount")]
    pub amount: Amount,
}

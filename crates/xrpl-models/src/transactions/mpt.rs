//! Multi-Purpose Token (MPT) transaction types.
//!
//! MPTs are a token primitive on the XRPL that provide a simpler alternative
//! to trust lines for representing fungible assets. This module defines the
//! four transaction types for creating, destroying, configuring, and
//! authorizing MPT issuances and holdings.
//!
//! # XRPL Documentation
//!
//! - <https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate>
//! - <https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancedestroy>
//! - <https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuanceset>
//! - <https://xrpl.org/docs/references/protocol/transactions/types/mptokenauthorize>

use serde::{Deserialize, Serialize};
use xrpl_types::currency::MptIssuanceId;
use xrpl_types::{AccountId, Blob};

// ---------------------------------------------------------------------------
// MPTokenIssuanceCreate — TransactionType = 54
// ---------------------------------------------------------------------------

/// An MPTokenIssuanceCreate transaction (TransactionType = 54).
///
/// Creates a new Multi-Purpose Token issuance on the ledger. The issuer is
/// the account that submits this transaction. Once created, individual
/// accounts can hold balances of this MPT by submitting an
/// [`MPTokenAuthorize`] transaction.
///
/// # Flags
///
/// - `tfMPTCanLock` (0x0002) — Issuer can lock individual balances.
/// - `tfMPTRequireAuth` (0x0004) — Holders must be authorized by the issuer.
/// - `tfMPTCanEscrow` (0x0008) — Tokens may be placed in escrow.
/// - `tfMPTCanTrade` (0x0010) — Tokens may be traded on the DEX.
/// - `tfMPTCanTransfer` (0x0020) — Tokens may be transferred between accounts.
/// - `tfMPTCanClawback` (0x0040) — Issuer can claw back tokens.
///
/// # Examples
///
/// ```
/// use xrpl_models::transactions::mpt::MPTokenIssuanceCreate;
///
/// let json = serde_json::json!({
///     "MaximumAmount": 1000000,
///     "AssetScale": 2,
///     "TransferFee": 100
/// });
/// let mpt: MPTokenIssuanceCreate = serde_json::from_value(json).unwrap();
/// assert_eq!(mpt.max_amount, Some(1000000));
/// assert_eq!(mpt.asset_scale, Some(2));
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenIssuanceCreate {
    /// The maximum number of tokens that can ever be issued for this MPT,
    /// expressed as a non-negative integer.
    ///
    /// If omitted, there is no maximum supply.
    #[serde(
        rename = "MaximumAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_amount: Option<i64>,

    /// The number of decimal places used by this token.
    ///
    /// For example, a value of `2` means the smallest unit is 0.01. Valid
    /// range is 0 to 10. If omitted, defaults to 0.
    #[serde(
        rename = "AssetScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_scale: Option<u8>,

    /// The fee charged on transfers between non-issuer accounts, in basis
    /// points (1/100th of a percent).
    ///
    /// Valid range is 0 to 50000 (0% to 50%). If omitted, no transfer fee
    /// is applied.
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u16>,

    /// Arbitrary metadata for this token issuance, encoded as a hex blob.
    ///
    /// Maximum length is 1024 bytes.
    #[serde(
        rename = "MPTokenMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub metadata: Option<Blob>,
}

// ---------------------------------------------------------------------------
// MPTokenIssuanceDestroy — TransactionType = 55
// ---------------------------------------------------------------------------

/// An MPTokenIssuanceDestroy transaction (TransactionType = 55).
///
/// Permanently removes an MPT issuance from the ledger. This can only
/// succeed if no tokens are currently outstanding (all holders have
/// returned or been clawed back their balances).
///
/// Only the original issuer can submit this transaction.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancedestroy>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenIssuanceDestroy {
    /// The identifier of the MPT issuance to destroy.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: MptIssuanceId,
}

// ---------------------------------------------------------------------------
// MPTokenIssuanceSet — TransactionType = 56
// ---------------------------------------------------------------------------

/// An MPTokenIssuanceSet transaction (TransactionType = 56).
///
/// Modifies properties of an existing MPT issuance. The issuer can use this
/// to lock or unlock individual holder balances, or to authorize or
/// unauthorize specific holders (when `tfMPTRequireAuth` was set on the
/// issuance).
///
/// # Flags
///
/// - `tfMPTLock` (0x0001) — Lock the specified holder's MPT balance.
/// - `tfMPTUnlock` (0x0002) — Unlock the specified holder's MPT balance.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuanceset>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenIssuanceSet {
    /// The identifier of the MPT issuance to modify.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: MptIssuanceId,

    /// The account whose holding of this MPT should be affected.
    ///
    /// Required when locking/unlocking a specific holder's balance.
    /// If omitted, the operation applies to the issuance itself.
    #[serde(rename = "Holder", default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// MPTokenAuthorize — TransactionType = 57
// ---------------------------------------------------------------------------

/// An MPTokenAuthorize transaction (TransactionType = 57).
///
/// Used in two contexts:
///
/// 1. **Holder opts in**: A non-issuer account submits this to create an
///    `MPToken` ledger entry, indicating willingness to hold the token.
///    This is required before the account can receive any balance.
///
/// 2. **Issuer authorizes holder**: When `tfMPTRequireAuth` is set on the
///    issuance, the issuer submits this with `holder` set to the account
///    being authorized.
///
/// # Flags
///
/// - `tfMPTUnauthorize` (0x0001) — When submitted by a holder, this
///   removes their `MPToken` object (opt-out). Balance must be zero.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/mptokenauthorize>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenAuthorize {
    /// The identifier of the MPT issuance to authorize for.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: MptIssuanceId,

    /// The account to authorize.
    ///
    /// Required when the issuer is authorizing a specific holder. Omitted
    /// when a holder is opting in (or out) on their own behalf.
    #[serde(rename = "Holder", default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<AccountId>,
}

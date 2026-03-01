//! Automated Market Maker (AMM) transaction types.
//!
//! This module defines all transaction types for interacting with the XRPL's
//! built-in Automated Market Maker: [`AMMCreate`], [`AMMDeposit`],
//! [`AMMWithdraw`], [`AMMVote`], [`AMMBid`], [`AMMDelete`], and
//! [`AMMClawback`].
//!
//! AMM pools hold two assets and issue LP tokens that represent proportional
//! ownership of the pool. The trading fee is set at creation and can be
//! adjusted via governance voting by LP token holders.

use serde::{Deserialize, Serialize};
use xrpl_types::currency::Issue;
use xrpl_types::{AccountId, Amount};

// ---------------------------------------------------------------------------
// AuthAccount — helper type for AMMBid
// ---------------------------------------------------------------------------

/// An account authorized to trade at a discounted fee in an AMM auction slot.
///
/// Used within [`AMMBid::auth_accounts`] to specify up to 4 accounts that
/// the slot holder authorizes to trade at the discounted fee.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthAccount {
    /// The authorized account's address.
    #[serde(rename = "Account")]
    pub account: AccountId,
}

impl crate::serde_helpers::StArrayElement for AuthAccount {
    const WRAPPER_KEY: &'static str = "AuthAccount";
}

// ---------------------------------------------------------------------------
// AMMCreate — TransactionType = 35
// ---------------------------------------------------------------------------

/// An AMMCreate transaction (TransactionType = 35).
///
/// Creates a new Automated Market Maker instance for a pair of assets. The
/// sender deposits the initial liquidity (`amount` and `amount2`) and sets
/// the `trading_fee` charged on swaps.
///
/// The AMM instance is uniquely identified by its asset pair. Only one AMM
/// can exist for any given pair of assets on the ledger. The creator receives
/// the initial LP tokens proportional to their deposit.
///
/// The `trading_fee` is specified in units of 1/100,000 (i.e., a value of
/// 1000 represents 1%). The maximum allowed fee is 1000 (1%).
///
/// # Examples
///
/// ```
/// use xrpl_models::transactions::amm::AMMCreate;
///
/// let json = serde_json::json!({
///     "Amount": "10000000",
///     "Amount2": {
///         "value": "100",
///         "currency": "USD",
///         "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
///     },
///     "TradingFee": 500
/// });
/// let amm: AMMCreate = serde_json::from_value(json).unwrap();
/// assert_eq!(amm.trading_fee, 500);
/// ```
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammcreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMCreate {
    /// The first asset to deposit into the AMM pool.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The second asset to deposit into the AMM pool.
    #[serde(rename = "Amount2")]
    pub amount2: Amount,

    /// The fee to charge for trades against this AMM, in units of 1/100,000.
    ///
    /// A value of 1 means 0.001%. The maximum is 1000 (1%).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}

// ---------------------------------------------------------------------------
// AMMDeposit — TransactionType = 36
// ---------------------------------------------------------------------------

/// An AMMDeposit transaction (TransactionType = 36).
///
/// Deposits assets into an existing AMM pool. The depositor receives LP
/// tokens proportional to their share of the pool after the deposit.
///
/// Several deposit modes are supported depending on which optional fields
/// are provided:
///
/// - **Two-asset deposit**: Both `amount` and `amount2` are specified.
/// - **Single-asset deposit**: Only `amount` is specified.
/// - **LP token target**: Only `lp_token` is specified (deposit whatever is
///   needed to receive exactly that many LP tokens).
/// - **Single-asset with LP limit**: `amount` and `lp_token` are specified.
/// - **Single-asset with effective price**: `amount` and `e_price` are
///   specified.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammdeposit>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMDeposit {
    /// The first asset identifying the AMM pool.
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The second asset identifying the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,

    /// Amount of the first asset to deposit.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,

    /// Amount of the second asset to deposit.
    #[serde(rename = "Amount2", default, skip_serializing_if = "Option::is_none")]
    pub amount2: Option<Amount>,

    /// The maximum effective price the depositor is willing to pay.
    ///
    /// Used with single-asset deposits to limit slippage. The deposit fails
    /// if the effective price would exceed this value.
    #[serde(rename = "EPrice", default, skip_serializing_if = "Option::is_none")]
    pub e_price: Option<Amount>,

    /// The amount of LP tokens the depositor wants to receive, or the
    /// maximum LP tokens to consume (depending on the deposit mode).
    #[serde(rename = "LPToken", default, skip_serializing_if = "Option::is_none")]
    pub lp_token: Option<Amount>,
}

// ---------------------------------------------------------------------------
// AMMWithdraw — TransactionType = 37
// ---------------------------------------------------------------------------

/// An AMMWithdraw transaction (TransactionType = 37).
///
/// Withdraws assets from an AMM pool by redeeming LP tokens. Several
/// withdrawal modes are supported depending on which optional fields
/// are provided:
///
/// - **Two-asset withdrawal**: Both `amount` and `amount2` are specified.
/// - **Single-asset withdrawal**: Only `amount` is specified.
/// - **LP token withdrawal**: Only `lp_token` is specified (burn exactly
///   that many LP tokens and receive proportional assets).
/// - **Single-asset with LP limit**: `amount` and `lp_token` are specified.
/// - **Single-asset with effective price**: `amount` and `e_price` are
///   specified.
/// - **Withdraw all**: No amount fields specified; burns all LP tokens.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammwithdraw>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMWithdraw {
    /// The first asset identifying the AMM pool.
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The second asset identifying the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,

    /// Amount of the first asset to withdraw.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,

    /// Amount of the second asset to withdraw.
    #[serde(rename = "Amount2", default, skip_serializing_if = "Option::is_none")]
    pub amount2: Option<Amount>,

    /// The minimum effective price the withdrawer is willing to accept.
    ///
    /// Used with single-asset withdrawals to limit slippage. The withdrawal
    /// fails if the effective price would be below this value.
    #[serde(rename = "EPrice", default, skip_serializing_if = "Option::is_none")]
    pub e_price: Option<Amount>,

    /// The amount of LP tokens to burn for the withdrawal.
    #[serde(rename = "LPToken", default, skip_serializing_if = "Option::is_none")]
    pub lp_token: Option<Amount>,
}

// ---------------------------------------------------------------------------
// AMMVote — TransactionType = 38
// ---------------------------------------------------------------------------

/// An AMMVote transaction (TransactionType = 38).
///
/// Votes on the trading fee for an AMM pool. LP token holders can vote to
/// adjust the pool's trading fee. The effective fee is a weighted average of
/// all votes, where each vote is weighted by the voter's LP token balance.
///
/// Up to 8 votes are tracked. If a new voter has more LP tokens than the
/// smallest existing voter, the new vote replaces the smallest.
///
/// The `trading_fee` is specified in units of 1/100,000 (i.e., a value of
/// 1000 represents 1%). The maximum allowed fee is 1000 (1%).
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammvote>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMVote {
    /// The first asset identifying the AMM pool.
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The second asset identifying the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,

    /// The proposed trading fee, in units of 1/100,000.
    ///
    /// A value of 1 means 0.001%. The maximum is 1000 (1%).
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}

// ---------------------------------------------------------------------------
// AMMBid — TransactionType = 39
// ---------------------------------------------------------------------------

/// An AMMBid transaction (TransactionType = 39).
///
/// Bids on the AMM's auction slot. The auction slot grants the holder (and
/// optionally up to 4 authorized accounts) a discounted trading fee for a
/// 24-hour period. The slot is won by bidding LP tokens.
///
/// If `bid_min` is provided, the transaction fails if the minimum bid price
/// would be below that amount. If `bid_max` is provided, the transaction
/// fails if the cost exceeds that amount.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammbid>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMBid {
    /// The first asset identifying the AMM pool.
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The second asset identifying the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,

    /// Minimum price the bidder is willing to pay for the slot (in LP tokens).
    ///
    /// If the winning bid would be below this amount, the transaction fails.
    #[serde(rename = "BidMin", default, skip_serializing_if = "Option::is_none")]
    pub bid_min: Option<Amount>,

    /// Maximum price the bidder is willing to pay for the slot (in LP tokens).
    ///
    /// If the cost exceeds this amount, the transaction fails.
    #[serde(rename = "BidMax", default, skip_serializing_if = "Option::is_none")]
    pub bid_max: Option<Amount>,

    /// Up to 4 additional accounts authorized to trade at the discounted fee
    /// for the duration of the auction slot.
    #[serde(rename = "AuthAccounts", default, skip_serializing_if = "Option::is_none")]
    pub auth_accounts: Option<crate::serde_helpers::StArray<AuthAccount>>,
}

// ---------------------------------------------------------------------------
// AMMDelete — TransactionType = 40
// ---------------------------------------------------------------------------

/// An AMMDelete transaction (TransactionType = 40).
///
/// Deletes an empty AMM instance from the ledger. An AMM can only be deleted
/// when it holds no assets and has no outstanding LP tokens. This typically
/// happens after all liquidity has been withdrawn.
///
/// This transaction exists to reclaim the reserve locked by the AMM's ledger
/// entries. Anyone can submit it once the AMM is empty.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammdelete>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMDelete {
    /// The first asset identifying the AMM pool.
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The second asset identifying the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,
}

// ---------------------------------------------------------------------------
// AMMClawback — TransactionType = 31
// ---------------------------------------------------------------------------

/// An AMMClawback transaction (TransactionType = 31).
///
/// Allows a token issuer to claw back their issued tokens from an AMM pool.
/// This is used when the issuer has enabled clawback on their token and needs
/// to recover tokens held by a specific account within the AMM.
///
/// The `holder` is the account whose position in the AMM is being clawed
/// back. If `amount` is provided, the clawback is limited to that quantity;
/// otherwise the entire position is clawed back.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ammclawback>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMMClawback {
    /// The account whose tokens are being clawed back from the AMM.
    #[serde(rename = "Holder")]
    pub holder: AccountId,

    /// The first asset identifying the AMM pool (the clawback-enabled token).
    #[serde(rename = "Asset")]
    pub asset: Issue,

    /// The second asset identifying the AMM pool.
    #[serde(rename = "Asset2")]
    pub asset2: Issue,

    /// The maximum amount to claw back. If omitted, the entire position is
    /// clawed back.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
}

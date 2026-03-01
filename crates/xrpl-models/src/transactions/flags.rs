//! Transaction flag definitions.
//!
//! Each transaction type may have specific flags that modify its behavior.
//! These are set in the `Flags` field (a `UInt32`).
//!
//! # Examples
//!
//! ```
//! use xrpl_models::transactions::flags::{PaymentFlags, OfferCreateFlags};
//!
//! // Combine flags with bitwise OR
//! let flags = PaymentFlags::PARTIAL_PAYMENT | PaymentFlags::LIMIT_QUALITY;
//! assert!(flags.contains(PaymentFlags::PARTIAL_PAYMENT));
//!
//! // Convert to the raw u32 for the transaction's Flags field
//! let raw: u32 = flags.bits();
//! assert_eq!(raw, 0x0002_0000 | 0x0004_0000);
//!
//! // Check individual offer flags
//! let offer_flags = OfferCreateFlags::IMMEDIATE_OR_CANCEL;
//! assert!(offer_flags.contains(OfferCreateFlags::IMMEDIATE_OR_CANCEL));
//! assert!(!offer_flags.contains(OfferCreateFlags::FILL_OR_KILL));
//! ```

use bitflags::bitflags;

bitflags! {
    /// Flags for [`Payment`] transactions.
    ///
    /// These flags control path-finding behavior and partial payment semantics.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PaymentFlags: u32 {
        /// Do not use the default path; only use paths included in the
        /// `Paths` field. This is intended to force the transaction to take
        /// arbitrage opportunities. Most clients do not need this.
        /// (tfNoRippleDirect)
        const NO_RIPPLE_DIRECT = 0x0001_0000;

        /// If the specified `Amount` cannot be sent without spending more
        /// than `SendMax`, reduce the received amount instead of failing
        /// outright. See [Partial Payments](https://xrpl.org/partial-payments.html).
        /// (tfPartialPayment)
        const PARTIAL_PAYMENT = 0x0002_0000;

        /// Only take paths where all the conversions have an input:output
        /// ratio that is equal or better than the ratio of
        /// `Amount`:`SendMax`. (tfLimitQuality)
        const LIMIT_QUALITY = 0x0004_0000;
    }
}

bitflags! {
    /// Flags for [`TrustSet`] transactions.
    ///
    /// These flags control authorization, rippling, and freeze states on
    /// trust lines between two accounts.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TrustSetFlags: u32 {
        /// Authorize the other party to hold currency issued by this
        /// account. Only valid when the issuer has enabled Authorized Trust
        /// Lines. (tfSetfAuth)
        const SET_AUTH = 0x0001_0000;

        /// Enable the No Ripple flag on this trust line. (tfSetNoRipple)
        const SET_NO_RIPPLE = 0x0002_0000;

        /// Disable the No Ripple flag on this trust line. (tfClearNoRipple)
        const CLEAR_NO_RIPPLE = 0x0004_0000;

        /// Freeze the trust line. (tfSetFreeze)
        const SET_FREEZE = 0x0010_0000;

        /// Unfreeze the trust line. (tfClearFreeze)
        const CLEAR_FREEZE = 0x0020_0000;

        /// Deep-freeze the trust line, preventing the counterparty from
        /// sending or receiving the issued currency through this trust line.
        /// Requires the Deep Freeze amendment. (tfSetDeepFreeze)
        const SET_DEEP_FREEZE = 0x0040_0000;

        /// Remove a deep freeze from the trust line. (tfClearDeepFreeze)
        const CLEAR_DEEP_FREEZE = 0x0080_0000;
    }
}

bitflags! {
    /// Flags for [`OfferCreate`] transactions.
    ///
    /// These flags control offer matching behavior on the decentralized
    /// exchange.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OfferCreateFlags: u32 {
        /// The offer does not consume offers that exactly match it; instead
        /// it becomes an Offer object in the ledger. It still consumes
        /// offers that cross it. (tfPassive)
        const PASSIVE = 0x0001_0000;

        /// Treat the offer as an Immediate or Cancel order. The offer never
        /// creates an Offer object in the ledger: it only trades as much
        /// as it can by consuming existing offers at the time the
        /// transaction is processed. (tfImmediateOrCancel)
        const IMMEDIATE_OR_CANCEL = 0x0002_0000;

        /// Treat the offer as a Fill or Kill order. The offer never creates
        /// an Offer object in the ledger and is cancelled if it cannot be
        /// fully filled at the time of execution. (tfFillOrKill)
        const FILL_OR_KILL = 0x0004_0000;

        /// Exchange the entire `TakerGets` amount, even if it means
        /// obtaining more than the `TakerPays` amount in exchange.
        /// (tfSell)
        const SELL = 0x0008_0000;
    }
}

bitflags! {
    /// Flags for [`NFTokenMint`] transactions.
    ///
    /// These flags control the properties of a newly minted NFToken.
    /// Unlike most transaction flags, NFTokenMint flags occupy the lower
    /// bits of the `Flags` field.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NFTokenMintFlags: u32 {
        /// The issuer (or an entity authorized by the issuer) can destroy
        /// the minted NFToken. Without this flag, the NFToken cannot be
        /// burned even by the issuer. (tfBurnable)
        const BURNABLE = 0x0000_0001;

        /// The minted NFToken can only be bought or sold for XRP.
        /// (tfOnlyXRP)
        const ONLY_XRP = 0x0000_0002;

        /// Automatically create trust lines to hold transfer fees received
        /// from transferring the NFToken. Without this flag, an attempt to
        /// transfer the NFToken to a holder who does not have the required
        /// trust line fails. (tfTrustLine)
        const TRUSTLINE = 0x0000_0004;

        /// The minted NFToken can be transferred to another holder. Without
        /// this flag, the token can only be transferred back to the issuer.
        /// (tfTransferable)
        const TRANSFERABLE = 0x0000_0008;
    }
}

bitflags! {
    /// Flags for [`NFTokenCreateOffer`] transactions.
    ///
    /// Determines whether the offer is a buy or sell offer.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NFTokenCreateOfferFlags: u32 {
        /// If set, indicates that the offer is a sell offer. Otherwise the
        /// offer is a buy offer. (tfSellNFToken)
        const SELL_NFTOKEN = 0x0000_0001;
    }
}

bitflags! {
    /// Flags for [`AMMDeposit`] transactions.
    ///
    /// These flags control the deposit mode for an Automated Market Maker
    /// pool. Exactly one mode flag must be set.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AMMDepositFlags: u32 {
        /// Deposit a specified amount of LP Tokens to receive both assets
        /// proportionally. (tfLPToken)
        const LP_TOKEN = 0x0001_0000;

        /// Deposit a single asset to the AMM pool. (tfSingleAsset)
        const SINGLE_ASSET = 0x0008_0000;

        /// Deposit both assets to the AMM pool in specified amounts.
        /// (tfTwoAsset)
        const TWO_ASSET = 0x0010_0000;

        /// Deposit a single asset and receive a specified amount of LP
        /// Tokens. (tfOneAssetLPToken)
        const ONE_ASSET_LP_TOKEN = 0x0020_0000;

        /// Deposit up to a specified amount of one asset to receive a
        /// target amount of LP Tokens. (tfLimitLPToken)
        const LIMIT_LP_TOKEN = 0x0040_0000;

        /// Deposit both assets to an empty AMM pool, setting the initial
        /// price. (tfTwoAssetIfEmpty)
        const TWO_ASSET_IF_EMPTY = 0x0080_0000;
    }
}

bitflags! {
    /// Flags for [`AMMWithdraw`] transactions.
    ///
    /// These flags control the withdrawal mode from an Automated Market
    /// Maker pool. Exactly one mode flag must be set.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AMMWithdrawFlags: u32 {
        /// Return a specified amount of LP Tokens to receive both assets
        /// proportionally. (tfLPToken)
        const LP_TOKEN = 0x0001_0000;

        /// Return all LP Tokens held by the account, receiving both assets
        /// proportionally. (tfWithdrawAll)
        const WITHDRAW_ALL = 0x0002_0000;

        /// Return all LP Tokens held by the account, receiving a single
        /// specified asset. (tfOneAssetWithdrawAll)
        const ONE_ASSET_WITHDRAW_ALL = 0x0004_0000;

        /// Withdraw a specified amount of a single asset from the pool.
        /// (tfSingleAsset)
        const SINGLE_ASSET = 0x0008_0000;

        /// Withdraw specified amounts of both assets from the pool.
        /// (tfTwoAsset)
        const TWO_ASSET = 0x0010_0000;

        /// Withdraw a single asset, receiving a specified amount, and
        /// return a calculated number of LP Tokens. (tfOneAssetLPToken)
        const ONE_ASSET_LP_TOKEN = 0x0020_0000;

        /// Withdraw up to a specified amount of one asset, returning no
        /// more than a specified amount of LP Tokens. (tfLimitLPToken)
        const LIMIT_LP_TOKEN = 0x0040_0000;
    }
}

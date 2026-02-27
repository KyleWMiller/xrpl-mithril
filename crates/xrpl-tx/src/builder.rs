//! Fluent transaction builders for common XRPL transaction types.
//!
//! Each builder produces an [`UnsignedTransaction<Transaction>`] ready for
//! autofill and signing.
//!
//! # Examples
//!
//! ```
//! use xrpl_tx::builder::PaymentBuilder;
//! use xrpl_types::{AccountId, Amount, XrpAmount};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let unsigned = PaymentBuilder::new()
//!     .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?)
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
//!     .build()?;
//! # Ok(())
//! # }
//! ```

use xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_models::transactions::{
    self, Memo, Transaction, TransactionCommon,
};
use xrpl_types::{AccountId, Amount, Hash256, IssuedAmount, XrpAmount};

use crate::error::TxError;

// ---------------------------------------------------------------------------
// Common builder helpers
// ---------------------------------------------------------------------------

/// Shared state for common transaction fields across all builders.
#[derive(Debug, Clone, Default)]
struct CommonBuilder {
    account: Option<AccountId>,
    fee: Option<Amount>,
    sequence: Option<u32>,
    flags: Option<u32>,
    last_ledger_sequence: Option<u32>,
    memos: Option<Vec<Memo>>,
    network_id: Option<u32>,
    source_tag: Option<u32>,
    ticket_sequence: Option<u32>,
}

impl CommonBuilder {
    fn build(self) -> Result<TransactionCommon, TxError> {
        let account = self
            .account
            .ok_or_else(|| TxError::Validation("account is required".into()))?;

        // Default fee to 0 drops — autofill will set the real value
        let fee = self
            .fee
            .unwrap_or(Amount::Xrp(XrpAmount::ZERO));

        Ok(TransactionCommon {
            account,
            fee,
            sequence: self.sequence.unwrap_or(0),
            flags: self.flags,
            last_ledger_sequence: self.last_ledger_sequence,
            account_txn_id: None,
            memos: self.memos,
            network_id: self.network_id,
            source_tag: self.source_tag,
            signing_pub_key: None,
            txn_signature: None,
            ticket_sequence: self.ticket_sequence,
            signers: None,
        })
    }
}

macro_rules! impl_common_setters {
    ($builder:ident) => {
        impl $builder {
            /// Set the sending account.
            #[must_use]
            pub fn account(mut self, account: AccountId) -> Self {
                self.common.account = Some(account);
                self
            }

            /// Set the transaction fee in drops. If not set, autofill will populate it.
            #[must_use]
            pub fn fee(mut self, fee: Amount) -> Self {
                self.common.fee = Some(fee);
                self
            }

            /// Set the account sequence number. If not set, autofill will populate it.
            #[must_use]
            pub fn sequence(mut self, sequence: u32) -> Self {
                self.common.sequence = Some(sequence);
                self
            }

            /// Set transaction flags.
            #[must_use]
            pub fn flags(mut self, flags: u32) -> Self {
                self.common.flags = Some(flags);
                self
            }

            /// Set the last ledger sequence. If not set, autofill will populate it.
            #[must_use]
            pub fn last_ledger_sequence(mut self, seq: u32) -> Self {
                self.common.last_ledger_sequence = Some(seq);
                self
            }

            /// Attach memos to the transaction.
            #[must_use]
            pub fn memos(mut self, memos: Vec<Memo>) -> Self {
                self.common.memos = Some(memos);
                self
            }

            /// Set the network ID (for non-mainnet chains).
            #[must_use]
            pub fn network_id(mut self, id: u32) -> Self {
                self.common.network_id = Some(id);
                self
            }

            /// Set the source tag.
            #[must_use]
            pub fn source_tag(mut self, tag: u32) -> Self {
                self.common.source_tag = Some(tag);
                self
            }

            /// Use a ticket instead of a sequence number.
            #[must_use]
            pub fn ticket_sequence(mut self, ticket: u32) -> Self {
                self.common.ticket_sequence = Some(ticket);
                self
            }
        }
    };
}

// ---------------------------------------------------------------------------
// PaymentBuilder
// ---------------------------------------------------------------------------

/// Builder for Payment transactions.
#[derive(Debug, Clone, Default)]
pub struct PaymentBuilder {
    common: CommonBuilder,
    destination: Option<AccountId>,
    amount: Option<Amount>,
    send_max: Option<Amount>,
    deliver_min: Option<Amount>,
    destination_tag: Option<u32>,
    invoice_id: Option<Hash256>,
    paths: Option<Vec<Vec<transactions::payment::PathStep>>>,
}

impl_common_setters!(PaymentBuilder);

impl PaymentBuilder {
    /// Create a new Payment builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the destination account.
    #[must_use]
    pub fn destination(mut self, dest: AccountId) -> Self {
        self.destination = Some(dest);
        self
    }

    /// Set the payment amount.
    #[must_use]
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the maximum source amount (for cross-currency payments).
    #[must_use]
    pub fn send_max(mut self, amount: Amount) -> Self {
        self.send_max = Some(amount);
        self
    }

    /// Set the minimum delivered amount (for partial payments).
    #[must_use]
    pub fn deliver_min(mut self, amount: Amount) -> Self {
        self.deliver_min = Some(amount);
        self
    }

    /// Set the destination tag.
    #[must_use]
    pub fn destination_tag(mut self, tag: u32) -> Self {
        self.destination_tag = Some(tag);
        self
    }

    /// Set the invoice ID.
    #[must_use]
    pub fn invoice_id(mut self, id: Hash256) -> Self {
        self.invoice_id = Some(id);
        self
    }

    /// Set payment paths.
    #[must_use]
    pub fn paths(mut self, paths: Vec<Vec<transactions::payment::PathStep>>) -> Self {
        self.paths = Some(paths);
        self
    }

    /// Build the unsigned Payment transaction.
    ///
    /// # Errors
    ///
    /// Returns [`TxError::Validation`] if required fields are missing.
    pub fn build(self) -> Result<UnsignedTransaction<Transaction>, TxError> {
        let common = self.common.build()?;
        let destination = self
            .destination
            .ok_or_else(|| TxError::Validation("destination is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| TxError::Validation("amount is required".into()))?;

        let fields = transactions::payment::Payment {
            destination,
            amount,
            send_max: self.send_max,
            deliver_min: self.deliver_min,
            destination_tag: self.destination_tag,
            invoice_id: self.invoice_id,
            paths: self.paths,
        };

        let tx = Transaction::Payment { common, fields };
        Ok(UnsignedTransaction::new(tx))
    }
}

// ---------------------------------------------------------------------------
// TrustSetBuilder
// ---------------------------------------------------------------------------

/// Builder for TrustSet transactions.
#[derive(Debug, Clone, Default)]
pub struct TrustSetBuilder {
    common: CommonBuilder,
    limit_amount: Option<IssuedAmount>,
    quality_in: Option<u32>,
    quality_out: Option<u32>,
}

impl_common_setters!(TrustSetBuilder);

impl TrustSetBuilder {
    /// Create a new TrustSet builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the trust line limit amount (currency + issuer + limit).
    #[must_use]
    pub fn limit_amount(mut self, amount: IssuedAmount) -> Self {
        self.limit_amount = Some(amount);
        self
    }

    /// Set the inbound quality rate.
    #[must_use]
    pub fn quality_in(mut self, quality: u32) -> Self {
        self.quality_in = Some(quality);
        self
    }

    /// Set the outbound quality rate.
    #[must_use]
    pub fn quality_out(mut self, quality: u32) -> Self {
        self.quality_out = Some(quality);
        self
    }

    /// Build the unsigned TrustSet transaction.
    ///
    /// # Errors
    ///
    /// Returns [`TxError::Validation`] if required fields are missing.
    pub fn build(self) -> Result<UnsignedTransaction<Transaction>, TxError> {
        let common = self.common.build()?;
        let limit_amount = self
            .limit_amount
            .ok_or_else(|| TxError::Validation("limit_amount is required".into()))?;

        let fields = transactions::trust_set::TrustSet {
            limit_amount,
            quality_in: self.quality_in,
            quality_out: self.quality_out,
        };

        let tx = Transaction::TrustSet { common, fields };
        Ok(UnsignedTransaction::new(tx))
    }
}

// ---------------------------------------------------------------------------
// OfferCreateBuilder
// ---------------------------------------------------------------------------

/// Builder for OfferCreate transactions.
#[derive(Debug, Clone, Default)]
pub struct OfferCreateBuilder {
    common: CommonBuilder,
    taker_pays: Option<Amount>,
    taker_gets: Option<Amount>,
    expiration: Option<u32>,
    offer_sequence: Option<u32>,
}

impl_common_setters!(OfferCreateBuilder);

impl OfferCreateBuilder {
    /// Create a new OfferCreate builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the amount the offer creator pays (what the taker gets).
    #[must_use]
    pub fn taker_pays(mut self, amount: Amount) -> Self {
        self.taker_pays = Some(amount);
        self
    }

    /// Set the amount the offer creator gets (what the taker pays).
    #[must_use]
    pub fn taker_gets(mut self, amount: Amount) -> Self {
        self.taker_gets = Some(amount);
        self
    }

    /// Set the offer expiration time.
    #[must_use]
    pub fn expiration(mut self, exp: u32) -> Self {
        self.expiration = Some(exp);
        self
    }

    /// Replace a previous offer by sequence number.
    #[must_use]
    pub fn offer_sequence(mut self, seq: u32) -> Self {
        self.offer_sequence = Some(seq);
        self
    }

    /// Build the unsigned OfferCreate transaction.
    ///
    /// # Errors
    ///
    /// Returns [`TxError::Validation`] if required fields are missing.
    pub fn build(self) -> Result<UnsignedTransaction<Transaction>, TxError> {
        let common = self.common.build()?;
        let taker_pays = self
            .taker_pays
            .ok_or_else(|| TxError::Validation("taker_pays is required".into()))?;
        let taker_gets = self
            .taker_gets
            .ok_or_else(|| TxError::Validation("taker_gets is required".into()))?;

        let fields = transactions::offer::OfferCreate {
            taker_pays,
            taker_gets,
            expiration: self.expiration,
            offer_sequence: self.offer_sequence,
        };

        let tx = Transaction::OfferCreate { common, fields };
        Ok(UnsignedTransaction::new(tx))
    }
}

// ---------------------------------------------------------------------------
// EscrowCreateBuilder
// ---------------------------------------------------------------------------

/// Builder for EscrowCreate transactions.
#[derive(Debug, Clone, Default)]
pub struct EscrowCreateBuilder {
    common: CommonBuilder,
    destination: Option<AccountId>,
    amount: Option<Amount>,
    finish_after: Option<u32>,
    cancel_after: Option<u32>,
    condition: Option<xrpl_types::Blob>,
    destination_tag: Option<u32>,
}

impl_common_setters!(EscrowCreateBuilder);

impl EscrowCreateBuilder {
    /// Create a new EscrowCreate builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the destination account.
    #[must_use]
    pub fn destination(mut self, dest: AccountId) -> Self {
        self.destination = Some(dest);
        self
    }

    /// Set the escrowed amount.
    #[must_use]
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the finish-after time (Ripple epoch seconds).
    #[must_use]
    pub fn finish_after(mut self, time: u32) -> Self {
        self.finish_after = Some(time);
        self
    }

    /// Set the cancel-after time (Ripple epoch seconds).
    #[must_use]
    pub fn cancel_after(mut self, time: u32) -> Self {
        self.cancel_after = Some(time);
        self
    }

    /// Set the crypto-condition for release.
    #[must_use]
    pub fn condition(mut self, condition: xrpl_types::Blob) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Set the destination tag.
    #[must_use]
    pub fn destination_tag(mut self, tag: u32) -> Self {
        self.destination_tag = Some(tag);
        self
    }

    /// Build the unsigned EscrowCreate transaction.
    ///
    /// # Errors
    ///
    /// Returns [`TxError::Validation`] if required fields are missing.
    pub fn build(self) -> Result<UnsignedTransaction<Transaction>, TxError> {
        let common = self.common.build()?;
        let destination = self
            .destination
            .ok_or_else(|| TxError::Validation("destination is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| TxError::Validation("amount is required".into()))?;

        let fields = transactions::escrow::EscrowCreate {
            destination,
            amount,
            finish_after: self.finish_after,
            cancel_after: self.cancel_after,
            condition: self.condition,
            destination_tag: self.destination_tag,
        };

        let tx = Transaction::EscrowCreate { common, fields };
        Ok(UnsignedTransaction::new(tx))
    }
}

// ---------------------------------------------------------------------------
// AMMCreateBuilder
// ---------------------------------------------------------------------------

/// Builder for AMMCreate transactions.
#[derive(Debug, Clone, Default)]
pub struct AmmCreateBuilder {
    common: CommonBuilder,
    amount: Option<Amount>,
    amount2: Option<Amount>,
    trading_fee: Option<u16>,
}

impl_common_setters!(AmmCreateBuilder);

impl AmmCreateBuilder {
    /// Create a new AMMCreate builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the first asset amount.
    #[must_use]
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the second asset amount.
    #[must_use]
    pub fn amount2(mut self, amount: Amount) -> Self {
        self.amount2 = Some(amount);
        self
    }

    /// Set the trading fee in basis points (0-1000).
    #[must_use]
    pub fn trading_fee(mut self, fee: u16) -> Self {
        self.trading_fee = Some(fee);
        self
    }

    /// Build the unsigned AMMCreate transaction.
    ///
    /// # Errors
    ///
    /// Returns [`TxError::Validation`] if required fields are missing.
    pub fn build(self) -> Result<UnsignedTransaction<Transaction>, TxError> {
        let common = self.common.build()?;
        let amount = self
            .amount
            .ok_or_else(|| TxError::Validation("amount is required".into()))?;
        let amount2 = self
            .amount2
            .ok_or_else(|| TxError::Validation("amount2 is required".into()))?;
        let trading_fee = self
            .trading_fee
            .ok_or_else(|| TxError::Validation("trading_fee is required".into()))?;

        let fields = transactions::amm::AMMCreate {
            amount,
            amount2,
            trading_fee,
        };

        let tx = Transaction::AMMCreate { common, fields };
        Ok(UnsignedTransaction::new(tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_builder_basic() {
        let result = PaymentBuilder::new()
            .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().expect("addr"))
            .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse().expect("addr"))
            .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("drops")))
            .sequence(1)
            .build();
        assert!(result.is_ok());

        let unsigned = result.expect("built");
        assert_eq!(unsigned.common().sequence, 1);
        assert_eq!(unsigned.inner().transaction_type(), "Payment");
    }

    #[test]
    fn payment_builder_missing_destination() {
        let result = PaymentBuilder::new()
            .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().expect("addr"))
            .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("drops")))
            .build();
        assert!(matches!(result, Err(TxError::Validation(_))));
    }

    #[test]
    fn payment_builder_missing_account() {
        let result = PaymentBuilder::new()
            .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse().expect("addr"))
            .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("drops")))
            .build();
        assert!(matches!(result, Err(TxError::Validation(_))));
    }

    #[test]
    fn offer_create_builder() {
        let result = OfferCreateBuilder::new()
            .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().expect("addr"))
            .taker_pays(Amount::Xrp(XrpAmount::from_drops(5_000_000).expect("drops")))
            .taker_gets(Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("drops")))
            .build();
        assert!(result.is_ok());
    }
}

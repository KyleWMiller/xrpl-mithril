//! Transaction building, autofilling, and submission for the XRPL.
//!
//! This crate provides:
//! - Fluent transaction builders
//! - Fee, sequence, and `LastLedgerSequence` autofill
//! - Typed signing via [`UnsignedTransaction`] / [`TypedSignedTransaction`]
//! - Submit-and-wait with ledger tracking
//! - Reliable submission with retry on transient failures

#![forbid(unsafe_code)]

pub mod autofill;
pub mod builder;
pub mod error;
pub mod reliable;
pub mod submit;

pub use error::TxError;
pub use reliable::{sign_transaction, submit_transaction};
pub use submit::{submit_and_wait, TransactionResult};

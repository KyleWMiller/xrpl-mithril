//! Transaction building, autofilling, and submission for the XRPL.
//!
//! This crate provides:
//! - Fluent transaction builder
//! - Fee, sequence, and `LastLedgerSequence` autofill
//! - Submit-and-wait with ledger tracking
//! - Reliable submission with retry on transient failures

#![forbid(unsafe_code)]

// Modules will be added in Phase 3
// pub mod builder;
// pub mod autofill;
// pub mod submit;
// pub mod reliable;

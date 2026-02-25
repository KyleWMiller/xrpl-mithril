//! Transaction types, ledger objects, and request/response types for the XRP Ledger.
//!
//! This crate defines the data structures for all XRPL protocol objects:
//! - Transaction types (Payment, OfferCreate, TrustSet, etc.)
//! - Ledger entry types (AccountRoot, RippleState, Offer, etc.) — Phase 1
//! - JSON-RPC request/response types — Phase 3

#![forbid(unsafe_code)]

pub mod transactions;

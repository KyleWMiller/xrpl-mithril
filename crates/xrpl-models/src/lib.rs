//! Transaction types, ledger objects, and request/response types for the XRP Ledger.
//!
//! This crate defines the data structures for all XRPL protocol objects:
//! - Transaction types (Payment, OfferCreate, TrustSet, etc.)
//! - Ledger entry types (AccountRoot, RippleState, Offer, etc.)
//! - JSON-RPC request/response types for all public API methods

#![forbid(unsafe_code)]

pub mod ledger;
pub mod requests;
pub mod responses;
pub mod transactions;

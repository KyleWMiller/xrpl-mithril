//! Core protocol types for the XRP Ledger.
//!
//! This crate provides the fundamental types used throughout the XRPL protocol:
//! accounts, amounts, currency codes, hashes, timestamps, and variable-length blobs.
//!
//! All types use the newtype pattern to prevent type confusion at compile time.
//! Invalid states are unrepresentable — constructors validate inputs.

#![forbid(unsafe_code)]

pub mod account;
pub mod amount;
pub mod blob;
pub mod currency;
pub mod error;
pub mod hash;
pub mod serde_helpers;
pub mod timestamp;

pub use account::AccountId;
pub use amount::{Amount, IssuedAmount, IssuedValue, MptAmount, XrpAmount};
pub use blob::Blob;
pub use currency::{CurrencyCode, Issue, MptIssuanceId};
pub use error::TypeError;
pub use hash::{Hash128, Hash160, Hash192, Hash256, UInt384, UInt512, UInt96};
pub use timestamp::RippleTimestamp;

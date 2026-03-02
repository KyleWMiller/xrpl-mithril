#![doc(html_logo_url = "https://raw.githubusercontent.com/KyleWMiller/xrpl-mithril/main/assets/mithrilLogo.png")]
//! Core protocol types for the XRP Ledger.
//!
//! This crate provides the fundamental types used throughout the XRPL protocol:
//! accounts, amounts, currency codes, hashes, timestamps, and variable-length blobs.
//!
//! All types use the newtype pattern to prevent type confusion at compile time.
//! Invalid states are unrepresentable — constructors validate inputs.
//!
//! # Examples
//!
//! Parsing an account address and constructing an XRP amount:
//!
//! ```
//! use xrpl_types::{AccountId, XrpAmount};
//!
//! let account: AccountId = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().unwrap();
//! let amount = XrpAmount::from_drops(1_000_000).unwrap(); // 1 XRP
//!
//! assert_eq!(account.to_string(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
//! assert_eq!(amount.drops(), 1_000_000);
//! ```
//!
//! Building an issued currency amount:
//!
//! ```
//! use xrpl_types::{AccountId, IssuedAmount, IssuedValue, CurrencyCode};
//!
//! let issuer: AccountId = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().unwrap();
//! let value = IssuedValue::from_decimal_string("1.5").unwrap();
//! let currency = CurrencyCode::from_ascii("USD").unwrap();
//!
//! let amount = IssuedAmount { value, currency, issuer };
//! assert_eq!(amount.value.to_decimal_string(), "1.5");
//! ```
//!
//! Working with transaction and ledger hashes:
//!
//! ```
//! use xrpl_types::Hash256;
//!
//! let hash = Hash256::from_hex(
//!     "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
//! ).unwrap();
//! assert_eq!(hash.as_bytes().len(), 32);
//! ```

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

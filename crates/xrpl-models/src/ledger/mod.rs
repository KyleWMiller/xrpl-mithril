//! XRPL ledger entry types.
//!
//! Every object stored in the XRP Ledger's state tree has a type.
//! This module defines structs for each ledger entry type, matching
//! the JSON representations returned by methods like `account_info`,
//! `ledger_entry`, and `ledger_data`.
//!
//! Each struct derives `Serialize` and `Deserialize` with field-level
//! `#[serde(rename = "PascalCase")]` attributes to match the XRPL
//! JSON convention.

pub mod account_root;
pub mod amm;
pub mod check;
pub mod credential;
pub mod deposit_preauth;
pub mod did;
pub mod directory;
pub mod escrow;
pub mod fee_settings;
pub mod mpt;
pub mod nft_page;
pub mod offer;
pub mod oracle;
pub mod pay_channel;
pub mod ripple_state;
pub mod signer_list;

// Re-export all ledger entry types for convenience.
pub use account_root::AccountRoot;
pub use amm::Amm;
pub use check::Check;
pub use credential::Credential;
pub use deposit_preauth::DepositPreauth;
pub use did::Did;
pub use directory::DirectoryNode;
pub use escrow::Escrow;
pub use fee_settings::FeeSettings;
pub use mpt::{MpToken, MptIssuance};
pub use nft_page::{NfToken, NftTokenPage};
pub use offer::Offer;
pub use oracle::{Oracle, PriceData};
pub use pay_channel::PayChannel;
pub use ripple_state::RippleState;
pub use signer_list::{SignerEntry, SignerList};

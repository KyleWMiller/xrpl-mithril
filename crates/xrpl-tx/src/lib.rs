#![doc(html_logo_url = "https://raw.githubusercontent.com/KyleWMiller/xrpl-mithril/main/assets/logo.png")]
//! Transaction building, autofilling, and submission for the XRPL.
//!
//! This crate provides:
//! - Fluent transaction builders ([`builder`])
//! - Fee, sequence, and `LastLedgerSequence` autofill ([`autofill`])
//! - Typed signing via [`xrpl_models::transactions::wrapper::UnsignedTransaction`] /
//!   [`xrpl_models::transactions::wrapper::TypedSignedTransaction`]
//! - Submit-and-wait with ledger tracking ([`submit`])
//! - Reliable submission with retry on transient failures ([`reliable`])
//!
//! # Examples
//!
//! Full pipeline: build a payment, autofill network fields, sign, and submit.
//!
//! ```no_run
//! use xrpl_tx::builder::PaymentBuilder;
//! use xrpl_tx::reliable::submit_transaction;
//! use xrpl_client::JsonRpcClient;
//! use xrpl_wallet::{Wallet, Algorithm};
//! use xrpl_types::{Amount, XrpAmount};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to testnet
//! let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
//!
//! // Load a funded wallet
//! let wallet = Wallet::from_seed_encoded("sEdT7wHTCLzDG7Ue4312Kp4QA389Xmb")?;
//!
//! // Build a payment
//! let unsigned = PaymentBuilder::new()
//!     .account(*wallet.account_id())
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
//!     .build()?;
//!
//! // Autofill, sign, and submit in one call
//! let result = submit_transaction(&client, unsigned, &wallet).await?;
//! println!("Validated in ledger {}: {}", result.ledger_index, result.result_code);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod autofill;
pub mod builder;
pub mod error;
pub mod reliable;
pub mod submit;

pub use error::TxError;
pub use reliable::{sign_transaction, submit_transaction};
pub use submit::{submit_and_wait, TransactionResult};

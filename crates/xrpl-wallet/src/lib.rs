//! Key generation, signing, and address management for the XRPL.
//!
//! This crate provides:
//! - Key pair generation for secp256k1 (ECDSA) and Ed25519
//! - Seed encoding/decoding (sXXX format)
//! - Classic address and X-address conversion
//! - Transaction signing (single-sign and multi-sign)
//!
//! # Crypto Backends
//!
//! By default, this crate uses pure-Rust cryptography:
//! - `k256` for secp256k1 ECDSA
//! - `ed25519-dalek` for Ed25519
//!
//! An optional `native-crypto` feature enables C-backed `secp256k1` for
//! environments where performance is critical.
//!
//! # Quick Start
//!
//! ```
//! use xrpl_wallet::{Wallet, Algorithm};
//!
//! // Generate a random Ed25519 wallet
//! let wallet = Wallet::generate(Algorithm::Ed25519).unwrap();
//! println!("Address: {}", wallet.classic_address());
//! ```

#![forbid(unsafe_code)]

pub mod address;
pub mod algorithm;
pub mod error;
pub mod keypair;
pub mod seed;
pub mod signer;

// Re-exports for convenience
pub use algorithm::Algorithm;
pub use error::WalletError;
pub use keypair::{Keypair, Wallet};
pub use seed::Seed;
pub use signer::{sign, multi_sign, combine_signatures, SignedTransaction, Signer};

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

#![forbid(unsafe_code)]

// Modules will be added in Phase 2
// pub mod keypair;
// pub mod seed;
// pub mod address;
// pub mod signer;
// pub mod mnemonic;

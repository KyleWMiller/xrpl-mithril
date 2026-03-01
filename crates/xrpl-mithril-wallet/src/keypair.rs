//! Key pair and wallet types.
//!
//! A [`Keypair`] holds a private key, public key, and algorithm identifier.
//! A [`Wallet`] wraps a `Keypair` with a cached [`AccountId`] and classic
//! address for convenient use.
//!
//! # Examples
//!
//! ```
//! use xrpl_mithril_wallet::{Wallet, Algorithm};
//!
//! // Generate a random wallet
//! let wallet = Wallet::generate(Algorithm::Ed25519).unwrap();
//! println!("Address: {}", wallet.classic_address());
//! println!("Public key: {}", wallet.public_key_hex());
//! ```

use zeroize::{Zeroize, Zeroizing};

use xrpl_mithril_types::AccountId;

use crate::address::derive_account_id;
use crate::algorithm::{ed25519_ops, secp256k1_ops, Algorithm};
use crate::error::WalletError;
use crate::seed::Seed;

/// A cryptographic key pair for signing XRPL transactions.
///
/// Private key material is wrapped in [`Zeroizing`] to ensure it is zeroed
/// from memory when the keypair is dropped.
pub struct Keypair {
    algorithm: Algorithm,
    private_key: Zeroizing<[u8; 32]>,
    public_key: Vec<u8>, // 33 bytes: compressed secp256k1 or 0xED + ed25519
}

impl Keypair {
    /// Construct a keypair from raw components.
    ///
    /// # Errors
    ///
    /// Returns [`WalletError::InvalidSecretKey`] if the secret is not valid
    /// for the given algorithm.
    pub(crate) fn from_raw(
        algorithm: Algorithm,
        mut secret: [u8; 32],
        public_key: Vec<u8>,
    ) -> Result<Self, WalletError> {
        let kp = Self {
            algorithm,
            private_key: Zeroizing::new(secret),
            public_key,
        };
        // Zeroize the stack copy
        secret.zeroize();
        Ok(kp)
    }

    /// Returns the algorithm used by this key pair.
    #[must_use]
    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Returns the public key bytes.
    ///
    /// - secp256k1: 33-byte compressed SEC1 point
    /// - Ed25519: 33 bytes (`0xED` prefix + 32-byte key)
    #[must_use]
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Returns the public key as an uppercase hex string.
    #[must_use]
    pub fn public_key_hex(&self) -> String {
        hex::encode_upper(&self.public_key)
    }

    /// Derive the [`AccountId`] from this key pair's public key.
    #[must_use]
    pub fn account_id(&self) -> AccountId {
        derive_account_id(&self.public_key)
    }

    /// Returns the classic address (e.g., `rXXX...`) for this key pair.
    #[must_use]
    pub fn classic_address(&self) -> String {
        self.account_id().to_classic_address()
    }

    /// Sign a message according to the key's algorithm.
    ///
    /// - **secp256k1**: `message` must be a 32-byte pre-computed hash
    ///   (SHA-512/Half). Returns a DER-encoded ECDSA signature.
    /// - **Ed25519**: `message` is the raw signing data (prefix + serialized
    ///   transaction). Returns a 64-byte Ed25519 signature.
    ///
    /// # Errors
    ///
    /// Returns [`WalletError::SigningFailed`] if the signing operation fails.
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, WalletError> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let hash: &[u8; 32] = message
                    .try_into()
                    .map_err(|_| WalletError::SigningFailed(
                        format!("secp256k1 requires a 32-byte hash, got {} bytes", message.len()),
                    ))?;
                secp256k1_ops::sign(&self.private_key, hash)
            }
            Algorithm::Ed25519 => {
                Ok(ed25519_ops::sign(&self.private_key, message))
            }
        }
    }

    /// Verify a signature against a message using this key pair's public key.
    ///
    /// The same algorithm-specific rules as [`sign`](Self::sign) apply to the
    /// message format.
    ///
    /// # Errors
    ///
    /// Returns [`WalletError::VerificationFailed`] if the signature is invalid.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), WalletError> {
        match self.algorithm {
            Algorithm::Secp256k1 => {
                let hash: &[u8; 32] = message
                    .try_into()
                    .map_err(|_| WalletError::VerificationFailed(
                        format!("secp256k1 requires a 32-byte hash, got {} bytes", message.len()),
                    ))?;
                secp256k1_ops::verify(&self.public_key, hash, signature)
            }
            Algorithm::Ed25519 => {
                let raw_pubkey: &[u8; 32] = self.public_key[1..]
                    .try_into()
                    .map_err(|_| WalletError::VerificationFailed(
                        "Ed25519 public key must be 33 bytes (0xED prefix + 32)".into(),
                    ))?;
                ed25519_ops::verify(raw_pubkey, message, signature)
            }
        }
    }
}

/// A high-level XRPL wallet with cached account identity.
///
/// Wraps a [`Keypair`] and caches the derived [`AccountId`] and classic
/// address string. Use this as the primary entry point for key management.
pub struct Wallet {
    keypair: Keypair,
    account_id: AccountId,
    classic_address: String,
}

impl Wallet {
    /// Create a wallet from a [`Seed`] and [`Algorithm`].
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_mithril_wallet::{Wallet, Algorithm, Seed};
    ///
    /// let seed = Seed::from_passphrase("masterpassphrase");
    /// let wallet = Wallet::from_seed(&seed, Algorithm::Secp256k1).unwrap();
    /// assert_eq!(wallet.classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WalletError`] if key derivation fails.
    pub fn from_seed(seed: &Seed, algorithm: Algorithm) -> Result<Self, WalletError> {
        let keypair = seed.derive_keypair(algorithm)?;
        Ok(Self::from_keypair(keypair))
    }

    /// Create a wallet from an encoded seed string (e.g., `sXXX...`).
    ///
    /// Defaults to secp256k1. Use [`Wallet::from_seed`] if you need to
    /// specify the algorithm explicitly.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_mithril_wallet::Wallet;
    ///
    /// // Genesis account seed
    /// let wallet = Wallet::from_seed_encoded("snoPBrXtMeMyMHUVTgbuqAfg1SUTb").unwrap();
    /// assert_eq!(wallet.classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WalletError`] if the seed is invalid or derivation fails.
    pub fn from_seed_encoded(encoded: &str) -> Result<Self, WalletError> {
        let seed = Seed::from_encoded(encoded)?;
        Self::from_seed(&seed, Algorithm::Secp256k1)
    }

    /// Create a wallet from an encoded seed string with a specific algorithm.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_mithril_wallet::{Wallet, Algorithm};
    ///
    /// let wallet = Wallet::from_seed_encoded_with_algorithm(
    ///     "snoPBrXtMeMyMHUVTgbuqAfg1SUTb",
    ///     Algorithm::Secp256k1,
    /// ).unwrap();
    /// assert_eq!(wallet.classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WalletError`] if the seed is invalid or derivation fails.
    pub fn from_seed_encoded_with_algorithm(
        encoded: &str,
        algorithm: Algorithm,
    ) -> Result<Self, WalletError> {
        let seed = Seed::from_encoded(encoded)?;
        Self::from_seed(&seed, algorithm)
    }

    /// Generate a new wallet with a random seed.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_mithril_wallet::{Wallet, Algorithm};
    ///
    /// let wallet = Wallet::generate(Algorithm::Ed25519).unwrap();
    /// assert!(wallet.classic_address().starts_with('r'));
    /// assert_eq!(wallet.public_key()[0], 0xED); // Ed25519 prefix
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WalletError`] if key derivation fails (astronomically unlikely).
    pub fn generate(algorithm: Algorithm) -> Result<Self, WalletError> {
        let seed = Seed::random();
        Self::from_seed(&seed, algorithm)
    }

    /// Wrap an existing [`Keypair`] in a `Wallet`.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_mithril_wallet::{Wallet, Seed, Algorithm};
    ///
    /// let seed = Seed::from_passphrase("masterpassphrase");
    /// let keypair = seed.derive_keypair(Algorithm::Secp256k1).unwrap();
    /// let wallet = Wallet::from_keypair(keypair);
    /// assert_eq!(wallet.classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    /// ```
    #[must_use]
    pub fn from_keypair(keypair: Keypair) -> Self {
        let account_id = keypair.account_id();
        let classic_address = account_id.to_classic_address();
        Self {
            keypair,
            account_id,
            classic_address,
        }
    }

    /// Returns the wallet's [`AccountId`].
    #[must_use]
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    /// Returns the classic address string (e.g., `rHb9CJAWyB4...`).
    #[must_use]
    pub fn classic_address(&self) -> &str {
        &self.classic_address
    }

    /// Returns the public key bytes.
    #[must_use]
    pub fn public_key(&self) -> &[u8] {
        self.keypair.public_key()
    }

    /// Returns the public key as an uppercase hex string.
    #[must_use]
    pub fn public_key_hex(&self) -> String {
        self.keypair.public_key_hex()
    }

    /// Returns a reference to the underlying [`Keypair`].
    #[must_use]
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Returns the algorithm used by this wallet.
    #[must_use]
    pub fn algorithm(&self) -> Algorithm {
        self.keypair.algorithm()
    }
}

// Zeroize is handled by Keypair's Zeroizing<[u8; 32]> field.

impl core::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Wallet")
            .field("algorithm", &self.keypair.algorithm())
            .field("classic_address", &self.classic_address)
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Keypair")
            .field("algorithm", &self.algorithm)
            .field("public_key", &self.public_key_hex())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_from_genesis_seed() {
        let wallet =
            Wallet::from_seed_encoded("snoPBrXtMeMyMHUVTgbuqAfg1SUTb").expect("valid seed");
        assert_eq!(wallet.classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
        assert_eq!(wallet.algorithm(), Algorithm::Secp256k1);
    }

    #[test]
    fn wallet_generate_secp256k1() {
        let wallet = Wallet::generate(Algorithm::Secp256k1).expect("generate");
        assert_eq!(wallet.algorithm(), Algorithm::Secp256k1);
        assert!(wallet.classic_address().starts_with('r'));
        assert_eq!(wallet.public_key().len(), 33);
    }

    #[test]
    fn wallet_generate_ed25519() {
        let wallet = Wallet::generate(Algorithm::Ed25519).expect("generate");
        assert_eq!(wallet.algorithm(), Algorithm::Ed25519);
        assert!(wallet.classic_address().starts_with('r'));
        assert_eq!(wallet.public_key().len(), 33);
        assert_eq!(wallet.public_key()[0], 0xED);
    }

    #[test]
    fn wallet_debug_does_not_leak_private_key() {
        let wallet = Wallet::generate(Algorithm::Ed25519).expect("generate");
        let debug_str = format!("{wallet:?}");
        // Debug should show address but not private key material
        assert!(debug_str.contains("classic_address"));
        assert!(!debug_str.contains("private_key"));
    }

    #[test]
    fn keypair_sign_verify_secp256k1() {
        let seed = Seed::from_passphrase("test-keypair-sign");
        let kp = seed
            .derive_keypair(Algorithm::Secp256k1)
            .expect("derive");
        let hash = [0xAB; 32];
        let sig = kp.sign(&hash).expect("sign");
        kp.verify(&hash, &sig).expect("verify should pass");
    }

    #[test]
    fn keypair_sign_verify_ed25519() {
        let seed = Seed::from_passphrase("test-keypair-sign-ed25519");
        let kp = seed.derive_keypair(Algorithm::Ed25519).expect("derive");
        let message = b"hello XRPL";
        let sig = kp.sign(message).expect("sign");
        kp.verify(message, &sig).expect("verify should pass");
    }
}

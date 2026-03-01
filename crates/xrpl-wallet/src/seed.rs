//! XRPL seed encoding, decoding, and key derivation.
//!
//! A [`Seed`] is 16 bytes of entropy that deterministically produces a key pair.
//! Seeds are encoded as base58check strings starting with `s` (version byte `0x21`).
//!
//! # Key Derivation
//!
//! The derivation algorithm differs by key type:
//!
//! - **secp256k1**: Two-step derivation producing a root key pair and then an
//!   account key pair via scalar addition.
//! - **Ed25519**: Single-step derivation: `SHA-512(0xED || seed)` → first 32
//!   bytes → Ed25519 private key.
//!
//! # Examples
//!
//! ```
//! use xrpl_wallet::seed::Seed;
//! use xrpl_wallet::algorithm::Algorithm;
//!
//! // Generate a random seed
//! let seed = Seed::random();
//!
//! // Encode to sXXX format
//! let encoded = seed.encode();
//! assert!(encoded.starts_with('s'));
//!
//! // Decode back
//! let decoded = Seed::from_encoded(&encoded).unwrap();
//! assert_eq!(seed.as_bytes(), decoded.as_bytes());
//! ```

use sha2::{Digest, Sha512};
use zeroize::Zeroize;

use crate::algorithm::{ed25519_ops, secp256k1_ops, Algorithm};
use crate::error::WalletError;
use crate::keypair::Keypair;

/// Version byte for XRPL seed encoding (base58check).
const SEED_VERSION: u8 = 0x21;

/// Maximum number of iterations when searching for a valid secp256k1 key.
const MAX_SEQUENCE: u32 = 100;

/// A 16-byte XRPL seed.
///
/// Seeds are the root entropy from which key pairs are derived. They are
/// encoded as base58check strings with version byte `0x21` (producing strings
/// starting with `s`).
#[derive(Clone)]
pub struct Seed([u8; 16]);

impl Drop for Seed {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Seed {
    /// Generate a random seed using the operating system's CSPRNG.
    #[must_use]
    pub fn random() -> Self {
        let mut bytes = [0u8; 16];
        rand::fill(&mut bytes);
        Self(bytes)
    }

    /// Create a seed from raw bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Create a seed from a passphrase.
    ///
    /// Computes `SHA-512(passphrase)` and takes the first 16 bytes. This
    /// matches the behaviour of the XRPL `wallet_propose` command with a
    /// passphrase parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_wallet::Seed;
    ///
    /// let seed = Seed::from_passphrase("masterpassphrase");
    /// assert_eq!(seed.encode(), "snoPBrXtMeMyMHUVTgbuqAfg1SUTb");
    /// ```
    #[must_use]
    pub fn from_passphrase(passphrase: &str) -> Self {
        let hash = Sha512::digest(passphrase.as_bytes());
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&hash[..16]);
        Self(bytes)
    }

    /// Decode a seed from a base58check-encoded string (starting with `s`).
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_wallet::Seed;
    ///
    /// let seed = Seed::from_encoded("snoPBrXtMeMyMHUVTgbuqAfg1SUTb").unwrap();
    /// // Re-encode to verify round-trip
    /// assert_eq!(seed.encode(), "snoPBrXtMeMyMHUVTgbuqAfg1SUTb");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WalletError::InvalidSeed`] if the string is not valid
    /// base58check or has the wrong version byte.
    pub fn from_encoded(encoded: &str) -> Result<Self, WalletError> {
        let decoded = bs58::decode(encoded)
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .with_check(None)
            .into_vec()
            .map_err(|e| WalletError::InvalidSeed(e.to_string()))?;

        if decoded.is_empty() || decoded[0] != SEED_VERSION {
            return Err(WalletError::InvalidSeed(
                "invalid version byte for seed".into(),
            ));
        }

        let payload = &decoded[1..];
        if payload.len() != 16 {
            return Err(WalletError::InvalidSeedLength {
                expected: 16,
                actual: payload.len(),
            });
        }

        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(payload);
        Ok(Self(bytes))
    }

    /// Encode this seed as a base58check string (starting with `s`).
    #[must_use]
    pub fn encode(&self) -> String {
        bs58::encode(&self.0)
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .with_check_version(SEED_VERSION)
            .into_string()
    }

    /// Returns the raw 16-byte seed.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Derive a [`Keypair`] from this seed using the given algorithm.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_wallet::{Seed, Algorithm};
    ///
    /// let seed = Seed::from_passphrase("masterpassphrase");
    /// let keypair = seed.derive_keypair(Algorithm::Secp256k1).unwrap();
    /// assert_eq!(keypair.classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    ///
    /// // Ed25519 derivation from the same seed produces a different address
    /// let ed_keypair = seed.derive_keypair(Algorithm::Ed25519).unwrap();
    /// assert_ne!(ed_keypair.classic_address(), keypair.classic_address());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`WalletError::KeyDerivationFailed`] if derivation fails
    /// (astronomically unlikely for well-formed seeds).
    pub fn derive_keypair(&self, algorithm: Algorithm) -> Result<Keypair, WalletError> {
        match algorithm {
            Algorithm::Secp256k1 => derive_secp256k1(self),
            Algorithm::Ed25519 => derive_ed25519(self),
        }
    }
}

// ---------------------------------------------------------------------------
// secp256k1 derivation (two-step: root → account)
// ---------------------------------------------------------------------------

/// Derive a secp256k1 keypair from a seed using the XRPL key derivation
/// algorithm.
///
/// 1. Root key: iterate `SHA-512(seed || u32_be(seq))`, take first 32 bytes,
///    check if valid secp256k1 scalar. Increment `seq` if not.
/// 2. Account key: iterate `SHA-512(root_pubkey || u32_be(0) || u32_be(sub_seq))`,
///    take first 32 bytes. Account private key = root_private + intermediate mod n.
fn derive_secp256k1(seed: &Seed) -> Result<Keypair, WalletError> {
    // Step 1: root key pair
    let mut root_secret = [0u8; 32];
    let mut found = false;
    for seq in 0..MAX_SEQUENCE {
        let mut hasher = Sha512::new();
        hasher.update(seed.as_bytes());
        hasher.update(seq.to_be_bytes());
        let hash = hasher.finalize();
        root_secret.copy_from_slice(&hash[..32]);

        if secp256k1_ops::is_valid_secret(&root_secret) {
            found = true;
            break;
        }
    }
    if !found {
        return Err(WalletError::KeyDerivationFailed(
            "could not derive valid root key from seed".into(),
        ));
    }

    let root_pubkey = secp256k1_ops::public_key(&root_secret)?;

    // Step 2: account key pair (family = 0)
    // Hash root_pubkey || family(0) || sub_seq → take first 32 bytes as intermediate.
    // Add intermediate to root_secret mod n → account secret key.
    // Retry with incremented sub_seq if the sum is invalid (astronomically unlikely).
    let mut account_secret = [0u8; 32];
    let mut intermediate = [0u8; 32];
    found = false;
    for sub_seq in 0..MAX_SEQUENCE {
        let mut hasher = Sha512::new();
        hasher.update(root_pubkey);
        hasher.update(0u32.to_be_bytes()); // family index
        hasher.update(sub_seq.to_be_bytes());
        let hash = hasher.finalize();
        intermediate.copy_from_slice(&hash[..32]);

        if let Ok(sum) = secp256k1_ops::scalar_add(&root_secret, &intermediate) {
            if secp256k1_ops::is_valid_secret(&sum) {
                account_secret = sum;
                found = true;
                break;
            }
        }
    }
    if !found {
        return Err(WalletError::KeyDerivationFailed(
            "could not derive account key from root key".into(),
        ));
    }

    let account_pubkey = secp256k1_ops::public_key(&account_secret)?;

    // Zeroize intermediates
    root_secret.zeroize();
    intermediate.zeroize();

    Keypair::from_raw(Algorithm::Secp256k1, account_secret, account_pubkey.to_vec())
}

// ---------------------------------------------------------------------------
// Ed25519 derivation (single step)
// ---------------------------------------------------------------------------

/// Derive an Ed25519 keypair from a seed.
///
/// `SHA-512(0xED || seed)` → first 32 bytes → Ed25519 private key.
fn derive_ed25519(seed: &Seed) -> Result<Keypair, WalletError> {
    let mut hasher = Sha512::new();
    hasher.update([0xED]);
    hasher.update(seed.as_bytes());
    let hash = hasher.finalize();

    let mut secret = [0u8; 32];
    secret.copy_from_slice(&hash[..32]);

    let raw_pubkey = ed25519_ops::public_key(&secret);

    // XRPL prefixes Ed25519 public keys with 0xED (33 bytes total)
    let mut pubkey = Vec::with_capacity(33);
    pubkey.push(0xED);
    pubkey.extend_from_slice(&raw_pubkey);

    Keypair::from_raw(Algorithm::Ed25519, secret, pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_encode_decode_round_trip() {
        let seed = Seed::random();
        let encoded = seed.encode();
        assert!(encoded.starts_with('s'));

        let decoded = Seed::from_encoded(&encoded).expect("decode should succeed");
        assert_eq!(seed.as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn seed_from_passphrase_deterministic() {
        let s1 = Seed::from_passphrase("masterpassphrase");
        let s2 = Seed::from_passphrase("masterpassphrase");
        assert_eq!(s1.as_bytes(), s2.as_bytes());
    }

    #[test]
    fn seed_from_passphrase_differs() {
        let s1 = Seed::from_passphrase("hello");
        let s2 = Seed::from_passphrase("world");
        assert_ne!(s1.as_bytes(), s2.as_bytes());
    }

    #[test]
    fn seed_invalid_encoded_rejected() {
        assert!(Seed::from_encoded("not-a-valid-seed").is_err());
        assert!(Seed::from_encoded("").is_err());
    }

    #[test]
    fn genesis_seed_known_vector() {
        // The "masterpassphrase" seed encodes to "snoPBrXtMeMyMHUVTgbuqAfg1SUTb"
        let seed = Seed::from_passphrase("masterpassphrase");
        let encoded = seed.encode();
        assert_eq!(encoded, "snoPBrXtMeMyMHUVTgbuqAfg1SUTb");
    }

    #[test]
    fn derive_secp256k1_from_genesis_seed() {
        let seed = Seed::from_passphrase("masterpassphrase");
        let keypair = seed
            .derive_keypair(Algorithm::Secp256k1)
            .expect("derivation should succeed");
        assert_eq!(keypair.algorithm(), Algorithm::Secp256k1);
        assert_eq!(keypair.public_key().len(), 33);

        // The genesis account address
        let address = keypair.classic_address();
        assert_eq!(address, "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
    }

    #[test]
    fn derive_ed25519_produces_prefixed_pubkey() {
        let seed = Seed::random();
        let keypair = seed
            .derive_keypair(Algorithm::Ed25519)
            .expect("derivation should succeed");
        assert_eq!(keypair.algorithm(), Algorithm::Ed25519);
        assert_eq!(keypair.public_key().len(), 33);
        // Ed25519 pubkeys start with 0xED
        assert_eq!(keypair.public_key()[0], 0xED);
    }

    #[test]
    fn derive_secp256k1_deterministic() {
        let seed = Seed::from_bytes([0xAA; 16]);
        let kp1 = seed
            .derive_keypair(Algorithm::Secp256k1)
            .expect("derive 1");
        let kp2 = seed
            .derive_keypair(Algorithm::Secp256k1)
            .expect("derive 2");
        assert_eq!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn derive_ed25519_deterministic() {
        let seed = Seed::from_bytes([0xBB; 16]);
        let kp1 = seed.derive_keypair(Algorithm::Ed25519).expect("derive 1");
        let kp2 = seed.derive_keypair(Algorithm::Ed25519).expect("derive 2");
        assert_eq!(kp1.public_key(), kp2.public_key());
    }
}

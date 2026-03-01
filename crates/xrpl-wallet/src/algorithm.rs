//! XRPL key algorithms and low-level cryptographic operations.
//!
//! This module defines the two signature algorithms supported by the XRPL and
//! encapsulates all direct calls to cryptographic crates behind feature gates.
//! No other module in this crate should import `k256`, `secp256k1`, or
//! `ed25519_dalek` directly.

// Require at least one secp256k1 backend.
#[cfg(not(any(feature = "pure-rust-crypto", feature = "native-crypto")))]
compile_error!(
    "xrpl-wallet: enable either `pure-rust-crypto` (default) or `native-crypto` feature \
     to provide a secp256k1 backend"
);

use crate::error::WalletError;

/// The two signature algorithms supported by the XRPL.
///
/// Most XRPL accounts use [`Secp256k1`](Algorithm::Secp256k1). Choose
/// [`Ed25519`](Algorithm::Ed25519) for faster signing and smaller keys.
///
/// # Examples
///
/// ```
/// use xrpl_wallet::{Wallet, Algorithm};
///
/// // secp256k1 is the most widely used on the XRPL
/// let secp_wallet = Wallet::generate(Algorithm::Secp256k1).unwrap();
/// assert_eq!(secp_wallet.algorithm(), Algorithm::Secp256k1);
///
/// // Ed25519 produces public keys prefixed with 0xED
/// let ed_wallet = Wallet::generate(Algorithm::Ed25519).unwrap();
/// assert_eq!(ed_wallet.public_key()[0], 0xED);
///
/// // Display shows the algorithm name
/// assert_eq!(format!("{}", Algorithm::Secp256k1), "secp256k1");
/// assert_eq!(format!("{}", Algorithm::Ed25519), "ed25519");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Algorithm {
    /// ECDSA on the secp256k1 curve. The default and most common algorithm.
    Secp256k1,
    /// Ed25519 (EdDSA). Used by accounts whose seed was derived with the Ed25519 family.
    Ed25519,
}

impl core::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Secp256k1 => write!(f, "secp256k1"),
            Self::Ed25519 => write!(f, "ed25519"),
        }
    }
}

// ---------------------------------------------------------------------------
// secp256k1 operations — pure Rust backend (k256)
// ---------------------------------------------------------------------------

#[cfg(all(feature = "pure-rust-crypto", not(feature = "native-crypto")))]
pub(crate) mod secp256k1_impl {
    use super::*;
    use k256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
    use k256::ecdsa::signature::SignatureEncoding;
    use k256::ecdsa::{SigningKey, VerifyingKey};
    use k256::elliptic_curve::ops::Reduce;
    use k256::{Scalar, U256};

    /// Check whether `bytes` is a valid secp256k1 secret key (non-zero, < curve order).
    pub(crate) fn is_valid_secret(bytes: &[u8; 32]) -> bool {
        SigningKey::from_slice(bytes).is_ok()
    }

    /// Derive the 33-byte compressed public key from a 32-byte secret.
    pub(crate) fn public_key(secret: &[u8; 32]) -> Result<[u8; 33], WalletError> {
        let sk = SigningKey::from_slice(secret)
            .map_err(|e| WalletError::InvalidSecretKey(e.to_string()))?;
        let vk = sk.verifying_key();
        let point = vk.to_encoded_point(true);
        let mut pubkey = [0u8; 33];
        pubkey.copy_from_slice(point.as_bytes());
        Ok(pubkey)
    }

    /// Add two 32-byte scalars modulo the secp256k1 curve order.
    ///
    /// Used during XRPL account key derivation: `account_key = root_key + intermediate`.
    pub(crate) fn scalar_add(a: &[u8; 32], b: &[u8; 32]) -> Result<[u8; 32], WalletError> {
        let sa = <Scalar as Reduce<U256>>::reduce_bytes(&(*a).into());
        let sb = <Scalar as Reduce<U256>>::reduce_bytes(&(*b).into());
        let sum = sa + sb;
        if sum.is_zero().into() {
            return Err(WalletError::KeyDerivationFailed(
                "scalar addition produced zero".into(),
            ));
        }
        Ok(sum.to_bytes().into())
    }

    /// Sign a 32-byte pre-hashed message with ECDSA secp256k1.
    ///
    /// Returns the DER-encoded signature. The `hash` must be a SHA-512/Half
    /// signing hash — this function does **not** hash the input again.
    pub(crate) fn sign(secret: &[u8; 32], hash: &[u8; 32]) -> Result<Vec<u8>, WalletError> {
        let sk = SigningKey::from_slice(secret)
            .map_err(|e| WalletError::InvalidSecretKey(e.to_string()))?;
        let sig: k256::ecdsa::Signature = sk
            .sign_prehash(hash)
            .map_err(|e| WalletError::SigningFailed(e.to_string()))?;
        Ok(sig.to_der().to_vec())
    }

    /// Verify a DER-encoded ECDSA secp256k1 signature against a 32-byte hash.
    pub(crate) fn verify(
        pubkey: &[u8],
        hash: &[u8; 32],
        signature: &[u8],
    ) -> Result<(), WalletError> {
        let vk = VerifyingKey::from_sec1_bytes(pubkey)
            .map_err(|e| WalletError::InvalidPublicKey(e.to_string()))?;
        let sig = k256::ecdsa::Signature::from_der(signature)
            .map_err(|e| WalletError::InvalidSignature(e.to_string()))?;
        vk.verify_prehash(hash, &sig)
            .map_err(|e| WalletError::VerificationFailed(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// secp256k1 operations — native C backend (rust-secp256k1)
// ---------------------------------------------------------------------------

#[cfg(feature = "native-crypto")]
pub(crate) mod secp256k1_impl {
    use super::*;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

    /// Check whether `bytes` is a valid secp256k1 secret key.
    pub(crate) fn is_valid_secret(bytes: &[u8; 32]) -> bool {
        SecretKey::from_byte_array(*bytes).is_ok()
    }

    /// Derive the 33-byte compressed public key from a 32-byte secret.
    pub(crate) fn public_key(secret: &[u8; 32]) -> Result<[u8; 33], WalletError> {
        let secp = Secp256k1::new();
        let sk = SecretKey::from_byte_array(*secret)
            .map_err(|e| WalletError::InvalidSecretKey(e.to_string()))?;
        let pk = PublicKey::from_secret_key(&secp, &sk);
        Ok(pk.serialize())
    }

    /// Add two 32-byte scalars modulo the secp256k1 curve order.
    pub(crate) fn scalar_add(a: &[u8; 32], b: &[u8; 32]) -> Result<[u8; 32], WalletError> {
        let mut sk = SecretKey::from_byte_array(*a)
            .map_err(|e| WalletError::KeyDerivationFailed(e.to_string()))?;
        let tweak = secp256k1::Scalar::from_be_bytes(*b)
            .map_err(|e| WalletError::KeyDerivationFailed(e.to_string()))?;
        sk = sk
            .add_tweak(&tweak)
            .map_err(|e| WalletError::KeyDerivationFailed(e.to_string()))?;
        Ok(sk.secret_bytes())
    }

    /// Sign a 32-byte pre-hashed message with ECDSA secp256k1 (DER-encoded).
    pub(crate) fn sign(secret: &[u8; 32], hash: &[u8; 32]) -> Result<Vec<u8>, WalletError> {
        let secp = Secp256k1::new();
        let sk = SecretKey::from_byte_array(*secret)
            .map_err(|e| WalletError::InvalidSecretKey(e.to_string()))?;
        let msg = Message::from_digest(*hash);
        let sig = secp.sign_ecdsa(msg, &sk);
        Ok(sig.serialize_der().to_vec())
    }

    /// Verify a DER-encoded ECDSA secp256k1 signature against a 32-byte hash.
    pub(crate) fn verify(
        pubkey: &[u8],
        hash: &[u8; 32],
        signature: &[u8],
    ) -> Result<(), WalletError> {
        let secp = Secp256k1::new();
        let pk = PublicKey::from_slice(pubkey)
            .map_err(|e| WalletError::InvalidPublicKey(e.to_string()))?;
        let msg = Message::from_digest(*hash);
        let sig = secp256k1::ecdsa::Signature::from_der(signature)
            .map_err(|e| WalletError::InvalidSignature(e.to_string()))?;
        secp.verify_ecdsa(msg, &sig, &pk)
            .map_err(|e| WalletError::VerificationFailed(e.to_string()))
    }
}

// Re-export the active secp256k1 implementation under a unified namespace.
pub(crate) use secp256k1_impl as secp256k1_ops;

// ---------------------------------------------------------------------------
// Ed25519 operations — always uses ed25519-dalek
// ---------------------------------------------------------------------------

pub(crate) mod ed25519_ops {
    use super::*;
    use ed25519_dalek::{Signer, Verifier};

    /// Derive the 32-byte Ed25519 public key from a 32-byte secret.
    pub(crate) fn public_key(secret: &[u8; 32]) -> [u8; 32] {
        let sk = ed25519_dalek::SigningKey::from_bytes(secret);
        sk.verifying_key().to_bytes()
    }

    /// Sign a message with Ed25519.
    ///
    /// Unlike secp256k1, Ed25519 signs the **raw signing data** (prefix +
    /// serialized transaction), not a pre-computed hash. Ed25519's internal
    /// algorithm applies its own SHA-512 hashing.
    pub(crate) fn sign(secret: &[u8; 32], message: &[u8]) -> Vec<u8> {
        let sk = ed25519_dalek::SigningKey::from_bytes(secret);
        let sig = sk.sign(message);
        sig.to_bytes().to_vec()
    }

    /// Verify an Ed25519 signature against a message.
    pub(crate) fn verify(
        pubkey: &[u8; 32],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), WalletError> {
        let vk = ed25519_dalek::VerifyingKey::from_bytes(pubkey)
            .map_err(|e| WalletError::InvalidPublicKey(e.to_string()))?;
        let sig = ed25519_dalek::Signature::from_slice(signature)
            .map_err(|e| WalletError::InvalidSignature(e.to_string()))?;
        vk.verify(message, &sig)
            .map_err(|e| WalletError::VerificationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secp256k1_public_key_is_33_bytes() {
        // A known-valid 32-byte secret (not zero, not >= curve order)
        let secret = [1u8; 32];
        let pubkey = secp256k1_ops::public_key(&secret).expect("valid secret");
        assert_eq!(pubkey.len(), 33);
        // Compressed keys start with 0x02 or 0x03
        assert!(pubkey[0] == 0x02 || pubkey[0] == 0x03);
    }

    #[test]
    fn secp256k1_sign_verify_round_trip() {
        let secret = [42u8; 32];
        let hash = [0xABu8; 32];

        let pubkey = secp256k1_ops::public_key(&secret).expect("valid secret");
        let signature = secp256k1_ops::sign(&secret, &hash).expect("sign");
        secp256k1_ops::verify(&pubkey, &hash, &signature).expect("verify should succeed");
    }

    #[test]
    fn secp256k1_verify_wrong_hash_fails() {
        let secret = [42u8; 32];
        let hash = [0xABu8; 32];
        let wrong_hash = [0xCDu8; 32];

        let pubkey = secp256k1_ops::public_key(&secret).expect("valid secret");
        let signature = secp256k1_ops::sign(&secret, &hash).expect("sign");
        assert!(secp256k1_ops::verify(&pubkey, &wrong_hash, &signature).is_err());
    }

    #[test]
    fn secp256k1_scalar_add_non_zero() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let sum = secp256k1_ops::scalar_add(&a, &b).expect("scalar add");
        // Sum should differ from both inputs
        assert_ne!(sum, a);
        assert_ne!(sum, b);
    }

    #[test]
    fn secp256k1_zero_secret_rejected() {
        let zero = [0u8; 32];
        assert!(!secp256k1_ops::is_valid_secret(&zero));
    }

    #[test]
    fn ed25519_public_key_is_32_bytes() {
        let secret = [1u8; 32];
        let pubkey = ed25519_ops::public_key(&secret);
        assert_eq!(pubkey.len(), 32);
    }

    #[test]
    fn ed25519_sign_verify_round_trip() {
        let secret = [42u8; 32];
        let message = b"test message for Ed25519 signing";

        let pubkey = ed25519_ops::public_key(&secret);
        let signature = ed25519_ops::sign(&secret, message);
        assert_eq!(signature.len(), 64);
        ed25519_ops::verify(&pubkey, message, &signature).expect("verify should succeed");
    }

    #[test]
    fn ed25519_verify_wrong_message_fails() {
        let secret = [42u8; 32];
        let message = b"correct message";
        let wrong_message = b"wrong message";

        let pubkey = ed25519_ops::public_key(&secret);
        let signature = ed25519_ops::sign(&secret, message);
        assert!(ed25519_ops::verify(&pubkey, wrong_message, &signature).is_err());
    }

    #[test]
    fn algorithm_display() {
        assert_eq!(format!("{}", Algorithm::Secp256k1), "secp256k1");
        assert_eq!(format!("{}", Algorithm::Ed25519), "ed25519");
    }
}

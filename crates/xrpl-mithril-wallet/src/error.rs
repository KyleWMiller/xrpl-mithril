//! Error types for wallet operations.

use xrpl_mithril_codec::error::CodecError;
use xrpl_mithril_types::error::TypeError;

/// Errors that can occur during wallet operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WalletError {
    /// Seed string is not valid base58check or has wrong version byte.
    #[error("invalid seed encoding: {0}")]
    InvalidSeed(String),

    /// Seed byte slice has wrong length.
    #[error("invalid seed length: expected {expected}, got {actual}")]
    InvalidSeedLength {
        /// Expected number of bytes.
        expected: usize,
        /// Actual number of bytes.
        actual: usize,
    },

    /// Secret key bytes are not a valid scalar for the chosen curve.
    #[error("invalid secret key: {0}")]
    InvalidSecretKey(String),

    /// Public key bytes are not a valid point for the chosen curve.
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Key derivation exhausted all candidate sequences without finding a valid key.
    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// Cryptographic signing operation failed.
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// Signature verification failed.
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// Signature bytes are malformed (wrong length or invalid encoding).
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// No crypto backend is enabled for the requested algorithm.
    #[error("no crypto backend available: {0}")]
    NoCryptoBackend(String),

    /// Error from the binary codec layer.
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),

    /// Error from the type layer.
    #[error("type error: {0}")]
    Type(#[from] TypeError),
}

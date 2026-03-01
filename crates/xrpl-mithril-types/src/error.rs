//! Error types for type construction and validation.

/// Errors that can occur when constructing or validating protocol types.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TypeError {
    /// XRP amount exceeds the maximum of 100 billion XRP (10^17 drops).
    #[error("XRP amount {0} exceeds maximum of 100_000_000_000_000_000 drops")]
    XrpAmountOverflow(u64),

    /// Issued currency mantissa is outside the valid range [10^15, 10^16).
    #[error("issued currency mantissa {0} is outside valid range [10^15, 10^16)")]
    InvalidMantissa(i64),

    /// Issued currency exponent is outside the valid range [-96, 80].
    #[error("issued currency exponent {0} is outside valid range [-96, 80]")]
    InvalidExponent(i8),

    /// Currency code is invalid (all zeros = XRP, byte 0 must be 0x00 for standard codes).
    #[error("invalid currency code")]
    InvalidCurrencyCode,

    /// Account ID has invalid length (expected 20 bytes).
    #[error("invalid account ID length: expected 20, got {0}")]
    InvalidAccountIdLength(usize),

    /// Invalid base58check encoding for an address.
    #[error("invalid address encoding: {0}")]
    InvalidAddress(String),

    /// Hash has invalid length.
    #[error("invalid hash length: expected {expected}, got {actual}")]
    InvalidHashLength {
        /// Expected number of bytes.
        expected: usize,
        /// Actual number of bytes provided.
        actual: usize,
    },

    /// Timestamp is before the Ripple epoch (2000-01-01T00:00:00Z).
    #[error("timestamp is before the Ripple epoch")]
    TimestampBeforeEpoch,

    /// Invalid hex encoding.
    #[error("invalid hex encoding: {0}")]
    InvalidHex(String),
}

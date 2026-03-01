//! Error types for the binary codec.

/// Errors that can occur during binary serialization or deserialization.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CodecError {
    /// Unexpected end of data during deserialization.
    #[error("unexpected end of data: needed {needed} bytes, have {available}")]
    UnexpectedEnd {
        /// Number of bytes needed.
        needed: usize,
        /// Number of bytes available.
        available: usize,
    },

    /// Unknown field type/code combination.
    #[error("unknown field: type_code={type_code}, field_code={field_code}")]
    UnknownField {
        /// The type code from the field header.
        type_code: u16,
        /// The field code from the field header.
        field_code: u16,
    },

    /// Variable-length encoding exceeds maximum size.
    #[error("VL length {0} exceeds maximum of 918744")]
    VlLengthOverflow(usize),

    /// Invalid amount encoding in binary data.
    #[error("invalid amount encoding: {0}")]
    InvalidAmount(String),

    /// Invalid field ordering (fields must be in canonical order).
    #[error("invalid field ordering: {current} (sort key {current_key:?}) must come after {previous} (sort key {previous_key:?})")]
    InvalidFieldOrder {
        /// The field that is out of order.
        current: String,
        /// Sort key of the current field.
        current_key: (u16, u16),
        /// The previous field.
        previous: String,
        /// Sort key of the previous field.
        previous_key: (u16, u16),
    },

    /// A field name from JSON was not found in the definitions registry.
    #[error("unknown field name: {0}")]
    UnknownFieldName(String),

    /// JSON value has wrong type for the field.
    #[error("type mismatch for field '{field}': expected {expected}, got {got}")]
    TypeMismatch {
        /// The field name.
        field: String,
        /// Expected type description.
        expected: String,
        /// Actual type description.
        got: String,
    },

    /// Invalid hex string in a field value.
    #[error("invalid hex in field '{field}': {reason}")]
    InvalidHex {
        /// The field name.
        field: String,
        /// The hex decode error description.
        reason: String,
    },

    /// Generic serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Generic deserialization error.
    #[error("deserialization error: {0}")]
    Deserialization(String),
}

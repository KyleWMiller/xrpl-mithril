//! Error types for transaction building, autofill, and submission.

/// Errors that can occur during transaction construction, signing, or submission.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TxError {
    /// Error from the network client layer.
    #[error("client error: {0}")]
    Client(#[from] xrpl_client::ClientError),

    /// Error from the wallet/signing layer.
    #[error("wallet error: {0}")]
    Wallet(#[from] xrpl_wallet::WalletError),

    /// Error from the binary codec layer.
    #[error("codec error: {0}")]
    Codec(#[from] xrpl_codec::error::CodecError),

    /// Error from the type layer.
    #[error("type error: {0}")]
    Type(#[from] xrpl_types::TypeError),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Autofill failed to fetch required data from the network.
    #[error("autofill failed: {0}")]
    AutofillFailed(String),

    /// Transaction was not validated within the expected ledger range.
    #[error("transaction not validated: LastLedgerSequence {last_ledger_sequence} passed")]
    NotValidated {
        /// The LastLedgerSequence that has now been surpassed.
        last_ledger_sequence: u32,
    },

    /// Transaction was included in a ledger but the engine rejected it.
    #[error("transaction failed with result: {result_code}")]
    TransactionFailed {
        /// The tec/tef/tem/ter result code.
        result_code: String,
        /// Human-readable description.
        result_message: String,
    },

    /// Transaction building validation failed (missing required fields, etc.).
    #[error("validation error: {0}")]
    Validation(String),
}

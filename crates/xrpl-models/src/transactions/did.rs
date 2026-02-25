//! Decentralized Identifier (DID) transaction types.
//!
//! The DID amendment adds support for W3C-style Decentralized Identifiers on
//! the XRP Ledger. Accounts can publish a DID document, a URI, and an
//! attestation blob directly on-ledger.
//!
//! Two transactions manage the DID lifecycle:
//! - [`DIDSet`] — Creates or updates a DID associated with the sender's account.
//! - [`DIDDelete`] — Removes the DID from the ledger.

use serde::{Deserialize, Serialize};
use xrpl_types::Blob;

// ---------------------------------------------------------------------------
// DIDSet — TransactionType = 49
// ---------------------------------------------------------------------------

/// A DIDSet transaction (TransactionType = 49).
///
/// Creates or updates the DID object associated with the sending account.
/// At least one of `data`, `uri`, or `attestation` must be provided. If the
/// account already has a DID object, the provided fields overwrite the
/// existing values; omitted fields are left unchanged.
///
/// The DID object is keyed to the account, so each account can have at most
/// one DID entry in the ledger.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/didset>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDSet {
    /// The DID document content.
    ///
    /// Typically a JSON-LD document conforming to the W3C DID specification,
    /// hex-encoded. Maximum 256 bytes.
    #[serde(rename = "DIDDocument", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Blob>,

    /// A URI associated with this DID.
    ///
    /// Can point to a DID document hosted off-ledger, or any other relevant
    /// resource. Maximum 256 bytes.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,

    /// An attestation blob associated with this DID.
    ///
    /// Issuer-defined attestation data. Maximum 256 bytes.
    #[serde(rename = "Attestation", default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Blob>,
}

// ---------------------------------------------------------------------------
// DIDDelete — TransactionType = 50
// ---------------------------------------------------------------------------

/// A DIDDelete transaction (TransactionType = 50).
///
/// Removes the DID object associated with the sending account from the ledger.
/// This transaction has no type-specific fields — the account to delete the
/// DID for is the `Account` in [`TransactionCommon`](super::TransactionCommon).
///
/// After deletion, the account's owner reserve for the DID entry is freed.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/diddelete>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DIDDelete {
    // No type-specific fields.
    // The account whose DID to delete is specified in TransactionCommon.
}

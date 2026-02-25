//! Credential transaction types.
//!
//! The Credentials amendment introduces on-ledger attestations where an issuer
//! can create a credential for a subject account. The subject must explicitly
//! accept it before it becomes active in the ledger.
//!
//! Three transactions manage the credential lifecycle:
//! - [`CredentialCreate`] — Issuer creates a credential for a subject.
//! - [`CredentialAccept`] — Subject accepts a pending credential.
//! - [`CredentialDelete`] — Issuer, subject, or anyone (if expired) deletes a credential.

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Blob};

// ---------------------------------------------------------------------------
// CredentialCreate — TransactionType = 58
// ---------------------------------------------------------------------------

/// A CredentialCreate transaction (TransactionType = 58).
///
/// Creates a new credential on the ledger. The issuer (the `Account` in
/// [`TransactionCommon`](super::TransactionCommon)) attests to a claim about
/// the `subject` account. The credential is not active until the subject
/// accepts it via [`CredentialAccept`].
///
/// The `credential_type` field is an opaque blob that identifies the kind of
/// credential (e.g., KYC status, accredited investor, etc.). Its semantics
/// are defined by the issuer and are not interpreted by the protocol.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/credentialcreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialCreate {
    /// The account that is the subject of this credential.
    #[serde(rename = "Subject")]
    pub subject: AccountId,

    /// The type of credential, as defined by the issuer.
    ///
    /// This is an opaque binary value up to 64 bytes. Convention may assign
    /// meaning (e.g., UTF-8 encoded string), but the ledger treats it as raw
    /// bytes for indexing purposes.
    #[serde(rename = "CredentialType")]
    pub credential_type: Blob,

    /// Optional expiration time, in seconds since the Ripple Epoch
    /// (2000-01-01T00:00:00Z).
    ///
    /// After this time, the credential is considered expired and can be
    /// deleted by anyone.
    #[serde(rename = "Expiration", default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    /// Optional URI pointing to additional credential data off-ledger.
    ///
    /// Typically a URL to a verifiable credential document or metadata.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Blob>,
}

// ---------------------------------------------------------------------------
// CredentialAccept — TransactionType = 59
// ---------------------------------------------------------------------------

/// A CredentialAccept transaction (TransactionType = 59).
///
/// Accepts a credential that was previously created by an issuer via
/// [`CredentialCreate`]. The transaction sender (the `Account` in
/// [`TransactionCommon`](super::TransactionCommon)) must be the subject
/// of the credential.
///
/// Once accepted, the credential becomes active on the ledger and can be
/// used for authorization in Permissioned Domains and Deposit Authorization.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/credentialaccept>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialAccept {
    /// The account that issued the credential being accepted.
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,

    /// The credential type to accept, matching the value used in the
    /// original [`CredentialCreate`].
    #[serde(rename = "CredentialType")]
    pub credential_type: Blob,
}

// ---------------------------------------------------------------------------
// CredentialDelete — TransactionType = 60
// ---------------------------------------------------------------------------

/// A CredentialDelete transaction (TransactionType = 60).
///
/// Deletes a credential from the ledger. Can be submitted by:
/// - The **issuer** — can always delete credentials they issued.
/// - The **subject** — can always delete credentials about themselves.
/// - **Anyone** — can delete a credential that has expired.
///
/// Both `subject` and `issuer` must be specified to identify the credential.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/credentialdelete>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialDelete {
    /// The account that is the subject of the credential to delete.
    #[serde(rename = "Subject")]
    pub subject: AccountId,

    /// The account that issued the credential to delete.
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,

    /// The credential type, matching the value used in the original
    /// [`CredentialCreate`].
    #[serde(rename = "CredentialType")]
    pub credential_type: Blob,
}

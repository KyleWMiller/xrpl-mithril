//! Account management transaction types.
//!
//! This module defines transactions that manage XRPL accounts: setting account
//! properties, deleting accounts, configuring regular keys, managing signer
//! lists, pre-authorizing depositors, and creating tickets.

use serde::{Deserialize, Serialize};
use xrpl_types::{AccountId, Blob, Hash256};

// ---------------------------------------------------------------------------
// SignerEntry — building block for SignerListSet
// ---------------------------------------------------------------------------

/// A single entry in a [`SignerListSet`] transaction's signer list.
///
/// Each entry identifies an account that can participate in multi-signing
/// and the weight that account's signature carries toward the quorum.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/signerlistset#signerentry-fields>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerEntry {
    /// The account address of an authorized signer.
    #[serde(rename = "Account")]
    pub account: AccountId,

    /// The weight of this signer's signature toward the quorum.
    #[serde(rename = "SignerWeight")]
    pub signer_weight: u16,
}

impl crate::serde_helpers::StArrayElement for SignerEntry {
    const WRAPPER_KEY: &'static str = "SignerEntry";
}

// ---------------------------------------------------------------------------
// AccountSet — TransactionType = 3
// ---------------------------------------------------------------------------

/// An AccountSet transaction (TransactionType = 3).
///
/// Modifies the properties of an account in the ledger. This is the
/// general-purpose transaction for changing account settings such as flags,
/// domain, transfer rate, tick size, and more.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/accountset>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountSet {
    /// An account flag to disable. See [`AccountSetFlag`](https://xrpl.org/docs/references/protocol/transactions/types/accountset#accountset-flags)
    /// for valid values.
    #[serde(rename = "ClearFlag", default, skip_serializing_if = "Option::is_none")]
    pub clear_flag: Option<u32>,

    /// The domain that owns this account, as lowercase ASCII hex-encoded bytes.
    ///
    /// To remove the domain, set this to an empty [`Blob`].
    #[serde(rename = "Domain", default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<Blob>,

    /// An arbitrary 256-bit hash, conventionally the MD5 hash of an email
    /// address for retrieving a Gravatar image.
    ///
    /// (The protocol field is technically Hash128, but we use Hash256 for
    /// simplicity in stubs.)
    #[serde(rename = "EmailHash", default, skip_serializing_if = "Option::is_none")]
    pub email_hash: Option<Hash256>,

    /// A public key for sending encrypted messages to this account.
    #[serde(rename = "MessageKey", default, skip_serializing_if = "Option::is_none")]
    pub message_key: Option<Blob>,

    /// An account flag to enable. See [`AccountSetFlag`](https://xrpl.org/docs/references/protocol/transactions/types/accountset#accountset-flags)
    /// for valid values.
    #[serde(rename = "SetFlag", default, skip_serializing_if = "Option::is_none")]
    pub set_flag: Option<u32>,

    /// The fee to charge when users transfer this account's issued currencies,
    /// represented as billionths of a unit. A value of `1000000000` (10^9)
    /// means no fee. Cannot be less than `1000000000` or more than
    /// `2000000000`, except `0` which removes the transfer rate.
    #[serde(rename = "TransferRate", default, skip_serializing_if = "Option::is_none")]
    pub transfer_rate: Option<u32>,

    /// Sets the number of significant digits for exchange rates of Offers
    /// involving currencies issued by this account. Valid values are `3`
    /// through `15`, or `0` to clear.
    #[serde(rename = "TickSize", default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<u8>,

    /// Another account that is authorized to mint NFTokens on behalf of this
    /// account. Requires the `lsfMinterAuthorized` flag.
    #[serde(rename = "NFTokenMinter", default, skip_serializing_if = "Option::is_none")]
    pub nftoken_minter: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// AccountDelete — TransactionType = 21
// ---------------------------------------------------------------------------

/// An AccountDelete transaction (TransactionType = 21).
///
/// Deletes an account from the ledger. The remaining XRP balance (minus the
/// transaction cost) is sent to the `destination`. The account must own fewer
/// than 1000 directory entries, and the account's sequence number plus 256
/// must be less than the current ledger index.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/accountdelete>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountDelete {
    /// The account to receive the remaining XRP balance.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// Arbitrary destination tag that identifies a hosted recipient or other
    /// information for the recipient of the remaining balance.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
}

// ---------------------------------------------------------------------------
// SetRegularKey — TransactionType = 5
// ---------------------------------------------------------------------------

/// A SetRegularKey transaction (TransactionType = 5).
///
/// Assigns, changes, or removes a regular key pair associated with an
/// account. A regular key can authorize transactions for the account without
/// exposing the master key.
///
/// To remove the regular key, submit this transaction without the
/// `regular_key` field.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/setregularkey>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetRegularKey {
    /// The classic address of the new regular key pair, or `None` to remove
    /// the existing regular key.
    #[serde(rename = "RegularKey", default, skip_serializing_if = "Option::is_none")]
    pub regular_key: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// SignerListSet — TransactionType = 12
// ---------------------------------------------------------------------------

/// A SignerListSet transaction (TransactionType = 12).
///
/// Creates, replaces, or removes a list of signers that can authorize
/// transactions from this account via multi-signing.
///
/// To delete the signer list, set `signer_quorum` to `0` and omit
/// `signer_entries`.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/signerlistset>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerListSet {
    /// The target quorum for the signer list. A multi-signed transaction
    /// succeeds only if the sum of weights of the provided signatures meets
    /// or exceeds this value.
    ///
    /// Set to `0` to delete the signer list.
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,

    /// Array of [`SignerEntry`] objects indicating the accounts and weights
    /// in the signer list. Omit when deleting the signer list.
    #[serde(rename = "SignerEntries", default, skip_serializing_if = "Option::is_none")]
    pub signer_entries: Option<crate::serde_helpers::StArray<SignerEntry>>,
}

// ---------------------------------------------------------------------------
// DepositPreauth — TransactionType = 19
// ---------------------------------------------------------------------------

/// A DepositPreauth transaction (TransactionType = 19).
///
/// Pre-authorizes an account to deliver payments to this account even when
/// deposit authorization is enabled (`asfDepositAuth`). Submit with exactly
/// one of `authorize` or `unauthorize`.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/depositpreauth>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositPreauth {
    /// The account to pre-authorize.
    ///
    /// Mutually exclusive with `unauthorize`.
    #[serde(rename = "Authorize", default, skip_serializing_if = "Option::is_none")]
    pub authorize: Option<AccountId>,

    /// The account whose pre-authorization should be revoked.
    ///
    /// Mutually exclusive with `authorize`.
    #[serde(rename = "Unauthorize", default, skip_serializing_if = "Option::is_none")]
    pub unauthorize: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// TicketCreate — TransactionType = 10
// ---------------------------------------------------------------------------

/// A TicketCreate transaction (TransactionType = 10).
///
/// Sets aside one or more sequence numbers as Tickets, which can be used
/// later to send transactions out of normal sequence order.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/ticketcreate>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketCreate {
    /// The number of Tickets to create. Must be between 1 and 250 (inclusive).
    #[serde(rename = "TicketCount")]
    pub ticket_count: u32,
}

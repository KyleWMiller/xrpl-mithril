//! Cross-chain bridge transaction types.
//!
//! This module defines all transactions for the XRPL cross-chain bridge
//! feature (XLS-38d), which enables value transfer between a locking chain
//! and an issuing chain via a federation of witness servers.
//!
//! The bridge is identified by an [`XChainBridge`] descriptor that appears
//! in every cross-chain transaction.

use serde::{Deserialize, Serialize};
use xrpl_mithril_types::currency::Issue;
use xrpl_mithril_types::{AccountId, Amount, Blob};

// ---------------------------------------------------------------------------
// XChainBridge — common descriptor
// ---------------------------------------------------------------------------

/// Identifies a cross-chain bridge.
///
/// Every cross-chain transaction references this structure to specify which
/// bridge the transaction applies to. A bridge is uniquely identified by
/// its four fields: the door accounts and asset types on both chains.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchaincreatebridge#xchainbridge-fields>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainBridge {
    /// The door account on the locking chain.
    #[serde(rename = "LockingChainDoor")]
    pub locking_chain_door: AccountId,

    /// The asset (currency) on the locking chain.
    #[serde(rename = "LockingChainIssue")]
    pub locking_chain_issue: Issue,

    /// The door account on the issuing chain.
    #[serde(rename = "IssuingChainDoor")]
    pub issuing_chain_door: AccountId,

    /// The asset (currency) on the issuing chain.
    #[serde(rename = "IssuingChainIssue")]
    pub issuing_chain_issue: Issue,
}

// ---------------------------------------------------------------------------
// XChainCreateBridge — TransactionType = 48
// ---------------------------------------------------------------------------

/// An XChainCreateBridge transaction (TransactionType = 48).
///
/// Creates a new cross-chain bridge on the ledger. This transaction is
/// submitted by the door account on one chain to establish the bridge
/// parameters.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchaincreatebridge>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainCreateBridge {
    /// The bridge to create.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The amount of XRP, in drops, to reward witness servers for providing
    /// attestations.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,

    /// The minimum amount of XRP, in drops, required for an
    /// [`XChainAccountCreateCommit`] transaction. If not present, account
    /// creation via the bridge is not allowed.
    #[serde(
        rename = "MinAccountCreateAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_account_create_amount: Option<Amount>,
}

// ---------------------------------------------------------------------------
// XChainModifyBridge — TransactionType = 47
// ---------------------------------------------------------------------------

/// An XChainModifyBridge transaction (TransactionType = 47).
///
/// Modifies the parameters of an existing cross-chain bridge. Can update
/// the signature reward and/or the minimum account create amount.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchainmodifybridge>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainModifyBridge {
    /// The bridge to modify.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The new signature reward for witness servers. If omitted, the reward
    /// is not changed.
    #[serde(rename = "SignatureReward", default, skip_serializing_if = "Option::is_none")]
    pub signature_reward: Option<Amount>,

    /// The new minimum amount for account creation transactions. If omitted,
    /// the minimum is not changed.
    #[serde(
        rename = "MinAccountCreateAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_account_create_amount: Option<Amount>,
}

// ---------------------------------------------------------------------------
// XChainCreateClaimID — TransactionType = 41
// ---------------------------------------------------------------------------

/// An XChainCreateClaimID transaction (TransactionType = 41).
///
/// Creates a new cross-chain claim ID on the destination chain. The claim ID
/// is used to pair the funds locked on the locking chain (via
/// [`XChainCommit`]) with a claim on the issuing chain (via
/// [`XChainClaim`]).
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchaincreateclaimid>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainCreateClaimID {
    /// The bridge this claim ID applies to.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The amount to pay witness servers for providing signatures. Must match
    /// the bridge's `SignatureReward`.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,

    /// The account on the source chain that will lock or burn funds for this
    /// claim.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: AccountId,
}

// ---------------------------------------------------------------------------
// XChainCommit — TransactionType = 42
// ---------------------------------------------------------------------------

/// An XChainCommit transaction (TransactionType = 42).
///
/// Locks funds on the locking chain (or burns them on the issuing chain)
/// for cross-chain transfer. The `xchain_claim_id` associates this
/// commitment with a specific claim on the other chain.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchaincommit>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainCommit {
    /// The bridge to commit funds to.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The claim ID from an [`XChainCreateClaimID`] on the destination chain.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: u64,

    /// The amount to commit to the bridge.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The destination account on the other chain. If provided, funds can
    /// only be claimed by this account.
    #[serde(
        rename = "OtherChainDestination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub other_chain_destination: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// XChainClaim — TransactionType = 43
// ---------------------------------------------------------------------------

/// An XChainClaim transaction (TransactionType = 43).
///
/// Claims funds on the destination chain that were committed via
/// [`XChainCommit`] on the source chain. The witness servers must have
/// submitted enough attestations for this claim ID.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchainclaim>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainClaim {
    /// The bridge to claim from.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The claim ID for this cross-chain transfer.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: u64,

    /// The account on the destination chain that receives the funds.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// Arbitrary tag for the destination.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,

    /// The amount to claim. Must match the committed amount.
    #[serde(rename = "Amount")]
    pub amount: Amount,
}

// ---------------------------------------------------------------------------
// XChainAccountCreateCommit — TransactionType = 44
// ---------------------------------------------------------------------------

/// An XChainAccountCreateCommit transaction (TransactionType = 44).
///
/// Commits funds on the locking chain to create a new account on the issuing
/// chain. This is used when the destination account does not yet exist on
/// the other chain.
///
/// The `amount` must be at least the bridge's `MinAccountCreateAmount`.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchainaccountcreatecommit>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainAccountCreateCommit {
    /// The bridge to commit funds to for account creation.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The destination account to create on the other chain.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The amount to commit. Must be at least `MinAccountCreateAmount`.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The amount to pay witness servers for providing signatures.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
}

// ---------------------------------------------------------------------------
// XChainAddClaimAttestation — TransactionType = 45
// ---------------------------------------------------------------------------

/// An XChainAddClaimAttestation transaction (TransactionType = 45).
///
/// Submitted by a witness server to attest that funds were locked (or burned)
/// on the source chain for a specific claim ID. Once enough attestations are
/// collected, the claim can be completed.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchainaddclaimattestation>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainAddClaimAttestation {
    /// The bridge this attestation applies to.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// The claim ID being attested.
    #[serde(rename = "XChainClaimID")]
    pub xchain_claim_id: u64,

    /// The amount that was committed on the source chain.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The account on the source chain that committed the funds.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: AccountId,

    /// The public key of the witness server providing this attestation.
    #[serde(rename = "PublicKey")]
    pub public_key: Blob,

    /// The signature from the witness server, proving it observed the
    /// commitment on the source chain.
    #[serde(rename = "Signature")]
    pub signature: Blob,

    /// The account of the witness server that signed this attestation.
    #[serde(rename = "AttestationSignerAccount")]
    pub attestation_signer_account: AccountId,

    /// The account that should receive the attestation reward.
    #[serde(rename = "AttestationRewardAccount")]
    pub attestation_reward_account: AccountId,

    /// Whether the event being attested occurred on the locking chain (`1`)
    /// or the issuing chain (`0`).
    #[serde(rename = "WasLockingChainSend")]
    pub was_locking_chain_send: u8,

    /// The destination account for the claim, if one was specified in the
    /// original [`XChainCommit`].
    #[serde(rename = "Destination", default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<AccountId>,
}

// ---------------------------------------------------------------------------
// XChainAddAccountCreateAttestation — TransactionType = 46
// ---------------------------------------------------------------------------

/// An XChainAddAccountCreateAttestation transaction (TransactionType = 46).
///
/// Submitted by a witness server to attest that funds were committed on the
/// source chain for account creation via [`XChainAccountCreateCommit`]. Once
/// enough attestations are collected, the new account is created on the
/// destination chain.
///
/// # XRPL Documentation
///
/// <https://xrpl.org/docs/references/protocol/transactions/types/xchainaddaccountcreateattestation>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainAddAccountCreateAttestation {
    /// The bridge this attestation applies to.
    #[serde(rename = "XChainBridge")]
    pub xchain_bridge: XChainBridge,

    /// A counter for account creation transactions. Each account creation
    /// commitment increments this value on the bridge.
    #[serde(rename = "XChainAccountCreateCount")]
    pub xchain_account_create_count: u64,

    /// The amount that was committed for account creation.
    #[serde(rename = "Amount")]
    pub amount: Amount,

    /// The signature reward committed by the account creator.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,

    /// The destination account to be created on the other chain.
    #[serde(rename = "Destination")]
    pub destination: AccountId,

    /// The account on the source chain that submitted the account creation
    /// commit.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: AccountId,

    /// The public key of the witness server providing this attestation.
    #[serde(rename = "PublicKey")]
    pub public_key: Blob,

    /// The signature from the witness server.
    #[serde(rename = "Signature")]
    pub signature: Blob,

    /// The account of the witness server that signed this attestation.
    #[serde(rename = "AttestationSignerAccount")]
    pub attestation_signer_account: AccountId,

    /// The account that should receive the attestation reward.
    #[serde(rename = "AttestationRewardAccount")]
    pub attestation_reward_account: AccountId,

    /// Whether the event being attested occurred on the locking chain (`1`)
    /// or the issuing chain (`0`).
    #[serde(rename = "WasLockingChainSend")]
    pub was_locking_chain_send: u8,
}

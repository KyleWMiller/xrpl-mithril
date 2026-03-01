# XRPL Transaction Type Catalog

> **Source**: `crates/xrpl-codec/src/definitions.json` from rippled v3.1.0+
> **Date**: 2026-02-24
> **Status**: Phase 0 Research Document

This document catalogs every transaction type defined in the XRP Ledger protocol,
extracted from the canonical `definitions.json` and cross-referenced with the XRPL
protocol specification and rippled source code.

---

## Table of Contents

1. [Type Code Registry](#1-type-code-registry)
2. [Common Transaction Fields](#2-common-transaction-fields)
3. [Per-Transaction Specifications](#3-per-transaction-specifications)
   - 3.1 [Payment](#31-payment)
   - 3.2 [DEX (Offers)](#32-dex-offers)
   - 3.3 [Trust Lines](#33-trust-lines)
   - 3.4 [Escrow](#34-escrow)
   - 3.5 [Payment Channels](#35-payment-channels)
   - 3.6 [Checks](#36-checks)
   - 3.7 [AMM](#37-amm)
   - 3.8 [NFT (Non-Fungible Tokens)](#38-nft-non-fungible-tokens)
   - 3.9 [MPT (Multi-Purpose Tokens)](#39-mpt-multi-purpose-tokens)
   - 3.10 [Credentials](#310-credentials)
   - 3.11 [Oracle](#311-oracle)
   - 3.12 [DID (Decentralized Identifiers)](#312-did-decentralized-identifiers)
   - 3.13 [Account Management](#313-account-management)
   - 3.14 [XChain Bridge](#314-xchain-bridge)
   - 3.15 [Permissioned Domains](#315-permissioned-domains)
   - 3.16 [Delegation](#316-delegation)
   - 3.17 [Vault](#317-vault)
   - 3.18 [Lending (Loan/LoanBroker)](#318-lending-loanloanbroker)
   - 3.19 [Batch](#319-batch)
   - 3.20 [Pseudo-Transactions](#320-pseudo-transactions)
4. [Field Types Reference](#4-field-types-reference)

---

## 1. Type Code Registry

Complete listing of all transaction types from `definitions.json` TRANSACTION_TYPES,
sorted by type code.

| Type Code | Transaction Type | Category | Amendment |
|-----------|-----------------|----------|-----------|
| 0 | Payment | Payment | (Original) |
| 1 | EscrowCreate | Escrow | Escrow (2017) |
| 2 | EscrowFinish | Escrow | Escrow (2017) |
| 3 | AccountSet | Account Mgmt | (Original) |
| 4 | EscrowCancel | Escrow | Escrow (2017) |
| 5 | SetRegularKey | Account Mgmt | (Original) |
| 7 | OfferCreate | DEX | (Original) |
| 8 | OfferCancel | DEX | (Original) |
| 10 | TicketCreate | Account Mgmt | TicketBatch (2021) |
| 12 | SignerListSet | Account Mgmt | MultiSign (2016) |
| 13 | PaymentChannelCreate | Payment Channels | PayChan (2017) |
| 14 | PaymentChannelFund | Payment Channels | PayChan (2017) |
| 15 | PaymentChannelClaim | Payment Channels | PayChan (2017) |
| 16 | CheckCreate | Checks | Checks (2018) |
| 17 | CheckCash | Checks | Checks (2018) |
| 18 | CheckCancel | Checks | Checks (2018) |
| 19 | DepositPreauth | Account Mgmt | DepositPreauth (2018) |
| 20 | TrustSet | Trust Lines | (Original) |
| 21 | AccountDelete | Account Mgmt | DeletableAccounts (2020) |
| 25 | NFTokenMint | NFT | NonFungibleTokensV1_1 (2022) |
| 26 | NFTokenBurn | NFT | NonFungibleTokensV1_1 (2022) |
| 27 | NFTokenCreateOffer | NFT | NonFungibleTokensV1_1 (2022) |
| 28 | NFTokenCancelOffer | NFT | NonFungibleTokensV1_1 (2022) |
| 29 | NFTokenAcceptOffer | NFT | NonFungibleTokensV1_1 (2022) |
| 30 | Clawback | Trust Lines | Clawback (2024) |
| 31 | AMMClawback | AMM | AMMClawback (2025) |
| 35 | AMMCreate | AMM | AMM (2024) |
| 36 | AMMDeposit | AMM | AMM (2024) |
| 37 | AMMWithdraw | AMM | AMM (2024) |
| 38 | AMMVote | AMM | AMM (2024) |
| 39 | AMMBid | AMM | AMM (2024) |
| 40 | AMMDelete | AMM | AMM (2024) |
| 41 | XChainCreateClaimID | XChain Bridge | XChainBridge (2024) |
| 42 | XChainCommit | XChain Bridge | XChainBridge (2024) |
| 43 | XChainClaim | XChain Bridge | XChainBridge (2024) |
| 44 | XChainAccountCreateCommit | XChain Bridge | XChainBridge (2024) |
| 45 | XChainAddClaimAttestation | XChain Bridge | XChainBridge (2024) |
| 46 | XChainAddAccountCreateAttestation | XChain Bridge | XChainBridge (2024) |
| 47 | XChainModifyBridge | XChain Bridge | XChainBridge (2024) |
| 48 | XChainCreateBridge | XChain Bridge | XChainBridge (2024) |
| 49 | DIDSet | DID | DID (2024) |
| 50 | DIDDelete | DID | DID (2024) |
| 51 | OracleSet | Oracle | PriceOracle (2024) |
| 52 | OracleDelete | Oracle | PriceOracle (2024) |
| 53 | LedgerStateFix | Other | LedgerStateFix (2025) |
| 54 | MPTokenIssuanceCreate | MPT | MPTokensV1 (2025) |
| 55 | MPTokenIssuanceDestroy | MPT | MPTokensV1 (2025) |
| 56 | MPTokenIssuanceSet | MPT | MPTokensV1 (2025) |
| 57 | MPTokenAuthorize | MPT | MPTokensV1 (2025) |
| 58 | CredentialCreate | Credentials | Credentials (2025) |
| 59 | CredentialAccept | Credentials | Credentials (2025) |
| 60 | CredentialDelete | Credentials | Credentials (2025) |
| 61 | NFTokenModify | NFT | DynamicNFT (2025) |
| 62 | PermissionedDomainSet | Permissioned Domains | PermissionedDomains (2025) |
| 63 | PermissionedDomainDelete | Permissioned Domains | PermissionedDomains (2025) |
| 64 | DelegateSet | Delegation | DelegateKeys (2025) |
| 65 | VaultCreate | Vault | Vault (2026) |
| 66 | VaultSet | Vault | Vault (2026) |
| 67 | VaultDelete | Vault | Vault (2026) |
| 68 | VaultDeposit | Vault | Vault (2026) |
| 69 | VaultWithdraw | Vault | Vault (2026) |
| 70 | VaultClawback | Vault | Vault (2026) |
| 71 | Batch | Batch | Batch (2026) |
| 74 | LoanBrokerSet | Lending | Lending (2026) |
| 75 | LoanBrokerDelete | Lending | Lending (2026) |
| 76 | LoanBrokerCoverDeposit | Lending | Lending (2026) |
| 77 | LoanBrokerCoverWithdraw | Lending | Lending (2026) |
| 78 | LoanBrokerCoverClawback | Lending | Lending (2026) |
| 80 | LoanSet | Lending | Lending (2026) |
| 81 | LoanDelete | Lending | Lending (2026) |
| 82 | LoanManage | Lending | Lending (2026) |
| 84 | LoanPay | Lending | Lending (2026) |
| 100 | EnableAmendment | Pseudo-Tx | (Original) |
| 101 | SetFee | Pseudo-Tx | (Original) |
| 102 | UNLModify | Pseudo-Tx | NegativeUNL (2021) |

**Note**: Type codes 6, 9, 11, 22-24, 32-34, 72-73, 79, 83, 85-99 are currently unassigned.
The `Invalid` type has code -1.

**Total**: 83 transaction types (80 user-submittable + 3 pseudo-transactions).

---

## 2. Common Transaction Fields

Every transaction (except pseudo-transactions) includes these fields. Pseudo-transactions
use a subset.

### Required Fields

| Field | XRPL Type | Serialization | Description |
|-------|-----------|---------------|-------------|
| `TransactionType` | UInt16 | nth=2 | Identifies the transaction type (see registry above) |
| `Account` | AccountID | nth=1, VL-encoded | The account originating the transaction |
| `Fee` | Amount | nth=8 | XRP fee to destroy (in drops). Auto-filled by clients |
| `Sequence` | UInt32 | nth=4 | Account sequence number. Set to 0 if using TicketSequence |
| `SigningPubKey` | Blob | nth=3, VL-encoded | Public key of the signing key pair. Empty string for multi-signed txns |
| `TxnSignature` | Blob | nth=4, VL-encoded, NOT signing field | The signature. Empty for multi-signed txns |

### Optional Fields (Available on All Transactions)

| Field | XRPL Type | Serialization | Description |
|-------|-----------|---------------|-------------|
| `Flags` | UInt32 | nth=2 | Bitfield of transaction flags. 0 if omitted |
| `LastLedgerSequence` | UInt32 | nth=27 | Highest ledger index this txn can appear in. Strongly recommended |
| `AccountTxnID` | Hash256 | nth=9 | Hash of the previous transaction from this account. Requires `asfAccountTxnID` on AccountSet |
| `SourceTag` | UInt32 | nth=3 | Arbitrary source tag (for hosted wallets) |
| `Memos` | STArray | nth=9 | Array of Memo objects for arbitrary data |
| `Signers` | STArray | nth=3, NOT signing field | Array of Signer objects for multi-signing |
| `TicketSequence` | UInt32 | nth=41 | Ticket to consume instead of a sequence number. Sequence must be 0 |
| `NetworkID` | UInt32 | nth=1 | Network identifier. Required for chains with ID > 1024 |

### Memo Object Structure

Each element in the `Memos` array is a `Memo` (STObject, nth=10) containing:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `MemoType` | Blob | Optional | Hex-encoded type hint (conventionally MIME type) |
| `MemoData` | Blob | Optional | Hex-encoded arbitrary data |
| `MemoFormat` | Blob | Optional | Hex-encoded format hint (e.g., "text/plain") |

### Signer Object Structure

Each element in the `Signers` array is a `Signer` (STObject, nth=16) containing:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Account` | AccountID | Required | Address of the signer |
| `SigningPubKey` | Blob | Required | Public key of the signer |
| `TxnSignature` | Blob | Required | This signer's signature |

### Universal Transaction Flags

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfFullyCanonicalSig` | `0x80000000` | Require fully-canonical signature. Enabled by default since `RequireFullyCanonicalSig` amendment |

---

## 3. Per-Transaction Specifications

### 3.1 Payment

#### Payment (Type 0)

The foundational transaction type. Transfers value between accounts. Supports XRP,
issued currencies (IOUs), and MPT amounts. Can use cross-currency paths.

**Amendment**: Original (pre-amendment)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Destination` | AccountID | Required | Account to receive the payment |
| `Amount` | Amount | Required | Amount to deliver. XRP in drops, or issued currency/MPT object |
| `DestinationTag` | UInt32 | Optional | Tag for the destination (hosted wallet routing) |
| `InvoiceID` | Hash256 | Optional | Arbitrary 256-bit hash for invoice identification |
| `SendMax` | Amount | Optional | Maximum source amount to send. Required for cross-currency payments |
| `DeliverMin` | Amount | Optional | Minimum amount to deliver. Only valid with `tfPartialPayment` |
| `Paths` | PathSet | Optional | Payment paths for cross-currency routing. Auto-filled by clients |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfNoRippleDirect` | `0x00010000` | Do not use the default path; only use specified paths |
| `tfPartialPayment` | `0x00020000` | Allow partial payment (deliver less than Amount). Requires `DeliverMin` or accept any delivered amount |
| `tfLimitQuality` | `0x00040000` | Only take paths where quality (exchange rate) is at least as good as the destination amount / source amount |

**Constraints**:
- `Amount` must be positive
- `SendMax` must not be XRP if `Amount` is XRP (direct XRP-to-XRP needs no SendMax)
- If `tfPartialPayment` is not set, the full `Amount` must be delivered or the txn fails
- Cross-currency payments require `SendMax` and optionally `Paths`
- MPT amounts cannot use paths (direct only)

---

### 3.2 DEX (Offers)

#### OfferCreate (Type 7)

Places an order on the decentralized exchange.

**Amendment**: Original

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `TakerPays` | Amount | Required | Amount the offer taker must pay (what you want to receive) |
| `TakerGets` | Amount | Required | Amount the offer taker gets (what you are selling) |
| `Expiration` | UInt32 | Optional | Ripple epoch time after which the offer expires |
| `OfferSequence` | UInt32 | Optional | Sequence number of a previous offer to cancel |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfPassive` | `0x00010000` | Do not match against offers at the same exchange rate |
| `tfImmediateOrCancel` | `0x00020000` | Treat as IoC: fill what you can and cancel the rest |
| `tfFillOrKill` | `0x00040000` | Treat as FoK: either fill entirely or cancel |
| `tfSell` | `0x00080000` | Sell mode: exchange the full TakerGets amount even if it means receiving more than TakerPays |

**Constraints**:
- `tfImmediateOrCancel` and `tfFillOrKill` are mutually exclusive
- Cannot create an offer with both sides being XRP
- `TakerPays` and `TakerGets` must be positive

#### OfferCancel (Type 8)

Cancels an existing offer.

**Amendment**: Original

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `OfferSequence` | UInt32 | Required | Sequence number of the offer to cancel |

**Flags**: None transaction-specific.

---

### 3.3 Trust Lines

#### TrustSet (Type 20)

Creates or modifies a trust line between two accounts.

**Amendment**: Original

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LimitAmount` | Amount | Required | Object specifying the trust line limit (currency, issuer, value) |
| `QualityIn` | UInt32 | Optional | Incoming quality ratio (value / 1,000,000,000). 0 = use default |
| `QualityOut` | UInt32 | Optional | Outgoing quality ratio. 0 = use default |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfSetfAuth` | `0x00010000` | Authorize the other party to hold currency from this account |
| `tfSetNoRipple` | `0x00020000` | Set No Ripple flag on this trust line |
| `tfClearNoRipple` | `0x00040000` | Clear No Ripple flag on this trust line |
| `tfSetFreeze` | `0x00100000` | Freeze the trust line |
| `tfClearFreeze` | `0x00200000` | Unfreeze the trust line |

**Constraints**:
- `LimitAmount.issuer` must not be the account sending the transaction
- `LimitAmount.currency` must not be XRP
- Setting both `tfSetNoRipple` and `tfClearNoRipple` is invalid

#### Clawback (Type 30)

Allows an issuer to claw back issued currency or MPT tokens from a holder.

**Amendment**: Clawback (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Amount` | Amount | Required | Amount to claw back. For IOU: issuer must be the sender's address, holder is derived from Amount. For MPT: includes MPTokenIssuanceID |
| `Holder` | AccountID | Optional | The MPT holder to claw back from (required for MPT clawback) |

**Flags**: None transaction-specific.

**Constraints**:
- The transaction sender must be the issuer of the currency/MPT
- The issuer must have enabled the `lsfAllowTrustLineClawback` flag on their account
- Cannot claw back XRP
- For IOU clawback, the `Amount.issuer` field is set to the holder's address (counterintuitive but this is how rippled works)

---

### 3.4 Escrow

#### EscrowCreate (Type 1)

Creates an escrow that holds XRP, issued currency, or MPT until conditions are met.

**Amendment**: Escrow (2017, original XRP-only); TokenEscrow / XLS-85 (2026, token + MPT support)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Destination` | AccountID | Required | Account to receive the escrowed funds |
| `Amount` | Amount | Required | Amount to escrow |
| `DestinationTag` | UInt32 | Optional | Tag for the destination |
| `CancelAfter` | UInt32 | Optional | Ripple epoch time after which the escrow can be cancelled |
| `FinishAfter` | UInt32 | Optional | Ripple epoch time after which the escrow can be finished |
| `Condition` | Blob | Optional | PREIMAGE-SHA-256 crypto-condition (DER-encoded) |

**Flags**: None transaction-specific.

**Constraints**:
- At least one of `Condition`, `FinishAfter`, or `CancelAfter` must be specified
- If both `FinishAfter` and `CancelAfter` are present, `FinishAfter` must be before `CancelAfter`
- `Amount` must be positive
- With TokenEscrow amendment: supports issued currency and MPT amounts in addition to XRP

#### EscrowFinish (Type 2)

Completes an escrow and delivers the held funds.

**Amendment**: Escrow (2017)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Owner` | AccountID | Required | Account that created the escrow |
| `OfferSequence` | UInt32 | Required | Sequence number of the EscrowCreate transaction |
| `Condition` | Blob | Optional | Must match the condition from EscrowCreate |
| `Fulfillment` | Blob | Optional | PREIMAGE-SHA-256 crypto-condition fulfillment. Required if escrow has a condition |

**Flags**: None transaction-specific.

**Constraints**:
- If the escrow has a `Condition`, both `Condition` and `Fulfillment` must be provided
- If the escrow has a `FinishAfter`, the close time of the parent ledger must be after it
- If the escrow has a `CancelAfter`, the close time must be before it

#### EscrowCancel (Type 4)

Cancels an expired escrow and returns funds to the creator.

**Amendment**: Escrow (2017)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Owner` | AccountID | Required | Account that created the escrow |
| `OfferSequence` | UInt32 | Required | Sequence number of the EscrowCreate transaction |

**Flags**: None transaction-specific.

**Constraints**:
- The escrow must have a `CancelAfter` time, and the parent ledger close time must be past it

---

### 3.5 Payment Channels

#### PaymentChannelCreate (Type 13)

Creates a unidirectional payment channel.

**Amendment**: PayChan (2017)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Destination` | AccountID | Required | Recipient of channel payments |
| `Amount` | Amount | Required | XRP (in drops) to set aside in the channel |
| `SettleDelay` | UInt32 | Required | Time (in seconds) the source must wait before closing the channel after requesting closure |
| `PublicKey` | Blob | Required | Public key of the key pair for signing claims (33 bytes, secp256k1 compressed) |
| `DestinationTag` | UInt32 | Optional | Tag for the destination |
| `CancelAfter` | UInt32 | Optional | Immutable expiration time (Ripple epoch) |

**Flags**: None transaction-specific.

**Constraints**:
- `Amount` must be XRP (drops)
- `Destination` must not equal `Account`
- `SettleDelay` must be positive

#### PaymentChannelFund (Type 14)

Adds more XRP to an existing payment channel.

**Amendment**: PayChan (2017)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Channel` | Hash256 | Required | Hash of the payment channel to fund |
| `Amount` | Amount | Required | XRP (in drops) to add to the channel |
| `Expiration` | UInt32 | Optional | New mutable expiration time. Must be later than existing or channel close time |

**Flags**: None transaction-specific.

#### PaymentChannelClaim (Type 15)

Claims XRP from a payment channel, or requests/closes the channel.

**Amendment**: PayChan (2017)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Channel` | Hash256 | Required | Hash of the payment channel |
| `Amount` | Amount | Optional | XRP (in drops) to claim |
| `Balance` | Amount | Optional | New total amount of XRP delivered by this channel |
| `Signature` | Blob | Optional | Signature of the claim (hex-encoded) |
| `PublicKey` | Blob | Optional | Public key for verifying the claim signature |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfRenew` | `0x00010000` | Clear the channel's close request (source only) |
| `tfClose` | `0x00020000` | Request channel closure. If combined with claim, processes claim first |

---

### 3.6 Checks

#### CheckCreate (Type 16)

Creates a Check object, which is like a deferred payment.

**Amendment**: Checks (2018)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Destination` | AccountID | Required | Account that can cash the check |
| `SendMax` | Amount | Required | Maximum amount the check can debit from the sender |
| `DestinationTag` | UInt32 | Optional | Tag for the destination |
| `Expiration` | UInt32 | Optional | Ripple epoch time after which the check expires |
| `InvoiceID` | Hash256 | Optional | Arbitrary 256-bit hash for reference |

**Flags**: None transaction-specific.

#### CheckCash (Type 17)

Cashes a Check, pulling funds from the sender.

**Amendment**: Checks (2018)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `CheckID` | Hash256 | Required | Object ID of the Check to cash |
| `Amount` | Amount | Optional | Exact amount to cash. Mutually exclusive with `DeliverMin` |
| `DeliverMin` | Amount | Optional | Minimum amount to cash. Mutually exclusive with `Amount` |

**Flags**: None transaction-specific.

**Constraints**:
- Exactly one of `Amount` or `DeliverMin` must be provided
- Only the `Destination` of the check can cash it

#### CheckCancel (Type 18)

Cancels a Check, removing it from the ledger.

**Amendment**: Checks (2018)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `CheckID` | Hash256 | Required | Object ID of the Check to cancel |

**Flags**: None transaction-specific.

**Constraints**:
- Can be cancelled by the sender, the destination, or anyone if the check is expired

---

### 3.7 AMM

#### AMMCreate (Type 35)

Creates a new Automated Market Maker (AMM) instance for a token pair.

**Amendment**: AMM (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Amount` | Amount | Required | First asset to deposit into the AMM pool |
| `Amount2` | Amount | Required | Second asset to deposit into the AMM pool |
| `TradingFee` | UInt16 | Required | Fee to charge for trades (in units of 1/100,000; max 1000 = 1%) |

**Flags**: None transaction-specific.

**Constraints**:
- An AMM for this asset pair must not already exist
- Neither amount can be zero
- `TradingFee` must be 0-1000 (0% to 1%)
- Assets must be distinct
- XRP, issued currencies, and MPTs are valid AMM assets

#### AMMDeposit (Type 36)

Deposits assets into an existing AMM pool.

**Amendment**: AMM (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Asset` | Issue | Required | Definition of the first asset in the AMM pair |
| `Asset2` | Issue | Required | Definition of the second asset in the AMM pair |
| `Amount` | Amount | Optional | Amount of the first asset to deposit |
| `Amount2` | Amount | Optional | Amount of the second asset to deposit |
| `EPrice` | Amount | Optional | Effective price limit (maximum LP tokens per asset) |
| `LPTokenOut` | Amount | Optional | Exact amount of LP tokens to receive |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfLPToken` | `0x00010000` | Deposit to receive exactly `LPTokenOut` LP tokens (double-asset proportional) |
| `tfSingleAsset` | `0x00080000` | Single-asset deposit of `Amount` only |
| `tfTwoAsset` | `0x00100000` | Deposit both `Amount` and `Amount2` |
| `tfOneAssetLPToken` | `0x00200000` | Deposit `Amount` to get `LPTokenOut` LP tokens |
| `tfLimitLPToken` | `0x00400000` | Deposit `Amount` with `EPrice` as effective price limit |
| `tfTwoAssetIfEmpty` | `0x00800000` | Special case: deposit both assets if AMM pool is empty |

**Constraints**:
- Exactly one of the deposit mode flags must be set
- Required fields depend on the mode flag selected

#### AMMWithdraw (Type 37)

Withdraws assets from an AMM pool.

**Amendment**: AMM (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Asset` | Issue | Required | Definition of the first asset in the AMM pair |
| `Asset2` | Issue | Required | Definition of the second asset in the AMM pair |
| `Amount` | Amount | Optional | Amount of first asset to withdraw |
| `Amount2` | Amount | Optional | Amount of second asset to withdraw |
| `EPrice` | Amount | Optional | Effective price limit (minimum LP tokens per asset) |
| `LPTokenIn` | Amount | Optional | LP tokens to return to the AMM |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfLPToken` | `0x00010000` | Return `LPTokenIn` to withdraw both assets proportionally |
| `tfWithdrawAll` | `0x00020000` | Return all LP tokens, withdraw both assets |
| `tfOneAssetWithdrawAll` | `0x00040000` | Return all LP tokens, withdraw single asset `Amount` |
| `tfSingleAsset` | `0x00080000` | Withdraw `Amount` of single asset |
| `tfTwoAsset` | `0x00100000` | Withdraw `Amount` and `Amount2` |
| `tfOneAssetLPToken` | `0x00200000` | Return `LPTokenIn` to withdraw `Amount` (single asset) |
| `tfLimitLPToken` | `0x00400000` | Withdraw `Amount` with `EPrice` as effective price limit |

#### AMMVote (Type 38)

Votes on the AMM trading fee.

**Amendment**: AMM (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Asset` | Issue | Required | Definition of the first asset in the AMM pair |
| `Asset2` | Issue | Required | Definition of the second asset in the AMM pair |
| `TradingFee` | UInt16 | Required | Proposed trading fee (0-1000, in 1/100,000 units) |

**Flags**: None transaction-specific.

#### AMMBid (Type 39)

Bids on the AMM auction slot for discounted trading fees.

**Amendment**: AMM (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Asset` | Issue | Required | Definition of the first asset in the AMM pair |
| `Asset2` | Issue | Required | Definition of the second asset in the AMM pair |
| `BidMin` | Amount | Optional | Minimum bid in LP tokens |
| `BidMax` | Amount | Optional | Maximum bid in LP tokens |
| `AuthAccounts` | STArray | Optional | Up to 4 accounts authorized to trade at the discounted fee |

**Flags**: None transaction-specific.

**Constraints**:
- `AuthAccounts` can contain at most 4 `AuthAccount` entries
- Bids are in LP tokens of the AMM

#### AMMDelete (Type 40)

Deletes an empty AMM instance.

**Amendment**: AMM (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Asset` | Issue | Required | Definition of the first asset in the AMM pair |
| `Asset2` | Issue | Required | Definition of the second asset in the AMM pair |

**Flags**: None transaction-specific.

**Constraints**:
- The AMM pool must be empty (both asset balances effectively zero)

#### AMMClawback (Type 31)

Allows an issuer to claw back their issued tokens from an AMM pool.

**Amendment**: AMMClawback (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Holder` | AccountID | Required | The LP token holder whose share of the AMM is being clawed back |
| `Asset` | Issue | Required | The asset being clawed back (must be issued by the transaction sender) |
| `Asset2` | Issue | Required | The other asset in the AMM pair |
| `Amount` | Amount | Optional | Maximum amount to claw back. If omitted, claws back the holder's entire share |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfClawTwoAssets` | `0x00000001` | Claw back both assets from the AMM (instead of just the issuer's asset) |

---

### 3.8 NFT (Non-Fungible Tokens)

#### NFTokenMint (Type 25)

Mints a new NFToken.

**Amendment**: NonFungibleTokensV1_1 (2022)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `NFTokenTaxon` | UInt32 | Required | Issuer-defined taxon for this token (category/collection identifier) |
| `Issuer` | AccountID | Optional | Issuer of the token if different from Account (authorized minter) |
| `TransferFee` | UInt16 | Optional | Fee charged on secondary sales (0-50000, representing 0.000%-50.000%) |
| `URI` | Blob | Optional | URI for off-ledger data (max 256 bytes) |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfBurnable` | `0x00000001` | Issuer can burn this NFT even after transfer |
| `tfOnlyXRP` | `0x00000002` | NFT can only be traded for XRP |
| `tfTrustLine` | `0x00000004` | Automatically create trust lines for transfer fees |
| `tfTransferable` | `0x00000008` | NFT can be transferred to parties other than the issuer |
| `tfMutable` | `0x00000010` | NFT metadata can be modified (DynamicNFT amendment) |

#### NFTokenBurn (Type 26)

Burns (destroys) an NFToken.

**Amendment**: NonFungibleTokensV1_1 (2022)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `NFTokenID` | Hash256 | Required | TokenID of the NFToken to burn |
| `Owner` | AccountID | Optional | Owner of the NFToken if different from Account (issuer burning) |

**Flags**: None transaction-specific.

#### NFTokenCreateOffer (Type 27)

Creates an offer to buy or sell an NFToken.

**Amendment**: NonFungibleTokensV1_1 (2022)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `NFTokenID` | Hash256 | Required | TokenID of the NFToken |
| `Amount` | Amount | Required | Amount offered (0 for free transfer on sell offers) |
| `Owner` | AccountID | Optional | Owner of the NFToken (required for buy offers) |
| `Destination` | AccountID | Optional | Only this account can accept the offer |
| `Expiration` | UInt32 | Optional | Offer expiration (Ripple epoch) |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfSellNFToken` | `0x00000001` | This is a sell offer (sender owns the NFT). Omit for buy offer |

#### NFTokenCancelOffer (Type 28)

Cancels one or more NFToken offers.

**Amendment**: NonFungibleTokensV1_1 (2022)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `NFTokenOffers` | Vector256 | Required | Array of NFTokenOffer object IDs to cancel |

**Flags**: None transaction-specific.

#### NFTokenAcceptOffer (Type 29)

Accepts an offer to buy or sell an NFToken. Can also broker between a buy and sell offer.

**Amendment**: NonFungibleTokensV1_1 (2022)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `NFTokenBuyOffer` | Hash256 | Optional | Object ID of the buy offer to accept |
| `NFTokenSellOffer` | Hash256 | Optional | Object ID of the sell offer to accept |
| `NFTokenBrokerFee` | Amount | Optional | Fee kept by the broker (brokered mode only) |

**Flags**: None transaction-specific.

**Constraints**:
- At least one of `NFTokenBuyOffer` or `NFTokenSellOffer` must be specified
- For brokered mode (both specified), the amounts must be compatible and the broker fee must not exceed the difference

#### NFTokenModify (Type 61)

Modifies a mutable NFToken's metadata.

**Amendment**: DynamicNFT (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `NFTokenID` | Hash256 | Required | TokenID of the mutable NFToken to modify |
| `Owner` | AccountID | Optional | Current owner if sender is the issuer modifying someone else's token |
| `URI` | Blob | Optional | New URI (max 256 bytes). If omitted, clears the URI |
| `MutableFlags` | UInt32 | Optional | New mutable flags value [VERIFY] |

**Flags**: None transaction-specific.

**Constraints**:
- The NFToken must have been minted with `tfMutable` flag
- Only the issuer can modify the NFToken's metadata

---

### 3.9 MPT (Multi-Purpose Tokens)

#### MPTokenIssuanceCreate (Type 54)

Creates a new Multi-Purpose Token issuance.

**Amendment**: MPTokensV1 (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `AssetScale` | UInt8 | Optional | Number of decimal places (0-15). Default 0 |
| `MaximumAmount` | UInt64 | Optional | Maximum number of tokens that can ever exist |
| `TransferFee` | UInt16 | Optional | Transfer fee (0-50000, representing 0.000%-50.000%) |
| `MPTokenMetadata` | Blob | Optional | Arbitrary metadata (max 1024 bytes) |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfMPTCanLock` | `0x00000002` | Issuer can lock individual balances |
| `tfMPTRequireAuth` | `0x00000004` | Holders must be authorized before holding tokens |
| `tfMPTCanEscrow` | `0x00000008` | Tokens can be escrowed |
| `tfMPTCanTrade` | `0x00000010` | Tokens can be traded on the DEX |
| `tfMPTCanTransfer` | `0x00000020` | Tokens can be transferred between non-issuer accounts |
| `tfMPTCanClawback` | `0x00000040` | Issuer can claw back tokens |

#### MPTokenIssuanceDestroy (Type 55)

Destroys an MPToken issuance (only if no tokens are outstanding).

**Amendment**: MPTokensV1 (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `MPTokenIssuanceID` | Hash192 | Required | ID of the issuance to destroy |

**Flags**: None transaction-specific.

**Constraints**:
- Sender must be the issuer
- `OutstandingAmount` must be 0

#### MPTokenIssuanceSet (Type 56)

Modifies properties of an existing MPToken issuance.

**Amendment**: MPTokensV1 (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `MPTokenIssuanceID` | Hash192 | Required | ID of the issuance to modify |
| `Holder` | AccountID | Optional | Specific holder to lock/unlock. If omitted, applies to the issuance globally |
| `MPTokenMetadata` | Blob | Optional | Updated metadata [VERIFY - DynamicMPT amendment] |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfMPTLock` | `0x00000001` | Lock the specified holder (or all holders if Holder omitted) |
| `tfMPTUnlock` | `0x00000002` | Unlock the specified holder (or all holders) |

**Constraints**:
- `tfMPTLock` and `tfMPTUnlock` are mutually exclusive
- The issuance must have been created with `tfMPTCanLock` to use lock/unlock

#### MPTokenAuthorize (Type 57)

Authorizes an account to hold an MPToken, or an account opts in to holding.

**Amendment**: MPTokensV1 (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `MPTokenIssuanceID` | Hash192 | Required | ID of the MPToken issuance |
| `Holder` | AccountID | Optional | When sent by the issuer: the account to authorize. When omitted: sender is opting in/out |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfMPTUnauthorize` | `0x00000001` | Unauthorize/opt-out instead of authorize/opt-in |

**Constraints**:
- If the issuance has `tfMPTRequireAuth`, the issuer must authorize each holder
- A non-issuer sends this to create their MPToken object (opt-in) or delete it (opt-out with zero balance)

---

### 3.10 Credentials

#### CredentialCreate (Type 58)

Creates an on-ledger credential (attestation from an issuer about a subject).

**Amendment**: Credentials (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Subject` | AccountID | Required | The account the credential is about |
| `CredentialType` | Blob | Required | Type identifier for the credential (max 64 bytes, min 1 byte) |
| `URI` | Blob | Optional | URI pointing to credential data (max 256 bytes) |
| `Expiration` | UInt32 | Optional | Credential expiration time (Ripple epoch) |

**Flags**: None transaction-specific.

**Constraints**:
- `Account` (sender) is the credential issuer
- `Subject` must be a different account from the issuer
- The `CredentialType` serves as a unique key: only one credential per (issuer, subject, type) triple

#### CredentialAccept (Type 59)

Accepts a credential issued to the sender.

**Amendment**: Credentials (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Issuer` | AccountID | Required | The account that issued the credential |
| `CredentialType` | Blob | Required | Type identifier of the credential to accept |

**Flags**: None transaction-specific.

**Constraints**:
- Sender must be the `Subject` of the credential
- Credential must exist and not yet be accepted

#### CredentialDelete (Type 60)

Deletes a credential from the ledger.

**Amendment**: Credentials (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Subject` | AccountID | Required | The subject of the credential |
| `Issuer` | AccountID | Required | The issuer of the credential |
| `CredentialType` | Blob | Required | Type identifier of the credential to delete |

**Flags**: None transaction-specific.

**Constraints**:
- Can be deleted by the issuer or the subject
- Can be deleted by anyone if expired

---

### 3.11 Oracle

#### OracleSet (Type 51)

Creates or updates a price oracle instance.

**Amendment**: PriceOracle (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `OracleDocumentID` | UInt32 | Required | Unique identifier for this oracle (scoped to the account) |
| `Provider` | Blob | Optional | Oracle provider identifier (required on creation, max 256 bytes) |
| `AssetClass` | Blob | Optional | Asset class description (e.g. "currency", max 16 bytes) |
| `URI` | Blob | Optional | URI for supplementary data (max 256 bytes) |
| `LastUpdateTime` | UInt32 | Required | Timestamp of the last data update (Ripple epoch). Must be within 300s of last close |
| `PriceDataSeries` | STArray | Required | Array of PriceData objects (max 10 entries) |

**PriceData Object Structure** (STObject, nth=32):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `BaseAsset` | Currency | Required | Base asset identifier |
| `QuoteAsset` | Currency | Required | Quote asset identifier |
| `AssetPrice` | UInt64 | Optional | Price of base in terms of quote. Omit to delete this pair |
| `Scale` | UInt8 | Optional | Scaling factor (price = AssetPrice * 10^(-Scale)) |

**Flags**: None transaction-specific.

**Constraints**:
- `PriceDataSeries` must have 1-10 entries
- On creation, `Provider` is required
- Each (BaseAsset, QuoteAsset) pair must be unique within the series
- `LastUpdateTime` must not be in the future (> 300s past last close)

#### OracleDelete (Type 52)

Deletes a price oracle.

**Amendment**: PriceOracle (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `OracleDocumentID` | UInt32 | Required | Identifier of the oracle to delete |

**Flags**: None transaction-specific.

---

### 3.12 DID (Decentralized Identifiers)

#### DIDSet (Type 49)

Creates or updates a DID document associated with the sender's account.

**Amendment**: DID (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `DIDDocument` | Blob | Optional | DID document content (max 256 bytes) |
| `URI` | Blob | Optional | URI associated with the DID (max 256 bytes) |
| `Data` | Blob | Optional | Public attestation data (max 256 bytes) |

**Flags**: None transaction-specific.

**Constraints**:
- At least one of `DIDDocument`, `URI`, or `Data` must be provided
- To clear a field, provide it as an empty string

#### DIDDelete (Type 50)

Deletes the DID document associated with the sender's account.

**Amendment**: DID (2024)

**Transaction-Specific Fields**: None (only common fields).

**Flags**: None transaction-specific.

---

### 3.13 Account Management

#### AccountSet (Type 3)

Modifies properties of the sender's account.

**Amendment**: Original

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `SetFlag` | UInt32 | Optional | AccountRoot flag to enable |
| `ClearFlag` | UInt32 | Optional | AccountRoot flag to disable |
| `Domain` | Blob | Optional | Domain name (hex-encoded, max 256 bytes). Empty to clear |
| `EmailHash` | Hash128 | Optional | MD5 hash of email for Gravatar (deprecated) |
| `MessageKey` | Blob | Optional | Public key for encrypted messaging |
| `TransferRate` | UInt32 | Optional | Fee for transfers (1000000000 = no fee; max 2000000000 = 100% fee). 0 to clear |
| `TickSize` | UInt8 | Optional | Tick size for offers (3-15 significant digits). 0 to disable |
| `WalletLocator` | Hash256 | Optional | Arbitrary 256-bit value |
| `WalletSize` | UInt32 | Optional | Unused |
| `NFTokenMinter` | AccountID | Optional | Account authorized to mint NFTs on behalf of this account. Empty to clear |

**AccountRoot Flag Values** (used with `SetFlag`/`ClearFlag`):

| Flag Value | Constant | Description |
|------------|----------|-------------|
| 1 | `asfRequireDest` | Require destination tag on incoming payments |
| 2 | `asfRequireAuth` | Require authorization for trust lines |
| 3 | `asfDisallowXRP` | Discourage incoming XRP payments |
| 4 | `asfDisableMaster` | Disable the master key pair |
| 5 | `asfAccountTxnID` | Track most recent transaction ID |
| 6 | `asfNoFreeze` | Permanently give up ability to freeze trust lines |
| 7 | `asfGlobalFreeze` | Freeze all trust lines from this account |
| 8 | `asfDefaultRipple` | Enable rippling on all new trust lines by default |
| 9 | `asfDepositAuth` | Require authorization for deposits |
| 10 | `asfAuthorizedNFTokenMinter` | Enable authorized NFT minting |
| 12 | `asfDisallowIncomingNFTokenOffer` | Block incoming NFToken offers |
| 13 | `asfDisallowIncomingCheck` | Block incoming checks |
| 14 | `asfDisallowIncomingPayChan` | Block incoming payment channels |
| 15 | `asfDisallowIncomingTrustline` | Block incoming trust lines |
| 16 | `asfAllowTrustLineClawback` | Enable clawback on this account's issued tokens |

**Flags (Flags field)**: None transaction-specific.

#### SetRegularKey (Type 5)

Assigns or removes a regular key pair for the account.

**Amendment**: Original

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `RegularKey` | AccountID | Optional | Classic address of the new regular key. Omit to remove |

**Flags**: None transaction-specific.

#### SignerListSet (Type 12)

Creates, updates, or removes a multi-signing list.

**Amendment**: MultiSign (2016)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `SignerQuorum` | UInt32 | Required | Minimum combined weight of signatures needed. 0 to delete the list |
| `SignerEntries` | STArray | Optional | Array of SignerEntry objects (required if SignerQuorum > 0) |

**SignerEntry Object Structure** (STObject, nth=11):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Account` | AccountID | Required | Address of the signer |
| `SignerWeight` | UInt16 | Required | Weight of this signer's signature |
| `WalletLocator` | Hash256 | Optional | Arbitrary 256-bit identifier |

**Flags**: None transaction-specific.

**Constraints**:
- `SignerEntries` must have 1-32 entries (with `ExpandedSignerList` amendment; otherwise 1-8)
- `SignerQuorum` must be achievable by the combined weights
- The account itself cannot be in its own signer list

#### TicketCreate (Type 10)

Creates one or more Tickets for future use as sequence number alternatives.

**Amendment**: TicketBatch (2021)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `TicketCount` | UInt32 | Required | Number of tickets to create (1-250) |

**Flags**: None transaction-specific.

#### DepositPreauth (Type 19)

Pre-authorizes an account to deliver payments to the sender (when sender has `DepositAuth` enabled).

**Amendment**: DepositPreauth (2018)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Authorize` | AccountID | Optional | Account to authorize. Mutually exclusive with `Unauthorize` |
| `Unauthorize` | AccountID | Optional | Account to remove authorization. Mutually exclusive with `Authorize` |
| `AuthorizeCredentials` | STArray | Optional | Credentials to authorize (credential-based deposit preauth) |
| `UnauthorizeCredentials` | STArray | Optional | Credentials to remove authorization |

**Flags**: None transaction-specific.

**Constraints**:
- Exactly one of `Authorize`, `Unauthorize`, `AuthorizeCredentials`, or `UnauthorizeCredentials` must be provided
- Cannot authorize yourself

#### AccountDelete (Type 21)

Deletes the sender's account and transfers remaining XRP to a destination.

**Amendment**: DeletableAccounts (2020)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Destination` | AccountID | Required | Account to receive remaining XRP |
| `DestinationTag` | UInt32 | Optional | Tag for the destination |
| `CredentialIDs` | Vector256 | Optional | Credential IDs for authorized deposit [VERIFY] |

**Flags**: None transaction-specific.

**Constraints**:
- Account sequence number must be at least 256 less than the current ledger sequence
- Account must own no ledger objects (no offers, trust lines, escrows, etc.)
- Remaining XRP goes to `Destination` minus the transaction fee

#### LedgerStateFix (Type 53)

Administrative transaction to fix specific ledger state issues.

**Amendment**: LedgerStateFix (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LedgerFixType` | UInt16 | Required | The type of ledger fix to apply |
| `Owner` | AccountID | Optional | Account whose state needs fixing |

**Flags**: None transaction-specific.

**Constraints**:
- The specific fixes available depend on the `LedgerFixType` value
- Currently used to fix NFTokenPage inconsistencies and other known ledger state bugs

---

### 3.14 XChain Bridge

#### XChainCreateBridge (Type 48)

Creates a new cross-chain bridge.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification (locking/issuing chain doors and issues) |
| `SignatureReward` | Amount | Required | Reward for attestation signers |
| `MinAccountCreateAmount` | Amount | Optional | Minimum XRP for cross-chain account creation |

**XChainBridge Object Structure**:

| Field | XRPL Type | Description |
|-------|-----------|-------------|
| `LockingChainDoor` | AccountID | Door account on the locking chain |
| `LockingChainIssue` | Issue | Asset definition on the locking chain |
| `IssuingChainDoor` | AccountID | Door account on the issuing chain |
| `IssuingChainIssue` | Issue | Asset definition on the issuing chain |

**Flags**: None transaction-specific.

**Constraints**:
- Must be submitted by a door account
- The door account must own the bridge

#### XChainModifyBridge (Type 47)

Modifies the parameters of an existing bridge.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `SignatureReward` | Amount | Optional | New signature reward |
| `MinAccountCreateAmount` | Amount | Optional | New minimum account create amount. To clear, use 0 |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfClearAccountCreateAmount` | `0x00010000` | Clear the MinAccountCreateAmount |

#### XChainCreateClaimID (Type 41)

Creates a new claim ID for a cross-chain transfer.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `SignatureReward` | Amount | Required | Reward to pay attestation signers |
| `OtherChainSource` | AccountID | Required | Account on the source chain that will commit funds |

**Flags**: None transaction-specific.

#### XChainCommit (Type 42)

Commits funds to a cross-chain transfer.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `XChainClaimID` | UInt64 | Required | Claim ID for this transfer |
| `Amount` | Amount | Required | Amount to commit |
| `OtherChainDestination` | AccountID | Optional | Destination on the other chain |

**Flags**: None transaction-specific.

#### XChainClaim (Type 43)

Claims funds on the destination chain.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `XChainClaimID` | UInt64 | Required | Claim ID for this transfer |
| `Destination` | AccountID | Required | Destination account for the funds |
| `Amount` | Amount | Required | Amount to claim |
| `DestinationTag` | UInt32 | Optional | Tag for the destination |

**Flags**: None transaction-specific.

#### XChainAccountCreateCommit (Type 44)

Commits funds for creating a new account on the destination chain.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `Destination` | AccountID | Required | New account to create on the destination chain |
| `Amount` | Amount | Required | Amount to fund the new account with |
| `SignatureReward` | Amount | Required | Reward for attestation signers |

**Flags**: None transaction-specific.

#### XChainAddClaimAttestation (Type 45)

Adds an attestation from a witness server for a cross-chain claim.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `XChainClaimID` | UInt64 | Required | Claim ID |
| `Amount` | Amount | Required | Attested amount |
| `OtherChainSource` | AccountID | Required | Source account on the other chain |
| `Destination` | AccountID | Optional | Destination account |
| `PublicKey` | Blob | Required | Public key of the attestation signer |
| `Signature` | Blob | Required | Signature of the attestation |
| `AttestationSignerAccount` | AccountID | Required | Account of the attestation signer |
| `AttestationRewardAccount` | AccountID | Required | Account to receive the attestation reward |
| `WasLockingChainSend` | UInt8 | Required | 1 if the original commit was on the locking chain, 0 otherwise |

**Flags**: None transaction-specific.

#### XChainAddAccountCreateAttestation (Type 46)

Adds an attestation for a cross-chain account creation.

**Amendment**: XChainBridge (2024)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `XChainBridge` | XChainBridge | Required | Bridge specification |
| `Amount` | Amount | Required | Attested amount |
| `Destination` | AccountID | Required | Account to create |
| `SignatureReward` | Amount | Required | Attested signature reward |
| `PublicKey` | Blob | Required | Public key of the attestation signer |
| `Signature` | Blob | Required | Attestation signature |
| `AttestationSignerAccount` | AccountID | Required | Account of the attestation signer |
| `AttestationRewardAccount` | AccountID | Required | Account to receive the reward |
| `WasLockingChainSend` | UInt8 | Required | 1 if original commit on locking chain |
| `XChainAccountCreateCount` | UInt64 | Required | Counter for account creation |

**Flags**: None transaction-specific.

---

### 3.15 Permissioned Domains

#### PermissionedDomainSet (Type 62)

Creates or updates a permissioned domain, which defines a set of credentials
required for access.

**Amendment**: PermissionedDomains (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `DomainID` | Hash256 | Optional | ID of existing domain to update. Omit to create new |
| `AcceptedCredentials` | STArray | Required | Array of Credential objects defining accepted credential types |

**Credential Object in AcceptedCredentials** (STObject, nth=33):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Issuer` | AccountID | Required | Credential issuer account |
| `CredentialType` | Blob | Required | Credential type identifier |

**Flags**: None transaction-specific.

**Constraints**:
- `AcceptedCredentials` must have at least 1 entry
- Maximum entries in `AcceptedCredentials` is 10 [VERIFY]

#### PermissionedDomainDelete (Type 63)

Deletes a permissioned domain.

**Amendment**: PermissionedDomains (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `DomainID` | Hash256 | Required | ID of the domain to delete |

**Flags**: None transaction-specific.

---

### 3.16 Delegation

#### DelegateSet (Type 64)

Sets or removes a delegate for the sender's account. A delegate is an account
authorized to submit certain transactions on behalf of the delegating account.

**Amendment**: DelegateKeys (2025)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Delegate` | AccountID | Required | Account to grant or revoke delegation to |
| `Permissions` | STArray | Optional | Array of Permission objects specifying allowed operations. Omit to remove delegate |

**Permission Object Structure** (STObject, nth=15):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `PermissionValue` | UInt32 | Required | Numeric identifier of the permitted transaction type or operation |

**Flags**: None transaction-specific.

**Constraints**:
- Cannot delegate to yourself
- The `Permissions` array specifies which transaction types the delegate may submit [VERIFY exact permission values]
- Providing an empty `Permissions` array or omitting it removes the delegation

---

### 3.17 Vault

> **Status**: Code-complete in rippled, not yet activated on mainnet. Details are
> based on the rippled source and may change before amendment activation.

#### VaultCreate (Type 65)

Creates a new vault that can hold assets (tokens or MPTs) and issue shares.

**Amendment**: Vault (2026) [VERIFY amendment name]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Asset` | Issue | Required | Asset the vault accepts for deposits |
| `ShareMPTID` | Hash192 | Optional | Pre-existing MPToken issuance to use for vault shares. If omitted, vault creates its own share token [VERIFY] |
| `Data` | Blob | Optional | Arbitrary metadata (max 256 bytes) [VERIFY] |
| `DomainID` | Hash256 | Optional | Permissioned domain restricting vault access [VERIFY] |
| `ManagementFeeRate` | UInt16 | Optional | Annual management fee rate [VERIFY units] |
| `WithdrawalPolicy` | UInt8 | Optional | Withdrawal policy (0 = standard, 1 = restricted) [VERIFY values] |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfVaultPrivate` | `0x00000001` | [VERIFY] Vault requires permissioned access |

**Constraints**:
- The vault creator becomes the vault owner/manager [VERIFY]
- Share tokens represent proportional ownership of vault assets

#### VaultSet (Type 66)

Modifies vault parameters.

**Amendment**: Vault (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `VaultID` | Hash256 | Required | ID of the vault to modify |
| `Data` | Blob | Optional | Updated metadata [VERIFY] |
| `DomainID` | Hash256 | Optional | Updated permissioned domain [VERIFY] |
| `ManagementFeeRate` | UInt16 | Optional | Updated management fee rate [VERIFY] |
| `WithdrawalPolicy` | UInt8 | Optional | Updated withdrawal policy [VERIFY] |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfVaultSetPrivate` | `0x00000001` | [VERIFY] Update privacy setting |

#### VaultDelete (Type 67)

Deletes an empty vault.

**Amendment**: Vault (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `VaultID` | Hash256 | Required | ID of the vault to delete |

**Flags**: None transaction-specific.

**Constraints**:
- Vault must have zero assets remaining [VERIFY]
- Only the vault owner can delete it

#### VaultDeposit (Type 68)

Deposits assets into a vault, receiving share tokens in return.

**Amendment**: Vault (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `VaultID` | Hash256 | Required | ID of the vault |
| `Amount` | Amount | Required | Amount to deposit |

**Flags**: None transaction-specific.

#### VaultWithdraw (Type 69)

Withdraws assets from a vault by returning share tokens.

**Amendment**: Vault (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `VaultID` | Hash256 | Required | ID of the vault |
| `Amount` | Amount | Required | Amount to withdraw (or share tokens to redeem) [VERIFY semantics] |

**Flags**: None transaction-specific.

#### VaultClawback (Type 70)

Allows the vault asset issuer to claw back assets from the vault.

**Amendment**: Vault (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `VaultID` | Hash256 | Required | ID of the vault |
| `Holder` | AccountID | Optional | Specific holder to claw back from [VERIFY] |
| `Amount` | Amount | Optional | Amount to claw back [VERIFY] |

**Flags**: None transaction-specific.

---

### 3.18 Lending (Loan/LoanBroker)

> **Status**: In active development in rippled. Highly likely to change before
> amendment activation. All fields marked [VERIFY].

The Lending protocol introduces two main components:
1. **LoanBroker** -- An entity that facilitates loans, manages cover deposits, and defines loan terms
2. **Loan** -- Individual loan instances between a broker and borrower

#### LoanBrokerSet (Type 74)

Creates or updates a loan broker configuration.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanBrokerID` | Hash256 | Optional | ID of existing broker to update. Omit to create [VERIFY] |
| `Asset` | Issue | Required | Asset for the loans [VERIFY] |
| `InterestRate` | UInt32 | Optional | Interest rate [VERIFY units] |
| `LateInterestRate` | UInt32 | Optional | Late payment interest rate [VERIFY] |
| `CloseInterestRate` | UInt32 | Optional | Early close interest rate [VERIFY] |
| `OverpaymentInterestRate` | UInt32 | Optional | Overpayment interest rate [VERIFY] |
| `OverpaymentFee` | UInt32 | Optional | Fee for overpayment [VERIFY] |
| `CoverRateMinimum` | UInt32 | Optional | Minimum cover rate required [VERIFY] |
| `CoverRateLiquidation` | UInt32 | Optional | Cover rate at which liquidation occurs [VERIFY] |
| `PaymentInterval` | UInt32 | Optional | Time between payments (seconds) [VERIFY] |
| `GracePeriod` | UInt32 | Optional | Grace period for late payments (seconds) [VERIFY] |
| `LoanScale` | Int32 | Optional | Scale factor for loan amounts [VERIFY] |

**Flags**: [VERIFY]

#### LoanBrokerDelete (Type 75)

Deletes a loan broker.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanBrokerID` | Hash256 | Required | ID of the broker to delete [VERIFY] |

#### LoanBrokerCoverDeposit (Type 76)

Deposits collateral/cover into a loan broker.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanBrokerID` | Hash256 | Required | [VERIFY] |
| `Amount` | Amount | Required | Amount to deposit as cover [VERIFY] |

#### LoanBrokerCoverWithdraw (Type 77)

Withdraws cover from a loan broker.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanBrokerID` | Hash256 | Required | [VERIFY] |
| `Amount` | Amount | Required | Amount to withdraw [VERIFY] |

#### LoanBrokerCoverClawback (Type 78)

Claws back cover from a loan broker (issuer action).

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanBrokerID` | Hash256 | Required | [VERIFY] |
| `Amount` | Amount | Optional | Amount to claw back [VERIFY] |

#### LoanSet (Type 80)

Creates or modifies a loan.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanID` | Hash256 | Optional | ID of existing loan to modify. Omit to create [VERIFY] |
| `LoanBrokerID` | Hash256 | Required | Associated loan broker [VERIFY] |
| `Borrower` | AccountID | Optional | Borrower account [VERIFY] |
| `VaultID` | Hash256 | Optional | Associated vault [VERIFY] |
| `LoanSequence` | UInt32 | Optional | [VERIFY] |
| `PaymentInterval` | UInt32 | Optional | [VERIFY] |
| `GracePeriod` | UInt32 | Optional | [VERIFY] |
| `PaymentTotal` | UInt32 | Optional | Total number of payments [VERIFY] |

**Fields from definitions.json related to loans** (likely used in Loan ledger objects and/or transactions):
- `PrincipalOutstanding` (Number) -- Outstanding principal
- `PrincipalRequested` (Number) -- Requested principal amount
- `TotalValueOutstanding` (Number) -- Total outstanding value
- `PeriodicPayment` (Number) -- Periodic payment amount
- `ManagementFeeOutstanding` (Number) -- Outstanding management fees
- `LoanOriginationFee` (Number) -- Origination fee
- `LoanServiceFee` (Number) -- Service fee
- `LatePaymentFee` (Number) -- Late payment fee
- `ClosePaymentFee` (Number) -- Early close fee

#### LoanDelete (Type 81)

Deletes a completed or cancelled loan.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanID` | Hash256 | Required | [VERIFY] |

#### LoanManage (Type 82)

Manages a loan's state (approval, disbursement, etc.).

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanID` | Hash256 | Required | [VERIFY] |

**Flags**: [VERIFY - likely uses flags or a dedicated field to indicate the management action]

#### LoanPay (Type 84)

Makes a payment on a loan.

**Amendment**: Lending (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `LoanID` | Hash256 | Required | [VERIFY] |
| `Amount` | Amount | Required | Payment amount [VERIFY] |

---

### 3.19 Batch

> **Status**: Code-complete in rippled. Enables atomic execution of multiple
> transactions as a single unit.

#### Batch (Type 71)

Submits a batch of transactions to be executed atomically.

**Amendment**: Batch (2026) [VERIFY]

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `RawTransactions` | STArray | Required | Array of RawTransaction objects (the inner transactions) |
| `BatchSigners` | STArray | Optional | Array of BatchSigner objects (signers for inner transactions). NOT a signing field |

**RawTransaction Object Structure** (STObject, nth=34):

Contains a full serialized transaction as its inner content [VERIFY exact structure].

**BatchSigner Object Structure** (STObject, nth=35):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Account` | AccountID | Required | Account signing an inner transaction [VERIFY] |
| `SigningPubKey` | Blob | Required | Public key [VERIFY] |
| `TxnSignature` | Blob | Required | Signature [VERIFY] |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfAllOrNothing` | `0x00000001` | [VERIFY] All inner transactions must succeed or all are rolled back |
| `tfOnlyOne` | `0x00000002` | [VERIFY] Only the first successful inner transaction is committed |
| `tfUntilFailure` | `0x00000004` | [VERIFY] Execute inner transactions until one fails |

**Constraints**:
- Maximum number of inner transactions [VERIFY limit]
- Inner transactions cannot themselves be Batch transactions
- The outer Batch transaction pays the fee; inner transactions may have Fee=0 [VERIFY]
- `ParentBatchID` (Hash256) links inner transactions to their parent batch

---

### 3.20 Pseudo-Transactions

Pseudo-transactions are not submitted by users. They are injected by the consensus
process to record administrative ledger changes.

#### EnableAmendment (Type 100)

Records the status of a protocol amendment.

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `Amendment` | Hash256 | Required | Hash identifying the amendment |

**Flags**:

| Flag | Hex Value | Description |
|------|-----------|-------------|
| `tfGotMajority` | `0x00010000` | Amendment has reached supermajority support |
| `tfLostMajority` | `0x00020000` | Amendment has lost supermajority support |

(No flag = amendment is now enabled.)

#### SetFee (Type 101)

Records a change to the transaction cost or reserve requirements.

**Transaction-Specific Fields** (post-XRPFees amendment):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `BaseFeeDrops` | Amount | Required | Base transaction fee in drops |
| `ReserveBaseDrops` | Amount | Required | Base reserve in drops |
| `ReserveIncrementDrops` | Amount | Required | Reserve increment per owned object in drops |

**Legacy Fields** (pre-XRPFees amendment):

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `BaseFee` | UInt64 | Required | Base fee in fee units (hex) |
| `ReferenceFeeUnits` | UInt32 | Required | Reference fee units |
| `ReserveBase` | UInt32 | Required | Base reserve in drops |
| `ReserveIncrement` | UInt32 | Required | Reserve increment in drops |

#### UNLModify (Type 102)

Records changes to the Negative UNL (trusted validators temporarily offline).

**Amendment**: NegativeUNL (2021)

**Transaction-Specific Fields**:

| Field | XRPL Type | Required | Description |
|-------|-----------|----------|-------------|
| `UNLModifyDisabling` | UInt8 | Required | 1 to add to negative UNL, 0 to remove |
| `UNLModifyValidator` | Blob | Required | Public key of the validator being modified |
| `LedgerSequence` | UInt32 | Required | Ledger sequence when this modification takes effect |

---

## 4. Field Types Reference

Mapping from XRPL serialization types to their type codes (from definitions.json TYPES)
and recommended Rust representations.

| XRPL Type | Type Code | Size | Rust Type | Description |
|-----------|-----------|------|-----------|-------------|
| `UInt8` | 16 | 1 byte | `u8` | Unsigned 8-bit integer |
| `UInt16` | 1 | 2 bytes | `u16` | Unsigned 16-bit integer |
| `UInt32` | 2 | 4 bytes | `u32` | Unsigned 32-bit integer |
| `UInt64` | 3 | 8 bytes | `u64` | Unsigned 64-bit integer |
| `Int32` | 10 | 4 bytes | `i32` | Signed 32-bit integer |
| `Hash128` | 4 | 16 bytes | `[u8; 16]` | 128-bit hash |
| `Hash160` | 17 | 20 bytes | `[u8; 20]` | 160-bit hash (used for currency/issuer in book directory) |
| `Hash192` | 21 | 24 bytes | `[u8; 24]` | 192-bit hash (MPTokenIssuanceID) |
| `Hash256` | 5 | 32 bytes | `[u8; 32]` | 256-bit hash (tx hash, ledger hash, object IDs) |
| `Amount` | 6 | Variable | `enum Amount { Xrp(u64), Iou{..}, Mpt{..} }` | XRP drops (8 bytes), IOU (48 bytes), or MPT (variable) |
| `Blob` | 7 | Variable (VL-encoded) | `Vec<u8>` | Variable-length binary data |
| `AccountID` | 8 | 20 bytes (VL-encoded) | `[u8; 20]` / `AccountId` newtype | Account identifier (160-bit) |
| `Number` | 9 | 8 bytes | `i64` or custom fixed-point | Signed number type (used in lending) |
| `STObject` | 14 | Variable | Nested struct | Serialized object (terminated by ObjectEndMarker) |
| `STArray` | 15 | Variable | `Vec<STObject>` | Array of objects (terminated by ArrayEndMarker) |
| `PathSet` | 18 | Variable | `Vec<Vec<PathStep>>` | Payment path set |
| `Vector256` | 19 | Variable (VL-encoded) | `Vec<[u8; 32]>` | Array of 256-bit hashes |
| `UInt96` | 20 | 12 bytes | `[u8; 12]` | 96-bit unsigned integer |
| `UInt384` | 22 | 48 bytes | `[u8; 48]` | 384-bit unsigned integer |
| `UInt512` | 23 | 64 bytes | `[u8; 64]` | 512-bit unsigned integer |
| `Issue` | 24 | Variable | `struct Issue { currency, issuer }` | Currency + issuer pair (asset definition) |
| `XChainBridge` | 25 | Variable | `struct XChainBridge { .. }` | Bridge specification (4 fields) |
| `Currency` | 26 | 20 bytes | `[u8; 20]` / `CurrencyCode` newtype | Currency code (standard 3-char or 160-bit hex) |

### VL-Encoded Fields

Fields with `isVLEncoded: true` are preceded by a variable-length prefix:
- Length 0-192: 1 byte prefix
- Length 193-12480: 2 byte prefix
- Length 12481-918744: 3 byte prefix

VL-encoded types from definitions.json: `Blob`, `AccountID`, `Vector256`.

### Field ID Encoding

Each field is identified by a (type_code, field_code) pair, where `type_code` comes
from the TYPES table and `field_code` is the `nth` value from the FIELDS array.

Encoding rules:
- If both type_code and field_code are < 16: single byte `(type_code << 4) | field_code`
- If type_code >= 16 and field_code < 16: `(0 << 4) | field_code`, then type_code byte
- If type_code < 16 and field_code >= 16: `(type_code << 4) | 0`, then field_code byte
- If both >= 16: `0x00`, then type_code byte, then field_code byte

### Canonical Field Ordering

Fields are serialized in canonical order: sorted first by type_code (ascending),
then by field_code (ascending) within each type. This deterministic ordering
ensures identical binary representation regardless of JSON field order.

---

## Appendix A: All Fields from definitions.json

For reference, all serializable fields extracted from definitions.json, grouped by type.

### UInt8 Fields
| Field | nth | Description |
|-------|-----|-------------|
| `CloseResolution` | 1 | Ledger close time resolution |
| `Method` | 2 | [Context-specific] |
| `TransactionResult` | 3 | Transaction engine result code |
| `Scale` | 4 | Scaling factor (oracle price data) |
| `AssetScale` | 5 | MPT decimal places |
| `TickSize` | 16 | AccountSet tick size |
| `UNLModifyDisabling` | 17 | UNL modification direction |
| `HookResult` | 18 | Hook execution result |
| `WasLockingChainSend` | 19 | Bridge attestation direction flag |
| `WithdrawalPolicy` | 20 | Vault withdrawal policy |

### UInt16 Fields
| Field | nth | Description |
|-------|-----|-------------|
| `LedgerEntryType` | 1 | Ledger entry type code |
| `TransactionType` | 2 | Transaction type code |
| `SignerWeight` | 3 | Multi-sign signer weight |
| `TransferFee` | 4 | NFT/MPT transfer fee |
| `TradingFee` | 5 | AMM trading fee |
| `DiscountedFee` | 6 | AMM discounted fee |
| `Version` | 16 | [Context-specific] |
| `LedgerFixType` | 21 | LedgerStateFix type identifier |
| `ManagementFeeRate` | 22 | Vault management fee |

### UInt32 Fields (Selected Transaction-Relevant)
| Field | nth | Description |
|-------|-----|-------------|
| `NetworkID` | 1 | Chain network identifier |
| `Flags` | 2 | Transaction/object flags |
| `SourceTag` | 3 | Source routing tag |
| `Sequence` | 4 | Account sequence number |
| `DestinationTag` | 14 | Destination routing tag |
| `LastUpdateTime` | 15 | Oracle last update time |
| `QualityIn` | 20 | TrustSet incoming quality |
| `QualityOut` | 21 | TrustSet outgoing quality |
| `OfferSequence` | 25 | Offer to cancel |
| `LastLedgerSequence` | 27 | Transaction expiration ledger |
| `SetFlag` | 33 | AccountSet flag to enable |
| `ClearFlag` | 34 | AccountSet flag to disable |
| `SignerQuorum` | 35 | Multi-sign quorum |
| `CancelAfter` | 36 | Escrow cancel time |
| `FinishAfter` | 37 | Escrow finish time |
| `SettleDelay` | 39 | PayChan settle delay |
| `TicketCount` | 40 | Number of tickets to create |
| `TicketSequence` | 41 | Ticket to use as sequence |
| `NFTokenTaxon` | 42 | NFT collection taxon |
| `OracleDocumentID` | 51 | Oracle document identifier |
| `PermissionValue` | 52 | Delegation permission value |
| `MutableFlags` | 54 | NFTokenModify mutable flags |
| `PaymentInterval` | 55 | Loan payment interval |
| `GracePeriod` | 56 | Loan grace period |
| `PaymentTotal` | 60 | Loan total payments |
| `LoanSequence` | 61 | Loan sequence number |
| `CoverRateMinimum` | 62 | Loan broker min cover rate |
| `CoverRateLiquidation` | 63 | Loan broker liquidation cover rate |
| `OverpaymentFee` | 64 | Loan overpayment fee |
| `InterestRate` | 65 | Loan interest rate |
| `LateInterestRate` | 66 | Loan late interest rate |
| `CloseInterestRate` | 67 | Loan early close interest rate |
| `OverpaymentInterestRate` | 68 | Loan overpayment interest rate |

### UInt64 Fields (Selected Transaction-Relevant)
| Field | nth | Description |
|-------|-----|-------------|
| `XChainClaimID` | 20 | Bridge claim identifier |
| `XChainAccountCreateCount` | 21 | Bridge account creation counter |
| `AssetPrice` | 23 | Oracle asset price |
| `MaximumAmount` | 24 | MPT maximum supply |
| `OutstandingAmount` | 25 | MPT outstanding supply |
| `MPTAmount` | 26 | MPT amount in transfers |

### Hash256 Fields (Selected Transaction-Relevant)
| Field | nth | Description |
|-------|-----|-------------|
| `AccountTxnID` | 9 | Previous account transaction |
| `NFTokenID` | 10 | NFToken identifier |
| `AMMID` | 14 | AMM instance identifier |
| `InvoiceID` | 17 | Invoice reference hash |
| `Amendment` | 19 | Amendment identifier hash |
| `Channel` | 22 | Payment channel hash |
| `CheckID` | 24 | Check object ID |
| `NFTokenBuyOffer` | 28 | Buy offer object ID |
| `NFTokenSellOffer` | 29 | Sell offer object ID |
| `DomainID` | 34 | Permissioned domain ID |
| `VaultID` | 35 | Vault identifier |
| `ParentBatchID` | 36 | Parent batch transaction hash |
| `LoanBrokerID` | 37 | Loan broker identifier |
| `LoanID` | 38 | Loan identifier |

### Amount Fields
| Field | nth | Description |
|-------|-----|-------------|
| `Amount` | 1 | Primary amount |
| `Balance` | 2 | Balance |
| `LimitAmount` | 3 | Trust line limit |
| `TakerPays` | 4 | DEX offer: what taker pays |
| `TakerGets` | 5 | DEX offer: what taker gets |
| `Fee` | 8 | Transaction fee |
| `SendMax` | 9 | Maximum source amount |
| `DeliverMin` | 10 | Minimum delivered amount |
| `Amount2` | 11 | AMM second asset amount |
| `BidMin` | 12 | AMM minimum bid |
| `BidMax` | 13 | AMM maximum bid |
| `MinimumOffer` | 16 | [Deprecated] |
| `NFTokenBrokerFee` | 19 | NFT broker fee |
| `BaseFeeDrops` | 22 | SetFee base fee |
| `ReserveBaseDrops` | 23 | SetFee reserve base |
| `ReserveIncrementDrops` | 24 | SetFee reserve increment |
| `LPTokenOut` | 25 | AMM LP tokens to receive |
| `LPTokenIn` | 26 | AMM LP tokens to return |
| `EPrice` | 27 | AMM effective price limit |
| `Price` | 28 | [Context-specific] |
| `SignatureReward` | 29 | Bridge attestation reward |
| `MinAccountCreateAmount` | 30 | Bridge min account creation amount |

### AccountID Fields
| Field | nth | Description |
|-------|-----|-------------|
| `Account` | 1 | Transaction sender |
| `Owner` | 2 | Object owner (escrow, NFT) |
| `Destination` | 3 | Payment/escrow destination |
| `Issuer` | 4 | Token issuer |
| `Authorize` | 5 | DepositPreauth authorize target |
| `Unauthorize` | 6 | DepositPreauth unauthorize target |
| `RegularKey` | 8 | Regular key address |
| `NFTokenMinter` | 9 | Authorized NFT minter |
| `Holder` | 11 | MPT/AMM token holder |
| `Delegate` | 12 | Delegated account |
| `OtherChainSource` | 18 | Bridge source on other chain |
| `OtherChainDestination` | 19 | Bridge destination on other chain |
| `AttestationSignerAccount` | 20 | Bridge attestation signer |
| `AttestationRewardAccount` | 21 | Bridge attestation reward recipient |
| `LockingChainDoor` | 22 | Bridge locking chain door |
| `IssuingChainDoor` | 23 | Bridge issuing chain door |
| `Subject` | 24 | Credential subject |
| `Borrower` | 25 | Loan borrower |
| `Counterparty` | 26 | Loan counterparty [VERIFY] |

### Hash192 Fields
| Field | nth | Description |
|-------|-----|-------------|
| `MPTokenIssuanceID` | 1 | MPToken issuance identifier |
| `ShareMPTID` | 2 | Vault share MPToken ID |

### Issue Fields
| Field | nth | Description |
|-------|-----|-------------|
| `LockingChainIssue` | 1 | Bridge locking chain asset |
| `IssuingChainIssue` | 2 | Bridge issuing chain asset |
| `Asset` | 3 | AMM/Vault first asset |
| `Asset2` | 4 | AMM second asset |

### Currency Fields
| Field | nth | Description |
|-------|-----|-------------|
| `BaseAsset` | 1 | Oracle base asset |
| `QuoteAsset` | 2 | Oracle quote asset |

### Number Fields (Int64-like, used in lending)
| Field | nth | Description |
|-------|-----|-------------|
| `Number` | 1 | Generic number |
| `AssetsAvailable` | 2 | Vault available assets [VERIFY] |
| `AssetsMaximum` | 3 | Vault maximum assets [VERIFY] |
| `AssetsTotal` | 4 | Vault total assets [VERIFY] |
| `LossUnrealized` | 5 | Unrealized loss [VERIFY] |
| `DebtTotal` | 6 | Total debt [VERIFY] |
| `DebtMaximum` | 7 | Maximum debt [VERIFY] |
| `CoverAvailable` | 8 | Available cover [VERIFY] |
| `LoanOriginationFee` | 9 | Loan origination fee |
| `LoanServiceFee` | 10 | Loan service fee |
| `LatePaymentFee` | 11 | Late payment fee |
| `ClosePaymentFee` | 12 | Early close payment fee |
| `PrincipalOutstanding` | 13 | Outstanding principal |
| `PrincipalRequested` | 14 | Requested principal |
| `TotalValueOutstanding` | 15 | Total outstanding value |
| `PeriodicPayment` | 16 | Periodic payment amount |
| `ManagementFeeOutstanding` | 17 | Outstanding management fee |

### Int32 Fields
| Field | nth | Description |
|-------|-----|-------------|
| `LoanScale` | 1 | Loan amount scale factor |

---

## Appendix B: Ledger Entry Types

For reference, ledger entry types from definitions.json (these are the objects
created/modified by the transactions above).

| Type Code | Ledger Entry | Related Transactions |
|-----------|-------------|---------------------|
| 55 | NFTokenOffer | NFTokenCreateOffer, NFTokenCancelOffer, NFTokenAcceptOffer |
| 67 | Check | CheckCreate, CheckCash, CheckCancel |
| 73 | DID | DIDSet, DIDDelete |
| 78 | NegativeUNL | UNLModify |
| 80 | NFTokenPage | NFTokenMint, NFTokenBurn |
| 83 | SignerList | SignerListSet |
| 84 | Ticket | TicketCreate |
| 97 | AccountRoot | AccountSet, SetRegularKey, AccountDelete |
| 100 | DirectoryNode | (Internal directory management) |
| 102 | Amendments | EnableAmendment |
| 104 | LedgerHashes | (Automatic) |
| 105 | Bridge | XChainCreateBridge, XChainModifyBridge |
| 111 | Offer | OfferCreate, OfferCancel |
| 112 | DepositPreauth | DepositPreauth |
| 113 | XChainOwnedClaimID | XChainCreateClaimID, XChainClaim |
| 114 | RippleState | TrustSet, Payment (trustline creation) |
| 115 | FeeSettings | SetFee |
| 116 | XChainOwnedCreateAccountClaimID | XChainAccountCreateCommit |
| 117 | Escrow | EscrowCreate, EscrowFinish, EscrowCancel |
| 120 | PayChannel | PaymentChannelCreate, PaymentChannelFund, PaymentChannelClaim |
| 121 | AMM | AMMCreate, AMMDeposit, AMMWithdraw, AMMVote, AMMBid, AMMDelete |
| 126 | MPTokenIssuance | MPTokenIssuanceCreate, MPTokenIssuanceDestroy, MPTokenIssuanceSet |
| 127 | MPToken | MPTokenAuthorize |
| 128 | Oracle | OracleSet, OracleDelete |
| 129 | Credential | CredentialCreate, CredentialAccept, CredentialDelete |
| 130 | PermissionedDomain | PermissionedDomainSet, PermissionedDomainDelete |
| 131 | Delegate | DelegateSet |
| 132 | Vault | VaultCreate, VaultSet, VaultDelete, VaultDeposit, VaultWithdraw |
| 136 | LoanBroker | LoanBrokerSet, LoanBrokerDelete, LoanBrokerCoverDeposit/Withdraw/Clawback |
| 137 | Loan | LoanSet, LoanDelete, LoanManage, LoanPay |

---

## Appendix C: Implementation Priority for xrpl-mithril

Based on mainnet activation status and ecosystem demand:

### Tier 1 -- Must Have (Live on Mainnet)
1. Payment (0)
2. OfferCreate (7), OfferCancel (8)
3. TrustSet (20)
4. AccountSet (3), SetRegularKey (5), AccountDelete (21)
5. SignerListSet (12), TicketCreate (10)
6. EscrowCreate (1), EscrowFinish (2), EscrowCancel (4)
7. PaymentChannelCreate (13), PaymentChannelFund (14), PaymentChannelClaim (15)
8. CheckCreate (16), CheckCash (17), CheckCancel (18)
9. DepositPreauth (19)
10. NFTokenMint (25), NFTokenBurn (26), NFTokenCreateOffer (27), NFTokenCancelOffer (28), NFTokenAcceptOffer (29)
11. AMMCreate (35), AMMDeposit (36), AMMWithdraw (37), AMMVote (38), AMMBid (39), AMMDelete (40)
12. MPTokenIssuanceCreate (54), MPTokenIssuanceDestroy (55), MPTokenIssuanceSet (56), MPTokenAuthorize (57)
13. CredentialCreate (58), CredentialAccept (59), CredentialDelete (60)
14. OracleSet (51), OracleDelete (52)
15. DIDSet (49), DIDDelete (50)
16. Clawback (30), AMMClawback (31)
17. NFTokenModify (61)

### Tier 2 -- Important (Activated or Voting)
18. XChain Bridge (types 41-48)
19. PermissionedDomainSet (62), PermissionedDomainDelete (63)
20. DelegateSet (64)
21. LedgerStateFix (53)

### Tier 3 -- Forward-Looking (Code Complete, Not Voting)
22. VaultCreate (65), VaultSet (66), VaultDelete (67), VaultDeposit (68), VaultWithdraw (69), VaultClawback (70)
23. Batch (71)
24. LoanBrokerSet (74), LoanBrokerDelete (75), LoanBrokerCoverDeposit (76), LoanBrokerCoverWithdraw (77), LoanBrokerCoverClawback (78)
25. LoanSet (80), LoanDelete (81), LoanManage (82), LoanPay (84)

### Pseudo-Transactions (Deserialize Only)
26. EnableAmendment (100), SetFee (101), UNLModify (102)

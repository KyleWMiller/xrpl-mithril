# XRPL Binary Serialization Format

> Reference document for xrpl-mithril codec implementation.
> All field codes, type codes, and properties are verified against the live
> `crates/xrpl-codec/src/definitions.json` file in this repository.

---

## Table of Contents

1. [definitions.json Structure](#1-definitionsjson-structure)
2. [Field ID Encoding](#2-field-id-encoding)
3. [Canonical Field Ordering](#3-canonical-field-ordering)
4. [Type Serialization](#4-type-serialization)
5. [Amount Encoding](#5-amount-encoding)
6. [Signing Serialization](#6-signing-serialization)
7. [STObject and STArray Nesting](#7-stobject-and-starray-nesting)
8. [PathSet Encoding](#8-pathset-encoding)
9. [Newer Types: Issue, XChainBridge, Currency](#9-newer-types-issue-xchainbridge-currency)

---

## 1. definitions.json Structure

The file `definitions.json` is the canonical machine-readable description of the XRPL
binary protocol. It contains four top-level keys: `TYPES`, `FIELDS`,
`TRANSACTION_TYPES`, and `LEDGER_ENTRY_TYPES` (plus `TRANSACTION_RESULTS` for
result codes).

### 1.1 TYPES

Each entry maps a type name to its numeric **type code**. The type code determines
both how the value is serialized and how the field header is encoded.

| Type Name      | Code  | Size / Notes                              |
|----------------|------:|-------------------------------------------|
| `NotPresent`   |     0 | Sentinel; never serialized                |
| `UInt16`       |     1 | 2 bytes, big-endian                       |
| `UInt32`       |     2 | 4 bytes, big-endian                       |
| `UInt64`       |     3 | 8 bytes, big-endian                       |
| `Hash128`      |     4 | 16 bytes, fixed                           |
| `Hash256`      |     5 | 32 bytes, fixed                           |
| `Amount`       |     6 | Variable (8, 48, or 32 bytes)             |
| `Blob`         |     7 | VL-encoded (length-prefixed)              |
| `AccountID`    |     8 | VL-encoded (always 20 bytes content)      |
| `Number`       |     9 | 8 bytes (XFL fixed-point, lending fields) |
| `Int32`        |    10 | 4 bytes, big-endian, signed               |
| `Int64`        |    11 | 8 bytes, big-endian, signed               |
| `STObject`     |    14 | Nested object, terminated by 0xE1         |
| `STArray`      |    15 | Nested array, terminated by 0xF1          |
| `UInt8`        |    16 | 1 byte                                    |
| `Hash160`      |    17 | 20 bytes, fixed                           |
| `PathSet`      |    18 | Custom encoding (see section 8)           |
| `Vector256`    |    19 | VL-encoded array of Hash256 values        |
| `UInt96`       |    20 | 12 bytes, big-endian                      |
| `Hash192`      |    21 | 24 bytes, fixed                           |
| `UInt384`      |    22 | 48 bytes, big-endian                      |
| `UInt512`      |    23 | 64 bytes, big-endian                      |
| `Issue`        |    24 | 20 bytes (XRP) or 40 bytes (IOU)          |
| `XChainBridge` |    25 | Composite (see section 9)                 |
| `Currency`     |    26 | 20 bytes, fixed                           |
| `Done`         |    -1 | Sentinel; never serialized                |
| `Unknown`      |    -2 | Sentinel; never serialized                |
| `Transaction`  | 10001 | Pseudo-type; not serialized directly      |
| `LedgerEntry`  | 10002 | Pseudo-type; not serialized directly      |
| `Validation`   | 10003 | Pseudo-type; not serialized directly      |
| `Metadata`     | 10004 | Pseudo-type; not serialized directly      |

**Key insight for codec implementation**: Type codes 1-15 fit in a single nibble
and use the compact field header encoding. Type codes >= 16 require an extra byte
in the header. See section 2.

### 1.2 FIELDS

The `FIELDS` array contains `[name, properties]` pairs. Each field has:

- **`nth`** -- The field code (used with type_code to form the field ID header).
  Values 1-15 use compact encoding; values >= 16 require an extra byte.
- **`type`** -- The type name (maps to `TYPES` for the type code).
- **`isSerialized`** -- Whether this field is included in binary serialization.
  Fields with `false` are JSON-only (e.g., `hash`, `index`, `taker_gets_funded`).
- **`isSigningField`** -- Whether this field is included when computing a
  transaction signature. Fields like `TxnSignature` and `Signers` are serialized
  but NOT signing fields (they would be zero or absent at signing time).
- **`isVLEncoded`** -- Whether the binary value is preceded by a variable-length
  prefix. True for `Blob`, `AccountID`, and `Vector256` types.

Representative examples from the file:

| Field Name         | nth | Type       | isSerialized | isSigningField | isVLEncoded |
|--------------------|----:|------------|:------------:|:--------------:|:-----------:|
| `TransactionType`  |   2 | `UInt16`   | true         | true           | false       |
| `Flags`            |   2 | `UInt32`   | true         | true           | false       |
| `Sequence`         |   4 | `UInt32`   | true         | true           | false       |
| `LastLedgerSequence` | 27 | `UInt32` | true         | true           | false       |
| `Amount`           |   1 | `Amount`   | true         | true           | false       |
| `Fee`              |   8 | `Amount`   | true         | true           | false       |
| `SigningPubKey`    |   3 | `Blob`     | true         | **true**       | true        |
| `TxnSignature`    |   4 | `Blob`     | true         | **false**      | true        |
| `Account`          |   1 | `AccountID`| true         | true           | true        |
| `Destination`      |   3 | `AccountID`| true         | true           | true        |
| `Signers`          |   3 | `STArray`  | true         | **false**      | false       |
| `Memos`            |   9 | `STArray`  | true         | true           | false       |
| `Memo`             |  10 | `STObject` | true         | true           | false       |
| `Paths`            |   1 | `PathSet`  | true         | true           | false       |
| `MPTokenIssuanceID`|   1 | `Hash192`  | true         | true           | false       |
| `hash`             | 257 | `Hash256`  | **false**    | false          | false       |
| `taker_gets_funded`| 258 | `Amount`   | **false**    | false          | false       |

### 1.3 TRANSACTION_TYPES

Each entry maps a transaction type name to its numeric code. This code is written
into the `TransactionType` field (UInt16, nth=2) during serialization.

| Transaction Type                       | Code |
|----------------------------------------|-----:|
| `Payment`                              |    0 |
| `EscrowCreate`                         |    1 |
| `EscrowFinish`                         |    2 |
| `AccountSet`                           |    3 |
| `EscrowCancel`                         |    4 |
| `SetRegularKey`                        |    5 |
| `OfferCreate`                          |    7 |
| `OfferCancel`                          |    8 |
| `TicketCreate`                         |   10 |
| `SignerListSet`                        |   12 |
| `PaymentChannelCreate`                 |   13 |
| `PaymentChannelFund`                   |   14 |
| `PaymentChannelClaim`                  |   15 |
| `CheckCreate`                          |   16 |
| `CheckCash`                            |   17 |
| `CheckCancel`                          |   18 |
| `DepositPreauth`                       |   19 |
| `TrustSet`                             |   20 |
| `AccountDelete`                        |   21 |
| `NFTokenMint`                          |   25 |
| `NFTokenBurn`                          |   26 |
| `NFTokenCreateOffer`                   |   27 |
| `NFTokenCancelOffer`                   |   28 |
| `NFTokenAcceptOffer`                   |   29 |
| `Clawback`                             |   30 |
| `AMMClawback`                          |   31 |
| `AMMCreate`                            |   35 |
| `AMMDeposit`                           |   36 |
| `AMMWithdraw`                          |   37 |
| `AMMVote`                              |   38 |
| `AMMBid`                               |   39 |
| `AMMDelete`                            |   40 |
| `XChainCreateClaimID`                  |   41 |
| `XChainCommit`                         |   42 |
| `XChainClaim`                          |   43 |
| `XChainAccountCreateCommit`            |   44 |
| `XChainAddClaimAttestation`            |   45 |
| `XChainAddAccountCreateAttestation`    |   46 |
| `XChainModifyBridge`                   |   47 |
| `XChainCreateBridge`                   |   48 |
| `DIDSet`                               |   49 |
| `DIDDelete`                            |   50 |
| `OracleSet`                            |   51 |
| `OracleDelete`                         |   52 |
| `LedgerStateFix`                       |   53 |
| `MPTokenIssuanceCreate`                |   54 |
| `MPTokenIssuanceDestroy`               |   55 |
| `MPTokenIssuanceSet`                   |   56 |
| `MPTokenAuthorize`                     |   57 |
| `CredentialCreate`                     |   58 |
| `CredentialAccept`                     |   59 |
| `CredentialDelete`                     |   60 |
| `NFTokenModify`                        |   61 |
| `PermissionedDomainSet`                |   62 |
| `PermissionedDomainDelete`             |   63 |
| `DelegateSet`                          |   64 |
| `VaultCreate`                          |   65 |
| `VaultSet`                             |   66 |
| `VaultDelete`                          |   67 |
| `VaultDeposit`                         |   68 |
| `VaultWithdraw`                        |   69 |
| `VaultClawback`                        |   70 |
| `Batch`                                |   71 |
| `LoanBrokerSet`                        |   74 |
| `LoanBrokerDelete`                     |   75 |
| `LoanBrokerCoverDeposit`              |   76 |
| `LoanBrokerCoverWithdraw`             |   77 |
| `LoanBrokerCoverClawback`             |   78 |
| `LoanSet`                              |   80 |
| `LoanDelete`                           |   81 |
| `LoanManage`                           |   82 |
| `LoanPay`                              |   84 |
| `EnableAmendment`                      |  100 |
| `SetFee`                               |  101 |
| `UNLModify`                            |  102 |
| `Invalid`                              |   -1 |

### 1.4 LEDGER_ENTRY_TYPES

Each entry maps a ledger object type name to its numeric code. This code is
written into the `LedgerEntryType` field (UInt16, nth=1) during serialization.

| Ledger Entry Type                        | Code |
|------------------------------------------|-----:|
| `NFTokenOffer`                           |   55 |
| `Check`                                  |   67 |
| `DID`                                    |   73 |
| `NegativeUNL`                            |   78 |
| `NFTokenPage`                            |   80 |
| `SignerList`                             |   83 |
| `Ticket`                                 |   84 |
| `AccountRoot`                            |   97 |
| `DirectoryNode`                          |  100 |
| `Amendments`                             |  102 |
| `LedgerHashes`                           |  104 |
| `Bridge`                                 |  105 |
| `Offer`                                  |  111 |
| `DepositPreauth`                         |  112 |
| `XChainOwnedClaimID`                     |  113 |
| `RippleState`                            |  114 |
| `FeeSettings`                            |  115 |
| `XChainOwnedCreateAccountClaimID`        |  116 |
| `Escrow`                                 |  117 |
| `PayChannel`                             |  120 |
| `AMM`                                    |  121 |
| `MPTokenIssuance`                        |  126 |
| `MPToken`                                |  127 |
| `Oracle`                                 |  128 |
| `Credential`                             |  129 |
| `PermissionedDomain`                     |  130 |
| `Delegate`                               |  131 |
| `Vault`                                  |  132 |
| `LoanBroker`                             |  136 |
| `Loan`                                   |  137 |
| `Invalid`                                |   -1 |

---

## 2. Field ID Encoding

Every serialized field begins with a **field ID header** that encodes two values:
the **type code** (from `TYPES`) and the **field code** (the `nth` value from
`FIELDS`). The header is 1, 2, or 3 bytes depending on whether each value fits
in a nibble (< 16) or requires a full byte.

### 2.1 The Four Cases

```
Let tc = type_code, fc = field_code (nth)

Case 1: tc < 16  AND  fc < 16   -->  1 byte
  Byte 0:  (tc << 4) | fc

Case 2: tc < 16  AND  fc >= 16  -->  2 bytes
  Byte 0:  (tc << 4) | 0x00          (low nibble = 0 signals "fc follows")
  Byte 1:  fc

Case 3: tc >= 16 AND  fc < 16   -->  2 bytes
  Byte 0:  0x00 | fc                 (high nibble = 0 signals "tc follows")
  Byte 1:  tc

Case 4: tc >= 16 AND  fc >= 16  -->  3 bytes
  Byte 0:  0x00                      (both nibbles = 0)
  Byte 1:  tc
  Byte 2:  fc
```

### 2.2 Concrete Examples

**Example 1: `TransactionType` -- UInt16 (tc=1), nth=2**

Both tc=1 and fc=2 are < 16, so this is Case 1:
```
Header = (1 << 4) | 2 = 0x12
```
Result: `[0x12]` (1 byte)

**Example 2: `Flags` -- UInt32 (tc=2), nth=2**

Both tc=2 and fc=2 are < 16, so this is Case 1:
```
Header = (2 << 4) | 2 = 0x22
```
Result: `[0x22]` (1 byte)

**Example 3: `Fee` -- Amount (tc=6), nth=8**

Both tc=6 and fc=8 are < 16, so this is Case 1:
```
Header = (6 << 4) | 8 = 0x68
```
Result: `[0x68]` (1 byte)

**Example 4: `TickSize` -- UInt8 (tc=16), nth=16**

Both tc=16 and fc=16 are >= 16, so this is Case 4:
```
Byte 0 = 0x00
Byte 1 = 0x10 (16)
Byte 2 = 0x10 (16)
```
Result: `[0x00, 0x10, 0x10]` (3 bytes)

**Example 5: `CloseResolution` -- UInt8 (tc=16), nth=1**

tc=16 is >= 16, fc=1 is < 16, so this is Case 3:
```
Byte 0 = 0x00 | 1 = 0x01
Byte 1 = 0x10 (16)
```
Result: `[0x01, 0x10]` (2 bytes)

**Example 6: `Fulfillment` -- Blob (tc=7), nth=16**

tc=7 is < 16, fc=16 is >= 16, so this is Case 2:
```
Byte 0 = (7 << 4) | 0 = 0x70
Byte 1 = 0x10 (16)
```
Result: `[0x70, 0x10]` (2 bytes)

**Example 7: `LastLedgerSequence` -- UInt32 (tc=2), nth=27**

tc=2 is < 16, fc=27 is >= 16, so this is Case 2:
```
Byte 0 = (2 << 4) | 0 = 0x20
Byte 1 = 0x1B (27)
```
Result: `[0x20, 0x1B]` (2 bytes)

**Example 8: `Paths` -- PathSet (tc=18), nth=1**

tc=18 is >= 16, fc=1 is < 16, so this is Case 3:
```
Byte 0 = 0x00 | 1 = 0x01
Byte 1 = 0x12 (18)
```
Result: `[0x01, 0x12]` (2 bytes)

---

## 3. Canonical Field Ordering

When serializing an object (transaction, ledger entry, or inner object), fields
MUST be written in **canonical order**:

1. **Primary sort**: by `type_code` (ascending)
2. **Secondary sort**: by `field_code` / `nth` (ascending) within the same type

Only fields where `isSerialized == true` are included in the binary output.

### Why This Matters

Canonical ordering ensures that every node, wallet, and tool produces identical
binary representations for the same logical object. This is critical because:
- Transaction hashes are computed over the canonical binary form
- Signatures are computed over the canonical binary form (with certain fields excluded)
- Ledger entry hashes depend on canonical serialization

### Ordering Example: A Simple Payment

Given a Payment transaction with these fields:

| Field               | Type      | type_code | nth (field_code) |
|---------------------|-----------|----------:|------------------:|
| `TransactionType`   | UInt16    |         1 |                 2 |
| `Flags`             | UInt32    |         2 |                 2 |
| `Sequence`          | UInt32    |         2 |                 4 |
| `DestinationTag`    | UInt32    |         2 |                14 |
| `LastLedgerSequence`| UInt32    |         2 |                27 |
| `Amount`            | Amount    |         6 |                 1 |
| `Fee`               | Amount    |         6 |                 8 |
| `SigningPubKey`     | Blob      |         7 |                 3 |
| `TxnSignature`     | Blob      |         7 |                 4 |
| `Account`           | AccountID |         8 |                 1 |
| `Destination`       | AccountID |         8 |                 3 |

Canonical serialization order (sorted by type_code, then nth):

1. `TransactionType` (tc=1, nth=2)
2. `Flags` (tc=2, nth=2)
3. `Sequence` (tc=2, nth=4)
4. `DestinationTag` (tc=2, nth=14)
5. `LastLedgerSequence` (tc=2, nth=27)
6. `Amount` (tc=6, nth=1)
7. `Fee` (tc=6, nth=8)
8. `SigningPubKey` (tc=7, nth=3)
9. `TxnSignature` (tc=7, nth=4)
10. `Account` (tc=8, nth=1)
11. `Destination` (tc=8, nth=3)

---

## 4. Type Serialization

### 4.1 Fixed-Width Integer Types

All integers are serialized in **big-endian** (network byte order) with no length
prefix.

| Type      | type_code | Width   | Byte Layout                                  |
|-----------|----------:|--------:|----------------------------------------------|
| `UInt8`   |        16 | 1 byte  | `[u8]`                                       |
| `UInt16`  |         1 | 2 bytes | `[hi, lo]`                                   |
| `UInt32`  |         2 | 4 bytes | `[b3, b2, b1, b0]` (b3 = MSB)               |
| `UInt64`  |         3 | 8 bytes | `[b7, b6, ..., b0]` (b7 = MSB)              |
| `Int32`   |        10 | 4 bytes | `[b3, b2, b1, b0]` signed, two's complement |
| `Int64`   |        11 | 8 bytes | `[b7, b6, ..., b0]` signed, two's complement |
| `UInt96`  |        20 | 12 bytes| `[b11, b10, ..., b0]`                        |
| `UInt384` |        22 | 48 bytes| `[b47, b46, ..., b0]`                        |
| `UInt512` |        23 | 64 bytes| `[b63, b62, ..., b0]`                        |
| `Number`  |         9 | 8 bytes | XFL (eXtended FLoating point) format         |

Example -- `Flags` field with value `0x00000000` (no flags set):
```
Field header: 0x22                   (tc=2, nth=2, Case 1)
Value:        0x00 0x00 0x00 0x00    (UInt32, 4 bytes BE)
Full:  22 00000000
```

Example -- `Sequence` field with value 42:
```
Field header: 0x24                   (tc=2, nth=4, Case 1)
Value:        0x00 0x00 0x00 0x2A    (42 in big-endian)
Full:  24 0000002A
```

### 4.2 Fixed-Width Hash Types

Hashes are fixed-size byte arrays with no length prefix. They are serialized
as-is in their natural byte order.

| Type      | type_code | Width    |
|-----------|----------:|---------:|
| `Hash128` |         4 | 16 bytes |
| `Hash160` |        17 | 20 bytes |
| `Hash192` |        21 | 24 bytes |
| `Hash256` |         5 | 32 bytes |

Example fields:
- `EmailHash` (Hash128, nth=1): 16 bytes, header `0x41`
- `TakerPaysCurrency` (Hash160, nth=1): 20 bytes, header `[0x01, 0x11]` (tc=17 >= 16, Case 3)
- `MPTokenIssuanceID` (Hash192, nth=1): 24 bytes, header `[0x01, 0x15]` (tc=21 >= 16, Case 3)
- `LedgerHash` (Hash256, nth=1): 32 bytes, header `0x51`

### 4.3 Variable-Length (VL) Encoding

Types where `isVLEncoded == true` are prefixed with a length indicator before the
content bytes. The length prefix itself uses a variable-width encoding scheme:

```
Length 0 to 192 (inclusive):
  1 byte:  [length]

Length 193 to 12480:
  2 bytes: [byte1, byte2]
  length = 193 + ((byte1 - 193) * 256) + byte2

Length 12481 to 918744:
  3 bytes: [byte1, byte2, byte3]
  length = 12481 + ((byte1 - 241) * 65536) + (byte2 * 256) + byte3
```

**Decoding the ranges:**

| Byte 1 Range | Prefix Bytes | Length Range       | Formula                                                  |
|-------------:|-----------:|--------------------|----------------------------------------------------------|
|     0 -- 192 |            1 | 0 -- 192           | length = byte1                                           |
|   193 -- 240 |            2 | 193 -- 12480       | length = 193 + (byte1 - 193) * 256 + byte2               |
|   241 -- 254 |            3 | 12481 -- 918744    | length = 12481 + (byte1 - 241) * 65536 + byte2 * 256 + byte3 |

**Encoding the ranges (inverse):**

```
if length <= 192:
    emit [length]

elif length <= 12480:
    adjusted = length - 193
    byte1 = 193 + (adjusted >> 8)
    byte2 = adjusted & 0xFF
    emit [byte1, byte2]

elif length <= 918744:
    adjusted = length - 12481
    byte1 = 241 + (adjusted >> 16)
    byte2 = (adjusted >> 8) & 0xFF
    byte3 = adjusted & 0xFF
    emit [byte1, byte2, byte3]
```

VL-encoded types in definitions.json:
- **`Blob`** (tc=7) -- e.g., `SigningPubKey`, `TxnSignature`, `Fulfillment`, `Condition`, `Domain`, `MPTokenMetadata`, `CredentialType`
- **`AccountID`** (tc=8) -- e.g., `Account`, `Destination`, `Issuer`, `Owner` (content is always 20 bytes)
- **`Vector256`** (tc=19) -- e.g., `Indexes`, `Hashes`, `Amendments`, `NFTokenOffers`, `CredentialIDs`

Example -- `Account` field with a 20-byte account ID:
```
Field header:  0x81        (tc=8, nth=1, Case 1)
VL prefix:     0x14        (20 in decimal, fits in 1 byte)
Content:       <20 bytes of AccountID>
Full:  81 14 <20 bytes>
```

Example -- `SigningPubKey` field with a 33-byte secp256k1 public key:
```
Field header:  0x73        (tc=7, nth=3, Case 1)
VL prefix:     0x21        (33 in decimal)
Content:       <33 bytes of compressed public key>
Full:  73 21 <33 bytes>
```

### 4.4 Vector256

`Vector256` (tc=19) is VL-encoded. The content is a concatenation of Hash256
values (each 32 bytes). The VL prefix encodes the total byte length (which
must be a multiple of 32).

Example -- `Amendments` field with 2 amendment hashes:
```
Field header:  [0x03, 0x13]     (tc=19 >= 16, nth=3 < 16, Case 3)
VL prefix:     0x40              (64 bytes = 2 * 32)
Content:       <32 bytes hash 1> <32 bytes hash 2>
```

---

## 5. Amount Encoding

The `Amount` type (tc=6) is polymorphic. The first byte determines the format:

- **XRP amount**: 8 bytes total (bit 63 = 0)
- **IOU (issued currency) amount**: 48 bytes total (bit 63 = 1, with non-zero mantissa or specific zero pattern)
- **MPT amount**: 32 bytes total (bit 63 = 1, with MPT indicator; added by MPT amendment)

### 5.1 XRP Amount (8 bytes)

XRP amounts are stored as drops (1 XRP = 1,000,000 drops) in a 64-bit value
with special bit layout:

```
Bit 63 (MSB):  0          (marks this as XRP, not IOU/MPT)
Bit 62:        sign       (1 = positive, 0 = negative)
Bits 61-0:     drops      (absolute value, max 10^17)
```

**Important**: Positive amounts have bit 62 SET (= 1). This is the opposite
of standard two's complement. The "not XRP" flag in bit 63 and the positive
flag in bit 62 together mean the first byte of a positive XRP amount is
always `0x40` + high bits of drops.

#### Concrete Examples

**Positive 1 XRP (1,000,000 drops):**
```
drops = 1000000 = 0x0000_0000_000F_4240
With bit 62 set: 0x4000_0000_000F_4240

Bytes: 40 00 00 00 00 0F 42 40
```

**Positive 0 XRP:**
```
Canonical zero: 0x4000_0000_0000_0000

Bytes: 40 00 00 00 00 00 00 00
```

**Negative 1 XRP (negative amounts appear in metadata):**
```
drops = 1000000 = 0x0000_0000_000F_4240
Bit 62 NOT set (negative): 0x0000_0000_000F_4240

Bytes: 00 00 00 00 00 0F 42 40
```

### 5.2 IOU Amount (48 bytes)

Issued currency amounts use a custom floating-point representation followed by
currency code and issuer.

```
Bytes 0-7:   Amount value (8 bytes, custom float)
Bytes 8-27:  Currency code (20 bytes)
Bytes 28-47: Issuer AccountID (20 bytes)
```

#### Amount Value (8-byte Custom Float)

```
Bit 63 (MSB):  1           (marks as "not XRP")
Bit 62:        sign        (1 = positive, 0 = negative)
Bits 61-54:    exponent    (8 bits, biased by 97; range -96 to +80)
Bits 53-0:     mantissa    (54 bits)
```

The mantissa is normalized so that the most significant digit is always
between 1 and 9 (i.e., `10^15 <= mantissa < 10^16` when non-zero). This
gives 16 significant decimal digits.

- **Exponent bias**: The stored exponent has 97 added. So stored value 97
  means actual exponent 0.
- **Actual value**: `(-1)^sign * mantissa * 10^(exponent - 97)`
- **Zero**: Canonical zero has bits 63=1, 62=0, and all other bits 0.
  Byte representation: `0x80 0x00 0x00 0x00 0x00 0x00 0x00 0x00`

#### Currency Code (20 bytes)

There are two formats for the 20-byte currency code:

**Standard currency (3-character ASCII):**
```
Bytes 0-11:  0x00 (12 zero bytes)
Bytes 12-14: 3 ASCII characters (e.g., "USD" = 0x55 0x53 0x44)
Bytes 15-19: 0x00 (5 zero bytes)
```

The first byte MUST be 0x00 to distinguish from non-standard codes.

**Non-standard (160-bit) currency:**
```
Bytes 0-19: arbitrary 20 bytes, where byte 0 != 0x00
```

**XRP currency code (special case):**
```
All 20 bytes = 0x00
```

#### IOU Concrete Example

1.5 USD issued by rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh:

```
Sign: positive (bit 62 = 1)
Mantissa: 1500000000000000 (1.5 * 10^15, normalized)
Exponent: -15 + 97 = 82 (0x52 stored)

Amount bits:
  Bit 63 = 1 (not XRP)
  Bit 62 = 1 (positive)
  Bits 61-54 = 01010010 (82 = 0x52, but shifted: exponent field)
  Mantissa = 1500000000000000

Currency: 0x0000000000000000000000005553440000000000 (USD)
Issuer: 20-byte AccountID of rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh
```

The full 48-byte IOU amount is the concatenation of these three parts.

#### IOU Zero

The canonical zero for IOU amounts (with currency and issuer):
```
Amount: 0x80 0x00 0x00 0x00 0x00 0x00 0x00 0x00
Currency: <20 bytes currency code>
Issuer: <20 bytes issuer AccountID>
```
Total: 48 bytes. Note that even for zero, the currency and issuer are present.

### 5.3 MPT Amount (32 bytes)

Multi-Purpose Token amounts were introduced with the MPT amendment. They use
a distinct encoding within the Amount type.

```
Byte 0, bit 7:   1          (not XRP)
Byte 0, bit 6:   sign       (1 = positive, 0 = negative)
Byte 0, bit 5:   1          (MPT indicator -- distinguishes from IOU)
Bytes 0-7:       Value (64-bit integer, with top 3 bits as above)
Bytes 8-31:      MPTokenIssuanceID (24 bytes = Hash192)
```

The MPT indicator bit (bit 5 of byte 0, i.e., bit 61 of the 64-bit value)
being set to 1 distinguishes MPT amounts from IOU amounts where that bit is
part of the exponent. For IOU amounts, the exponent field occupies bits 61-54,
but the protocol ensures valid IOU exponent values do not collide with the MPT
pattern.

Total: 8 bytes (value) + 24 bytes (MPTokenIssuanceID) = **32 bytes**.

**Positive MPT zero:**
```
Byte 0 = 0x60  (bits: 0110_0000 -- not XRP, positive, MPT flag)
Bytes 1-7 = 0x00
Bytes 8-31 = <24 bytes MPTokenIssuanceID>
```

---

## 6. Signing Serialization

### 6.1 Hash Prefixes

Before hashing, a 4-byte prefix is prepended to identify the purpose:

| Purpose                  | ASCII  | Hex Prefix     |
|--------------------------|--------|----------------|
| Transaction signing      | `STX\0`| `0x53545800`   |
| Multi-sign signing       | `SMT\0`| `0x534D5400`   |
| Transaction ID (hash)   | `TXN\0`| `0x54584E00`   |
| Inner transaction (batch)| `STX\0`| `0x53545800`   |

Additional prefixes (used for ledger objects, validations, etc.):

| Purpose                  | ASCII  | Hex Prefix     |
|--------------------------|--------|----------------|
| Ledger node ID           | `MLN\0`| `0x4D4C4E00`   |
| Inner node (SHAMap)      | `MIN\0`| `0x4D494E00`   |
| Ledger master hash       | `LWR\0`| `0x4C575200`   |
| Validation signing       | `VAL\0`| `0x56414C00`   |
| Proposal signing         | `PRP\0`| `0x50525000`   |

### 6.2 SHA-512/Half

XRPL uses **SHA-512/Half** for all hashing: compute SHA-512 of the input, then
take the first 256 bits (32 bytes) of the output. This gives 256-bit hashes
with the security properties of SHA-512.

### 6.3 Signing Data Construction

**Single signature:**
```
signing_data = 0x53545800 || serialize_for_signing(tx)
hash = SHA-512/Half(signing_data)
signature = sign(hash, private_key)
```

**Multi-signature:**
```
signing_data = 0x534D5400 || serialize_for_signing(tx) || signer_account_id
hash = SHA-512/Half(signing_data)
signature = sign(hash, private_key)
```

The `signer_account_id` is the 20-byte AccountID of the signer (NOT VL-encoded,
just raw 20 bytes appended after the transaction body).

**Transaction ID:**
```
tx_blob = serialize_full(tx)    (includes TxnSignature, Signers)
tx_id = SHA-512/Half(0x54584E00 || tx_blob)
```

### 6.4 Signing Fields vs. Non-Signing Fields

When serializing a transaction for signing (`serialize_for_signing`), only
fields with `isSigningField == true` are included. The critical non-signing
fields that are EXCLUDED from the signing serialization:

| Field Name               | Type       | nth | isSigningField | Notes                           |
|--------------------------|------------|----:|:--------------:|---------------------------------|
| `TxnSignature`           | Blob       |   4 | false          | The signature itself            |
| `Signers`                | STArray    |   3 | false          | Multi-sign signature array      |
| `Signature`              | Blob       |   6 | false          | Validation signatures           |
| `MasterSignature`        | Blob       |  18 | false          | Master key signatures           |
| `BatchSigners`           | STArray    |  31 | false          | Batch transaction signers       |
| `CounterpartySignature`  | STObject   |  37 | false          | Loan counterparty signature     |

**Key fact**: `SigningPubKey` (Blob, nth=3) **IS** a signing field
(`isSigningField == true`). It is included in the signing serialization even
though it carries signer identity. When signing, the `SigningPubKey` field
is present (set to the signer's public key for single-sign, or empty bytes
for multi-sign) and the `TxnSignature` field is omitted entirely.

### 6.5 Signing Serialization Process

1. Collect all fields of the transaction where `isSigningField == true`
2. Sort them in canonical order (by type_code, then field_code)
3. Serialize each field (header + value) in order
4. Prepend the appropriate hash prefix
5. Compute SHA-512/Half of the result

---

## 7. STObject and STArray Nesting

### 7.1 STObject (type_code = 14)

An STObject is serialized as:
1. The field header identifying which STObject field this is
2. The inner fields, serialized in canonical order
3. The **object end marker**: byte `0xE1`

The end marker `0xE1` encodes as a field header with type_code=14, nth=1
(which is `ObjectEndMarker` in definitions.json). Using Case 1 encoding:
`(14 << 4) | 1 = 0xE1`.

**Exception**: The outermost transaction or ledger object is also an STObject
but does NOT have a surrounding field header or end marker -- it is the root
container. Only *inner* (nested) STObjects have the header and end marker.

### 7.2 STArray (type_code = 15)

An STArray is serialized as:
1. The field header identifying which STArray field this is
2. For each element in the array:
   a. The element's field header (always an STObject field, e.g., `Memo`, `SignerEntry`)
   b. The inner fields of that element in canonical order
   c. Object end marker `0xE1`
3. The **array end marker**: byte `0xF1`

The end marker `0xF1` encodes as a field header with type_code=15, nth=1
(which is `ArrayEndMarker`). Using Case 1 encoding:
`(15 << 4) | 1 = 0xF1`.

### 7.3 Concrete Example: Memos

Consider a transaction with one memo containing a MemoType and MemoData:

```json
{
  "Memos": [
    {
      "Memo": {
        "MemoType": "746578742F706C61696E",
        "MemoData": "48656C6C6F"
      }
    }
  ]
}
```

Field definitions involved:
- `Memos`: STArray, nth=9, header = `(15 << 4) | 9 = 0xF9`
- `Memo`: STObject, nth=10, header = `(14 << 4) | 10 = 0xEA`
- `MemoType`: Blob, nth=12, header = `(7 << 4) | 12 = 0x7C`
- `MemoData`: Blob, nth=13, header = `(7 << 4) | 13 = 0x7D`

Binary serialization:
```
F9                          -- Memos array header (STArray, nth=9)
  EA                        --   Memo object header (STObject, nth=10)
    7C                      --     MemoType field header (Blob, nth=12)
    0A                      --     VL prefix: 10 bytes
    746578742F706C61696E    --     "text/plain" in hex
    7D                      --     MemoData field header (Blob, nth=13)
    05                      --     VL prefix: 5 bytes
    48656C6C6F              --     "Hello" in hex
  E1                        --   Memo object end marker
F1                          -- Memos array end marker
```

Note the canonical ordering within the Memo object: MemoType (Blob, nth=12)
comes before MemoData (Blob, nth=13) because both have the same type_code (7)
and 12 < 13. If MemoFormat (Blob, nth=14) were also present, it would come
after MemoData.

### 7.4 Nested Object Example: Signers

```json
{
  "Signers": [
    {
      "Signer": {
        "Account": "r...",
        "TxnSignature": "...",
        "SigningPubKey": "..."
      }
    }
  ]
}
```

Field definitions:
- `Signers`: STArray, nth=3, header = `(15 << 4) | 3 = 0xF3`
- `Signer`: STObject, nth=16, header = `(14 << 4) | 0 = 0xE0` + `0x10` (Case 2: tc=14 < 16, fc=16 >= 16)

Binary:
```
F3                      -- Signers array header
  E0 10                 --   Signer object header (STObject, nth=16, Case 2)
    73 21 <33 bytes>    --     SigningPubKey (Blob nth=3, VL=33)
    74 XX <sig bytes>   --     TxnSignature (Blob nth=4, VL=variable)
    81 14 <20 bytes>    --     Account (AccountID nth=1, VL=20)
  E1                    --   Signer object end marker
F1                      -- Signers array end marker
```

---

## 8. PathSet Encoding

The `PathSet` type (tc=18) uses a custom encoding for payment paths. A PathSet
is a collection of alternative payment paths, where each path is a sequence of
steps through the XRPL order book.

### 8.1 Path Step Encoding

Each step in a path consists of up to three optional components, indicated by
a **type byte** (bitfield):

| Bit  | Hex   | Component           | Size     |
|-----:|------:|---------------------|----------|
|    0 | 0x01  | Account (AccountID) | 20 bytes |
|    1 | 0x02  | Currency code       | 20 bytes |
|    2 | 0x04  | Issuer (AccountID)  | 20 bytes |

A step is serialized as:
1. Type byte (1 byte)
2. If bit 0 set: 20-byte AccountID
3. If bit 1 set: 20-byte currency code
4. If bit 2 set: 20-byte issuer AccountID

Steps can have any combination of the three flags. The size of a step is
1 byte (type) + 0, 20, 40, or 60 bytes depending on which flags are set.

### 8.2 Path Boundaries and End Marker

| Byte   | Meaning                                |
|-------:|----------------------------------------|
| `0xFF` | Path boundary (separates alternative paths) |
| `0x00` | End of PathSet                         |

### 8.3 Full PathSet Layout

```
[step] [step] ... 0xFF [step] [step] ... 0xFF [step] ... 0x00
|--- path 1 ---|       |--- path 2 ---|       |--- path N ---|
```

### 8.4 Concrete Example

A PathSet with two paths:
- Path 1: through currency USD issued by rIssuer
- Path 2: through account rIntermediary

```
Field header: [0x01, 0x12]   (tc=18 >= 16, nth=1 < 16, Case 3)

Path 1, Step 1 (currency + issuer):
  06                              -- type byte: 0x02 | 0x04 = 0x06
  <20 bytes USD currency code>
  <20 bytes rIssuer AccountID>

FF                                -- path boundary

Path 2, Step 1 (account only):
  01                              -- type byte: 0x01
  <20 bytes rIntermediary AccountID>

00                                -- end of PathSet
```

---

## 9. Newer Types: Issue, XChainBridge, Currency

These types were added in recent protocol amendments and use type codes >= 16,
requiring the 2-byte or 3-byte field header encoding.

### 9.1 Issue (type_code = 24)

The `Issue` type represents a currency specification (currency + optional issuer).
It is used by AMM and XChainBridge fields.

Fields using Issue type:
- `LockingChainIssue` (nth=1)
- `IssuingChainIssue` (nth=2)
- `Asset` (nth=3)
- `Asset2` (nth=4)

**Encoding:**

For **XRP** (issuer-less):
```
<20 bytes: all zeros (XRP currency code)>
```
Total: 20 bytes

For **issued currencies**:
```
<20 bytes: currency code>
<20 bytes: issuer AccountID>
```
Total: 40 bytes

The deserializer determines the format by checking whether the 20-byte
currency code is all zeros (XRP) or not (issued currency with issuer following).

**Note**: The Issue type is NOT VL-encoded. The length is implicitly determined
by the currency code content.

Field header example for `Asset` (tc=24, nth=3):
```
tc=24 >= 16, fc=3 < 16 --> Case 3
Byte 0 = 0x00 | 3 = 0x03
Byte 1 = 0x18 (24)
Header: [0x03, 0x18]
```

### 9.2 XChainBridge (type_code = 25)

The `XChainBridge` type is a composite that describes a cross-chain bridge
configuration. There is one field using this type:
- `XChainBridge` (nth=1)

**Encoding:**

```
<20 bytes: LockingChainDoor AccountID (VL-encoded with 0x14 prefix)>
<20 or 40 bytes: LockingChainIssue (Issue encoding)>
<20 bytes: IssuingChainDoor AccountID (VL-encoded with 0x14 prefix)>
<20 or 40 bytes: IssuingChainIssue (Issue encoding)>
```

The XChainBridge is serialized as a sequence of its inner components. Each
AccountID component uses VL-encoding (1-byte length prefix of 0x14 = 20),
and each Issue component uses the Issue encoding described above.

Field header for `XChainBridge` (tc=25, nth=1):
```
tc=25 >= 16, fc=1 < 16 --> Case 3
Byte 0 = 0x00 | 1 = 0x01
Byte 1 = 0x19 (25)
Header: [0x01, 0x19]
```

### 9.3 Currency (type_code = 26)

The `Currency` type represents a bare currency code without an issuer. It is
used for Oracle price data fields.

Fields using Currency type:
- `BaseAsset` (nth=1)
- `QuoteAsset` (nth=2)

**Encoding:**
```
<20 bytes: currency code>
```

Fixed 20 bytes, using the same currency code format as in IOU amounts:
- Standard 3-char currency: 12 zero bytes + 3 ASCII bytes + 5 zero bytes
- Non-standard: 20 arbitrary bytes (first byte != 0x00)
- XRP: 20 zero bytes

Field header example for `BaseAsset` (tc=26, nth=1):
```
tc=26 >= 16, fc=1 < 16 --> Case 3
Byte 0 = 0x00 | 1 = 0x01
Byte 1 = 0x1A (26)
Header: [0x01, 0x1A]
```

---

## Appendix A: Quick Reference -- All Serializable Type Codes

| Code | Type Name      | Fixed Size | VL-Encoded | Notes                     |
|-----:|----------------|------------|:----------:|---------------------------|
|    1 | UInt16         | 2 bytes    | No         |                           |
|    2 | UInt32         | 4 bytes    | No         |                           |
|    3 | UInt64         | 8 bytes    | No         |                           |
|    4 | Hash128        | 16 bytes   | No         |                           |
|    5 | Hash256        | 32 bytes   | No         |                           |
|    6 | Amount         | 8/48/32    | No         | XRP/IOU/MPT               |
|    7 | Blob           | Variable   | Yes        |                           |
|    8 | AccountID      | 20 bytes   | Yes        | Content always 20 bytes   |
|    9 | Number         | 8 bytes    | No         | XFL format (lending)      |
|   10 | Int32          | 4 bytes    | No         | Signed                    |
|   11 | Int64          | 8 bytes    | No         | Signed                    |
|   14 | STObject       | Variable   | No         | End marker 0xE1           |
|   15 | STArray        | Variable   | No         | End marker 0xF1           |
|   16 | UInt8          | 1 byte     | No         |                           |
|   17 | Hash160        | 20 bytes   | No         |                           |
|   18 | PathSet        | Variable   | No         | Custom encoding           |
|   19 | Vector256      | Variable   | Yes        | Multiple Hash256 values   |
|   20 | UInt96         | 12 bytes   | No         |                           |
|   21 | Hash192        | 24 bytes   | No         |                           |
|   22 | UInt384        | 48 bytes   | No         |                           |
|   23 | UInt512        | 64 bytes   | No         |                           |
|   24 | Issue          | 20/40      | No         | XRP or currency+issuer    |
|   25 | XChainBridge   | Variable   | No         | Composite                 |
|   26 | Currency       | 20 bytes   | No         | Bare currency code        |

## Appendix B: Implementation Notes for xrpl-mithril

### Field ID Computation

```rust
fn encode_field_id(type_code: u16, field_code: u16) -> Vec<u8> {
    match (type_code < 16, field_code < 16) {
        (true, true) => {
            // Case 1: both fit in nibble
            vec![(type_code as u8) << 4 | field_code as u8]
        }
        (true, false) => {
            // Case 2: type fits, field needs extra byte
            vec![(type_code as u8) << 4, field_code as u8]
        }
        (false, true) => {
            // Case 3: field fits, type needs extra byte
            vec![field_code as u8, type_code as u8]
        }
        (false, false) => {
            // Case 4: both need extra bytes
            vec![0x00, type_code as u8, field_code as u8]
        }
    }
}
```

### Amount Type Discrimination

When deserializing an Amount field, the first byte determines the format:

```
if bit 63 == 0:
    -> XRP amount (8 bytes total)
elif bit 61 == 1:
    -> MPT amount (32 bytes total: 8 + 24)
else:
    -> IOU amount (48 bytes total: 8 + 20 + 20)
```

### VL Length Encoding

```rust
fn encode_vl_length(length: usize) -> Vec<u8> {
    if length <= 192 {
        vec![length as u8]
    } else if length <= 12480 {
        let adjusted = length - 193;
        vec![193 + (adjusted >> 8) as u8, (adjusted & 0xFF) as u8]
    } else if length <= 918744 {
        let adjusted = length - 12481;
        vec![
            241 + (adjusted >> 16) as u8,
            ((adjusted >> 8) & 0xFF) as u8,
            (adjusted & 0xFF) as u8,
        ]
    } else {
        panic!("VL length exceeds maximum of 918744");
    }
}
```

### End Markers

```rust
const OBJECT_END_MARKER: u8 = 0xE1;  // STObject end (tc=14, nth=1)
const ARRAY_END_MARKER: u8 = 0xF1;   // STArray end (tc=15, nth=1)
const PATH_BOUNDARY: u8 = 0xFF;      // PathSet path separator
const PATHSET_END: u8 = 0x00;        // PathSet end marker
```

### Hash Prefixes

```rust
const HASH_PREFIX_TRANSACTION_SIGN: [u8; 4] = [0x53, 0x54, 0x58, 0x00]; // STX\0
const HASH_PREFIX_MULTI_SIGN: [u8; 4]       = [0x53, 0x4D, 0x54, 0x00]; // SMT\0
const HASH_PREFIX_TRANSACTION_ID: [u8; 4]   = [0x54, 0x58, 0x4E, 0x00]; // TXN\0
```

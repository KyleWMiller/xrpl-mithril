# Architecture Decision Record — xrpl-oxide

This document captures key architectural decisions for the xrpl-oxide project. Each decision follows a structured template to preserve context, alternatives considered, and consequences.

Decisions are numbered sequentially and should not be renumbered. Superseded decisions get their status updated to "Superseded by ADR-XX" rather than being deleted.

---

## ADR-001: Decimal Representation for Issued Currency Amounts

**Date**: 2026-02-24
**Status**: Accepted

### Context

XRPL issued currency amounts use a custom floating-point format on the wire: a 54-bit mantissa (normalized to the range 10^15 to 10^16 - 1) combined with an 8-bit biased exponent (range -96 to +80). The sign bit is separate. This is not IEEE 754 — it is a bespoke format designed for deterministic ledger consensus.

We need a Rust type that can faithfully represent every valid XRPL issued currency amount, reject every invalid one, and convert efficiently to and from the binary wire format.

### Options Considered

1. **String-based representation (like xrpl-py / xrpl-rust)** — Store amounts as `String` and parse on demand. This is what existing SDKs do. The problem is that a `String` can hold literally anything: `"banana"`, `""`, `"99999999999999999999999999999"`. Every function that receives an amount must re-validate it. Invalid states are freely representable, which contradicts the project's core design principle.

2. **`rust_decimal::Decimal`** — A general-purpose 128-bit decimal type. It can represent values outside the XRPL-valid range (exponents beyond -96..+80, mantissas beyond 54 bits), so validation is still needed at every boundary. It is heap-free but 16 bytes versus our 9-byte wire format. Conversion to/from the wire format requires non-trivial normalization logic. Arithmetic is convenient but may silently produce out-of-range results.

3. **Custom `IssuedValue { mantissa: i64, exponent: i8 }` matching the wire format** — The internal representation is the wire format. Construction validates invariants (mantissa range, exponent range, normalization). The type is `Copy`, 9 bytes, and converts to binary with zero computation. Invalid states are unrepresentable because the only way to create an `IssuedValue` is through validated constructors.

### Decision

Option 3: Custom `IssuedValue { mantissa: i64, exponent: i8 }`.

The internal representation mirrors the XRPL wire format directly. All invariants are enforced at construction time:

- Mantissa is zero (for the zero value) or in the range `1_000_000_000_000_000..=9_999_999_999_999_999`
- Exponent is in the range `-96..=80`
- Sign is encoded in the mantissa's sign

Binary serialization is a direct bitfield pack/unpack with no intermediate conversion. JSON serialization produces the string format that XRPL JSON-RPC expects.

For users who need decimal arithmetic (computing exchange rates, aggregating balances), the `rust_decimal` crate is available behind a `decimal-arithmetic` feature gate. Conversion between `IssuedValue` and `rust_decimal::Decimal` is provided, with range-checked conversion back.

### Consequences

- Serialization to/from binary is zero-cost — the struct fields ARE the wire fields.
- The type is `Copy` and stack-allocated, enabling efficient pass-by-value.
- Arithmetic is not natively supported on `IssuedValue` — users must convert to `Decimal` or implement their own logic. This is intentional: the SDK's job is faithful protocol representation, not a math library.
- Every `IssuedValue` in memory is guaranteed valid. Functions that accept `IssuedValue` never need to re-validate.
- Adding arithmetic support later (via trait implementations) is backward-compatible.

---

## ADR-002: Transaction Enum Strategy

**Date**: 2026-02-24
**Status**: Accepted

### Context

XRPL has over 70 transaction types as of rippled v3.1.0, and the protocol adds new types with each major amendment. The SDK needs a Rust representation that:

- Allows typed access to transaction-specific fields
- Supports serialization/deserialization to JSON and binary
- Is forward-compatible with new transaction types added by future amendments
- Enables compile-time completeness checking when handling transactions

### Options Considered

1. **Trait-based polymorphism (`dyn Transaction`)** — Define a `Transaction` trait, implement it for each transaction struct. Use `Box<dyn Transaction>` for heterogeneous collections. This introduces dynamic dispatch overhead, makes deserialization complex (need a type registry to reconstruct concrete types), and loses compile-time exhaustiveness checking. Pattern matching requires downcasting.

2. **`#[non_exhaustive] enum Transaction` with one variant per type** — A single enum with variants like `Payment(Payment)`, `OfferCreate(OfferCreate)`, etc. Each variant wraps a dedicated struct. `#[non_exhaustive]` ensures downstream code includes a wildcard arm, so adding new variants is not a breaking change.

3. **Flat structs with a `TransactionType` discriminator** — Each transaction is a standalone struct with a `transaction_type: TransactionType` field. No unifying enum. This makes heterogeneous collections awkward (need `Box<dyn Any>` or similar), and deserialization requires manual dispatch on the type field.

### Decision

Option 2: `#[non_exhaustive] enum Transaction` with one variant per transaction type.

Each transaction type gets its own struct (e.g., `Payment`, `OfferCreate`, `EscrowCreate`) with strongly-typed fields. The `Transaction` enum wraps them all. Common fields (account, fee, sequence, signing_pub_key, etc.) are accessible via methods on the enum that delegate to the inner struct, or via a `CommonFields` struct embedded in each variant's type.

```rust
#[non_exhaustive]
pub enum Transaction {
    Payment(Payment),
    OfferCreate(OfferCreate),
    EscrowCreate(EscrowCreate),
    // ... 70+ variants
}
```

Serde deserialization uses the `TransactionType` field as an internally-tagged discriminator, mapping JSON like `{"TransactionType": "Payment", ...}` to the correct variant.

### Consequences

- `match` on `Transaction` gives compile-time exhaustiveness checking (with a required wildcard arm due to `#[non_exhaustive]`).
- Adding a new transaction type is a minor version bump, not a breaking change.
- Serde's tagged enum deserialization handles JSON dispatch automatically.
- Binary codec dispatch maps `TransactionType` codes to variants.
- The enum is large (sized to the largest variant). For hot paths that only care about one type, functions should accept the concrete struct directly rather than the enum.
- No dynamic dispatch, no heap allocation for the enum itself.

---

## ADR-003: Newtype Strategy for Domain Types

**Date**: 2026-02-24
**Status**: Accepted

### Context

XRPL's protocol has many values that share the same byte representation but carry entirely different semantics:

- `AccountId` and `Hash160` are both `[u8; 20]`
- `Hash256`, `UInt256`, and `LedgerIndex` (by hash) are all `[u8; 32]`
- `CurrencyCode` is `[u8; 20]` but with specific format constraints (standard vs non-standard codes)
- `Blob` and `SigningPubKey` are both `Vec<u8>`

Mixing these up is a protocol-level bug. Passing a `Hash160` where an `AccountId` is expected would produce a valid-looking but semantically wrong transaction.

### Options Considered

1. **Type aliases (`type AccountId = [u8; 20]`)** — Zero overhead, zero safety. The compiler treats `AccountId` and `Hash160` as identical. A function accepting `AccountId` happily takes a `Hash160` with no warning. This defeats the purpose of having named types.

2. **Strong newtypes (`pub struct AccountId([u8; 20])`)** — Each domain type is a distinct type that wraps its byte representation. The compiler rejects mixing them. Each type gets its own `Display`, `FromStr`, `Serialize`/`Deserialize`, and conversion implementations. Runtime cost is zero — the newtype is transparent in memory.

### Decision

Option 2: Strong newtypes for all domain-specific byte arrays and identifiers.

Every protocol-level identifier gets its own newtype struct:

```rust
pub struct AccountId([u8; 20]);
pub struct Hash256([u8; 32]);
pub struct Hash128([u8; 16]);
pub struct CurrencyCode([u8; 20]);
pub struct Blob(Vec<u8>);
```

Each newtype provides:

- `AsRef<[u8]>` and `as_bytes()` for raw access when needed
- `Display` / `FromStr` with format-appropriate encoding (Base58Check for AccountId, hex for hashes)
- `Serialize` / `Deserialize` matching XRPL JSON conventions
- Validated constructors where the type has format constraints (e.g., `CurrencyCode` enforces the 3-character standard code vs 160-bit non-standard code distinction)

### Consequences

- The compiler catches type confusion at zero runtime cost. Passing a `Hash256` where an `AccountId` is expected is a compile error.
- Each type can have domain-specific formatting. `AccountId` displays as Base58Check (`rXXX...`), `Hash256` displays as uppercase hex — both via `Display`, no ambiguity.
- Construction validation means every instance is guaranteed well-formed. A `CurrencyCode` always contains either a valid 3-character standard code or a valid 160-bit non-standard code.
- Ergonomic cost is real but manageable: explicit `.into()` or `.from()` calls are needed at boundaries. This is a feature, not a bug — it forces the developer to be intentional about type conversions.
- `#[repr(transparent)]` on newtypes ensures FFI compatibility and guarantees the compiler does not add padding.

---

## ADR-004: Signed vs Unsigned Transaction Types

**Date**: 2026-02-24
**Status**: Accepted

### Context

Transaction signing is the critical security boundary in any XRPL SDK. The workflow is:

1. Build an unsigned transaction (set fields, autofill fee/sequence/last_ledger_sequence)
2. Sign it (produce `TxnSignature` and `SigningPubKey` fields)
3. Submit the signed transaction to the network

Submitting an unsigned transaction is always a bug. Signing an already-signed transaction is at best wasteful, at worst a security concern. The type system should make these mistakes impossible.

### Options Considered

1. **Single `Transaction` type with optional signature fields** — `txn_signature: Option<Blob>`, `signing_pub_key: Option<Blob>`. Submission checks at runtime whether the fields are `Some`. This is how most XRPL SDKs work. It is simple but allows the mistake we want to prevent: nothing stops you from calling `submit()` on an unsigned transaction until runtime.

2. **Separate `UnsignedTransaction<T>` / `SignedTransaction<T>` wrapper types** — The signing function consumes an `UnsignedTransaction<T>` and returns a `SignedTransaction<T>`. Submission only accepts `SignedTransaction<T>`. The unsigned variant cannot be submitted because the method does not exist on its type.

### Decision

Option 2: Separate wrapper types that enforce the build-sign-submit workflow at the type level.

```rust
pub struct UnsignedTransaction<T: Signable> {
    inner: T,
    common: CommonFields,  // fee, sequence, etc. — no signature fields
}

pub struct SignedTransaction<T: Signable> {
    inner: T,
    common: CommonFields,
    signature: Blob,
    signing_pub_key: Blob,
}
```

The signing function's type signature enforces the state transition:

```rust
fn sign<T: Signable>(
    tx: UnsignedTransaction<T>,
    wallet: &Wallet,
) -> Result<SignedTransaction<T>, SigningError>;
```

Submission only accepts signed transactions:

```rust
async fn submit<T: Signable>(
    &self,
    tx: &SignedTransaction<T>,
) -> Result<SubmitResponse, ClientError>;
```

### Consequences

- Submitting an unsigned transaction is a compile error. The `submit` method simply does not exist for `UnsignedTransaction`.
- The signing workflow is self-documenting: the type signatures show the required sequence of operations.
- Multi-signing is supported by a separate `MultiSignedTransaction<T>` type that collects multiple `Signer` entries.
- Serialization differs by state: unsigned transactions serialize without signature fields (for signing hash computation), signed transactions include them (for submission).
- The generic parameter `T` preserves access to transaction-specific fields through all states. You can inspect a `SignedTransaction<Payment>` and still access `payment.destination`.
- Cost: slightly more complex API surface. Users must understand the type state pattern. Builder ergonomics and clear documentation mitigate this.

---

## ADR-005: Field Definitions Strategy

**Date**: 2026-02-24
**Status**: Accepted

### Context

The XRPL binary codec needs metadata about every field in the protocol: its type code, field code, whether it is serialized, whether it is signing-only, and its sort order within an object. This metadata is canonically defined in a `definitions.json` file maintained alongside rippled.

The codec must know field definitions to serialize and deserialize any XRPL object. The question is when and how to load this metadata.

### Options Considered

1. **Runtime-only (load JSON at startup, like xrpl-rust)** — Parse `definitions.json` into a `HashMap` at initialization. Simple but has startup cost, requires the JSON file to be bundled or fetched, and every field lookup goes through a hash map.

2. **Compile-time only (const arrays, no runtime loading)** — Use `const` arrays or a build script to generate Rust code from `definitions.json`. Zero startup cost, field lookups can be compile-time constants. Downside: if the protocol adds new fields, users must wait for an SDK release.

3. **Compile-time primary with optional runtime override** — Compile known fields as constants for performance. Provide a runtime registry (behind a feature gate) that can load a newer `definitions.json` to handle fields added by amendments that shipped after the SDK release.

### Decision

Option 3: Compile-time primary with optional runtime override.

The build pipeline works as follows:

- A build script (or proc macro) reads `definitions.json` and generates `const` field definitions as Rust code. These cover all fields known at SDK build time.
- Field lookup for known fields is a match on const values — no allocation, no hash map, zero cost.
- An optional `runtime-definitions` feature gate enables a `FieldRegistry` that can load a newer `definitions.json` at runtime. Lookups check the runtime registry first, then fall back to compile-time constants.
- The runtime registry is additive only — it cannot override or remove compile-time definitions, only add new ones.

### Consequences

- Default usage (no feature gate) has zero startup cost and zero allocation for field metadata. Field codes are inlined as constants by the compiler.
- Forward compatibility: users who encounter new protocol fields before an SDK release can load an updated `definitions.json` without waiting for a new crate version. This is critical for XRPL, where amendments can add fields at any time.
- The compile-time definitions serve as a correctness baseline. The runtime registry can only extend, not contradict, the compiled-in definitions.
- Build script adds a small amount of complexity to the build pipeline. This is a one-time cost that pays for itself in runtime performance and correctness.
- The generated code can include doc comments and type information derived from the JSON, improving IDE support.

---

## ADR-006: Crypto Backend Strategy

**Date**: 2026-02-24
**Status**: Accepted

### Context

XRPL supports two signature algorithms:

- **secp256k1 (ECDSA)**: The default and most common. Used by most existing accounts.
- **Ed25519**: The alternative, used by accounts whose key is derived with the `ed25519` family prefix.

Production-quality pure Rust implementations exist for both: `k256` (from the RustCrypto project) for secp256k1 and `ed25519-dalek` for Ed25519. C-backed alternatives also exist: the `secp256k1` crate (wrapping Bitcoin Core's `libsecp256k1`) is faster for high-throughput scenarios.

The project's stated goal is `#![forbid(unsafe_code)]` for our codebase. However, upstream crates may internally use unsafe (both `k256` and the C FFI crates do). The question is which backends to support and how.

### Options Considered

1. **Pure Rust only (`k256` + `ed25519-dalek`)** — Simplest dependency tree, aligns with pure Rust philosophy. Performance is adequate for SDK use (signing at human-interaction speed). Drawback: users who want maximum throughput (e.g., batch signing tools, load testing) cannot opt into faster backends.

2. **C FFI only (`secp256k1` C wrapper + `ed25519-dalek`)** — Maximum performance. Drawback: requires C compiler toolchain, harder cross-compilation, breaks the "pure Rust builds with `cargo build`" story.

3. **Feature-gated dual backend** — Default to pure Rust (`k256` + `ed25519-dalek`). Provide an opt-in `native-crypto` feature that switches to C-backed `secp256k1`. Signing code is generic over the backend via the `ecdsa` and `signature` trait crates.

### Decision

Option 3: Feature-gated dual backend.

The feature configuration:

- **`pure-rust-crypto` (default)**: Uses `k256` for secp256k1 ECDSA and `ed25519-dalek` for Ed25519. No C dependencies. Builds anywhere `cargo` does.
- **`native-crypto` (opt-in)**: Uses the `secp256k1` crate (C FFI to libsecp256k1) for secp256k1 ECDSA. Ed25519 stays on `ed25519-dalek` (it is already fast enough, and there is no standard C FFI crate for Ed25519 in the Rust ecosystem).

The signing layer is generic over the backend via the `signature::Signer` and `ecdsa::SigningKey` traits. Wallet code does not know or care which backend is active — it operates on trait bounds.

```rust
// Wallet code is backend-agnostic
pub fn sign_secp256k1(
    signing_key: &impl ecdsa::signature::Signer<ecdsa::Signature<Secp256k1>>,
    message: &[u8],
) -> Signature { ... }
```

### Consequences

- Default builds are pure Rust: `cargo build` with no C toolchain required. The project maintains `#![forbid(unsafe_code)]` for our own code regardless of which backend is selected.
- Users who need maximum secp256k1 performance (batch tools, benchmarks, integration into systems that already depend on `libsecp256k1`) can opt in with a single feature flag.
- The trait-based abstraction means the signing API is identical regardless of backend. Switching backends is a `Cargo.toml` change, not a code change.
- Testing must cover both backends in CI. A matrix build ensures both feature configurations are exercised.
- Future backends (e.g., hardware security module support via PKCS#11) can be added as additional feature gates without changing the signing API.
- The `native-crypto` feature does not compromise our `#![forbid(unsafe_code)]` guarantee. The C FFI is entirely within the upstream `secp256k1` crate. Our code only sees safe Rust trait implementations.

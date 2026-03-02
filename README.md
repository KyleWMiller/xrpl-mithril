<p align="center">
  <img src="https://raw.githubusercontent.com/KyleWMiller/xrpl-mithril/main/assets/mithrilLogo.png" alt="xrpl-mithril logo" width="200">
</p>

<h1 align="center">xrpl-mithril</h1>

<p align="center">
  <a href="https://crates.io/crates/xrpl-mithril"><img src="https://img.shields.io/crates/v/xrpl-mithril.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/xrpl-mithril"><img src="https://docs.rs/xrpl-mithril/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/KyleWMiller/xrpl-mithril/actions"><img src="https://github.com/KyleWMiller/xrpl-mithril/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License: MIT/Apache-2.0"></a>
  <a href="#minimum-supported-rust-version"><img src="https://img.shields.io/badge/MSRV-1.85-orange.svg" alt="MSRV: 1.85"></a>
  <a href="https://github.com/rust-secure-code/safety-dance/"><img src="https://img.shields.io/badge/unsafe-forbidden-success.svg" alt="unsafe forbidden"></a>
</p>

**A next-generation, pure Rust SDK for the XRP Ledger.**

xrpl-mithril is a Rust-native SDK targeting the 2026 XRPL protocol surface -- rippled v3.1.0 and beyond. It covers 50+ transaction types including Multi-Purpose Tokens, Token Escrow (XLS-85), AMM, Credentials, DynamicNFT, and every other mainnet feature through February 2026. The entire codebase enforces `#![forbid(unsafe_code)]` and builds on any platform without a C toolchain. Swap in the native `libsecp256k1` backend with a single feature flag when you need maximum throughput.

## Quick Start

Send 10 XRP on testnet:

```rust
use xrpl_mithril::client::JsonRpcClient;
use xrpl_mithril::tx::builder::PaymentBuilder;
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::{sign_transaction, submit_and_wait};
use xrpl_mithril::types::{Amount, XrpAmount};
use xrpl_mithril::wallet::{Algorithm, Wallet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate wallets
    let sender = Wallet::generate(Algorithm::Ed25519)?;
    let receiver = Wallet::generate(Algorithm::Secp256k1)?;

    // 2. Build a Payment transaction
    let mut unsigned = PaymentBuilder::new()
        .account(*sender.account_id())
        .destination(*receiver.account_id())
        .amount(Amount::Xrp(XrpAmount::from_drops(10_000_000)?)) // 10 XRP
        .build()?;

    // 3. Connect and autofill (fee, sequence, last ledger sequence)
    let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
    autofill(&client, &mut unsigned).await?;

    // 4. Sign -- produces a SignedTransaction (can't submit unsigned!)
    let signed = sign_transaction(&unsigned, &sender)?;

    // 5. Submit and wait for validation
    let result = submit_and_wait(&client, &signed).await?;
    println!("Validated in ledger {}: {}", result.ledger_index, result.result_code);
    Ok(())
}
```

See [17 runnable examples](examples/) covering payments, escrow, AMM, MPTs, NFTs, multi-sign, credentials, and more.

## Installation

**Default (pure Rust, no C toolchain required):**

```toml
[dependencies]
xrpl-mithril = "0.5.3"
```

**With native cryptography backend (for maximum secp256k1 performance):**

```toml
[dependencies]
xrpl-mithril = { version = "0.5.3", features = ["native-crypto"] }
```

By default, xrpl-mithril uses pure Rust cryptography (`k256` + `ed25519-dalek`). The `native-crypto` feature swaps in `libsecp256k1` via the `secp256k1` crate for ~2x faster ECDSA signing and verification. Both backends expose the identical API -- switching is a `Cargo.toml` change, not a code change.

### Individual Crates

For a smaller dependency footprint, depend on the crates you need:

| Crate | Re-export | Purpose | Standalone |
|-------|-----------|---------|:----------:|
| `xrpl-mithril-types` | `xrpl_mithril::types` | Core protocol types (AccountId, Amount, Hash, CurrencyCode) | Yes |
| `xrpl-mithril-codec` | `xrpl_mithril::codec` | Binary serialization/deserialization (rippled wire format) | Yes |
| `xrpl-mithril-models` | `xrpl_mithril::models` | 50+ transaction types, 17 ledger entry types | Yes |
| `xrpl-mithril-wallet` | `xrpl_mithril::wallet` | Key generation, signing, address encoding | Yes |
| `xrpl-mithril-client` | `xrpl_mithril::client` | JSON-RPC + WebSocket clients (rustls TLS) | Yes |
| `xrpl-mithril-tx` | `xrpl_mithril::tx` | Transaction building, autofill, reliable submission | No |
| `xrpl-mithril` | — | Facade: re-exports everything | — |

## Why xrpl-mithril?

### 1. Type-Level Safety

`UnsignedTransaction<T>` and `SignedTransaction<T>` enforce the build-sign-submit workflow at compile time. Submitting an unsigned transaction is not a runtime error -- it is a compile error. The method does not exist on the type.

### 2. Full 2026 Protocol Coverage

Targets rippled v3.1.0+. Covers Multi-Purpose Tokens (MPTs), Token Escrow for all asset types (XLS-85), DynamicNFT, AMM (XLS-30), Credentials, Price Oracles, and 50+ transaction types. Not a port of an older SDK -- researched from the rippled source and XRPL specification directly.

### 3. `#![forbid(unsafe_code)]`

Zero `unsafe` blocks in the entire codebase. Enforced at the workspace level via `[workspace.lints.rust] unsafe_code = "forbid"`. Dependencies may use unsafe internally (they are audited by their communities), but every line of xrpl-mithril code is provably safe.

### 4. Pure Rust by Default, Native by Choice

Builds on any platform with `cargo build`. No C compiler, no OpenSSL, no system dependencies. TLS via `rustls`, cryptography via RustCrypto. Opt into `libsecp256k1` when you need the throughput.

### 5. Transport-Agnostic Client

The `Client` trait abstracts over JSON-RPC and WebSocket transports. Write your application logic once, swap transports without changing code. WebSocket subscriptions provide real-time ledger streams.

### 6. Rust 2024 Edition

Edition 2024, MSRV 1.85. Uses `let_chains`, async closures, and the latest language features. This is not legacy Rust code getting maintained -- it is written for the Rust of today.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     xrpl-mithril                        │
│                (facade — re-exports all)                │
└──┬──────┬──────┬───────┬───────┬───────┬────────────────┘
   │      │      │       │       │       │
┌──▼──┐┌──▼──┐┌──▼───┐┌──▼───┐┌──▼────┐┌─▼──┐
│types││codec││models││wallet││client ││ tx │
│     ││     ││      ││      ││       ││    │
│     ││     ││      ││ k256 ││reqwest││    │
│     ││     ││      ││  or  ││  +    ││    │
│     ││     ││      ││secp- ││tungst-││    │
│     ││     ││      ││256k1 ││enite  ││    │
└──┬──┘└──┬──┘└──┬───┘└──┬───┘└──┬────┘└─┬──┘
   │      │      │       │       │       │
   │   ┌──┘      │       │       │       │
   │   │ depends │       │       │       │
   │   │ on types│       │       │       │
   └───┴─────────┘       │       │       │
                         │       │       │
              ┌──────────┴───────┴───────┘
              │ tx depends on
              │ client, wallet, models, codec
              └──────────────────────────────
```

**~15,750 lines of library code** across 7 crates, plus **~3,720 lines of examples** across 17 runnable demos. Every crate enforces `#![forbid(unsafe_code)]` and `clippy::unwrap_used = "deny"`.

## Feature Comparison

| Feature | xrpl-mithril | sephynox/xrpl-rust v0.5.0 |
|---------|:---:|:---:|
| Last updated | Feb 2026 | Aug 2025 |
| Rust edition | 2024 | 2021 |
| `#![forbid(unsafe_code)]` | Yes | No |
| Type-state signing (compile-time safety) | Yes | No |
| Multi-Purpose Tokens (MPTs) | Yes | No |
| Token Escrow (XLS-85) | Yes | No |
| DynamicNFT | Yes | No |
| AMM (XLS-30) | Yes | Partial |
| Credentials | Yes | No |
| Price Oracles | Yes | No |
| Transaction types covered | 50+ | ~25 |
| Ledger entry types | 17 | ~10 |
| Dual cryptography backends | Yes | No |
| WebSocket subscriptions | Yes | Yes |
| JSON-RPC client | Yes | Yes |
| TLS via rustls (no OpenSSL) | Yes | No |
| Fluent transaction builders | Yes | No |
| Autofill (fee/seq/LLS) | Yes | Yes |
| Reliable submission | Yes | Yes |
| Runnable examples | 17 | ~5 |

xrpl-mithril is not a fork or port of any existing SDK. It was designed from scratch by studying the rippled source code and XRPL protocol specification to provide idiomatic Rust coverage of the 2026 protocol surface.

## Examples

| Example | Description | Run |
|---------|-------------|-----|
| `account_info` | Read-only queries via JSON-RPC | `cargo run -p xrpl-mithril --example account_info` |
| `basic_payment` | Full XRP transfer lifecycle | `cargo run -p xrpl-mithril --example basic_payment` |
| `subscribe_ledger` | Real-time ledger stream via WebSocket | `cargo run -p xrpl-mithril --example subscribe_ledger` |
| `token_escrow` | Time-based escrow lifecycle (XLS-85) | `cargo run -p xrpl-mithril --example token_escrow` |
| `token_escrow_mpt` | MPT escrow lifecycle (XLS-85) | `cargo run -p xrpl-mithril --example token_escrow_mpt` |
| `mpt_operations` | MPT issuance and transfer | `cargo run -p xrpl-mithril --example mpt_operations` |
| `amm_lifecycle` | AMM create, deposit, withdraw (XLS-30) | `cargo run -p xrpl-mithril --example amm_lifecycle` |
| `nft_lifecycle` | NFT mint, offers, DynamicNFT (XLS-46) | `cargo run -p xrpl-mithril --example nft_lifecycle` |
| `check_operations` | Check create, cash, cancel | `cargo run -p xrpl-mithril --example check_operations` |
| `clawback` | Token issuer clawback (XLS-39) | `cargo run -p xrpl-mithril --example clawback` |
| `credentials` | On-ledger credential system | `cargo run -p xrpl-mithril --example credentials` |
| `dex_offers` | DEX order book operations | `cargo run -p xrpl-mithril --example dex_offers` |
| `did_operations` | Decentralized Identifiers | `cargo run -p xrpl-mithril --example did_operations` |
| `multi_sign` | Multi-signature transactions | `cargo run -p xrpl-mithril --example multi_sign` |
| `payment_channels` | Payment channel lifecycle | `cargo run -p xrpl-mithril --example payment_channels` |
| `price_oracle` | Price oracle data | `cargo run -p xrpl-mithril --example price_oracle` |
| `trustline_payment` | Trust lines and issued currencies | `cargo run -p xrpl-mithril --example trustline_payment` |

All examples connect to XRPL testnet and require network access. Fund test wallets via the [testnet faucet](https://faucet.altnet.rippletest.net/).

### WebSocket Subscriptions

```rust
use futures::StreamExt;
use xrpl_mithril::client::{Client, WebSocketClient};
use xrpl_mithril::models::requests::subscription::SubscribeRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WebSocketClient::connect("wss://s.altnet.rippletest.net:51233").await?;
    let mut stream = client.subscribe_stream()?;

    client.request(SubscribeRequest {
        streams: Some(vec!["ledger".to_string()]),
        accounts: None,
        accounts_proposed: None,
        books: None,
    }).await?;

    while let Some(msg) = stream.next().await {
        if msg["type"].as_str() == Some("ledgerClosed") {
            println!("Ledger #{}: {} txns",
                msg["ledger_index"], msg["txn_count"]);
        }
    }
    Ok(())
}
```

## Design Philosophy

- **Invalid states are unrepresentable.** `IssuedValue` stores mantissa and exponent matching the XRPL wire format exactly. Construction validates invariants. Every instance in memory is guaranteed valid.

- **Strong newtypes prevent type confusion.** `AccountId`, `Hash256`, `CurrencyCode`, and `Blob` are all distinct types wrapping byte arrays. Passing a hash where an account ID is expected is a compile error, not a protocol-level bug discovered in production.

- **`#[non_exhaustive]` enums for protocol evolution.** The `Transaction` and `TransactionType` enums use `#[non_exhaustive]`, so new XRPL amendments can add transaction types without breaking downstream code. Unknown types are preserved in a catch-all `Unknown` variant.

- **Zero-cost cryptography abstraction.** The signing layer is generic over the cryptography backend via the `ecdsa` and `signature` trait crates. Backend selection is resolved at compile time with no runtime dispatch.

- **Architecture decisions are documented.** See [research/design-decisions.md](research/design-decisions.md) for 6 ADRs covering decimal representation, transaction enum strategy, newtype strategy, signed vs. unsigned transaction types, field definitions, and cryptography backend selection.

## Protocol Coverage

| Category | Features | Status |
|----------|----------|:------:|
| **Payments** | XRP, issued currency, cross-currency, partial payments | Complete |
| **Tokens** | Trust lines, MPT issuance/authorize/transfer | Complete |
| **Escrow** | XRP escrow, token escrow, MPT escrow (XLS-85) | Complete |
| **DEX** | Offers, AMM create/deposit/withdraw/vote/bid (XLS-30) | Complete |
| **NFTs** | Mint, burn, offers, DynamicNFT, modify (XLS-46) | Complete |
| **Credentials** | Create, accept, delete | Complete |
| **Identity** | DIDs (set/delete) | Complete |
| **Oracles** | Price oracle set/delete (XLS-31) | Complete |
| **Checks** | Create, cash, cancel | Complete |
| **Channels** | Payment channel create/fund/claim | Complete |
| **Clawback** | Token clawback, AMM clawback (XLS-39) | Complete |
| **Cross-chain** | XChain bridge (8 transaction types, XLS-35) | Complete |
| **Account** | AccountSet, AccountDelete, SignerListSet, TicketCreate | Complete |
| **Lending** | XLS-66 (Lending Protocol) | Tracking |

## Minimum Supported Rust Version

xrpl-mithril requires **Rust 1.85.0** or later (the first stable release supporting edition 2024). MSRV is enforced in CI and will not be raised without a minor version bump.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

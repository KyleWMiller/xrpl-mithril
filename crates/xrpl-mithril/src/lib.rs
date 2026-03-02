#![doc(html_logo_url = "https://raw.githubusercontent.com/KyleWMiller/xrpl-mithril/main/assets/mithrilLogo.png")]
//!
//! <div align="center">
//! <img src="https://raw.githubusercontent.com/KyleWMiller/xrpl-mithril/main/assets/mithrilLogo.png" width="200" alt="xrpl-mithril">
//!
//! # xrpl-mithril
//!
//! **A next-generation, pure Rust SDK for the XRP Ledger.**
//! </div>
//!
//! xrpl-mithril targets the 2026 XRPL protocol surface (rippled v3.1.0+),
//! covering 50+ transaction types including Multi-Purpose Tokens, Token Escrow
//! (XLS-85), AMM, Credentials, DynamicNFT, and every mainnet feature through
//! February 2026. The entire codebase enforces `#![forbid(unsafe_code)]`.
//!
//! # Quick Start
//!
//! Send 10 XRP on testnet:
//!
//! ```no_run
//! use xrpl_mithril::client::JsonRpcClient;
//! use xrpl_mithril::tx::builder::PaymentBuilder;
//! use xrpl_mithril::tx::autofill::autofill;
//! use xrpl_mithril::tx::{sign_transaction, submit_and_wait};
//! use xrpl_mithril::types::{Amount, XrpAmount};
//! use xrpl_mithril::wallet::{Algorithm, Wallet};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let sender = Wallet::generate(Algorithm::Ed25519)?;
//!
//! let mut unsigned = PaymentBuilder::new()
//!     .account(*sender.account_id())
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(10_000_000)?))
//!     .build()?;
//!
//! let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
//! autofill(&client, &mut unsigned).await?;
//!
//! let signed = sign_transaction(&unsigned, &sender)?;
//! let result = submit_and_wait(&client, &signed).await?;
//! println!("Validated in ledger {}: {}", result.ledger_index, result.result_code);
//! # Ok(())
//! # }
//! ```
//!
//! Or use the one-liner convenience function:
//!
//! ```no_run
//! use xrpl_mithril::client::JsonRpcClient;
//! use xrpl_mithril::tx::builder::PaymentBuilder;
//! use xrpl_mithril::tx::submit_transaction;
//! use xrpl_mithril::types::{Amount, XrpAmount};
//! use xrpl_mithril::wallet::{Algorithm, Wallet};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
//! let wallet = Wallet::from_seed_encoded("sEdT7wHTCLzDG7Ue4312Kp4QA389Xmb")?;
//!
//! let tx = PaymentBuilder::new()
//!     .account(*wallet.account_id())
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
//!     .build()?;
//!
//! let result = submit_transaction(&client, tx, &wallet).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Installation
//!
//! **Default (pure Rust, no C toolchain required):**
//!
//! ```toml
//! [dependencies]
//! xrpl-mithril = "0.5.1"
//! ```
//!
//! **With native cryptography for maximum secp256k1 performance:**
//!
//! ```toml
//! [dependencies]
//! xrpl-mithril = { version = "0.5.1", features = ["native-crypto"] }
//! ```
//!
//! # Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|:-------:|-------------|
//! | `pure-rust-crypto` | Yes | `k256` + `ed25519-dalek` — builds anywhere, no C compiler |
//! | `native-crypto` | No | `libsecp256k1` via `secp256k1` crate — ~2x faster ECDSA |
//!
//! Both backends expose the identical API. Switching is a `Cargo.toml` change,
//! not a code change.
//!
//! # Wallet Operations
//!
//! ```
//! use xrpl_mithril::wallet::{Algorithm, Wallet};
//! use xrpl_mithril::wallet::address::{encode_x_address, decode_x_address};
//!
//! // Generate a random wallet
//! let wallet = Wallet::generate(Algorithm::Ed25519).unwrap();
//! println!("Address: {}", wallet.account_id().to_classic_address());
//!
//! // Restore from an encoded seed
//! let wallet = Wallet::from_seed_encoded("snoPBrXtMeMyMHUVTgbuqAfg1SUTb").unwrap();
//! assert_eq!(
//!     wallet.account_id().to_classic_address(),
//!     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
//! );
//!
//! // X-address encoding (returns String directly, no Result)
//! let x_addr = encode_x_address(wallet.account_id(), Some(12345), false);
//! let (account, tag, is_test) = decode_x_address(&x_addr).unwrap();
//! assert_eq!(tag, Some(12345));
//! ```
//!
//! # Transaction Builders
//!
//! Fluent builders are provided for common transaction types. Every builder
//! produces an [`models::transactions::wrapper::UnsignedTransaction`]
//! ready for autofill and signing.
//!
//! ```
//! use xrpl_mithril::tx::builder::{
//!     PaymentBuilder, TrustSetBuilder, OfferCreateBuilder, EscrowCreateBuilder,
//! };
//! use xrpl_mithril::types::*;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // XRP payment
//! let payment = PaymentBuilder::new()
//!     .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?)
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(5_000_000)?))
//!     .build()?;
//!
//! // Trust line
//! let issuer: AccountId = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?;
//! let trust_set = TrustSetBuilder::new()
//!     .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?)
//!     .limit_amount(IssuedAmount {
//!         value: IssuedValue::from_decimal_string("1000000")?,
//!         currency: CurrencyCode::from_ascii("USD")?,
//!         issuer,
//!     })
//!     .build()?;
//!
//! // DEX offer
//! let offer = OfferCreateBuilder::new()
//!     .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?)
//!     .taker_pays(Amount::Xrp(XrpAmount::from_drops(50_000_000)?))
//!     .taker_gets(Amount::Issued(IssuedAmount {
//!         value: IssuedValue::from_decimal_string("100")?,
//!         currency: CurrencyCode::from_ascii("USD")?,
//!         issuer,
//!     }))
//!     .build()?;
//!
//! // Time-locked escrow
//! let escrow = EscrowCreateBuilder::new()
//!     .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?)
//!     .destination(issuer)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(10_000_000)?))
//!     .finish_after(820_000_000)
//!     .cancel_after(830_000_000)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! # WebSocket Subscriptions
//!
//! ```no_run
//! use futures::StreamExt;
//! use xrpl_mithril::client::{Client, WebSocketClient};
//! use xrpl_mithril::models::requests::subscription::SubscribeRequest;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = WebSocketClient::connect("wss://s.altnet.rippletest.net:51233").await?;
//! let mut stream = client.subscribe_stream()?;
//!
//! client.request(SubscribeRequest {
//!     streams: Some(vec!["ledger".to_string()]),
//!     accounts: None,
//!     accounts_proposed: None,
//!     books: None,
//! }).await?;
//!
//! while let Some(msg) = stream.next().await {
//!     if msg["type"].as_str() == Some("ledgerClosed") {
//!         println!("Ledger #{}: {} txns",
//!             msg["ledger_index"], msg["txn_count"]);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Binary Codec
//!
//! Serialize transactions to the XRPL binary wire format and back:
//!
//! ```
//! use xrpl_mithril::codec::{serializer, deserializer};
//!
//! let tx = serde_json::json!({
//!     "TransactionType": "Payment",
//!     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!     "Amount": "1000000",
//!     "Fee": "12",
//!     "Sequence": 1
//! });
//!
//! let map = tx.as_object().unwrap();
//! let mut bytes = Vec::new();
//! serializer::serialize_json_object(map, &mut bytes, false).unwrap();
//! let decoded = deserializer::deserialize_object(&bytes).unwrap();
//! assert_eq!(decoded["TransactionType"], "Payment");
//! ```
//!
//! # Multi-Signature Transactions
//!
//! ```
//! use xrpl_mithril::wallet::{Wallet, Algorithm};
//! use xrpl_mithril::wallet::signer::{multi_sign, combine_signatures};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let signer1 = Wallet::generate(Algorithm::Ed25519)?;
//! let signer2 = Wallet::generate(Algorithm::Secp256k1)?;
//!
//! let tx_json: serde_json::Map<String, serde_json::Value> =
//!     serde_json::from_value(serde_json::json!({
//!         "TransactionType": "Payment",
//!         "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!         "Amount": "1000000",
//!         "Fee": "12",
//!         "Sequence": 1
//!     }))?;
//!
//! let sig1 = multi_sign(&tx_json, &signer1)?;
//! let sig2 = multi_sign(&tx_json, &signer2)?;
//! let combined = combine_signatures(&tx_json, vec![sig1, sig2])?;
//! assert!(combined.tx_json.contains_key("Signers"));
//! # Ok(())
//! # }
//! ```
//!
//! # Signing and Verification
//!
//! ```
//! use xrpl_mithril::wallet::{Wallet, Algorithm};
//! use xrpl_mithril::wallet::signer::sign;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let wallet = Wallet::from_seed_encoded("snoPBrXtMeMyMHUVTgbuqAfg1SUTb")?;
//!
//! let tx_json: serde_json::Map<String, serde_json::Value> =
//!     serde_json::from_value(serde_json::json!({
//!         "TransactionType": "Payment",
//!         "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!         "Amount": "1000000",
//!         "Fee": "12",
//!         "Sequence": 1
//!     }))?;
//!
//! let signed = sign(&tx_json, &wallet)?;
//! println!("Hash: {}", signed.hash);
//! println!("Blob: {}", signed.tx_blob);
//! assert!(signed.tx_json.contains_key("TxnSignature"));
//! # Ok(())
//! # }
//! ```
//!
//! # Working with Types
//!
//! Core protocol types enforce validity at construction time:
//!
//! ```
//! use xrpl_mithril::types::*;
//!
//! // Amounts
//! let xrp = XrpAmount::from_drops(1_000_000).unwrap();
//! assert_eq!(xrp, XrpAmount::ONE_XRP);
//!
//! let issued = IssuedValue::from_decimal_string("99.5").unwrap();
//! assert_eq!(issued.to_decimal_string(), "99.5");
//!
//! // Account addresses
//! let account: AccountId = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().unwrap();
//! assert_eq!(account.to_classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
//!
//! // Currency codes
//! let usd = CurrencyCode::from_ascii("USD").unwrap();
//! assert_eq!(&usd.as_ascii().unwrap(), b"USD");
//!
//! // Hashes
//! let hash = Hash256::from_hex(
//!     "4C1A1B1E1F1D1C1B1A191817161514131211100F0E0D0C0B0A09080706050403"
//! ).unwrap();
//! assert_eq!(hash.as_bytes().len(), 32);
//! ```
//!
//! # Crate Organization
//!
//! | Crate | Re-export | Purpose |
//! |-------|-----------|---------|
//! | `xrpl-mithril-types` | [`types`] | Core protocol types (amounts, accounts, hashes, currencies) |
//! | `xrpl-mithril-codec` | [`codec`] | Binary serialization/deserialization (XRPL wire format) |
//! | `xrpl-mithril-models` | [`models`] | 50+ transaction types, 17 ledger entry types, request/response types |
//! | `xrpl-mithril-wallet` | [`wallet`] | Key generation, signing, seed/address encoding |
//! | `xrpl-mithril-client` | [`client`] | JSON-RPC and WebSocket clients (rustls TLS, no OpenSSL) |
//! | `xrpl-mithril-tx` | [`tx`] | Transaction builders, autofill, reliable submission |
//!
//! Depend on `xrpl-mithril` to get everything, or pick individual crates
//! for a smaller dependency footprint. All crates enforce
//! `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

pub use xrpl_mithril_types as types;
pub use xrpl_mithril_codec as codec;
pub use xrpl_mithril_models as models;
pub use xrpl_mithril_wallet as wallet;
pub use xrpl_mithril_client as client;
pub use xrpl_mithril_tx as tx;

//! Integration tests for the full transaction submission pipeline.
//!
//! These tests require network access and a funded testnet account.
//! Run with: `cargo test -- --ignored`

use xrpl_client::{Client, JsonRpcClient};
use xrpl_models::requests::server::ServerInfoRequest;
use xrpl_tx::autofill::autofill;
use xrpl_tx::builder::PaymentBuilder;
use xrpl_tx::reliable::sign_transaction;
use xrpl_types::{Amount, XrpAmount};
use xrpl_wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";

#[tokio::test]
#[ignore]
async fn autofill_populates_fields() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");

    // Use genesis account (always exists on testnet)
    let mut unsigned = PaymentBuilder::new()
        .account(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
                .parse()
                .expect("addr"),
        )
        .destination(
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
                .parse()
                .expect("addr"),
        )
        .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("drops")))
        .build()
        .expect("build");

    // Before autofill: sequence is 0, fee is 0, last_ledger is None
    assert_eq!(unsigned.common().sequence, 0);
    assert!(unsigned.common().last_ledger_sequence.is_none());

    autofill(&client, &mut unsigned).await.expect("autofill");

    // After autofill: all fields should be populated
    assert!(unsigned.common().sequence > 0);
    assert!(unsigned.common().last_ledger_sequence.is_some());
    // Fee should be non-zero
    if let Amount::Xrp(ref xrp) = unsigned.common().fee {
        assert!(xrp.drops() > 0);
    }
}

#[tokio::test]
#[ignore]
async fn sign_transaction_produces_valid_blob() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");

    let wallet = Wallet::generate(Algorithm::Ed25519).expect("wallet");

    let mut unsigned = PaymentBuilder::new()
        .account(wallet.account_id().clone())
        .destination(
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
                .parse()
                .expect("addr"),
        )
        .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("drops")))
        .build()
        .expect("build");

    // Manually set fields (skip autofill since wallet isn't funded)
    unsigned.common_mut().sequence = 1;
    unsigned.common_mut().fee = Amount::Xrp(XrpAmount::from_drops(12).expect("drops"));
    unsigned.common_mut().last_ledger_sequence = Some(100);

    let signed = sign_transaction(&unsigned, &wallet).expect("sign");

    // Verify signed transaction properties
    assert!(!signed.tx_blob().is_empty());
    assert_eq!(signed.hash().len(), 64);
    assert!(signed.hash().chars().all(|c| c.is_ascii_hexdigit()));
    assert!(signed.tx_json().contains_key("TxnSignature"));
    assert!(signed.tx_json().contains_key("SigningPubKey"));
}

#[tokio::test]
#[ignore]
async fn server_info_returns_validated_ledger() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client
        .request(ServerInfoRequest {})
        .await
        .expect("server_info");
    let ledger = resp.info.validated_ledger.expect("should have validated ledger");
    assert!(ledger.seq > 0);
}

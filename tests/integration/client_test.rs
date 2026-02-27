//! Integration tests for the JSON-RPC and WebSocket clients.
//!
//! These tests require network access to the XRPL Testnet.
//! Run with: `cargo test -- --ignored`

use xrpl_client::{Client, JsonRpcClient, WebSocketClient};
use xrpl_models::requests::{
    account::AccountInfoRequest,
    ledger::{LedgerClosedRequest, LedgerCurrentRequest, LedgerRequest},
    server::{FeeRequest, PingRequest, ServerInfoRequest},
    LedgerShortcut, LedgerSpecifier,
};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const TESTNET_WS: &str = "wss://s.altnet.rippletest.net:51233";

// A well-known testnet genesis account
const GENESIS_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

// -----------------------------------------------------------------------
// JSON-RPC tests
// -----------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn jsonrpc_ping() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let _resp = client.request(PingRequest {}).await.expect("ping");
}

#[tokio::test]
#[ignore]
async fn jsonrpc_server_info() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client
        .request(ServerInfoRequest {})
        .await
        .expect("server_info");
    assert!(resp.info.build_version.is_some());
    assert!(resp.info.validated_ledger.is_some());
}

#[tokio::test]
#[ignore]
async fn jsonrpc_fee() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client.request(FeeRequest {}).await.expect("fee");
    // Fee drops should parse to a number
    let _: u64 = resp
        .drops
        .open_ledger_fee
        .parse()
        .expect("valid fee number");
}

#[tokio::test]
#[ignore]
async fn jsonrpc_account_info() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client
        .request(AccountInfoRequest {
            account: GENESIS_ACCOUNT.parse().expect("valid address"),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await
        .expect("account_info");
    assert!(resp.account_data.sequence > 0);
}

#[tokio::test]
#[ignore]
async fn jsonrpc_ledger_current() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client
        .request(LedgerCurrentRequest {})
        .await
        .expect("ledger_current");
    assert!(resp.ledger_current_index > 0);
}

#[tokio::test]
#[ignore]
async fn jsonrpc_ledger_closed() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client
        .request(LedgerClosedRequest {})
        .await
        .expect("ledger_closed");
    assert!(resp.ledger_index > 0);
}

#[tokio::test]
#[ignore]
async fn jsonrpc_ledger() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    let resp = client
        .request(LedgerRequest {
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            ..LedgerRequest::default()
        })
        .await
        .expect("ledger");
    assert!(resp.ledger.ledger_index.is_some());
}

#[tokio::test]
#[ignore]
async fn jsonrpc_error_handling() {
    let client = JsonRpcClient::new(TESTNET_RPC).expect("valid url");
    // Query a non-existent account
    let result = client
        .request(AccountInfoRequest {
            account: "rXXXXXXXXXXXXXXXXXXXXXXXXXXXhq"
                .parse()
                .expect("valid address format"),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await;
    assert!(result.is_err());
}

// -----------------------------------------------------------------------
// WebSocket tests
// -----------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn websocket_server_info() {
    let client = WebSocketClient::connect(TESTNET_WS)
        .await
        .expect("ws connect");
    assert!(client.is_connected());

    let resp = client
        .request(ServerInfoRequest {})
        .await
        .expect("server_info");
    assert!(resp.info.build_version.is_some());
}

#[tokio::test]
#[ignore]
async fn websocket_fee() {
    let client = WebSocketClient::connect(TESTNET_WS)
        .await
        .expect("ws connect");
    let resp = client.request(FeeRequest {}).await.expect("fee");
    let _: u64 = resp
        .drops
        .open_ledger_fee
        .parse()
        .expect("valid fee number");
}

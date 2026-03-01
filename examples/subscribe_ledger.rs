//! Example: WebSocket Ledger Subscription
//!
//! Connects to the XRPL testnet via WebSocket, subscribes to the ledger stream,
//! and prints each new validated ledger as it closes (~3-5 seconds apart).
//!
//! Run: `cargo run -p xrpl-mithril --example subscribe_ledger`
//! Requires: Network access to XRPL testnet

use futures::StreamExt;

use xrpl_mithril::xrpl_client::{Client, WebSocketClient};
use xrpl_mithril::xrpl_models::requests::subscription::SubscribeRequest;

const TESTNET_WS: &str = "wss://s.altnet.rippletest.net:51233";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Connect via WebSocket ---
    println!("Connecting to XRPL testnet via WebSocket at {TESTNET_WS}...");
    let client = WebSocketClient::connect(TESTNET_WS).await?;
    println!("  Connected: {}", client.is_connected());

    // --- 2. Set up subscription stream ---
    // The subscription stream must be created BEFORE sending the subscribe request,
    // so we don't miss any messages.
    let mut stream = client.subscribe_stream()?;

    // --- 3. Subscribe to the ledger stream ---
    println!("\nSubscribing to ledger stream...");
    let _sub_response = client
        .request(SubscribeRequest {
            streams: Some(vec!["ledger".to_string()]),
            accounts: None,
            accounts_proposed: None,
            books: None,
        })
        .await?;
    println!("  Subscribed. Waiting for ledger closes...\n");

    // --- 4. Listen for ledger close events ---
    // Ledgers close every ~3-5 seconds on testnet.
    // We'll listen for 5 ledger closes then exit.
    let mut count = 0u32;
    let max_ledgers = 5u32;

    while let Some(msg) = stream.next().await {
        // Subscription messages are raw serde_json::Value.
        // Check if this is a ledgerClosed event.
        let msg_type = msg["type"].as_str().unwrap_or("");
        if msg_type != "ledgerClosed" {
            continue;
        }

        count += 1;

        let ledger_index = msg["ledger_index"].as_u64().unwrap_or(0);
        let ledger_hash = msg["ledger_hash"].as_str().unwrap_or("unknown");
        let txn_count = msg["txn_count"].as_u64().unwrap_or(0);
        let close_time = msg["ledger_time"].as_u64().unwrap_or(0);

        println!("Ledger #{count}/{max_ledgers}:");
        println!("  Index:      {ledger_index}");
        println!("  Hash:       {ledger_hash}");
        println!("  Txn count:  {txn_count}");
        println!("  Close time: {close_time} (Ripple epoch)");
        println!();

        if count >= max_ledgers {
            println!("Received {max_ledgers} ledgers. Disconnecting.");
            break;
        }
    }

    println!("Done.");
    Ok(())
}

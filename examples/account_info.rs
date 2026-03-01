//! Example: Account Info Query
//!
//! Demonstrates read-only queries against the XRPL testnet using the JSON-RPC client.
//! No wallet or signing required — just connect and query.
//!
//! Run: `cargo run -p xrpl-mithril --example account_info`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::{AccountInfoRequest, AccountLinesRequest};
use xrpl_mithril::xrpl_models::requests::server::ServerInfoRequest;
use xrpl_mithril::xrpl_models::requests::LedgerSpecifier;
use xrpl_mithril::xrpl_types::AccountId;

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";

// Genesis account — always exists on testnet
const GENESIS_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Connect to testnet ---
    println!("Connecting to XRPL testnet at {TESTNET_RPC}...");
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 2. Query server info ---
    println!("\n--- Server Info ---");
    let server_info = client.request(ServerInfoRequest {}).await?;
    let info = &server_info.info;
    println!("  Build version:     {:?}", info.build_version);
    println!("  Server state:      {:?}", info.server_state);
    println!("  Complete ledgers:  {:?}", info.complete_ledgers);
    if let Some(ref validated) = info.validated_ledger {
        println!("  Validated ledger:  seq={}", validated.seq);
        println!("  Base fee (XRP):    {:?}", validated.base_fee_xrp);
        println!("  Reserve base:      {:?}", validated.reserve_base_xrp);
    }

    // --- 3. Query account info ---
    let account: AccountId = GENESIS_ACCOUNT.parse()?;

    println!("\n--- Account Info for {GENESIS_ACCOUNT} ---");
    let account_info = client
        .request(AccountInfoRequest {
            account,
            ledger_index: Some(LedgerSpecifier::Named(
                xrpl_mithril::xrpl_models::requests::LedgerShortcut::Validated,
            )),
            queue: None,
            signer_lists: None,
        })
        .await?;

    let data = &account_info.account_data;
    println!("  Address:       {}", data.account.to_classic_address());
    println!("  Balance:       {} drops", data.balance);
    println!("  Sequence:      {}", data.sequence);
    println!("  Flags:         0x{:08X}", data.flags);
    println!("  Owner count:   {}", data.owner_count);
    println!("  Validated:     {:?}", account_info.validated);

    // Convert drops to XRP for display
    let drops: u64 = data.balance.parse()?;
    let xrp = drops as f64 / 1_000_000.0;
    println!("  Balance (XRP): {xrp:.6}");

    // --- 4. Query trust lines ---
    println!("\n--- Trust Lines ---");
    let lines_response = client
        .request(AccountLinesRequest {
            account,
            ledger_index: None,
            peer: None,
            limit: Some(10),
            marker: None,
        })
        .await?;

    if lines_response.lines.is_empty() {
        println!("  (no trust lines found)");
    } else {
        for line in &lines_response.lines {
            println!(
                "  {} {} (peer: {}, limit: {})",
                line.balance,
                line.currency,
                line.account.to_classic_address(),
                line.limit,
            );
        }
    }

    println!("\nDone.");
    Ok(())
}

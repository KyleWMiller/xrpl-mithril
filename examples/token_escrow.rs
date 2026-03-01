//! Example: XRP Escrow (Time-Based)
//!
//! Demonstrates the escrow lifecycle: create a time-locked escrow, wait for the
//! release time, then finish the escrow to deliver the funds.
//!
//! Run: `cargo run -p xrpl-mithril --example token_escrow`
//! Requires: Network access to XRPL testnet + ~30 seconds for escrow timing

use std::time::{SystemTime, UNIX_EPOCH};

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::AccountInfoRequest;
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::escrow::{EscrowCreate, EscrowFinish};
use xrpl_mithril::xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::xrpl_models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::xrpl_tx::autofill::autofill;
use xrpl_mithril::xrpl_tx::submit::submit_and_wait;
use xrpl_mithril::xrpl_tx::{sign_transaction, TransactionResult};
use xrpl_mithril::xrpl_types::{AccountId, Amount, XrpAmount};
use xrpl_mithril::xrpl_wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";

/// Ripple epoch offset: seconds between Unix epoch and Ripple epoch (2000-01-01T00:00:00Z).
const RIPPLE_EPOCH_OFFSET: u64 = 946_684_800;

/// Get the current time as a Ripple epoch timestamp.
///
/// # Errors
///
/// Returns an error if the system clock is before the Unix epoch.
fn ripple_now() -> Result<u32, Box<dyn std::error::Error>> {
    let unix_now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok((unix_now - RIPPLE_EPOCH_OFFSET) as u32)
}

/// Fund a wallet using the XRPL testnet faucet.
async fn fund_wallet(
    http: &reqwest::Client,
    wallet: &Wallet,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "destination": wallet.classic_address(),
    });
    http.post(FAUCET_URL)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    println!("  Funded: {}", wallet.classic_address());
    Ok(())
}

/// Build a TransactionCommon with placeholder values (autofill will set fee/seq/LLS).
fn make_common(account: AccountId) -> TransactionCommon {
    TransactionCommon {
        account,
        fee: Amount::Xrp(XrpAmount::ZERO),
        sequence: 0,
        flags: None,
        last_ledger_sequence: None,
        account_txn_id: None,
        memos: None,
        network_id: None,
        source_tag: None,
        signing_pub_key: None,
        txn_signature: None,
        ticket_sequence: None,
        signers: None,
    }
}

/// Query and display an account's XRP balance.
async fn get_balance(
    client: &JsonRpcClient,
    account: AccountId,
) -> Result<u64, Box<dyn std::error::Error>> {
    let info = client
        .request(AccountInfoRequest {
            account,
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;
    let drops: u64 = info.account_data.balance.parse()?;
    Ok(drops)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let sender = Wallet::generate(Algorithm::Ed25519)?;
    let receiver = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Sender:   {}", sender.classic_address());
    println!("  Receiver: {}", receiver.classic_address());

    // --- 2. Fund both wallets via testnet faucet ---
    // Both must exist on-ledger: sender to create the escrow, receiver as destination.
    // Receiver also needs funds later to pay the EscrowFinish fee.
    println!("\nFunding wallets...");
    let http = reqwest::Client::new();
    fund_wallet(&http, &sender).await?;
    fund_wallet(&http, &receiver).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 3. Create time-locked escrow ---
    // Set finish_after to 10 seconds from now, cancel_after to 60 seconds.
    let finish_after = ripple_now()? + 10;
    let cancel_after = ripple_now()? + 60;
    let escrow_drops = 5_000_000u64; // 5 XRP

    println!("\nCreating escrow:");
    println!("  Amount:       {escrow_drops} drops (5 XRP)");
    println!("  Finish after: {finish_after} (Ripple epoch, ~10s from now)");
    println!("  Cancel after: {cancel_after} (Ripple epoch, ~60s from now)");

    let escrow_create = Transaction::EscrowCreate {
        common: make_common(*sender.account_id()),
        fields: EscrowCreate {
            destination: *receiver.account_id(),
            amount: Amount::Xrp(XrpAmount::from_drops(escrow_drops)?),
            finish_after: Some(finish_after),
            cancel_after: Some(cancel_after),
            condition: None,
            destination_tag: None,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(escrow_create);
    autofill(&client, &mut unsigned_create).await?;

    // Remember the sequence number — we need it to finish the escrow
    let create_sequence = unsigned_create.common().sequence;
    println!("  Escrow create sequence: {create_sequence}");

    let signed_create = sign_transaction(&unsigned_create, &sender)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  Escrow created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 4. Wait for finish_after to pass ---
    let wait_secs = 12; // 10s + 2s buffer
    println!("\nWaiting {wait_secs} seconds for finish_after to pass...");
    tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

    // --- 5. Finish the escrow ---
    println!("Finishing escrow...");
    let escrow_finish = Transaction::EscrowFinish {
        common: make_common(*receiver.account_id()),
        fields: EscrowFinish {
            owner: *sender.account_id(),
            offer_sequence: create_sequence,
            condition: None,
            fulfillment: None,
        },
    };

    let mut unsigned_finish = UnsignedTransaction::new(escrow_finish);
    autofill(&client, &mut unsigned_finish).await?;

    let signed_finish = sign_transaction(&unsigned_finish, &receiver)?;
    let finish_result: TransactionResult = submit_and_wait(&client, &signed_finish).await?;

    println!("\n  Escrow finished!");
    println!("  Hash:   {}", finish_result.hash);
    println!("  Result: {}", finish_result.result_code);
    println!("  Ledger: {}", finish_result.ledger_index);

    // --- 6. Verify balances ---
    let receiver_balance = get_balance(&client, *receiver.account_id()).await?;
    let receiver_xrp = receiver_balance as f64 / 1_000_000.0;
    println!("\n  Receiver balance: {receiver_xrp:.6} XRP");
    println!("  (includes faucet funding + 5 XRP from escrow, minus fees)");

    println!("\nDone.");
    Ok(())
}

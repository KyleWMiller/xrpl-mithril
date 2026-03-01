//! Example: Basic XRP Payment
//!
//! Demonstrates the full transaction lifecycle: fund wallets via the testnet faucet,
//! build a Payment transaction, autofill, sign, submit, and wait for validation.
//!
//! Run: `cargo run -p xrpl-mithril --example basic_payment`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountInfoRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::tx::builder::PaymentBuilder;
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::{sign_transaction, submit_and_wait, TransactionResult};
use xrpl_mithril::types::{AccountId, Amount, XrpAmount};
use xrpl_mithril::wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";

/// Fund a wallet using the XRPL testnet faucet.
///
/// Returns the classic address and the funded wallet.
async fn fund_wallet(
    http: &reqwest::Client,
    wallet: &Wallet,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "destination": wallet.classic_address(),
    });

    let resp = http
        .post(FAUCET_URL)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    Ok(resp)
}

/// Query and display an account's XRP balance.
async fn print_balance(
    client: &JsonRpcClient,
    label: &str,
    account: AccountId,
) -> Result<(), Box<dyn std::error::Error>> {
    let info = client
        .request(AccountInfoRequest {
            account,
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;

    let drops: u64 = info.account_data.balance.parse()?;
    let xrp = drops as f64 / 1_000_000.0;
    println!("  {label}: {xrp:.6} XRP ({drops} drops)");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let sender = Wallet::generate(Algorithm::Ed25519)?;
    let receiver = Wallet::generate(Algorithm::Secp256k1)?;

    println!("  Sender:   {}", sender.classic_address());
    println!("  Receiver: {}", receiver.classic_address());

    // --- 2. Fund sender via testnet faucet ---
    println!("\nFunding sender via testnet faucet...");
    let http = reqwest::Client::new();
    let faucet_resp = fund_wallet(&http, &sender).await?;
    println!(
        "  Faucet response: {}",
        serde_json::to_string_pretty(&faucet_resp)?
    );

    // Wait for the faucet funding to be validated
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Connect and check balances ---
    let client = JsonRpcClient::new(TESTNET_RPC)?;
    println!("\nBalances before payment:");
    print_balance(&client, "Sender", *sender.account_id()).await?;

    // --- 4. Build a Payment transaction ---
    let payment_amount = XrpAmount::from_drops(10_000_000)?; // 10 XRP
    println!("\nBuilding Payment: 10 XRP from sender to receiver...");

    let unsigned = PaymentBuilder::new()
        .account(*sender.account_id())
        .destination(*receiver.account_id())
        .amount(Amount::Xrp(payment_amount))
        .destination_tag(42)
        .build()?;

    println!(
        "  Transaction type: {}",
        unsigned.inner().transaction_type()
    );

    // --- 5. Autofill ---
    println!("\nAutofilling transaction (fee, sequence, last_ledger_sequence)...");
    let mut unsigned = unsigned;
    autofill(&client, &mut unsigned).await?;

    println!("  Fee:                {:?}", unsigned.common().fee);
    println!("  Sequence:           {}", unsigned.common().sequence);
    println!(
        "  LastLedgerSequence: {:?}",
        unsigned.common().last_ledger_sequence
    );

    // --- 6. Sign ---
    println!("\nSigning transaction...");
    let signed = sign_transaction(&unsigned, &sender)?;
    println!("  Hash:    {}", signed.hash());
    println!("  Blob:    {} bytes", signed.tx_blob().len() / 2);
    // Print first 80 chars of blob for debugging
    let blob_preview = &signed.tx_blob()[..std::cmp::min(80, signed.tx_blob().len())];
    println!("  Blob[0..40]: {blob_preview}...");

    // --- 7. Submit and wait ---
    println!("\nSubmitting and waiting for validation...");
    let result: TransactionResult = submit_and_wait(&client, &signed).await?;

    println!("\nTransaction validated!");
    println!("  Hash:        {}", result.hash);
    println!("  Result:      {}", result.result_code);
    println!("  Ledger:      {}", result.ledger_index);

    // --- 8. Check final balances ---
    println!("\nBalances after payment:");
    print_balance(&client, "Sender", *sender.account_id()).await?;
    print_balance(&client, "Receiver", *receiver.account_id()).await?;

    println!("\nDone.");
    Ok(())
}

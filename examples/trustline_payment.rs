//! Example: Trust Line + Issued Currency Payment
//!
//! Demonstrates the classic XRPL token flow: set up a trust line for an issued
//! currency, then transfer tokens from issuer to holder.
//!
//! Run: `cargo run -p xrpl-mithril --example trustline_payment`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountLinesRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::models::transactions::payment::Payment;
use xrpl_mithril::models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::builder::TrustSetBuilder;
use xrpl_mithril::tx::{sign_transaction, submit_and_wait, TransactionResult};
use xrpl_mithril::types::amount::{IssuedAmount, IssuedValue};
use xrpl_mithril::types::currency::CurrencyCode;
use xrpl_mithril::types::{AccountId, Amount, XrpAmount};
use xrpl_mithril::wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let issuer = Wallet::generate(Algorithm::Ed25519)?;
    let holder = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Issuer: {}", issuer.classic_address());
    println!("  Holder: {}", holder.classic_address());

    // --- 2. Fund both wallets via testnet faucet ---
    println!("\nFunding wallets...");
    let http = reqwest::Client::new();
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &holder).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 3. Holder creates a TrustSet to the issuer for "TST" currency ---
    println!("\nHolder creating trust line for TST (limit 1000)...");

    let trust_limit = IssuedAmount {
        value: IssuedValue::from_decimal_string("1000")?,
        currency: CurrencyCode::from_ascii("TST")?,
        issuer: *issuer.account_id(),
    };

    let mut unsigned_trust = TrustSetBuilder::new()
        .account(*holder.account_id())
        .limit_amount(trust_limit)
        .build()?;

    autofill(&client, &mut unsigned_trust).await?;

    println!("  Fee:      {:?}", unsigned_trust.common().fee);
    println!("  Sequence: {}", unsigned_trust.common().sequence);

    let signed_trust = sign_transaction(&unsigned_trust, &holder)?;
    let trust_result: TransactionResult = submit_and_wait(&client, &signed_trust).await?;

    println!("\n  TrustSet validated!");
    println!("  Hash:   {}", trust_result.hash);
    println!("  Result: {}", trust_result.result_code);
    println!("  Ledger: {}", trust_result.ledger_index);

    // --- 4. Issuer sends 100 TST to holder ---
    println!("\nIssuer sending 100 TST to holder...");

    let payment = Transaction::Payment {
        common: make_common(*issuer.account_id()),
        fields: Payment {
            destination: *holder.account_id(),
            amount: Amount::Issued(IssuedAmount {
                value: IssuedValue::from_decimal_string("100")?,
                currency: CurrencyCode::from_ascii("TST")?,
                issuer: *issuer.account_id(),
            }),
            send_max: None,
            deliver_min: None,
            destination_tag: None,
            invoice_id: None,
            paths: None,
        },
    };

    let mut unsigned_payment = UnsignedTransaction::new(payment);
    autofill(&client, &mut unsigned_payment).await?;

    println!("  Fee:      {:?}", unsigned_payment.common().fee);
    println!("  Sequence: {}", unsigned_payment.common().sequence);

    let signed_payment = sign_transaction(&unsigned_payment, &issuer)?;
    let payment_result: TransactionResult = submit_and_wait(&client, &signed_payment).await?;

    println!("\n  Payment validated!");
    println!("  Hash:   {}", payment_result.hash);
    println!("  Result: {}", payment_result.result_code);
    println!("  Ledger: {}", payment_result.ledger_index);

    // --- 5. Query trust line balance ---
    println!("\nQuerying holder's trust lines...");

    let lines_response = client
        .request(AccountLinesRequest {
            account: *holder.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            peer: Some(*issuer.account_id()),
            limit: None,
            marker: None,
        })
        .await?;

    for line in &lines_response.lines {
        println!("  Currency: {}", line.currency);
        println!("  Balance:  {}", line.balance);
        println!("  Limit:    {}", line.limit);
    }

    println!("\nDone.");
    Ok(())
}

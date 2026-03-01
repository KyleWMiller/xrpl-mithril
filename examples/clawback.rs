//! Example: Token Clawback (XLS-39)
//!
//! Demonstrates how a token issuer can claw back (reclaim) issued tokens from
//! a holder. The issuer must enable clawback BEFORE creating any trust lines.
//!
//! Run: `cargo run -p xrpl-mithril --example clawback`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::AccountLinesRequest;
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::account::AccountSet;
use xrpl_mithril::xrpl_models::transactions::clawback::Clawback;
use xrpl_mithril::xrpl_models::transactions::payment::Payment;
use xrpl_mithril::xrpl_models::transactions::trust_set::TrustSet;
use xrpl_mithril::xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::xrpl_models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::xrpl_tx::autofill::autofill;
use xrpl_mithril::xrpl_tx::submit::submit_and_wait;
use xrpl_mithril::xrpl_tx::{sign_transaction, TransactionResult};
use xrpl_mithril::xrpl_types::amount::{IssuedAmount, IssuedValue};
use xrpl_mithril::xrpl_types::currency::CurrencyCode;
use xrpl_mithril::xrpl_types::{AccountId, Amount, XrpAmount};
use xrpl_mithril::xrpl_wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";

/// AccountSet flag: allow trust line clawback. Must be set before any trust lines exist.
const ASF_ALLOW_TRUSTLINE_CLAWBACK: u32 = 16;

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

/// Query the holder's trust line balance for a given currency and issuer.
async fn get_trust_line_balance(
    client: &JsonRpcClient,
    holder: AccountId,
    issuer: AccountId,
    currency: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let lines_resp = client
        .request(AccountLinesRequest {
            account: holder,
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            peer: Some(issuer),
            limit: None,
            marker: None,
        })
        .await?;

    for line in &lines_resp.lines {
        if line.currency == currency {
            return Ok(line.balance.clone());
        }
    }

    Ok("0".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::new();
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let issuer = Wallet::generate(Algorithm::Ed25519)?;
    let holder = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Issuer: {}", issuer.classic_address());
    println!("  Holder: {}", holder.classic_address());

    // --- 2. Fund both wallets ---
    println!("\nFunding wallets...");
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &holder).await?;
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Enable clawback on the issuer account ---
    // IMPORTANT: This must be done BEFORE any trust lines are created.
    // Once an account has trust lines, enabling clawback is no longer possible.
    println!("\nEnabling clawback on issuer (asfAllowTrustLineClawback)...");
    let account_set_tx = Transaction::AccountSet {
        common: make_common(*issuer.account_id()),
        fields: AccountSet {
            set_flag: Some(ASF_ALLOW_TRUSTLINE_CLAWBACK),
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        },
    };

    let mut unsigned_set = UnsignedTransaction::new(account_set_tx);
    autofill(&client, &mut unsigned_set).await?;
    let signed_set = sign_transaction(&unsigned_set, &issuer)?;
    let set_result: TransactionResult = submit_and_wait(&client, &signed_set).await?;

    println!("  Clawback enabled!");
    println!("  Hash:   {}", set_result.hash);
    println!("  Result: {}", set_result.result_code);

    // --- 4. Holder creates trust line to issuer for "TST" ---
    println!("\nHolder creating trust line for TST (limit: 1000)...");
    let trust_set_tx = Transaction::TrustSet {
        common: make_common(*holder.account_id()),
        fields: TrustSet {
            limit_amount: IssuedAmount {
                value: IssuedValue::from_decimal_string("1000")?,
                currency: CurrencyCode::from_ascii("TST")?,
                issuer: *issuer.account_id(),
            },
            quality_in: None,
            quality_out: None,
        },
    };

    let mut unsigned_trust = UnsignedTransaction::new(trust_set_tx);
    autofill(&client, &mut unsigned_trust).await?;
    let signed_trust = sign_transaction(&unsigned_trust, &holder)?;
    let trust_result: TransactionResult = submit_and_wait(&client, &signed_trust).await?;

    println!("  Trust line created!");
    println!("  Hash:   {}", trust_result.hash);
    println!("  Result: {}", trust_result.result_code);

    // --- 5. Issuer sends 100 TST to holder ---
    println!("\nIssuer sending 100 TST to holder...");
    let payment_tx = Transaction::Payment {
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

    let mut unsigned_payment = UnsignedTransaction::new(payment_tx);
    autofill(&client, &mut unsigned_payment).await?;
    let signed_payment = sign_transaction(&unsigned_payment, &issuer)?;
    let payment_result: TransactionResult = submit_and_wait(&client, &signed_payment).await?;

    println!("  Payment sent!");
    println!("  Hash:   {}", payment_result.hash);
    println!("  Result: {}", payment_result.result_code);

    // --- 6. Query holder's trust line balance (should be 100) ---
    let balance_before = get_trust_line_balance(
        &client,
        *holder.account_id(),
        *issuer.account_id(),
        "TST",
    )
    .await?;
    println!("\nHolder TST balance before clawback: {balance_before}");

    // --- 7. Issuer claws back 50 TST from holder ---
    // CRITICAL: In the Clawback amount, the `issuer` field is the HOLDER's address
    // (the account to claw back from), NOT the token issuer. The transaction sender
    // (common.account) is the actual token issuer.
    println!("\nIssuer clawing back 50 TST from holder...");
    let clawback_tx = Transaction::Clawback {
        common: make_common(*issuer.account_id()),
        fields: Clawback {
            amount: Amount::Issued(IssuedAmount {
                value: IssuedValue::from_decimal_string("50")?,
                currency: CurrencyCode::from_ascii("TST")?,
                issuer: *holder.account_id(),
            }),
        },
    };

    let mut unsigned_clawback = UnsignedTransaction::new(clawback_tx);
    autofill(&client, &mut unsigned_clawback).await?;
    let signed_clawback = sign_transaction(&unsigned_clawback, &issuer)?;
    let clawback_result: TransactionResult = submit_and_wait(&client, &signed_clawback).await?;

    println!("  Clawback executed!");
    println!("  Hash:   {}", clawback_result.hash);
    println!("  Result: {}", clawback_result.result_code);

    // --- 8. Query holder's trust line balance again (should be 50) ---
    let balance_after = get_trust_line_balance(
        &client,
        *holder.account_id(),
        *issuer.account_id(),
        "TST",
    )
    .await?;
    println!("\nHolder TST balance after clawback: {balance_after}");

    println!("\nDone.");
    Ok(())
}

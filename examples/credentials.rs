//! Example: Credentials Lifecycle (XLS-70)
//!
//! Demonstrates the on-ledger credential system: an issuer creates a KYC
//! credential for a subject, the subject accepts it, and then deletes it.
//!
//! Run: `cargo run -p xrpl-mithril --example credentials`
//! Requires: Network access to XRPL testnet with Credentials amendment active

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::AccountObjectsRequest;
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::credential::{
    CredentialAccept, CredentialCreate, CredentialDelete,
};
use xrpl_mithril::xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::xrpl_models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::xrpl_tx::autofill::autofill;
use xrpl_mithril::xrpl_tx::submit::submit_and_wait;
use xrpl_mithril::xrpl_tx::{sign_transaction, TransactionResult};
use xrpl_mithril::xrpl_types::{AccountId, Amount, Blob, XrpAmount};
use xrpl_mithril::xrpl_wallet::{Algorithm, Wallet};

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

/// Query credentials owned by an account via account_objects and print them.
async fn query_credentials(
    client: &JsonRpcClient,
    label: &str,
    account: AccountId,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let resp = client
        .request(AccountObjectsRequest {
            account,
            object_type: Some("credential".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    println!("  {label}: {} credential object(s)", resp.account_objects.len());
    for (i, obj) in resp.account_objects.iter().enumerate() {
        println!(
            "    [{i}] {}",
            serde_json::to_string_pretty(obj)?
        );
    }

    Ok(resp.account_objects)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::new();
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let issuer = Wallet::generate(Algorithm::Ed25519)?;
    let subject = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Issuer:  {}", issuer.classic_address());
    println!("  Subject: {}", subject.classic_address());

    // --- 2. Fund both wallets ---
    println!("\nFunding wallets...");
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &subject).await?;
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Issuer creates a KYC credential for the subject ---
    let credential_type = Blob::new(b"KYC".to_vec());
    let credential_uri = Blob::new(b"https://example.com/kyc/verify".to_vec());

    println!("\nStep 1: Issuer creates credential...");
    println!("  CredentialType: KYC (3 bytes)");
    println!("  URI:            https://example.com/kyc/verify");

    let create_tx = Transaction::CredentialCreate {
        common: make_common(*issuer.account_id()),
        fields: CredentialCreate {
            subject: *subject.account_id(),
            credential_type: credential_type.clone(),
            expiration: None,
            uri: Some(credential_uri),
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(create_tx);
    autofill(&client, &mut unsigned_create).await?;

    let signed_create = sign_transaction(&unsigned_create, &issuer)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  Credential created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 4. Query credential (should exist but not yet accepted) ---
    println!("\nStep 2: Querying subject's credential objects (pre-accept)...");
    query_credentials(&client, "Subject credentials", *subject.account_id()).await?;

    // --- 5. Subject accepts the credential ---
    println!("\nStep 3: Subject accepts credential...");

    let accept_tx = Transaction::CredentialAccept {
        common: make_common(*subject.account_id()),
        fields: CredentialAccept {
            issuer: *issuer.account_id(),
            credential_type: credential_type.clone(),
        },
    };

    let mut unsigned_accept = UnsignedTransaction::new(accept_tx);
    autofill(&client, &mut unsigned_accept).await?;

    let signed_accept = sign_transaction(&unsigned_accept, &subject)?;
    let accept_result: TransactionResult = submit_and_wait(&client, &signed_accept).await?;

    println!("\n  Credential accepted!");
    println!("  Hash:   {}", accept_result.hash);
    println!("  Result: {}", accept_result.result_code);
    println!("  Ledger: {}", accept_result.ledger_index);

    // --- 6. Query credential again (should show as accepted) ---
    println!("\nStep 4: Querying subject's credential objects (post-accept)...");
    query_credentials(&client, "Subject credentials", *subject.account_id()).await?;

    // --- 7. Subject deletes the credential ---
    println!("\nStep 5: Subject deletes credential...");

    let delete_tx = Transaction::CredentialDelete {
        common: make_common(*subject.account_id()),
        fields: CredentialDelete {
            subject: *subject.account_id(),
            issuer: *issuer.account_id(),
            credential_type: credential_type.clone(),
        },
    };

    let mut unsigned_delete = UnsignedTransaction::new(delete_tx);
    autofill(&client, &mut unsigned_delete).await?;

    let signed_delete = sign_transaction(&unsigned_delete, &subject)?;
    let delete_result: TransactionResult = submit_and_wait(&client, &signed_delete).await?;

    println!("\n  Credential deleted!");
    println!("  Hash:   {}", delete_result.hash);
    println!("  Result: {}", delete_result.result_code);
    println!("  Ledger: {}", delete_result.ledger_index);

    // --- 8. Query one last time (should be empty) ---
    println!("\nStep 6: Querying subject's credential objects (post-delete)...");
    let remaining = query_credentials(&client, "Subject credentials", *subject.account_id()).await?;

    if remaining.is_empty() {
        println!("\n  Confirmed: no credential objects remain on the subject account.");
    } else {
        println!("\n  Warning: {} credential object(s) still present.", remaining.len());
    }

    println!("\nDone.");
    Ok(())
}

//! Example: DID Operations (XLS-40)
//!
//! Demonstrates the Decentralized Identifier lifecycle on the XRP Ledger:
//! create a DID, query it, update it, and delete it.
//!
//! Run: `cargo run -p xrpl-mithril --example did_operations`
//! Requires: Network access to XRPL testnet with DID amendment active

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountObjectsRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::models::transactions::did::{DIDDelete, DIDSet};
use xrpl_mithril::models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::submit::submit_and_wait;
use xrpl_mithril::tx::{sign_transaction, TransactionResult};
use xrpl_mithril::types::{AccountId, Amount, Blob, XrpAmount};
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
    let http = reqwest::Client::new();
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 1. Generate wallet and fund via faucet ---
    println!("Generating wallet...");
    let wallet = Wallet::generate(Algorithm::Ed25519)?;
    println!("  Address: {}", wallet.classic_address());

    println!("\nFunding wallet via testnet faucet...");
    fund_wallet(&http, &wallet).await?;
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 2. Create DID (DIDSet) ---
    println!("\nCreating DID...");
    println!("  Data: did:xrpl:1");
    println!("  URI:  https://example.com/did/1");

    let did_set_tx = Transaction::DIDSet {
        common: make_common(*wallet.account_id()),
        fields: DIDSet {
            data: Some(Blob::new(b"did:xrpl:1".to_vec())),
            uri: Some(Blob::new(b"https://example.com/did/1".to_vec())),
            attestation: None,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(did_set_tx);
    autofill(&client, &mut unsigned_create).await?;

    let signed_create = sign_transaction(&unsigned_create, &wallet)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  DID created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 3. Query DID via AccountObjectsRequest ---
    println!("\nQuerying DID objects...");
    let objects_resp = client
        .request(AccountObjectsRequest {
            account: *wallet.account_id(),
            object_type: Some("did".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    println!(
        "  DID objects: {}",
        serde_json::to_string_pretty(&objects_resp.account_objects)?
    );

    // --- 4. Update DID (DIDSet with new data) ---
    println!("\nUpdating DID with new data...");
    println!("  Data: did:xrpl:1:v2");

    let did_update_tx = Transaction::DIDSet {
        common: make_common(*wallet.account_id()),
        fields: DIDSet {
            data: Some(Blob::new(b"did:xrpl:1:v2".to_vec())),
            uri: None,
            attestation: None,
        },
    };

    let mut unsigned_update = UnsignedTransaction::new(did_update_tx);
    autofill(&client, &mut unsigned_update).await?;

    let signed_update = sign_transaction(&unsigned_update, &wallet)?;
    let update_result: TransactionResult = submit_and_wait(&client, &signed_update).await?;

    println!("\n  DID updated!");
    println!("  Hash:   {}", update_result.hash);
    println!("  Result: {}", update_result.result_code);
    println!("  Ledger: {}", update_result.ledger_index);

    // --- 5. Delete DID (DIDDelete) ---
    println!("\nDeleting DID...");

    let did_delete_tx = Transaction::DIDDelete {
        common: make_common(*wallet.account_id()),
        fields: DIDDelete {},
    };

    let mut unsigned_delete = UnsignedTransaction::new(did_delete_tx);
    autofill(&client, &mut unsigned_delete).await?;

    let signed_delete = sign_transaction(&unsigned_delete, &wallet)?;
    let delete_result: TransactionResult = submit_and_wait(&client, &signed_delete).await?;

    println!("\n  DID deleted!");
    println!("  Hash:   {}", delete_result.hash);
    println!("  Result: {}", delete_result.result_code);
    println!("  Ledger: {}", delete_result.ledger_index);

    // --- 6. Verify deletion via AccountObjectsRequest ---
    println!("\nVerifying DID deletion...");
    let objects_after = client
        .request(AccountObjectsRequest {
            account: *wallet.account_id(),
            object_type: Some("did".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if objects_after.account_objects.is_empty() {
        println!("  DID objects after deletion: (none -- deletion confirmed)");
    } else {
        println!(
            "  DID objects after deletion: {}",
            serde_json::to_string_pretty(&objects_after.account_objects)?
        );
    }

    println!("\nDone.");
    Ok(())
}

//! Example: Price Oracle (XLS-47)
//!
//! Demonstrates on-ledger price feeds: create an oracle with XRP/USD price data,
//! update the price, and delete the oracle.
//!
//! Run: `cargo run -p xrpl-mithril --example price_oracle`
//! Requires: Network access to XRPL testnet with Price Oracle amendment active

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountObjectsRequest;
use xrpl_mithril::models::requests::ledger::LedgerRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::models::transactions::oracle::{OracleDelete, OracleSet, PriceData};
use xrpl_mithril::models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::submit::submit_and_wait;
use xrpl_mithril::tx::{sign_transaction, TransactionResult};
use xrpl_mithril::types::currency::CurrencyCode;
use xrpl_mithril::types::{AccountId, Amount, Blob, XrpAmount};
use xrpl_mithril::wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";

/// Ripple epoch offset: seconds between Unix epoch (1970) and Ripple epoch (2000).
const RIPPLE_EPOCH_OFFSET: u32 = 946_684_800;

/// Fetch the current time as Unix epoch seconds, derived from the latest
/// validated ledger's close time.
///
/// OracleSet's `LastUpdateTime` uses **Unix epoch** (seconds since 1970-01-01),
/// unlike most XRPL timestamp fields which use Ripple epoch (seconds since
/// 2000-01-01). The ledger's `close_time` is in Ripple epoch, so we convert
/// by adding the offset. Using the ledger's time (rather than the system clock)
/// ensures we stay within the required 300-second window.
async fn oracle_update_time(client: &JsonRpcClient) -> Result<u32, Box<dyn std::error::Error>> {
    let resp = client
        .request(LedgerRequest {
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            ..Default::default()
        })
        .await?;
    let ripple_time = resp
        .ledger
        .close_time
        .ok_or("validated ledger missing close_time")?;
    Ok(ripple_time + RIPPLE_EPOCH_OFFSET)
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

    // --- 2. Create oracle with XRP/USD price (OracleSet) ---
    // Price = 52000 * 10^(-4) = $5.20
    println!("\nCreating oracle with XRP/USD price data...");
    println!("  Oracle Document ID: 1");
    println!("  Provider:           xrpl-mithril-example");
    println!("  Asset Class:        currency");
    println!("  XRP/USD Price:      52000 * 10^(-4) = $5.20");

    let oracle_create_tx = Transaction::OracleSet {
        common: make_common(*wallet.account_id()),
        fields: OracleSet {
            oracle_document_id: 1,
            provider: Some(Blob::new(b"xrpl-mithril-example".to_vec())),
            asset_class: Some(Blob::new(b"currency".to_vec())),
            last_update_time: Some(oracle_update_time(&client).await?),
            price_data_series: Some(vec![PriceData {
                base_asset: CurrencyCode::from_ascii("XRP")?,
                quote_asset: CurrencyCode::from_ascii("USD")?,
                asset_price: Some(52000),
                scale: Some(4),
            }].into()),
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(oracle_create_tx);
    autofill(&client, &mut unsigned_create).await?;

    let signed_create = sign_transaction(&unsigned_create, &wallet)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  Oracle created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 3. Query oracle via AccountObjectsRequest ---
    println!("\nQuerying oracle objects...");
    let objects_resp = client
        .request(AccountObjectsRequest {
            account: *wallet.account_id(),
            object_type: Some("oracle".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    println!(
        "  Oracle objects: {}",
        serde_json::to_string_pretty(&objects_resp.account_objects)?
    );

    // --- 4. Update oracle with new price (OracleSet) ---
    // Price = 53500 * 10^(-4) = $5.35
    println!("\nUpdating oracle with new XRP/USD price...");
    println!("  XRP/USD Price:      53500 * 10^(-4) = $5.35");

    let oracle_update_tx = Transaction::OracleSet {
        common: make_common(*wallet.account_id()),
        fields: OracleSet {
            oracle_document_id: 1,
            provider: None,
            asset_class: None,
            last_update_time: Some(oracle_update_time(&client).await?),
            price_data_series: Some(vec![PriceData {
                base_asset: CurrencyCode::from_ascii("XRP")?,
                quote_asset: CurrencyCode::from_ascii("USD")?,
                asset_price: Some(53500),
                scale: Some(4),
            }].into()),
        },
    };

    let mut unsigned_update = UnsignedTransaction::new(oracle_update_tx);
    autofill(&client, &mut unsigned_update).await?;

    let signed_update = sign_transaction(&unsigned_update, &wallet)?;
    let update_result: TransactionResult = submit_and_wait(&client, &signed_update).await?;

    println!("\n  Oracle updated!");
    println!("  Hash:   {}", update_result.hash);
    println!("  Result: {}", update_result.result_code);
    println!("  Ledger: {}", update_result.ledger_index);

    // --- 5. Delete oracle (OracleDelete) ---
    println!("\nDeleting oracle...");

    let oracle_delete_tx = Transaction::OracleDelete {
        common: make_common(*wallet.account_id()),
        fields: OracleDelete {
            oracle_document_id: 1,
        },
    };

    let mut unsigned_delete = UnsignedTransaction::new(oracle_delete_tx);
    autofill(&client, &mut unsigned_delete).await?;

    let signed_delete = sign_transaction(&unsigned_delete, &wallet)?;
    let delete_result: TransactionResult = submit_and_wait(&client, &signed_delete).await?;

    println!("\n  Oracle deleted!");
    println!("  Hash:   {}", delete_result.hash);
    println!("  Result: {}", delete_result.result_code);
    println!("  Ledger: {}", delete_result.ledger_index);

    // --- 6. Verify deletion ---
    println!("\nVerifying oracle deletion...");
    let objects_after = client
        .request(AccountObjectsRequest {
            account: *wallet.account_id(),
            object_type: Some("oracle".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if objects_after.account_objects.is_empty() {
        println!("  Oracle objects after deletion: (none -- deletion confirmed)");
    } else {
        println!(
            "  Oracle objects after deletion: {}",
            serde_json::to_string_pretty(&objects_after.account_objects)?
        );
    }

    println!("\nDone.");
    Ok(())
}

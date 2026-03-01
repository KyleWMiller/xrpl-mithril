//! Example: Check Operations
//!
//! Demonstrates the Check lifecycle: create a check, cash it (exact amount),
//! create another check and cancel it.
//!
//! Run: `cargo run -p xrpl-mithril --example check_operations`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::{AccountInfoRequest, AccountObjectsRequest};
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::payment::{CheckCancel, CheckCash, CheckCreate};
use xrpl_mithril::xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::xrpl_models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::xrpl_tx::autofill::autofill;
use xrpl_mithril::xrpl_tx::submit::submit_and_wait;
use xrpl_mithril::xrpl_tx::{sign_transaction, TransactionResult};
use xrpl_mithril::xrpl_types::{AccountId, Amount, Hash256, XrpAmount};
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

/// Query and return an account's XRP balance in drops.
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
    println!("\nFunding wallets...");
    let http = reqwest::Client::new();
    fund_wallet(&http, &sender).await?;
    fund_wallet(&http, &receiver).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 3. Create a check for 25 XRP ---
    println!("\nCreating check for 25 XRP from sender to receiver...");
    let check_create = Transaction::CheckCreate {
        common: make_common(*sender.account_id()),
        fields: CheckCreate {
            destination: *receiver.account_id(),
            send_max: Amount::Xrp(XrpAmount::from_drops(25_000_000)?),
            destination_tag: None,
            expiration: None,
            invoice_id: None,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(check_create);
    autofill(&client, &mut unsigned_create).await?;

    let signed_create = sign_transaction(&unsigned_create, &sender)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("  Check created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 4. Find the Check ID from account_objects ---
    println!("\nLooking up Check object on ledger...");
    let objects = client
        .request(AccountObjectsRequest {
            account: *sender.account_id(),
            object_type: Some("check".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    let check_obj = &objects.account_objects[0];
    let check_id_str = check_obj["index"]
        .as_str()
        .ok_or("missing check index")?;
    let check_id = Hash256::from_hex(check_id_str)?;
    println!("  Check ID: {check_id_str}");

    // --- 5. Receiver cashes the check for exact amount 25 XRP ---
    println!("\nReceiver cashing check for 25 XRP...");
    let check_cash = Transaction::CheckCash {
        common: make_common(*receiver.account_id()),
        fields: CheckCash {
            check_id,
            amount: Some(Amount::Xrp(XrpAmount::from_drops(25_000_000)?)),
            deliver_min: None,
        },
    };

    let mut unsigned_cash = UnsignedTransaction::new(check_cash);
    autofill(&client, &mut unsigned_cash).await?;

    let signed_cash = sign_transaction(&unsigned_cash, &receiver)?;
    let cash_result: TransactionResult = submit_and_wait(&client, &signed_cash).await?;

    println!("  Check cashed!");
    println!("  Hash:   {}", cash_result.hash);
    println!("  Result: {}", cash_result.result_code);
    println!("  Ledger: {}", cash_result.ledger_index);

    // --- 6. Create another check for 10 XRP ---
    println!("\nCreating second check for 10 XRP from sender to receiver...");
    let check_create_2 = Transaction::CheckCreate {
        common: make_common(*sender.account_id()),
        fields: CheckCreate {
            destination: *receiver.account_id(),
            send_max: Amount::Xrp(XrpAmount::from_drops(10_000_000)?),
            destination_tag: None,
            expiration: None,
            invoice_id: None,
        },
    };

    let mut unsigned_create_2 = UnsignedTransaction::new(check_create_2);
    autofill(&client, &mut unsigned_create_2).await?;

    let signed_create_2 = sign_transaction(&unsigned_create_2, &sender)?;
    let create_result_2: TransactionResult = submit_and_wait(&client, &signed_create_2).await?;

    println!("  Second check created!");
    println!("  Hash:   {}", create_result_2.hash);
    println!("  Result: {}", create_result_2.result_code);
    println!("  Ledger: {}", create_result_2.ledger_index);

    // --- 7. Find the second Check ID ---
    println!("\nLooking up second Check object on ledger...");
    let objects_2 = client
        .request(AccountObjectsRequest {
            account: *sender.account_id(),
            object_type: Some("check".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    let check_obj_2 = &objects_2.account_objects[0];
    let check_id_str_2 = check_obj_2["index"]
        .as_str()
        .ok_or("missing check index")?;
    let check_id_2 = Hash256::from_hex(check_id_str_2)?;
    println!("  Check ID: {check_id_str_2}");

    // --- 8. Cancel the second check ---
    println!("\nSender cancelling second check...");
    let check_cancel = Transaction::CheckCancel {
        common: make_common(*sender.account_id()),
        fields: CheckCancel {
            check_id: check_id_2,
        },
    };

    let mut unsigned_cancel = UnsignedTransaction::new(check_cancel);
    autofill(&client, &mut unsigned_cancel).await?;

    let signed_cancel = sign_transaction(&unsigned_cancel, &sender)?;
    let cancel_result: TransactionResult = submit_and_wait(&client, &signed_cancel).await?;

    println!("  Check cancelled!");
    println!("  Hash:   {}", cancel_result.hash);
    println!("  Result: {}", cancel_result.result_code);
    println!("  Ledger: {}", cancel_result.ledger_index);

    // --- 9. Print final balances ---
    println!("\nFinal balances:");
    let sender_balance = get_balance(&client, *sender.account_id()).await?;
    let receiver_balance = get_balance(&client, *receiver.account_id()).await?;
    let sender_xrp = sender_balance as f64 / 1_000_000.0;
    let receiver_xrp = receiver_balance as f64 / 1_000_000.0;
    println!("  Sender:   {sender_xrp:.6} XRP ({sender_balance} drops)");
    println!("  Receiver: {receiver_xrp:.6} XRP ({receiver_balance} drops)");

    println!("\nDone.");
    Ok(())
}

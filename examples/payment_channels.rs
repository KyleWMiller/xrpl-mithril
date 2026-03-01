//! Example: Payment Channels
//!
//! Demonstrates the payment channel lifecycle: create a channel, fund it with
//! additional XRP, query channel state, and close it.
//!
//! Run: `cargo run -p xrpl-mithril --example payment_channels`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::{AccountChannelsRequest, AccountInfoRequest};
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::payment_channel::{
    PaymentChannelClaim, PaymentChannelCreate, PaymentChannelFund,
};
use xrpl_mithril::xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::xrpl_models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::xrpl_tx::autofill::autofill;
use xrpl_mithril::xrpl_tx::submit::submit_and_wait;
use xrpl_mithril::xrpl_tx::{sign_transaction, TransactionResult};
use xrpl_mithril::xrpl_types::{AccountId, Amount, Blob, Hash256, XrpAmount};
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
    let source = Wallet::generate(Algorithm::Ed25519)?;
    let destination = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Source:      {}", source.classic_address());
    println!("  Destination: {}", destination.classic_address());

    // --- 2. Fund both wallets via testnet faucet ---
    println!("\nFunding wallets...");
    let http = reqwest::Client::new();
    fund_wallet(&http, &source).await?;
    fund_wallet(&http, &destination).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = JsonRpcClient::new(TESTNET_RPC)?;

    let source_balance_before = get_balance(&client, *source.account_id()).await?;
    let dest_balance_before = get_balance(&client, *destination.account_id()).await?;
    println!("\nBalances before payment channel:");
    println!(
        "  Source:      {:.6} XRP",
        source_balance_before as f64 / 1_000_000.0
    );
    println!(
        "  Destination: {:.6} XRP",
        dest_balance_before as f64 / 1_000_000.0
    );

    // --- 3. Create a payment channel ---
    println!("\nStep 1: Creating payment channel (10 XRP, settle_delay=60s)...");

    let create_tx = Transaction::PaymentChannelCreate {
        common: make_common(*source.account_id()),
        fields: PaymentChannelCreate {
            destination: *destination.account_id(),
            amount: Amount::Xrp(XrpAmount::from_drops(10_000_000)?),
            settle_delay: 60,
            public_key: Blob::new(source.public_key().to_vec()),
            cancel_after: None,
            destination_tag: None,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(create_tx);
    autofill(&client, &mut unsigned_create).await?;

    println!("  Fee:      {:?}", unsigned_create.common().fee);
    println!("  Sequence: {}", unsigned_create.common().sequence);

    let signed_create = sign_transaction(&unsigned_create, &source)?;
    println!("  Hash:     {}", signed_create.hash());

    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;
    println!("\n  Channel created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 4. Query channels to get the channel ID ---
    println!("\nStep 2: Querying account channels...");

    let channels = client
        .request(AccountChannelsRequest {
            account: *source.account_id(),
            destination_account: Some(*destination.account_id()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if channels.channels.is_empty() {
        return Err("No channels found after creation".into());
    }

    let channel_info = &channels.channels[0];
    let channel_id_str = &channel_info.channel_id;
    let channel_id = Hash256::from_hex(channel_id_str)?;

    println!("  Channel ID:    {channel_id_str}");
    println!("  Amount:        {} drops", channel_info.amount);
    println!("  Balance:       {} drops", channel_info.balance);
    println!("  Settle delay:  {}s", channel_info.settle_delay);

    // --- 5. Fund the channel with 5 more XRP ---
    println!("\nStep 3: Funding channel with 5 more XRP...");

    let fund_tx = Transaction::PaymentChannelFund {
        common: make_common(*source.account_id()),
        fields: PaymentChannelFund {
            channel: channel_id,
            amount: Amount::Xrp(XrpAmount::from_drops(5_000_000)?),
            expiration: None,
        },
    };

    let mut unsigned_fund = UnsignedTransaction::new(fund_tx);
    autofill(&client, &mut unsigned_fund).await?;

    let signed_fund = sign_transaction(&unsigned_fund, &source)?;
    println!("  Hash: {}", signed_fund.hash());

    let fund_result: TransactionResult = submit_and_wait(&client, &signed_fund).await?;
    println!("\n  Channel funded!");
    println!("  Hash:   {}", fund_result.hash);
    println!("  Result: {}", fund_result.result_code);
    println!("  Ledger: {}", fund_result.ledger_index);

    // Verify updated channel amount
    let channels_after_fund = client
        .request(AccountChannelsRequest {
            account: *source.account_id(),
            destination_account: Some(*destination.account_id()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if let Some(ch) = channels_after_fund.channels.first() {
        println!("  Updated amount: {} drops (was 10000000)", ch.amount);
    }

    // --- 6. Close the channel ---
    // The source requests channel closure using the tfClose flag.
    // Without a claim, this initiates the settle_delay countdown.
    // After settle_delay expires, a second close request would finalize it.
    println!("\nStep 4: Requesting channel closure (tfClose)...");

    let mut close_common = make_common(*source.account_id());
    close_common.flags = Some(0x00010000); // tfClose

    let close_tx = Transaction::PaymentChannelClaim {
        common: close_common,
        fields: PaymentChannelClaim {
            channel: channel_id,
            balance: None,
            amount: None,
            signature: None,
            public_key: None,
        },
    };

    let mut unsigned_close = UnsignedTransaction::new(close_tx);
    autofill(&client, &mut unsigned_close).await?;

    let signed_close = sign_transaction(&unsigned_close, &source)?;
    println!("  Hash: {}", signed_close.hash());

    let close_result: TransactionResult = submit_and_wait(&client, &signed_close).await?;
    println!("\n  Channel close requested!");
    println!("  Hash:   {}", close_result.hash);
    println!("  Result: {}", close_result.result_code);
    println!("  Ledger: {}", close_result.ledger_index);
    println!("  Note:   Channel will close after settle_delay (60s) expires.");
    println!("          Unclaimed XRP returns to the source address.");

    // --- 7. Final balances ---
    println!("\nFinal balances:");
    let source_balance_after = get_balance(&client, *source.account_id()).await?;
    let dest_balance_after = get_balance(&client, *destination.account_id()).await?;
    println!(
        "  Source:      {:.6} XRP",
        source_balance_after as f64 / 1_000_000.0
    );
    println!(
        "  Destination: {:.6} XRP",
        dest_balance_after as f64 / 1_000_000.0
    );

    let source_spent = source_balance_before.saturating_sub(source_balance_after);
    println!(
        "\n  Source spent: {:.6} XRP (channel deposit + fees; will recover after settle_delay)",
        source_spent as f64 / 1_000_000.0
    );

    println!("\nDone.");
    Ok(())
}

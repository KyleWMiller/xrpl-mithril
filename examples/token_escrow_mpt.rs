//! Example: Token Escrow with MPT (XLS-85)
//!
//! Demonstrates escrow with Multi-Purpose Tokens: create an MPT issuance with
//! escrow enabled, transfer tokens, create a time-locked MPT escrow, and finish it.
//!
//! Run: `cargo run -p xrpl-mithril --example token_escrow_mpt`
//! Requires: Network access to XRPL testnet with TokenEscrow amendment (XLS-85) active

// NOTE: This example requires the TokenEscrow amendment (XLS-85), activated
// on mainnet Feb 13 2026. If running on testnet, ensure the amendment is active.

use std::time::{SystemTime, UNIX_EPOCH};

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountInfoRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::models::transactions::escrow::{EscrowCreate, EscrowFinish};
use xrpl_mithril::models::transactions::mpt::{MPTokenAuthorize, MPTokenIssuanceCreate};
use xrpl_mithril::models::transactions::payment::Payment;
use xrpl_mithril::models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::submit::submit_and_wait;
use xrpl_mithril::tx::{sign_transaction, TransactionResult};
use xrpl_mithril::types::currency::MptIssuanceId;
use xrpl_mithril::types::{AccountId, Amount, MptAmount, XrpAmount};
use xrpl_mithril::wallet::{Algorithm, Wallet};

const TESTNET_RPC: &str = "https://s.altnet.rippletest.net:51234";
const FAUCET_URL: &str = "https://faucet.altnet.rippletest.net/accounts";

/// Ripple epoch offset: seconds between Unix epoch and Ripple epoch (2000-01-01T00:00:00Z).
const RIPPLE_EPOCH_OFFSET: u64 = 946_684_800;

/// MPT flags
const TF_MPT_CAN_TRANSFER: u32 = 0x0020;
const TF_MPT_CAN_ESCROW: u32 = 0x0008;

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

/// Derive an MPT Issuance ID from the create transaction's sequence and the issuer's account.
///
/// The MPT Issuance ID is 24 bytes: sequence (4 bytes big-endian) || account_id (20 bytes).
fn derive_mpt_issuance_id(sequence: u32, issuer: &AccountId) -> MptIssuanceId {
    let mut bytes = [0u8; 24];
    bytes[0..4].copy_from_slice(&sequence.to_be_bytes());
    bytes[4..24].copy_from_slice(issuer.as_bytes());
    MptIssuanceId::from_bytes(bytes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::new();
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let issuer = Wallet::generate(Algorithm::Ed25519)?;
    let sender = Wallet::generate(Algorithm::Secp256k1)?;
    let receiver = Wallet::generate(Algorithm::Ed25519)?;
    println!("  Issuer:   {}", issuer.classic_address());
    println!("  Sender:   {}", sender.classic_address());
    println!("  Receiver: {}", receiver.classic_address());

    // --- 2. Fund all three wallets ---
    println!("\nFunding wallets...");
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &sender).await?;
    fund_wallet(&http, &receiver).await?;
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Create MPT issuance with escrow + transfer flags ---
    println!("\nCreating MPT issuance...");
    println!("  Max supply:  1,000,000 tokens");
    println!("  Asset scale: 2 (0.01 smallest unit)");
    println!("  Flags:       tfMPTCanTransfer | tfMPTCanEscrow");

    let mut common = make_common(*issuer.account_id());
    common.flags = Some(TF_MPT_CAN_TRANSFER | TF_MPT_CAN_ESCROW);

    let create_tx = Transaction::MPTokenIssuanceCreate {
        common,
        fields: MPTokenIssuanceCreate {
            max_amount: Some(1_000_000),
            asset_scale: Some(2),
            transfer_fee: None,
            metadata: None,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(create_tx);
    autofill(&client, &mut unsigned_create).await?;

    // Remember the sequence -- needed to derive the MPT Issuance ID
    let create_sequence = unsigned_create.common().sequence;

    let signed_create = sign_transaction(&unsigned_create, &issuer)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  Issuance created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);

    // Derive the MPT Issuance ID
    let mpt_id = derive_mpt_issuance_id(create_sequence, issuer.account_id());
    println!("  MPT ID: {mpt_id}");

    // --- 4. Sender opts in (MPTokenAuthorize) ---
    println!("\nSender authorizing (opt-in) for MPT...");
    let sender_auth_tx = Transaction::MPTokenAuthorize {
        common: make_common(*sender.account_id()),
        fields: MPTokenAuthorize {
            mpt_issuance_id: mpt_id,
            holder: None,
        },
    };

    let mut unsigned_sender_auth = UnsignedTransaction::new(sender_auth_tx);
    autofill(&client, &mut unsigned_sender_auth).await?;
    let signed_sender_auth = sign_transaction(&unsigned_sender_auth, &sender)?;
    let sender_auth_result: TransactionResult =
        submit_and_wait(&client, &signed_sender_auth).await?;

    println!("  Authorized!");
    println!("  Hash:   {}", sender_auth_result.hash);
    println!("  Result: {}", sender_auth_result.result_code);

    // --- 5. Receiver opts in (MPTokenAuthorize) ---
    println!("\nReceiver authorizing (opt-in) for MPT...");
    let receiver_auth_tx = Transaction::MPTokenAuthorize {
        common: make_common(*receiver.account_id()),
        fields: MPTokenAuthorize {
            mpt_issuance_id: mpt_id,
            holder: None,
        },
    };

    let mut unsigned_receiver_auth = UnsignedTransaction::new(receiver_auth_tx);
    autofill(&client, &mut unsigned_receiver_auth).await?;
    let signed_receiver_auth = sign_transaction(&unsigned_receiver_auth, &receiver)?;
    let receiver_auth_result: TransactionResult =
        submit_and_wait(&client, &signed_receiver_auth).await?;

    println!("  Authorized!");
    println!("  Hash:   {}", receiver_auth_result.hash);
    println!("  Result: {}", receiver_auth_result.result_code);

    // --- 6. Transfer 1000 MPT units from issuer to sender ---
    let transfer_amount = 1000i64;
    println!("\nTransferring {transfer_amount} MPT units from issuer to sender...");

    let payment_tx = Transaction::Payment {
        common: make_common(*issuer.account_id()),
        fields: Payment {
            destination: *sender.account_id(),
            amount: Amount::Mpt(MptAmount {
                value: transfer_amount,
                mpt_issuance_id: mpt_id,
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

    println!("  Transfer complete!");
    println!("  Hash:   {}", payment_result.hash);
    println!("  Result: {}", payment_result.result_code);

    // --- 7. Create time-locked escrow with MPT ---
    let escrow_amount = 500i64;
    let finish_after = ripple_now()? + 10;
    let cancel_after = ripple_now()? + 60;

    println!("\nCreating MPT escrow:");
    println!("  Amount:       {escrow_amount} MPT units");
    println!("  From:         sender -> receiver");
    println!("  Finish after: {finish_after} (Ripple epoch, ~10s from now)");
    println!("  Cancel after: {cancel_after} (Ripple epoch, ~60s from now)");

    let escrow_create_tx = Transaction::EscrowCreate {
        common: make_common(*sender.account_id()),
        fields: EscrowCreate {
            destination: *receiver.account_id(),
            amount: Amount::Mpt(MptAmount {
                value: escrow_amount,
                mpt_issuance_id: mpt_id,
            }),
            finish_after: Some(finish_after),
            cancel_after: Some(cancel_after),
            condition: None,
            destination_tag: None,
        },
    };

    let mut unsigned_escrow = UnsignedTransaction::new(escrow_create_tx);
    autofill(&client, &mut unsigned_escrow).await?;

    // Remember the sequence number -- needed to finish the escrow
    let escrow_sequence = unsigned_escrow.common().sequence;
    println!("  Escrow create sequence: {escrow_sequence}");

    let signed_escrow = sign_transaction(&unsigned_escrow, &sender)?;
    let escrow_result: TransactionResult = submit_and_wait(&client, &signed_escrow).await?;

    println!("\n  Escrow created!");
    println!("  Hash:   {}", escrow_result.hash);
    println!("  Result: {}", escrow_result.result_code);
    println!("  Ledger: {}", escrow_result.ledger_index);

    // --- 8. Wait for finish_after to pass ---
    let wait_secs = 12; // 10s + 2s buffer
    println!("\nWaiting {wait_secs} seconds for finish_after to pass...");
    tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

    // --- 9. Finish the escrow ---
    println!("Finishing escrow...");
    let escrow_finish_tx = Transaction::EscrowFinish {
        common: make_common(*receiver.account_id()),
        fields: EscrowFinish {
            owner: *sender.account_id(),
            offer_sequence: escrow_sequence,
            condition: None,
            fulfillment: None,
        },
    };

    let mut unsigned_finish = UnsignedTransaction::new(escrow_finish_tx);
    autofill(&client, &mut unsigned_finish).await?;

    let signed_finish = sign_transaction(&unsigned_finish, &receiver)?;
    let finish_result: TransactionResult = submit_and_wait(&client, &signed_finish).await?;

    println!("\n  Escrow finished!");
    println!("  Hash:   {}", finish_result.hash);
    println!("  Result: {}", finish_result.result_code);
    println!("  Ledger: {}", finish_result.ledger_index);

    // --- 10. Verify final account states ---
    println!("\nFinal account states:");
    let issuer_info = client
        .request(AccountInfoRequest {
            account: *issuer.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;
    let sender_info = client
        .request(AccountInfoRequest {
            account: *sender.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;
    let receiver_info = client
        .request(AccountInfoRequest {
            account: *receiver.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;

    println!(
        "  Issuer XRP balance:   {} drops",
        issuer_info.account_data.balance
    );
    println!(
        "  Sender XRP balance:   {} drops",
        sender_info.account_data.balance
    );
    println!(
        "  Receiver XRP balance: {} drops",
        receiver_info.account_data.balance
    );
    println!("  (MPT balances are visible via account_objects, not account_info)");
    println!("  Expected: sender started with 1000 MPT, escrowed 500, receiver received 500 via escrow");

    println!("\nDone.");
    Ok(())
}

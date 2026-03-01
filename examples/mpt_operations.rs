//! Example: Multi-Purpose Token (MPT) Operations
//!
//! Demonstrates the MPT lifecycle: create an issuance, authorize a holder,
//! and transfer tokens. MPTs are the XRPL's modern token primitive alongside
//! trust lines (activated in rippled 2.3.0+).
//!
//! Run: `cargo run -p xrpl-mithril --example mpt_operations`
//! Requires: Network access to XRPL testnet with MPT support

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountInfoRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
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

/// MPT flags
const TF_MPT_CAN_TRANSFER: u32 = 0x0020;

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
    let holder = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Issuer: {}", issuer.classic_address());
    println!("  Holder: {}", holder.classic_address());

    // --- 2. Fund both wallets ---
    println!("\nFunding wallets...");
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &holder).await?;
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Create MPT Issuance ---
    println!("\nCreating MPT issuance...");
    println!("  Max supply:    1,000,000 tokens");
    println!("  Asset scale:   2 (0.01 smallest unit)");
    println!("  Transfer fee:  100 basis points (1%)");
    println!("  Flags:         tfMPTCanTransfer");

    let mut common = make_common(*issuer.account_id());
    common.flags = Some(TF_MPT_CAN_TRANSFER);

    let create_tx = Transaction::MPTokenIssuanceCreate {
        common,
        fields: MPTokenIssuanceCreate {
            max_amount: Some(1_000_000),
            asset_scale: Some(2),
            transfer_fee: Some(100),
            metadata: None,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(create_tx);
    autofill(&client, &mut unsigned_create).await?;

    // Remember the sequence — needed to derive the MPT Issuance ID
    let create_sequence = unsigned_create.common().sequence;

    let signed_create = sign_transaction(&unsigned_create, &issuer)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  Issuance created!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);

    // Derive the MPT Issuance ID
    let mpt_id = derive_mpt_issuance_id(create_sequence, issuer.account_id());
    println!("  MPT ID: {mpt_id}");

    // --- 4. Holder opts in (MPTokenAuthorize) ---
    println!("\nHolder authorizing (opt-in) for MPT...");
    let authorize_tx = Transaction::MPTokenAuthorize {
        common: make_common(*holder.account_id()),
        fields: MPTokenAuthorize {
            mpt_issuance_id: mpt_id,
            holder: None, // Holder opting in for themselves
        },
    };

    let mut unsigned_auth = UnsignedTransaction::new(authorize_tx);
    autofill(&client, &mut unsigned_auth).await?;
    let signed_auth = sign_transaction(&unsigned_auth, &holder)?;
    let auth_result: TransactionResult = submit_and_wait(&client, &signed_auth).await?;

    println!("  Authorized!");
    println!("  Hash:   {}", auth_result.hash);
    println!("  Result: {}", auth_result.result_code);

    // --- 5. Transfer tokens from issuer to holder ---
    let transfer_amount = 500i64; // 5.00 tokens (asset_scale = 2)
    println!("\nTransferring {transfer_amount} units (5.00 tokens) from issuer to holder...");

    let payment_tx = Transaction::Payment {
        common: make_common(*issuer.account_id()),
        fields: Payment {
            destination: *holder.account_id(),
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

    // --- 6. Verify via account_info ---
    println!("\nFinal account states:");
    let issuer_info = client
        .request(AccountInfoRequest {
            account: *issuer.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;
    let holder_info = client
        .request(AccountInfoRequest {
            account: *holder.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            queue: None,
            signer_lists: None,
        })
        .await?;

    println!(
        "  Issuer XRP balance: {} drops",
        issuer_info.account_data.balance
    );
    println!(
        "  Holder XRP balance: {} drops",
        holder_info.account_data.balance
    );
    println!("  (MPT balances are visible via account_objects, not account_info)");

    println!("\nDone.");
    Ok(())
}

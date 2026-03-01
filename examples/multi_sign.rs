//! Example: Multi-Signature Transaction
//!
//! Demonstrates multi-signing: set up a signer list with 2-of-3 quorum,
//! then build, multi-sign, combine, and submit a payment.
//!
//! Run: `cargo run -p xrpl-mithril --example multi_sign`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::AccountInfoRequest;
use xrpl_mithril::xrpl_models::requests::transaction::{SubmitRequest, TxRequest};
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::account::{SignerEntry, SignerListSet};
use xrpl_mithril::xrpl_models::transactions::payment::Payment;
use xrpl_mithril::xrpl_models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::xrpl_models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::xrpl_tx::autofill::autofill;
use xrpl_mithril::xrpl_tx::submit::submit_and_wait;
use xrpl_mithril::xrpl_tx::{sign_transaction, TransactionResult};
use xrpl_mithril::xrpl_types::{AccountId, Amount, XrpAmount};
use xrpl_mithril::xrpl_wallet::signer::{combine_signatures, multi_sign};
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

/// Query and display an account's XRP balance.
async fn get_balance(
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
    let http = reqwest::Client::new();
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 1. Generate wallets ---
    println!("Generating wallets...");
    let main_wallet = Wallet::generate(Algorithm::Ed25519)?;
    let signer1 = Wallet::generate(Algorithm::Ed25519)?;
    let signer2 = Wallet::generate(Algorithm::Secp256k1)?;
    let signer3 = Wallet::generate(Algorithm::Ed25519)?;
    let destination = Wallet::generate(Algorithm::Ed25519)?;

    println!("  Main wallet:  {}", main_wallet.classic_address());
    println!("  Signer 1:     {} (Ed25519)", signer1.classic_address());
    println!("  Signer 2:     {} (secp256k1)", signer2.classic_address());
    println!("  Signer 3:     {} (Ed25519)", signer3.classic_address());
    println!("  Destination:  {}", destination.classic_address());

    // --- 2. Fund main wallet and destination ---
    // Only the main wallet and destination need funding. Signers do not need
    // on-ledger accounts -- they just sign; the fee comes from main_wallet.
    // However, the destination must exist on-ledger to receive a payment.
    println!("\nFunding wallets via testnet faucet...");
    fund_wallet(&http, &main_wallet).await?;
    fund_wallet(&http, &destination).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Set up signer list on main wallet (2-of-3 quorum) ---
    println!("\nSetting up signer list (quorum=2, 3 signers with weight=1 each)...");

    let signer_list_tx = Transaction::SignerListSet {
        common: make_common(*main_wallet.account_id()),
        fields: SignerListSet {
            signer_quorum: 2,
            signer_entries: Some(vec![
                SignerEntry {
                    account: *signer1.account_id(),
                    signer_weight: 1,
                },
                SignerEntry {
                    account: *signer2.account_id(),
                    signer_weight: 1,
                },
                SignerEntry {
                    account: *signer3.account_id(),
                    signer_weight: 1,
                },
            ].into()),
        },
    };

    let mut unsigned_signer_list = UnsignedTransaction::new(signer_list_tx);
    autofill(&client, &mut unsigned_signer_list).await?;
    let signed_signer_list = sign_transaction(&unsigned_signer_list, &main_wallet)?;
    let signer_list_result: TransactionResult =
        submit_and_wait(&client, &signed_signer_list).await?;

    println!("  SignerListSet validated!");
    println!("  Hash:   {}", signer_list_result.hash);
    println!("  Result: {}", signer_list_result.result_code);
    println!("  Ledger: {}", signer_list_result.ledger_index);

    // --- 4. Check balances before payment ---
    println!("\nBalances before multi-signed payment:");
    get_balance(&client, "Main wallet", *main_wallet.account_id()).await?;
    get_balance(&client, "Destination", *destination.account_id()).await?;

    // --- 5. Build a payment from main wallet to destination ---
    println!("\nBuilding Payment: 5 XRP from main wallet to destination...");

    let payment_tx = Transaction::Payment {
        common: make_common(*main_wallet.account_id()),
        fields: Payment {
            destination: *destination.account_id(),
            amount: Amount::Xrp(XrpAmount::from_drops(5_000_000)?),
            send_max: None,
            deliver_min: None,
            destination_tag: None,
            invoice_id: None,
            paths: None,
        },
    };

    let mut unsigned_payment = UnsignedTransaction::new(payment_tx);
    autofill(&client, &mut unsigned_payment).await?;

    // Multi-signed transactions require a higher fee: base_fee * (1 + num_signers).
    // With 2 signers that would be 12 * 3 = 36 drops minimum, but we use 120
    // drops to leave comfortable margin.
    unsigned_payment.common_mut().fee = Amount::Xrp(XrpAmount::from_drops(120)?);

    println!("  Fee (adjusted for multi-sign): {:?}", unsigned_payment.common().fee);
    println!("  Sequence:                      {}", unsigned_payment.common().sequence);
    println!(
        "  LastLedgerSequence:            {:?}",
        unsigned_payment.common().last_ledger_sequence
    );

    // --- 6. Extract JSON map for multi-signing ---
    let tx_json = unsigned_payment.to_json_map()?;

    // --- 7. Multi-sign with signer1 and signer2 (meets quorum of 2) ---
    println!("\nMulti-signing with signer1 and signer2...");

    let sig1 = multi_sign(&tx_json, &signer1)?;
    println!("  Signer 1: {}", sig1.account);
    println!("    PubKey:    {}...{}", &sig1.signing_pub_key[..16], &sig1.signing_pub_key[sig1.signing_pub_key.len() - 8..]);
    println!("    Signature: {}...", &sig1.txn_signature[..32]);

    let sig2 = multi_sign(&tx_json, &signer2)?;
    println!("  Signer 2: {}", sig2.account);
    println!("    PubKey:    {}...{}", &sig2.signing_pub_key[..16], &sig2.signing_pub_key[sig2.signing_pub_key.len() - 8..]);
    println!("    Signature: {}...", &sig2.txn_signature[..32]);

    // --- 8. Combine signatures ---
    println!("\nCombining signatures...");
    let signed = combine_signatures(&tx_json, vec![sig1, sig2])?;

    println!("  Transaction hash: {}", signed.hash);
    println!("  Blob size:        {} bytes", signed.tx_blob.len() / 2);

    // --- 9. Submit via SubmitRequest ---
    println!("\nSubmitting multi-signed transaction...");

    let submit_resp = client
        .request(SubmitRequest {
            tx_blob: signed.tx_blob.clone(),
            fail_hard: None,
        })
        .await?;

    println!("  Engine result: {}", submit_resp.engine_result);
    println!("  Message:       {}", submit_resp.engine_result_message);

    if submit_resp.engine_result.starts_with("tem") || submit_resp.engine_result.starts_with("tef")
    {
        return Err(format!(
            "Transaction rejected: {} - {}",
            submit_resp.engine_result, submit_resp.engine_result_message
        )
        .into());
    }

    // --- 10. Poll for validation via TxRequest ---
    println!("\nWaiting for validation...");

    let mut validated = false;
    for attempt in 1..=30 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        match client
            .request(TxRequest {
                transaction: signed.hash.clone(),
                binary: None,
                min_ledger: None,
                max_ledger: None,
            })
            .await
        {
            Ok(resp) if resp.validated == Some(true) => {
                let result_code = resp
                    .meta
                    .as_ref()
                    .and_then(|m| m["TransactionResult"].as_str())
                    .unwrap_or("unknown");

                let ledger_index = resp.ledger_index.unwrap_or(0);

                println!("  Validated in ledger {ledger_index} (attempt {attempt})");
                println!("  TransactionResult: {result_code}");
                validated = true;
                break;
            }
            Ok(_) => {
                // Not validated yet, keep polling
                if attempt % 5 == 0 {
                    println!("  Still waiting... (attempt {attempt}/30)");
                }
            }
            Err(e) => {
                // txnNotFound is expected while the tx is pending
                let err_str = format!("{e}");
                if !err_str.contains("txnNotFound") {
                    return Err(format!("Error polling transaction: {e}").into());
                }
            }
        }
    }

    if !validated {
        return Err("Transaction was not validated within 30 attempts.".into());
    }

    // --- 11. Check final balances ---
    println!("\nBalances after multi-signed payment:");
    get_balance(&client, "Main wallet", *main_wallet.account_id()).await?;
    get_balance(&client, "Destination", *destination.account_id()).await?;

    println!("\nDone.");
    Ok(())
}

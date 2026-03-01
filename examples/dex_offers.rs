//! Example: DEX Offer Operations
//!
//! Demonstrates the XRPL's built-in decentralized exchange: create a trust line,
//! issue tokens, place an offer on the order book, query it, and cancel it.
//!
//! Run: `cargo run -p xrpl-mithril --example dex_offers`
//! Requires: Network access to XRPL testnet

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::account::AccountOffersRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::models::transactions::offer::{OfferCancel, OfferCreate};
use xrpl_mithril::models::transactions::payment::Payment;
use xrpl_mithril::models::transactions::trust_set::TrustSet;
use xrpl_mithril::models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::submit::submit_and_wait;
use xrpl_mithril::tx::{sign_transaction, TransactionResult};
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
    let trader = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Issuer: {}", issuer.classic_address());
    println!("  Trader: {}", trader.classic_address());

    // --- 2. Fund both wallets via testnet faucet ---
    println!("\nFunding wallets...");
    let http = reqwest::Client::new();
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &trader).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 3. Trader creates a trust line for "FOO" currency from issuer ---
    println!("\nTrader creating trust line for FOO (limit 10000)...");

    let trust_set = Transaction::TrustSet {
        common: make_common(*trader.account_id()),
        fields: TrustSet {
            limit_amount: IssuedAmount {
                value: IssuedValue::from_decimal_string("10000")?,
                currency: CurrencyCode::from_ascii("FOO")?,
                issuer: *issuer.account_id(),
            },
            quality_in: None,
            quality_out: None,
        },
    };

    let mut unsigned_trust = UnsignedTransaction::new(trust_set);
    autofill(&client, &mut unsigned_trust).await?;

    println!("  Fee:      {:?}", unsigned_trust.common().fee);
    println!("  Sequence: {}", unsigned_trust.common().sequence);

    let signed_trust = sign_transaction(&unsigned_trust, &trader)?;
    let trust_result: TransactionResult = submit_and_wait(&client, &signed_trust).await?;

    println!("\n  TrustSet validated!");
    println!("  Hash:   {}", trust_result.hash);
    println!("  Result: {}", trust_result.result_code);
    println!("  Ledger: {}", trust_result.ledger_index);

    // --- 4. Issuer sends 1000 FOO to trader ---
    println!("\nIssuer sending 1000 FOO to trader...");

    let payment = Transaction::Payment {
        common: make_common(*issuer.account_id()),
        fields: Payment {
            destination: *trader.account_id(),
            amount: Amount::Issued(IssuedAmount {
                value: IssuedValue::from_decimal_string("1000")?,
                currency: CurrencyCode::from_ascii("FOO")?,
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

    // --- 5. Trader creates an offer: sell 100 FOO for 50 XRP ---
    println!("\nTrader creating offer: sell 100 FOO for 50 XRP...");

    let offer_create = Transaction::OfferCreate {
        common: make_common(*trader.account_id()),
        fields: OfferCreate {
            taker_pays: Amount::Issued(IssuedAmount {
                value: IssuedValue::from_decimal_string("100")?,
                currency: CurrencyCode::from_ascii("FOO")?,
                issuer: *issuer.account_id(),
            }),
            taker_gets: Amount::Xrp(XrpAmount::from_drops(50_000_000)?),
            expiration: None,
            offer_sequence: None,
        },
    };

    let mut unsigned_offer = UnsignedTransaction::new(offer_create);
    autofill(&client, &mut unsigned_offer).await?;

    // Remember the sequence number so we can cancel the offer later
    let create_sequence = unsigned_offer.common().sequence;
    println!("  Fee:      {:?}", unsigned_offer.common().fee);
    println!("  Sequence: {create_sequence}");

    let signed_offer = sign_transaction(&unsigned_offer, &trader)?;
    let offer_result: TransactionResult = submit_and_wait(&client, &signed_offer).await?;

    println!("\n  OfferCreate validated!");
    println!("  Hash:   {}", offer_result.hash);
    println!("  Result: {}", offer_result.result_code);
    println!("  Ledger: {}", offer_result.ledger_index);

    // --- 6. Query trader's open offers ---
    println!("\nQuerying trader's open offers...");

    let offers_response = client
        .request(AccountOffersRequest {
            account: *trader.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    println!("  Found {} offer(s):", offers_response.offers.len());
    for offer in &offers_response.offers {
        println!("    Seq:        {}", offer.seq);
        println!("    TakerPays:  {:?}", offer.taker_pays);
        println!("    TakerGets:  {:?}", offer.taker_gets);
    }

    // --- 7. Cancel the offer ---
    println!("\nCancelling offer (sequence {create_sequence})...");

    let offer_cancel = Transaction::OfferCancel {
        common: make_common(*trader.account_id()),
        fields: OfferCancel {
            offer_sequence: create_sequence,
        },
    };

    let mut unsigned_cancel = UnsignedTransaction::new(offer_cancel);
    autofill(&client, &mut unsigned_cancel).await?;

    println!("  Fee:      {:?}", unsigned_cancel.common().fee);
    println!("  Sequence: {}", unsigned_cancel.common().sequence);

    let signed_cancel = sign_transaction(&unsigned_cancel, &trader)?;
    let cancel_result: TransactionResult = submit_and_wait(&client, &signed_cancel).await?;

    println!("\n  OfferCancel validated!");
    println!("  Hash:   {}", cancel_result.hash);
    println!("  Result: {}", cancel_result.result_code);
    println!("  Ledger: {}", cancel_result.ledger_index);

    // --- 8. Query again to confirm offer is gone ---
    println!("\nQuerying trader's offers after cancellation...");

    let offers_after = client
        .request(AccountOffersRequest {
            account: *trader.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    println!("  Found {} offer(s).", offers_after.offers.len());

    println!("\nDone.");
    Ok(())
}

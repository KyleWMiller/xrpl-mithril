//! Example: AMM Lifecycle (XLS-30)
//!
//! Demonstrates the Automated Market Maker workflow: set up a token, create an
//! AMM pool with XRP + issued currency, deposit additional liquidity, query pool
//! state, and withdraw all liquidity.
//!
//! Run: `cargo run -p xrpl-mithril --example amm_lifecycle`
//! Requires: Network access to XRPL testnet with AMM amendment active

use xrpl_mithril::client::{Client, JsonRpcClient};
use xrpl_mithril::models::requests::amm::AmmInfoRequest;
use xrpl_mithril::models::requests::server::ServerInfoRequest;
use xrpl_mithril::models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::models::transactions::account::AccountSet;
use xrpl_mithril::models::transactions::amm::{AMMCreate, AMMDeposit, AMMWithdraw};
use xrpl_mithril::models::transactions::payment::Payment;
use xrpl_mithril::models::transactions::trust_set::TrustSet;
use xrpl_mithril::models::transactions::wrapper::UnsignedTransaction;
use xrpl_mithril::models::transactions::{Transaction, TransactionCommon};
use xrpl_mithril::tx::autofill::autofill;
use xrpl_mithril::tx::submit::submit_and_wait;
use xrpl_mithril::tx::{sign_transaction, TransactionResult};
use xrpl_mithril::types::amount::{IssuedAmount, IssuedValue};
use xrpl_mithril::types::currency::{CurrencyCode, Issue};
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
    let lp_provider = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Issuer:      {}", issuer.classic_address());
    println!("  LP Provider: {}", lp_provider.classic_address());

    // --- 2. Fund both wallets via testnet faucet ---
    println!("\nFunding wallets...");
    let http = reqwest::Client::new();
    fund_wallet(&http, &issuer).await?;
    fund_wallet(&http, &lp_provider).await?;

    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 3. Issuer enables Default Ripple (required for AMM with issued tokens) ---
    // Without this flag, trust lines don't permit rippling and AMMCreate fails
    // with terNO_RIPPLE. Must be set BEFORE trust lines are created.
    println!("\nIssuer enabling Default Ripple (asfDefaultRipple = 8)...");

    let account_set = Transaction::AccountSet {
        common: make_common(*issuer.account_id()),
        fields: AccountSet {
            set_flag: Some(8), // asfDefaultRipple
            clear_flag: None,
            domain: None,
            email_hash: None,
            message_key: None,
            transfer_rate: None,
            tick_size: None,
            nftoken_minter: None,
        },
    };

    let mut unsigned_acctset = UnsignedTransaction::new(account_set);
    autofill(&client, &mut unsigned_acctset).await?;

    println!("  Fee:      {:?}", unsigned_acctset.common().fee);
    println!("  Sequence: {}", unsigned_acctset.common().sequence);

    let signed_acctset = sign_transaction(&unsigned_acctset, &issuer)?;
    let acctset_result: TransactionResult = submit_and_wait(&client, &signed_acctset).await?;

    println!("\n  AccountSet validated!");
    println!("  Hash:   {}", acctset_result.hash);
    println!("  Result: {}", acctset_result.result_code);
    println!("  Ledger: {}", acctset_result.ledger_index);

    // --- 4. LP provider creates a trust line for "AMM" currency from issuer ---
    println!("\nLP provider creating trust line for AMM (limit 10000)...");

    let trust_set = Transaction::TrustSet {
        common: make_common(*lp_provider.account_id()),
        fields: TrustSet {
            limit_amount: IssuedAmount {
                value: IssuedValue::from_decimal_string("10000")?,
                currency: CurrencyCode::from_ascii("AMM")?,
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

    let signed_trust = sign_transaction(&unsigned_trust, &lp_provider)?;
    let trust_result: TransactionResult = submit_and_wait(&client, &signed_trust).await?;

    println!("\n  TrustSet validated!");
    println!("  Hash:   {}", trust_result.hash);
    println!("  Result: {}", trust_result.result_code);
    println!("  Ledger: {}", trust_result.ledger_index);

    // --- 5. Issuer sends 500 AMM tokens to LP provider ---
    println!("\nIssuer sending 500 AMM tokens to LP provider...");

    let payment = Transaction::Payment {
        common: make_common(*issuer.account_id()),
        fields: Payment {
            destination: *lp_provider.account_id(),
            amount: Amount::Issued(IssuedAmount {
                value: IssuedValue::from_decimal_string("500")?,
                currency: CurrencyCode::from_ascii("AMM")?,
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

    // --- 6. LP provider creates AMM pool (25 XRP + 250 AMM tokens, 0.5% fee) ---
    // AMMCreate requires a special fee equal to one owner reserve increment (~2 XRP).
    // Fetch the reserve increment from server_info.
    println!("\nLP provider creating AMM pool: 25 XRP + 250 AMM tokens (0.5% fee)...");

    let server_info = client.request(ServerInfoRequest {}).await?;
    let reserve_inc_xrp = server_info
        .info
        .validated_ledger
        .as_ref()
        .and_then(|l| l.reserve_inc_xrp)
        .unwrap_or(2.0);
    let reserve_inc_drops = (reserve_inc_xrp * 1_000_000.0) as u64;
    println!("  Owner reserve increment: {} drops ({} XRP)", reserve_inc_drops, reserve_inc_xrp);

    let amm_create = Transaction::AMMCreate {
        common: make_common(*lp_provider.account_id()),
        fields: AMMCreate {
            amount: Amount::Xrp(XrpAmount::from_drops(25_000_000)?),
            amount2: Amount::Issued(IssuedAmount {
                value: IssuedValue::from_decimal_string("250")?,
                currency: CurrencyCode::from_ascii("AMM")?,
                issuer: *issuer.account_id(),
            }),
            trading_fee: 500,
        },
    };

    let mut unsigned_create = UnsignedTransaction::new(amm_create);
    autofill(&client, &mut unsigned_create).await?;

    // Override fee — AMMCreate requires one owner reserve increment, not the normal base fee.
    unsigned_create.common_mut().fee = Amount::Xrp(XrpAmount::from_drops(reserve_inc_drops)?);

    println!("  Fee:      {:?}", unsigned_create.common().fee);
    println!("  Sequence: {}", unsigned_create.common().sequence);

    let signed_create = sign_transaction(&unsigned_create, &lp_provider)?;
    let create_result: TransactionResult = submit_and_wait(&client, &signed_create).await?;

    println!("\n  AMMCreate validated!");
    println!("  Hash:   {}", create_result.hash);
    println!("  Result: {}", create_result.result_code);
    println!("  Ledger: {}", create_result.ledger_index);

    // --- 7. Query AMM info ---
    println!("\nQuerying AMM info...");

    let amm_currency = CurrencyCode::from_ascii("AMM")?;
    let amm_info = client
        .request(AmmInfoRequest {
            amm_account: None,
            asset: Some(Issue::Xrp),
            asset2: Some(Issue::Issued {
                currency: amm_currency,
                issuer: *issuer.account_id(),
            }),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
        })
        .await?;

    println!(
        "  AMM Account:  {}",
        amm_info.amm.account.as_deref().unwrap_or("unknown")
    );
    println!("  Asset 1:      {}", amm_info.amm.amount);
    println!("  Asset 2:      {}", amm_info.amm.amount2);
    println!(
        "  LP Token:     {}",
        amm_info
            .amm
            .lp_token
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!(
        "  Trading Fee:  {}",
        amm_info
            .amm
            .trading_fee
            .map(|f| format!("{} ({}%)", f, f as f64 / 1000.0))
            .unwrap_or_else(|| "unknown".to_string())
    );

    // --- 8. Deposit more liquidity (single-asset deposit of 10 XRP) ---
    println!("\nLP provider depositing 10 XRP (single-asset deposit)...");

    let mut deposit_common = make_common(*lp_provider.account_id());
    deposit_common.flags = Some(0x00080000); // tfSingleAsset

    let amm_deposit = Transaction::AMMDeposit {
        common: deposit_common,
        fields: AMMDeposit {
            asset: Issue::Xrp,
            asset2: Issue::Issued {
                currency: CurrencyCode::from_ascii("AMM")?,
                issuer: *issuer.account_id(),
            },
            amount: Some(Amount::Xrp(XrpAmount::from_drops(10_000_000)?)),
            amount2: None,
            e_price: None,
            lp_token: None,
        },
    };

    let mut unsigned_deposit = UnsignedTransaction::new(amm_deposit);
    autofill(&client, &mut unsigned_deposit).await?;

    println!("  Fee:      {:?}", unsigned_deposit.common().fee);
    println!("  Sequence: {}", unsigned_deposit.common().sequence);

    let signed_deposit = sign_transaction(&unsigned_deposit, &lp_provider)?;
    let deposit_result: TransactionResult = submit_and_wait(&client, &signed_deposit).await?;

    println!("\n  AMMDeposit validated!");
    println!("  Hash:   {}", deposit_result.hash);
    println!("  Result: {}", deposit_result.result_code);
    println!("  Ledger: {}", deposit_result.ledger_index);

    // --- 9. Withdraw all liquidity ---
    println!("\nLP provider withdrawing all liquidity...");

    let mut withdraw_common = make_common(*lp_provider.account_id());
    withdraw_common.flags = Some(0x00020000); // tfWithdrawAll

    let amm_withdraw = Transaction::AMMWithdraw {
        common: withdraw_common,
        fields: AMMWithdraw {
            asset: Issue::Xrp,
            asset2: Issue::Issued {
                currency: CurrencyCode::from_ascii("AMM")?,
                issuer: *issuer.account_id(),
            },
            amount: None,
            amount2: None,
            e_price: None,
            lp_token: None,
        },
    };

    let mut unsigned_withdraw = UnsignedTransaction::new(amm_withdraw);
    autofill(&client, &mut unsigned_withdraw).await?;

    println!("  Fee:      {:?}", unsigned_withdraw.common().fee);
    println!("  Sequence: {}", unsigned_withdraw.common().sequence);

    let signed_withdraw = sign_transaction(&unsigned_withdraw, &lp_provider)?;
    let withdraw_result: TransactionResult = submit_and_wait(&client, &signed_withdraw).await?;

    println!("\n  AMMWithdraw validated!");
    println!("  Hash:   {}", withdraw_result.hash);
    println!("  Result: {}", withdraw_result.result_code);
    println!("  Ledger: {}", withdraw_result.ledger_index);

    println!("\nDone.");
    Ok(())
}

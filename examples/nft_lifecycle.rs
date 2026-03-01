//! Example: NFT Lifecycle + Dynamic NFT (XLS-46)
//!
//! Demonstrates the full NFT workflow: mint a mutable NFT, create a sell offer,
//! buyer accepts the offer, issuer modifies the URI (DynamicNFT), and verify.
//!
//! Run: `cargo run -p xrpl-mithril --example nft_lifecycle`
//! Requires: Network access to XRPL testnet with DynamicNFT amendment active

use xrpl_mithril::xrpl_client::{Client, JsonRpcClient};
use xrpl_mithril::xrpl_models::requests::account::{AccountNftsRequest, AccountObjectsRequest};
use xrpl_mithril::xrpl_models::requests::{LedgerShortcut, LedgerSpecifier};
use xrpl_mithril::xrpl_models::transactions::nft::{
    NFTokenAcceptOffer, NFTokenCreateOffer, NFTokenMint, NFTokenModify,
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

/// NFT mint flags.
const TF_BURNABLE: u32 = 0x0000_0001;
const TF_TRANSFERABLE: u32 = 0x0000_0008;
const TF_MUTABLE: u32 = 0x0000_0010;

/// NFT sell offer flag.
const TF_SELL_NFTOKEN: u32 = 0x0000_0001;

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

/// Decode a hex-encoded URI string to a UTF-8 string.
fn decode_hex_uri(hex_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for chunk in hex_str.as_bytes().chunks(2) {
        if chunk.len() != 2 {
            return Err("odd-length hex string".into());
        }
        let high = hex_nibble(chunk[0]).ok_or("invalid hex character")?;
        let low = hex_nibble(chunk[1]).ok_or("invalid hex character")?;
        bytes.push((high << 4) | low);
    }
    Ok(String::from_utf8(bytes)?)
}

/// Convert a single ASCII hex character to its nibble value.
fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::new();
    let client = JsonRpcClient::new(TESTNET_RPC)?;

    // --- 1. Generate creator and buyer wallets ---
    println!("Generating wallets...");
    let creator = Wallet::generate(Algorithm::Ed25519)?;
    let buyer = Wallet::generate(Algorithm::Secp256k1)?;
    println!("  Creator: {}", creator.classic_address());
    println!("  Buyer:   {}", buyer.classic_address());

    // --- 2. Fund both wallets via testnet faucet ---
    println!("\nFunding wallets...");
    fund_wallet(&http, &creator).await?;
    fund_wallet(&http, &buyer).await?;
    println!("  Waiting for funding to validate...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- 3. Mint a mutable NFT ---
    // Flags: tfBurnable | tfTransferable | tfMutable = 0x00000019
    let mint_flags = TF_BURNABLE | TF_TRANSFERABLE | TF_MUTABLE;
    println!("\nMinting NFT...");
    println!("  Flags:        0x{mint_flags:08X} (burnable + transferable + mutable)");
    println!("  Taxon:        1");
    println!("  Transfer fee: 500 basis points (5%)");
    println!("  URI:          https://example.com/nft/1");

    let mut common = make_common(*creator.account_id());
    common.flags = Some(mint_flags);

    let mint_tx = Transaction::NFTokenMint {
        common,
        fields: NFTokenMint {
            nftoken_taxon: 1,
            issuer: None,
            transfer_fee: Some(500),
            uri: Some(Blob::new(b"https://example.com/nft/1".to_vec())),
        },
    };

    let mut unsigned_mint = UnsignedTransaction::new(mint_tx);
    autofill(&client, &mut unsigned_mint).await?;
    let signed_mint = sign_transaction(&unsigned_mint, &creator)?;
    let mint_result: TransactionResult = submit_and_wait(&client, &signed_mint).await?;

    println!("\n  NFT minted!");
    println!("  Hash:   {}", mint_result.hash);
    println!("  Result: {}", mint_result.result_code);
    println!("  Ledger: {}", mint_result.ledger_index);

    // --- 4. Query creator's NFTs to get the NFTokenID ---
    println!("\nQuerying creator's NFTs...");
    let nfts = client
        .request(AccountNftsRequest {
            account: *creator.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if nfts.account_nfts.is_empty() {
        return Err("No NFTs found after minting".into());
    }

    let nft = &nfts.account_nfts[0];
    let nftoken_id = nft.nftoken_id;
    println!("  NFTokenID:    {nftoken_id}");
    println!("  Issuer:       {}", nft.issuer);
    println!("  Taxon:        {}", nft.nftoken_taxon);
    println!("  Flags:        0x{:08X}", nft.flags);
    println!("  Transfer fee: {:?}", nft.transfer_fee);
    if let Some(ref uri_hex) = nft.uri {
        println!("  URI:          {}", decode_hex_uri(uri_hex)?);
    }

    // --- 5. Create a sell offer for 5 XRP ---
    println!("\nCreating sell offer for 5 XRP...");
    let mut offer_common = make_common(*creator.account_id());
    offer_common.flags = Some(TF_SELL_NFTOKEN);

    let create_offer_tx = Transaction::NFTokenCreateOffer {
        common: offer_common,
        fields: NFTokenCreateOffer {
            nftoken_id,
            amount: Amount::Xrp(XrpAmount::from_drops(5_000_000)?),
            owner: None,
            destination: None,
            expiration: None,
        },
    };

    let mut unsigned_offer = UnsignedTransaction::new(create_offer_tx);
    autofill(&client, &mut unsigned_offer).await?;
    let signed_offer = sign_transaction(&unsigned_offer, &creator)?;
    let offer_result: TransactionResult = submit_and_wait(&client, &signed_offer).await?;

    println!("\n  Sell offer created!");
    println!("  Hash:   {}", offer_result.hash);
    println!("  Result: {}", offer_result.result_code);
    println!("  Ledger: {}", offer_result.ledger_index);

    // --- 6. Get the sell offer ID from account_objects ---
    println!("\nQuerying creator's NFT offers...");
    let objects = client
        .request(AccountObjectsRequest {
            account: *creator.account_id(),
            object_type: Some("nft_offer".to_string()),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if objects.account_objects.is_empty() {
        return Err("No NFT offers found after creating sell offer".into());
    }

    let offer_id_str = objects.account_objects[0]["index"]
        .as_str()
        .ok_or("missing offer index")?;
    let offer_id = Hash256::from_hex(offer_id_str)?;
    println!("  Sell offer ID: {offer_id}");

    // --- 7. Buyer accepts the sell offer ---
    println!("\nBuyer accepting sell offer...");
    let accept_tx = Transaction::NFTokenAcceptOffer {
        common: make_common(*buyer.account_id()),
        fields: NFTokenAcceptOffer {
            nftoken_sell_offer: Some(offer_id),
            nftoken_buy_offer: None,
            nftoken_broker_fee: None,
        },
    };

    let mut unsigned_accept = UnsignedTransaction::new(accept_tx);
    autofill(&client, &mut unsigned_accept).await?;
    let signed_accept = sign_transaction(&unsigned_accept, &buyer)?;
    let accept_result: TransactionResult = submit_and_wait(&client, &signed_accept).await?;

    println!("\n  Offer accepted! NFT transferred to buyer.");
    println!("  Hash:   {}", accept_result.hash);
    println!("  Result: {}", accept_result.result_code);
    println!("  Ledger: {}", accept_result.ledger_index);

    // --- 8. Creator modifies the NFT URI (DynamicNFT) ---
    println!("\nCreator modifying NFT URI (DynamicNFT)...");
    println!("  New URI: https://example.com/nft/1/v2");

    let modify_tx = Transaction::NFTokenModify {
        common: make_common(*creator.account_id()),
        fields: NFTokenModify {
            nftoken_id,
            owner: Some(*buyer.account_id()),
            uri: Some(Blob::new(b"https://example.com/nft/1/v2".to_vec())),
        },
    };

    let mut unsigned_modify = UnsignedTransaction::new(modify_tx);
    autofill(&client, &mut unsigned_modify).await?;
    let signed_modify = sign_transaction(&unsigned_modify, &creator)?;
    let modify_result: TransactionResult = submit_and_wait(&client, &signed_modify).await?;

    println!("\n  NFT URI modified!");
    println!("  Hash:   {}", modify_result.hash);
    println!("  Result: {}", modify_result.result_code);
    println!("  Ledger: {}", modify_result.ledger_index);

    // --- 9. Verify buyer owns the NFT with updated URI ---
    println!("\nVerifying buyer's NFTs...");
    let buyer_nfts = client
        .request(AccountNftsRequest {
            account: *buyer.account_id(),
            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Validated)),
            limit: None,
            marker: None,
        })
        .await?;

    if buyer_nfts.account_nfts.is_empty() {
        return Err("Buyer has no NFTs after accepting offer".into());
    }

    let buyer_nft = &buyer_nfts.account_nfts[0];
    println!("  Buyer owns NFT: {}", buyer_nft.nftoken_id);
    println!("  Issuer:         {}", buyer_nft.issuer);
    println!("  Flags:          0x{:08X}", buyer_nft.flags);
    if let Some(ref uri_hex) = buyer_nft.uri {
        println!("  URI (updated):  {}", decode_hex_uri(uri_hex)?);
    }

    println!("\nDone.");
    Ok(())
}

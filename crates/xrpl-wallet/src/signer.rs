//! Transaction signing for single-sign and multi-sign.
//!
//! This module provides functions to sign XRPL transactions using a [`Wallet`].
//! It integrates with [`xrpl_codec::signing`] for hash computation and
//! serialization.
//!
//! # Signing Flow
//!
//! 1. Compute the signing data/hash from the transaction JSON
//! 2. Sign with the wallet's private key
//! 3. Attach `SigningPubKey` and `TxnSignature` to the transaction
//! 4. Compute the transaction ID
//!
//! # Algorithm Differences
//!
//! - **secp256k1**: Signs the SHA-512/Half hash (32 bytes) of the signing data.
//! - **Ed25519**: Signs the raw signing data bytes (Ed25519 handles its own hashing internally).

use serde_json::{Map, Value};

use xrpl_codec::signing;

use crate::algorithm::Algorithm;
use crate::error::WalletError;
use crate::keypair::Wallet;

/// A fully signed transaction ready for submission.
#[derive(Debug, Clone)]
pub struct SignedTransaction {
    /// The transaction JSON with `SigningPubKey` and `TxnSignature` attached.
    pub tx_json: Map<String, Value>,
    /// Hex-encoded binary serialization of the signed transaction.
    pub tx_blob: String,
    /// The transaction ID (uppercase hex, 64 characters).
    pub hash: String,
}

/// A single signer's contribution to a multi-signed transaction.
#[derive(Debug, Clone)]
pub struct Signer {
    /// The signer's classic address.
    pub account: String,
    /// The signer's public key (hex).
    pub signing_pub_key: String,
    /// The signer's signature (hex).
    pub txn_signature: String,
}

/// Sign a transaction with a single wallet.
///
/// Takes a transaction as a JSON object (must contain at least `TransactionType`,
/// `Account`, `Fee`, and `Sequence`), signs it, and returns a [`SignedTransaction`]
/// with the signature fields attached and the transaction ID computed.
///
/// # Errors
///
/// Returns [`WalletError`] if signing or serialization fails.
pub fn sign(
    tx: &Map<String, Value>,
    wallet: &Wallet,
) -> Result<SignedTransaction, WalletError> {
    let signature = compute_signature(tx, wallet)?;

    // Build the signed transaction JSON
    let mut signed_tx = tx.clone();
    signed_tx.insert(
        "SigningPubKey".into(),
        Value::String(wallet.public_key_hex()),
    );
    signed_tx.insert(
        "TxnSignature".into(),
        Value::String(hex::encode_upper(&signature)),
    );

    // Compute the binary blob
    let mut blob_buf = Vec::new();
    xrpl_codec::serializer::serialize_json_object(&signed_tx, &mut blob_buf, false)?;
    let tx_blob = hex::encode_upper(&blob_buf);

    // Compute the transaction ID
    let hash = signing::transaction_id_hex(&signed_tx)?;

    Ok(SignedTransaction {
        tx_json: signed_tx,
        tx_blob,
        hash,
    })
}

/// Produce a single signer's contribution for multi-signing.
///
/// In multi-signing, each signer signs a variant of the transaction that
/// includes their account ID in the hash. The outer transaction's
/// `SigningPubKey` is set to an empty string.
///
/// # Errors
///
/// Returns [`WalletError`] if signing fails.
pub fn multi_sign(
    tx: &Map<String, Value>,
    wallet: &Wallet,
) -> Result<Signer, WalletError> {
    let account_id = wallet.account_id();
    let signature = compute_multi_signature(tx, wallet, account_id.as_bytes())?;

    Ok(Signer {
        account: wallet.classic_address().to_string(),
        signing_pub_key: wallet.public_key_hex(),
        txn_signature: hex::encode_upper(&signature),
    })
}

/// Combine multiple [`Signer`]s into a multi-signed transaction.
///
/// The signers are sorted by account address (lexicographic) as required by
/// the XRPL protocol. The outer `SigningPubKey` is set to an empty string.
///
/// # Errors
///
/// Returns [`WalletError`] if serialization or hash computation fails.
pub fn combine_signatures(
    tx: &Map<String, Value>,
    mut signers: Vec<Signer>,
) -> Result<SignedTransaction, WalletError> {
    // Sort signers by account (XRPL requires canonical ordering)
    signers.sort_by(|a, b| a.account.cmp(&b.account));

    // Build the Signers array
    let signers_array: Vec<Value> = signers
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "Signer": {
                    "Account": s.account,
                    "SigningPubKey": s.signing_pub_key,
                    "TxnSignature": s.txn_signature,
                }
            })
        })
        .collect();

    let mut signed_tx = tx.clone();
    // Multi-signed transactions have an empty SigningPubKey
    signed_tx.insert("SigningPubKey".into(), Value::String(String::new()));
    signed_tx.insert("Signers".into(), Value::Array(signers_array));

    // Compute the binary blob
    let mut blob_buf = Vec::new();
    xrpl_codec::serializer::serialize_json_object(&signed_tx, &mut blob_buf, false)?;
    let tx_blob = hex::encode_upper(&blob_buf);

    // Compute the transaction ID
    let hash = signing::transaction_id_hex(&signed_tx)?;

    Ok(SignedTransaction {
        tx_json: signed_tx,
        tx_blob,
        hash,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute the signature for single-signing.
fn compute_signature(
    tx: &Map<String, Value>,
    wallet: &Wallet,
) -> Result<Vec<u8>, WalletError> {
    let keypair = wallet.keypair();
    match keypair.algorithm() {
        Algorithm::Secp256k1 => {
            // secp256k1 signs the SHA-512/Half hash
            let hash = signing::signing_hash(tx)?;
            keypair.sign(&hash)
        }
        Algorithm::Ed25519 => {
            // Ed25519 signs the raw signing data (handles its own hashing)
            let data = signing::signing_data(tx)?;
            keypair.sign(&data)
        }
    }
}

/// Compute the signature for multi-signing.
fn compute_multi_signature(
    tx: &Map<String, Value>,
    wallet: &Wallet,
    signer_account_id: &[u8; 20],
) -> Result<Vec<u8>, WalletError> {
    let keypair = wallet.keypair();
    match keypair.algorithm() {
        Algorithm::Secp256k1 => {
            let hash = signing::multi_signing_hash(tx, signer_account_id)?;
            keypair.sign(&hash)
        }
        Algorithm::Ed25519 => {
            let data = signing::multi_signing_data(tx, signer_account_id)?;
            keypair.sign(&data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Algorithm;
    use crate::seed::Seed;

    fn genesis_wallet() -> Wallet {
        let seed = Seed::from_passphrase("masterpassphrase");
        Wallet::from_seed(&seed, Algorithm::Secp256k1).expect("genesis wallet")
    }

    fn sample_payment() -> Map<String, Value> {
        let tx = serde_json::json!({
            "TransactionType": "Payment",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000",
            "Fee": "12",
            "Sequence": 1u32,
            "LastLedgerSequence": 100u32,
        });
        tx.as_object().cloned().expect("valid object")
    }

    #[test]
    fn sign_payment_secp256k1() {
        let wallet = genesis_wallet();
        let tx = sample_payment();
        let signed = sign(&tx, &wallet).expect("sign");

        // Verify required fields are present
        assert!(signed.tx_json.contains_key("SigningPubKey"));
        assert!(signed.tx_json.contains_key("TxnSignature"));
        assert!(!signed.tx_blob.is_empty());
        assert_eq!(signed.hash.len(), 64);
        assert!(signed.hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn sign_payment_ed25519() {
        let seed = Seed::from_passphrase("ed25519-test-seed");
        let wallet = Wallet::from_seed(&seed, Algorithm::Ed25519).expect("wallet");
        let mut tx = sample_payment();
        // Update the Account to match this wallet
        tx.insert(
            "Account".into(),
            Value::String(wallet.classic_address().to_string()),
        );

        let signed = sign(&tx, &wallet).expect("sign");
        assert!(signed.tx_json.contains_key("SigningPubKey"));
        assert!(signed.tx_json.contains_key("TxnSignature"));

        // Ed25519 signatures are always 128 hex chars (64 bytes)
        let sig_hex = signed.tx_json["TxnSignature"].as_str().expect("string");
        assert_eq!(sig_hex.len(), 128);
    }

    #[test]
    fn sign_deterministic() {
        let wallet = genesis_wallet();
        let tx = sample_payment();
        let signed1 = sign(&tx, &wallet).expect("sign 1");
        let signed2 = sign(&tx, &wallet).expect("sign 2");

        // Same wallet + same tx → same signature (ECDSA with RFC 6979 is deterministic)
        assert_eq!(signed1.hash, signed2.hash);
        assert_eq!(signed1.tx_blob, signed2.tx_blob);
    }

    #[test]
    fn multi_sign_produces_signer() {
        let wallet = genesis_wallet();
        let tx = sample_payment();
        let signer = multi_sign(&tx, &wallet).expect("multi_sign");

        assert_eq!(signer.account, "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
        assert!(!signer.signing_pub_key.is_empty());
        assert!(!signer.txn_signature.is_empty());
    }

    #[test]
    fn multi_sign_different_signers_different_signatures() {
        let wallet1 = genesis_wallet();
        let seed2 = Seed::from_passphrase("second-signer");
        let wallet2 = Wallet::from_seed(&seed2, Algorithm::Secp256k1).expect("wallet2");

        let tx = sample_payment();
        let sig1 = multi_sign(&tx, &wallet1).expect("sig1");
        let sig2 = multi_sign(&tx, &wallet2).expect("sig2");

        assert_ne!(sig1.txn_signature, sig2.txn_signature);
    }

    #[test]
    fn combine_signatures_produces_signed_tx() {
        let wallet1 = genesis_wallet();
        let seed2 = Seed::from_passphrase("second-signer");
        let wallet2 = Wallet::from_seed(&seed2, Algorithm::Secp256k1).expect("wallet2");

        let tx = sample_payment();
        let sig1 = multi_sign(&tx, &wallet1).expect("sig1");
        let sig2 = multi_sign(&tx, &wallet2).expect("sig2");

        let combined = combine_signatures(&tx, vec![sig1, sig2]).expect("combine");

        // Verify structure
        assert!(combined.tx_json.contains_key("Signers"));
        let signers = combined.tx_json["Signers"].as_array().expect("array");
        assert_eq!(signers.len(), 2);

        // SigningPubKey should be empty for multi-signed tx
        assert_eq!(
            combined.tx_json["SigningPubKey"].as_str().expect("str"),
            ""
        );

        // Signers should be sorted by account
        let first_account = signers[0]["Signer"]["Account"].as_str().expect("str");
        let second_account = signers[1]["Signer"]["Account"].as_str().expect("str");
        assert!(first_account <= second_account);
    }
}

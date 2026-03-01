//! Transaction submission and validation waiting.
//!
//! Provides [`submit_and_wait`] which submits a signed transaction and
//! polls until it is validated or the `LastLedgerSequence` is passed.
//!
//! # Examples
//!
//! ```no_run
//! use xrpl_mithril_tx::autofill::autofill;
//! use xrpl_mithril_tx::builder::PaymentBuilder;
//! use xrpl_mithril_tx::reliable::sign_transaction;
//! use xrpl_mithril_tx::submit::submit_and_wait;
//! use xrpl_mithril_client::JsonRpcClient;
//! use xrpl_mithril_wallet::{Wallet, Algorithm};
//! use xrpl_mithril_types::{Amount, XrpAmount};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
//! let wallet = Wallet::from_seed_encoded("sEdT7wHTCLzDG7Ue4312Kp4QA389Xmb")?;
//!
//! let mut unsigned = PaymentBuilder::new()
//!     .account(*wallet.account_id())
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
//!     .build()?;
//!
//! autofill(&client, &mut unsigned).await?;
//! let signed = sign_transaction(&unsigned, &wallet)?;
//! let result = submit_and_wait(&client, &signed).await?;
//! println!("Hash: {}, Result: {}", result.hash, result.result_code);
//! # Ok(())
//! # }
//! ```

use xrpl_mithril_client::Client;
use xrpl_mithril_models::requests::{
    server::ServerInfoRequest,
    transaction::{SubmitRequest, TxRequest},
};
use xrpl_mithril_models::transactions::wrapper::{Signable, TypedSignedTransaction};

use crate::error::TxError;

/// The result of a successfully submitted and validated transaction.
///
/// Returned by [`submit_and_wait`] after the transaction is included in a
/// validated ledger with a `tes` (success) result code.
///
/// # Examples
///
/// ```no_run
/// use xrpl_mithril_tx::submit::TransactionResult;
///
/// # fn show(result: TransactionResult) {
/// println!("TX {} validated in ledger {}", result.hash, result.ledger_index);
/// assert_eq!(result.result_code, "tesSUCCESS");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TransactionResult {
    /// The transaction hash.
    pub hash: String,
    /// The engine result code (e.g., `"tesSUCCESS"`, `"tecPATH_PARTIAL"`).
    pub result_code: String,
    /// The ledger index where the transaction was included.
    pub ledger_index: u32,
}

/// Submit a signed transaction and wait for it to be validated.
///
/// This function:
/// 1. Submits the signed `tx_blob` to the network
/// 2. Checks the preliminary result — fails fast on `tem`/`tef` codes
/// 3. Polls the `tx` method until the transaction is validated or
///    `LastLedgerSequence` is surpassed
///
/// # Errors
///
/// - [`TxError::TransactionFailed`] if the preliminary result is a permanent
///   failure (`tem`, `tef`) or the transaction is included but failed
/// - [`TxError::NotValidated`] if `LastLedgerSequence` passes without inclusion
/// - [`TxError::Client`] on network errors
pub async fn submit_and_wait<T: Signable>(
    client: &impl Client,
    signed: &TypedSignedTransaction<T>,
) -> Result<TransactionResult, TxError> {
    // 1. Submit
    let submit_resp = client
        .request(SubmitRequest {
            tx_blob: signed.tx_blob().to_string(),
            fail_hard: None,
        })
        .await?;

    tracing::info!(
        hash = signed.hash(),
        engine_result = %submit_resp.engine_result,
        "transaction submitted"
    );

    // 2. Check preliminary result
    let preliminary = &submit_resp.engine_result;
    if preliminary.starts_with("tem") || preliminary.starts_with("tef") {
        return Err(TxError::TransactionFailed {
            result_code: preliminary.clone(),
            result_message: submit_resp.engine_result_message.clone(),
        });
    }

    // 3. Poll for validation
    let last_ledger = signed
        .inner()
        .common()
        .last_ledger_sequence
        .unwrap_or(u32::MAX);

    loop {
        // Small delay before polling
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Check if the transaction is validated
        match client
            .request(TxRequest {
                transaction: signed.hash().to_string(),
                binary: Some(false),
                min_ledger: None,
                max_ledger: None,
            })
            .await
        {
            Ok(tx_resp) if tx_resp.validated == Some(true) => {
                let result_code = tx_resp
                    .meta
                    .as_ref()
                    .and_then(|m| m.get("TransactionResult"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let ledger_index = tx_resp.ledger_index.unwrap_or(0);

                tracing::info!(
                    hash = signed.hash(),
                    result = %result_code,
                    ledger = ledger_index,
                    "transaction validated"
                );

                if result_code.starts_with("tes") {
                    return Ok(TransactionResult {
                        hash: signed.hash().to_string(),
                        result_code,
                        ledger_index,
                    });
                } else {
                    return Err(TxError::TransactionFailed {
                        result_code,
                        result_message: String::new(),
                    });
                }
            }
            Ok(_) => {
                // Not validated yet — continue polling
            }
            Err(xrpl_mithril_client::ClientError::RpcError {
                error: Some(ref e), ..
            }) if e == "txnNotFound" => {
                // Transaction not found yet — check if we've passed LastLedgerSequence
            }
            Err(e) => return Err(TxError::Client(e)),
        }

        // Check current ledger vs LastLedgerSequence
        let server = client
            .request(ServerInfoRequest {})
            .await
            .map_err(|e| TxError::AutofillFailed(format!("server_info: {e}")))?;
        let current = server
            .info
            .validated_ledger
            .map(|l| l.seq)
            .unwrap_or(0);

        if current > last_ledger {
            return Err(TxError::NotValidated {
                last_ledger_sequence: last_ledger,
            });
        }

        tracing::debug!(
            hash = signed.hash(),
            current_ledger = current,
            last_ledger = last_ledger,
            "waiting for validation"
        );
    }
}

//! High-level convenience functions for the complete transaction lifecycle.
//!
//! [`submit_transaction`] combines autofill, signing, and reliable submission
//! into a single call.

use xrpl_client::Client;
use xrpl_models::transactions::wrapper::{
    Signable, TypedSignedTransaction, UnsignedTransaction,
};
use xrpl_models::transactions::Transaction;
use xrpl_wallet::Wallet;

use crate::autofill::autofill;
use crate::error::TxError;
use crate::submit::{submit_and_wait, TransactionResult};

/// Sign an [`UnsignedTransaction`], producing a [`TypedSignedTransaction`].
///
/// Bridges between the typed wrapper world and the existing JSON-map-based
/// signer in `xrpl-wallet`. The existing `xrpl_wallet::sign()` function is
/// called internally.
///
/// # Examples
///
/// ```no_run
/// use xrpl_tx::builder::PaymentBuilder;
/// use xrpl_tx::reliable::sign_transaction;
/// use xrpl_wallet::{Wallet, Algorithm};
/// use xrpl_types::{Amount, XrpAmount};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let wallet = Wallet::from_seed_encoded("sEdT7wHTCLzDG7Ue4312Kp4QA389Xmb")?;
///
/// let unsigned = PaymentBuilder::new()
///     .account(*wallet.account_id())
///     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
///     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
///     .sequence(1)
///     .build()?;
///
/// let signed = sign_transaction(&unsigned, &wallet)?;
/// println!("TX hash: {}", signed.hash());
/// println!("TX blob: {}", signed.tx_blob());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`TxError`] if JSON serialization or signing fails.
pub fn sign_transaction<T: Signable>(
    tx: &UnsignedTransaction<T>,
    wallet: &Wallet,
) -> Result<TypedSignedTransaction<T>, TxError> {
    let map = tx.to_json_map()?;
    let signed = xrpl_wallet::sign(&map, wallet)?;
    Ok(TypedSignedTransaction::new(
        tx.inner().clone(),
        signed.tx_json,
        signed.tx_blob,
        signed.hash,
    ))
}

/// Autofill, sign, and submit a transaction in one call.
///
/// This is the highest-level convenience function. It:
/// 1. Autofills missing Fee, Sequence, and LastLedgerSequence
/// 2. Signs with the provided wallet
/// 3. Submits and waits for validation
///
/// # Examples
///
/// ```no_run
/// use xrpl_tx::builder::PaymentBuilder;
/// use xrpl_tx::reliable::submit_transaction;
/// use xrpl_client::JsonRpcClient;
/// use xrpl_wallet::{Wallet, Algorithm};
/// use xrpl_types::{Amount, XrpAmount};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
/// let wallet = Wallet::from_seed_encoded("sEdT7wHTCLzDG7Ue4312Kp4QA389Xmb")?;
///
/// let unsigned = PaymentBuilder::new()
///     .account(*wallet.account_id())
///     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
///     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
///     .build()?;
///
/// let result = submit_transaction(&client, unsigned, &wallet).await?;
/// println!("Validated in ledger {}: {}", result.ledger_index, result.result_code);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`TxError`] on any failure in the pipeline (network, signing,
/// validation timeout, transaction rejection).
pub async fn submit_transaction(
    client: &impl Client,
    mut tx: UnsignedTransaction<Transaction>,
    wallet: &Wallet,
) -> Result<TransactionResult, TxError> {
    autofill(client, &mut tx).await?;
    let signed = sign_transaction(&tx, wallet)?;
    submit_and_wait(client, &signed).await
}

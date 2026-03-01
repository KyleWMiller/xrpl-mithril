//! Transaction autofill -- populates Fee, Sequence, and LastLedgerSequence
//! by querying the network.
//!
//! # Examples
//!
//! ```no_run
//! use xrpl_tx::autofill::autofill;
//! use xrpl_tx::builder::PaymentBuilder;
//! use xrpl_client::JsonRpcClient;
//! use xrpl_types::{Amount, XrpAmount};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = JsonRpcClient::new("https://s.altnet.rippletest.net:51234")?;
//!
//! let mut unsigned = PaymentBuilder::new()
//!     .account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?)
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse()?)
//!     .amount(Amount::Xrp(XrpAmount::from_drops(1_000_000)?))
//!     .build()?;
//!
//! autofill(&client, &mut unsigned).await?;
//! // Fee, Sequence, and LastLedgerSequence are now populated
//! # Ok(())
//! # }
//! ```

use xrpl_client::Client;
use xrpl_models::requests::{
    account::AccountInfoRequest,
    server::{FeeRequest, ServerInfoRequest},
    LedgerShortcut, LedgerSpecifier,
};
use xrpl_models::transactions::wrapper::{Signable, UnsignedTransaction};
use xrpl_types::{Amount, XrpAmount};

use crate::error::TxError;

/// Options controlling autofill behavior.
#[derive(Debug, Clone)]
pub struct AutofillOptions {
    /// Number of ledgers ahead of the current validated ledger to set
    /// `LastLedgerSequence`. Default: 20 (~60-80 seconds at ~3-4s/ledger).
    pub ledger_offset: u32,
}

impl Default for AutofillOptions {
    fn default() -> Self {
        Self { ledger_offset: 20 }
    }
}

/// Autofill missing common fields on an unsigned transaction.
///
/// Populates these fields if they are not already set:
/// - **Fee** — from the `fee` RPC method (open ledger fee)
/// - **Sequence** — from the `account_info` method
/// - **LastLedgerSequence** — current validated ledger + 20
///
/// Uses parallel network fetches where possible.
///
/// # Errors
///
/// Returns [`TxError::AutofillFailed`] if any network query fails.
pub async fn autofill<T: Signable>(
    client: &impl Client,
    tx: &mut UnsignedTransaction<T>,
) -> Result<(), TxError> {
    autofill_with_options(client, tx, AutofillOptions::default()).await
}

/// Autofill with custom options.
///
/// # Errors
///
/// Returns [`TxError`] if network queries fail or returned data is invalid.
pub async fn autofill_with_options<T: Signable>(
    client: &impl Client,
    tx: &mut UnsignedTransaction<T>,
    options: AutofillOptions,
) -> Result<(), TxError> {
    let common = tx.common();

    // Determine what needs filling
    let needs_fee = matches!(common.fee, Amount::Xrp(ref xrp) if xrp.drops() == 0);
    let needs_sequence = common.sequence == 0 && common.ticket_sequence.is_none();
    let needs_last_ledger = common.last_ledger_sequence.is_none();

    let account = common.account;

    // Fetch fee and account_info in parallel
    let (fee_result, account_result) = tokio::join!(
        async {
            if needs_fee {
                Some(client.request(FeeRequest {}).await)
            } else {
                None
            }
        },
        async {
            if needs_sequence {
                Some(
                    client
                        .request(AccountInfoRequest {
                            account,
                            ledger_index: Some(LedgerSpecifier::Named(LedgerShortcut::Current)),
                            queue: Some(true),
                            signer_lists: None,
                        })
                        .await,
                )
            } else {
                None
            }
        }
    );

    // Apply fee
    if let Some(fee_resp) = fee_result {
        let fee_resp = fee_resp
            .map_err(|e| TxError::AutofillFailed(format!("fee request failed: {e}")))?;
        let drops: u64 = fee_resp
            .drops
            .open_ledger_fee
            .parse()
            .map_err(|e| TxError::AutofillFailed(format!("invalid fee value: {e}")))?;
        let xrp = XrpAmount::from_drops(drops)
            .map_err(|e| TxError::AutofillFailed(format!("fee out of range: {e}")))?;
        tx.common_mut().fee = Amount::Xrp(xrp);
    }

    // Apply sequence
    if let Some(acct_resp) = account_result {
        let acct_resp = acct_resp
            .map_err(|e| TxError::AutofillFailed(format!("account_info failed: {e}")))?;
        tx.common_mut().sequence = acct_resp.account_data.sequence;
    }

    // Apply LastLedgerSequence
    if needs_last_ledger {
        let server_resp = client
            .request(ServerInfoRequest {})
            .await
            .map_err(|e| TxError::AutofillFailed(format!("server_info failed: {e}")))?;

        let current_ledger = server_resp
            .info
            .validated_ledger
            .map(|l| l.seq)
            .unwrap_or(0);

        tx.common_mut().last_ledger_sequence = Some(current_ledger + options.ledger_offset);
    }

    Ok(())
}

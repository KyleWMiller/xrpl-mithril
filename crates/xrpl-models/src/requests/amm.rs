//! AMM-related request types.

use serde::Serialize;
use xrpl_types::{AccountId, Issue};

use super::{LedgerSpecifier, XrplRequest};
use crate::responses::amm::AmmInfoResponse;

/// Request information about an Automated Market Maker (AMM) instance.
#[derive(Debug, Clone, Serialize)]
pub struct AmmInfoRequest {
    /// The AMM's account address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amm_account: Option<AccountId>,
    /// One of the AMM's assets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<Issue>,
    /// The other AMM asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset2: Option<Issue>,
    /// Which ledger to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerSpecifier>,
}

impl XrplRequest for AmmInfoRequest {
    type Response = AmmInfoResponse;
    fn method(&self) -> &'static str {
        "amm_info"
    }
}

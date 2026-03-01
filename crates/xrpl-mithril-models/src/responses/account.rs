//! Account-related response types.

use serde::Deserialize;
use xrpl_mithril_types::{AccountId, Amount, Hash256};

use crate::requests::Marker;

/// Response from the `account_info` method.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::responses::account::AccountInfoResponse;
///
/// let json = serde_json::json!({
///     "account_data": {
///         "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///         "Balance": "10000000000",
///         "Sequence": 1,
///         "Flags": 0,
///         "OwnerCount": 0
///     },
///     "ledger_index": 12345,
///     "validated": true
/// });
///
/// let response: AccountInfoResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.account_data.sequence, 1);
/// assert_eq!(response.validated, Some(true));
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfoResponse {
    /// The account data from the ledger.
    pub account_data: AccountRootData,
    /// Queue information (if requested).
    #[serde(default)]
    pub queue_data: Option<serde_json::Value>,
    /// Signer lists (if requested).
    #[serde(default)]
    pub signer_lists: Option<Vec<serde_json::Value>>,
    /// The ledger index used (if using "current" ledger).
    pub ledger_current_index: Option<u32>,
    /// The ledger index used (if using validated/closed).
    pub ledger_index: Option<u32>,
    /// Whether this data is from a validated ledger.
    pub validated: Option<bool>,
}

/// Account root data as returned in responses.
///
/// This is a response-oriented type that uses flexible deserialization.
/// For the full ledger entry type, see `crate::ledger::AccountRoot`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountRootData {
    /// The account address.
    pub account: AccountId,
    /// XRP balance in drops.
    pub balance: String,
    /// Sequence number.
    pub sequence: u32,
    /// Account flags.
    #[serde(default)]
    pub flags: u32,
    /// Number of objects owned.
    #[serde(default)]
    pub owner_count: u32,
    /// Previous transaction hash.
    pub previous_txn_id: Option<Hash256>,
    /// Ledger sequence of the previous transaction.
    pub previous_txn_lgr_seq: Option<u32>,
    /// Transfer rate (if set).
    pub transfer_rate: Option<u32>,
    /// Domain (hex-encoded).
    pub domain: Option<String>,
    /// Regular key address.
    pub regular_key: Option<AccountId>,
    /// Ticket count.
    pub ticket_count: Option<u32>,
    /// Ledger entry index.
    #[serde(default)]
    pub index: Option<String>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `account_lines` method.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::responses::account::AccountLinesResponse;
///
/// let json = serde_json::json!({
///     "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "lines": [{
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///         "balance": "10",
///         "currency": "USD",
///         "limit": "100",
///         "limit_peer": "0"
///     }]
/// });
///
/// let response: AccountLinesResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.lines.len(), 1);
/// assert_eq!(response.lines[0].currency, "USD");
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountLinesResponse {
    /// The queried account.
    pub account: AccountId,
    /// Trust lines.
    pub lines: Vec<TrustLine>,
    /// Ledger index used.
    pub ledger_current_index: Option<u32>,
    /// Validated ledger index.
    pub ledger_index: Option<u32>,
    /// Whether this data is from a validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker for the next page.
    pub marker: Option<Marker>,
}

/// A trust line as returned by account_lines.
#[derive(Debug, Clone, Deserialize)]
pub struct TrustLine {
    /// The counterparty account.
    pub account: AccountId,
    /// The balance (positive = we hold, negative = they hold).
    pub balance: String,
    /// The currency code.
    pub currency: String,
    /// Our trust limit.
    pub limit: String,
    /// Their trust limit.
    pub limit_peer: String,
    /// Quality in.
    pub quality_in: Option<u32>,
    /// Quality out.
    pub quality_out: Option<u32>,
    /// Whether we set the no_ripple flag.
    pub no_ripple: Option<bool>,
    /// Whether the peer set no_ripple.
    pub no_ripple_peer: Option<bool>,
    /// Whether the line is frozen.
    pub freeze: Option<bool>,
    /// Whether the peer froze the line.
    pub freeze_peer: Option<bool>,
}

/// Response from the `account_channels` method.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::responses::account::AccountChannelsResponse;
///
/// let json = serde_json::json!({
///     "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "channels": [{
///         "channel_id": "ABC123",
///         "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///         "destination_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///         "amount": "10000000",
///         "balance": "0",
///         "settle_delay": 86400
///     }]
/// });
///
/// let response: AccountChannelsResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.channels.len(), 1);
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AccountChannelsResponse {
    /// The queried account.
    pub account: AccountId,
    /// Payment channels.
    pub channels: Vec<ChannelInfo>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// A payment channel entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelInfo {
    /// The channel ID.
    pub channel_id: String,
    /// The source account.
    pub account: AccountId,
    /// The destination account.
    pub destination_account: AccountId,
    /// Total amount allocated to this channel (drops).
    pub amount: String,
    /// Amount already paid out (drops).
    pub balance: String,
    /// Public key for claim verification.
    pub public_key: Option<String>,
    /// Seconds the channel must stay open after requesting close.
    pub settle_delay: u32,
    /// Expiration time.
    pub expiration: Option<u32>,
    /// Cancel-after time.
    pub cancel_after: Option<u32>,
    /// Source tag.
    pub source_tag: Option<u32>,
    /// Destination tag.
    pub destination_tag: Option<u32>,
}

/// Response from the `account_currencies` method.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountCurrenciesResponse {
    /// Currencies this account can receive.
    pub receive_currencies: Vec<String>,
    /// Currencies this account can send.
    pub send_currencies: Vec<String>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
}

/// Response from the `account_nfts` method.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountNftsResponse {
    /// The queried account.
    pub account: AccountId,
    /// NFTs owned by the account.
    pub account_nfts: Vec<AccountNft>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// An NFT entry in account_nfts response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountNft {
    /// The NFToken ID.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Hash256,
    /// The issuer of this NFT.
    pub issuer: AccountId,
    /// The taxon.
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,
    /// Flags.
    pub flags: u32,
    /// Transfer fee (0-50000 in basis points).
    pub transfer_fee: Option<u16>,
    /// URI (hex-encoded).
    #[serde(rename = "URI")]
    pub uri: Option<String>,
    /// Serial number.
    pub nft_serial: Option<u32>,
}

/// Response from the `account_objects` method.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountObjectsResponse {
    /// The queried account.
    pub account: AccountId,
    /// Ledger objects owned by the account.
    pub account_objects: Vec<serde_json::Value>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// Response from the `account_offers` method.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountOffersResponse {
    /// The queried account.
    pub account: AccountId,
    /// Open offers.
    pub offers: Vec<AccountOffer>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// An offer entry in account_offers response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountOffer {
    /// Sequence number of the offer.
    pub seq: u32,
    /// Flags.
    pub flags: u32,
    /// Amount being offered.
    pub taker_gets: Amount,
    /// Amount being requested.
    pub taker_pays: Amount,
    /// Quality (exchange rate).
    pub quality: Option<String>,
    /// Expiration.
    pub expiration: Option<u32>,
}

/// Response from the `account_tx` method.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTxResponse {
    /// The queried account.
    pub account: AccountId,
    /// Transaction entries.
    pub transactions: Vec<AccountTransaction>,
    /// Ledger index range min.
    pub ledger_index_min: Option<i64>,
    /// Ledger index range max.
    pub ledger_index_max: Option<i64>,
    /// Whether from validated ledger.
    pub validated: Option<bool>,
    /// Pagination marker.
    pub marker: Option<Marker>,
}

/// A transaction entry in account_tx response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTransaction {
    /// The transaction data.
    pub tx: Option<serde_json::Value>,
    /// Transaction in JSON format (API v2).
    pub tx_json: Option<serde_json::Value>,
    /// Transaction metadata.
    pub meta: Option<serde_json::Value>,
    /// Whether this transaction was validated.
    pub validated: Option<bool>,
}

/// Response from the `gateway_balances` method.
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayBalancesResponse {
    /// The queried account.
    pub account: AccountId,
    /// Total obligations.
    pub obligations: Option<serde_json::Value>,
    /// Balances held by the hot wallets.
    pub balances: Option<serde_json::Value>,
    /// Frozen balances.
    pub frozen_balances: Option<serde_json::Value>,
    /// Assets held by the gateway.
    pub assets: Option<serde_json::Value>,
    /// Ledger index used.
    pub ledger_index: Option<u32>,
}

/// Response from the `noripple_check` method.
#[derive(Debug, Clone, Deserialize)]
pub struct NorippleCheckResponse {
    /// Ledger index used.
    pub ledger_current_index: Option<u32>,
    /// Problems found.
    pub problems: Vec<String>,
    /// Suggested fix transactions.
    pub transactions: Option<Vec<serde_json::Value>>,
}

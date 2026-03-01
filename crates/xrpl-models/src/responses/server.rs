//! Server information response types.

use serde::Deserialize;

/// Response from the `fee` method.
///
/// # Examples
///
/// ```
/// use xrpl_models::responses::server::FeeResponse;
///
/// let json = serde_json::json!({
///     "current_ledger_size": "0",
///     "current_queue_size": "0",
///     "drops": {
///         "base_fee": "10",
///         "median_fee": "5000",
///         "minimum_fee": "10",
///         "open_ledger_fee": "10"
///     },
///     "expected_ledger_size": "1000",
///     "ledger_current_index": 12345,
///     "max_queue_size": "2000"
/// });
///
/// let response: FeeResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.drops.base_fee, "10");
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct FeeResponse {
    /// Current transaction cost in drops.
    pub current_ledger_size: Option<String>,
    /// Current queue size.
    pub current_queue_size: Option<String>,
    /// Fee drop amounts.
    pub drops: FeeDrops,
    /// Expected ledger size.
    pub expected_ledger_size: Option<String>,
    /// Ledger index this fee applies to.
    pub ledger_current_index: Option<u32>,
    /// Fee level info.
    pub levels: Option<FeeLevels>,
    /// Maximum queue size.
    pub max_queue_size: Option<String>,
}

/// Fee amounts in drops.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeDrops {
    /// The minimum fee required to be included in a ledger.
    pub base_fee: String,
    /// The median fee of recent transactions.
    pub median_fee: String,
    /// The minimum fee to get into the open ledger.
    pub minimum_fee: String,
    /// The fee to jump the open ledger queue.
    pub open_ledger_fee: String,
}

/// Fee escalation levels.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeLevels {
    /// The median level of recent transactions.
    pub median_level: String,
    /// The minimum level to be queued.
    pub minimum_level: String,
    /// The level to enter the open ledger.
    pub open_ledger_level: String,
    /// The reference fee level.
    pub reference_level: String,
}

/// Response from the `server_info` method.
///
/// # Examples
///
/// ```
/// use xrpl_models::responses::server::ServerInfoResponse;
///
/// let json = serde_json::json!({
///     "info": {
///         "build_version": "2.3.0",
///         "server_state": "full",
///         "complete_ledgers": "1-100000"
///     }
/// });
///
/// let response: ServerInfoResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.info.build_version, Some("2.3.0".to_string()));
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfoResponse {
    /// Server information.
    pub info: ServerInfo,
}

/// Detailed server information.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    /// The build version of the server.
    pub build_version: Option<String>,
    /// Time the server started.
    pub time: Option<String>,
    /// Server uptime in seconds.
    pub uptime: Option<u64>,
    /// Server state string (e.g., "full", "proposing").
    pub server_state: Option<String>,
    /// Validated ledger info.
    pub validated_ledger: Option<ValidatedLedgerInfo>,
    /// Complete ledger ranges available.
    pub complete_ledgers: Option<String>,
    /// Load factor.
    pub load_factor: Option<f64>,
    /// Network ID.
    pub network_id: Option<u32>,
    /// Additional fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Info about the last validated ledger.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatedLedgerInfo {
    /// The base fee in XRP.
    pub base_fee_xrp: Option<f64>,
    /// Ledger hash.
    pub hash: Option<String>,
    /// Reserve base in XRP.
    pub reserve_base_xrp: Option<f64>,
    /// Reserve increment in XRP.
    pub reserve_inc_xrp: Option<f64>,
    /// Ledger sequence number.
    pub seq: u32,
}

/// Response from the `server_state` method.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerStateResponse {
    /// Server state information.
    pub state: serde_json::Value,
}

/// Response from the `manifest` method.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestResponse {
    /// The requested manifest data.
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `server_definitions` method.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerDefinitionsResponse {
    /// Field definitions from the server.
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

/// Response from the `ping` method.
#[derive(Debug, Clone, Deserialize)]
pub struct PingResponse {
    /// Additional fields (usually empty).
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

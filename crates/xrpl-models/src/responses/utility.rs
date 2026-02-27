//! Utility response types.

use serde::Deserialize;

/// Response from the `random` method.
#[derive(Debug, Clone, Deserialize)]
pub struct RandomResponse {
    /// A random 256-bit hex string.
    pub random: String,
}

/// Response from the `json` escape-hatch method.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonResponse {
    /// The raw response data.
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

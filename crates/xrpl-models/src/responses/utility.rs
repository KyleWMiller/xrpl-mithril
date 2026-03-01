//! Utility response types.

use serde::Deserialize;

/// Response from the `random` method.
///
/// # Examples
///
/// ```
/// use xrpl_models::responses::utility::RandomResponse;
///
/// let json = serde_json::json!({
///     "random": "E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7"
/// });
///
/// let response: RandomResponse = serde_json::from_value(json).unwrap();
/// assert_eq!(response.random.len(), 64);
/// ```
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

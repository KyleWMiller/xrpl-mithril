//! Utility request types.

use serde::Serialize;

use super::XrplRequest;
use crate::responses::utility::{JsonResponse, RandomResponse};

/// Request a random 256-bit value from the server.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_models::requests::RandomRequest;
///
/// let request = RandomRequest {};
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct RandomRequest {}

impl XrplRequest for RandomRequest {
    type Response = RandomResponse;
    fn method(&self) -> &'static str {
        "random"
    }
}

/// Send a raw JSON command to the server.
///
/// This is an escape hatch for methods not yet covered by typed requests.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRequest {
    /// The method name.
    #[serde(skip)]
    pub method_name: String,
    /// The method parameters.
    #[serde(flatten)]
    pub params: serde_json::Value,
}

impl XrplRequest for JsonRequest {
    type Response = JsonResponse;
    fn method(&self) -> &'static str {
        // Safety: we need a &'static str but have a String.
        // This leaks the string intentionally for the rare case of dynamic methods.
        // In practice, callers should use typed requests.
        Box::leak(self.method_name.clone().into_boxed_str())
    }
}

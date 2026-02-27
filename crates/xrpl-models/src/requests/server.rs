//! Server information and status request types.

use serde::Serialize;

use super::XrplRequest;
use crate::responses::server::{
    FeeResponse, ManifestResponse, PingResponse, ServerDefinitionsResponse, ServerInfoResponse,
    ServerStateResponse,
};

/// Request the current transaction fee information.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FeeRequest {}

impl XrplRequest for FeeRequest {
    type Response = FeeResponse;
    fn method(&self) -> &'static str {
        "fee"
    }
}

/// Request detailed server information.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ServerInfoRequest {}

impl XrplRequest for ServerInfoRequest {
    type Response = ServerInfoResponse;
    fn method(&self) -> &'static str {
        "server_info"
    }
}

/// Request server state information (lower-level than server_info).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ServerStateRequest {}

impl XrplRequest for ServerStateRequest {
    type Response = ServerStateResponse;
    fn method(&self) -> &'static str {
        "server_state"
    }
}

/// Request the manifest for a validator public key.
#[derive(Debug, Clone, Serialize)]
pub struct ManifestRequest {
    /// The base58-encoded public key of the validator.
    pub public_key: String,
}

impl XrplRequest for ManifestRequest {
    type Response = ManifestResponse;
    fn method(&self) -> &'static str {
        "manifest"
    }
}

/// Request the server's protocol field definitions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ServerDefinitionsRequest {
    /// If provided, the server only returns data if its hash differs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl XrplRequest for ServerDefinitionsRequest {
    type Response = ServerDefinitionsResponse;
    fn method(&self) -> &'static str {
        "server_definitions"
    }
}

/// Ping the server to confirm connectivity.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PingRequest {}

impl XrplRequest for PingRequest {
    type Response = PingResponse;
    fn method(&self) -> &'static str {
        "ping"
    }
}

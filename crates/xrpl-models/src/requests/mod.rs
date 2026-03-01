//! Typed request types for all XRPL JSON-RPC and WebSocket methods.
//!
//! Each request struct implements [`XrplRequest`], which associates the request
//! with its corresponding response type and RPC method name.

pub mod account;
pub mod amm;
pub mod ledger;
pub mod nft;
pub mod oracle;
pub mod path;
pub mod server;
pub mod subscription;
pub mod transaction;
pub mod utility;

use serde::{Deserialize, Serialize};
use xrpl_types::Hash256;

/// Trait implemented by all typed XRPL RPC requests.
///
/// Associates each request with its response type and method name, enabling
/// the `Client` trait (from the `xrpl-client` crate) to provide fully typed
/// request/response pairs.
pub trait XrplRequest: Serialize {
    /// The response type returned by this request.
    type Response: for<'de> Deserialize<'de>;

    /// The JSON-RPC method name (e.g., `"account_info"`, `"submit"`).
    fn method(&self) -> &'static str;

    /// The API version to use. Defaults to 2.
    fn api_version(&self) -> u8 {
        2
    }
}

/// Specifies which ledger to query.
///
/// Used by most request types that accept a `ledger_index` or `ledger_hash`
/// parameter.
///
/// # Examples
///
/// ```
/// use xrpl_models::requests::LedgerSpecifier;
///
/// // Use a named shortcut:
/// let json = serde_json::to_value(&LedgerSpecifier::Named(
///     xrpl_models::requests::LedgerShortcut::Validated,
/// )).unwrap();
/// assert_eq!(json, serde_json::json!("validated"));
///
/// // Use a specific ledger index:
/// let json = serde_json::to_value(&LedgerSpecifier::Index(12345)).unwrap();
/// assert_eq!(json, serde_json::json!(12345));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum LedgerSpecifier {
    /// A named ledger shortcut.
    Named(LedgerShortcut),
    /// A specific ledger sequence number.
    Index(u32),
    /// A specific ledger by hash.
    Hash(LedgerHashSpecifier),
}

/// Named ledger shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LedgerShortcut {
    /// The most recent ledger that has been validated by consensus.
    #[serde(rename = "validated")]
    Validated,
    /// The most recent ledger that has been closed for voting.
    #[serde(rename = "closed")]
    Closed,
    /// The server's current working ledger (in-progress).
    #[serde(rename = "current")]
    Current,
}

/// Specifier for a ledger by its hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerHashSpecifier {
    /// The identifying hash of the ledger version.
    pub ledger_hash: Hash256,
}

/// Opaque pagination marker returned by paginated responses.
///
/// Pass this back in the next request to continue pagination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker(pub serde_json::Value);

// Re-export key types for convenience
pub use account::*;
pub use amm::*;
pub use ledger::*;
pub use nft::*;
pub use oracle::*;
pub use path::*;
pub use server::*;
pub use subscription::*;
pub use transaction::*;
pub use utility::*;

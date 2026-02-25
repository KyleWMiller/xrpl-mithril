//! XRPL account identifiers.
//!
//! An [`AccountId`] is a 20-byte identifier derived from a public key via
//! SHA-256 followed by RIPEMD-160. It is displayed as a base58check-encoded
//! "classic address" (starting with 'r').

use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Version byte for XRPL classic addresses.
const CLASSIC_ADDRESS_VERSION: u8 = 0;

/// A 20-byte XRPL account identifier.
///
/// This is the canonical representation used in binary serialization.
/// Classic addresses (r...) and X-addresses are display/parsing formats.
///
/// In JSON, serialized as a base58check classic address string
/// (e.g., `"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId([u8; 20]);

impl AccountId {
    /// The byte length of an account identifier.
    pub const LEN: usize = 20;

    /// Creates an `AccountId` from a 20-byte array.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Creates an `AccountId` from a byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::InvalidAccountIdLength`] if the slice is not exactly 20 bytes.
    pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        let bytes: [u8; 20] = slice
            .try_into()
            .map_err(|_| TypeError::InvalidAccountIdLength(slice.len()))?;
        Ok(Self(bytes))
    }

    /// Creates an `AccountId` from a classic address string (starting with 'r').
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::InvalidAddress`] if the string is not a valid base58check
    /// classic address with version byte 0x00.
    pub fn from_classic_address(address: &str) -> Result<Self, TypeError> {
        let decoded = bs58::decode(address)
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .with_check(None)
            .into_vec()
            .map_err(|e| TypeError::InvalidAddress(e.to_string()))?;

        if decoded.is_empty() || decoded[0] != CLASSIC_ADDRESS_VERSION {
            return Err(TypeError::InvalidAddress(
                "invalid version byte for classic address".into(),
            ));
        }

        Self::from_slice(&decoded[1..])
    }

    /// Returns the classic address string (starting with 'r').
    #[must_use]
    pub fn to_classic_address(&self) -> String {
        bs58::encode(self.0)
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .with_check_version(CLASSIC_ADDRESS_VERSION)
            .into_string()
    }

    /// Returns the raw 20-byte representation.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// The account ID representing the genesis account (all zeros).
    pub const ZERO: Self = Self([0u8; 20]);
}

impl Serialize for AccountId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_classic_address())
    }
}

impl<'de> Deserialize<'de> for AccountId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_classic_address(&s).map_err(serde::de::Error::custom)
    }
}

impl core::fmt::Debug for AccountId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AccountId({})", self.to_classic_address())
    }
}

impl core::fmt::Display for AccountId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_classic_address())
    }
}

impl core::str::FromStr for AccountId {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_classic_address(s)
    }
}

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 20]> for AccountId {
    fn from(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_address_round_trip() {
        let address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let account =
            AccountId::from_classic_address(address).expect("should decode valid address");
        assert_eq!(account.to_classic_address(), address);
    }

    #[test]
    fn display_shows_classic_address() {
        let address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let account =
            AccountId::from_classic_address(address).expect("should decode valid address");
        assert_eq!(format!("{account}"), address);
    }

    #[test]
    fn from_str_works() {
        let address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let account: AccountId = address.parse().expect("should parse valid address");
        assert_eq!(account.to_classic_address(), address);
    }

    #[test]
    fn json_round_trip() {
        let address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let account =
            AccountId::from_classic_address(address).expect("should decode valid address");
        let json = serde_json::to_string(&account).expect("should serialize");
        assert_eq!(json, format!("\"{address}\""));
        let decoded: AccountId = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, account);
    }

    #[test]
    fn invalid_address_rejected() {
        assert!(AccountId::from_classic_address("invalid").is_err());
        assert!(AccountId::from_classic_address("").is_err());
    }

    #[test]
    fn zero_account_encodes() {
        let zero = AccountId::ZERO;
        let address = zero.to_classic_address();
        let decoded =
            AccountId::from_classic_address(&address).expect("should decode zero address");
        assert_eq!(decoded, zero);
    }
}

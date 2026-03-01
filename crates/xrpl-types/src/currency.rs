//! Currency codes and asset identifiers.
//!
//! The XRPL protocol uses several representations for assets:
//! - [`CurrencyCode`]: A 20-byte currency identifier (3-char standard or hex)
//! - [`MptIssuanceId`]: A 24-byte (192-bit) MPT issuance identifier
//! - [`Issue`]: A unified asset identifier (XRP, issued currency, or MPT)

use crate::account::AccountId;
use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A 20-byte XRPL currency code.
///
/// Two formats:
/// - **Standard**: 3 ASCII characters in bytes 12-14, all other bytes zero.
///   Byte 0 is 0x00 to indicate standard format.
/// - **Non-standard**: 20 arbitrary bytes where byte 0 is NOT 0x00.
///
/// All zeros represents XRP (but XRP should use [`Issue::Xrp`] instead).
///
/// In JSON, standard codes serialize as 3-char strings (e.g., `"USD"`),
/// non-standard codes serialize as 40-char uppercase hex strings.
///
/// # Examples
///
/// Standard 3-character currency code:
///
/// ```
/// use xrpl_types::CurrencyCode;
///
/// let usd = CurrencyCode::from_ascii("USD").unwrap();
/// assert!(usd.is_standard());
/// assert_eq!(format!("{usd}"), "USD");
/// ```
///
/// Non-standard currency code from hex:
///
/// ```
/// use xrpl_types::CurrencyCode;
///
/// let hex_code = "0158415500000000000000000000000000000000";
/// let code = CurrencyCode::from_hex(hex_code).unwrap();
/// assert!(!code.is_standard());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrencyCode([u8; 20]);

impl CurrencyCode {
    /// The byte length of a currency code.
    pub const LEN: usize = 20;

    /// Creates a currency code from a raw 20-byte array.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Creates a standard 3-character currency code (e.g., "USD", "EUR").
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::InvalidCurrencyCode`] if the string is not exactly 3 ASCII characters,
    /// or if it would encode as XRP (all zeros).
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::CurrencyCode;
    ///
    /// let usd = CurrencyCode::from_ascii("USD").unwrap();
    /// assert!(usd.is_standard());
    /// assert_eq!(usd.as_ascii(), Some([b'U', b'S', b'D']));
    ///
    /// // Non-3-character codes are rejected:
    /// assert!(CurrencyCode::from_ascii("US").is_err());
    /// assert!(CurrencyCode::from_ascii("USDT").is_err());
    /// ```
    pub fn from_ascii(code: &str) -> Result<Self, TypeError> {
        let bytes = code.as_bytes();
        if bytes.len() != 3 {
            return Err(TypeError::InvalidCurrencyCode);
        }
        // Standard codes must be printable ASCII
        if !bytes.iter().all(|b| b.is_ascii_alphanumeric()) {
            return Err(TypeError::InvalidCurrencyCode);
        }

        let mut result = [0u8; 20];
        result[12] = bytes[0];
        result[13] = bytes[1];
        result[14] = bytes[2];

        Ok(Self(result))
    }

    /// Creates a currency code from a hex string.
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::InvalidHex`] if not valid hex or wrong length.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::CurrencyCode;
    ///
    /// // Non-standard 20-byte hex currency code (40 hex chars):
    /// let hex_code = "0158415500000000000000000000000000000000";
    /// let code = CurrencyCode::from_hex(hex_code).unwrap();
    /// assert!(!code.is_standard());
    ///
    /// // Wrong length returns an error:
    /// assert!(CurrencyCode::from_hex("ABCD").is_err());
    /// ```
    pub fn from_hex(hex_str: &str) -> Result<Self, TypeError> {
        let bytes = hex::decode(hex_str).map_err(|e| TypeError::InvalidHex(e.to_string()))?;
        if bytes.len() != 20 {
            return Err(TypeError::InvalidCurrencyCode);
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns `true` if this is a standard 3-character currency code.
    #[must_use]
    pub const fn is_standard(&self) -> bool {
        self.0[0] == 0x00
    }

    /// Returns the raw 20-byte representation.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Returns the 3-character ASCII code for standard currencies.
    ///
    /// Returns `None` for non-standard (hex) currency codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::CurrencyCode;
    ///
    /// let usd = CurrencyCode::from_ascii("USD").unwrap();
    /// assert_eq!(usd.as_ascii(), Some([b'U', b'S', b'D']));
    ///
    /// // Non-standard codes return None:
    /// let hex_code = "0158415500000000000000000000000000000000";
    /// let code = CurrencyCode::from_hex(hex_code).unwrap();
    /// assert_eq!(code.as_ascii(), None);
    /// ```
    #[must_use]
    pub fn as_ascii(&self) -> Option<[u8; 3]> {
        if !self.is_standard() {
            return None;
        }
        Some([self.0[12], self.0[13], self.0[14]])
    }
}

impl Serialize for CurrencyCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ascii) = self.as_ascii() {
            if let Ok(s) = core::str::from_utf8(&ascii) {
                return serializer.serialize_str(s);
            }
        }
        serializer.serialize_str(&hex::encode_upper(self.0))
    }
}

impl<'de> Deserialize<'de> for CurrencyCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s.len() == 3 {
            // Standard 3-char currency code
            Self::from_ascii(&s).map_err(serde::de::Error::custom)
        } else if s.len() == 40 {
            // Non-standard 20-byte hex
            Self::from_hex(&s).map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom(format!(
                "currency code must be 3 chars or 40 hex chars, got {} chars",
                s.len()
            )))
        }
    }
}

impl core::fmt::Debug for CurrencyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ascii) = self.as_ascii() {
            if let Ok(s) = core::str::from_utf8(&ascii) {
                return write!(f, "CurrencyCode({s})");
            }
        }
        write!(f, "CurrencyCode({})", hex::encode_upper(self.0))
    }
}

impl core::fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ascii) = self.as_ascii() {
            if let Ok(s) = core::str::from_utf8(&ascii) {
                return write!(f, "{s}");
            }
        }
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

/// A 24-byte (192-bit) Multi-Purpose Token issuance identifier.
///
/// Uniquely identifies an MPT issuance on the ledger.
///
/// In JSON, serialized as a 48-char uppercase hex string.
///
/// # Examples
///
/// ```
/// use xrpl_types::MptIssuanceId;
///
/// // 48 hex chars = 24 bytes:
/// let id = MptIssuanceId::from_hex(
///     "000000010000000000000000AABBCCDDAABBCCDDAABBCCDD"
/// ).unwrap();
/// assert_eq!(id.as_bytes().len(), 24);
///
/// let json = serde_json::to_string(&id).unwrap();
/// let decoded: MptIssuanceId = serde_json::from_str(&json).unwrap();
/// assert_eq!(decoded, id);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MptIssuanceId([u8; 24]);

impl MptIssuanceId {
    /// The byte length of an MPT issuance ID.
    pub const LEN: usize = 24;

    /// Creates an `MptIssuanceId` from a 24-byte array.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 24]) -> Self {
        Self(bytes)
    }

    /// Creates from a hex string.
    ///
    /// # Errors
    ///
    /// Returns an error if the hex is invalid or the wrong length.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::MptIssuanceId;
    ///
    /// // 48 hex chars = 24 bytes:
    /// let id = MptIssuanceId::from_hex(
    ///     "000000010000000000000000AABBCCDDAABBCCDDAABBCCDD"
    /// ).unwrap();
    /// assert_eq!(id.as_bytes().len(), 24);
    ///
    /// // Wrong length returns an error:
    /// assert!(MptIssuanceId::from_hex("ABAB").is_err());
    /// ```
    pub fn from_hex(hex_str: &str) -> Result<Self, TypeError> {
        let bytes = hex::decode(hex_str).map_err(|e| TypeError::InvalidHex(e.to_string()))?;
        if bytes.len() != 24 {
            return Err(TypeError::InvalidHashLength {
                expected: 24,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 24];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns the raw 24-byte representation.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 24] {
        &self.0
    }
}

impl Serialize for MptIssuanceId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(self.0))
    }
}

impl<'de> Deserialize<'de> for MptIssuanceId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl core::fmt::Debug for MptIssuanceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MptIssuanceId({})", hex::encode_upper(self.0))
    }
}

impl core::fmt::Display for MptIssuanceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

impl From<[u8; 24]> for MptIssuanceId {
    fn from(bytes: [u8; 24]) -> Self {
        Self(bytes)
    }
}

/// A unified asset identifier for the XRPL.
///
/// Represents what asset a field refers to. Used in AMM transactions
/// (the `Asset` and `Asset2` fields) and other contexts where an asset
/// must be identified without an amount.
///
/// JSON representations:
/// - XRP: `{"currency": "XRP"}`
/// - Issued: `{"currency": "USD", "issuer": "rHb9..."}`
/// - MPT: `{"mpt_issuance_id": "0000..."}`
///
/// # Examples
///
/// ```
/// use xrpl_types::{AccountId, CurrencyCode, Issue};
///
/// // XRP as an asset:
/// let xrp = Issue::Xrp;
/// let json = serde_json::to_string(&xrp).unwrap();
/// assert_eq!(json, r#"{"currency":"XRP"}"#);
///
/// // An issued currency asset:
/// let issuer: AccountId = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().unwrap();
/// let usd = Issue::Issued {
///     currency: CurrencyCode::from_ascii("USD").unwrap(),
///     issuer,
/// };
/// let json = serde_json::to_string(&usd).unwrap();
/// assert!(json.contains("\"currency\":\"USD\""));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Issue {
    /// Native XRP (no issuer needed).
    Xrp,
    /// An issued currency (IOU) identified by currency code and issuer.
    Issued {
        /// The currency code.
        currency: CurrencyCode,
        /// The issuer's account.
        issuer: AccountId,
    },
    /// A Multi-Purpose Token identified by its issuance ID.
    Mpt(MptIssuanceId),
}

impl Serialize for Issue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Issue::Xrp => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("currency", "XRP")?;
                map.end()
            }
            Issue::Issued { currency, issuer } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("currency", currency)?;
                map.serialize_entry("issuer", issuer)?;
                map.end()
            }
            Issue::Mpt(id) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("mpt_issuance_id", id)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Issue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map: serde_json::Map<String, serde_json::Value> =
            serde_json::Map::deserialize(deserializer)?;

        if let Some(mpt_id) = map.get("mpt_issuance_id") {
            let id: MptIssuanceId =
                serde_json::from_value(mpt_id.clone()).map_err(serde::de::Error::custom)?;
            return Ok(Issue::Mpt(id));
        }

        if let Some(currency_val) = map.get("currency") {
            let currency_str = currency_val
                .as_str()
                .ok_or_else(|| serde::de::Error::custom("currency must be a string"))?;

            if currency_str == "XRP" {
                return Ok(Issue::Xrp);
            }

            let currency: CurrencyCode =
                serde_json::from_value(currency_val.clone()).map_err(serde::de::Error::custom)?;

            let issuer_val = map
                .get("issuer")
                .ok_or_else(|| serde::de::Error::custom("issued currency requires an issuer"))?;
            let issuer: AccountId =
                serde_json::from_value(issuer_val.clone()).map_err(serde::de::Error::custom)?;

            return Ok(Issue::Issued { currency, issuer });
        }

        Err(serde::de::Error::custom(
            "Issue must have 'currency' or 'mpt_issuance_id'",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_code_standard_json_round_trip() {
        let usd = CurrencyCode::from_ascii("USD").expect("valid code");
        let json = serde_json::to_string(&usd).expect("should serialize");
        assert_eq!(json, "\"USD\"");
        let decoded: CurrencyCode = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, usd);
    }

    #[test]
    fn currency_code_nonstandard_json_round_trip() {
        let mut bytes = [0u8; 20];
        bytes[0] = 0x01; // non-standard
        bytes[1] = 0xFF;
        let code = CurrencyCode::from_bytes(bytes);
        let json = serde_json::to_string(&code).expect("should serialize");
        let decoded: CurrencyCode = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, code);
    }

    #[test]
    fn mpt_issuance_id_json_round_trip() {
        let bytes = [0xABu8; 24];
        let id = MptIssuanceId::from_bytes(bytes);
        let json = serde_json::to_string(&id).expect("should serialize");
        let decoded: MptIssuanceId = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, id);
    }

    #[test]
    fn issue_xrp_json_round_trip() {
        let issue = Issue::Xrp;
        let json = serde_json::to_string(&issue).expect("should serialize");
        assert_eq!(json, r#"{"currency":"XRP"}"#);
        let decoded: Issue = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, issue);
    }

    #[test]
    fn issue_issued_json_round_trip() {
        let issuer =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").expect("valid");
        let issue = Issue::Issued {
            currency: CurrencyCode::from_ascii("USD").expect("valid"),
            issuer,
        };
        let json = serde_json::to_string(&issue).expect("should serialize");
        let decoded: Issue = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, issue);
    }

    #[test]
    fn issue_mpt_json_round_trip() {
        let issue = Issue::Mpt(MptIssuanceId::from_bytes([0xAB; 24]));
        let json = serde_json::to_string(&issue).expect("should serialize");
        let decoded: Issue = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, issue);
    }
}

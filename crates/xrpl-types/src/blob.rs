//! Variable-length binary data.
//!
//! [`Blob`] represents the XRPL `Blob` type (type code 7), which is
//! variable-length binary data with a VL-encoded length prefix in
//! the binary format.
//!
//! In JSON, serialized as an uppercase hex string.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Variable-length binary data.
///
/// Used for fields like `PublicKey`, `SigningPubKey`, `TxnSignature`,
/// `Domain`, `MemoType`, `MemoData`, `Fulfillment`, `Condition`, etc.
///
/// In JSON, serialized as an uppercase hex string.
///
/// # Examples
///
/// Creating from raw bytes:
///
/// ```
/// use xrpl_types::Blob;
///
/// let blob = Blob::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
/// assert_eq!(blob.len(), 4);
/// assert_eq!(format!("{blob}"), "DEADBEEF");
/// ```
///
/// Creating from a hex string:
///
/// ```
/// use xrpl_types::Blob;
///
/// let blob = Blob::from_hex("DEADBEEF").unwrap();
/// assert_eq!(blob.as_bytes(), &[0xDE, 0xAD, 0xBE, 0xEF]);
/// ```
///
/// JSON round-trip:
///
/// ```
/// use xrpl_types::Blob;
///
/// let blob = Blob::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
/// let json = serde_json::to_string(&blob).unwrap();
/// assert_eq!(json, "\"DEADBEEF\"");
///
/// let decoded: Blob = serde_json::from_str(&json).unwrap();
/// assert_eq!(decoded, blob);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Blob(Vec<u8>);

impl Blob {
    /// Creates a new `Blob` from a byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::Blob;
    ///
    /// let blob = Blob::new(vec![0x01, 0x02, 0x03]);
    /// assert_eq!(blob.len(), 3);
    /// ```
    #[must_use]
    pub const fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Creates an empty `Blob`.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Creates a `Blob` from a hex string (accepts uppercase or lowercase).
    ///
    /// # Errors
    ///
    /// Returns `TypeError::InvalidHex` if the string is not valid hex.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::Blob;
    ///
    /// let blob = Blob::from_hex("DEADBEEF").unwrap();
    /// assert_eq!(blob.as_bytes(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    ///
    /// // Lowercase hex is also accepted:
    /// let blob2 = Blob::from_hex("deadbeef").unwrap();
    /// assert_eq!(blob2, blob);
    /// ```
    pub fn from_hex(hex_str: &str) -> Result<Self, crate::error::TypeError> {
        let bytes =
            hex::decode(hex_str).map_err(|e| crate::error::TypeError::InvalidHex(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Returns the raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes the `Blob` and returns the inner byte vector.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl Serialize for Blob {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(&self.0))
    }
}

impl<'de> Deserialize<'de> for Blob {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(bytes))
    }
}

impl From<Vec<u8>> for Blob {
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<&[u8]> for Blob {
    fn from(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for Blob {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", hex::encode_upper(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let blob = Blob::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let json = serde_json::to_string(&blob).expect("should serialize");
        assert_eq!(json, "\"DEADBEEF\"");
        let decoded: Blob = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, blob);
    }

    #[test]
    fn accepts_lowercase_hex() {
        let json = "\"deadbeef\"";
        let blob: Blob = serde_json::from_str(json).expect("should accept lowercase");
        assert_eq!(blob.as_bytes(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn empty_blob() {
        let blob = Blob::empty();
        let json = serde_json::to_string(&blob).expect("should serialize");
        assert_eq!(json, "\"\"");
        let decoded: Blob = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, blob);
    }
}

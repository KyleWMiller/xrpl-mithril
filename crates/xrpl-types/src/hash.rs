//! Fixed-size hash types used in the XRPL protocol.
//!
//! These correspond to the XRPL binary format types:
//! - [`Hash128`] — 16 bytes (type code 4)
//! - [`Hash160`] — 20 bytes (type code 17)
//! - [`Hash256`] — 32 bytes (type code 5)
//!
//! Additional large unsigned integer types used as hashes in the protocol:
//! - [`UInt96`] — 12 bytes (type code 20)
//! - [`Hash192`] — 24 bytes (type code 21)
//! - [`UInt384`] — 48 bytes (type code 22)
//! - [`UInt512`] — 64 bytes (type code 23)
//!
//! All hash types serialize to/from uppercase hex strings in JSON.

use crate::error::TypeError;

macro_rules! define_hash_type {
    (
        $(#[$meta:meta])*
        $name:ident, $len:literal, serde
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name([u8; $len]);

        define_hash_type!(@impls $name, $len);
        define_hash_type!(@serde $name, $len);
    };
    (
        $(#[$meta:meta])*
        $name:ident, $len:literal
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name([u8; $len]);

        define_hash_type!(@impls $name, $len);
    };
    (@serde $name:ident, $len:literal) => {
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&hex::encode_upper(self.0))
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = <String as serde::Deserialize>::deserialize(deserializer)?;
                let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
                let arr: [u8; $len] = bytes.try_into().map_err(|_| {
                    serde::de::Error::custom(
                        concat!("expected ", stringify!($len), " bytes for ", stringify!($name))
                    )
                })?;
                Ok(Self(arr))
            }
        }
    };
    (@impls $name:ident, $len:literal) => {

        impl $name {
            /// The byte length of this hash type.
            pub const LEN: usize = $len;

            /// All zeros.
            pub const ZERO: Self = Self([0u8; $len]);

            /// Creates from a byte array.
            #[must_use]
            pub const fn from_bytes(bytes: [u8; $len]) -> Self {
                Self(bytes)
            }

            /// Creates from a byte slice.
            ///
            /// # Errors
            ///
            /// Returns [`TypeError::InvalidHashLength`] if the slice length doesn't match.
            pub fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
                let bytes: [u8; $len] = slice.try_into().map_err(|_| TypeError::InvalidHashLength {
                    expected: $len,
                    actual: slice.len(),
                })?;
                Ok(Self(bytes))
            }

            /// Creates from a hex string (accepts uppercase or lowercase).
            ///
            /// # Errors
            ///
            /// Returns [`TypeError::InvalidHex`] if the string is not valid hex,
            /// or [`TypeError::InvalidHashLength`] if the decoded length doesn't match.
            pub fn from_hex(hex_str: &str) -> Result<Self, TypeError> {
                let bytes = hex::decode(hex_str)
                    .map_err(|e| TypeError::InvalidHex(e.to_string()))?;
                Self::from_slice(&bytes)
            }

            /// Returns the raw byte representation.
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; $len] {
                &self.0
            }

            /// Returns `true` if all bytes are zero.
            #[must_use]
            pub fn is_zero(&self) -> bool {
                self.0.iter().all(|&b| b == 0)
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}({})", stringify!($name), hex::encode_upper(self.0))
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", hex::encode_upper(self.0))
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl From<[u8; $len]> for $name {
            fn from(bytes: [u8; $len]) -> Self {
                Self(bytes)
            }
        }
    };
}

define_hash_type!(
    /// A 128-bit hash (16 bytes). XRPL type code 4.
    ///
    /// Used for `EmailHash` fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::Hash128;
    ///
    /// let hash = Hash128::from_hex("00000000000000000000000000000000").unwrap();
    /// assert!(hash.is_zero());
    /// ```
    Hash128,
    16,
    serde
);

define_hash_type!(
    /// A 160-bit hash (20 bytes). XRPL type code 17.
    ///
    /// Used for `TakerPaysCurrency`, `TakerPaysIssuer`, etc.
    Hash160,
    20,
    serde
);

define_hash_type!(
    /// A 256-bit hash (32 bytes). XRPL type code 5.
    ///
    /// The most common hash type, used for transaction hashes, ledger hashes,
    /// `PreviousTxnID`, `AccountTxnID`, `InvoiceID`, `Channel`, etc.
    ///
    /// # Examples
    ///
    /// Creating from a hex string:
    ///
    /// ```
    /// use xrpl_types::Hash256;
    ///
    /// let hash = Hash256::from_hex(
    ///     "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
    /// ).unwrap();
    /// assert_eq!(hash.as_bytes().len(), 32);
    /// assert!(!hash.is_zero());
    /// ```
    ///
    /// Display outputs uppercase hex:
    ///
    /// ```
    /// use xrpl_types::Hash256;
    ///
    /// let hash = Hash256::from_hex(
    ///     "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
    /// ).unwrap();
    /// assert_eq!(
    ///     format!("{hash}"),
    ///     "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
    /// );
    /// ```
    ///
    /// JSON round-trip:
    ///
    /// ```
    /// use xrpl_types::Hash256;
    ///
    /// let hash = Hash256::from_hex(
    ///     "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
    /// ).unwrap();
    /// let json = serde_json::to_string(&hash).unwrap();
    /// let decoded: Hash256 = serde_json::from_str(&json).unwrap();
    /// assert_eq!(decoded, hash);
    /// ```
    Hash256,
    32,
    serde
);

define_hash_type!(
    /// A 96-bit unsigned integer (12 bytes). XRPL type code 20.
    UInt96,
    12,
    serde
);

define_hash_type!(
    /// A 192-bit hash (24 bytes). XRPL type code 21.
    Hash192,
    24,
    serde
);

define_hash_type!(
    /// A 384-bit unsigned integer (48 bytes). XRPL type code 22.
    UInt384,
    48,
    serde
);

define_hash_type!(
    /// A 512-bit unsigned integer (64 bytes). XRPL type code 23.
    UInt512,
    64,
    serde
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash256_hex_round_trip() {
        let hex_str = "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9";
        let hash = Hash256::from_hex(hex_str).expect("should decode valid hex");
        assert_eq!(format!("{hash}"), hex_str);
    }

    #[test]
    fn hash256_json_round_trip() {
        let hex_str = "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9";
        let hash = Hash256::from_hex(hex_str).expect("should decode valid hex");
        let json = serde_json::to_string(&hash).expect("should serialize");
        assert_eq!(json, format!("\"{hex_str}\""));
        let decoded: Hash256 = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, hash);
    }

    #[test]
    fn hash256_accepts_lowercase_hex() {
        let lower = "c53ecf838647fa5a4c780377025fec7999ab4182590510ca461444b207ab74a9";
        let json = format!("\"{lower}\"");
        let hash: Hash256 = serde_json::from_str(&json).expect("should accept lowercase");
        assert_eq!(
            format!("{hash}"),
            "C53ECF838647FA5A4C780377025FEC7999AB4182590510CA461444B207AB74A9"
        );
    }

    #[test]
    fn hash128_json_round_trip() {
        let hex_str = "00000000000000000000000000000000";
        let hash = Hash128::from_hex(hex_str).expect("should decode");
        let json = serde_json::to_string(&hash).expect("should serialize");
        let decoded: Hash128 = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, hash);
    }
}

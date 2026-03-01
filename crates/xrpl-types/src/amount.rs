//! XRPL amount types.
//!
//! The XRPL protocol has three fundamentally different amount representations:
//! - [`XrpAmount`]: Native XRP in drops (1 XRP = 1,000,000 drops)
//! - [`IssuedAmount`]: Issued currency (IOU) with a custom floating-point value
//! - [`MptAmount`]: Multi-Purpose Token amount with an integer value
//!
//! The [`Amount`] enum unifies these for fields that accept any amount type.
//!
//! ## JSON representations
//!
//! - XRP: `"1000000"` (string of drops)
//! - Issued: `{"value": "1.5", "currency": "USD", "issuer": "rHb9..."}`
//! - MPT: `{"value": "100", "mpt_issuance_id": "0000..."}`

use crate::account::AccountId;
use crate::currency::{CurrencyCode, MptIssuanceId};
use crate::error::TypeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Maximum XRP supply in drops: 100 billion XRP = 10^17 drops.
pub const MAX_XRP_DROPS: u64 = 100_000_000_000_000_000;

/// Drops per XRP.
pub const DROPS_PER_XRP: u64 = 1_000_000;

/// An XRP amount in drops. Always non-negative.
///
/// 1 XRP = 1,000,000 drops. Maximum: 100 billion XRP (10^17 drops).
///
/// In JSON, serialized as a string of drops (e.g., `"1000000"` for 1 XRP).
///
/// # Examples
///
/// ```
/// use xrpl_types::XrpAmount;
///
/// let one_xrp = XrpAmount::from_drops(1_000_000).unwrap();
/// assert_eq!(one_xrp.drops(), 1_000_000);
/// assert_eq!(one_xrp, XrpAmount::ONE_XRP);
///
/// // Amounts exceeding the maximum supply are rejected:
/// assert!(XrpAmount::from_drops(100_000_000_000_000_001).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XrpAmount(u64);

impl XrpAmount {
    /// Zero drops.
    pub const ZERO: Self = Self(0);

    /// One drop (smallest unit).
    pub const ONE_DROP: Self = Self(1);

    /// One XRP (1,000,000 drops).
    pub const ONE_XRP: Self = Self(DROPS_PER_XRP);

    /// Creates an `XrpAmount` from drops.
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::XrpAmountOverflow`] if the value exceeds 10^17 drops.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::XrpAmount;
    ///
    /// let amount = XrpAmount::from_drops(5_000_000).unwrap(); // 5 XRP
    /// assert_eq!(amount.drops(), 5_000_000);
    /// ```
    pub const fn from_drops(drops: u64) -> Result<Self, TypeError> {
        if drops > MAX_XRP_DROPS {
            return Err(TypeError::XrpAmountOverflow(drops));
        }
        Ok(Self(drops))
    }

    /// Returns the amount in drops.
    #[must_use]
    pub const fn drops(&self) -> u64 {
        self.0
    }

    /// Returns `true` if the amount is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Serialize for XrpAmount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for XrpAmount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let drops: u64 = s.parse().map_err(serde::de::Error::custom)?;
        Self::from_drops(drops).map_err(serde::de::Error::custom)
    }
}

/// The minimum normalized mantissa for issued currency amounts: 10^15.
pub const MIN_IOU_MANTISSA: i64 = 1_000_000_000_000_000;

/// The maximum normalized mantissa for issued currency amounts: 10^16 - 1.
pub const MAX_IOU_MANTISSA: i64 = 9_999_999_999_999_999;

/// The minimum exponent for issued currency amounts.
pub const MIN_IOU_EXPONENT: i8 = -96;

/// The maximum exponent for issued currency amounts.
pub const MAX_IOU_EXPONENT: i8 = 80;

/// An issued currency value in XRPL's custom floating-point format.
///
/// Represents: `mantissa * 10^exponent`
///
/// The mantissa is normalized to [10^15, 10^16) for non-zero values,
/// matching the XRPL binary wire format exactly. This enables zero-cost
/// serialization.
///
/// In JSON, serialized as a decimal string (e.g., `"1.5"`, `"100"`, `"0"`).
///
/// # Examples
///
/// Creating from a decimal string (recommended):
///
/// ```
/// use xrpl_types::IssuedValue;
///
/// let value = IssuedValue::from_decimal_string("1.5").unwrap();
/// assert_eq!(value.to_decimal_string(), "1.5");
/// assert!(value.is_positive());
/// ```
///
/// Creating from normalized mantissa and exponent:
///
/// ```
/// use xrpl_types::IssuedValue;
///
/// // 100 = 1_000_000_000_000_000 * 10^-13
/// let value = IssuedValue::new(1_000_000_000_000_000, -13).unwrap();
/// assert_eq!(value.to_decimal_string(), "100");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IssuedValue {
    /// Signed mantissa. For non-zero values: |mantissa| is in [10^15, 10^16).
    /// Sign of the mantissa determines the sign of the amount.
    /// Zero is represented as mantissa=0, exponent=0.
    mantissa: i64,
    /// Exponent in the range [-96, 80]. Biased by 97 in wire format.
    exponent: i8,
}

impl IssuedValue {
    /// The zero value.
    pub const ZERO: Self = Self {
        mantissa: 0,
        exponent: 0,
    };

    /// Creates a new `IssuedValue` from a normalized mantissa and exponent.
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::InvalidMantissa`] if the absolute mantissa is not in [10^15, 10^16)
    /// for non-zero values, or [`TypeError::InvalidExponent`] if the exponent is outside [-96, 80].
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::IssuedValue;
    ///
    /// // 1.5 = 1_500_000_000_000_000 * 10^-15
    /// let value = IssuedValue::new(1_500_000_000_000_000, -15).unwrap();
    /// assert_eq!(value.mantissa(), 1_500_000_000_000_000);
    /// assert_eq!(value.exponent(), -15);
    ///
    /// // Zero mantissa always yields ZERO:
    /// let zero = IssuedValue::new(0, 0).unwrap();
    /// assert!(zero.is_zero());
    /// ```
    pub const fn new(mantissa: i64, exponent: i8) -> Result<Self, TypeError> {
        if mantissa == 0 {
            return Ok(Self::ZERO);
        }

        let abs_mantissa = mantissa.unsigned_abs() as i64;
        if abs_mantissa < MIN_IOU_MANTISSA || abs_mantissa > MAX_IOU_MANTISSA {
            return Err(TypeError::InvalidMantissa(mantissa));
        }
        if exponent < MIN_IOU_EXPONENT || exponent > MAX_IOU_EXPONENT {
            return Err(TypeError::InvalidExponent(exponent));
        }

        Ok(Self { mantissa, exponent })
    }

    /// Returns the mantissa.
    #[must_use]
    pub const fn mantissa(&self) -> i64 {
        self.mantissa
    }

    /// Returns the exponent.
    #[must_use]
    pub const fn exponent(&self) -> i8 {
        self.exponent
    }

    /// Returns `true` if the value is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.mantissa == 0
    }

    /// Returns `true` if the value is positive (non-zero).
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.mantissa > 0
    }

    /// Returns `true` if the value is negative.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.mantissa < 0
    }

    /// Converts to a decimal string representation.
    ///
    /// Produces the minimal string representation without trailing zeros.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::IssuedValue;
    ///
    /// assert_eq!(IssuedValue::ZERO.to_decimal_string(), "0");
    ///
    /// let val = IssuedValue::from_decimal_string("-0.001").unwrap();
    /// assert_eq!(val.to_decimal_string(), "-0.001");
    /// ```
    #[must_use]
    pub fn to_decimal_string(&self) -> String {
        if self.mantissa == 0 {
            return "0".to_string();
        }

        let negative = self.mantissa < 0;
        let abs_mantissa = self.mantissa.unsigned_abs();

        // Strip trailing zeros from the mantissa to get the significant digits
        let mut sig = abs_mantissa;
        let mut trailing_zeros: i32 = 0;
        while sig % 10 == 0 && sig > 0 {
            sig /= 10;
            trailing_zeros += 1;
        }

        let sig_str = sig.to_string();
        let sig_len = sig_str.len() as i32;

        // The effective exponent after stripping trailing zeros
        let eff_exp = self.exponent as i32 + trailing_zeros;

        // Position of the decimal point relative to the significant digits:
        // If eff_exp >= 0, the number is an integer (sig * 10^eff_exp)
        // If -eff_exp < sig_len, the decimal point falls within the digits
        // If -eff_exp >= sig_len, we need leading zeros after "0."
        let result = if eff_exp >= 0 {
            // Integer: append zeros
            let mut s = sig_str;
            for _ in 0..eff_exp {
                s.push('0');
            }
            s
        } else {
            let decimal_places = (-eff_exp) as usize;
            if decimal_places < sig_len as usize {
                // Decimal point falls within the digits
                let int_digits = sig_len as usize - decimal_places;
                let mut s = String::with_capacity(sig_len as usize + 1);
                s.push_str(&sig_str[..int_digits]);
                s.push('.');
                s.push_str(&sig_str[int_digits..]);
                s
            } else {
                // Need leading zeros: "0.000...digits"
                let leading_zeros = decimal_places - sig_len as usize;
                let mut s = String::with_capacity(2 + leading_zeros + sig_len as usize);
                s.push_str("0.");
                for _ in 0..leading_zeros {
                    s.push('0');
                }
                s.push_str(&sig_str);
                s
            }
        };

        if negative {
            format!("-{result}")
        } else {
            result
        }
    }

    /// Parses from a decimal string.
    ///
    /// # Errors
    ///
    /// Returns [`TypeError::InvalidMantissa`] if the value cannot be represented.
    ///
    /// # Examples
    ///
    /// ```
    /// use xrpl_types::IssuedValue;
    ///
    /// let value = IssuedValue::from_decimal_string("100").unwrap();
    /// assert_eq!(value.to_decimal_string(), "100");
    ///
    /// let negative = IssuedValue::from_decimal_string("-1.5").unwrap();
    /// assert!(negative.is_negative());
    /// ```
    pub fn from_decimal_string(s: &str) -> Result<Self, TypeError> {
        let s = s.trim();

        if s == "0" || s == "0.0" || s == "-0" || s == "0." {
            return Ok(Self::ZERO);
        }

        let negative = s.starts_with('-');
        let s = if negative { &s[1..] } else { s };

        // Split into integer and fractional parts
        let (int_part, frac_part) = if let Some(dot_pos) = s.find('.') {
            (&s[..dot_pos], &s[dot_pos + 1..])
        } else {
            (s, "")
        };

        // Build the full digit string (integer + fractional, no decimal point)
        let mut digits = String::with_capacity(int_part.len() + frac_part.len());
        digits.push_str(int_part);
        digits.push_str(frac_part);

        // Remove leading zeros
        let digits = digits.trim_start_matches('0');
        if digits.is_empty() {
            return Ok(Self::ZERO);
        }

        // The exponent is negative the number of fractional digits
        let raw_exponent = -(frac_part.len() as i64);

        // Parse the digits as a u64
        let raw_mantissa: u64 = digits
            .parse()
            .map_err(|_| TypeError::InvalidMantissa(0))?;

        if raw_mantissa == 0 {
            return Ok(Self::ZERO);
        }

        // Normalize: mantissa must be in [10^15, 10^16)
        let mut mantissa = raw_mantissa;
        let mut exponent = raw_exponent;

        // Scale up if too small
        while mantissa < MIN_IOU_MANTISSA as u64 && exponent > MIN_IOU_EXPONENT as i64 - 15 {
            mantissa *= 10;
            exponent -= 1;
        }

        // Scale down if too large
        while mantissa >= 10 * MIN_IOU_MANTISSA as u64 {
            // Round: check if we need to round up
            let remainder = mantissa % 10;
            mantissa /= 10;
            if remainder >= 5 {
                mantissa += 1;
            }
            exponent += 1;
        }

        let signed_mantissa = if negative {
            -(mantissa as i64)
        } else {
            mantissa as i64
        };

        let exponent_i8 = i8::try_from(exponent).map_err(|_| TypeError::InvalidExponent(0))?;

        Self::new(signed_mantissa, exponent_i8)
    }
}

impl Serialize for IssuedValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_decimal_string())
    }
}

impl<'de> Deserialize<'de> for IssuedValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_decimal_string(&s).map_err(serde::de::Error::custom)
    }
}

/// An issued currency (IOU/trustline token) amount.
///
/// Combines a value with a currency code and issuer account.
///
/// In JSON: `{"value": "1.5", "currency": "USD", "issuer": "rHb9..."}`
///
/// # Examples
///
/// ```
/// use xrpl_types::{AccountId, CurrencyCode, IssuedAmount, IssuedValue};
///
/// let issuer: AccountId = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse().unwrap();
/// let amount = IssuedAmount {
///     value: IssuedValue::from_decimal_string("1.5").unwrap(),
///     currency: CurrencyCode::from_ascii("USD").unwrap(),
///     issuer,
/// };
///
/// let json = serde_json::to_string(&amount).unwrap();
/// assert!(json.contains("\"value\":\"1.5\""));
/// assert!(json.contains("\"currency\":\"USD\""));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct IssuedAmount {
    /// The amount value.
    pub value: IssuedValue,
    /// The currency code (3-char standard or 20-byte non-standard).
    pub currency: CurrencyCode,
    /// The issuer's account ID.
    pub issuer: AccountId,
}

impl Serialize for IssuedAmount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("value", &self.value.to_decimal_string())?;
        map.serialize_entry("currency", &self.currency)?;
        map.serialize_entry("issuer", &self.issuer)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for IssuedAmount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            value: String,
            currency: CurrencyCode,
            issuer: AccountId,
        }
        let h = Helper::deserialize(deserializer)?;
        let value =
            IssuedValue::from_decimal_string(&h.value).map_err(serde::de::Error::custom)?;
        Ok(IssuedAmount {
            value,
            currency: h.currency,
            issuer: h.issuer,
        })
    }
}

/// A Multi-Purpose Token (MPT) amount.
///
/// MPT amounts are simple signed integers combined with the issuance identifier.
///
/// In JSON: `{"value": "100", "mpt_issuance_id": "0000..."}`
///
/// # Examples
///
/// ```
/// use xrpl_types::{MptAmount, MptIssuanceId};
///
/// let amount = MptAmount {
///     value: 1000,
///     mpt_issuance_id: MptIssuanceId::from_bytes([0xAB; 24]),
/// };
///
/// let json = serde_json::to_string(&amount).unwrap();
/// assert!(json.contains("\"value\":\"1000\""));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MptAmount {
    /// The token amount (signed 64-bit integer).
    pub value: i64,
    /// The unique identifier for the MPT issuance (192 bits).
    pub mpt_issuance_id: MptIssuanceId,
}

impl Serialize for MptAmount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("value", &self.value.to_string())?;
        map.serialize_entry("mpt_issuance_id", &self.mpt_issuance_id)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for MptAmount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            value: String,
            mpt_issuance_id: MptIssuanceId,
        }
        let h = Helper::deserialize(deserializer)?;
        let value: i64 = h.value.parse().map_err(serde::de::Error::custom)?;
        Ok(MptAmount {
            value,
            mpt_issuance_id: h.mpt_issuance_id,
        })
    }
}

/// An amount that can be XRP, an issued currency, or an MPT.
///
/// This is used for transaction fields like `Amount`, `Fee`, `SendMax`, etc.
/// The XRPL protocol distinguishes these by their binary encoding format.
///
/// JSON representations:
/// - XRP: `"1000000"` (string of drops)
/// - Issued: `{"value": "1.5", "currency": "USD", "issuer": "rHb9..."}`
/// - MPT: `{"value": "100", "mpt_issuance_id": "0000..."}`
///
/// # Examples
///
/// Converting from specific types using `From`:
///
/// ```
/// use xrpl_types::{Amount, XrpAmount};
///
/// let xrp = XrpAmount::from_drops(1_000_000).unwrap();
/// let amount: Amount = xrp.into();
/// assert!(matches!(amount, Amount::Xrp(_)));
/// ```
///
/// Deserializing from JSON auto-detects the variant:
///
/// ```
/// use xrpl_types::Amount;
///
/// // XRP amounts are strings of drops:
/// let xrp: Amount = serde_json::from_str("\"1000000\"").unwrap();
/// assert!(matches!(xrp, Amount::Xrp(_)));
///
/// // Issued amounts are objects with "currency":
/// let json = r#"{"value":"1.5","currency":"USD","issuer":"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"}"#;
/// let iou: Amount = serde_json::from_str(json).unwrap();
/// assert!(matches!(iou, Amount::Issued(_)));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Amount {
    /// Native XRP amount in drops.
    Xrp(XrpAmount),
    /// Issued currency (IOU/trustline token) amount.
    Issued(IssuedAmount),
    /// Multi-Purpose Token amount.
    Mpt(MptAmount),
}

impl Serialize for Amount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Amount::Xrp(xrp) => xrp.serialize(serializer),
            Amount::Issued(issued) => issued.serialize(serializer),
            Amount::Mpt(mpt) => mpt.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;

        match &value {
            serde_json::Value::String(s) => {
                // XRP: string of drops
                let drops: u64 = s.parse().map_err(serde::de::Error::custom)?;
                let xrp = XrpAmount::from_drops(drops).map_err(serde::de::Error::custom)?;
                Ok(Amount::Xrp(xrp))
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("mpt_issuance_id") {
                    // MPT amount
                    let mpt: MptAmount =
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                    Ok(Amount::Mpt(mpt))
                } else if map.contains_key("currency") {
                    // Issued amount
                    let issued: IssuedAmount =
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                    Ok(Amount::Issued(issued))
                } else {
                    Err(serde::de::Error::custom(
                        "amount object must have 'currency' or 'mpt_issuance_id'",
                    ))
                }
            }
            _ => Err(serde::de::Error::custom(
                "amount must be a string (XRP) or object (issued/MPT)",
            )),
        }
    }
}

impl From<XrpAmount> for Amount {
    fn from(xrp: XrpAmount) -> Self {
        Self::Xrp(xrp)
    }
}

impl From<IssuedAmount> for Amount {
    fn from(issued: IssuedAmount) -> Self {
        Self::Issued(issued)
    }
}

impl From<MptAmount> for Amount {
    fn from(mpt: MptAmount) -> Self {
        Self::Mpt(mpt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- XrpAmount tests ---

    #[test]
    fn xrp_amount_json_round_trip() {
        let amt = XrpAmount::from_drops(1_000_000).expect("valid");
        let json = serde_json::to_string(&amt).expect("should serialize");
        assert_eq!(json, "\"1000000\"");
        let decoded: XrpAmount = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, amt);
    }

    #[test]
    fn xrp_amount_zero() {
        let json = serde_json::to_string(&XrpAmount::ZERO).expect("should serialize");
        assert_eq!(json, "\"0\"");
    }

    // --- IssuedValue tests ---

    #[test]
    fn issued_value_to_decimal_zero() {
        assert_eq!(IssuedValue::ZERO.to_decimal_string(), "0");
    }

    #[test]
    fn issued_value_to_decimal_one_point_five() {
        // 1.5 = 1500000000000000 * 10^-15
        let val = IssuedValue::new(1_500_000_000_000_000, -15).expect("valid");
        assert_eq!(val.to_decimal_string(), "1.5");
    }

    #[test]
    fn issued_value_to_decimal_hundred() {
        // 100 = 1000000000000000 * 10^-13
        let val = IssuedValue::new(1_000_000_000_000_000, -13).expect("valid");
        assert_eq!(val.to_decimal_string(), "100");
    }

    #[test]
    fn issued_value_to_decimal_small() {
        // 0.001 = 1000000000000000 * 10^-18
        let val = IssuedValue::new(1_000_000_000_000_000, -18).expect("valid");
        assert_eq!(val.to_decimal_string(), "0.001");
    }

    #[test]
    fn issued_value_to_decimal_negative() {
        let val = IssuedValue::new(-1_500_000_000_000_000, -15).expect("valid");
        assert_eq!(val.to_decimal_string(), "-1.5");
    }

    #[test]
    fn issued_value_from_decimal_round_trip() {
        let test_cases = ["0", "1.5", "-1.5", "100", "0.001", "-0.001"];
        for s in &test_cases {
            let val = IssuedValue::from_decimal_string(s).expect(&format!("should parse '{s}'"));
            let rendered = val.to_decimal_string();
            assert_eq!(&rendered, s, "round-trip failed for '{s}'");
        }
    }

    #[test]
    fn issued_value_json_round_trip() {
        let val = IssuedValue::new(1_500_000_000_000_000, -15).expect("valid");
        let json = serde_json::to_string(&val).expect("should serialize");
        assert_eq!(json, "\"1.5\"");
        let decoded: IssuedValue = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, val);
    }

    // --- IssuedAmount tests ---

    #[test]
    fn issued_amount_json_round_trip() {
        let issuer =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").expect("valid");
        let amt = IssuedAmount {
            value: IssuedValue::new(1_500_000_000_000_000, -15).expect("valid"),
            currency: CurrencyCode::from_ascii("USD").expect("valid"),
            issuer,
        };
        let json = serde_json::to_string(&amt).expect("should serialize");
        assert!(json.contains("\"value\":\"1.5\""));
        assert!(json.contains("\"currency\":\"USD\""));
        assert!(json.contains("\"issuer\":\"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh\""));

        let decoded: IssuedAmount = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, amt);
    }

    // --- MptAmount tests ---

    #[test]
    fn mpt_amount_json_round_trip() {
        let amt = MptAmount {
            value: 1000,
            mpt_issuance_id: MptIssuanceId::from_bytes([0xAB; 24]),
        };
        let json = serde_json::to_string(&amt).expect("should serialize");
        assert!(json.contains("\"value\":\"1000\""));
        assert!(json.contains("\"mpt_issuance_id\""));

        let decoded: MptAmount = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, amt);
    }

    // --- Amount enum tests ---

    #[test]
    fn amount_xrp_json_round_trip() {
        let amt = Amount::Xrp(XrpAmount::from_drops(1_000_000).expect("valid"));
        let json = serde_json::to_string(&amt).expect("should serialize");
        assert_eq!(json, "\"1000000\"");
        let decoded: Amount = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, amt);
    }

    #[test]
    fn amount_issued_json_round_trip() {
        let issuer =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").expect("valid");
        let amt = Amount::Issued(IssuedAmount {
            value: IssuedValue::new(1_500_000_000_000_000, -15).expect("valid"),
            currency: CurrencyCode::from_ascii("USD").expect("valid"),
            issuer,
        });
        let json = serde_json::to_string(&amt).expect("should serialize");
        let decoded: Amount = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, amt);
    }

    #[test]
    fn amount_mpt_json_round_trip() {
        let amt = Amount::Mpt(MptAmount {
            value: 500,
            mpt_issuance_id: MptIssuanceId::from_bytes([0xCD; 24]),
        });
        let json = serde_json::to_string(&amt).expect("should serialize");
        let decoded: Amount = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(decoded, amt);
    }

    #[test]
    fn amount_discriminates_by_json_shape() {
        // XRP is a string
        let xrp: Amount = serde_json::from_str("\"1000000\"").expect("should parse XRP");
        assert!(matches!(xrp, Amount::Xrp(_)));

        // IOU is an object with currency
        let iou_json = r#"{"value":"1.5","currency":"USD","issuer":"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"}"#;
        let iou: Amount = serde_json::from_str(iou_json).expect("should parse IOU");
        assert!(matches!(iou, Amount::Issued(_)));

        // MPT is an object with mpt_issuance_id
        // 48 hex chars = 24 bytes
        let mpt_json = r#"{"value":"100","mpt_issuance_id":"CDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCDCD"}"#;
        let mpt: Amount = serde_json::from_str(mpt_json).expect("should parse MPT");
        assert!(matches!(mpt, Amount::Mpt(_)));
    }
}

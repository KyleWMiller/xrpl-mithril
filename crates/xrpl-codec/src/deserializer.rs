//! Binary deserialization for XRPL objects.
//!
//! Parses canonical XRPL binary format back into `serde_json::Map` representations.
//! From there, typed Rust structs can be obtained via `serde_json::from_value`.

use crate::definitions;
use crate::error::CodecError;
use crate::field_code::{decode_field_id, decode_vl_length};

/// Object end marker: 0xE1
const OBJECT_END_MARKER_TC: u16 = 14;
const OBJECT_END_MARKER_FC: u16 = 1;

/// Array end marker: 0xF1
const ARRAY_END_MARKER_TC: u16 = 15;
const ARRAY_END_MARKER_FC: u16 = 1;

/// Deserialize a binary XRPL object (transaction or ledger entry) into a JSON map.
///
/// The binary data should be the canonical encoding of the outer object
/// (no surrounding field header or end marker for the root object).
///
/// # Errors
///
/// Returns [`CodecError`] if the binary data is malformed.
pub fn deserialize_object(data: &[u8]) -> Result<serde_json::Map<String, serde_json::Value>, CodecError> {
    let (map, _consumed) = deserialize_fields(data, false)?;
    Ok(map)
}

/// Deserialize fields from binary data, optionally stopping at an end marker.
///
/// Returns the parsed map and the number of bytes consumed.
fn deserialize_fields(
    data: &[u8],
    expect_end_marker: bool,
) -> Result<(serde_json::Map<String, serde_json::Value>, usize), CodecError> {
    let mut map = serde_json::Map::new();
    let mut pos = 0;

    while pos < data.len() {
        // Decode field ID
        let (type_code, field_code, header_len) = decode_field_id(&data[pos..])?;
        pos += header_len;

        // Check for end markers
        if type_code == OBJECT_END_MARKER_TC && field_code == OBJECT_END_MARKER_FC {
            if expect_end_marker {
                return Ok((map, pos));
            }
            // Unexpected end marker in root object -- stop parsing
            return Ok((map, pos));
        }
        if type_code == ARRAY_END_MARKER_TC && field_code == ARRAY_END_MARKER_FC {
            if expect_end_marker {
                return Ok((map, pos));
            }
            return Ok((map, pos));
        }

        // Look up field definition
        let field_def = definitions::field_by_code(type_code, field_code);
        let field_name = field_def
            .map(|d| d.name)
            .unwrap_or("unknown");

        // Deserialize value based on type code
        let (value, consumed) = deserialize_value(type_code, &data[pos..])?;
        pos += consumed;

        if field_name != "unknown" {
            // Convert TransactionType and LedgerEntryType numeric codes to string names
            let value = match field_name {
                "TransactionType" => {
                    if let Some(code) = value.as_u64() {
                        if let Some(name) = definitions::tx_type_name(code as u16) {
                            serde_json::Value::String(name.to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                _ => value,
            };
            map.insert(field_name.to_string(), value);
        }
    }

    if expect_end_marker {
        return Err(CodecError::Deserialization(
            "expected end marker but reached end of data".into(),
        ));
    }

    Ok((map, pos))
}

/// Deserialize a value based on its XRPL type code.
///
/// Returns the JSON value and the number of bytes consumed.
fn deserialize_value(
    type_code: u16,
    data: &[u8],
) -> Result<(serde_json::Value, usize), CodecError> {
    match type_code {
        // UInt8 (16)
        16 => deserialize_uint8(data),
        // UInt16 (1)
        1 => deserialize_uint16(data),
        // UInt32 (2)
        2 => deserialize_uint32(data),
        // UInt64 (3)
        3 => deserialize_uint64(data),
        // Hash128 (4)
        4 => deserialize_fixed_hash(data, 16),
        // Hash256 (5)
        5 => deserialize_fixed_hash(data, 32),
        // Amount (6)
        6 => deserialize_amount(data),
        // Blob (7)
        7 => deserialize_blob(data),
        // AccountID (8)
        8 => deserialize_account_id(data),
        // Number (9) - XFL format, treat as UInt64
        9 => deserialize_uint64(data),
        // Int32 (10)
        10 => deserialize_int32(data),
        // Int64 (11)
        11 => deserialize_int64(data),
        // STObject (14)
        14 => deserialize_st_object(data),
        // STArray (15)
        15 => deserialize_st_array(data),
        // Hash160 (17)
        17 => deserialize_fixed_hash(data, 20),
        // PathSet (18)
        18 => deserialize_pathset(data),
        // Vector256 (19)
        19 => deserialize_vector256(data),
        // UInt96 (20)
        20 => deserialize_fixed_hash(data, 12),
        // Hash192 (21)
        21 => deserialize_fixed_hash(data, 24),
        // UInt384 (22)
        22 => deserialize_fixed_hash(data, 48),
        // UInt512 (23)
        23 => deserialize_fixed_hash(data, 64),
        // Issue (24)
        24 => deserialize_issue(data),
        // XChainBridge (25)
        25 => deserialize_xchain_bridge(data),
        // Currency (26)
        26 => deserialize_fixed_hash(data, 20),
        _ => Err(CodecError::Deserialization(format!(
            "unknown type code: {type_code}"
        ))),
    }
}

fn ensure_bytes(data: &[u8], needed: usize) -> Result<(), CodecError> {
    if data.len() < needed {
        return Err(CodecError::UnexpectedEnd {
            needed,
            available: data.len(),
        });
    }
    Ok(())
}

fn deserialize_uint8(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 1)?;
    Ok((serde_json::Value::Number(data[0].into()), 1))
}

fn deserialize_uint16(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 2)?;
    let val = u16::from_be_bytes([data[0], data[1]]);
    Ok((serde_json::Value::Number(val.into()), 2))
}

fn deserialize_uint32(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 4)?;
    let val = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    Ok((serde_json::Value::Number(val.into()), 4))
}

fn deserialize_uint64(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 8)?;
    let val = u64::from_be_bytes(data[..8].try_into().map_err(|_| CodecError::UnexpectedEnd {
        needed: 8,
        available: data.len(),
    })?);
    // UInt64 values in JSON are strings (to avoid precision loss)
    Ok((serde_json::Value::String(val.to_string()), 8))
}

fn deserialize_int32(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 4)?;
    let val = i32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    Ok((serde_json::Value::Number(val.into()), 4))
}

fn deserialize_int64(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 8)?;
    let val = i64::from_be_bytes(data[..8].try_into().map_err(|_| CodecError::UnexpectedEnd {
        needed: 8,
        available: data.len(),
    })?);
    Ok((serde_json::Value::String(val.to_string()), 8))
}

fn deserialize_fixed_hash(
    data: &[u8],
    len: usize,
) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, len)?;
    let hex_str = hex::encode_upper(&data[..len]);
    Ok((serde_json::Value::String(hex_str), len))
}

fn deserialize_blob(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let (vl_len, vl_consumed) = decode_vl_length(data)?;
    let total = vl_consumed + vl_len;
    ensure_bytes(data, total)?;
    let hex_str = hex::encode_upper(&data[vl_consumed..total]);
    Ok((serde_json::Value::String(hex_str), total))
}

fn deserialize_account_id(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let (vl_len, vl_consumed) = decode_vl_length(data)?;
    if vl_len != 20 {
        return Err(CodecError::Deserialization(format!(
            "AccountID must be 20 bytes, got {vl_len}"
        )));
    }
    let total = vl_consumed + 20;
    ensure_bytes(data, total)?;

    let account_bytes = &data[vl_consumed..total];
    let address = bs58::encode(account_bytes)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check_version(0)
        .into_string();

    Ok((serde_json::Value::String(address), total))
}

/// Deserialize an Amount (polymorphic: XRP, IOU, or MPT).
///
/// The first byte determines the variant:
/// - bit 7=1 → IOU (48 bytes: 8 value + 20 currency + 20 issuer)
/// - bit 7=0, bit 5=1 → MPT (33 bytes: 1 flags + 8 value + 24 issuance ID)
/// - bit 7=0, bit 5=0 → XRP (8 bytes)
fn deserialize_amount(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 1)?;

    let first_byte = data[0];
    let is_iou = (first_byte & 0x80) != 0; // bit 7 = 1 means IOU
    let is_mpt = (first_byte & 0x20) != 0; // bit 5 = 1 means MPT

    if is_iou {
        // IOU: 48 bytes total (8 value + 20 currency + 20 issuer)
        ensure_bytes(data, 48)?;

        let raw = u64::from_be_bytes(data[..8].try_into().map_err(|_| {
            CodecError::UnexpectedEnd {
                needed: 8,
                available: data.len(),
            }
        })?);

        // Decode custom float
        let value_str = decode_iou_value(raw)?;

        // Currency: bytes 8-27
        let currency_str = decode_currency_code(&data[8..28]);

        // Issuer: bytes 28-47
        let issuer_address = bs58::encode(&data[28..48])
            .with_alphabet(bs58::Alphabet::RIPPLE)
            .with_check_version(0)
            .into_string();

        let mut map = serde_json::Map::new();
        map.insert("value".to_string(), serde_json::Value::String(value_str));
        map.insert("currency".to_string(), serde_json::Value::String(currency_str));
        map.insert("issuer".to_string(), serde_json::Value::String(issuer_address));

        Ok((serde_json::Value::Object(map), 48))
    } else if is_mpt {
        // MPT: 33 bytes total (1 flags + 8 value + 24 MptIssuanceId)
        // Layout matches rippled STAmount: add8(flags) + add64(value) + addBitString(mptID)
        ensure_bytes(data, 33)?;

        let positive = (first_byte & 0x40) != 0; // bit 6 = sign

        // Reconstruct value the same way rippled does:
        // rippled reads 8 bytes (flags + 7 high value bytes), then 1 more byte,
        // and combines: mValue = (initial_u64 << 8) | extra_byte
        // This shifts the flags byte out and gives the full 64-bit value.
        let initial = u64::from_be_bytes(data[..8].try_into().map_err(|_| {
            CodecError::UnexpectedEnd {
                needed: 8,
                available: data.len(),
            }
        })?);
        let abs_value = (initial << 8) | (data[8] as u64);

        let value: i64 = if positive {
            abs_value as i64
        } else {
            -(abs_value as i64)
        };

        let mpt_id_hex = hex::encode_upper(&data[9..33]);

        let mut map = serde_json::Map::new();
        map.insert("value".to_string(), serde_json::Value::String(value.to_string()));
        map.insert("mpt_issuance_id".to_string(), serde_json::Value::String(mpt_id_hex));

        Ok((serde_json::Value::Object(map), 33))
    } else {
        // XRP: 8 bytes total
        ensure_bytes(data, 8)?;

        let raw = u64::from_be_bytes(data[..8].try_into().map_err(|_| {
            CodecError::UnexpectedEnd {
                needed: 8,
                available: data.len(),
            }
        })?);

        let positive = (raw & 0x4000_0000_0000_0000) != 0; // bit 62
        let drops = raw & 0x3FFF_FFFF_FFFF_FFFF; // bits 61-0

        let drops_str = if positive {
            drops.to_string()
        } else {
            format!("-{drops}")
        };

        Ok((serde_json::Value::String(drops_str), 8))
    }
}

/// Decode an IOU value from the 8-byte custom float format.
fn decode_iou_value(raw: u64) -> Result<String, CodecError> {
    // Check for zero: bit 63=1 and rest all zeros (except bit 63)
    if raw == 0x8000_0000_0000_0000 {
        return Ok("0".to_string());
    }

    let positive = (raw & 0x4000_0000_0000_0000) != 0;
    let biased_exp = ((raw >> 54) & 0xFF) as i16;
    let mantissa = raw & 0x003F_FFFF_FFFF_FFFF;

    let exponent = biased_exp - 97;

    // Use IssuedValue to convert to decimal string
    let signed_mantissa = if positive {
        mantissa as i64
    } else {
        -(mantissa as i64)
    };

    let issued_value = xrpl_types::IssuedValue::new(signed_mantissa, exponent as i8)
        .map_err(|e| CodecError::InvalidAmount(format!("invalid IOU value: {e}")))?;

    Ok(issued_value.to_decimal_string())
}

/// Decode a 20-byte currency code to its string representation.
fn decode_currency_code(bytes: &[u8]) -> String {
    if bytes.iter().all(|&b| b == 0) {
        return "XRP".to_string();
    }

    // Check if standard (byte 0 == 0x00)
    if bytes[0] == 0x00 {
        let ascii = [bytes[12], bytes[13], bytes[14]];
        if let Ok(s) = core::str::from_utf8(&ascii) {
            if s.chars().all(|c| c.is_ascii_alphanumeric()) {
                return s.to_string();
            }
        }
    }

    // Non-standard: return as hex
    hex::encode_upper(bytes)
}

fn deserialize_st_object(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let (map, consumed) = deserialize_fields(data, true)?;
    Ok((serde_json::Value::Object(map), consumed))
}

fn deserialize_st_array(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let mut arr = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Read the element wrapper field header
        let (type_code, field_code, header_len) = decode_field_id(&data[pos..])?;
        pos += header_len;

        // Check for array end marker
        if type_code == ARRAY_END_MARKER_TC && field_code == ARRAY_END_MARKER_FC {
            return Ok((serde_json::Value::Array(arr), pos));
        }

        // The wrapper field name
        let wrapper_name = definitions::field_by_code(type_code, field_code)
            .map(|d| d.name)
            .unwrap_or("unknown");

        // Deserialize inner fields until object end marker
        let (inner_map, consumed) = deserialize_fields(&data[pos..], true)?;
        pos += consumed;

        // Wrap in the element name
        let mut wrapper = serde_json::Map::new();
        wrapper.insert(wrapper_name.to_string(), serde_json::Value::Object(inner_map));
        arr.push(serde_json::Value::Object(wrapper));
    }

    Err(CodecError::Deserialization(
        "expected array end marker but reached end of data".into(),
    ))
}

fn deserialize_pathset(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let mut paths: Vec<serde_json::Value> = Vec::new();
    let mut current_path: Vec<serde_json::Value> = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        let type_byte = data[pos];

        if type_byte == 0x00 {
            // PathSet end
            pos += 1;
            if !current_path.is_empty() {
                paths.push(serde_json::Value::Array(current_path));
            }
            return Ok((serde_json::Value::Array(paths), pos));
        }

        if type_byte == 0xFF {
            // Path boundary
            pos += 1;
            paths.push(serde_json::Value::Array(current_path));
            current_path = Vec::new();
            continue;
        }

        // Path step
        pos += 1; // consume type byte
        let mut step = serde_json::Map::new();

        if type_byte & 0x01 != 0 {
            // Account: 20 bytes
            ensure_bytes(&data[pos..], 20)?;
            let address = bs58::encode(&data[pos..pos + 20])
                .with_alphabet(bs58::Alphabet::RIPPLE)
                .with_check_version(0)
                .into_string();
            step.insert("account".to_string(), serde_json::Value::String(address));
            pos += 20;
        }
        if type_byte & 0x10 != 0 {
            // Currency: 20 bytes
            ensure_bytes(&data[pos..], 20)?;
            let currency = decode_currency_code(&data[pos..pos + 20]);
            step.insert("currency".to_string(), serde_json::Value::String(currency));
            pos += 20;
        }
        if type_byte & 0x20 != 0 {
            // Issuer: 20 bytes
            ensure_bytes(&data[pos..], 20)?;
            let address = bs58::encode(&data[pos..pos + 20])
                .with_alphabet(bs58::Alphabet::RIPPLE)
                .with_check_version(0)
                .into_string();
            step.insert("issuer".to_string(), serde_json::Value::String(address));
            pos += 20;
        }

        current_path.push(serde_json::Value::Object(step));
    }

    Err(CodecError::Deserialization(
        "expected PathSet end marker but reached end of data".into(),
    ))
}

fn deserialize_vector256(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let (vl_len, vl_consumed) = decode_vl_length(data)?;
    let total = vl_consumed + vl_len;
    ensure_bytes(data, total)?;

    if vl_len % 32 != 0 {
        return Err(CodecError::Deserialization(format!(
            "Vector256 length {vl_len} is not a multiple of 32"
        )));
    }

    let count = vl_len / 32;
    let mut arr = Vec::with_capacity(count);
    for i in 0..count {
        let start = vl_consumed + i * 32;
        let hex_str = hex::encode_upper(&data[start..start + 32]);
        arr.push(serde_json::Value::String(hex_str));
    }

    Ok((serde_json::Value::Array(arr), total))
}

fn deserialize_issue(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    ensure_bytes(data, 20)?;

    // Check if XRP (all zeros)
    if data[..20].iter().all(|&b| b == 0) {
        let mut map = serde_json::Map::new();
        map.insert("currency".to_string(), serde_json::Value::String("XRP".to_string()));
        return Ok((serde_json::Value::Object(map), 20));
    }

    // Check if this is an MPT issue (we need to detect this somehow)
    // For now, treat as IOU: currency (20 bytes) + issuer (20 bytes)
    ensure_bytes(data, 40)?;
    let currency = decode_currency_code(&data[..20]);
    let issuer = bs58::encode(&data[20..40])
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check_version(0)
        .into_string();

    let mut map = serde_json::Map::new();
    map.insert("currency".to_string(), serde_json::Value::String(currency));
    map.insert("issuer".to_string(), serde_json::Value::String(issuer));

    Ok((serde_json::Value::Object(map), 40))
}

fn deserialize_xchain_bridge(data: &[u8]) -> Result<(serde_json::Value, usize), CodecError> {
    let mut map = serde_json::Map::new();
    let mut pos = 0;

    // LockingChainDoor: VL-encoded AccountID
    let (door_val, consumed) = deserialize_account_id(&data[pos..])?;
    map.insert("LockingChainDoor".to_string(), door_val);
    pos += consumed;

    // LockingChainIssue: Issue type
    let (issue_val, consumed) = deserialize_issue(&data[pos..])?;
    map.insert("LockingChainIssue".to_string(), issue_val);
    pos += consumed;

    // IssuingChainDoor: VL-encoded AccountID
    let (door_val, consumed) = deserialize_account_id(&data[pos..])?;
    map.insert("IssuingChainDoor".to_string(), door_val);
    pos += consumed;

    // IssuingChainIssue: Issue type
    let (issue_val, consumed) = deserialize_issue(&data[pos..])?;
    map.insert("IssuingChainIssue".to_string(), issue_val);
    pos += consumed;

    Ok((serde_json::Value::Object(map), pos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializer;

    #[test]
    fn round_trip_simple_fields() {
        let tx = serde_json::json!({
            "TransactionType": 0u16,
            "Flags": 0u32,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "Amount": "1000000"
        });

        let map = tx.as_object().expect("should be object");
        let mut buf = Vec::new();
        serializer::serialize_json_object(map, &mut buf, false).expect("should serialize");

        let decoded = deserialize_object(&buf).expect("should deserialize");

        // Verify key fields round-trip
        assert_eq!(decoded.get("TransactionType").and_then(|v| v.as_str()), Some("Payment"));
        assert_eq!(decoded.get("Sequence").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(decoded.get("Fee").and_then(|v| v.as_str()), Some("12"));
        assert_eq!(
            decoded.get("Account").and_then(|v| v.as_str()),
            Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
        );
        assert_eq!(
            decoded.get("Amount").and_then(|v| v.as_str()),
            Some("1000000")
        );
    }

    #[test]
    fn round_trip_iou_amount() {
        let tx = serde_json::json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Amount": {
                "value": "1.5",
                "currency": "USD",
                "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
            }
        });

        let map = tx.as_object().expect("should be object");
        let mut buf = Vec::new();
        serializer::serialize_json_object(map, &mut buf, false).expect("should serialize");

        let decoded = deserialize_object(&buf).expect("should deserialize");

        let amount = decoded.get("Amount").expect("should have Amount");
        let amount_obj = amount.as_object().expect("should be object");
        assert_eq!(amount_obj.get("value").and_then(|v| v.as_str()), Some("1.5"));
        assert_eq!(amount_obj.get("currency").and_then(|v| v.as_str()), Some("USD"));
    }

    #[test]
    fn round_trip_with_blob() {
        let tx = serde_json::json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001"
        });

        let map = tx.as_object().expect("should be object");
        let mut buf = Vec::new();
        serializer::serialize_json_object(map, &mut buf, false).expect("should serialize");

        let decoded = deserialize_object(&buf).expect("should deserialize");
        assert_eq!(
            decoded.get("SigningPubKey").and_then(|v| v.as_str()),
            Some("ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001")
        );
    }
}

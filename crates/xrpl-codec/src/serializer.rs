//! Canonical binary serialization for XRPL objects.
//!
//! This module serializes XRPL transactions and ledger objects from their JSON
//! representation to the canonical binary format used on the wire and for hashing.
//!
//! # Approach
//!
//! The serializer takes a `serde_json::Map` (the JSON representation of a transaction
//! or ledger object) and produces a `Vec<u8>` of canonical binary. It uses the field
//! definitions from [`crate::definitions`] to look up each JSON key's type code and
//! field code, then encodes the value according to the XRPL binary format spec.
//!
//! This JSON-intermediated approach leverages the existing `#[serde(rename)]` attributes
//! on all 83+ transaction type structs, avoiding the need for per-type binary mapping code.

use crate::definitions::{self, FieldDef};
use crate::error::CodecError;
use crate::field_code::{encode_field_id, encode_vl_length};

/// Object end marker: type_code=14 (STObject), nth=1 => (14 << 4) | 1 = 0xE1
const OBJECT_END_MARKER: u8 = 0xE1;

/// Array end marker: type_code=15 (STArray), nth=1 => (15 << 4) | 1 = 0xF1
const ARRAY_END_MARKER: u8 = 0xF1;

/// PathSet boundary marker (between paths).
const PATHSET_BOUNDARY: u8 = 0xFF;

/// PathSet end marker.
const PATHSET_END: u8 = 0x00;

/// Path step type flag: account.
const PATH_STEP_ACCOUNT: u8 = 0x01;

/// Path step type flag: currency.
const PATH_STEP_CURRENCY: u8 = 0x10;

/// Path step type flag: issuer.
const PATH_STEP_ISSUER: u8 = 0x20;

/// Serialize a JSON object (transaction/ledger entry) to canonical XRPL binary.
///
/// This is the main entry point. The JSON map's keys are field names matching
/// `definitions.json` (e.g., "Account", "Fee", "TransactionType").
///
/// If `for_signing` is true, fields where `is_signing_field == false` are excluded
/// (e.g., TxnSignature, Signers).
///
/// # Examples
///
/// Serialize a minimal Payment transaction:
///
/// ```
/// use serde_json::json;
/// use xrpl_codec::serializer::serialize_json_object;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Flags": 0u32,
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000"
/// });
///
/// let map = tx.as_object().expect("json object");
/// let mut buf = Vec::new();
/// serialize_json_object(map, &mut buf, false)?;
///
/// // First byte encodes TransactionType field header (type_code=1, field_code=2 -> 0x12)
/// assert_eq!(buf[0], 0x12);
/// # Ok::<(), xrpl_codec::error::CodecError>(())
/// ```
///
/// Serialize for signing (excludes TxnSignature):
///
/// ```
/// use serde_json::json;
/// use xrpl_codec::serializer::serialize_json_object;
///
/// let tx = json!({
///     "TransactionType": "Payment",
///     "Sequence": 1u32,
///     "Fee": "12",
///     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Amount": "1000000",
///     "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
///     "TxnSignature": "DEADBEEF"
/// });
///
/// let map = tx.as_object().expect("json object");
///
/// let mut full_buf = Vec::new();
/// serialize_json_object(map, &mut full_buf, false)?;
///
/// let mut signing_buf = Vec::new();
/// serialize_json_object(map, &mut signing_buf, true)?;
///
/// // Signing output is shorter because TxnSignature is excluded
/// assert!(signing_buf.len() < full_buf.len());
/// # Ok::<(), xrpl_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError`] if a field name is unknown, a value has the wrong type,
/// or encoding fails.
pub fn serialize_json_object(
    map: &serde_json::Map<String, serde_json::Value>,
    buf: &mut Vec<u8>,
    for_signing: bool,
) -> Result<(), CodecError> {
    // Collect fields with their definitions
    let mut fields: Vec<(&FieldDef, &str, &serde_json::Value)> = Vec::new();

    for (key, value) in map {
        // Skip the "TransactionType" string label when it's not the numeric field
        // (the numeric code will be serialized separately)
        let field_def = match definitions::field_by_name(key) {
            Some(def) => def,
            None => {
                // Skip unknown fields silently (forward compatibility)
                continue;
            }
        };

        if !field_def.is_serialized {
            continue;
        }
        if for_signing && !field_def.is_signing_field {
            continue;
        }

        fields.push((field_def, key, value));
    }

    // Sort by canonical order: (type_code, nth)
    fields.sort_by_key(|(def, _, _)| def.sort_key());

    // Serialize each field
    for (field_def, key, value) in fields {
        serialize_field(field_def, key, value, buf)?;
    }

    Ok(())
}

/// Serialize a single field: field ID header + value.
fn serialize_field(
    field_def: &FieldDef,
    key: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    // Write field ID header
    encode_field_id(field_def.type_code, field_def.nth, buf);

    // Serialize the value based on the field's type code
    serialize_value(field_def.type_code, key, value, field_def.is_vl_encoded, buf)
}

/// Serialize a value based on its XRPL type code.
fn serialize_value(
    type_code: u16,
    field_name: &str,
    value: &serde_json::Value,
    is_vl_encoded: bool,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    match type_code {
        // UInt8 (16)
        16 => serialize_uint8(field_name, value, buf),
        // UInt16 (1)
        1 => serialize_uint16(field_name, value, buf),
        // UInt32 (2)
        2 => serialize_uint32(field_name, value, buf),
        // UInt64 (3)
        3 => serialize_uint64(field_name, value, buf),
        // Hash128 (4)
        4 => serialize_hash(field_name, value, 16, buf),
        // Hash256 (5)
        5 => serialize_hash(field_name, value, 32, buf),
        // Amount (6)
        6 => serialize_amount(field_name, value, buf),
        // Blob (7)
        7 => serialize_blob(field_name, value, buf),
        // AccountID (8)
        8 => serialize_account_id(field_name, value, buf),
        // Number (9) -- XFL format, 8 bytes
        9 => serialize_uint64(field_name, value, buf),
        // Int32 (10)
        10 => serialize_int32(field_name, value, buf),
        // Int64 (11)
        11 => serialize_int64(field_name, value, buf),
        // STObject (14)
        14 => serialize_st_object(field_name, value, buf),
        // STArray (15)
        15 => serialize_st_array(field_name, value, buf),
        // Hash160 (17)
        17 => serialize_hash(field_name, value, 20, buf),
        // PathSet (18)
        18 => serialize_pathset(field_name, value, buf),
        // Vector256 (19)
        19 => serialize_vector256(field_name, value, buf),
        // UInt96 (20)
        20 => serialize_hash(field_name, value, 12, buf),
        // Hash192 (21)
        21 => serialize_hash(field_name, value, 24, buf),
        // UInt384 (22)
        22 => serialize_hash(field_name, value, 48, buf),
        // UInt512 (23)
        23 => serialize_hash(field_name, value, 64, buf),
        // Issue (24)
        24 => serialize_issue(field_name, value, buf),
        // XChainBridge (25)
        25 => serialize_xchain_bridge(field_name, value, buf),
        // Currency (26)
        26 => serialize_currency(field_name, value, buf),
        _ => {
            if is_vl_encoded {
                serialize_blob(field_name, value, buf)
            } else {
                Err(CodecError::TypeMismatch {
                    field: field_name.to_string(),
                    expected: format!("known type code, got {type_code}"),
                    got: "unknown".to_string(),
                })
            }
        }
    }
}

fn serialize_uint8(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    let n = json_to_u64(field, value)?;
    buf.push(n as u8);
    Ok(())
}

fn serialize_uint16(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    let n = match value {
        serde_json::Value::String(s) => {
            // UInt16 fields like TransactionType and LedgerEntryType use string names
            // in JSON that map to numeric codes. Resolve via definitions.
            match field {
                "TransactionType" => definitions::tx_type_code(s)
                    .map(|c| c as u64)
                    .ok_or_else(|| CodecError::Serialization(format!(
                        "unknown TransactionType: '{s}'"
                    )))?,
                "LedgerEntryType" => definitions::le_type_code(s)
                    .map(|c| c as u64)
                    .ok_or_else(|| CodecError::Serialization(format!(
                        "unknown LedgerEntryType: '{s}'"
                    )))?,
                _ => s.parse::<u64>().map_err(|_| CodecError::TypeMismatch {
                    field: field.to_string(),
                    expected: "numeric string or integer".to_string(),
                    got: format!("'{s}'"),
                })?,
            }
        }
        _ => json_to_u64(field, value)?,
    };
    buf.extend_from_slice(&(n as u16).to_be_bytes());
    Ok(())
}

fn serialize_uint32(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    let n = json_to_u64(field, value)?;
    buf.extend_from_slice(&(n as u32).to_be_bytes());
    Ok(())
}

fn serialize_uint64(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    // UInt64 in XRPL JSON can be a string (for large values) or a number
    let n = match value {
        serde_json::Value::String(s) => s.parse::<u64>().map_err(|_| CodecError::TypeMismatch {
            field: field.to_string(),
            expected: "numeric string".to_string(),
            got: format!("'{s}'"),
        })?,
        serde_json::Value::Number(_) => json_to_u64(field, value)?,
        _ => return Err(type_mismatch(field, "number or string", value)),
    };
    buf.extend_from_slice(&n.to_be_bytes());
    Ok(())
}

fn serialize_int32(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    let n = value
        .as_i64()
        .ok_or_else(|| type_mismatch(field, "integer", value))?;
    buf.extend_from_slice(&(n as i32).to_be_bytes());
    Ok(())
}

fn serialize_int64(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    let n = match value {
        serde_json::Value::String(s) => s.parse::<i64>().map_err(|_| CodecError::TypeMismatch {
            field: field.to_string(),
            expected: "numeric string".to_string(),
            got: format!("'{s}'"),
        })?,
        _ => value
            .as_i64()
            .ok_or_else(|| type_mismatch(field, "integer", value))?,
    };
    buf.extend_from_slice(&n.to_be_bytes());
    Ok(())
}

fn serialize_hash(
    field: &str,
    value: &serde_json::Value,
    expected_len: usize,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let hex_str = value
        .as_str()
        .ok_or_else(|| type_mismatch(field, "hex string", value))?;
    let bytes = hex::decode(hex_str).map_err(|e| CodecError::InvalidHex {
        field: field.to_string(),
        reason: e.to_string(),
    })?;
    if bytes.len() != expected_len {
        return Err(CodecError::TypeMismatch {
            field: field.to_string(),
            expected: format!("{expected_len} bytes"),
            got: format!("{} bytes", bytes.len()),
        });
    }
    buf.extend_from_slice(&bytes);
    Ok(())
}

fn serialize_blob(field: &str, value: &serde_json::Value, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    let hex_str = value
        .as_str()
        .ok_or_else(|| type_mismatch(field, "hex string", value))?;
    let bytes = hex::decode(hex_str).map_err(|e| CodecError::InvalidHex {
        field: field.to_string(),
        reason: e.to_string(),
    })?;
    encode_vl_length(bytes.len(), buf)?;
    buf.extend_from_slice(&bytes);
    Ok(())
}

fn serialize_account_id(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let address = value
        .as_str()
        .ok_or_else(|| type_mismatch(field, "address string", value))?;

    // Decode the base58check classic address to get the 20-byte account ID
    let decoded = bs58::decode(address)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check(None)
        .into_vec()
        .map_err(|e| CodecError::Serialization(format!("invalid address '{address}': {e}")))?;

    if decoded.len() != 21 || decoded[0] != 0 {
        return Err(CodecError::Serialization(format!(
            "invalid address version for '{address}'"
        )));
    }

    let account_bytes = &decoded[1..]; // 20 bytes
    // AccountID is VL-encoded in binary (always 20 bytes, but still VL-prefixed)
    encode_vl_length(20, buf)?;
    buf.extend_from_slice(account_bytes);
    Ok(())
}

/// Serialize an Amount field. The Amount type is polymorphic in the XRPL binary format:
///
/// - XRP: 8 bytes (bit 63 = 0)
/// - IOU: 48 bytes (8-byte custom float + 20-byte currency + 20-byte issuer)
/// - MPT: 32 bytes (8-byte value + 24-byte MptIssuanceId)
fn serialize_amount(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    match value {
        serde_json::Value::String(drops_str) => {
            // XRP amount: string of drops
            serialize_xrp_amount(field, drops_str, buf)
        }
        serde_json::Value::Object(map) => {
            if map.contains_key("mpt_issuance_id") {
                serialize_mpt_amount(field, map, buf)
            } else if map.contains_key("currency") {
                serialize_iou_amount(field, map, buf)
            } else {
                Err(CodecError::InvalidAmount(format!(
                    "amount object for '{field}' must have 'currency' or 'mpt_issuance_id'"
                )))
            }
        }
        _ => Err(type_mismatch(field, "string or object", value)),
    }
}

/// Serialize XRP amount as 8 bytes.
///
/// Bit layout:
/// - Bit 63: 0 (marks as XRP)
/// - Bit 62: 1 if positive/zero, 0 if negative
/// - Bits 61-0: absolute drops value
fn serialize_xrp_amount(field: &str, drops_str: &str, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    // XRP amounts can be negative in some contexts (e.g., delivered_amount metadata)
    let negative = drops_str.starts_with('-');
    let abs_str = if negative { &drops_str[1..] } else { drops_str };

    let drops: u64 = abs_str.parse().map_err(|_| {
        CodecError::InvalidAmount(format!("invalid XRP drops value for '{field}': '{drops_str}'"))
    })?;

    // Bit 63 = 0 (XRP), Bit 62 = positive flag
    let mut encoded = drops & 0x3FFF_FFFF_FFFF_FFFF; // clear top 2 bits, keep drops
    if !negative {
        encoded |= 0x4000_0000_0000_0000; // set bit 62 (positive)
    }
    // Bit 63 stays 0 (XRP marker)

    buf.extend_from_slice(&encoded.to_be_bytes());
    Ok(())
}

/// Serialize IOU amount as 48 bytes.
///
/// Bytes 0-7: Custom float (bit 63=1, bit 62=sign, bits 61-54=exponent+97, bits 53-0=mantissa)
/// Bytes 8-27: Currency code (20 bytes)
/// Bytes 28-47: Issuer AccountID (20 bytes, NOT VL-encoded)
fn serialize_iou_amount(
    field: &str,
    map: &serde_json::Map<String, serde_json::Value>,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let value_str = map
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CodecError::InvalidAmount(format!("IOU amount '{field}' missing 'value'")))?;

    // Parse the decimal value into XRPL's custom float
    let value_bytes = encode_iou_value(field, value_str)?;
    buf.extend_from_slice(&value_bytes);

    // Currency code: 20 bytes
    let currency_str = map
        .get("currency")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CodecError::InvalidAmount(format!("IOU amount '{field}' missing 'currency'"))
        })?;
    let currency_bytes = encode_currency_code(field, currency_str)?;
    buf.extend_from_slice(&currency_bytes);

    // Issuer: 20 bytes (raw account ID, NOT VL-encoded)
    let issuer_str = map
        .get("issuer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CodecError::InvalidAmount(format!("IOU amount '{field}' missing 'issuer'"))
        })?;
    let issuer_bytes = decode_account_id(field, issuer_str)?;
    buf.extend_from_slice(&issuer_bytes);

    Ok(())
}

/// Encode an IOU value string to 8 bytes of XRPL custom float format.
fn encode_iou_value(field: &str, value_str: &str) -> Result<[u8; 8], CodecError> {
    // Parse using IssuedValue's logic
    let issued_value = xrpl_types::IssuedValue::from_decimal_string(value_str)
        .map_err(|e| CodecError::InvalidAmount(format!("invalid IOU value for '{field}': {e}")))?;

    if issued_value.is_zero() {
        // IOU zero: bit 63=1, rest zeros
        return Ok(0x8000_0000_0000_0000u64.to_be_bytes());
    }

    let mantissa = issued_value.mantissa();
    let exponent = issued_value.exponent();
    let negative = mantissa < 0;
    let abs_mantissa = mantissa.unsigned_abs();

    // Build the 8-byte encoded value:
    // Bit 63: 1 (not XRP)
    // Bit 62: 1 if positive, 0 if negative
    // Bits 61-54: exponent + 97 (8 bits)
    // Bits 53-0: mantissa (54 bits)
    let biased_exp = (exponent as i16 + 97) as u64;

    let mut encoded: u64 = 0;
    encoded |= 1 << 63; // bit 63: not XRP
    if !negative {
        encoded |= 1 << 62; // bit 62: positive
    }
    encoded |= (biased_exp & 0xFF) << 54; // bits 61-54: exponent
    encoded |= abs_mantissa & 0x003F_FFFF_FFFF_FFFF; // bits 53-0: mantissa

    Ok(encoded.to_be_bytes())
}

/// Serialize MPT amount as 33 bytes.
///
/// Layout (matching rippled's STAmount serialization):
/// - Byte 0: flags — bit 5=1 (MPT flag), bit 6=sign (1=positive), bit 7=0
///   Positive MPT → 0x60, negative MPT → 0x20
/// - Bytes 1-8: 64-bit absolute value (big-endian)
/// - Bytes 9-32: MptIssuanceId (24 bytes)
fn serialize_mpt_amount(
    field: &str,
    map: &serde_json::Map<String, serde_json::Value>,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let value_str = map
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CodecError::InvalidAmount(format!("MPT amount '{field}' missing 'value'")))?;

    let value: i64 = value_str.parse().map_err(|_| {
        CodecError::InvalidAmount(format!("invalid MPT value for '{field}': '{value_str}'"))
    })?;

    let negative = value < 0;
    let abs_value = value.unsigned_abs();

    // 1-byte flags (matches rippled cMPToken>>56 | cPositive>>56):
    // Bit 5: 1 (MPT flag)
    // Bit 6: 1 if positive/zero, 0 if negative
    // Bit 7: 0 (not IOU — MPT shares bit 7=0 with XRP)
    let mut flags: u8 = 0x20; // MPT flag
    if !negative {
        flags |= 0x40; // positive
    }
    buf.push(flags);

    // 8-byte absolute value
    buf.extend_from_slice(&abs_value.to_be_bytes());

    // MptIssuanceId: 24 bytes
    let id_str = map
        .get("mpt_issuance_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CodecError::InvalidAmount(format!(
                "MPT amount '{field}' missing 'mpt_issuance_id'"
            ))
        })?;
    let id_bytes = hex::decode(id_str).map_err(|e| CodecError::InvalidHex {
        field: field.to_string(),
        reason: e.to_string(),
    })?;
    if id_bytes.len() != 24 {
        return Err(CodecError::InvalidAmount(format!(
            "MPT issuance ID for '{field}' must be 24 bytes, got {}",
            id_bytes.len()
        )));
    }
    buf.extend_from_slice(&id_bytes);

    Ok(())
}

/// Serialize an STObject (nested object with end marker).
fn serialize_st_object(
    _field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let map = value
        .as_object()
        .ok_or_else(|| CodecError::Serialization("STObject must be a JSON object".into()))?;
    serialize_json_object(map, buf, false)?;
    buf.push(OBJECT_END_MARKER);
    Ok(())
}

/// Serialize an STArray.
///
/// Each element is wrapped in a single-key object where the key is the element type
/// (e.g., "Memo", "Signer"). Each element is serialized as:
/// 1. The wrapper field header
/// 2. Inner fields in canonical order
/// 3. Object end marker (0xE1)
///
/// After all elements: array end marker (0xF1)
fn serialize_st_array(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let arr = value
        .as_array()
        .ok_or_else(|| type_mismatch(field, "array", value))?;

    for element in arr {
        let element_obj = element
            .as_object()
            .ok_or_else(|| CodecError::Serialization("STArray element must be an object".into()))?;

        // Each array element is a single-key wrapper object
        // e.g., {"Memo": {"MemoType": "...", "MemoData": "..."}}
        for (wrapper_key, inner_value) in element_obj {
            let wrapper_def = definitions::field_by_name(wrapper_key).ok_or_else(|| {
                CodecError::UnknownFieldName(wrapper_key.clone())
            })?;

            // Write the wrapper field header
            encode_field_id(wrapper_def.type_code, wrapper_def.nth, buf);

            // Serialize inner fields
            if let Some(inner_map) = inner_value.as_object() {
                serialize_json_object(inner_map, buf, false)?;
            }

            // End marker for this element
            buf.push(OBJECT_END_MARKER);
        }
    }

    buf.push(ARRAY_END_MARKER);
    Ok(())
}

/// Serialize a PathSet.
///
/// A PathSet is an array of paths, where each path is an array of steps.
/// Each step has optional account, currency, and issuer fields.
fn serialize_pathset(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let paths = value
        .as_array()
        .ok_or_else(|| type_mismatch(field, "array of paths", value))?;

    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            buf.push(PATHSET_BOUNDARY); // 0xFF between paths
        }

        let steps = path
            .as_array()
            .ok_or_else(|| CodecError::Serialization("path must be an array of steps".into()))?;

        for step in steps {
            let step_obj = step.as_object().ok_or_else(|| {
                CodecError::Serialization("path step must be an object".into())
            })?;

            // Determine step type flags
            let mut type_byte: u8 = 0;
            if step_obj.contains_key("account") {
                type_byte |= PATH_STEP_ACCOUNT;
            }
            if step_obj.contains_key("currency") {
                type_byte |= PATH_STEP_CURRENCY;
            }
            if step_obj.contains_key("issuer") {
                type_byte |= PATH_STEP_ISSUER;
            }

            buf.push(type_byte);

            // Write the data for each present field (in fixed order: account, currency, issuer)
            if let Some(account_val) = step_obj.get("account") {
                let addr = account_val.as_str().ok_or_else(|| {
                    CodecError::Serialization("path step account must be a string".into())
                })?;
                let bytes = decode_account_id(field, addr)?;
                buf.extend_from_slice(&bytes);
            }
            if let Some(currency_val) = step_obj.get("currency") {
                let curr_str = currency_val.as_str().ok_or_else(|| {
                    CodecError::Serialization("path step currency must be a string".into())
                })?;
                let bytes = encode_currency_code(field, curr_str)?;
                buf.extend_from_slice(&bytes);
            }
            if let Some(issuer_val) = step_obj.get("issuer") {
                let addr = issuer_val.as_str().ok_or_else(|| {
                    CodecError::Serialization("path step issuer must be a string".into())
                })?;
                let bytes = decode_account_id(field, addr)?;
                buf.extend_from_slice(&bytes);
            }
        }
    }

    buf.push(PATHSET_END); // 0x00
    Ok(())
}

/// Serialize a Vector256 (VL-encoded array of Hash256 values).
fn serialize_vector256(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let arr = value
        .as_array()
        .ok_or_else(|| type_mismatch(field, "array of hex strings", value))?;

    let total_len = arr.len() * 32;
    encode_vl_length(total_len, buf)?;

    for item in arr {
        let hex_str = item
            .as_str()
            .ok_or_else(|| type_mismatch(field, "hex string", item))?;
        let bytes = hex::decode(hex_str).map_err(|e| CodecError::InvalidHex {
            field: field.to_string(),
            reason: e.to_string(),
        })?;
        if bytes.len() != 32 {
            return Err(CodecError::TypeMismatch {
                field: field.to_string(),
                expected: "32-byte hash".to_string(),
                got: format!("{}-byte value", bytes.len()),
            });
        }
        buf.extend_from_slice(&bytes);
    }

    Ok(())
}

/// Serialize an Issue (asset identifier).
///
/// - XRP: 20 bytes of zeros
/// - IOU: 20-byte currency code + 20-byte issuer = 40 bytes
/// - MPT: Not yet clear if Issue supports MPT directly; handle currency+issuer path
fn serialize_issue(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let map = value
        .as_object()
        .ok_or_else(|| type_mismatch(field, "object", value))?;

    if let Some(mpt_id) = map.get("mpt_issuance_id") {
        // MPT Issue: just the 24-byte issuance ID
        let id_str = mpt_id.as_str().ok_or_else(|| {
            CodecError::Serialization("mpt_issuance_id must be a string".into())
        })?;
        let id_bytes = hex::decode(id_str).map_err(|e| CodecError::InvalidHex {
            field: field.to_string(),
            reason: e.to_string(),
        })?;
        // For Issue type, MPT is 24 bytes per the protocol
        buf.extend_from_slice(&id_bytes);
        return Ok(());
    }

    let currency_str = map
        .get("currency")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CodecError::Serialization("Issue must have 'currency'".into()))?;

    if currency_str == "XRP" {
        // XRP: 20 bytes of zeros (currency code for XRP)
        buf.extend_from_slice(&[0u8; 20]);
    } else {
        // IOU: currency (20 bytes) + issuer (20 bytes)
        let currency_bytes = encode_currency_code(field, currency_str)?;
        buf.extend_from_slice(&currency_bytes);

        let issuer_str = map
            .get("issuer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CodecError::Serialization("Issue with non-XRP currency must have 'issuer'".into())
            })?;
        let issuer_bytes = decode_account_id(field, issuer_str)?;
        buf.extend_from_slice(&issuer_bytes);
    }

    Ok(())
}

/// Serialize an XChainBridge composite type.
///
/// Structure: LockingChainDoor (AccountID, VL) + LockingChainIssue (Issue) +
///            IssuingChainDoor (AccountID, VL) + IssuingChainIssue (Issue)
fn serialize_xchain_bridge(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let map = value
        .as_object()
        .ok_or_else(|| type_mismatch(field, "object", value))?;

    // LockingChainDoor
    if let Some(door) = map.get("LockingChainDoor") {
        serialize_account_id("LockingChainDoor", door, buf)?;
    }
    // LockingChainIssue
    if let Some(issue) = map.get("LockingChainIssue") {
        serialize_issue("LockingChainIssue", issue, buf)?;
    }
    // IssuingChainDoor
    if let Some(door) = map.get("IssuingChainDoor") {
        serialize_account_id("IssuingChainDoor", door, buf)?;
    }
    // IssuingChainIssue
    if let Some(issue) = map.get("IssuingChainIssue") {
        serialize_issue("IssuingChainIssue", issue, buf)?;
    }

    Ok(())
}

/// Serialize a bare Currency type (20 bytes).
fn serialize_currency(
    field: &str,
    value: &serde_json::Value,
    buf: &mut Vec<u8>,
) -> Result<(), CodecError> {
    let s = value
        .as_str()
        .ok_or_else(|| type_mismatch(field, "currency string", value))?;
    let bytes = encode_currency_code(field, s)?;
    buf.extend_from_slice(&bytes);
    Ok(())
}

// --- Helper functions ---

/// Decode a classic address to raw 20-byte account ID.
fn decode_account_id(field: &str, address: &str) -> Result<[u8; 20], CodecError> {
    let decoded = bs58::decode(address)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check(None)
        .into_vec()
        .map_err(|e| CodecError::Serialization(format!("invalid address for '{field}': {e}")))?;

    if decoded.len() != 21 || decoded[0] != 0 {
        return Err(CodecError::Serialization(format!(
            "invalid address version for '{field}'"
        )));
    }

    let mut result = [0u8; 20];
    result.copy_from_slice(&decoded[1..]);
    Ok(result)
}

/// Encode a currency code string to 20 bytes.
fn encode_currency_code(field: &str, s: &str) -> Result<[u8; 20], CodecError> {
    if s == "XRP" {
        return Ok([0u8; 20]);
    }
    if s.len() == 3 {
        // Standard 3-char code
        let bytes = s.as_bytes();
        let mut result = [0u8; 20];
        result[12] = bytes[0];
        result[13] = bytes[1];
        result[14] = bytes[2];
        Ok(result)
    } else if s.len() == 40 {
        // Non-standard hex
        let bytes = hex::decode(s).map_err(|e| CodecError::InvalidHex {
            field: field.to_string(),
            reason: e.to_string(),
        })?;
        let mut result = [0u8; 20];
        result.copy_from_slice(&bytes);
        Ok(result)
    } else {
        Err(CodecError::TypeMismatch {
            field: field.to_string(),
            expected: "3-char or 40-hex currency code".to_string(),
            got: format!("{}-char string", s.len()),
        })
    }
}

fn json_to_u64(field: &str, value: &serde_json::Value) -> Result<u64, CodecError> {
    value
        .as_u64()
        .ok_or_else(|| type_mismatch(field, "unsigned integer", value))
}

fn type_mismatch(field: &str, expected: &str, value: &serde_json::Value) -> CodecError {
    let got = match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    };
    CodecError::TypeMismatch {
        field: field.to_string(),
        expected: expected.to_string(),
        got: got.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialize_xrp_amount_positive() {
        let mut buf = Vec::new();
        serialize_xrp_amount("Fee", "1000000", &mut buf).expect("should encode");
        // 1,000,000 = 0x0F4240
        // With bit 62 set (positive): 0x4000_0000_000F_4240
        assert_eq!(buf, vec![0x40, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x42, 0x40]);
    }

    #[test]
    fn serialize_xrp_amount_zero() {
        let mut buf = Vec::new();
        serialize_xrp_amount("Fee", "0", &mut buf).expect("should encode");
        // Zero with positive flag: 0x4000_0000_0000_0000
        assert_eq!(buf, vec![0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn serialize_iou_value_zero() {
        let bytes = encode_iou_value("Amount", "0").expect("should encode");
        // IOU zero: 0x80 followed by 7 zero bytes
        assert_eq!(bytes, [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn serialize_iou_value_positive() {
        // 1.5 = mantissa 1500000000000000, exponent -15
        // biased_exp = -15 + 97 = 82 = 0x52
        // mantissa = 1500000000000000 = 0x5_5730_7DC6_2580 (but only lower 54 bits)
        let bytes = encode_iou_value("Amount", "1.5").expect("should encode");
        assert_eq!(bytes[0] & 0xC0, 0xC0); // bits 63,62 both set (not-XRP, positive)
    }

    #[test]
    fn serialize_simple_payment_fields() {
        // Build a minimal payment JSON
        let tx = json!({
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
        serialize_json_object(map, &mut buf, false).expect("should serialize");

        // Verify we got some output
        assert!(!buf.is_empty());

        // Verify the first field is TransactionType (UInt16, tc=1, fc=2) = 0x12
        assert_eq!(buf[0], 0x12);
        // TransactionType value = 0 (Payment) = [0x00, 0x00]
        assert_eq!(buf[1], 0x00);
        assert_eq!(buf[2], 0x00);
    }

    #[test]
    fn canonical_field_ordering() {
        // Verify that fields are serialized in canonical order regardless of JSON key order
        let tx1 = json!({
            "Fee": "12",
            "TransactionType": 0u16,
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Sequence": 1u32
        });
        let tx2 = json!({
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Sequence": 1u32,
            "TransactionType": 0u16,
            "Fee": "12"
        });

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        serialize_json_object(tx1.as_object().expect("obj"), &mut buf1, false).expect("ser1");
        serialize_json_object(tx2.as_object().expect("obj"), &mut buf2, false).expect("ser2");

        assert_eq!(buf1, buf2, "canonical ordering must be deterministic");
    }

    #[test]
    fn signing_excludes_signature() {
        let tx = json!({
            "TransactionType": 0u16,
            "Sequence": 1u32,
            "Fee": "12",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
            "TxnSignature": "DEADBEEF"
        });

        let map = tx.as_object().expect("obj");

        let mut buf_full = Vec::new();
        let mut buf_signing = Vec::new();
        serialize_json_object(map, &mut buf_full, false).expect("full");
        serialize_json_object(map, &mut buf_signing, true).expect("signing");

        // Signing should be shorter (TxnSignature excluded)
        assert!(
            buf_signing.len() < buf_full.len(),
            "signing blob should be smaller than full blob"
        );
    }
}

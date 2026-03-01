//! Field ID encoding and variable-length (VL) prefix encoding.
//!
//! Every serialized field in the XRPL binary format begins with a 1-3 byte
//! field ID header encoding the type_code and field_code (nth).
//!
//! Additionally, variable-length types (Blob, AccountID, Vector256) are
//! preceded by a 1-3 byte length prefix.

use crate::error::CodecError;

/// Encode a field ID header from type_code and field_code into the buffer.
///
/// The encoding uses 1-3 bytes depending on whether type_code and field_code
/// are less than 16:
///
/// - Both < 16: 1 byte `(tc << 4) | fc`
/// - tc < 16, fc >= 16: 2 bytes `(tc << 4) | 0x00, fc`
/// - tc >= 16, fc < 16: 2 bytes `0x00 | fc, tc`
/// - Both >= 16: 3 bytes `0x00, tc, fc`
///
/// # Examples
///
/// ```
/// use xrpl_mithril_codec::field_code::encode_field_id;
///
/// // TransactionType: type_code=1, field_code=2 -> single byte 0x12
/// let mut buf = Vec::new();
/// encode_field_id(1, 2, &mut buf);
/// assert_eq!(buf, vec![0x12]);
///
/// // LastLedgerSequence: type_code=2, field_code=27 -> two bytes
/// let mut buf = Vec::new();
/// encode_field_id(2, 27, &mut buf);
/// assert_eq!(buf, vec![0x20, 0x1B]);
///
/// // Both >= 16 -> three bytes
/// let mut buf = Vec::new();
/// encode_field_id(16, 16, &mut buf);
/// assert_eq!(buf, vec![0x00, 0x10, 0x10]);
/// ```
pub fn encode_field_id(type_code: u16, field_code: u16, buf: &mut Vec<u8>) {
    let tc = type_code as u8;
    let fc = field_code as u8;

    match (type_code < 16, field_code < 16) {
        (true, true) => {
            buf.push((tc << 4) | fc);
        }
        (true, false) => {
            buf.push(tc << 4);
            buf.push(fc);
        }
        (false, true) => {
            buf.push(fc);
            buf.push(tc);
        }
        (false, false) => {
            buf.push(0x00);
            buf.push(tc);
            buf.push(fc);
        }
    }
}

/// Decode a field ID header from the data, returning (type_code, field_code, bytes_consumed).
///
/// # Examples
///
/// ```
/// use xrpl_mithril_codec::field_code::{encode_field_id, decode_field_id};
///
/// // Encode then decode round-trips correctly
/// let mut buf = Vec::new();
/// encode_field_id(1, 2, &mut buf); // TransactionType
///
/// let (type_code, field_code, consumed) = decode_field_id(&buf)?;
/// assert_eq!(type_code, 1);
/// assert_eq!(field_code, 2);
/// assert_eq!(consumed, 1); // single-byte encoding
///
/// // Decode a two-byte field ID
/// let mut buf = Vec::new();
/// encode_field_id(2, 27, &mut buf); // LastLedgerSequence
/// let (tc, fc, consumed) = decode_field_id(&buf)?;
/// assert_eq!((tc, fc), (2, 27));
/// assert_eq!(consumed, 2);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError::UnexpectedEnd`] if there aren't enough bytes.
pub fn decode_field_id(data: &[u8]) -> Result<(u16, u16, usize), CodecError> {
    if data.is_empty() {
        return Err(CodecError::UnexpectedEnd {
            needed: 1,
            available: 0,
        });
    }

    let b0 = data[0];
    let high = (b0 >> 4) as u16;
    let low = (b0 & 0x0F) as u16;

    match (high, low) {
        (0, 0) => {
            // Both >= 16: 3 bytes
            if data.len() < 3 {
                return Err(CodecError::UnexpectedEnd {
                    needed: 3,
                    available: data.len(),
                });
            }
            Ok((data[1] as u16, data[2] as u16, 3))
        }
        (0, fc) => {
            // tc >= 16, fc < 16: 2 bytes
            if data.len() < 2 {
                return Err(CodecError::UnexpectedEnd {
                    needed: 2,
                    available: data.len(),
                });
            }
            Ok((data[1] as u16, fc, 2))
        }
        (tc, 0) => {
            // tc < 16, fc >= 16: 2 bytes
            if data.len() < 2 {
                return Err(CodecError::UnexpectedEnd {
                    needed: 2,
                    available: data.len(),
                });
            }
            Ok((tc, data[1] as u16, 2))
        }
        (tc, fc) => {
            // Both < 16: 1 byte
            Ok((tc, fc, 1))
        }
    }
}

/// Maximum VL-encoded length.
pub const MAX_VL_LENGTH: usize = 918_744;

/// Encode a variable-length prefix into the buffer.
///
/// The XRPL binary format uses a 1-3 byte length prefix for variable-length types:
///
/// - Length 0-192: 1 byte `[length]`
/// - Length 193-12480: 2 bytes, encoding `length = 193 + (b1 - 193) * 256 + b2`
/// - Length 12481-918744: 3 bytes, encoding `length = 12481 + (b1 - 241) * 65536 + b2 * 256 + b3`
///
/// # Examples
///
/// ```
/// use xrpl_mithril_codec::field_code::encode_vl_length;
///
/// // Small length (0-192): single byte
/// let mut buf = Vec::new();
/// encode_vl_length(20, &mut buf)?;
/// assert_eq!(buf, vec![0x14]);
///
/// // Medium length (193-12480): two bytes
/// let mut buf = Vec::new();
/// encode_vl_length(200, &mut buf)?;
/// assert_eq!(buf.len(), 2);
///
/// // Large length (12481-918744): three bytes
/// let mut buf = Vec::new();
/// encode_vl_length(12481, &mut buf)?;
/// assert_eq!(buf.len(), 3);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError::VlLengthOverflow`] if length exceeds 918,744.
pub fn encode_vl_length(length: usize, buf: &mut Vec<u8>) -> Result<(), CodecError> {
    if length <= 192 {
        buf.push(length as u8);
    } else if length <= 12480 {
        let adjusted = length - 193;
        let b1 = (adjusted / 256) as u8 + 193;
        let b2 = (adjusted % 256) as u8;
        buf.push(b1);
        buf.push(b2);
    } else if length <= MAX_VL_LENGTH {
        let adjusted = length - 12481;
        let b1 = (adjusted / 65536) as u8 + 241;
        let b2 = ((adjusted / 256) % 256) as u8;
        let b3 = (adjusted % 256) as u8;
        buf.push(b1);
        buf.push(b2);
        buf.push(b3);
    } else {
        return Err(CodecError::VlLengthOverflow(length));
    }
    Ok(())
}

/// Decode a variable-length prefix from the data, returning (length, bytes_consumed).
///
/// # Examples
///
/// ```
/// use xrpl_mithril_codec::field_code::{encode_vl_length, decode_vl_length};
///
/// // Round-trip: encode then decode
/// let mut buf = Vec::new();
/// encode_vl_length(100, &mut buf)?;
/// let (length, consumed) = decode_vl_length(&buf)?;
/// assert_eq!(length, 100);
/// assert_eq!(consumed, 1); // single-byte encoding for length <= 192
///
/// // Two-byte round-trip
/// let mut buf = Vec::new();
/// encode_vl_length(1000, &mut buf)?;
/// let (length, consumed) = decode_vl_length(&buf)?;
/// assert_eq!(length, 1000);
/// assert_eq!(consumed, 2);
/// # Ok::<(), xrpl_mithril_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns [`CodecError::UnexpectedEnd`] if there aren't enough bytes.
pub fn decode_vl_length(data: &[u8]) -> Result<(usize, usize), CodecError> {
    if data.is_empty() {
        return Err(CodecError::UnexpectedEnd {
            needed: 1,
            available: 0,
        });
    }

    let b0 = data[0] as usize;

    if b0 <= 192 {
        Ok((b0, 1))
    } else if b0 <= 240 {
        if data.len() < 2 {
            return Err(CodecError::UnexpectedEnd {
                needed: 2,
                available: data.len(),
            });
        }
        let b1 = data[1] as usize;
        let length = 193 + (b0 - 193) * 256 + b1;
        Ok((length, 2))
    } else if b0 <= 254 {
        if data.len() < 3 {
            return Err(CodecError::UnexpectedEnd {
                needed: 3,
                available: data.len(),
            });
        }
        let b1 = data[1] as usize;
        let b2 = data[2] as usize;
        let length = 12481 + (b0 - 241) * 65536 + b1 * 256 + b2;
        Ok((length, 3))
    } else {
        Err(CodecError::VlLengthOverflow(b0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Field ID encoding tests ---

    #[test]
    fn encode_both_small() {
        // TransactionType: tc=1, fc=2 -> 0x12
        let mut buf = Vec::new();
        encode_field_id(1, 2, &mut buf);
        assert_eq!(buf, vec![0x12]);
    }

    #[test]
    fn encode_tc_small_fc_large() {
        // LastLedgerSequence: tc=2, fc=27 -> [0x20, 0x1B]
        let mut buf = Vec::new();
        encode_field_id(2, 27, &mut buf);
        assert_eq!(buf, vec![0x20, 0x1B]);
    }

    #[test]
    fn encode_tc_large_fc_small() {
        // CloseResolution: tc=16, fc=1 -> [0x01, 0x10]
        let mut buf = Vec::new();
        encode_field_id(16, 1, &mut buf);
        assert_eq!(buf, vec![0x01, 0x10]);
    }

    #[test]
    fn encode_both_large() {
        // TickSize: tc=16, fc=16 -> [0x00, 0x10, 0x10]
        let mut buf = Vec::new();
        encode_field_id(16, 16, &mut buf);
        assert_eq!(buf, vec![0x00, 0x10, 0x10]);
    }

    #[test]
    fn decode_round_trip_all_cases() {
        let cases: Vec<(u16, u16)> = vec![
            (1, 2),   // both small
            (2, 27),  // tc small, fc large
            (16, 1),  // tc large, fc small
            (16, 16), // both large
            (6, 8),   // Fee: tc=6, fc=8
            (7, 3),   // SigningPubKey: tc=7, fc=3
            (8, 1),   // Account: tc=8, fc=1
            (15, 1),  // ArrayEndMarker
            (14, 1),  // ObjectEndMarker
        ];

        for (tc, fc) in cases {
            let mut buf = Vec::new();
            encode_field_id(tc, fc, &mut buf);
            let (dec_tc, dec_fc, consumed) =
                decode_field_id(&buf).expect(&format!("decode ({tc},{fc})"));
            assert_eq!((dec_tc, dec_fc), (tc, fc), "round-trip failed for ({tc},{fc})");
            assert_eq!(consumed, buf.len());
        }
    }

    // --- VL length encoding tests ---

    #[test]
    fn vl_single_byte() {
        let mut buf = Vec::new();
        encode_vl_length(20, &mut buf).expect("should encode");
        assert_eq!(buf, vec![0x14]); // 20 = 0x14
    }

    #[test]
    fn vl_zero() {
        let mut buf = Vec::new();
        encode_vl_length(0, &mut buf).expect("should encode");
        assert_eq!(buf, vec![0x00]);
    }

    #[test]
    fn vl_max_single_byte() {
        let mut buf = Vec::new();
        encode_vl_length(192, &mut buf).expect("should encode");
        assert_eq!(buf, vec![192]);
    }

    #[test]
    fn vl_two_bytes() {
        let mut buf = Vec::new();
        encode_vl_length(193, &mut buf).expect("should encode");
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn vl_three_bytes() {
        let mut buf = Vec::new();
        encode_vl_length(12481, &mut buf).expect("should encode");
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn vl_round_trip() {
        let test_lengths = [0, 1, 20, 33, 100, 192, 193, 200, 1000, 12480, 12481, 50000, 918744];

        for &length in &test_lengths {
            let mut buf = Vec::new();
            encode_vl_length(length, &mut buf).expect(&format!("encode {length}"));
            let (decoded, consumed) =
                decode_vl_length(&buf).expect(&format!("decode {length}"));
            assert_eq!(decoded, length, "round-trip failed for {length}");
            assert_eq!(consumed, buf.len(), "consumed wrong for {length}");
        }
    }

    #[test]
    fn vl_overflow() {
        let mut buf = Vec::new();
        assert!(encode_vl_length(918745, &mut buf).is_err());
    }
}

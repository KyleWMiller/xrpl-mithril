//! Round-trip and known-good vector tests for the XRPL binary codec.
//!
//! These tests verify byte-for-byte correctness against known-good vectors
//! from the ripple-binary-codec test suite (XRPLF/xrpl.js).

use xrpl_mithril_codec::{deserializer, serializer, signing};

/// Serialize a JSON object and return the hex string.
fn serialize_to_hex(json: &serde_json::Value) -> String {
    let map = json.as_object().expect("test vector must be a JSON object");
    let mut buf = Vec::new();
    serializer::serialize_json_object(map, &mut buf, false)
        .expect("serialization should not fail");
    hex::encode_upper(buf)
}

/// Test helper: serialize JSON to binary, compare against expected hex.
fn assert_serialization(name: &str, json: &serde_json::Value, expected_hex: &str) {
    let actual = serialize_to_hex(json);
    assert_eq!(
        actual,
        expected_hex.to_uppercase(),
        "serialization mismatch for '{name}'"
    );
}

/// Test helper: serialize then deserialize, verify key fields survive.
fn assert_round_trip(name: &str, json: &serde_json::Value) {
    let map = json.as_object().expect("JSON object");
    let mut buf = Vec::new();
    serializer::serialize_json_object(map, &mut buf, false)
        .unwrap_or_else(|e| panic!("serialization failed for '{name}': {e}"));

    let decoded = deserializer::deserialize_object(&buf)
        .unwrap_or_else(|e| panic!("deserialization failed for '{name}': {e}"));

    // Verify all numeric and string fields round-trip
    for (key, value) in map {
        // Skip TransactionType string — it's serialized as a numeric code
        if key == "TransactionType" {
            continue;
        }
        // Skip Paths — their JSON structure differs slightly (missing "type" field on deser)
        if key == "Paths" {
            continue;
        }

        if let Some(decoded_value) = decoded.get(key) {
            match value {
                serde_json::Value::Number(n) => {
                    assert_eq!(
                        decoded_value.as_u64(),
                        n.as_u64(),
                        "field '{key}' number mismatch in '{name}'"
                    );
                }
                serde_json::Value::String(s) => {
                    assert_eq!(
                        decoded_value.as_str().map(|s| s.to_uppercase()),
                        Some(s.to_uppercase()),
                        "field '{key}' string mismatch in '{name}'"
                    );
                }
                serde_json::Value::Object(_) => {
                    // For amount objects, verify the value field round-trips
                    if let (Some(orig_val), Some(dec_val)) = (
                        value.get("value").and_then(|v| v.as_str()),
                        decoded_value.get("value").and_then(|v| v.as_str()),
                    ) {
                        assert_eq!(
                            orig_val, dec_val,
                            "field '{key}.value' mismatch in '{name}'"
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

// ============================================================================
// Known-good serialization vectors from ripple-binary-codec
// ============================================================================

#[test]
fn vector_simple_xrp_payment() {
    let json = serde_json::json!({
        "Account": "r9TeThyi5xiuUUrFjtPKZiHcDxs7K9H6Rb",
        "Destination": "r4BPgS7DHebQiU31xWELvZawwSG2fSPJ7C",
        "TransactionType": "Payment",
        "Amount": "25000000",
        "Fee": "10",
        "Flags": 0u32,
        "Sequence": 2u32
    });

    assert_serialization(
        "Simple XRP Payment",
        &json,
        "120000220000000024000000026140000000017D784068400000000000000A81145CCB151F6E9D603F394AE778ACF10D3BECE874F68314E851BBBE79E328E43D68F43445368133DF5FBA5A",
    );
    assert_round_trip("Simple XRP Payment", &json);
}

#[test]
fn vector_xrp_payment_with_last_ledger_sequence() {
    let json = serde_json::json!({
        "Account": "rGWTUVmm1fB5QUjMYn8KfnyrFNgDiD9H9e",
        "Destination": "rw71Qs1UYQrSQ9hSgRohqNNQcyjCCfffkQ",
        "TransactionType": "Payment",
        "Amount": "200000",
        "Fee": "15",
        "Flags": 0u32,
        "Sequence": 144u32,
        "LastLedgerSequence": 6220218u32
    });

    assert_serialization(
        "XRP Payment with LastLedgerSequence",
        &json,
        "12000022000000002400000090201B005EE9BA614000000000030D4068400000000000000F8114AA1BD19D9E87BE8069FDBF6843653C43837C03C6831467FE6EC28E0464DD24FB2D62A492AAC697CFAD02",
    );
    assert_round_trip("XRP Payment with LastLedgerSequence", &json);
}

#[test]
fn vector_xrp_payment_with_destination_tag() {
    let json = serde_json::json!({
        "Account": "r4BPgS7DHebQiU31xWELvZawwSG2fSPJ7C",
        "Destination": "rBqSFEFg2B6GBMobtxnU1eLA1zbNC9NDGM",
        "TransactionType": "Payment",
        "Amount": "25000000",
        "Fee": "12",
        "Flags": 0u32,
        "Sequence": 1u32,
        "DestinationTag": 4146942154u64
    });

    assert_serialization(
        "XRP Payment with DestinationTag",
        &json,
        "120000220000000024000000012EF72D50CA6140000000017D784068400000000000000C8114E851BBBE79E328E43D68F43445368133DF5FBA5A831476DAC5E814CD4AA74142C3AB45E69A900E637AA2",
    );
    assert_round_trip("XRP Payment with DestinationTag", &json);
}

#[test]
fn vector_xrp_payment_with_source_tag() {
    let json = serde_json::json!({
        "Account": "rFLiPGytDEwC5heoqFcFAZoqPPmKBzX1o",
        "Destination": "rBsbetvMYuMkEeHZYizPMkpveCVH8EVQYd",
        "TransactionType": "Payment",
        "Amount": "500000",
        "Fee": "20",
        "SourceTag": 668920u32,
        "Sequence": 34954u32
    });

    assert_serialization(
        "XRP Payment with SourceTag",
        &json,
        "12000023000A34F8240000888A61400000000007A120684000000000000014811408F41F116A1F60D60296B16907F0A041BF10619783146E2F0455C46CF5DF61A1E58419A89D45459045EA",
    );
    assert_round_trip("XRP Payment with SourceTag", &json);
}

#[test]
fn vector_offer_create_with_iou() {
    let json = serde_json::json!({
        "TakerPays": "101204800",
        "Account": "rGFpans8aW7XZNEcNky6RHKyEdLvXPMnUn",
        "TransactionType": "OfferCreate",
        "Fee": "12",
        "Expiration": 1398443249u32,
        "TakerGets": {
            "currency": "CNY",
            "value": "4.2",
            "issuer": "rKiCet8SdvWxPXnAgYarFUXMh1zCPz432Y"
        },
        "Flags": 0u32,
        "Sequence": 6068u32
    });

    assert_serialization(
        "OfferCreate with IOU TakerGets",
        &json,
        "120007220000000024000017B42A535A8CF164400000000608434065D48EEBE0B40E8000000000000000000000000000434E590000000000CED6E99370D5C00EF4EBF72567DA99F5661BFB3A68400000000000000C8114AD6E583D47F90F29FD8B23225E6F905602B0292E",
    );
    assert_round_trip("OfferCreate with IOU TakerGets", &json);
}

#[test]
fn vector_offer_cancel() {
    let json = serde_json::json!({
        "Account": "rLpW9Reyn9YqZ8mxbq8nviXSp4TnHafVJQ",
        "TransactionType": "OfferCancel",
        "Fee": "12",
        "OfferSequence": 20763u32,
        "Flags": 0u32,
        "Sequence": 20769u32,
        "LastLedgerSequence": 6220009u32
    });

    assert_serialization(
        "OfferCancel",
        &json,
        "1200082200000000240000512120190000511B201B005EE8E968400000000000000C8114D0B32295596E50017E246FE85FC5982A1BD89CE4",
    );
    assert_round_trip("OfferCancel", &json);
}

#[test]
fn vector_trust_set() {
    let json = serde_json::json!({
        "Account": "rJMiz2rCMjZzEMijXNH1exNBryTQEjFd9S",
        "TransactionType": "TrustSet",
        "LimitAmount": {
            "currency": "WCG",
            "value": "10000000",
            "issuer": "rUx4xgE7bNWCCgGcXv1CCoQyTcCeZ275YG"
        },
        "Fee": "12",
        "Flags": 131072u32,
        "Sequence": 44u32
    });

    assert_serialization(
        "TrustSet",
        &json,
        "1200142200020000240000002C63D6438D7EA4C680000000000000000000000000005743470000000000832297BEF589D59F9C03A84F920F8D9128CC1CE468400000000000000C8114BE6C30732AE33CF2AF3344CE8172A6B9300183E3",
    );
    assert_round_trip("TrustSet", &json);
}

#[test]
fn vector_account_set_minimal() {
    let json = serde_json::json!({
        "Account": "rpP2GdsQwenNnFPefbXFgiTvEgJWQpq8Rw",
        "TransactionType": "AccountSet",
        "Fee": "10",
        "Flags": 0u32,
        "Sequence": 10598u32
    });

    assert_serialization(
        "AccountSet Minimal",
        &json,
        "1200032200000000240000296668400000000000000A81140F3D0C7D2CFAB2EC8295451F0B3CA038E8E9CDCD",
    );
    assert_round_trip("AccountSet Minimal", &json);
}

// ============================================================================
// Signed transaction with known hash
// ============================================================================

#[test]
fn vector_signed_offer_create_with_hash() {
    let json = serde_json::json!({
        "Account": "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys",
        "Expiration": 595640108u32,
        "Fee": "10",
        "Flags": 524288u32,
        "OfferSequence": 1752791u32,
        "Sequence": 1752792u32,
        "SigningPubKey": "03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3",
        "TakerGets": "15000000000",
        "TakerPays": {
            "currency": "USD",
            "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            "value": "7072.8"
        },
        "TransactionType": "OfferCreate",
        "TxnSignature": "30440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C"
    });

    let expected_binary = "120007220008000024001ABED82A2380BF2C2019001ABED764D55920AC9391400000000000000000000000000055534400000000000A20B3C85F482532A9578DBB3950B85CA06594D165400000037E11D60068400000000000000A732103EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3744630440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C8114DD76483FACDEE26E60D8A586BB58D09F27045C46";

    let expected_hash = "73734B611DDA23D3F5F62E20A173B78AB8406AC5015094DA53F53D39B9EDB06C";

    // Test serialization matches known binary
    assert_serialization("Signed OfferCreate", &json, expected_binary);

    // Test transaction ID computation
    let map = json.as_object().expect("JSON object");
    let tx_id = signing::transaction_id_hex(map).expect("transaction_id");
    assert_eq!(
        tx_id, expected_hash,
        "transaction ID mismatch for Signed OfferCreate"
    );

    // Test round-trip
    assert_round_trip("Signed OfferCreate", &json);
}

// ============================================================================
// Serializer -> Deserializer round-trip tests
// ============================================================================

#[test]
fn round_trip_iou_payment() {
    let json = serde_json::json!({
        "Account": "rHXUjUtk5eiPFYpg27izxHeZ1t4x835Ecn",
        "Destination": "r45dBj4S3VvMMYXxr9vHX4Z4Ma6ifPMCkK",
        "TransactionType": "Payment",
        "Amount": {
            "currency": "CNY",
            "value": "5000",
            "issuer": "r45dBj4S3VvMMYXxr9vHX4Z4Ma6ifPMCkK"
        },
        "Fee": "12",
        "SendMax": {
            "currency": "CNY",
            "value": "5050",
            "issuer": "rHXUjUtk5eiPFYpg27izxHeZ1t4x835Ecn"
        },
        "Flags": 0u32,
        "Sequence": 6u32,
        "DestinationTag": 736049272u32
    });

    assert_round_trip("IOU Payment", &json);
}

#[test]
fn round_trip_zero_amounts() {
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "TransactionType": "Payment",
        "Amount": "0",
        "Fee": "0",
        "Sequence": 0u32
    });

    assert_round_trip("Zero amounts", &json);
}

#[test]
fn round_trip_iou_zero_value() {
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "TransactionType": "TrustSet",
        "LimitAmount": {
            "currency": "USD",
            "value": "0",
            "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
        },
        "Fee": "12",
        "Sequence": 1u32
    });

    assert_round_trip("IOU zero value", &json);
}

#[test]
fn round_trip_with_memos() {
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "TransactionType": "Payment",
        "Amount": "1000000",
        "Fee": "12",
        "Sequence": 1u32,
        "Memos": [
            {
                "Memo": {
                    "MemoType": "746578742F706C61696E",
                    "MemoData": "48656C6C6F"
                }
            }
        ]
    });

    let map = json.as_object().expect("obj");
    let mut buf = Vec::new();
    serializer::serialize_json_object(map, &mut buf, false).expect("serialize");

    let decoded = deserializer::deserialize_object(&buf).expect("deserialize");

    // Verify memo survived
    let memos = decoded.get("Memos").expect("should have Memos");
    let arr = memos.as_array().expect("Memos should be array");
    assert_eq!(arr.len(), 1);

    let memo_wrapper = arr[0].as_object().expect("memo wrapper");
    let memo = memo_wrapper.get("Memo").expect("should have Memo").as_object().expect("Memo obj");
    assert_eq!(
        memo.get("MemoType").and_then(|v| v.as_str()),
        Some("746578742F706C61696E".to_uppercase().as_str())
    );
    assert_eq!(
        memo.get("MemoData").and_then(|v| v.as_str()),
        Some("48656C6C6F".to_uppercase().as_str())
    );
}

#[test]
fn round_trip_with_signing_fields() {
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "TransactionType": "Payment",
        "Amount": "1000000",
        "Fee": "12",
        "Sequence": 1u32,
        "SigningPubKey": "ED5F5AC43F527AE97194E860E5B28E6751B0B3BBEAC0780826AAF6DB9B3EE001",
        "TxnSignature": "AABBCCDDEE"
    });

    assert_round_trip("With signing fields", &json);
}

// ============================================================================
// Signing tests
// ============================================================================

#[test]
fn signing_data_produces_different_hash_than_tx_id() {
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "TransactionType": "Payment",
        "Amount": "1000000",
        "Fee": "12",
        "Sequence": 1u32,
        "TxnSignature": "AABBCCDD"
    });
    let map = json.as_object().expect("obj");

    let signing_hash = signing::signing_hash(map).expect("signing_hash");
    let tx_id = signing::transaction_id(map).expect("tx_id");

    // These must differ because:
    // 1. Different prefixes (STX\0 vs TXN\0)
    // 2. Signing data excludes TxnSignature
    assert_ne!(signing_hash, tx_id);
}

#[test]
fn multi_signing_differs_per_account() {
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "TransactionType": "Payment",
        "Amount": "1000000",
        "Fee": "12",
        "Sequence": 1u32,
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
    });
    let map = json.as_object().expect("obj");

    let signer_a = [0x11u8; 20];
    let signer_b = [0x22u8; 20];

    let hash_a = signing::multi_signing_hash(map, &signer_a).expect("a");
    let hash_b = signing::multi_signing_hash(map, &signer_b).expect("b");

    assert_ne!(hash_a, hash_b);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn max_xrp_amount() {
    // Maximum XRP is 100 billion drops = 100_000_000_000_000_000
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "TransactionType": "Payment",
        "Amount": "100000000000000000",
        "Fee": "12",
        "Sequence": 1u32
    });

    assert_round_trip("Max XRP amount", &json);
}

#[test]
fn empty_signing_pub_key() {
    // Empty SigningPubKey is valid (multi-signed transactions have empty SigningPubKey)
    let json = serde_json::json!({
        "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "TransactionType": "Payment",
        "Amount": "1000000",
        "Fee": "12",
        "Sequence": 1u32,
        "SigningPubKey": ""
    });

    assert_round_trip("Empty SigningPubKey", &json);
}

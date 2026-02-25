//! Build script that reads `definitions.json` and generates Rust constants
//! for XRPL field definitions, type codes, transaction types, and ledger entry types.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Definitions {
    #[serde(rename = "TYPES")]
    types: BTreeMap<String, i32>,
    #[serde(rename = "FIELDS")]
    fields: Vec<(String, FieldProps)>,
    #[serde(rename = "TRANSACTION_TYPES")]
    transaction_types: BTreeMap<String, i32>,
    #[serde(rename = "LEDGER_ENTRY_TYPES")]
    ledger_entry_types: BTreeMap<String, i32>,
    #[serde(rename = "TRANSACTION_RESULTS")]
    transaction_results: BTreeMap<String, i32>,
}

#[derive(Deserialize)]
struct FieldProps {
    #[serde(rename = "isSerialized")]
    is_serialized: bool,
    #[serde(rename = "isSigningField")]
    is_signing_field: bool,
    #[serde(rename = "isVLEncoded")]
    is_vl_encoded: bool,
    nth: i32,
    #[serde(rename = "type")]
    type_name: String,
}

fn to_screaming_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            let prev = name.chars().nth(i - 1).unwrap_or('_');
            if prev.is_lowercase() || prev.is_numeric() {
                result.push('_');
            } else if let Some(next) = name.chars().nth(i + 1) {
                if next.is_lowercase() {
                    result.push('_');
                }
            }
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}

fn main() {
    let defs_path = Path::new("src/definitions.json");
    println!("cargo::rerun-if-changed=src/definitions.json");

    let json = fs::read_to_string(defs_path).expect("Failed to read definitions.json");
    let defs: Definitions = serde_json::from_str(&json).expect("Failed to parse definitions.json");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = Path::new(&out_dir).join("generated_definitions.rs");

    let mut output = String::new();

    // --- Type code constants ---
    writeln!(output, "// Auto-generated from definitions.json. Do not edit.").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "// === Type Codes ===").unwrap();
    for (name, code) in &defs.types {
        if *code < 0 {
            continue; // Skip Unknown, Done, etc.
        }
        let const_name = format!("TYPE_{}", to_screaming_snake(name));
        writeln!(output, "pub const {const_name}: u16 = {code};").unwrap();
    }

    // --- FieldDef struct (already defined in definitions.rs, constants generated here) ---
    writeln!(output).unwrap();
    writeln!(output, "// === Field Definitions ===").unwrap();

    // Collect valid fields
    let mut valid_fields: Vec<(&str, &FieldProps, u16)> = Vec::new();
    for (name, props) in &defs.fields {
        if props.nth <= 0 {
            continue; // Skip Generic, Invalid, sentinel fields
        }
        let type_code = match defs.types.get(&props.type_name) {
            Some(&code) if code >= 0 => code as u16,
            _ => continue, // Skip fields with unknown/negative types
        };
        valid_fields.push((name.as_str(), props, type_code));
    }

    // Generate constants
    for (name, props, type_code) in &valid_fields {
        let const_name = format!("FIELD_{}", to_screaming_snake(name));
        let nth = props.nth as u16;
        let is_serialized = props.is_serialized;
        let is_signing_field = props.is_signing_field;
        let is_vl_encoded = props.is_vl_encoded;
        writeln!(
            output,
            r#"pub const {const_name}: FieldDef = FieldDef {{ name: "{name}", nth: {nth}, type_code: {type_code}, is_serialized: {is_serialized}, is_signing_field: {is_signing_field}, is_vl_encoded: {is_vl_encoded} }};"#,
        )
        .unwrap();
    }

    // Generate field_by_name lookup
    writeln!(output).unwrap();
    writeln!(output, "// === Field Lookup ===").unwrap();
    writeln!(
        output,
        "pub fn field_by_name(name: &str) -> Option<&'static FieldDef> {{"
    )
    .unwrap();
    writeln!(output, "    match name {{").unwrap();
    for (name, _, _) in &valid_fields {
        let const_name = format!("FIELD_{}", to_screaming_snake(name));
        writeln!(output, "        \"{name}\" => Some(&{const_name}),").unwrap();
    }
    writeln!(output, "        _ => None,").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();

    // Generate field_by_code lookup (type_code, field_code) -> FieldDef
    writeln!(output).unwrap();
    writeln!(
        output,
        "pub fn field_by_code(type_code: u16, field_code: u16) -> Option<&'static FieldDef> {{"
    )
    .unwrap();
    writeln!(output, "    match (type_code, field_code) {{").unwrap();
    for (name, _, type_code) in &valid_fields {
        let const_name = format!("FIELD_{}", to_screaming_snake(name));
        let nth = defs
            .fields
            .iter()
            .find(|(n, _)| n == *name)
            .map(|(_, p)| p.nth as u16)
            .unwrap_or(0);
        writeln!(
            output,
            "        ({type_code}, {nth}) => Some(&{const_name}),"
        )
        .unwrap();
    }
    writeln!(output, "        _ => None,").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();

    // --- Transaction type constants ---
    writeln!(output).unwrap();
    writeln!(output, "// === Transaction Types ===").unwrap();
    for (name, code) in &defs.transaction_types {
        if *code < 0 {
            continue;
        }
        let const_name = format!("TX_{}", to_screaming_snake(name));
        writeln!(output, "pub const {const_name}: u16 = {code};").unwrap();
    }

    // Generate tx_type_name lookup
    writeln!(output).unwrap();
    writeln!(
        output,
        "pub fn tx_type_name(code: u16) -> Option<&'static str> {{"
    )
    .unwrap();
    writeln!(output, "    match code {{").unwrap();
    for (name, code) in &defs.transaction_types {
        if *code < 0 {
            continue;
        }
        writeln!(output, "        {code} => Some(\"{name}\"),").unwrap();
    }
    writeln!(output, "        _ => None,").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();

    // Generate tx_type_code lookup
    writeln!(output).unwrap();
    writeln!(
        output,
        "pub fn tx_type_code(name: &str) -> Option<u16> {{"
    )
    .unwrap();
    writeln!(output, "    match name {{").unwrap();
    for (name, code) in &defs.transaction_types {
        if *code < 0 {
            continue;
        }
        writeln!(output, "        \"{name}\" => Some({code}),").unwrap();
    }
    writeln!(output, "        _ => None,").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();

    // --- Ledger entry type constants ---
    writeln!(output).unwrap();
    writeln!(output, "// === Ledger Entry Types ===").unwrap();
    for (name, code) in &defs.ledger_entry_types {
        if *code < 0 {
            continue;
        }
        let const_name = format!("LE_{}", to_screaming_snake(name));
        writeln!(output, "pub const {const_name}: u16 = {code};").unwrap();
    }

    // Generate le_type_code lookup
    writeln!(output).unwrap();
    writeln!(
        output,
        "pub fn le_type_code(name: &str) -> Option<u16> {{"
    )
    .unwrap();
    writeln!(output, "    match name {{").unwrap();
    for (name, code) in &defs.ledger_entry_types {
        if *code < 0 {
            continue;
        }
        writeln!(output, "        \"{name}\" => Some({code}),").unwrap();
    }
    writeln!(output, "        _ => None,").unwrap();
    writeln!(output, "    }}").unwrap();
    writeln!(output, "}}").unwrap();

    // --- Transaction result constants ---
    writeln!(output).unwrap();
    writeln!(output, "// === Transaction Results ===").unwrap();
    for (name, code) in &defs.transaction_results {
        let const_name = format!("RESULT_{}", to_screaming_snake(name));
        writeln!(output, "pub const {const_name}: i32 = {code};").unwrap();
    }

    fs::write(&out_path, output).expect("Failed to write generated definitions");
}

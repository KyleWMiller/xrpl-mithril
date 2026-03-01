//! Compile-time field definitions from `definitions.json`.
//!
//! This module contains constants for every XRPL field, type code,
//! transaction type, and ledger entry type. Generated at build time
//! from rippled's `definitions.json` (v3.1.0+).
//!
//! # Lookup functions
//!
//! - [`field_by_name`] — Look up a field definition by its JSON name
//! - [`field_by_code`] — Look up a field definition by (type_code, field_code)
//! - [`tx_type_name`] — Get the transaction type name from its numeric code
//! - [`tx_type_code`] — Get the transaction type code from its name

/// A field definition describing how a field is serialized in the XRPL binary format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDef {
    /// The field name as used in JSON (e.g., "Account", "Fee", "TransactionType").
    pub name: &'static str,
    /// The field code (nth value). Combined with type_code to form the field ID.
    pub nth: u16,
    /// The type code from the TYPES registry (e.g., 1=UInt16, 2=UInt32, 6=Amount).
    pub type_code: u16,
    /// Whether this field is included in binary serialization.
    pub is_serialized: bool,
    /// Whether this field is included in signing serialization.
    pub is_signing_field: bool,
    /// Whether this field uses variable-length (VL) encoding.
    pub is_vl_encoded: bool,
}

impl FieldDef {
    /// Returns the canonical sort key for binary serialization ordering.
    ///
    /// Fields are sorted by (type_code, nth) in ascending order.
    #[must_use]
    pub const fn sort_key(&self) -> (u16, u16) {
        (self.type_code, self.nth)
    }
}

// Include the generated constants, lookup functions, etc.
include!(concat!(env!("OUT_DIR"), "/generated_definitions.rs"));

/// Check if a field name corresponds to a signing field.
///
/// Returns `true` if the field exists in definitions and has
/// `is_signing_field == true`. Returns `false` if the field is unknown
/// or is not a signing field.
#[must_use]
pub fn is_signing_field(name: &str) -> bool {
    field_by_name(name).is_some_and(|def| def.is_signing_field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_type_field() {
        assert_eq!(FIELD_TRANSACTION_TYPE.type_code, TYPE_U_INT16);
        assert_eq!(FIELD_TRANSACTION_TYPE.nth, 2);
        assert!(FIELD_TRANSACTION_TYPE.is_serialized);
        assert!(FIELD_TRANSACTION_TYPE.is_signing_field);
    }

    #[test]
    fn flags_field() {
        assert_eq!(FIELD_FLAGS.type_code, TYPE_U_INT32);
        assert_eq!(FIELD_FLAGS.nth, 2);
        assert!(FIELD_FLAGS.is_serialized);
        assert!(FIELD_FLAGS.is_signing_field);
    }

    #[test]
    fn txn_signature_not_signing() {
        let field = field_by_name("TxnSignature").expect("should find TxnSignature");
        assert!(!field.is_signing_field);
        assert!(field.is_serialized);
        assert!(field.is_vl_encoded);
    }

    #[test]
    fn account_field() {
        let field = field_by_name("Account").expect("should find Account");
        assert_eq!(field.type_code, TYPE_ACCOUNT_ID);
        assert_eq!(field.nth, 1);
        assert!(field.is_vl_encoded);
    }

    #[test]
    fn field_by_code_lookup() {
        // TransactionType: type_code=1, nth=2
        let field = field_by_code(1, 2).expect("should find field (1,2)");
        assert_eq!(field.name, "TransactionType");
    }

    #[test]
    fn tx_type_lookups() {
        assert_eq!(tx_type_code("Payment"), Some(0));
        assert_eq!(tx_type_name(0), Some("Payment"));
        assert_eq!(tx_type_code("EscrowCreate"), Some(1));
    }

    #[test]
    fn fee_field() {
        let field = field_by_name("Fee").expect("should find Fee");
        assert_eq!(field.type_code, TYPE_AMOUNT);
        assert_eq!(field.nth, 8);
        assert!(field.is_serialized);
        assert!(field.is_signing_field);
    }
}

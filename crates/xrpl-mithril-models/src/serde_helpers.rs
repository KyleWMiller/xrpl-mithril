//! XRPL STArray wrapper-key serialization via the [`StArray<T>`] newtype.
//!
//! The XRPL protocol wraps STArray elements in single-key JSON objects.
//! For example, `Memos` serializes as:
//!
//! ```json
//! [{"Memo": {"MemoType": "...", "MemoData": "..."}}, ...]
//! ```
//!
//! Rather than annotating every STArray field with custom `serialize_with` /
//! `deserialize_with` attributes (which is easy to forget), this module
//! provides [`StArray<T>`] — a newtype around `Vec<T>` that handles the
//! wrapping automatically.
//!
//! Any type used as an STArray element implements [`StArrayElement`], which
//! declares its wrapper key. This makes the invariant **structural**: you
//! cannot forget the wrapper because it is part of the type.
//!
//! Deserialization accepts **both** wrapped and flat formats for robustness:
//! - Wrapped: server responses, binary codec round-trips
//! - Flat: user-constructed values, backwards compatibility

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Trait + Newtype
// ---------------------------------------------------------------------------

/// A type that can appear as an element in an XRPL STArray.
///
/// Each element type declares its JSON wrapper key (e.g. `"Memo"`, `"Signer"`).
/// This key is used automatically by [`StArray`] during serialization and
/// deserialization.
///
/// # Example
///
/// ```ignore
/// impl StArrayElement for Memo {
///     const WRAPPER_KEY: &'static str = "Memo";
/// }
/// ```
pub trait StArrayElement: Serialize + DeserializeOwned {
    /// The JSON wrapper key for this element type.
    const WRAPPER_KEY: &'static str;
}

/// A typed XRPL STArray that handles wrapper-key serialization automatically.
///
/// Use this instead of `Vec<T>` for any field whose XRPL type code is
/// STArray (15). The wrapper key comes from `T::WRAPPER_KEY`.
///
/// # Examples
///
/// ```ignore
/// // Construction
/// let memos: StArray<Memo> = StArray(vec![memo1, memo2]);
/// let memos: StArray<Memo> = vec![memo1, memo2].into();
///
/// // Access (via Deref)
/// assert_eq!(memos.len(), 2);
/// for memo in memos.iter() { /* ... */ }
///
/// // JSON output: [{"Memo": {...}}, {"Memo": {...}}]
/// let json = serde_json::to_value(&memos).unwrap();
/// ```
pub struct StArray<T: StArrayElement>(pub Vec<T>);

// --- Manual trait impls (cannot derive due to trait bound) -----------------

impl<T: StArrayElement + std::fmt::Debug> std::fmt::Debug for StArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StArray").field(&self.0).finish()
    }
}

impl<T: StArrayElement + Clone> Clone for StArray<T> {
    fn clone(&self) -> Self {
        StArray(self.0.clone())
    }
}

impl<T: StArrayElement + PartialEq> PartialEq for StArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: StArrayElement> Default for StArray<T> {
    fn default() -> Self {
        StArray(Vec::new())
    }
}

// --- Conversions -----------------------------------------------------------

impl<T: StArrayElement> From<Vec<T>> for StArray<T> {
    fn from(v: Vec<T>) -> Self {
        StArray(v)
    }
}

impl<T: StArrayElement> From<StArray<T>> for Vec<T> {
    fn from(st: StArray<T>) -> Self {
        st.0
    }
}

// --- Deref for transparent Vec access --------------------------------------

impl<T: StArrayElement> std::ops::Deref for StArray<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: StArrayElement> std::ops::DerefMut for StArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --- IntoIterator ----------------------------------------------------------

impl<T: StArrayElement> IntoIterator for StArray<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: StArrayElement> IntoIterator for &'a StArray<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// ---------------------------------------------------------------------------
// Serde
// ---------------------------------------------------------------------------

impl<T: StArrayElement> Serialize for StArray<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in &self.0 {
            let value =
                serde_json::to_value(item).map_err(serde::ser::Error::custom)?;
            let mut wrapper = serde_json::Map::with_capacity(1);
            wrapper.insert(T::WRAPPER_KEY.to_string(), value);
            seq.serialize_element(&wrapper)?;
        }
        seq.end()
    }
}

impl<'de, T: StArrayElement> Deserialize<'de> for StArray<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let arr: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
        let mut result = Vec::with_capacity(arr.len());
        for elem in arr {
            let item =
                unwrap_element::<T>(&elem, T::WRAPPER_KEY).map_err(serde::de::Error::custom)?;
            result.push(item);
        }
        Ok(StArray(result))
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Unwrap an element from either wrapped or flat format.
///
/// If the element is `{"WrapperKey": {inner...}}`, extracts the inner object.
/// If the element is `{inner fields...}` (flat), passes it through directly.
fn unwrap_element<T: DeserializeOwned>(
    value: &serde_json::Value,
    wrapper_key: &str,
) -> Result<T, String> {
    // Try wrapped format: {"WrapperKey": {inner fields...}}
    if let Some(obj) = value.as_object() {
        if obj.len() == 1 {
            if let Some(inner) = obj.get(wrapper_key) {
                return serde_json::from_value(inner.clone())
                    .map_err(|e| format!("failed to deserialize wrapped {wrapper_key}: {e}"));
            }
        }
    }
    // Fall back to flat format: {inner fields...}
    serde_json::from_value(value.clone())
        .map_err(|e| format!("failed to deserialize flat element: {e}"))
}

// ---------------------------------------------------------------------------
// UInt64 string-or-number deserialization
// ---------------------------------------------------------------------------

/// Serde helper for `Option<u64>` fields that may appear as a JSON number or
/// a JSON string in the XRPL protocol.
///
/// XRPL represents `UInt64` fields as strings in JSON to avoid JavaScript
/// precision loss, but some contexts (transaction submission, docs examples)
/// use plain numbers. This module accepts both on deserialization and
/// serializes as a number.
///
/// Usage:
/// ```ignore
/// #[serde(
///     default,
///     skip_serializing_if = "Option::is_none",
///     deserialize_with = "crate::serde_helpers::option_uint64::deserialize"
/// )]
/// pub my_field: Option<u64>,
/// ```
pub mod option_uint64 {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<serde_json::Value>::deserialize(deserializer)?;
        match value {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Number(n)) => n
                .as_u64()
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom("number is not a valid u64")),
            Some(serde_json::Value::String(s)) => s
                .parse::<u64>()
                .map(Some)
                .map_err(|e| serde::de::Error::custom(format!("invalid u64 string: {e}"))),
            Some(_) => Err(serde::de::Error::custom("expected number, string, or null")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Minimal test element type.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestEntry {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Value")]
        value: u32,
    }

    impl StArrayElement for TestEntry {
        const WRAPPER_KEY: &'static str = "TestEntry";
    }

    #[test]
    fn serialize_wraps_elements() {
        let arr: StArray<TestEntry> = StArray(vec![
            TestEntry { name: "a".into(), value: 1 },
            TestEntry { name: "b".into(), value: 2 },
        ]);

        let json = serde_json::to_value(&arr).unwrap();
        let expected = json!([
            {"TestEntry": {"Name": "a", "Value": 1}},
            {"TestEntry": {"Name": "b", "Value": 2}},
        ]);
        assert_eq!(json, expected);
    }

    #[test]
    fn deserialize_from_wrapped() {
        let json = json!([
            {"TestEntry": {"Name": "x", "Value": 10}},
            {"TestEntry": {"Name": "y", "Value": 20}},
        ]);

        let arr: StArray<TestEntry> = serde_json::from_value(json).unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].name, "x");
        assert_eq!(arr[1].value, 20);
    }

    #[test]
    fn deserialize_from_flat() {
        let json = json!([
            {"Name": "x", "Value": 10},
            {"Name": "y", "Value": 20},
        ]);

        let arr: StArray<TestEntry> = serde_json::from_value(json).unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].name, "x");
    }

    #[test]
    fn option_none_roundtrip() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Outer {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            entries: Option<StArray<TestEntry>>,
        }

        let outer = Outer { entries: None };
        let json = serde_json::to_value(&outer).unwrap();
        assert_eq!(json, json!({}));

        let back: Outer = serde_json::from_value(json).unwrap();
        assert!(back.entries.is_none());
    }

    #[test]
    fn option_some_roundtrip() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Outer {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            entries: Option<StArray<TestEntry>>,
        }

        let outer = Outer {
            entries: Some(StArray(vec![TestEntry { name: "z".into(), value: 99 }])),
        };
        let json = serde_json::to_value(&outer).unwrap();
        let expected = json!({
            "entries": [{"TestEntry": {"Name": "z", "Value": 99}}]
        });
        assert_eq!(json, expected);

        let back: Outer = serde_json::from_value(json).unwrap();
        assert_eq!(back, outer);
    }

    #[test]
    fn deref_provides_vec_access() {
        let arr: StArray<TestEntry> = vec![
            TestEntry { name: "a".into(), value: 1 },
        ]
        .into();

        // Vec methods via Deref
        assert_eq!(arr.len(), 1);
        assert!(!arr.is_empty());
        assert_eq!(arr[0].name, "a");

        // Iteration via IntoIterator
        let names: Vec<_> = arr.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["a"]);
    }

    #[test]
    fn empty_array_roundtrip() {
        let arr: StArray<TestEntry> = StArray(vec![]);
        let json = serde_json::to_value(&arr).unwrap();
        assert_eq!(json, json!([]));

        let back: StArray<TestEntry> = serde_json::from_value(json).unwrap();
        assert!(back.is_empty());
    }

    // -----------------------------------------------------------------------
    // STArray coverage tests — cross-validate against definitions.json
    // -----------------------------------------------------------------------
    //
    // These tests verify that every STArray field modeled in xrpl-models
    // corresponds to a real STArray (type_code=15) in the XRPL protocol
    // definitions, and that each wrapper key is a valid STObject (type_code=14).
    //
    // The full list of STArray fields from definitions.json (rippled v3.1.0):
    //   Signers, SignerEntries, Template, Necessary, Sufficient,
    //   AffectedNodes, Memos, NFTokens, Hooks, VoteSlots, AdditionalBooks,
    //   Majorities, DisabledValidators, HookExecutions, HookParameters,
    //   HookGrants, XChainClaimAttestations, XChainCreateAccountAttestations,
    //   PriceDataSeries, AuthAccounts, AuthorizeCredentials,
    //   UnauthorizeCredentials, AcceptedCredentials, Permissions,
    //   RawTransactions, BatchSigners
    //
    // Of these, the subset modeled as typed fields in xrpl-models:

    /// (field_name, wrapper_key) pairs for every StArray field we model.
    /// If you add a new StArray field to any struct, add it here.
    const MODELED_ST_ARRAY_FIELDS: &[(&str, &str)] = &[
        ("Memos", "Memo"),
        ("Signers", "Signer"),
        ("SignerEntries", "SignerEntry"),
        ("PriceDataSeries", "PriceData"),
        ("AuthAccounts", "AuthAccount"),
        ("NFTokens", "NFToken"),
        ("VoteSlots", "VoteEntry"),
    ];

    #[test]
    fn all_modeled_fields_are_st_array_in_definitions() {
        for (field_name, _wrapper_key) in MODELED_ST_ARRAY_FIELDS {
            let def = xrpl_mithril_codec::definitions::field_by_name(field_name)
                .unwrap_or_else(|| panic!("{field_name} not found in definitions.json"));
            assert_eq!(
                def.type_code, 15,
                "field '{field_name}' should be STArray (type_code=15), got {}",
                def.type_code
            );
        }
    }

    // -----------------------------------------------------------------------
    // option_uint64 tests
    // -----------------------------------------------------------------------

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Uint64Holder {
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "crate::serde_helpers::option_uint64::deserialize"
        )]
        value: Option<u64>,
    }

    #[test]
    fn option_uint64_from_number() {
        let holder: Uint64Holder = serde_json::from_value(json!({"value": 12345})).unwrap();
        assert_eq!(holder.value, Some(12345));
    }

    #[test]
    fn option_uint64_from_string() {
        let holder: Uint64Holder = serde_json::from_value(json!({"value": "67890"})).unwrap();
        assert_eq!(holder.value, Some(67890));
    }

    #[test]
    fn option_uint64_absent() {
        let holder: Uint64Holder = serde_json::from_value(json!({})).unwrap();
        assert_eq!(holder.value, None);
    }

    #[test]
    fn option_uint64_null() {
        let holder: Uint64Holder = serde_json::from_value(json!({"value": null})).unwrap();
        assert_eq!(holder.value, None);
    }

    #[test]
    fn option_uint64_round_trip() {
        let holder = Uint64Holder { value: Some(999) };
        let json = serde_json::to_value(&holder).unwrap();
        assert_eq!(json, json!({"value": 999}));
        let back: Uint64Holder = serde_json::from_value(json).unwrap();
        assert_eq!(back, holder);
    }

    #[test]
    fn wrapper_keys_are_st_object_in_definitions() {
        for (_field_name, wrapper_key) in MODELED_ST_ARRAY_FIELDS {
            let def = xrpl_mithril_codec::definitions::field_by_name(wrapper_key)
                .unwrap_or_else(|| panic!("wrapper key '{wrapper_key}' not found in definitions.json"));
            assert_eq!(
                def.type_code, 14,
                "wrapper key '{wrapper_key}' should be STObject (type_code=14), got {}",
                def.type_code
            );
        }
    }
}

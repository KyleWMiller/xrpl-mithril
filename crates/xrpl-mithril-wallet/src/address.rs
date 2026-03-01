//! Address derivation and X-address encoding.
//!
//! Provides functions to derive an [`AccountId`] from a public key and to
//! encode/decode X-addresses (XLS-0005).
//!
//! # AccountId Derivation
//!
//! ```text
//! account_id = RIPEMD-160(SHA-256(public_key_bytes))
//! ```
//!
//! Where `public_key_bytes` is:
//! - secp256k1: 33-byte compressed SEC1 point
//! - Ed25519: `0xED` prefix + 32-byte public key (33 bytes total)

use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use xrpl_mithril_types::AccountId;

use crate::error::WalletError;

/// Derive an [`AccountId`] from a public key.
///
/// The public key should be 33 bytes:
/// - secp256k1: compressed SEC1 encoding (starts with `0x02` or `0x03`)
/// - Ed25519: `0xED` prefix + 32-byte key
///
/// # Examples
///
/// ```
/// use xrpl_mithril_wallet::{Seed, Algorithm};
/// use xrpl_mithril_wallet::address::derive_account_id;
///
/// let seed = Seed::from_passphrase("masterpassphrase");
/// let keypair = seed.derive_keypair(Algorithm::Secp256k1).unwrap();
/// let account_id = derive_account_id(keypair.public_key());
/// assert_eq!(account_id.to_classic_address(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// ```
#[must_use]
pub fn derive_account_id(public_key: &[u8]) -> AccountId {
    let sha_hash = Sha256::digest(public_key);
    let ripe_hash = Ripemd160::digest(sha_hash);
    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&ripe_hash);
    AccountId::from_bytes(bytes)
}

// ---------------------------------------------------------------------------
// X-address encoding/decoding (XLS-0005)
// ---------------------------------------------------------------------------

/// X-address version bytes for mainnet.
const X_ADDRESS_PREFIX_MAIN: [u8; 2] = [0x05, 0x44];

/// X-address version bytes for testnet.
const X_ADDRESS_PREFIX_TEST: [u8; 2] = [0x04, 0x93];

/// Payload length: 2 (prefix) + 20 (account_id) + 1 (flags) + 8 (tag, zero-padded).
const X_ADDRESS_PAYLOAD_LEN: usize = 2 + 20 + 1 + 8;

/// Encode an [`AccountId`] as an X-address (XLS-0005).
///
/// X-addresses encode an account ID, an optional destination tag, and a
/// network flag (mainnet vs testnet) into a single base58check string
/// starting with `X` (mainnet) or `T` (testnet).
///
/// # Examples
///
/// ```
/// use xrpl_mithril_types::AccountId;
/// use xrpl_mithril_wallet::address::encode_x_address;
///
/// let account = AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
/// let x_addr = encode_x_address(&account, None, false);
/// assert!(x_addr.starts_with('X'));
/// ```
#[must_use]
pub fn encode_x_address(account_id: &AccountId, tag: Option<u32>, is_test: bool) -> String {
    let prefix = if is_test {
        X_ADDRESS_PREFIX_TEST
    } else {
        X_ADDRESS_PREFIX_MAIN
    };

    let mut payload = [0u8; X_ADDRESS_PAYLOAD_LEN];
    payload[0..2].copy_from_slice(&prefix);
    payload[2..22].copy_from_slice(account_id.as_bytes());

    match tag {
        Some(t) => {
            // Flags byte: 1 = has tag
            payload[22] = 0x01;
            // Tag as 8-byte little-endian (only first 4 bytes used, rest zero)
            payload[23..27].copy_from_slice(&t.to_le_bytes());
            // payload[27..31] remains zero
        }
        None => {
            // Flags byte: 0 = no tag
            payload[22] = 0x00;
            // Tag bytes all zero
        }
    }

    bs58::encode(&payload)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check()
        .into_string()
}

/// Decode an X-address into its components: account ID, optional tag, and
/// test network flag.
///
/// # Examples
///
/// ```
/// use xrpl_mithril_types::AccountId;
/// use xrpl_mithril_wallet::address::{encode_x_address, decode_x_address};
///
/// let account = AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
/// let x_addr = encode_x_address(&account, Some(12345), false);
///
/// let (decoded_id, tag, is_test) = decode_x_address(&x_addr).unwrap();
/// assert_eq!(decoded_id, account);
/// assert_eq!(tag, Some(12345));
/// assert!(!is_test);
/// ```
///
/// # Errors
///
/// Returns [`WalletError::InvalidSeed`] if the string is not a valid
/// X-address.
pub fn decode_x_address(
    x_address: &str,
) -> Result<(AccountId, Option<u32>, bool), WalletError> {
    let decoded = bs58::decode(x_address)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check(None)
        .into_vec()
        .map_err(|e| WalletError::InvalidSeed(format!("invalid X-address encoding: {e}")))?;

    if decoded.len() != X_ADDRESS_PAYLOAD_LEN {
        return Err(WalletError::InvalidSeed(format!(
            "invalid X-address length: expected {X_ADDRESS_PAYLOAD_LEN}, got {}",
            decoded.len()
        )));
    }

    let prefix = [decoded[0], decoded[1]];
    let is_test = if prefix == X_ADDRESS_PREFIX_MAIN {
        false
    } else if prefix == X_ADDRESS_PREFIX_TEST {
        true
    } else {
        return Err(WalletError::InvalidSeed(
            "invalid X-address prefix bytes".into(),
        ));
    };

    let account_id = AccountId::from_slice(&decoded[2..22]).map_err(WalletError::from)?;

    let flags = decoded[22];
    let tag = if flags == 0x01 {
        let mut tag_bytes = [0u8; 4];
        tag_bytes.copy_from_slice(&decoded[23..27]);
        // Verify remaining bytes are zero
        if decoded[27..31] != [0, 0, 0, 0] {
            return Err(WalletError::InvalidSeed(
                "X-address tag padding bytes are non-zero".into(),
            ));
        }
        Some(u32::from_le_bytes(tag_bytes))
    } else if flags == 0x00 {
        // Verify all tag bytes are zero
        if decoded[23..31] != [0, 0, 0, 0, 0, 0, 0, 0] {
            return Err(WalletError::InvalidSeed(
                "X-address has no-tag flag but non-zero tag bytes".into(),
            ));
        }
        None
    } else {
        return Err(WalletError::InvalidSeed(format!(
            "invalid X-address flags byte: {flags:#04x}"
        )));
    };

    Ok((account_id, tag, is_test))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_genesis_account_id() {
        // Genesis account public key (secp256k1, compressed)
        // This is derived from the "masterpassphrase" seed
        let seed = crate::seed::Seed::from_passphrase("masterpassphrase");
        let keypair = seed
            .derive_keypair(crate::algorithm::Algorithm::Secp256k1)
            .expect("derive keypair");
        let account_id = derive_account_id(keypair.public_key());
        assert_eq!(
            account_id.to_classic_address(),
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        );
    }

    #[test]
    fn x_address_round_trip_no_tag() {
        let account =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
                .expect("valid address");
        let x_addr = encode_x_address(&account, None, false);
        assert!(x_addr.starts_with('X'));

        let (decoded_id, decoded_tag, decoded_test) =
            decode_x_address(&x_addr).expect("decode X-address");
        assert_eq!(decoded_id, account);
        assert_eq!(decoded_tag, None);
        assert!(!decoded_test);
    }

    #[test]
    fn x_address_round_trip_with_tag() {
        let account =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
                .expect("valid address");
        let tag = 12345u32;
        let x_addr = encode_x_address(&account, Some(tag), false);

        let (decoded_id, decoded_tag, decoded_test) =
            decode_x_address(&x_addr).expect("decode X-address");
        assert_eq!(decoded_id, account);
        assert_eq!(decoded_tag, Some(tag));
        assert!(!decoded_test);
    }

    #[test]
    fn x_address_testnet() {
        let account =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
                .expect("valid address");
        let x_addr = encode_x_address(&account, None, true);
        // Testnet X-addresses start with 'T'
        assert!(x_addr.starts_with('T'));

        let (_, _, is_test) = decode_x_address(&x_addr).expect("decode");
        assert!(is_test);
    }

    #[test]
    fn x_address_max_tag() {
        let account =
            AccountId::from_classic_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
                .expect("valid address");
        let x_addr = encode_x_address(&account, Some(u32::MAX), false);

        let (_, decoded_tag, _) = decode_x_address(&x_addr).expect("decode");
        assert_eq!(decoded_tag, Some(u32::MAX));
    }

    #[test]
    fn invalid_x_address_rejected() {
        assert!(decode_x_address("not-an-x-address").is_err());
        assert!(decode_x_address("").is_err());
    }
}

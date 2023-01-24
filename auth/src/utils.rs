/*
 * Axelar ETH utils
 *
 */
use ethabi::decode;
use ethabi::encode;
use ethabi::Address;
use ethabi::ParamType;
use ethabi::Token;
use primitive_types::H256;
use sha3::{Digest, Keccak256};
use std::str::FromStr;
use uint::hex;

pub fn ecrecover(hash: H256, signature: &[u8]) -> Result<Address, ()> {
    assert_eq!(signature.len(), 65);

    let hash = secp256k1::Message::parse_slice(hash.as_bytes()).unwrap();
    let v = signature[64];
    let signature = secp256k1::Signature::parse_slice(&signature[0..64]).unwrap();
    let bit = match v {
        0..=26 => v,
        _ => v - 27,
    };

    if let Ok(recovery_id) = secp256k1::RecoveryId::parse(bit) {
        if let Ok(public_key) = secp256k1::recover(&hash, &signature, &recovery_id) {
            // recover returns a 65-byte key, but addresses come from the raw 64-byte key
            let r = sha3::Keccak256::digest(&public_key.serialize()[1..]);
            return Ok(Address::from_slice(&r[12..]));
        }
    }

    Err(())
}

/// Hash a message according to EIP-191.
///
/// The data is a UTF-8 encoded string and will enveloped as follows:
/// `"\x19Ethereum Signed Message:\n" + message.length + message` and hashed
/// using keccak256.
pub fn hash_message<S>(message: S) -> H256
where
    S: AsRef<[u8]>,
{
    keccak256(&prefix_message(message)).into()
}

/// Prefix a message according to EIP-191.
///
/// The data is a UTF-8 encoded string and will enveloped as follows:
/// `"\x19Ethereum Signed Message:\n" + message.length + message`.
pub fn prefix_message<S>(message: S) -> Vec<u8>
where
    S: AsRef<[u8]>,
{
    const PREFIX: &str = "\x19Ethereum Signed Message:\n32";
    let message = message.as_ref();
    let mut eth_message = format!("{}{}", PREFIX, message.len()).into_bytes();
    eth_message.extend_from_slice(message);
    eth_message
}

/// It takes a slice of bytes and returns a 32-byte hash
/// Compute the Keccak-256 hash of input bytes.
///
/// Panics if the computed hash is not the expected length (32 bytes).
///
/// Arguments:
///
/// * `bytes`: The bytes to hash.
///
/// Returns:
///
/// A 32 byte array
pub fn keccak256<S>(bytes: S) -> [u8; 32]
where
    S: AsRef<[u8]>,
{
    let hash = Keccak256::digest(bytes.as_ref());
    let hash: [u8; 32] = hash
        .as_slice()
        .try_into()
        .expect("hash is not the correct length");
    hash
}

/// Wrapped functionalities

pub fn sign_message<S>(message: S) -> String
where
    S: AsRef<[u8]>,
{
    let hash = hash_message(message);
    let full_hash = format!("{:#x}", hash);
    full_hash
}

/// It takes a byte array and a list of expected output types, and returns a list of tokens
///
/// Arguments:
///
/// * `data`: The data to decode.
/// * `expected_output_types`: The types of the values that are expected to be returned.
///
/// Returns:
///
/// A vector of tokens.
pub fn abi_decode(data: &[u8], expected_output_types: &[ParamType]) -> Result<Vec<Token>, String> {
    match decode(expected_output_types, data) {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(format!("Error decoding ABI-encoded data: {:?}", e)),
    }
}

/// It takes a vector of tokens and returns a vector of bytes
///
/// Arguments:
///
/// * `tokens`: A vector of tokens to encode.
///
/// Returns:
///
/// A vector of bytes.
pub fn abi_encode(tokens: Vec<Token>) -> Vec<u8> {
    encode(&tokens)
}

pub fn clean_payload(payload: String) -> Vec<u8> {
    let clean_payload = &payload[2..payload.len()];
    hex::decode(clean_payload).unwrap()
}

pub fn to_h256(payload: String) -> H256 {
    let clean_payload = &payload[2..payload.len()];
    H256::from_str(clean_payload).unwrap()
}

pub fn to_eth_hex_string(payload: [u8; 32]) -> String {
    format!("0x{}", hex::encode(payload))
}

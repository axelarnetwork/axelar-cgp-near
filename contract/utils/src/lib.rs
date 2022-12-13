/*
 * Axelar ETH utils
 *
 */

use ethabi::decode;
use ethabi::param_type::Reader;
use ethabi::token::LenientTokenizer;
use ethabi::token::StrictTokenizer;
use ethabi::ParamType;
use ethabi::Token;
use k256::ecdsa::{recoverable, signature::Signature, VerifyingKey};
use k256::elliptic_curve::rand_core::Error;
use primitive_types::H256;
use sha3::{Digest, Keccak256};
use uint::hex;

pub fn recover(message: &[u8], signature: &[u8]) -> VerifyingKey {
    let actual_signature = recoverable::Signature::from_bytes(signature).unwrap();
    let recovered_key = actual_signature
        .recover_verifying_key(message)
        .expect("couldn't recover pubkey");

    recovered_key
}

pub fn to_verifying_key(pubkey: [u8; 32]) -> VerifyingKey {
    let mut pubkey_bytes = [0u8; 65];
    pubkey_bytes[0] = 4;
    pubkey_bytes[1..65].copy_from_slice(&pubkey);
    let pubkey = VerifyingKey::from_sec1_bytes(&pubkey_bytes).unwrap();
    pubkey
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

/// Compute the Keccak-256 hash of input bytes.
///
/// Panics if the computed hash is not the expected length (32 bytes).
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
pub fn sign_message(message: String) -> String {
    let hash = hash_message(message);
    let full_hash = format!("{:#x}", hash);
    full_hash
}

pub fn abi_decode(payload: Vec<u8>, types: &[String]) -> Result<Vec<String>, ethabi::Error> {
    let types: Vec<ParamType> = types
        .iter()
        .map(|s| Reader::read(s))
        .collect::<Result<_, _>>()?;

    let payload_bytes = &hex::decode(&payload).unwrap();
    let result = decode(&types, &payload_bytes);
    assert_eq!(result.is_ok(), true);
    let values = result.unwrap();

    let result = values
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>();

    Ok(result)
}

// TODO: Implement json structure for input instead of the message that will be able to map to the Tokens required for ABI encoding
pub fn abi_encode(message: String) -> String {
    // assert_eq!(params.len() % 2, 0);

    // let params = params
    //     .iter()
    //     .tuples::<(_, _)>()
    //     .map(|(x, y)| Reader::read(x).map(|z| (z, y.as_str())))
    //     .collect::<Result<Vec<_>, _>>()?;

    // let tokens = parse_tokens(params.as_slice(), lenient)?;
    // let result = encode(&tokens);

    // Ok(hex::encode(result))
    "".to_string()
}

// /*
//  * Axelar ETH utils
//  *
//  */
// use ethabi::decode;
// use ethabi::encode;
// use ethabi::ParamType;
// use ethabi::Token;
// use k256::ecdsa::{recoverable, signature::Signature, VerifyingKey};
// use primitive_types::H256;
// use sha3::{Digest, Keccak256};

// pub fn recover(message: &[u8], signature: &[u8]) -> VerifyingKey {
//     let actual_signature = recoverable::Signature::from_bytes(signature).unwrap();
//     let recovered_key = actual_signature
//         .recover_verifying_key(message)
//         .expect("couldn't recover pubkey");

//     recovered_key
// }

// pub fn to_verifying_key(pubkey: [u8; 20]) -> VerifyingKey {
//     let mut pubkey_bytes = [0u8; 65];
//     pubkey_bytes[0] = 4;
//     pubkey_bytes[1..65].copy_from_slice(&pubkey);
//     let pubkey = VerifyingKey::from_sec1_bytes(&pubkey_bytes).unwrap();
//     pubkey
// }

// /// Hash a message according to EIP-191.
// ///
// /// The data is a UTF-8 encoded string and will enveloped as follows:
// /// `"\x19Ethereum Signed Message:\n" + message.length + message` and hashed
// /// using keccak256.
// pub fn hash_message<S>(message: S) -> H256
// where
//     S: AsRef<[u8]>,
// {
//     keccak256(&prefix_message(message)).into()
// }

// /// Prefix a message according to EIP-191.
// ///
// /// The data is a UTF-8 encoded string and will enveloped as follows:
// /// `"\x19Ethereum Signed Message:\n" + message.length + message`.
// pub fn prefix_message<S>(message: S) -> Vec<u8>
// where
//     S: AsRef<[u8]>,
// {
//     const PREFIX: &str = "\x19Ethereum Signed Message:\n32";
//     let message = message.as_ref();
//     let mut eth_message = format!("{}{}", PREFIX, message.len()).into_bytes();
//     eth_message.extend_from_slice(message);
//     eth_message
// }

// /// It takes a slice of bytes and returns a 32-byte hash
// /// Compute the Keccak-256 hash of input bytes.
// ///
// /// Panics if the computed hash is not the expected length (32 bytes).
// ///
// /// Arguments:
// ///
// /// * `bytes`: The bytes to hash.
// ///
// /// Returns:
// ///
// /// A 32 byte array
// pub fn keccak256<S>(bytes: S) -> [u8; 32]
// where
//     S: AsRef<[u8]>,
// {
//     let hash = Keccak256::digest(bytes.as_ref());
//     let hash: [u8; 32] = hash
//         .as_slice()
//         .try_into()
//         .expect("hash is not the correct length");
//     hash
// }

// /// Wrapped functionalities

// /// It takes a message, hashes it, and returns the hash as a hexadecimal string
// ///
// /// Arguments:
// ///
// /// * `message`: The message to be signed.
// ///
// /// Returns:
// ///
// /// The hash of the message
// pub fn sign_message(message: String) -> String {
//     let hash = hash_message(message);
//     let full_hash = format!("{:#x}", hash);
//     full_hash
// }

// /// It takes a byte array and a list of expected output types, and returns a list of tokens
// ///
// /// Arguments:
// ///
// /// * `data`: The data to decode.
// /// * `expected_output_types`: The types of the values that are expected to be returned.
// ///
// /// Returns:
// ///
// /// A vector of tokens.
// pub fn abi_decode(data: &[u8], expected_output_types: &[ParamType]) -> Result<Vec<Token>, String> {
//     match decode(expected_output_types, data) {
//         Ok(tokens) => Ok(tokens),
//         Err(e) => Err(format!("Error decoding ABI-encoded data: {:?}", e)),
//     }
// }

// /// It takes a vector of tokens and returns a vector of bytes
// ///
// /// Arguments:
// ///
// /// * `tokens`: A vector of tokens to encode.
// ///
// /// Returns:
// ///
// /// A vector of bytes.
// pub fn abi_encode(tokens: Vec<Token>) -> Vec<u8> {
//     encode(&tokens)
// }

/*
 * Axelar gateway contract
 *
 */

use ethabi::decode;
use ethabi::ParamType;
use ethabi::Token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use uint::hex;

pub const TGAS: u64 = 1_000_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Gateway {}

impl Default for Gateway {
    fn default() -> Self {
        Self {}
    }
}

#[near_bindgen]
impl Gateway {
    pub fn sign_message(&self, message: String) -> String {
        let hash = utils::hash_message(message);
        let full_hash = format!("{:#x}", hash);
        full_hash
    }

    pub fn abi_decode(&self, payload: String) -> Vec<String> {
        let payload_bytes = &hex::decode(&payload).unwrap();
        let result = decode(&[ParamType::String], &payload_bytes);
        assert_eq!(result.is_ok(), true);
        let values = result.unwrap();

        let result = values
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>();

        result
    }

    pub fn abi_encode(&self, message: String) -> String {
        let payload = ethabi::encode(&[Token::String(message.to_string())]);
        hex::encode(payload)
    }
}

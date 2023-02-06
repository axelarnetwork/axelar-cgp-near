use crate::events::OperatorshipTransferredEvent;
use crate::utils::{self, abi_encode, clean_payload, to_h256};
use crate::{utils::abi_decode, utils::keccak256, Axelar, AxelarExt};
use ethabi::ethereum_types::H160;
use ethabi::{Address, Token};
use near_contract_tools::owner::*;
use near_sdk::env;

use ethabi::ParamType;
use near_contract_tools::standard::nep297::Event;
use near_sdk::near_bindgen;
use primitive_types::H256;

/// A constant that is used to determine how many epochs old keys are valid for.
pub const OLD_KEY_RETENTION: u8 = 16;

/// Axelar Authentication Weighted Implementation
#[near_bindgen]
impl Axelar {
    /// It takes an epoch number and returns the hash of the block that was mined at that epoch
    ///
    /// Arguments:
    ///
    /// * `epoch`: The epoch number for which you want to get the hash.
    ///
    /// Returns:
    ///
    /// The hash of the block at the given epoch.
    pub fn hash_for_epoch(&self, epoch: u64) -> String {
        let hash = self.hash_for_epoch.get(&epoch).unwrap();
        utils::to_eth_hex_string(hash)
    }

    /// `epoch_for_hash` returns the epoch number for a given hash
    ///
    /// Arguments:
    ///
    /// * `hash`: The hash of the block to get the epoch for.
    ///
    /// Returns:
    ///
    /// The epoch for the hash.
    pub fn epoch_for_hash(&self, hash: String) -> u64 {
        let hash: [u8; 32] = clean_payload(hash).try_into().unwrap();
        self.epoch_for_hash.get(&hash).unwrap()
    }

    /// If the epoch of the operators is the same as the current epoch, and the epoch of the operators
    /// is not too old, then validate the signatures
    ///
    /// Arguments:
    ///
    /// * `message_hash`: The hash of the message to be signed.
    /// * `proof`: The proof that is being validated.
    ///
    /// Returns:
    ///
    /// A boolean value.
    pub fn validate_proof(&self, message_hash: String, proof: String) -> bool {
        let expected_output_types = vec![
            ParamType::Array(Box::new(ParamType::Address)),
            ParamType::Array(Box::new(ParamType::Uint(256))),
            ParamType::Uint(256),
            ParamType::Array(Box::new(ParamType::Bytes)),
        ];

        let payload = clean_payload(proof.clone());

        let tokens = abi_decode(&payload, &expected_output_types).unwrap();

        let (operators, weights, threshold, signatures) = (
            tokens[0].clone().into_array().unwrap(),
            tokens[1].clone().into_array().unwrap(),
            tokens[2].clone().into_uint().unwrap(),
            tokens[3].clone().into_array().unwrap(),
        );

        let encoded_operators = abi_encode(vec![
            Token::Array(operators.clone()),
            Token::Array(weights.clone()),
            Token::Uint(threshold.clone()),
        ]);

        let operators_hash = keccak256(&encoded_operators);
        let operators_epoch = self.epoch_for_hash.get(&operators_hash).unwrap();
        let epoch = self.current_epoch;

        if operators_epoch == 0 || epoch - operators_epoch >= OLD_KEY_RETENTION.into() {
            env::panic_str("Invalid epoch")
        }

        self.internal_validate_signatures(
            to_h256(message_hash),
            operators
                .clone()
                .into_iter()
                .map(|x| x.into_address().unwrap())
                .collect(),
            weights
                .clone()
                .into_iter()
                .map(|x| x.into_uint().unwrap().as_u32())
                .collect(),
            threshold.as_u32(),
            signatures.clone(),
        );

        operators_epoch == epoch
    }

    /// Only owner

    /// `transfer_operatorship` is a public function that requires the caller to be the owner, and then
    /// calls the internal function `internal_transfer_operatorship`
    ///
    /// Arguments:
    ///
    /// * `params`: Vec<u8>
    #[payable]
    pub fn transfer_operatorship(&mut self, params: String) -> bool {
        Self::require_owner();
        self.internal_transfer_operatorship(clean_payload(params))
    }

    /// Internal

    /// It takes in a list of addresses and a list of weights, and if the list of addresses is sorted
    /// and contains no duplicates, and if the list of weights is the same length as the list of
    /// addresses, and if the sum of the weights is greater than the threshold, then it emits an event
    ///
    /// Arguments:
    ///
    /// * `params`: The parameters passed to the function.
    pub(crate) fn internal_transfer_operatorship(&mut self, params: Vec<u8>) -> bool {
        let expected_output_types = vec![
            ParamType::Array(Box::new(ParamType::Address)),
            ParamType::Array(Box::new(ParamType::Uint(256))),
            ParamType::Uint(256),
        ];

        let tokens = abi_decode(&params, &expected_output_types).unwrap();

        let new_operators = tokens[0]
            .clone()
            .into_array()
            .unwrap()
            .into_iter()
            .map(|token| token.into_address().unwrap())
            .collect::<Vec<_>>();

        let new_weights = tokens[1]
            .clone()
            .into_array()
            .unwrap()
            .into_iter()
            .map(|token| token.into_uint().unwrap())
            .collect::<Vec<_>>();

        let new_threshold = tokens[2].clone().into_uint().unwrap();

        let operators_length = new_operators.len();
        let weights_length = new_weights.len();

        if operators_length == 0
            || !Axelar::internal_is_sorted_asc_and_contains_no_duplicate(new_operators.clone())
        {
            env::panic_str("Invalid operators");
        }

        if weights_length != operators_length {
            env::panic_str("Invalid weights");
        }

        let mut total_weight: u32 = 0;

        for i in 0..weights_length {
            total_weight += new_weights[i].low_u32();
        }

        if new_threshold.low_u32() == 0 || total_weight < new_threshold.low_u32() {
            env::panic_str("Invalid threshold");
        }

        let new_operators_hash = keccak256(params);

        let existing_epoch = self.epoch_for_hash.get(&new_operators_hash).unwrap_or(0);

        if existing_epoch > 0 {
            env::panic_str("Duplicate operators");
        }

        let epoch = self.current_epoch + 1;
        self.current_epoch = epoch;
        self.hash_for_epoch.insert(&epoch, &new_operators_hash);
        self.epoch_for_hash.insert(&new_operators_hash, &epoch);

        // Emit event
        let event = OperatorshipTransferredEvent {
            new_operators: new_operators
                .iter()
                .map(|x| format!("{:#x}", x))
                .collect::<Vec<_>>(),
            new_weights: new_weights.iter().map(|x| x.as_u64()).collect::<Vec<_>>(),
            new_threshold: new_threshold.as_u64(),
        };

        event.emit();

        true
    }

    /// It takes a list of operators, a list of weights, a threshold, and a list of signatures, and it
    /// checks that the signatures are valid
    ///
    /// Arguments:
    ///
    /// * `message_hash`: The hash of the message to be signed.
    /// * `operators`: The list of operators that are allowed to sign the transaction.
    /// * `weights`: The weight of each operator.
    /// * `threshold`: The minimum number of signatures required to validate the transaction.
    /// * `signatures`: A list of signatures.
    fn internal_validate_signatures(
        &self,
        message_hash: H256,
        operators: Vec<Address>,
        weights: Vec<u32>,
        threshold: u32,
        signatures: Vec<Token>,
    ) {
        let operator_length = operators.len();
        let mut operator_index = 0;
        let mut weight = 0;

        for i in 0..signatures.len() {
            let signature: &[u8] = &signatures[i].clone().into_bytes().unwrap();

            let signer = utils::ecrecover(message_hash, signature).unwrap();

            while operator_index < operator_length && operators[operator_index] != signer {
                operator_index += 1;
            }

            if operator_index >= operator_length {
                env::panic_str(
                    format!(
                        "Malformed signers. Operators {}",
                        operators
                            .iter()
                            .map(|x| format!("\"{}\"", x))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                    .as_str(),
                );
            }

            weight += weights[operator_index];

            if weight >= threshold {
                return;
            }

            operator_index += 1;
        }

        env::panic_str("Low signature weight");
    }

    /// > This function checks if the given vector of accounts is sorted in ascending order and contains
    /// no duplicate
    ///
    /// Arguments:
    ///
    /// * `accounts`: A vector of H160, which is a type of vector of 20 bytes.
    ///
    /// Returns:
    ///
    /// A boolean value.
    fn internal_is_sorted_asc_and_contains_no_duplicate(accounts: Vec<H160>) -> bool {
        for i in 0..(accounts.len() - 1) {
            if accounts[i] >= accounts[i + 1] {
                return false;
            }
        }

        return !accounts[0].is_zero();
    }
}

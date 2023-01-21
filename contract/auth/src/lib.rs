/*
 * Axelar Auth contract
 *
 */

// mod events;
// mod utils;

use ethabi::decode;
use ethabi::ParamType;
use ethabi::Token;

// use ethabi::ethereum_types::H160;
// use ethabi::{ParamType, Token};
// use events::OperatorshipTransferredEvent;
// use near_contract_tools::standard::nep297::Event;
// use near_contract_tools::{owner::*, Owner};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::predecessor_account_id;
use near_sdk::{near_bindgen, AccountId};
// use utils::{abi_decode, abi_encode, keccak256};

/// A constant that is used to determine how many epochs old keys are valid for.
pub const OLD_KEY_RETENTION: u8 = 16;

/// `AxelarAuthWeighted` is a struct that contains a `current_epoch` field, a `hash_for_epoch` field,
/// and an `epoch_for_hash` field.
///
/// Properties:
///
/// * `current_epoch`: The current epoch number.
/// * `hash_for_epoch`: A map of epochs to the hash of the block that was produced at that epoch.
/// * `epoch_for_hash`: This is a map that maps a hash to an epoch.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AxelarAuthWeighted {
    current_epoch: u64,
    // hash_for_epoch: LookupMap<u64, [u8; 32]>,
    // epoch_for_hash: LookupMap<[u8; 32], u64>,
}

/// This is a default implementation of the `AxelarAuthWeighted` struct.
// impl Default for AxelarAuthWeighted {
//     fn default() -> Self {
//         Self {
//             current_epoch: 0,
//             // hash_for_epoch: LookupMap::new(b"hash_for_epoch".to_vec()),
//             // epoch_for_hash: LookupMap::new(b"epoch_for_hash".to_vec()),
//         }
//     }
// }

#[near_bindgen]
impl AxelarAuthWeighted {
    /// `new` is called when the contract is first deployed, and it initializes the contract's state
    ///
    /// Arguments:
    ///
    /// * `recent_operators`: A list of account IDs that will be given operator status.
    ///
    /// Returns:
    ///
    /// The contract is being returned.
    #[init]
    pub fn new(recent_operators: AccountId) -> Self {
        let mut contract = Self {
            current_epoch: 0,
            // hash_for_epoch: LookupMap::new(b"hash_for_epoch".to_vec()),
            // epoch_for_hash: LookupMap::new(b"epoch_for_hash".to_vec()),
        };

        // Owner::init(&mut contract, &predecessor_account_id());

        // for operator in recent_operators {
        //     contract.transfer_operatorship(operator);
        // }

        contract
    }

    // /// > If the epoch of the operators is the same as the current epoch, and the epoch of the operators
    // /// is not too old, then validate the signatures
    // ///
    // /// Arguments:
    // ///
    // /// * `message_hash`: The hash of the message to be signed.
    // /// * `proof`: The proof that is being validated.
    // ///
    // /// Returns:
    // ///
    // /// A boolean value.
    // pub fn validate_proof(&self, message_hash: Box<[u8; 32]>, proof: Box<[u8]>) -> bool {
    //     let expected_output_types = vec![
    //         ParamType::Array(Box::new(ParamType::Address)),
    //         ParamType::Array(Box::new(ParamType::Uint(256))),
    //         ParamType::Uint(256),
    //         ParamType::Array(Box::new(ParamType::Bytes)),
    //     ];

    //     let tokens = abi_decode(&proof, &expected_output_types).unwrap();

    //     let (operators, weights, threshold, signatures) = (
    //         tokens[0].clone().into_array().unwrap(),
    //         tokens[1].clone().into_array().unwrap(),
    //         tokens[2].clone().into_uint().unwrap(),
    //         tokens[3].clone().into_array().unwrap(),
    //     );

    //     let encoded_operators = abi_encode(vec![
    //         Token::Array(operators.clone()),
    //         Token::Array(weights.clone()),
    //         Token::Uint(threshold.clone()),
    //     ]);

    //     let operators_hash = keccak256(&encoded_operators);
    //     let operators_epoch = self.epoch_for_hash.get(&operators_hash).unwrap();
    //     let epoch = self.current_epoch;

    //     if operators_epoch == 0 || epoch - operators_epoch >= OLD_KEY_RETENTION.into() {
    //         return false;
    //     }

    //     self.internal_validate_signatures(
    //         message_hash,
    //         operators
    //             .clone()
    //             .into_iter()
    //             .map(|x| x.into_address().unwrap())
    //             .collect(),
    //         weights
    //             .clone()
    //             .into_iter()
    //             .map(|x| x.into_uint().unwrap().as_u32())
    //             .collect(),
    //         threshold.as_u32(),
    //         signatures.clone(),
    //     );

    //     operators_epoch == epoch
    // }

    // /// Only owner

    // /// `transfer_operatorship` is a public function that requires the caller to be the owner, and then
    // /// calls the internal function `internal_transfer_operatorship`
    // ///
    // /// Arguments:
    // ///
    // /// * `params`: Vec<u8>
    // pub fn transfer_operatorship(&mut self, params: Vec<u8>) {
    //     Self::require_owner();
    //     self.internal_transfer_operatorship(params);
    // }

    // /// Internal

    // /// It takes in a list of addresses and a list of weights, and if the list of addresses is sorted
    // /// and contains no duplicates, and if the list of weights is the same length as the list of
    // /// addresses, and if the sum of the weights is greater than the threshold, then it emits an event
    // ///
    // /// Arguments:
    // ///
    // /// * `params`: The parameters passed to the function.
    // fn internal_transfer_operatorship(&mut self, params: Vec<u8>) {
    //     let expected_output_types = vec![
    //         ParamType::Array(Box::new(ParamType::Address)),
    //         ParamType::Array(Box::new(ParamType::Uint(256))),
    //         ParamType::Uint(256),
    //     ];

    //     let tokens = abi_decode(&params, &expected_output_types).unwrap();

    //     let new_operators = tokens[0]
    //         .clone()
    //         .into_array()
    //         .unwrap()
    //         .into_iter()
    //         .map(|token| token.into_address().unwrap())
    //         .collect::<Vec<_>>();

    //     let new_weights = tokens[1]
    //         .clone()
    //         .into_array()
    //         .unwrap()
    //         .into_iter()
    //         .map(|token| token.into_uint().unwrap())
    //         .collect::<Vec<_>>();

    //     let new_threshold = tokens[2].clone().into_uint().unwrap();

    //     let operators_length = new_operators.len();
    //     let weights_length = new_weights.len();

    //     if operators_length == 0
    //         || !self.internal_is_sorted_asc_and_contains_no_duplicate(new_operators.clone())
    //     {
    //         assert!(false, "Invalid operators");
    //     }

    //     if weights_length != operators_length {
    //         assert!(false, "Invalid weights");
    //     }

    //     let mut total_weight: u32 = 0;

    //     for i in 0..weights_length {
    //         total_weight += new_weights[i].low_u32();
    //     }

    //     if new_threshold.low_u32() == 0 || total_weight < new_threshold.low_u32() {
    //         assert!(false, "Invalid threshold");
    //     }

    //     let new_operators_hash = keccak256(params);

    //     if self
    //         .epoch_for_hash
    //         .get(&new_operators_hash)
    //         .expect("No epoch for provided hash")
    //         > 0
    //     {
    //         assert!(false, "Duplicate operators");
    //     }

    //     let epoch = self.current_epoch + 1;
    //     self.current_epoch = epoch;
    //     self.hash_for_epoch.insert(&epoch, &new_operators_hash);
    //     self.epoch_for_hash.insert(&new_operators_hash, &epoch);

    //     // Emit event
    //     let event = OperatorshipTransferredEvent {
    //         new_operators: format!(
    //             "[{}]",
    //             new_operators
    //                 .iter()
    //                 .map(|x| format!("\"{}\"", x))
    //                 .collect::<Vec<_>>()
    //                 .join(","),
    //         ),
    //         new_weights: format!(
    //             "[{}]",
    //             new_weights
    //                 .iter()
    //                 .map(|x| format!("{}", x))
    //                 .collect::<Vec<_>>()
    //                 .join(","),
    //         ),
    //         new_threshold: format!("{}", new_threshold),
    //     };

    //     event.emit();
    // }

    // /// It takes a list of operators, a list of weights, a threshold, and a list of signatures, and it
    // /// checks that the signatures are valid
    // ///
    // /// Arguments:
    // ///
    // /// * `message_hash`: The hash of the message to be signed.
    // /// * `operators`: The list of operators that are allowed to sign the transaction.
    // /// * `weights`: The weight of each operator.
    // /// * `threshold`: The minimum number of signatures required to validate the transaction.
    // /// * `signatures`: A list of signatures.
    // fn internal_validate_signatures(
    //     &self,
    //     message_hash: Box<[u8; 32]>,
    //     operators: Vec<Address>,
    //     weights: Vec<u32>,
    //     threshold: u32,
    //     signatures: Vec<Token>,
    // ) {
    //     let operator_length = operators.len();
    //     let mut operator_index = 0;
    //     let mut weight = 0;

    //     for i in 0..signatures.len() {
    //         let signature = signatures[i].clone().into_bytes().unwrap();
    //         let signer = utils::recover(&*message_hash, &signature);

    //         while operator_index < operator_length
    //             && utils::to_verifying_key(operators[operator_index].0) != signer
    //         {
    //             operator_index += 1;
    //         }

    //         if operator_index >= operator_length {
    //             panic!("Malformed signers");
    //         }

    //         weight += weights[operator_index];

    //         if weight >= threshold {
    //             return;
    //         }

    //         operator_index += 1;
    //     }

    //     assert!(weight < threshold, "Total weight is less than threshold");
    // }

    // /// > This function checks if the given vector of accounts is sorted in ascending order and contains
    // /// no duplicate
    // ///
    // /// Arguments:
    // ///
    // /// * `accounts`: A vector of H160, which is a type of vector of 20 bytes.
    // ///
    // /// Returns:
    // ///
    // /// A boolean value.
    // fn internal_is_sorted_asc_and_contains_no_duplicate(&mut self, accounts: Vec<H160>) -> bool {
    //     for i in 0..(accounts.len() - 1) {
    //         if accounts[i] >= accounts[i + 1] {
    //             return false;
    //         }
    //     }

    //     return !accounts[0].is_zero();
    // }
}

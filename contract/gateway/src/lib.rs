/*
 * Axelar gateway contract
 *
 */
mod events;
pub mod ext_traits;

pub use crate::ext_traits::*;

use ethabi::{ParamType, Token};
use events::{
    ContractCallApprovedEvent, ContractCallEvent, ExecutedEvent, OperatorshipTransferredEvent,
};
use near_contract_tools::standard::nep297::Event;
use near_contract_tools::{owner::*, Owner};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::predecessor_account_id;
use near_sdk::env::{self, keccak256};
use near_sdk::{near_bindgen, AccountId, Gas};
use utils::{abi_decode, abi_encode};

/// Defining a constant named TGAS with a value of 1,000,000,000,000.
pub const TGAS: u64 = 1_000_000_000_000;
/// Defining a constant string called SELECTOR_APPROVE_CONTRACT_CALL.
pub const SELECTOR_APPROVE_CONTRACT_CALL: &str = "approveContractCall";
/// Defining a constant string.
pub const SELECTOR_TRANSFER_OPERATORSHIP: &str = "transferOperatorship";

/// `AxelarGateway` is a struct that contains a `auth_module` field of type `AccountId`, a
/// `prefix_command_executed` field of type `Vec<u8>`, a `prefix_contract_call_approved` field of type
/// `Vec<u8>`, and a `bool_state` field of type `LookupMap<Vec<u8>, bool>`.
///
/// The `auth_module` field is the account ID of the account that will be used to authorize commands.
///
/// The `prefix_
///
/// Properties:
///
/// * `auth_module`: The account ID of the module that will be used to authorize commands.
/// * `prefix_command_executed`: This is the prefix that will be used to store the state of the command
/// executed.
/// * `prefix_contract_call_approved`: This is the prefix that will be used to store the approved
/// contract calls.
/// * `bool_state`: This is a map that stores the state of the contract.
#[near_bindgen]
#[derive(Owner, BorshDeserialize, BorshSerialize)]
pub struct AxelarGateway {
    auth_module: AccountId,
    prefix_command_executed: Vec<u8>,
    prefix_contract_call_approved: Vec<u8>,
    bool_state: LookupMap<Vec<u8>, bool>,
}

impl Default for AxelarGateway {
    /// `default()` is a function that returns a `Self` struct
    ///
    /// Returns:
    ///
    /// A new instance of the `AxelarAuth` struct.
    fn default() -> Self {
        Self {
            auth_module: AccountId::new_unchecked("axelar-auth.testnet".to_string()),
            prefix_command_executed: keccak256(b"command-executed"),
            prefix_contract_call_approved: keccak256(b"contract-call-approved"),
            bool_state: LookupMap::new(b"bool_state".to_vec()),
        }
    }
}

#[near_bindgen]
impl AxelarGateway {
    /// `new` is the constructor of the contract
    ///
    /// Arguments:
    ///
    /// * `auth_module`: The account id of the auth module.
    ///
    /// Returns:
    ///
    /// The contract is being returned.
    #[init]
    pub fn new(auth_module: AccountId) -> Self {
        let mut contract = Self {
            auth_module,
            prefix_command_executed: keccak256(b"command-executed"),
            prefix_contract_call_approved: keccak256(b"contract-call-approved"),
            bool_state: LookupMap::new(b"bool_state".to_vec()),
        };

        Owner::init(&mut contract, &predecessor_account_id());

        contract
    }

    /// It emits a `ContractCallEvent` event with the current account ID, the destination chain, the
    /// destination contract address, the payload hash, and the payload
    ///
    /// Arguments:
    ///
    /// * `destination_chain`: The chain that the contract is on.
    /// * `destination_contract_address`: The address of the contract you want to call.
    /// * `payload`: The payload to be sent to the destination contract.
    pub fn call_contract(
        destination_chain: String,
        destination_contract_address: String,
        payload: Vec<u8>,
    ) {
        let payload_hash = keccak256(&payload);
        let event = ContractCallEvent {
            address: predecessor_account_id().to_string(),
            destination_chain,
            destination_contract_address,
            payload_hash,
            payload,
        };
        Event::emit(&event);
    }

    // Execute command function
    ///
    /// The first part of the function is calling the `validate_proof` function of the
    /// `ext_auth_contract` module. This function takes in two parameters, the message hash and the
    /// proof. The message hash is the hash of the data that was signed by
    ///
    /// Arguments:
    ///
    /// * `input`: The input to the contract.
    pub fn execute(&mut self, input: Vec<u8>) {
        let tokens = abi_decode(&input, &vec![ParamType::Bytes, ParamType::Bytes]).unwrap();

        let data = tokens[0].clone().into_bytes().unwrap();
        let proof = tokens[0].clone().into_bytes().unwrap();

        let binding = utils::hash_message(keccak256(&data));
        let message_hash = binding.as_bytes().clone();

        ext_auth_contract::ext(self.auth_module.clone())
            .with_static_gas(Gas(5 * TGAS))
            .validate_proof(
                Box::new(message_hash.clone().try_into().unwrap()),
                Box::<[u8; 64]>::new(proof.clone().try_into().unwrap()).clone(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .validate_proof_callback(data),
            );
    }

    /// It takes a proof, validates it, and then executes the commands in the proof
    ///
    /// Arguments:
    ///
    /// * `data`: The data that was returned from the contract call.
    /// * `call_result`: Result<String, near_sdk::PromiseError> - This is the result of the callback.
    /// The callback is called with the result of the promise. The result is either a string or a
    /// PromiseError.
    #[private]
    pub fn validate_proof_callback(
        &mut self,
        data: Vec<u8>,
        #[callback_result] call_result: Result<String, near_sdk::PromiseError>,
    ) {
        if call_result.is_err() {
            env::panic_str("Failed to validate proof");
        }

        let mut allow_operatorship_transfer = call_result.unwrap().parse::<bool>().unwrap_or(false);

        let expected_output_types = vec![
            ParamType::Uint(256),
            ParamType::Array(Box::new(ParamType::FixedBytes(32))),
            ParamType::Array(Box::new(ParamType::String)),
            ParamType::Array(Box::new(ParamType::Bytes)),
        ];

        let data_tokens = abi_decode(&data, &expected_output_types).unwrap();

        let chain_id = data_tokens[0].clone().into_uint().unwrap().as_u64();
        let command_ids = data_tokens[1]
            .clone()
            .into_array()
            .unwrap()
            .into_iter()
            .map(|token| token.into_fixed_bytes().unwrap())
            .collect::<Vec<_>>();

        let commands = data_tokens[2]
            .clone()
            .into_array()
            .unwrap()
            .into_iter()
            .map(|token| token.into_string().unwrap())
            .collect::<Vec<_>>();

        let params = data_tokens[3]
            .clone()
            .into_array()
            .unwrap()
            .into_iter()
            .map(|token| token.into_bytes().unwrap())
            .collect::<Vec<_>>();

        // TODO: Update to NEAR chain id which we need to decide on
        if chain_id != 0 {
            env::panic_str("Invalid chain id");
        }

        let commands_length = command_ids.len();

        if commands_length != commands.len() || commands_length != params.len() {
            env::panic_str("Invalid commands");
        }

        for i in 0..commands_length {
            let command_id: [u8; 32] = command_ids[i].clone().try_into().unwrap();

            if self.is_command_executed(command_id) {
                continue;
            }

            let command = commands[i].clone();

            let success = true; // TODO: Figure this one out

            match command.as_str() {
                "approveContractCall" => {
                    self.internal_set_command_executed(command_id, true);
                    self.approve_contract_call(params[i].clone(), command_id);
                }
                "transferOperatorship" => {
                    if !allow_operatorship_transfer {
                        continue;
                    }

                    allow_operatorship_transfer = false;
                    self.internal_set_command_executed(command_id, true);
                    self.transfer_operatorship(params[i].clone(), command_id);
                }
                _ => {
                    continue;
                }
            };

            if success {
                let event = ExecutedEvent {
                    command_id: command_ids[i].clone(),
                };

                Event::emit(&event);
            } else {
                self.internal_set_command_executed(command_id, false);
            }
        }
    }

    /// `transfer_operatorship` transfers the operatorship of the contract to a new operator
    ///
    /// Arguments:
    ///
    /// * `new_operators_data`: The new operators data.
    /// * `_command_id`: The command id of the command that is being executed.
    pub fn transfer_operatorship(&self, new_operators_data: Vec<u8>, _command_id: [u8; 32]) {
        Self::require_owner();

        ext_auth_contract::ext(self.auth_module.clone())
            .with_static_gas(Gas(5 * TGAS))
            .transfer_operatorship(new_operators_data.clone());

        let event = OperatorshipTransferredEvent {
            new_operators_data: new_operators_data.clone(),
        };

        Event::emit(&event);
    }

    // Only Owner functions
    /// `approve_contract_call` is a function that is called by the owner of the contract to approve a
    /// contract call
    ///
    /// Arguments:
    ///
    /// * `params`: Vec<u8> - The parameters passed to the contract.
    /// * `command_id`: The command ID of the contract call.
    pub fn approve_contract_call(&mut self, params: Vec<u8>, command_id: [u8; 32]) {
        Self::require_owner();

        let expected_output_types = vec![
            ParamType::String,
            ParamType::String,
            ParamType::Address,
            ParamType::FixedBytes(32),
            ParamType::FixedBytes(32),
            ParamType::Uint(256),
        ];

        let tokens = abi_decode(&params, &expected_output_types).unwrap();

        let source_chain = tokens[0].clone().into_string().unwrap();
        let source_address = tokens[1].clone().into_string().unwrap();
        let contract_address = tokens[2].clone().into_address().unwrap().to_string();
        let payload_hash = tokens[3].clone().into_fixed_bytes().unwrap();
        let source_tx_hash = tokens[4].clone().into_fixed_bytes().unwrap();
        let source_event_index = tokens[5].clone().into_uint().unwrap().as_u64();

        self.internal_set_contract_call_approved(
            command_id.clone(),
            source_chain.clone(),
            source_address.clone(),
            contract_address.clone(),
            payload_hash.clone().try_into().unwrap(),
        );

        let event = ContractCallApprovedEvent {
            command_id: command_id.to_vec(),
            source_chain,
            source_address,
            contract_address,
            payload_hash,
            source_tx_hash,
            source_event_index,
        };

        Event::emit(&event);
    }

    // View functions

    /// It returns a boolean value indicating whether a contract call has been approved
    ///
    /// Arguments:
    ///
    /// * `command_id`: The command ID of the contract call.
    /// * `source_chain`: The chain that the contract call originated from.
    /// * `source_address`: The address of the contract that is calling the target contract.
    /// * `contract_address`: The address of the contract that is being called.
    /// * `payload_hash`: The hash of the payload that was sent to the contract.
    ///
    /// Returns:
    ///
    /// A boolean value.
    pub fn is_contract_call_approved(
        &self,
        command_id: [u8; 32],
        source_chain: String,
        source_address: String,
        contract_address: String,
        payload_hash: [u8; 32],
    ) -> bool {
        let key = self.internal_get_is_contract_call_approved_key(
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        );

        self.bool_state.get(&key).unwrap_or(false)
    }

    /// `auth_module` returns the `AccountId` of the `auth_module` of the `ContractInfo` struct
    ///
    /// Returns:
    ///
    /// The auth_module field of the struct.
    pub fn auth_module(&self) -> AccountId {
        self.auth_module.clone()
    }

    /// `is_command_executed` returns `true` if the command with the given `command_id` has been
    /// executed, and `false` otherwise
    ///
    /// Arguments:
    ///
    /// * `command_id`: The command ID of the command you want to check.
    ///
    /// Returns:
    ///
    /// A boolean value.
    pub fn is_command_executed(&self, command_id: [u8; 32]) -> bool {
        let key = self.internal_get_is_command_executed_key(command_id);
        self.bool_state.get(&key).unwrap_or(false)
    }

    // Payable functions

    // TODO: Add payable check and memory usage check to return tokens to user if overpaid
    /// If the contract call is approved, then set the approval to false and return true
    ///
    /// Arguments:
    ///
    /// * `command_id`: The ID of the command that is being validated.
    /// * `source_chain`: The chain that the contract call is coming from.
    /// * `source_address`: The address of the contract that is calling the function.
    /// * `payload_hash`: The hash of the payload that will be sent to the contract.
    ///
    /// Returns:
    ///
    /// A boolean value.
    #[payable]
    pub fn validate_contract_call(
        &mut self,
        command_id: [u8; 32],
        source_chain: String,
        source_address: String,
        payload_hash: [u8; 32],
    ) -> bool {
        let key = self.internal_get_is_contract_call_approved_key(
            command_id,
            source_chain,
            source_address,
            predecessor_account_id().to_string(),
            payload_hash,
        );

        let valid = self.bool_state.get(&key).unwrap_or(false);

        if valid {
            self.bool_state.insert(&key, &false);
        }

        valid
    }

    // Internal functions

    /// `internal_get_is_command_executed_key` is a function that takes a command_id as an argument and
    /// returns a vector of bytes
    ///
    /// Arguments:
    ///
    /// * `command_id`: The ID of the command that we want to check if it's executed or not.
    ///
    /// Returns:
    ///
    /// The keccak256 hash of the encoded prefix_command_executed and command_id.
    fn internal_get_is_command_executed_key(&self, command_id: [u8; 32]) -> Vec<u8> {
        let encoded = abi_encode(vec![
            Token::Bytes(self.prefix_command_executed.clone()),
            Token::FixedBytes(command_id.to_vec()),
        ]);

        keccak256(&encoded)
    }

    /// It takes a bunch of parameters and returns a `Vec<u8>` that is the keccak256 hash of the
    /// concatenation of the prefix and the parameters
    ///
    /// Arguments:
    ///
    /// * `command_id`: The command ID of the contract call.
    /// * `source_chain`: The chain that the contract call originated from.
    /// * `source_address`: The address of the contract that is calling the target contract.
    /// * `contract_address`: The address of the contract that is being called.
    /// * `payload_hash`: The hash of the payload that was sent to the contract.
    ///
    /// Returns:
    ///
    /// The keccak256 hash of the encoded data.
    fn internal_get_is_contract_call_approved_key(
        &self,
        command_id: [u8; 32],
        source_chain: String,
        source_address: String,
        contract_address: String,
        payload_hash: [u8; 32],
    ) -> Vec<u8> {
        let encoded = abi_encode(vec![
            Token::Bytes(self.prefix_contract_call_approved.clone()),
            Token::FixedBytes(command_id.to_vec()),
            Token::String(source_chain),
            Token::String(source_address),
            Token::String(contract_address),
            Token::FixedBytes(payload_hash.to_vec()),
        ]);

        keccak256(&encoded)
    }

    /// > This function sets the value of the `bool_state` map to `true` or `false` depending on the
    /// value of the `executed` parameter
    ///
    /// Arguments:
    ///
    /// * `command_id`: The command ID of the command that was executed.
    /// * `executed`: bool - whether the command has been executed or not
    fn internal_set_command_executed(&mut self, command_id: [u8; 32], executed: bool) {
        let key = self.internal_get_is_command_executed_key(command_id);
        self.bool_state.insert(&key, &executed);
    }

    /// It stores a boolean value in the state
    ///
    /// Arguments:
    ///
    /// * `command_id`: The command ID of the contract call.
    /// * `source_chain`: The chain that the contract call originated from.
    /// * `source_address`: The address of the contract that is calling the target contract.
    /// * `contract_address`: The address of the contract that is being called.
    /// * `payload_hash`: The hash of the payload that was sent to the contract.
    fn internal_set_contract_call_approved(
        &mut self,
        command_id: [u8; 32],
        source_chain: String,
        source_address: String,
        contract_address: String,
        payload_hash: [u8; 32],
    ) {
        let key = self.internal_get_is_contract_call_approved_key(
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        );
        self.bool_state.insert(&key, &true);
    }
}

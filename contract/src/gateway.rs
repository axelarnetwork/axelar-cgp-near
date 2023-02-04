use crate::events::{ContractCallApprovedEvent, ContractCallEvent, ExecutedEvent};
use crate::utils::{self, abi_encode, clean_payload};
use crate::{utils::abi_decode, utils::keccak256, Axelar, AxelarExt};
use ethabi::Token;
use near_contract_tools::owner::*;
use near_sdk::env::{self};

use ethabi::ParamType;
use near_contract_tools::standard::nep297::Event;
use near_sdk::env::predecessor_account_id;
use near_sdk::{near_bindgen, AccountId};
use uint::hex::{self};

/// Defining a constant string called SELECTOR_APPROVE_CONTRACT_CALL.
pub const SELECTOR_APPROVE_CONTRACT_CALL: &str = "approveContractCall";
/// Defining a constant string called SELECTOR_TRANSFER_OPERATORSHIP.
pub const SELECTOR_TRANSFER_OPERATORSHIP: &str = "transferOperatorship";

/// Axelar Gateway Implementation
#[near_bindgen]
impl Axelar {
    /// It emits a `ContractCallEvent` event with the current account ID, the destination chain, the
    /// destination contract address, the payload hash, and the payload
    ///
    /// Arguments:
    ///
    /// * `destination_chain`: The chain that the contract is on.
    /// * `destination_contract_address`: The address of the contract you want to call.
    /// * `payload`: The payload to be sent to the destination contract.  
    #[payable]
    pub fn call_contract(
        destination_chain: String,
        destination_contract_address: String,
        payload: String,
    ) -> ContractCallEvent {
        let payload_hash = keccak256(clean_payload(payload.clone()));

        let event = ContractCallEvent {
            address: predecessor_account_id().to_string(),
            destination_chain,
            destination_contract_address,
            payload_hash: utils::to_eth_hex_string(payload_hash.try_into().unwrap()),
            payload,
        };

        Event::emit(&event);

        event
    }

    #[payable]
    pub fn test_hash(&mut self, input: String) -> String {
        let payload = clean_payload(input.clone());

        let tokens = abi_decode(&payload, &vec![ParamType::Bytes, ParamType::Bytes]).unwrap();

        let data = tokens[0].clone().into_bytes().unwrap();

        let message = keccak256(data.clone());
        const PREFIX: &str = "\x19Ethereum Signed Message:\n32";
        let mut eth_message = PREFIX.as_bytes().to_vec();
        eth_message.extend_from_slice(message.as_ref());

        format!("0x{}", hex::encode(keccak256(eth_message)))
    }

    // Execute command function

    /// It takes a message hash and a proof, validates the proof, and then executes the commands in the
    /// message
    ///
    /// Arguments:
    ///
    /// * `message_hash`: The hash of the message that was signed by the operator.
    /// * `input`: The input to the contract. This is the data that is passed to the contract.
    ///
    /// Returns:
    ///
    /// The return value is a vector of booleans. Each boolean represents the result of the execution of
    /// a command.
    #[payable]
    pub fn execute(&mut self, input: String) -> Vec<bool> {
        let payload = clean_payload(input.clone());

        let tokens = abi_decode(&payload, &vec![ParamType::Bytes, ParamType::Bytes]).unwrap();

        let data = tokens[0].clone().into_bytes().unwrap();
        let proof = tokens[1].clone().into_bytes().unwrap();

        let message = keccak256(data.clone());
        const PREFIX: &str = "\x19Ethereum Signed Message:\n32";
        let mut eth_message = PREFIX.as_bytes().to_vec();
        eth_message.extend_from_slice(message.as_ref());

        let hash_message = format!("0x{}", hex::encode(keccak256(eth_message)));

        let mut allow_operatorship_transfer =
            self.validate_proof(hash_message, format!("0x{}", hex::encode(proof)));

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
            env::panic_str(format!("Invalid chain id: {}", chain_id).as_str());
        }

        let commands_length = command_ids.len();

        if commands_length != commands.len() || commands_length != params.len() {
            env::panic_str("Invalid commands");
        }

        let mut call_results: Vec<bool> = Vec::new();

        for i in 0..commands_length {
            let command_id: [u8; 32] = command_ids[i].clone().try_into().unwrap();

            if self.is_command_executed(format!("0x{}", hex::encode(command_id))) {
                continue;
            }

            let command = commands[i].clone();

            let success: bool;

            match command.as_str() {
                SELECTOR_APPROVE_CONTRACT_CALL => {
                    self.internal_set_command_executed(command_id, true);
                    success = self.internal_approve_contract_call(
                        params[i].clone(),
                        utils::to_eth_hex_string(command_id),
                    );
                }
                SELECTOR_TRANSFER_OPERATORSHIP => {
                    if !allow_operatorship_transfer {
                        continue;
                    }

                    allow_operatorship_transfer = false;
                    self.internal_set_command_executed(command_id, true);

                    success = self.internal_transfer_operatorship(params[i].clone());
                }
                _ => {
                    continue;
                }
            };

            if success {
                let event = ExecutedEvent {
                    command_id: utils::to_eth_hex_string(command_id),
                };

                Event::emit(&event);
            } else {
                self.internal_set_command_executed(command_id, false);
            }

            call_results.push(success);
        }

        call_results
    }

    /// Only Owner functions

    /// `approve_contract_call` is a function that is called by the `Bridge` contract on the source
    /// chain to approve a contract call
    ///
    /// Arguments:
    ///
    /// * `params`: The parameters of the contract call.
    /// * `command_id`: The ID of the command that was approved.
    ///
    /// Returns:
    ///
    /// A boolean value.
    pub fn approve_contract_call(&mut self, params: String, command_id: String) -> bool {
        Self::require_owner();
        let payload = clean_payload(params.clone());
        self.internal_approve_contract_call(payload, command_id)
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
        command_id: String,
        source_chain: String,
        source_address: String,
        contract_address: String,
        payload_hash: String,
    ) -> bool {
        let command: [u8; 32] = clean_payload(command_id).try_into().unwrap();
        let payload = clean_payload(payload_hash);

        let key = self.internal_get_is_contract_call_approved_key(
            command,
            source_chain,
            source_address,
            contract_address,
            payload.try_into().unwrap(),
        );

        self.bool_state.get(&key).unwrap_or(false)
    }

    /// `auth_module` returns the account id of the current account
    ///
    /// Returns:
    ///
    /// The account id of the current account.
    pub fn auth_module(&self) -> AccountId {
        env::current_account_id()
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
    pub fn is_command_executed(&self, command_id: String) -> bool {
        let command: [u8; 32] = clean_payload(command_id).try_into().unwrap();
        let key = self.internal_get_is_command_executed_key(command);
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
        command_id: String,
        source_chain: String,
        source_address: String,
        contract_address: String,
        payload_hash: String,
    ) -> bool {
        let command: [u8; 32] = clean_payload(command_id).try_into().unwrap();
        let payload = clean_payload(payload_hash);

        let key = self.internal_get_is_contract_call_approved_key(
            command,
            source_chain,
            source_address,
            contract_address,
            payload.try_into().unwrap(),
        );

        let valid = self.bool_state.get(&key).unwrap_or(false);

        if valid {
            self.bool_state.insert(&key, &false);
        }

        valid
    }

    // Internal functions

    /// `internal_approve_contract_call` is a function that is called by the `approve_contract_call`
    /// function in the `Bridge` contract
    ///
    /// Arguments:
    ///
    /// * `payload`: The payload of the contract call.
    /// * `command_id`: The ID of the command that was approved.
    ///
    /// Returns:
    ///
    /// A boolean value.
    pub fn internal_approve_contract_call(&mut self, payload: Vec<u8>, command_id: String) -> bool {
        Self::require_owner();

        let expected_output_types = vec![
            ParamType::String,
            ParamType::String,
            ParamType::Address,
            ParamType::FixedBytes(32),
            ParamType::FixedBytes(32),
            ParamType::Uint(256),
        ];

        let tokens = abi_decode(&payload, &expected_output_types).unwrap();

        let source_chain = tokens[0].clone().into_string().unwrap();
        let source_address = tokens[1].clone().into_string().unwrap();
        let contract_address = tokens[2].clone().into_address().unwrap();
        let payload_hash = tokens[3].clone().into_fixed_bytes().unwrap();
        let source_tx_hash = tokens[4].clone().into_fixed_bytes().unwrap();
        let source_event_index = tokens[5].clone().into_uint().unwrap().as_u64();

        let command = clean_payload(command_id.clone()).try_into().unwrap();
        let contract_address_cleaned = format!("{:#x}", contract_address);

        self.internal_set_contract_call_approved(
            command,
            source_chain.clone(),
            source_address.clone(),
            contract_address_cleaned.clone(),
            payload_hash.clone().try_into().unwrap(),
        );

        let event = ContractCallApprovedEvent {
            command_id,
            source_chain,
            source_address,
            contract_address: contract_address_cleaned,
            payload_hash: utils::to_eth_hex_string(payload_hash.try_into().unwrap()),
            source_tx_hash: utils::to_eth_hex_string(source_tx_hash.try_into().unwrap()),
            source_event_index,
        };

        Event::emit(&event);

        true
    }

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
    fn internal_get_is_command_executed_key(&self, command_id: [u8; 32]) -> [u8; 32] {
        let encoded = abi_encode(vec![
            Token::Bytes(self.prefix_command_executed.clone().to_vec()),
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
    ) -> [u8; 32] {
        let encoded = abi_encode(vec![
            Token::Bytes(self.prefix_contract_call_approved.clone().to_vec()),
            Token::FixedBytes(command_id.to_vec()),
            Token::String(source_chain),
            Token::String(source_address),
            Token::String(contract_address.to_lowercase()),
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

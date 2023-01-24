/*
 * Axelar Auth contract
 *
 */

mod auth_weighted;
mod events;
mod gateway;
mod utils;

use near_contract_tools::{owner::*, Owner};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::predecessor_account_id;
use near_sdk::near_bindgen;
use utils::clean_payload;
use utils::keccak256;

/// `Axelar` is a struct that contains a `current_epoch` field, a `hash_for_epoch` field, an
/// `epoch_for_hash` field, a `prefix_command_executed` field, a `prefix_contract_call_approved` field,
/// and a `bool_state` field.
///
/// The `current_epoch` field is a `u64` (unsigned 64-bit integer).
///
/// The `hash_for_epoch` field is a `LookupMap<u64, [u
///
/// Properties:
///
/// * `current_epoch`: The current epoch number.
/// * `hash_for_epoch`: This is a map that stores the hash of the block that was used to create the
/// epoch.
/// * `epoch_for_hash`: This is a mapping from a hash to an epoch.
/// * `prefix_command_executed`: This is the prefix for the key that stores the boolean value of whether
/// a command has been executed.
/// * `prefix_contract_call_approved`: This is the prefix for the key that stores the boolean value of
/// whether a contract call has been approved.
/// * `bool_state`: This is a map that stores the state of the contract.
#[near_bindgen]
#[derive(Owner, BorshDeserialize, BorshSerialize)]
pub struct Axelar {
    // Auth Weighted
    current_epoch: u64,
    hash_for_epoch: LookupMap<u64, [u8; 32]>,
    epoch_for_hash: LookupMap<[u8; 32], u64>,
    // Gateway
    prefix_command_executed: [u8; 32],
    prefix_contract_call_approved: [u8; 32],
    bool_state: LookupMap<[u8; 32], bool>,
}

/// This is a default implementation of the `Axelar` struct.
impl Default for Axelar {
    fn default() -> Self {
        Self {
            // Auth Weighted
            current_epoch: 0,
            hash_for_epoch: LookupMap::new(b"hash_for_epoch".to_vec()),
            epoch_for_hash: LookupMap::new(b"epoch_for_hash".to_vec()),
            // Gateway
            prefix_command_executed: keccak256(b"command-executed"),
            prefix_contract_call_approved: keccak256(b"contract-call-approved"),
            bool_state: LookupMap::new(b"bool_state".to_vec()),
        }
    }
}

#[near_bindgen]
impl Axelar {
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
    pub fn new(recent_operators: Vec<String>) -> Self {
        let mut contract = Self {
            // Auth Weighted
            current_epoch: 0,
            hash_for_epoch: LookupMap::new(b"hash_for_epoch".to_vec()),
            epoch_for_hash: LookupMap::new(b"epoch_for_hash".to_vec()),
            // Gateway
            prefix_command_executed: keccak256(b"command-executed"),
            prefix_contract_call_approved: keccak256(b"contract-call-approved"),
            bool_state: LookupMap::new(b"bool_state".to_vec()),
        };

        Owner::init(&mut contract, &predecessor_account_id());

        for operator in recent_operators {
            contract.internal_transfer_operatorship(clean_payload(operator));
        }

        contract
    }
}

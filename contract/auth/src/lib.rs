/*
 * Axelar Auth contract
 *
 */

use near_contract_tools::{owner::Owner, Owner};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::predecessor_account_id;
use near_sdk::near_bindgen;

pub const OLD_KEY_RETENTION: u8 = 16;

#[derive(Owner)]
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AxelarAuthWeighted {
    current_epoch: u64,
    hash_for_epoch: LookupMap<u64, u32>,
    epoch_for_hash: LookupMap<u32, u64>,
}

#[near_bindgen]
impl AxelarAuthWeighted {
    #[init]
    pub fn new(recent_operators: Vec<Vec<u8>>) -> Self {
        let mut contract = Self {
            current_epoch: 0,
            hash_for_epoch: LookupMap::new(b"hash_for_epoch".to_vec()),
            epoch_for_hash: LookupMap::new(b"epoch_for_hash".to_vec()),
        };

        Owner::init(&mut contract, &predecessor_account_id());

        for operator in recent_operators {
            contract.transfer_operatorship(operator);
        }

        contract
    }

    pub fn validate_proof(&self, message_hash: Vec<u8>, proof: Vec<u8>) -> bool {
        // TOOD: implement
        true
    }

    // Only owner
    pub fn transfer_operatorship(&mut self, params: Vec<u8>) {
        Self::require_owner();
        self.internal_transfer_operatorship(params);
    }

    /// Internal

    fn internal_transfer_operatorship(&mut self, params: Vec<u8>) {
        // TOOD: implement
    }

    fn internal_validate_signatures(
        &self,
        message_hash: [u8; 32],
        operators: Vec<[u8; 32]>,
        weights: Vec<u32>,
        threshold: u32,
        signatures: Vec<[u8; 64]>,
    ) {
        let operator_length = operators.len();
        let mut operator_index = 0;
        let mut weight = 0;

        for i in 0..signatures.len() {
            let signer = utils::recover(&message_hash, &signatures[i]);

            while operator_index < operator_length
                && utils::to_verifying_key(operators[operator_index]) != signer
            {
                operator_index += 1;
            }

            if operator_index >= operator_length {
                panic!("Malformed signers");
            }

            weight += weights[operator_index];

            if weight >= threshold {
                return;
            }

            operator_index += 1;
        }

        assert!(weight < threshold, "Total weight is less than threshold");
    }

    fn internal_is_sorted_asc_and_contains_no_duplicate(accounts: &[String]) -> bool {
        for i in 0..(accounts.len() - 1) {
            if accounts[i] >= accounts[i + 1] {
                return false;
            }
        }

        return accounts[0] != "";
    }
}

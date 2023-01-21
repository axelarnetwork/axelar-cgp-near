use near_sdk::ext_contract;

/// Interface of this contract, for callbacks
/// This is defining the interface of the contract that is being called.
#[ext_contract(this_contract)]
trait Callbacks {
    fn validate_proof_callback(&mut self, data: Vec<u8>);
}

/// AxelarAuth contract
/// Defining the interface of the contract that is being called.
#[ext_contract(ext_auth_contract)]
trait ExtProfilesContract {
    fn validate_proof(&self, message_hash: Box<[u8; 32]>, proof: Box<[u8]>) -> bool;
    fn transfer_operatorship(&mut self, params: Vec<u8>);
}

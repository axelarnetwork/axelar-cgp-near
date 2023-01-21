use near_contract_tools::event;

/// `ContractCallEvent` is emitted when a contract call is made to the gateway.
///
/// Properties:
///
/// * `address`: The address of the contract that emitted the event.
/// * `destination_chain`: The chain that the contract call is being made to.
/// * `destination_contract_address`: The address of the contract that will receive the call.
/// * `payload_hash`: The hash of the payload.
/// * `payload`: The payload of the contract call.
#[event(standard = "axelar_near", version = "1.0.0")]
pub struct ContractCallEvent {
    pub address: String,
    pub destination_chain: String,
    pub destination_contract_address: String,
    pub payload_hash: Vec<u8>,
    pub payload: Vec<u8>,
}

/// `ExecutedEvent` is emitted when a contract call is executed.
///
/// Properties:
///
/// * `command_id`: The command ID that was executed.
#[event(standard = "axelar_near", version = "1.0.0")]
pub struct ExecutedEvent {
    pub command_id: Vec<u8>,
}

/// `ContractCallApprovedEvent` is emitted when a contract call is approved.
///
/// Properties:
///
/// * `command_id`: The command ID of the command that was approved.
/// * `source_chain`: The chain that the contract call originated from.
/// * `source_address`: The address of the contract that is calling the target contract.
/// * `contract_address`: The address of the contract that was called.
/// * `payload_hash`: The hash of the payload that was sent to the contract.
/// * `source_tx_hash`: The hash of the transaction that triggered the event.
/// * `source_event_index`: The index of the event in the source chain.
#[event(standard = "axelar_near", version = "1.0.0")]
pub struct ContractCallApprovedEvent {
    pub command_id: Vec<u8>,
    pub source_chain: String,
    pub source_address: String,
    pub contract_address: String,
    pub payload_hash: Vec<u8>,
    pub source_tx_hash: Vec<u8>,
    pub source_event_index: u64,
}

/// `OperatorshipTransferredEvent` is emitted when the operatorship is transferred.
///
/// Properties:
///
/// * `new_operators_data`: The new operators data.
#[event(standard = "axelar_near", version = "1.0.0")]
pub struct OperatorshipTransferredEvent {
    pub new_operators_data: Vec<u8>,
}

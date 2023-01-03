use near_contract_tools::event;

#[event(standard = "axelar_near", version = "1.0.0")]
pub struct OperatorshipTransferredEvent {
    pub new_operators: String,
    pub new_weights: String,
    pub new_threshold: String,
}

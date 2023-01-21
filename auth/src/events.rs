use near_contract_tools::event;

/// `OperatorshipTransferredEvent` is emitted when the operatorship is transferred.
///
/// Properties:
///
/// * `new_operators`: The new list of operators.
/// * `new_weights`: A comma-separated list of weights for the new operators.
/// * `new_threshold`: The new threshold for the operatorship.
#[event(standard = "axelar_near", version = "1.0.0")]
pub struct OperatorshipTransferredEvent {
    pub new_operators: String,
    pub new_weights: String,
    pub new_threshold: String,
}

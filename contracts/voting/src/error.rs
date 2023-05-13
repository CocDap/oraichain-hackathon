use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("funds sent should be equal by voting weight")]
    InsufficientFundsSent {},

    #[error("Overflow")]
    OverflowError{},

    #[error("NoProposalId")]
    NoProposalId{},


}
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

    #[error("Proposal not found")]
    ProposalNotFound{},

    #[error("Proposal not in progress")]
    ProposalNotInProgress{},

    #[error("Proposal not in reviewed")]
    ProposalNotInReview{},

    #[error("Wrong coin sent")]
    WrongCoinSent {},

    #[error("Wrong fund coin (expected: {expected}, got: {got})")]
    WrongFundCoin { expected: String, got: String },

    #[error("CLR algorithm requires a budget constrain")]
    CLRConstrainRequired {},
    

}

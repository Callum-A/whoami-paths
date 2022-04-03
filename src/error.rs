use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficient funds sent to mint a path")]
    InsufficientFunds {},

    #[error("Token received is not the token configured for this contract")]
    UnrecognisedToken {},

    #[error("The root token has not been received yet")]
    NoRootToken {},

    #[error("The root token has already been set")]
    ExistingRootToken {},

    #[error("No CW20 payment needed to mint a path")]
    NoPaymentNeeded {},
}

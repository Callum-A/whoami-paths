use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

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

    #[error("No payment is needed to mint a path")]
    NoPaymentNeeded {},

    #[error("You have specified payment details but amount is set to 0")]
    InvalidPaymentAmount {},

    #[error("The token address provided is not a valid CW20 token")]
    InvalidCw20 {},
}

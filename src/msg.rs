use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenDetails {
    pub token_address: String, // Token contract address
    pub token_cost: Uint128,   // Amount of the token it costs to mint a path
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,          // Only the admin can withdraw the name if needed
    pub whoami_address: String, // Address of base whoami contract
    pub token_details: Option<TokenDetails>, // Users may have to pay in a token
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    MintPath { path: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    MintPath {
        path: String,
    }, // Equivalent to receive but with no cost
    UpdateAdmin {
        new_admin: String,
    },
    UpdateTokenDetails {
        new_token_details: Option<TokenDetails>,
    }, // Set to null to clear
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    TokenDetails {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenDetailsResponse {
    pub token_details: Option<TokenDetails>,
}

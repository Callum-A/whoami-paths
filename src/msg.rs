use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaymentDetails {
    Cw20 {
        token_address: String,
        amount: Uint128,
    },
    Native {
        denom: String,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,          // Only the admin can withdraw the name if needed
    pub whoami_address: String, // Address of base whoami contract
    pub payment_details: Option<PaymentDetails>, // Users may have to pay in a cw20 or a native token
    pub path_root_claim_blocks: Option<u64>, // Allow users with (de)NS root token to claim the path
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
    MintPath { path: String }, // Equivalent to receive but with no cost
    UpdateAdmin { new_admin: String },
    WithdrawPayments {},
    WithdrawRootToken {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    PaymentDetails {},
    PaymentDetailsBalance {},
    ClaimInfo { path: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PaymentDetailsResponse {
    pub payment_details: Option<PaymentDetails>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PaymentDetailsBalanceResponse {
    pub payment_details: Option<PaymentDetails>,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimInfoResponse {
    pub is_in_claim_window: bool,
    pub path_as_base_owner: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {
    pub path_root_claim_blocks: Option<u64>,
}

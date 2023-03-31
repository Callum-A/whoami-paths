use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::PaymentDetails;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub whoami_address: String,
    pub admin: Addr,
    pub token_id: Option<String>, // If we have received a name to mint paths off this will be the token_id
    pub path_root_claim_blocks: Option<u64>,
    pub initial_height: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PAYMENT_DETAILS: Item<PaymentDetails> = Item::new("payment_details");

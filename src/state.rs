use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::TokenDetails;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub whoami_address: String,
    pub admin: Addr,
    pub token_id: Option<String>, // If we have received a name to mint paths off this will be the token_id
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOKEN_DETAILS: Item<TokenDetails> = Item::new("token_details");

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg};
use crate::state::{Config, CONFIG, TOKEN_DETAILS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whoami-paths";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let whoami_address = deps.api.addr_validate(&msg.whoami_address)?;
    let admin = deps.api.addr_validate(&msg.admin)?;

    let config = Config {
        whoami_address: whoami_address.to_string(),
        admin: admin.clone(),
        token_id: None,
    };

    CONFIG.save(deps.storage, &config)?;

    if msg.token_details.is_some() {
        let token_details = msg.token_details.unwrap();
        deps.api.addr_validate(&token_details.token_address)?;
        TOKEN_DETAILS.save(deps.storage, &token_details)?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_receive) => execute_receive_cw20(deps, env, info, cw20_receive),
        ExecuteMsg::ReceiveNft(cw721_receive) => {
            execute_receive_cw721(deps, env, info, cw721_receive)
        }
        ExecuteMsg::MintPath { path } => execute_mint_path(deps, env, info, path),
    }
}

pub fn mint_path_msg(
    whoami_address: String,
    contract: String, // our contract address
    owner: String,    // the person who made the MintMsg call
    token_id: String,
    path: String,
) -> StdResult<Vec<WasmMsg>> {
    let whoami_msg = whoami::ExecuteMsg::MintPath(whoami::msg::MintMsg {
        token_id: path.clone(),
        owner: contract,
        token_uri: None,
        extension: whoami::Extension {
            image: None,
            image_data: None,
            email: None,
            external_url: None,
            public_name: None,
            public_bio: None,
            twitter_id: None,
            discord_id: None,
            telegram_id: None,
            keybase_id: None,
            validator_operator_address: None,
            contract_address: None,
            parent_token_id: Some(token_id.clone()),
            pgp_public_key: None,
        },
    });
    let wasm_msg1 = WasmMsg::Execute {
        contract_addr: whoami_address.clone(),
        msg: to_binary(&whoami_msg)?,
        funds: vec![],
    };

    let transfer_msg = whoami::ExecuteMsg::TransferNft {
        recipient: owner,
        token_id: format!("{}::{}", token_id, path),
    };
    let wasm_msg2 = WasmMsg::Execute {
        contract_addr: whoami_address,
        msg: to_binary(&transfer_msg)?,
        funds: vec![],
    };

    Ok(vec![wasm_msg1, wasm_msg2])
}

pub fn execute_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_receive: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let token_details = TOKEN_DETAILS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    if token_details.is_none() {
        // We do not need to pay a CW20 to mint
        return Err(ContractError::Unauthorized {});
    }
    let token_details = token_details.unwrap();

    if info.sender != token_details.token_address {
        // Unrecognised token
        return Err(ContractError::Unauthorized {});
    }

    if config.token_id.is_none() {
        // Token has not been received so cannot mint paths
        return Err(ContractError::Unauthorized {});
    }

    let token_id = config.token_id.unwrap();
    if cw20_receive.amount < token_details.token_cost {
        // Not enough sent
        return Err(ContractError::Unauthorized {});
    }

    let msg: ReceiveMsg = from_binary(&cw20_receive.msg)?;

    let wasm_msg = match msg {
        ReceiveMsg::MintPath { path } => mint_path_msg(
            config.whoami_address,
            env.contract.address.to_string(),
            cw20_receive.sender,
            token_id,
            path,
        )?,
    };

    Ok(Response::new().add_messages(wasm_msg))
}

pub fn execute_receive_cw721(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw721_receive: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.whoami_address != info.sender {
        // Coming from a different contract
        return Err(ContractError::Unauthorized {});
    }

    if config.token_id.is_some() {
        // We already have a token
        return Err(ContractError::Unauthorized {});
    }

    if cw721_receive.sender != config.admin {
        // Only admin can transfer in the name
        return Err(ContractError::Unauthorized {});
    }

    config.token_id = Some(cw721_receive.token_id);

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

pub fn execute_mint_path(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    path: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let token_details = TOKEN_DETAILS.may_load(deps.storage)?;

    if config.token_id.is_none() {
        // No token to mint off of
        return Err(ContractError::Unauthorized {});
    }
    let token_id = config.token_id.unwrap();

    if token_details.is_some() {
        // Have to pay a token
        return Err(ContractError::Unauthorized {});
    }

    let wasm_msg = mint_path_msg(
        config.whoami_address,
        env.contract.address.to_string(),
        info.sender.to_string(),
        token_id,
        path,
    )?;

    Ok(Response::new().add_messages(wasm_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::TokenDetails {} => to_binary(&TOKEN_DETAILS.load(deps.storage)?),
    }
}

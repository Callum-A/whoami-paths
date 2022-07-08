#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, TokenInfoResponse};
use cw721::Cw721ReceiveMsg;
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, PaymentDetails, PaymentDetailsResponse, QueryMsg, ReceiveMsg,
};
use crate::state::{Config, CONFIG, PAYMENT_DETAILS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whoami-paths";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn assert_cw20(deps: Deps, cw20_addr: &Addr) -> Result<(), ContractError> {
    let _resp: TokenInfoResponse = deps
        .querier
        .query_wasm_smart(cw20_addr, &cw20_base::msg::QueryMsg::TokenInfo {})
        .map_err(|_err| ContractError::InvalidCw20 {})?;
    Ok(())
}

pub fn mint_path_msg(
    whoami_address: String,
    contract: String, // our contract address
    owner: String,    // the person who made the MintMsg call
    token_id: String,
    path: String,
) -> StdResult<Vec<WasmMsg>> {
    let whoami_msg = whoami::msg::ExecuteMsg::MintPath(whoami::msg::MintMsg {
        token_id: path.clone(),
        owner: contract,
        token_uri: None,
        extension: whoami::msg::Extension {
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

    let transfer_msg = whoami::msg::ExecuteMsg::TransferNft {
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

fn mint(
    env: Env,
    whoami_address: String,
    token_id: String,
    path: String,
    address_minting_the_path: String,
    amount_paid: Uint128,
    amount_required: Uint128,
) -> Result<Response, ContractError> {
    if amount_paid != amount_required {
        return Err(ContractError::Unauthorized {});
    }

    let wasm_msg = mint_path_msg(
        whoami_address,
        env.contract.address.to_string(),
        address_minting_the_path,
        token_id,
        path,
    )?;
    Ok(Response::new()
        .add_attribute("action", "mint_path")
        .add_messages(wasm_msg))
}

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

    if let Some(payment_details) = msg.payment_details {
        match payment_details.clone() {
            PaymentDetails::Cw20 {
                token_address,
                amount,
            } => {
                let validated_addr = deps.api.addr_validate(&token_address)?;
                assert_cw20(deps.as_ref(), &validated_addr)?;
                if amount.is_zero() {
                    return Err(ContractError::InvalidPaymentAmount {});
                }
            }
            PaymentDetails::Native { denom: _, amount } => {
                if amount.is_zero() {
                    return Err(ContractError::InvalidPaymentAmount {});
                }
            }
        }
        PAYMENT_DETAILS.save(deps.storage, &payment_details)?;
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
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, env, info, new_admin),
        ExecuteMsg::WithdrawRootToken {} => execute_withdraw_root_token(deps, env, info),
    }
}

pub fn execute_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_receive: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let payment_details = PAYMENT_DETAILS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if payment_details.is_none() {
        // We do not need to pay a CW20 to mint
        return Err(ContractError::NoPaymentNeeded {});
    }

    if config.token_id.is_none() {
        // We have no token to mint off of
        return Err(ContractError::NoRootToken {});
    }
    let token_id = config.token_id.unwrap();

    let payment_details = payment_details.unwrap();

    match payment_details {
        PaymentDetails::Cw20 {
            amount,
            token_address,
        } => {
            if info.sender != token_address {
                // Unrecognised token
                return Err(ContractError::UnrecognisedToken {});
            }

            let recv_msg: ReceiveMsg = from_binary(&cw20_receive.msg)?;

            let path = match recv_msg {
                ReceiveMsg::MintPath { path } => path,
            };

            mint(
                env,
                config.whoami_address,
                token_id,
                path,
                cw20_receive.sender,
                cw20_receive.amount,
                amount,
            )
        }
        // TODO: Improve error
        PaymentDetails::Native { .. } => Err(ContractError::Unauthorized {}),
    }
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
        return Err(ContractError::ExistingRootToken {});
    }

    if cw721_receive.sender != config.admin {
        // Only admin can transfer in the name
        return Err(ContractError::Unauthorized {});
    }

    config.token_id = Some(cw721_receive.token_id);

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "receive_cw721"))
}

pub fn execute_mint_path(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    path: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let payment_details = PAYMENT_DETAILS.may_load(deps.storage)?;
    if config.token_id.is_none() {
        // No token to mint off of
        return Err(ContractError::NoRootToken {});
    }
    let token_id = config.token_id.unwrap();

    if let Some(payment_details) = payment_details {
        match payment_details {
            PaymentDetails::Native { denom, amount } => {
                let paid_amount = must_pay(&info, &denom)?;
                mint(
                    env,
                    config.whoami_address,
                    token_id,
                    path,
                    info.sender.to_string(),
                    paid_amount,
                    amount,
                )
            }
            // TODO: Improve error
            PaymentDetails::Cw20 { .. } => Err(ContractError::Unauthorized {}),
        }
    } else {
        mint(
            env,
            config.whoami_address,
            token_id,
            path,
            info.sender.to_string(),
            Uint128::zero(),
            Uint128::zero(),
        )
    }
}

pub fn execute_update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        // Only existing admin can set
        return Err(ContractError::Unauthorized {});
    }
    let old_admin = config.admin.clone();
    let validated_new_admin = deps.api.addr_validate(&new_admin)?;

    config.admin = validated_new_admin.clone();

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("old_admin", old_admin.to_string())
        .add_attribute("new_admin", validated_new_admin.to_string()))
}

pub fn execute_withdraw_root_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let admin = config.admin.clone();
    let whoami_address = config.whoami_address.clone();

    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    if config.token_id.is_none() {
        return Err(ContractError::NoRootToken {});
    }
    let token_id = config.token_id.unwrap();

    let transfer_msg = whoami::msg::ExecuteMsg::TransferNft {
        recipient: admin.to_string(),
        token_id,
    };
    let wasm_msg = WasmMsg::Execute {
        contract_addr: whoami_address,
        msg: to_binary(&transfer_msg)?,
        funds: vec![],
    };

    config.token_id = None;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "withdraw_root_token")
        .add_message(wasm_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::PaymentDetails {} => to_binary(&PaymentDetailsResponse {
            payment_details: PAYMENT_DETAILS.may_load(deps.storage)?,
        }),
    }
}

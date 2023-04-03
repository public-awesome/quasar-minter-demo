use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg};
use crate::state::{increment_token_index, Config, COLLECTION_ADDRESS, CONFIG, STATUS};

use base_factory::msg::{BaseMinterCreateMsg, ParamsResponse};

use base_factory::state::Extension;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, StdResult,
    Timestamp, WasmMsg,
};

use cw2::set_contract_version;
use cw721_base::MintMsg;
use cw_utils::{must_pay, nonpayable, parse_reply_instantiate_data};

use sg1::checked_fair_burn;
use sg2::query::Sg2QueryMsg;
use sg4::{QueryMsg, Status, StatusResponse, SudoMsg};
use sg721::{ExecuteMsg as Sg721ExecuteMsg, InstantiateMsg as Sg721InstantiateMsg};
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::math::U64Ext;
use sg_std::{Response, SubMsg, NATIVE_DENOM};
use url::Url;

const CONTRACT_NAME: &str = "crates.io:sg-quasar-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_SG721_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseMinterCreateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        collection_code_id: msg.collection_params.code_id,
    };

    // Use default start trading time if not provided
    let collection_info = msg.collection_params.info.clone();

    CONFIG.save(deps.storage, &config)?;

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_params.code_id,
        msg: to_binary(&Sg721InstantiateMsg {
            name: msg.collection_params.name.clone(),
            symbol: msg.collection_params.symbol,
            minter: env.contract.address.to_string(),
            collection_info,
        })?,
        funds: info.funds,
        admin: None,
        label: format!(
            "SG721-{}-{}",
            msg.collection_params.code_id,
            msg.collection_params.name.trim()
        ),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INSTANTIATE_SG721_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {
            token_uri,
        } => execute_mint_sender(deps, info, token_uri),
    }
}

pub fn execute_mint_sender(
    deps: DepsMut,
    info: MessageInfo,
    token_uri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let collection_address = COLLECTION_ADDRESS.load(deps.storage)?;

    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(collection_address.clone(), &Sg721QueryMsg::CollectionInfo {})?;

    let parsed_token_uri = Url::parse(&token_uri)?;
    if parsed_token_uri.scheme() != "ipfs" {
        return Err(ContractError::InvalidTokenURI {});
    }

    let mut res = Response::new();

    // Create mint msgs
    let mint_msg = Sg721ExecuteMsg::<Extension, Empty>::Mint(MintMsg::<Extension> {
        token_id: increment_token_index(deps.storage)?.to_string(),
        owner: info.sender.to_string(),
        token_uri: Some(token_uri.clone()),
        extension: None,
    });
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    res = res.add_message(msg);

    Ok(res
        .add_attribute("action", "mint")
        .add_attribute("sender", info.sender)
        .add_attribute("token_uri", token_uri))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateStatus {
            is_verified,
            is_blocked,
            is_explicit,
        } => update_status(deps, is_verified, is_blocked, is_explicit)
            .map_err(|_| ContractError::UpdateStatus {}),
    }
}

/// Only governance can update contract params
pub fn update_status(
    deps: DepsMut,
    is_verified: bool,
    is_blocked: bool,
    is_explicit: bool,
) -> StdResult<Response> {
    let mut status = STATUS.load(deps.storage)?;
    status.is_verified = is_verified;
    status.is_blocked = is_blocked;
    status.is_explicit = is_explicit;

    Ok(Response::new().add_attribute("action", "sudo_update_status"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Status {} => to_binary(&query_status(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        config,
    })
}

pub fn query_status(deps: Deps) -> StdResult<StatusResponse> {
    let status = STATUS.load(deps.storage)?;

    Ok(StatusResponse {
        status,
    })
}

// Reply callback triggered from sg721 contract instantiation in instantiate()
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_SG721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_address = res.contract_address;
            COLLECTION_ADDRESS.save(deps.storage, &Addr::unchecked(collection_address.clone()))?;
            Ok(Response::default()
                .add_attribute("action", "instantiate_sg721_reply")
                .add_attribute("sg721_address", collection_address))
        },
        Err(_) => Err(ContractError::InstantiateSg721Error {}),
    }
}

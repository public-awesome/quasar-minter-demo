use cosmwasm_std::StdResult;
use cw_paginate::paginate_map;
use cw_storage_plus::Bound;
use token_factory::Deps;

use crate::{
    msg::{AccountResponse, ActiveChannelResponse, ConfigResponse, DenomTraceResponse},
    state::{
        ACCOUNTS, ACCOUNT_CODE_ID, ACTIVE_CHANNELS, DEFAULT_TIMEOUT_SECS, DENOM_TRACES, TRANSFER,
    },
};

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        account_code_id: ACCOUNT_CODE_ID.load(deps.storage)?,
        transfer: TRANSFER.load(deps.storage)?.into(),
        default_timeout_secs: DEFAULT_TIMEOUT_SECS.load(deps.storage)?,
    })
}

pub fn denom_trace(deps: Deps, denom: String) -> StdResult<DenomTraceResponse> {
    Ok(DenomTraceResponse {
        trace: DENOM_TRACES.load(deps.storage, &denom)?,
        denom,
    })
}

pub fn denom_traces(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<DenomTraceResponse>> {
    let start = start_after.as_ref().map(|denom| Bound::exclusive(denom.as_str()));
    paginate_map(&DENOM_TRACES, deps.storage, start, limit, |denom, trace| {
        Ok(DenomTraceResponse {
            denom,
            trace,
        })
    })
}

pub fn account(
    deps: Deps,
    connection_id: String,
    controller: String,
) -> StdResult<AccountResponse> {
    Ok(AccountResponse {
        address: ACCOUNTS.load(deps.storage, (&connection_id, &controller))?.into(),
        connection_id,
        controller,
    })
}

pub fn accounts(
    deps: Deps,
    start_after: Option<(String, String)>,
    limit: Option<u32>,
) -> StdResult<Vec<AccountResponse>> {
    let start = start_after.as_ref().map(|(cid, con)| Bound::exclusive((cid.as_str(), con.as_str())));
    paginate_map(&ACCOUNTS, deps.storage, start, limit, |(connection_id, controller), address| {
        Ok(AccountResponse {
            connection_id,
            controller,
            address: address.into(),
        })
    })
}

pub fn active_channel(deps: Deps, connection_id: String) -> StdResult<ActiveChannelResponse> {
    Ok(ActiveChannelResponse {
        channel_id: ACTIVE_CHANNELS.load(deps.storage, &connection_id)?,
        connection_id,
    })
}

pub fn active_channels(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<ActiveChannelResponse>> {
    let start = start_after.as_ref().map(|cid| Bound::exclusive(cid.as_str()));
    paginate_map(&ACTIVE_CHANNELS, deps.storage, start, limit, |connection_id, channel_id| {
        Ok(ActiveChannelResponse {
            connection_id,
            channel_id,
        })
    })
}

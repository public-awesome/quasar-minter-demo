use crate::state::Config;
use base_factory::{msg::BaseMinterCreateMsg, state::BaseMinterParams};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub create_msg: BaseMinterCreateMsg,
    pub params: BaseMinterParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        token_uri: String,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

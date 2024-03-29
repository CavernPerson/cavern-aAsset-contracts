use cosmwasm_std::{Addr, Binary, DepsMut, QueryRequest, StdError, StdResult, WasmQuery};
use cosmwasm_storage::to_length_prefixed;

use crate::state::read_hub_contract;
use basset::hub::Config;

pub fn query_reward_contract(deps: &DepsMut) -> StdResult<Addr> {
    let hub_address = deps
        .api
        .addr_humanize(&read_hub_contract(deps.storage).unwrap())
        .unwrap();

    let config: Config = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: hub_address.to_string(),
        key: Binary::from(to_length_prefixed(b"config")),
    }))?;

    let address = config
        .reward_contract
        .ok_or_else(|| StdError::generic_err("the reward contract must have been registered"))?;

    Ok(address)
}

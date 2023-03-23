use crate::state::{read_config, read_state, store_state, Config, State};

use cosmwasm_std::{
    attr, CosmosMsg, Decimal256, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::swap::create_swap_msgs;

/// Swap all native tokens to reward_denom
/// Only hub_contract is allowed to execute
#[allow(clippy::if_same_then_else)]
#[allow(clippy::needless_collect)]
pub fn execute_swap(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    
    let config = read_config(deps.storage)?;

    if info.sender.as_str() != config.hub_contract {
        return Err(StdError::generic_err("unauthorized"));
    }

    let contr_addr = env.contract.address.clone();
    let balance = deps.querier.query_all_balances(contr_addr)?;
    let mut messages: Vec<CosmosMsg> = Vec::new();

    let reward_denom = config.reward_denom;

    let denoms: Vec<String> = balance.iter().map(|item| item.denom.clone()).collect();

    for coin in balance {
        if coin.denom == reward_denom.clone() || !denoms.contains(&coin.denom) {
            continue;
        }
        
        messages.append(&mut create_swap_msgs(
            deps.as_ref(),
            env.clone(),
            coin,
            reward_denom.to_string(),
        )?);
        
    }

    let res = Response::new()
        .add_messages(messages)
        .add_attributes(vec![attr("action", "swap")]);

    Ok(res)
}

/// Increase global_index according to claimed rewards amount
/// Only hub_contract is allowed to execute
pub fn execute_update_global_index(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    // Permission check
    if config.hub_contract != info.sender.as_str() {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // Zero staking balance check
    if state.total_balance.is_zero() {
        return Err(StdError::generic_err("No asset is bonded by Hub"));
    }

    let reward_denom = read_config(deps.storage)?.reward_denom;

    // Load the reward contract balance
    let balance = deps
        .querier
        .query_balance(env.contract.address, reward_denom.as_str())?;

    let previous_balance = state.prev_reward_balance;

    // claimed_rewards = current_balance - prev_balance;
    let claimed_rewards = balance.amount.checked_sub(previous_balance)?;

    state.prev_reward_balance = balance.amount;

    // global_index += claimed_rewards / total_balance;
    state.global_index += Decimal256::from_ratio(claimed_rewards, state.total_balance);
    store_state(deps.storage, &state)?;

    let attributes = vec![
        attr("action", "update_global_index"),
        attr("claimed_rewards", claimed_rewards),
    ];
    let res = Response::new().add_attributes(attributes);

    Ok(res)
}

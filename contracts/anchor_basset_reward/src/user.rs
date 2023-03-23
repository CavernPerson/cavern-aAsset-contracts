use std::convert::TryInto;
use crate::querier::query_token_contract;
use crate::state::{
    read_config, read_holder, read_holders, read_state, store_holder, store_state, Config, Holder,
    State,
};
use basset::reward::{AccruedRewardsResponse, HolderResponse, HoldersResponse};

use cosmwasm_std::{
    attr, BankMsg, Coin, CosmosMsg, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, Uint256
};
use std::str::FromStr;

pub fn execute_claim_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> StdResult<Response> {
    let holder_addr = info.sender;
    let holder_addr_raw = deps.api.addr_validate(holder_addr.as_str())?;
    let recipient = match recipient {
        Some(value) => deps.api.addr_validate(value.as_str()).unwrap(),
        None => holder_addr.clone(),
    };

    let mut holder: Holder = read_holder(deps.storage, &holder_addr_raw)?;
    let mut state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;

    let reward_with_decimals =
        calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    let all_reward_with_decimals = reward_with_decimals + holder.pending_rewards;
    let decimals = get_decimals(all_reward_with_decimals)?;

    let rewards = all_reward_with_decimals * Uint256::one();

    if rewards.is_zero() {
        return Err(StdError::generic_err("No rewards have accrued yet"));
    }

    let new_balance = (state.prev_reward_balance.checked_sub(rewards.try_into()?))?;
    state.prev_reward_balance = new_balance;
    store_state(deps.storage, &state)?;

    holder.pending_rewards = decimals;
    holder.index = state.global_index;
    store_holder(deps.storage, &holder_addr_raw, &holder)?;

    let bank_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![
            Coin {
                denom: config.reward_denom,
                amount: rewards.try_into()?,
            }],
    });

    let res = Response::new()
        .add_attributes(vec![
            attr("action", "claim_reward"),
            attr("holder_address", holder_addr),
            attr("rewards", rewards),
        ])
        .add_message(bank_msg);

    Ok(res)
}

pub fn execute_increase_balance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let owner_human = deps.api.addr_humanize(&config.hub_contract)?;
    let address_raw = deps.api.addr_validate(&address)?;
    let sender = info.sender;

    let token_address = query_token_contract(deps.as_ref(), owner_human)?;

    // Check sender is token contract
    if sender != token_address {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut state: State = read_state(deps.storage)?;
    let mut holder: Holder = read_holder(deps.storage, &address_raw)?;

    // get decimals
    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = rewards + holder.pending_rewards;
    holder.balance += amount;
    state.total_balance += amount;

    store_holder(deps.storage, &address_raw, &holder)?;
    store_state(deps.storage, &state)?;

    let attributes = vec![
        attr("action", "increase_balance"),
        attr("holder_address", address),
        attr("amount", amount),
    ];

    let res = Response::new().add_attributes(attributes);
    Ok(res)
}

pub fn execute_decrease_balance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let hub_contract = deps.api.addr_humanize(&config.hub_contract)?;
    let address_raw = deps.api.addr_validate(&address)?;

    // Check sender is token contract
    if query_token_contract(deps.as_ref(), hub_contract)?
        != deps.api.addr_validate(info.sender.as_str())?
    {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut state: State = read_state(deps.storage)?;
    let mut holder: Holder = read_holder(deps.storage, &address_raw)?;
    if holder.balance < amount {
        return Err(StdError::generic_err(format!(
            "Decrease amount cannot exceed user balance: {}",
            holder.balance
        )));
    }

    let rewards = calculate_decimal_rewards(state.global_index, holder.index, holder.balance)?;

    holder.index = state.global_index;
    holder.pending_rewards = rewards + holder.pending_rewards;
    holder.balance = (holder.balance.checked_sub(amount))?;
    state.total_balance = (state.total_balance.checked_sub(amount))?;

    store_holder(deps.storage, &address_raw, &holder)?;
    store_state(deps.storage, &state)?;

    let attributes = vec![
        attr("action", "decrease_balance"),
        attr("holder_address", address),
        attr("amount", amount),
    ];

    let res = Response::new().add_attributes(attributes);

    Ok(res)
}

pub fn query_accrued_rewards(deps: Deps, address: String) -> StdResult<AccruedRewardsResponse> {
    let global_index = read_state(deps.storage)?.global_index;

    let holder: Holder = read_holder(deps.storage, &deps.api.addr_validate(&address)?)?;
    let reward_with_decimals =
        calculate_decimal_rewards(global_index, holder.index, holder.balance)?;
    let all_reward_with_decimals = reward_with_decimals + holder.pending_rewards;

    let rewards = all_reward_with_decimals * Uint256::one();

    Ok(AccruedRewardsResponse { rewards : rewards.try_into()? })
}

pub fn query_holder(deps: Deps, address: String) -> StdResult<HolderResponse> {
    let holder: Holder = read_holder(deps.storage, &deps.api.addr_validate(&address)?)?;
    Ok(HolderResponse {
        address,
        balance: holder.balance,
        index: holder.index,
        pending_rewards: holder.pending_rewards,
    })
}

pub fn query_holders(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<HoldersResponse> {
    let holders: Vec<HolderResponse> = read_holders(deps, start_after, limit)?;

    Ok(HoldersResponse { holders })
}

// calculate the reward based on the sender's index and the global index.
fn calculate_decimal_rewards(
    global_index: Decimal256,
    user_index: Decimal256,
    user_balance: Uint128,
) -> StdResult<Decimal256> {
    let decimal_balance = Decimal256::from_ratio(user_balance, Uint128::new(1));
    Ok((global_index - user_index)*decimal_balance)
}

// calculate the reward with decimal
fn get_decimals(value: Decimal256) -> StdResult<Decimal256> {
    let stringed: &str = &value.to_string();
    let parts: &[&str] = &stringed.split('.').collect::<Vec<&str>>();
    match parts.len() {
        1 => Ok(Decimal256::zero()),
        2 => {
            let decimals = Decimal256::from_str(&("0.".to_owned() + parts[1]))?;
            Ok(decimals)
        }
        _ => Err(StdError::generic_err("Unexpected number of dots")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn proper_calculate_rewards() {
        let global_index = Decimal256::from_ratio(Uint128::new(9), Uint128::new(100));
        let user_index = Decimal256::zero();
        let user_balance = Uint128::new(1000);
        let reward = calculate_decimal_rewards(global_index, user_index, user_balance).unwrap();
        assert_eq!(reward.to_string(), "90");
    }

    #[test]
    pub fn proper_get_decimals() {
        let global_index = Decimal256::from_ratio(Uint128::new(9999999), Uint128::new(100000000));
        let user_index = Decimal256::zero();
        let user_balance = Uint128::new(10);
        let reward = get_decimals(
            calculate_decimal_rewards(global_index, user_index, user_balance).unwrap(),
        )
        .unwrap();
        assert_eq!(reward.to_string(), "0.9999999");
    }
}

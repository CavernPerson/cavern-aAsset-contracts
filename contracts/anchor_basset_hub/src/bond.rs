use crate::contract::{query_total_issued, slashing};
use crate::math::decimal_division;
use crate::state::{CONFIG, CURRENT_BATCH, PARAMETERS, STATE};
use basset::hub::State;
use cosmwasm_std::Coin;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StakingMsg, StdError,
    StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;

use lido_terra_validators_registry::common::calculate_delegations;
use lido_terra_validators_registry::msg::QueryMsg as QueryValidators;
use lido_terra_validators_registry::registry::ValidatorResponse;

pub fn execute_bond(mut deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let params = PARAMETERS.load(deps.storage)?;
    let coin_denom = params.underlying_coin_denom;
    let threshold = params.er_threshold;
    let recovery_fee = params.peg_recovery_fee;
    let config = CONFIG.load(deps.storage)?;

    // current batch requested fee is need for accurate exchange rate computation.
    let current_batch = CURRENT_BATCH.load(deps.storage)?;
    let requested_with_fee = current_batch.requested_with_fee;

    // coin must have be sent along with transaction and it should be in underlying coin denom
    if info.funds.len() > 1usize {
        return Err(StdError::generic_err(
            "More than one coin is sent; only one asset is supported",
        ));
    }

    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == coin_denom && x.amount > Uint128::zero())
        .ok_or_else(|| {
            StdError::generic_err(format!("No {} assets are provided to bond", coin_denom))
        })?;

    // check slashing
    slashing(&mut deps, env)?;

    let state = STATE.load(deps.storage)?;
    let sender = info.sender;

    // get the total supply
    let mut total_supply = query_total_issued(deps.as_ref()).unwrap_or_default();

    // peg recovery fee should be considered
    let mint_amount = decimal_division(payment.amount, state.exchange_rate);
    let mut mint_amount_with_fee = mint_amount;
    if state.exchange_rate < threshold {
        let max_peg_fee = mint_amount * recovery_fee;
        let required_peg_fee = ((total_supply + mint_amount + current_batch.requested_with_fee)
            .checked_sub(state.total_bond_amount + payment.amount))?;
        let peg_fee = Uint128::min(max_peg_fee, required_peg_fee);
        mint_amount_with_fee = (mint_amount.checked_sub(peg_fee))?;
    }

    // total supply should be updated for exchange rate calculation.
    total_supply += mint_amount_with_fee;

    // exchange rate should be updated for future
    STATE.update(deps.storage, |mut prev_state| -> StdResult<State> {
        prev_state.total_bond_amount += payment.amount;
        prev_state.update_exchange_rate(total_supply, requested_with_fee);
        Ok(prev_state)
    })?;

    let validators_registry_contract = if let Some(v) = config.validators_registry_contract {
        v
    } else {
        return Err(StdError::generic_err(
            "Validators registry contract address is empty",
        ));
    };
    let validators: Vec<ValidatorResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: validators_registry_contract.to_string(),
            msg: to_binary(&QueryValidators::GetValidatorsForDelegation {})?,
        }))?;

    if validators.is_empty() {
        return Err(StdError::generic_err("Validators registry is empty"));
    }

    let (_remaining_buffered_balance, delegations) =
        calculate_delegations(payment.amount, validators.as_slice())?;

    let mut external_call_msgs: Vec<cosmwasm_std::CosmosMsg> = vec![];
    for i in 0..delegations.len() {
        if delegations[i].is_zero() {
            continue;
        }
        external_call_msgs.push(cosmwasm_std::CosmosMsg::Staking(StakingMsg::Delegate {
            validator: validators[i].address.clone(),
            amount: Coin::new(delegations[i].u128(), payment.denom.as_str()),
        }));
    }

    // issue the basset token for sender
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: sender.to_string(),
        amount: mint_amount_with_fee,
    };

    let config = CONFIG.load(deps.storage)?;
    let token_address = config
        .token_contract
        .ok_or_else(|| StdError::generic_err("the token contract must have been registered"))?
        .to_string();

    external_call_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address,
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(external_call_msgs)
        .add_attributes(vec![
            attr("action", "mint"),
            attr("from", sender),
            attr("bonded", payment.amount),
            attr("minted", mint_amount_with_fee),
        ]))
}

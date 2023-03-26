use crate::state::{Parameters, CONFIG, PARAMETERS};
use basset::hub::Config;
use cosmwasm_std::{
    attr, CosmosMsg, Decimal, DepsMut, DistributionMsg, Env, MessageInfo, Response, StdError,
    StdResult,
};

/// Update general parameters
/// Only creator/owner is allowed to execute
#[allow(clippy::too_many_arguments)]
pub fn execute_update_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    epoch_period: Option<u64>,
    peg_recovery_fee: Option<Decimal>,
    er_threshold: Option<Decimal>,
) -> StdResult<Response> {
    // only owner can send this message.
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(StdError::generic_err("unauthorized"));
    }

    let params: Parameters = PARAMETERS.load(deps.storage)?;

    let new_params = Parameters {
        epoch_period: epoch_period.unwrap_or(params.epoch_period),
        underlying_coin_denom: params.underlying_coin_denom,
        unbonding_period: params.unbonding_period,
        peg_recovery_fee: peg_recovery_fee.unwrap_or(params.peg_recovery_fee),
        er_threshold: er_threshold.unwrap_or(params.er_threshold),
        reward_denom: params.reward_denom,
    };

    if new_params.peg_recovery_fee > Decimal::one() {
        return Err(StdError::generic_err(
            "peg_recovery_fee can not be greater than 1",
        ));
    }

    PARAMETERS.save(deps.storage, &new_params)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_params")]))
}

/// Update the config. Update the owner, reward and token contracts.
/// Only creator/owner is allowed to execute
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    reward_contract: Option<String>,
    token_contract: Option<String>,
    validators_registry_contract: Option<String>,
    //airdrop_registry_contract: Option<String>,
) -> StdResult<Response> {
    // only owner must be able to send this message.
    let conf = CONFIG.load(deps.storage)?;
    if info.sender != conf.creator {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    if let Some(o) = owner {
        let owner_raw = deps.api.addr_validate(o.as_str())?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.creator = owner_raw;
            Ok(last_config)
        })?;
    }
    if let Some(reward) = reward_contract {
        let reward_raw = deps.api.addr_validate(reward.as_str())?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.reward_contract = Some(reward_raw);
            Ok(last_config)
        })?;

        // register the reward contract for automate reward withdrawal.
        messages.push(CosmosMsg::Distribution(
            DistributionMsg::SetWithdrawAddress { address: reward },
        ));
    }

    if let Some(token) = token_contract {
        let token_raw = deps.api.addr_validate(token.as_str())?;

        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.token_contract = Some(token_raw);
            Ok(last_config)
        })?;
    }

    if let Some(validators_registry) = validators_registry_contract {
        let validators_raw = deps.api.addr_validate(&validators_registry)?;
        CONFIG.update(deps.storage, |mut last_config| -> StdResult<_> {
            last_config.validators_registry_contract = Some(validators_raw);
            Ok(last_config)
        })?;
    }
    /*
    if let Some(airdrop) = airdrop_registry_contract {
        let airdrop_raw = deps.api.addr_validate(airdrop.as_str())?;
        CONFIG.update(deps.storage, |mut last_config| -> StdResult<Config> {
            last_config.airdrop_registry_contract = Some(airdrop_raw);
            Ok(last_config)
        })?;
    }
    */
    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![attr("action", "update_config")]))
}

use cosmwasm_std::{Coin, Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type UnbondRequest = Vec<(u64, Uint128)>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub epoch_period: u64,
    pub underlying_coin_denom: String,
    pub unbonding_period: u64,
    pub peg_recovery_fee: Decimal,
    pub er_threshold: Decimal,
    pub reward_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct State {
    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub actual_unbonded_amount: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub creator: Addr,
    pub reward_contract: Option<Addr>,
    pub token_contract: Option<Addr>,
    pub validators_registry_contract: Option<Addr>,
    //pub airdrop_registry_contract: Option<Addr>,
}

impl State {
    pub fn update_exchange_rate(&mut self, total_issued: Uint128, requested_with_fee: Uint128) {
        let actual_supply = total_issued + requested_with_fee;
        if self.total_bond_amount.is_zero() || actual_supply.is_zero() {
            self.exchange_rate = Decimal::one()
        } else {
            self.exchange_rate = Decimal::from_ratio(self.total_bond_amount, actual_supply);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner's operations
    ////////////////////

    /// Set the owner
    UpdateConfig {
        owner: Option<String>,
        reward_contract: Option<String>,
        token_contract: Option<String>,
        validators_registry_contract: Option<String>,
        //airdrop_registry_contract: Option<String>,
    },

    /// update the parameters that is needed for the contract
    UpdateParams {
        epoch_period: Option<u64>,
        unbonding_period: Option<u64>,
        peg_recovery_fee: Option<Decimal>,
        er_threshold: Option<Decimal>,
    },

    ////////////////////
    /// User's operations
    ////////////////////

    /// Receives `amount` in underlying coin denom from sender.
    /// Delegate `amount` to multiple validators.
    /// Issue `amount` / exchange_rate for the user.
    Bond {
    },

    /// Update global index
    UpdateGlobalIndex {
        //airdrop_hooks: Option<Vec<Binary>>,
    },

    /// Send back unbonded coin to the user
    WithdrawUnbonded {},

    /// Check whether the slashing has happened or not
    CheckSlashing {},

    ////////////////////
    /// bAsset's operations
    ///////////////////

    /// Receive interface for send token.
    /// Unbond the underlying coin denom.
    /// Burn the received basset token.
    Receive(Cw20ReceiveMsg),


    ////////////////////
    /// internal operations
    ///////////////////
    RedelegateProxy {
        // delegator is automatically set to address of the calling contract
        src_validator: String,
        redelegations: Vec<(String, Coin)>, //(dst_validator, amount)
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    CurrentBatch {},
    WithdrawableUnbonded {
        address: String,
    },
    Parameters {},
    UnbondRequests {
        address: String,
    },
    AllHistory {
        start_from: Option<u64>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Unbond {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondHistory {
    pub batch_id: u64,
    pub time: u64,
    pub amount: Uint128,
    pub applied_exchange_rate: Decimal,
    pub withdraw_rate: Decimal,
    pub released: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub exchange_rate: Decimal,
    pub total_bond_amount: Uint128,
    pub last_index_modification: u64,
    pub prev_hub_balance: Uint128,
    pub actual_unbonded_amount: Uint128,
    pub last_unbonded_time: u64,
    pub last_processed_batch: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub reward_contract: Option<String>,
    pub token_contract: Option<String>,
    pub validator_registry_contract: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedValidatorsResponse {
    pub validators: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentBatchResponse {
    pub id: u64,
    pub requested_with_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondRequestsResponse {
    pub address: String,
    pub requests: UnbondRequest,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllHistoryResponse {
    pub history: Vec<UnbondHistory>,
}

use basset::dex_router::{
    AssetInfo, QueryMsg as SwapQueryMsg, SimulateSwapOperationsResponse, SwapOperation,
};
use basset::hub::Config;
use cosmwasm_std::from_binary;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::Empty;
use cosmwasm_std::Uint128;
use cosmwasm_std::{
    from_slice, to_binary, Api, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::marker::PhantomData;

pub const MOCK_HUB_CONTRACT_ADDR: &str = "hub";
pub const MOCK_REWARD_CONTRACT_ADDR: &str = "reward";
pub const MOCK_TOKEN_CONTRACT_ADDR: &str = "token";
pub const MOCK_REGISTRY_CONTRACT_ADDR: &str = "validator-registry";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = String::from(MOCK_CONTRACT_ADDR);
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(&contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                if *contract_addr == MOCK_HUB_CONTRACT_ADDR {
                    let prefix_config = to_length_prefixed(b"config").to_vec();
                    let api: MockApi = MockApi::default();
                    if key.as_slice().to_vec() == prefix_config {
                        let config = Config {
                            creator: api.addr_validate(&String::from("owner1")).unwrap(),
                            reward_contract: Some(
                                api.addr_validate(&String::from(MOCK_REWARD_CONTRACT_ADDR))
                                    .unwrap(),
                            ),
                            token_contract: Some(
                                api.addr_validate(&String::from(MOCK_TOKEN_CONTRACT_ADDR))
                                    .unwrap(),
                            ),
                            validators_registry_contract: Some(
                                api.addr_validate(&String::from(MOCK_REGISTRY_CONTRACT_ADDR))
                                    .unwrap(),
                            ),
                            /*
                            airdrop_registry_contract: Some(
                                api.addr_validate(&String::from("airdrop")).unwrap(),
                            ),
                            */
                        };
                        SystemResult::Ok(ContractResult::from(to_binary(&config)))
                    } else {
                        unimplemented!()
                    }
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if *contract_addr == "astroport_addr" {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::SimulateSwapOperations {
                            offer_amount,
                            operations,
                        } => {
                            #[allow(clippy::collapsible_match)]
                            if let SwapOperation::AstroSwap {
                                offer_asset_info, ..
                            } = operations[0].clone()
                            {
                                if let AssetInfo::NativeToken { denom: x } = offer_asset_info {
                                    if x == *"mnt" {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: "not covered".to_string(),
                                            request: msg.clone(),
                                        });
                                    }
                                }
                            }
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &SimulateSwapOperationsResponse {
                                    amount: offer_amount * Uint128::from(9u128)
                                        / Uint128::from(10u128),
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "not covered".to_string(),
                            request: msg.clone(),
                        }),
                    }
                } else if *contract_addr == "phoenix_addr" {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::SimulateSwapOperations {
                            offer_amount,
                            operations,
                        } => {
                            #[allow(clippy::collapsible_match)]
                            if let SwapOperation::TokenSwap {
                                offer_asset_info, ..
                            } = operations[0].clone()
                            {
                                if let AssetInfo::NativeToken { denom: x } = offer_asset_info {
                                    if x == *"mnt" {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: "not covered".to_string(),
                                            request: msg.clone(),
                                        });
                                    }
                                }
                            }
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &SimulateSwapOperationsResponse {
                                    amount: offer_amount * Uint128::from(11u128)
                                        / Uint128::from(10u128),
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "not covered".to_string(),
                            request: msg.clone(),
                        }),
                    }
                } else if *contract_addr == "terraswap_addr" {
                    match from_binary(msg).unwrap() {
                        SwapQueryMsg::SimulateSwapOperations {
                            offer_amount,
                            operations,
                        } => {
                            #[allow(clippy::collapsible_match)]
                            if let SwapOperation::TerraSwap {
                                offer_asset_info, ..
                            } = operations[0].clone()
                            {
                                if let AssetInfo::NativeToken { denom: x } = offer_asset_info {
                                    if x == *"mnt" {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: "not covered".to_string(),
                                            request: msg.clone(),
                                        });
                                    }
                                }
                            }
                            SystemResult::Ok(ContractResult::from(to_binary(
                                &SimulateSwapOperationsResponse {
                                    amount: offer_amount,
                                },
                            )))
                        }
                        _ => SystemResult::Err(SystemError::InvalidRequest {
                            error: "not covered".to_string(),
                            request: msg.clone(),
                        }),
                    }
                } else {
                    unimplemented!()
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier { base }
    }
}

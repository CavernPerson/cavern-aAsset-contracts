# 0.3.0
Take over of the contracts by CavernPerson. Include the lido-validator-registry contract that will allow for better validators management and automated staking choices. Remove all ANC and Airdrop related contracts and functions

# 0.2.1
Bug fix: `update_global_index` failed if there were tokens with unknown exchange rates  on the `anchor_basset_reward` contract balance. The solution is to handle only the tokens with known exchange rates.

# 0.2.0
Columbus-5 update

* Bump CosmWasm to [v0.16.0](https://github.com/CosmWasm/cosmwasm/releases/v0.16.0)

# 0.1.0

Initial Release

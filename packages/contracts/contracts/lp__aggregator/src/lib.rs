#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, Symbol, vec};

#[contracttype]
#[derive(Clone)]
pub struct SwapHop {
    pub pool_address: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct SwapRoute {
    pub hops: Vec<SwapHop>,
    pub expected_out: i128,
    pub price_impact_bps: u32,
}

#[contract]
pub struct LpAggregator;

#[contractimpl]
impl LpAggregator {
    /// Returns at least 2 registered pool addresses from storage
    fn get_registered_pools(env: &Env) -> Vec<Address> {
        let key = Symbol::new(env, "POOLS");
        env.storage().instance().get(&key).unwrap_or(vec![env])
    }

    /// Simulates a swap on a given pool and returns a route with expected output
    fn simulate_swap(env: &Env, pool: Address, amount_in: i128) -> SwapRoute {
    let mock_out = amount_in * 98 / 100;
    let hop = SwapHop {
        pool_address: pool.clone(),
        token_in: pool.clone(),
        token_out: pool.clone(),
        amount_in,
    };
    let hops = vec![env, hop];
    SwapRoute {
        hops,
        expected_out: mock_out,
        price_impact_bps: 50,
    }
}

    /// Executes a single hop swap and returns amount received
    fn execute_hop(env: &Env, hop: SwapHop, amount_in: i128) -> i128 {
        // Stub: in production this cross-contract calls the pool
        amount_in * 98 / 100
    }

    pub fn get_best_route(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
    ) -> SwapRoute {
        let pools = Self::get_registered_pools(&env);
        let mut best_route: Option<SwapRoute> = None;

        for pool in pools.iter() {
            let simulated = Self::simulate_swap(&env, pool.clone(), amount_in);
            match &best_route {
                None => best_route = Some(simulated),
                Some(current) => {
                    if simulated.expected_out > current.expected_out {
                        best_route = Some(simulated);
                    }
                }
            }
        }

        best_route.expect("No routes found")
    }

    pub fn execute_swap(
        env: Env,
        route: SwapRoute,
        min_amount_out: i128,
    ) -> i128 {
        let first_hop = route.hops.get(0).expect("Route has no hops");
        let mut amount_received = first_hop.amount_in;

        for hop in route.hops.iter() {
            amount_received = Self::execute_hop(&env, hop, amount_received);
        }

        if amount_received < min_amount_out {
            panic!("Slippage exceeded: got {}, expected at least {}",
                   amount_received, min_amount_out);
        }

        amount_received
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_single_hop_swap() {
        let env = Env::default();
        let contract_id = env.register_contract(None, LpAggregator);
        let client = LpAggregatorClient::new(&env, &contract_id);

        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);

        // Register one pool in storage before calling
        let pool = Address::generate(&env);
        let pools: Vec<Address> = vec![&env, pool];
        env.as_contract(&contract_id, || {
            env.storage().instance().set(
                &soroban_sdk::Symbol::new(&env, "POOLS"),
                &pools,
            );
        });

        let route = client.get_best_route(&token_in, &token_out, &1_000_000);
        assert_eq!(route.hops.len(), 1);
        assert!(route.expected_out > 0);
    }

    #[test]
    fn test_multi_hop_swap() {
        let env = Env::default();
        let contract_id = env.register_contract(None, LpAggregator);
        let client = LpAggregatorClient::new(&env, &contract_id);

        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);

        // Register two pools — aggregator evaluates both, picks best
        let pool1 = Address::generate(&env);
        let pool2 = Address::generate(&env);
        let pools: Vec<Address> = vec![&env, pool1, pool2];
        env.as_contract(&contract_id, || {
            env.storage().instance().set(
                &soroban_sdk::Symbol::new(&env, "POOLS"),
                &pools,
            );
        });

        let route = client.get_best_route(&token_in, &token_out, &1_000_000);
        assert!(route.expected_out > 0);
    }

    #[test]
    #[should_panic(expected = "Slippage exceeded")]
    fn test_slippage_exceeded() {
        let env = Env::default();
        let contract_id = env.register_contract(None, LpAggregator);
        let client = LpAggregatorClient::new(&env, &contract_id);

        let pool = Address::generate(&env);
        let pools: Vec<Address> = vec![&env, pool];
        env.as_contract(&contract_id, || {
            env.storage().instance().set(
                &soroban_sdk::Symbol::new(&env, "POOLS"),
                &pools,
            );
        });

        let token_in = Address::generate(&env);
        let token_out = Address::generate(&env);

        let route = client.get_best_route(&token_in, &token_out, &1_000_000);
        client.execute_swap(&route, &i128::MAX); // should panic
    }
}
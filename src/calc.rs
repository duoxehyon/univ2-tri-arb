use ethers::types::I256;
use serde::Deserialize;
use std::cell::RefCell;
use std::str::FromStr;
use tokio::sync::MutexGuard;

// use crate::contract_modules::uniswap_v2::swap_math::get_amount_out;
use crate::constants::WETH;
use crate::contract_modules::uniswap_v2::types::UniV2Pool;
use crate::state::State;
use ethers::types::{Address, U256};
use std::cmp::Ordering;

#[derive(Debug, Deserialize)]
pub struct NetPositiveCycle {
    pub profit: I256,
    pub optimal_in: U256,
    pub swap_amounts: Vec<U256>,
    pub cycle_addresses: Vec<Address>,
}

impl Ord for NetPositiveCycle {
    fn cmp(&self, other: &Self) -> Ordering {
        other.profit.cmp(&self.profit)
    }
}

impl PartialOrd for NetPositiveCycle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NetPositiveCycle {}

// Ordering based on profit
impl PartialEq for NetPositiveCycle {
    fn eq(&self, other: &Self) -> bool {
        self.profit == other.profit
    }
}

pub fn find_optimal_cycles(
    state: &MutexGuard<State>,
    affected_pairs: Option<Vec<Address>>,
) -> Vec<NetPositiveCycle> {
    let mut pointers: Vec<&Vec<crate::state::IndexedPair>> = Vec::new();

    match affected_pairs {
        Some(affected_pairs) => {
            affected_pairs.iter().for_each(|pair_address| {
                if let Some(cycle) = state.cycles_mapping.get(pair_address) {
                    pointers.extend(cycle.iter());
                }                
            });   
        }
        None => {
            for (_, cycles) in &state.cycles_mapping {
                pointers.extend(cycles.iter());
            }
        }
    }

    let mut net_profit_cycles = Vec::new();

    let weth = Address::from_str(WETH).unwrap();
    for cycle in pointers {
        let pairs = cycle
            .iter()
            .filter_map(|pair| state.pairs_mapping.get(&pair.address))
            .collect::<Vec<&RefCell<UniV2Pool>>>();

        let pairs_clone = pairs.clone();
        let profit_function =
            move |amount_in: U256| -> I256 { get_profit(weth, amount_in, &pairs_clone) };

        let optimal = maximize_profit(
            U256::one(),
            U256::from_dec_str("10000000000000000000000").unwrap(),
            U256::from_dec_str("10").unwrap(),
            profit_function,
        );

        let (profit, swap_amounts) = get_profit_with_amount(weth, optimal, &pairs);

        let mut cycle_internal = Vec::new();
        for pair in pairs {
            cycle_internal.push(pair.borrow().address);
        }

        if profit > I256::one() {
            let net_positive_cycle = NetPositiveCycle {
                profit,
                optimal_in: optimal,
                cycle_addresses: cycle_internal,
                swap_amounts,
            };
            net_profit_cycles.push(net_positive_cycle);
        }
    }

    net_profit_cycles.sort();
    net_profit_cycles.into_iter().take(5).collect()
}

// find optimal input before uni fees eats away our profits
// Quadratic search
fn maximize_profit(
    mut domain_min: U256,
    mut domain_max: U256,
    lowest_delta: U256,
    f: impl Fn(U256) -> I256,
) -> U256 {
    loop {
        if domain_max > domain_min {
            if (domain_max - domain_min) > lowest_delta {
                let mid = (domain_min + domain_max) / 2;

                let lower_mid = (mid + domain_min) / 2;
                let upper_mid = (mid + domain_max) / 2;

                let f_output_lower = f(lower_mid);
                let f_output_upper = f(upper_mid);

                if f_output_lower > f_output_upper {
                    domain_max = mid;
                } else {
                    domain_min = mid;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    (domain_max + domain_min) / 2
}

/// Calculates profit given (state updated) pairs
pub fn get_profit(token_in: Address, amount_in: U256, pairs: &Vec<&RefCell<UniV2Pool>>) -> I256 {
    let mut amount_out: U256 = amount_in;
    let mut token_in = token_in;
    for pair in pairs {
        let pair = pair.borrow();
        let fees;
        let (reserve0, reserve1) = if pair.token0 == token_in {
            fees = pair.fees1;
            (pair.reserve0, pair.reserve1)
        } else {
            fees = pair.fees0;
            (pair.reserve1, pair.reserve0)
        };
        amount_out = get_amount_out(amount_out, reserve0, reserve1, fees, pair.router_fee);
        token_in = if pair.token0 == token_in {
            pair.token1
        } else {
            pair.token0
        };
    }

    I256::from_raw(amount_out) - I256::from_raw(amount_in)
}

pub fn get_profit_with_amount(
    token_in: Address,
    amount_in: U256,
    pairs: &Vec<&RefCell<UniV2Pool>>,
) -> (I256, Vec<U256>) {
    let mut amount_out: U256 = amount_in;
    let mut token_in = token_in;
    let mut amounts = Vec::with_capacity(pairs.len() + 1);
    amounts.push(amount_in);
    for pair in pairs {
        let pair = pair.borrow();
        let fees;
        let (reserve0, reserve1) = if pair.token0 == token_in {
            fees = pair.fees1;
            (pair.reserve0, pair.reserve1)
        } else {
            fees = pair.fees0;
            (pair.reserve1, pair.reserve0)
        };
        amount_out = get_amount_out(amount_out, reserve0, reserve1, fees, pair.router_fee);
        amounts.push(amount_out);
        token_in = if pair.token0 == token_in {
            pair.token1
        } else {
            pair.token0
        };
    }

    (
        I256::from_raw(amount_out) - I256::from_raw(amount_in),
        amounts,
    )
}

// We don't want overflow / underflow at runtime + need to be a bit fast
pub fn get_amount_out(
    a_in: U256,
    reserve_in: U256,
    reserve_out: U256,
    fees: U256,
    router_fee: U256,
) -> U256 {
    if a_in == U256::zero() {
        return U256::zero();
    }
    let a_in_with_fee = a_in.saturating_mul(router_fee);
    let a_out = a_in_with_fee.saturating_mul(reserve_out)
        / U256::from(10000)
            .saturating_mul(reserve_in)
            .saturating_add(a_in_with_fee);

    a_out - a_out.saturating_mul(fees) / U256::from(10000)
}


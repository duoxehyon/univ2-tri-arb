use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use crate::contract_modules::uniswap_v2::types::UniV2Pool;
use crate::helpers;
use crate::constants::WETH;
use ethers::prelude::*;
use log::*;
use serde::{Serialize, Deserialize};
use tokio::sync::MutexGuard;

pub type Cycle = Vec<IndexedPair>;

/// Points to the addresses, this makes state updates easier
#[derive(Debug, Clone, Copy)]
pub struct IndexedPair {
    pub address: usize,

    pub token0: usize,
    pub token1: usize,
}

#[derive(Debug, Clone)]
pub struct PointerIndexedCycle {
    /// Cycle in indexed rep
    pub cycle: Vec<IndexedPair>,
}

pub struct State {
    /// For indexed pointer to address
    pub index_mapping: HashMap<usize, Address>,
    /// For address to indexed pointer
    pub address_mapping: HashMap<Address, usize>,
    /// Pointer to the pool
    pub pairs_mapping: HashMap<usize, RefCell<UniV2Pool>>,
    /// For easy access at pending state
    pub cycles_mapping: HashMap<Address, Vec<Cycle>>,
    // Real state of reserves to re apply after calc
    real_reserve_state: RefCell<HashMap<usize, [U256; 2]>>,
}

// Potential future state update
#[derive(Deserialize, Serialize, Debug)]
pub struct StateUpdateInternal {
    pub address: Address,
    pub reserve0: U256,
    pub reserve1: U256,
}

impl State {
    /// Initialize state
    pub fn new_state(pairs: &[UniV2Pool]) -> Self {
        let mut address_mapping = HashMap::new();
        let mut index_mapping = HashMap::new();
        let mut pairs_mapping = HashMap::new();

        for pair in pairs.iter() {
            let current_len = index_mapping.len();
            index_mapping.insert(current_len, pair.address);
            address_mapping.insert(pair.address, current_len);

            let token0_exists = address_mapping.contains_key(&pair.token0);
            if !token0_exists {
                let current_len = index_mapping.len();
                index_mapping.insert(current_len, pair.token0);
                address_mapping.insert(pair.token0, current_len);
            }

            let token1_exists = address_mapping.contains_key(&pair.token1);
            if !token1_exists {
                let current_len = index_mapping.len();
                index_mapping.insert(current_len, pair.token1);
                address_mapping.insert(pair.token1, current_len);
            }
        }

        let mut indexed_pairs = Vec::new();
        for pair in pairs {
            let indexed_pair = IndexedPair {
                address: *address_mapping.get(&pair.address).unwrap(),
                token0: *address_mapping.get(&pair.token0).unwrap(),
                token1: *address_mapping.get(&pair.token1).unwrap(),
            };

            indexed_pairs.push(indexed_pair);
            pairs_mapping.insert(
                *address_mapping.get(&pair.address).unwrap(),
                RefCell::new(pair.clone()),
            );
        }

        let weth_index = *address_mapping.get(&helpers::address(WETH)).unwrap();
        let now = std::time::Instant::now();

        let cycles = Self::find_cycles(
            &indexed_pairs,
            weth_index,
            weth_index,
            3, // should be enough
            &Vec::new(),
            &mut Vec::new(),
            &mut HashSet::new(),
        );

        info!("Number of cycles: {:?}", cycles.len());
        info!("Time took for finding all cycles: {:?}", now.elapsed());

        let mut cycles_mapping = HashMap::new();

        for indexed_cycle in cycles.iter() {
            for indexed_pair in indexed_cycle {
                cycles_mapping
                    .entry(index_mapping[&indexed_pair.address])
                    .or_insert_with(Vec::new)
                    .push(indexed_cycle.clone());
            }
        }

        let real_reserve_state = RefCell::new(HashMap::new());

        Self {
            index_mapping,
            address_mapping,
            pairs_mapping,
            cycles_mapping,
            real_reserve_state,
        }
    }

    /// Find cycles using DFS
    fn find_cycles(
        pairs: &[IndexedPair],
        token_in: usize,
        token_out: usize,
        max_hops: i32,
        current_pairs: &Vec<IndexedPair>,
        circles: &mut Vec<Cycle>,
        seen: &mut HashSet<usize>,
    ) -> Vec<Cycle> {
        let mut circles_copy: Vec<Cycle> = circles.clone();

        for pair in pairs {
            if seen.contains(&pair.address) {
                continue;
            }

            let temp_out: usize;
            if token_in == pair.token0 {
                temp_out = pair.token1;
            } else if token_in == pair.token1 {
                temp_out = pair.token0;
            } else {
                continue;
            }

            let mut new_seen = seen.clone();
            new_seen.insert(pair.address);

            if temp_out == token_out {
                let mut new_cycle = current_pairs.clone();
                new_cycle.push(*pair);
                circles_copy.push(new_cycle);
            } else if max_hops > 1 {
                let mut new_pairs: Vec<IndexedPair> = current_pairs.clone();
                new_pairs.push(*pair);
                circles_copy = Self::find_cycles(
                    pairs,
                    temp_out,
                    token_out,
                    max_hops - 1,
                    &new_pairs,
                    &mut circles_copy,
                    &mut new_seen,
                );
            }
        }

        circles_copy
    }

    pub fn apply_state_temp(state: &mut MutexGuard<State>, updates: Vec<StateUpdateInternal>) {
        for update in updates {
            let pair_address_index: usize = match state.address_mapping.get(&update.address) {
                Some(d) => *d,
                None => continue,
            };

            let mut pair = match state.pairs_mapping.get(&pair_address_index) {
                Some(d) => d.borrow_mut(),
                None => {
                    continue;
                }
            };

            state
                .real_reserve_state
                .borrow_mut()
                .insert(pair_address_index, [pair.reserve0, pair.reserve1]);

            pair.reserve0 = update.reserve0;
            pair.reserve1 = update.reserve1
        }
    }

    pub fn reset_temp_state(state: &mut MutexGuard<State>) {
        for (index, update) in state.real_reserve_state.borrow().iter() {
            let mut pair = match state.pairs_mapping.get(index) {
                Some(d) => d.borrow_mut(),
                None => continue,
            };
            pair.reserve0 = update[0];
            pair.reserve1 = update[1];
        }

        state.real_reserve_state.borrow_mut().clear();
    }
}

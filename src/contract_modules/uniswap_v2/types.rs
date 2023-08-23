use ethers::prelude::*;
use serde::{Deserialize, Serialize};

// Uniswap V2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniV2 {
    // Factory
    pub factory: Address,
    // Router
    pub router: Address,
    // Dex Fee
    pub fee: U256,
    // Init code hash
    pub init_code_hash: H256,
}
/// Uniswap V2 Pool (and its forks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniV2Pool {
    pub address: Address,

    pub token0: Address,
    pub token1: Address,

    pub reserve0: U256,
    pub reserve1: U256,

    // router fee
    pub router_fee: U256,
    //  token tax when token0 is in
    pub fees0: U256,
    //  token tax when token1 is in
    pub fees1: U256,
}

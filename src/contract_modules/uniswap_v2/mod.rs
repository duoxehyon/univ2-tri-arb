pub mod bindings;
pub mod calc;
pub mod checkpoint;
pub mod constants;
pub mod data_collector;
pub mod types;

use std::str::FromStr;
use ethers::types::{H256, U256};
use crate::{constants::UNISWAP_V2, helpers::address};
use types::UniV2;

pub fn get_uni_v2() -> Vec<UniV2> {
    UNISWAP_V2
        .iter()
        .map(|dex| UniV2 {
            router: address(dex.0),
            factory: address(dex.1),
            init_code_hash: H256::from_str(dex.2).unwrap(),
            fee: U256::from(dex.3),
        })
        .collect()
}
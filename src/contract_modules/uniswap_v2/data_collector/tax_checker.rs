use std::str::FromStr;

use crate::components::simulator::fork_db::ForkDB;
use crate::components::simulator::fork_factory::ForkFactory;
use crate::contract_modules::uniswap_v2::constants::*;

use ethers::abi::parse_abi;
use ethers::prelude::*;
use ethers::utils::parse_ether;
use revm::primitives::{Address as rAddress, Bytecode, U256 as rU256};
use revm::primitives::{ExecutionResult, Output, TransactTo};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_tax(
    token_in: Address,
    pair: Address,
    sand_box_db: ForkDB,
    latest_block: U64,
    fee: U256,
) -> Option<(U256, U256)> {
    let mut evm: revm::EVM<ForkDB> = revm::EVM::new();
    evm.database(sand_box_db);

    evm.env.block.number = rU256::from(latest_block.as_u64());
    evm.env.block.timestamp = rU256::from(get_current_unix_time_seconds());
    evm.env.block.basefee = rU256::from(1000000);
    evm.env.block.coinbase =
        rAddress::from_str("0xDecafC0FFEe15BAD000000000000000000000000").unwrap();

    let tx_data = build_tax_checker_data(token_in, pair, U256::from(10000), fee);

    evm.env.tx.caller = tax_checker_controller_address();
    evm.env.tx.transact_to = TransactTo::Call(tax_checker_address().0.into());
    evm.env.tx.data = tx_data.0;
    evm.env.tx.gas_limit = 7000000;
    evm.env.tx.gas_price = rU256::from(1000000000);
    evm.env.tx.value = rU256::ZERO;

    let result: ExecutionResult = match evm.transact_commit() {
        Ok(result) => result,
        Err(_) => {
            return None;
        }
    };

    let output = match result {
        ExecutionResult::Success { output, .. } => match output {
            Output::Call(o) => o,
            Output::Create(o, _) => o,
        },
        ExecutionResult::Revert { .. } => {
            return None;
        }
        ExecutionResult::Halt { .. } => {
            return None;
        }
    };

    let (buy_tax, sell_tax) = match decode_tax_checker_data(output.into()) {
        Ok(d) => d,
        Err(_) => {
            return None;
        }
    };

    if buy_tax > U256::from(9970) || sell_tax > U256::from(9970) {
        return None;
    }

    Some((buy_tax, sell_tax))
}

fn get_current_unix_time_seconds() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_secs()
}

pub fn insert_fake_approval(token: Address, pair: Address, fork_factory: &mut ForkFactory) {
    for i in 0..100 {
        let slot_new = map_location(U256::from(i), pair, tax_checker_address().0.into());
        fork_factory
            .insert_account_storage(
                token.0.into(),
                slot_new.into(),
                rU256::from_str("10000000000000000000000000000000000000000000000").unwrap(),
            )
            .unwrap();
    }
}

pub fn map_location(slot: U256, key: Address, key_after: Address) -> U256 {
    let key_slot_hash: U256 = ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address(key),
        abi::Token::Uint(slot),
    ]))
    .into();

    let slot: U256 = ethers::utils::keccak256(abi::encode(&[
        abi::Token::Address(key_after),
        abi::Token::Uint(key_slot_hash),
    ]))
    .into();

    slot
}

pub fn build_tax_checker_data(
    token_in: Address,
    target_pool: Address,
    out_of: U256,
    fees: U256,
) -> Bytes {
    let tax_contract = BaseContract::from(parse_abi(&[
        "function CheckTax(address token,address pair,uint256 outOf,uint256 fees) public returns(uint256,uint256)",
    ]).unwrap());

    tax_contract
        .encode("CheckTax", (token_in, target_pool, out_of, fees))
        .unwrap()
}

pub fn decode_tax_checker_data(output: Bytes) -> Result<(U256, U256), AbiError> {
    let tax_contract = BaseContract::from(parse_abi(&[
        "function CheckTax(address token,address pair,uint256 outOf,uint256 fees) public returns(uint256,uint256)",
    ]).unwrap());

    tax_contract.decode_output("CheckTax", output)
}

pub fn inject_tax_checker_code(fork_factory: &mut ForkFactory) {
    let account = revm::primitives::AccountInfo::new(
        rU256::from(0),
        0,
        Bytecode::new_raw(get_tax_checker_code().0),
    );
    fork_factory.insert_account_info(tax_checker_address().0.into(), account);

    // setup braindance contract controller
    let account =
        revm::primitives::AccountInfo::new(parse_ether(69).unwrap().into(), 0, Bytecode::default());
    fork_factory.insert_account_info(tax_checker_controller_address().0.into(), account);
}

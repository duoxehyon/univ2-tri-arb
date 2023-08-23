use crate::{constants::UniV2Factory, contract_modules::uniswap_v2::types::UniV2Pool};
use ethers::{
    abi::{ParamType, Token},
    prelude::abigen,
    providers::Middleware,
    types::{Address, Bytes, H160, U256},
};
use indicatif::ProgressBar;
use std::sync::Arc;

abigen!(
    GetUniswapV2PairsBatchRequest,
    "src/abi/BatchCollector.json";
);

pub async fn get_pairs_batch_request<M: Middleware>(
    factory: H160,
    from: U256,
    step: U256,
    middleware: Arc<M>,
) -> Vec<UniV2Pool> {
    let mut pairs = vec![];

    let constructor_args = Token::Tuple(vec![
        Token::Uint(from),
        Token::Uint(step),
        Token::Address(factory),
    ]);

    let deployer = GetUniswapV2PairsBatchRequest::deploy(middleware, constructor_args).unwrap();
    let return_data: Bytes = deployer.call_raw().await.unwrap();

    let return_data_tokens = ethers::abi::decode(
        &[ParamType::Array(Box::new(ParamType::Tuple(vec![
            ParamType::Address,   // pool address
            ParamType::Address,   // token a
            ParamType::Address,   // token b
            ParamType::Uint(112), // reserve 0
            ParamType::Uint(112), // reserve 1
        ])))],
        &return_data,
    )
    .unwrap();

    for tokens in return_data_tokens {
        if let Some(tokens_arr) = tokens.into_array() {
            for tup in tokens_arr {
                if let Some(pool_data) = tup.into_tuple() {
                    //If the pool token A is not zero, signaling that the pool data was populated
                    if !pool_data[0].to_owned().into_address().unwrap().is_zero() {
                        //Update the pool data
                        let pool_internal = UniV2Pool {
                            address: pool_data[0].to_owned().into_address().unwrap(),
                            token0: pool_data[1].to_owned().into_address().unwrap(),
                            token1: pool_data[2].to_owned().into_address().unwrap(),
                            reserve0: pool_data[3].to_owned().into_uint().unwrap(),
                            reserve1: pool_data[4].to_owned().into_uint().unwrap(),
                            router_fee: U256::from(9970),

                            fees0: U256::zero(),
                            fees1: U256::zero(),
                        };

                        pairs.push(pool_internal);
                    }
                }
            }
        }
    }

    pairs
}

pub async fn get_all_pairs_via_batched_calls<M: 'static + Middleware>(
    factory_address: Address,
    middleware: Arc<M>,
    progress_bar: ProgressBar,
) -> Vec<UniV2Pool> {
    let factory = UniV2Factory::new(factory_address, middleware.clone());

    let pairs_length: U256 = factory.all_pairs_length().call().await.unwrap();
    //Initialize the progress bar message
    progress_bar.set_length(pairs_length.as_u64());

    let mut pairs = vec![];
    let step = 150; //max batch size for this call until codesize is too large
    let mut idx_from = U256::zero();
    let mut idx_to = if step > pairs_length.as_usize() {
        pairs_length
    } else {
        U256::from(step)
    };

    for _ in (0..pairs_length.as_u128()).step_by(step) {
        pairs.append(
            &mut get_pairs_batch_request(factory_address, idx_from, idx_to, middleware.clone())
                .await,
        );

        idx_from = idx_to;

        if idx_to + step > pairs_length {
            idx_to = pairs_length - 1
        } else {
            idx_to = idx_to + step;
        }
        progress_bar.inc(step as u64);
    }
    progress_bar.reset();
    pairs
}

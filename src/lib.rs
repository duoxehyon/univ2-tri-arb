pub mod calc;
pub mod components;
pub mod config;
pub mod constants;
pub mod contract_modules;
pub mod helpers;
pub mod recon;
pub mod state;
pub mod states;
pub mod updater;
pub mod utils;

use config::Config;
use contract_modules::uniswap_v2::checkpoint::Storage;
use crossbeam_channel::unbounded;
use ethers::utils::format_units;
use state::State;
use std::time::{Duration, Instant};
use std::sync::Arc;

use log::*;
use tokio::sync::Mutex;
use ethers::prelude::*;

use crate::calc::find_optimal_cycles;
use crate::contract_modules::uniswap_v2::data_collector::data_collector::update_reserves;
use crate::contract_modules::uniswap_v2::get_uni_v2;
use crate::state::StateUpdateInternal;
use contract_modules::uniswap_v2;

pub fn init() {}

// TODO: make code less ugly
pub async fn run(at_exit: std::sync::mpsc::Receiver<()>) {
    info!("Starting...");
    tokio::task::spawn(exit(at_exit));

    let config = Config::new().await;
    let uni_v2 = get_uni_v2();
    let load = should_load_data_from_file();

    let mut pairs;

    if !load {
        let now = Instant::now();
        pairs = match uniswap_v2::data_collector::data_collector::get_all_pairs(
            uni_v2.clone(),
            config.wss.clone(),
        )
        .await
        {
            Some(d) => d,
            None => return,
        };
        info!("time took for query: {:?}", now.elapsed());
    } else {
        let storage = Storage::load_from_file("./db.json").expect("Failed on loading data");
        pairs = storage.pools;
    }

    let block = match config.wss.get_block_number().await {
        Ok(block) => block,
        Err(error) => {
            println!("An error occurred: {}", error);
            return;
        }
    };

    update_reserves(&mut pairs, uni_v2.clone(), config.wss.clone()).await;

    info!("Length of pairs: {:?}", pairs.len());

    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(state::State::new_state(&pairs)));

    // tokio::task::spawn(run_exit_save(at_exit, state.clone(), config.wss.clone()));
    let block_oracle = states::block_state::BlockOracle::new(config.wss.clone())
        .await
        .expect("Panic at block oracle creation");

    tokio::task::spawn(updater::start_updater(
        Arc::clone(&config.wss),
        state.clone(),
        block,
    ));

    // Give time to  sync Uni data
    std::thread::sleep(Duration::from_secs(20));

    let (s,r) = unbounded();
    recon::mempool::start_recon(state.clone(), config.wss.clone(), block_oracle.clone(), s).await;
    
    let decoded = hex::decode(constants::SYNC_TOPIC).unwrap();
    let sync_topic = H256::from_slice(&decoded);

    loop {
        let data = r.recv().unwrap();

        let mut state = state.lock().await;
        let mut pending_state_updates = Vec::new();
        let mut affected_pairs = Vec::new();

        for log in data.logs {
            let topics = match log.topics {
                Some(d) => d,
                None => continue
            };

            let data = match log.data {
                Some(d) => d,
                None => continue
            };

            let address = match log.address {
                Some(d) => d,
                None => continue
            };

            let mut reserve0 = U256::zero();
            let mut reserve1: U256 = U256::zero();
            let mut found_swap = false;

            for topic in topics {
                if topic == sync_topic {
                    reserve0 = U256::from_big_endian(&data[0..32]);
                    reserve1 = U256::from_big_endian(&data[32..]);
                    found_swap = true;
                }
            }
        
            if found_swap {    
                pending_state_updates.push(StateUpdateInternal {
                    address,
                    reserve0,
                    reserve1
                });

                affected_pairs.push(address);
            }
        }

        if pending_state_updates.is_empty() { continue }
        State::apply_state_temp(&mut state, pending_state_updates);

        let cycles = find_optimal_cycles(&state, Some(affected_pairs));
        
        let after: Duration = data.time.elapsed();
        if !cycles.is_empty() {
            info!(
                "                  ------> BackRun Tx Hash {:?}",
                data.tx.hash()
            );
            info!(
                "                  ------> Profit: {:.9} ",
                format_units(cycles[0].profit, "ether").unwrap()
            );
            info!(
                "                  ------> Optimal WETH In: {:.9} ",
                format_units(cycles[0].optimal_in, "ether").unwrap()
            );
            info!(
                "                  ------> E2E time: {:?} ",
                after
            );
            info!(
                "             ",
            );
        }

        State::reset_temp_state(&mut state);
    }
}

fn should_load_data_from_file() -> bool {
    let args: Vec<String> = std::env::args().collect();

    args.iter().any(|arg| arg == "load")
}

async fn exit(signal_at: std::sync::mpsc::Receiver<()>) {
    signal_at.recv().unwrap();
    std::process::exit(0);
}

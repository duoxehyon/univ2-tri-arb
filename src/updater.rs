use ethers::prelude::*;
use hex;
use log::*;
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;

use crate::{constants::SYNC_TOPIC, state::State};

pub async fn start_updater(ws_provider: Arc<Provider<Ws>>, state: Arc<Mutex<State>>, from: U64) {
    let now = Instant::now();

    let decoded = hex::decode(SYNC_TOPIC).unwrap();
    let sync_topic = H256::from_slice(&decoded);

    let mut from = from;
    let block = match ws_provider.get_block_number().await {
        Ok(d) => d,
        Err(error) => {
            error!("An error occurred: {}", error);
            return;
        }
    };

    while from < block {
        update_block(ws_provider.clone(), state.clone(), from, sync_topic).await;
        from += U64::one();
    }

    info!(
        "State updates from bot sync completed | Took: {:?}",
        now.elapsed()
    );
    loop_blocks(ws_provider, state, sync_topic).await;
}

pub async fn loop_blocks(
    ws_provider: Arc<Provider<Ws>>,
    state: Arc<Mutex<State>>,
    sync_topic: H256,
) {
    info!("Block updater started");
    let mut subscription = ws_provider.subscribe_blocks().await.unwrap();
    loop {
        if let Some(block) = subscription.next().await {
            update_block(
                ws_provider.clone(),
                state.clone(),
                block.number.unwrap(),
                sync_topic,
            )
            .await;
            // info!("------{:?}------",block.number);
        }
    }
}

async fn update_block(
    ws_provider: Arc<Provider<Ws>>,
    state: Arc<Mutex<State>>,
    block: U64,
    sync_topic: H256,
) {
    let block = match ws_provider.get_block(block).await {
        Ok(Some(d)) => d,
        Ok(None) => return,
        Err(error) => {
            println!("An error occurred: {}", error);
            return;
        }
    };
    let txes = block.transactions;

    for tx in txes {
        let tx_receipt = match ws_provider.get_transaction_receipt(tx).await {
            Ok(tx) => tx,
            Err(_) => continue,
        };
        if let Some(full_tx) = tx_receipt {
            let state_unlocked: tokio::sync::MutexGuard<State> = state.lock().await;

            let logs = full_tx.logs;
            for log in logs {
                let pointer = match state_unlocked.address_mapping.get(&log.address) {
                    Some(d) => *d,
                    None => continue,
                };

                for topic in log.topics {
                    if topic == sync_topic {
                        let mut pair = match state_unlocked.pairs_mapping.get(&pointer) {
                            Some(p) => p.borrow_mut(),
                            None => continue,
                        };

                        pair.reserve0 = U256::from_big_endian(&log.data[0..32]);
                        pair.reserve1 = U256::from_big_endian(&log.data[32..]);
                    }
                }
            }
            
        }
    }
}

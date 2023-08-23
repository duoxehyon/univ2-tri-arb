use crossbeam_channel::{Sender, TrySendError};
use ethers::prelude::*;
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::time::Instant;
use tokio::task::spawn;
use crate::states::block_state::BlockOracle;
use crate::utils::get_logs;
use crate::state::State;

pub struct FutureTx {
    pub tx: Transaction,
    pub logs: Vec<CallLogFrame>,
    pub time: Instant,
}

pub async fn start_recon(
    state: Arc<Mutex<State>>,
    wss: Arc<Provider<Ws>>,
    block_oracle: Arc<RwLock<BlockOracle>>,
    send_to: Sender<FutureTx>,
) {
    spawn(async move {
        let mut subscription: SubscriptionStream<Ws, TxHash> =
            wss.subscribe_pending_txs().await.expect("WSS gave up");

        loop {
            if let Some(tx_hash) = subscription.next().await {
                let mut full_tx = match wss.get_transaction(tx_hash).await {
                    Ok(Some(d)) => d,
                    _ => continue,
                };

                if full_tx.to.is_none() {
                    continue;
                }

                if let Ok(from) = full_tx.recover_from() {
                    full_tx.from = from;
                } else {
                    continue;
                }

                let latest_block;
                let next_base_fee;
                {
                    let block_oracle = block_oracle.read().await;
                    latest_block = BlockNumber::Number(block_oracle.latest_block.number);
                    next_base_fee = block_oracle.next_block.base_fee;
                }

                if full_tx.max_fee_per_gas.unwrap_or(U256::zero()) < next_base_fee {
                    continue;
                }

                let now = Instant::now();
                let logs = match get_logs(&wss, &full_tx, latest_block).await {
                    Some(d) => d,
                    None => continue,
                };

                let significant_logs = {
                    let state = state.lock().await;
                    logs.into_iter()
                        .filter_map(|log| {
                            let origin = log.address?;
                            let ptr = state.address_mapping.get(&origin)?;
                            if state.pairs_mapping.contains_key(ptr) {
                                Some(log)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<CallLogFrame>>()
                };

                if !significant_logs.is_empty() {
                    match send_to.try_send(FutureTx {
                        tx: full_tx,
                        logs: significant_logs,
                        time: now,
                    }) {
                        Ok(_) => (),
                        Err(TrySendError::Full(_)) => continue,
                        Err(TrySendError::Disconnected(_)) => break,
                    }
                }
            }
        }
    });
}
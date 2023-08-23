use std::sync::Arc;

use ethers::prelude::*;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct BlockInfo {
    pub number: U64,
    pub timestamp: U256,
    pub base_fee: U256,
}

impl BlockInfo {
    // Create a new `BlockInfo` instance
    pub fn new(number: U64, timestamp: U256, base_fee: U256) -> Self {
        Self {
            number,
            timestamp,
            base_fee,
        }
    }

    // Find the next block ahead of `prev_block`
    pub fn find_next_block_info(prev_block: Block<TxHash>) -> Self {
        let number = prev_block.number.unwrap_or_default() + 1;
        let timestamp = prev_block.timestamp + 12;
        let base_fee = Self::calculate_next_block_base_fee(prev_block);

        Self {
            number,
            timestamp,
            base_fee,
        }
    }

    pub fn calculate_next_block_base_fee(block: Block<TxHash>) -> U256 {
        // Get the block base fee per gas
        let current_base_fee_per_gas = block.base_fee_per_gas.unwrap_or_default();

        // Get the mount of gas used in the block
        let current_gas_used = block.gas_used;

        let current_gas_target = block.gas_limit / 2;

        if current_gas_used == current_gas_target {
            current_base_fee_per_gas
        } else if current_gas_used > current_gas_target {
            let gas_used_delta = current_gas_used - current_gas_target;
            let base_fee_per_gas_delta =
                current_base_fee_per_gas * gas_used_delta / current_gas_target / 8;

            return current_base_fee_per_gas + base_fee_per_gas_delta;
        } else {
            let gas_used_delta = current_gas_target - current_gas_used;
            let base_fee_per_gas_delta =
                current_base_fee_per_gas * gas_used_delta / current_gas_target / 8;

            return current_base_fee_per_gas - base_fee_per_gas_delta;
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockOracle {
    pub latest_block: BlockInfo,
    pub next_block: BlockInfo,
}

impl BlockOracle {
    // Create new latest block oracle
    pub async fn new(client: Arc<Provider<Ws>>) -> Result<Arc<RwLock<Self>>, ProviderError> {
        let latest_block = match client.get_block(BlockNumber::Latest).await {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

        let lb = if let Some(b) = latest_block {
            b
        } else {
            return Err(ProviderError::CustomError("Block not found".to_string()));
        };

        // latets block info
        let number = lb.number.unwrap();
        let timestamp = lb.timestamp;
        let base_fee = lb.base_fee_per_gas.unwrap_or_default();

        let latest_block = BlockInfo::new(number, timestamp, base_fee);

        // next block info
        let number = number + 1;
        let timestamp = timestamp + 12;
        let base_fee = BlockInfo::calculate_next_block_base_fee(lb);

        let next_block = BlockInfo::new(number, timestamp, base_fee);

        let oracle = Arc::new(RwLock::new(BlockOracle {
            latest_block,
            next_block,
        }));

        Self::start(oracle.clone(), client.clone()).await;

        Ok(oracle.clone())
    }

    async fn start(oracle: Arc<RwLock<BlockOracle>>, client: Arc<Provider<Ws>>) {
        tokio::task::spawn(async move {
            // loop so we can reconnect if the websocket connection is lost
            loop {
                let mut block_stream = if let Ok(stream) = client.subscribe_blocks().await {
                    stream
                } else {
                    panic!("Failed to create new block stream");
                };

                while let Some(block) = block_stream.next().await {
                    // lock the RwLock for write access and update the variable
                    {
                        let mut lock = oracle.write().await;
                        lock.update_block_number(block.number.unwrap());
                        lock.update_block_timestamp(block.timestamp);
                        lock.update_base_fee(block);
                    } // remove write lock due to being out of scope here
                }
            }
        });
    }

    // Updates block's number
    fn update_block_number(&mut self, block_number: U64) {
        self.latest_block.number = block_number;
        self.next_block.number = block_number + 1;
    }

    // Updates block's timestamp
    fn update_block_timestamp(&mut self, timestamp: U256) {
        self.latest_block.timestamp = timestamp;
        self.next_block.timestamp = timestamp + 12;
    }

    // Updates block's base fee
    fn update_base_fee(&mut self, latest_block: Block<TxHash>) {
        self.latest_block.base_fee = latest_block.base_fee_per_gas.unwrap_or_default();
        self.next_block.base_fee = BlockInfo::calculate_next_block_base_fee(latest_block);
    }
}

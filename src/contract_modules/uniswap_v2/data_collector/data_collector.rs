use super::collector::get_all_pairs_via_batched_calls;

use crate::components::simulator::fork_factory::ForkFactory;
use crate::contract_modules::uniswap_v2::data_collector::tax_checker::{
    get_tax, inject_tax_checker_code, insert_fake_approval,
};
use crate::contract_modules::uniswap_v2::types::{UniV2, UniV2Pool};
use crate::helpers::address;

use ethers::prelude::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::*;
use revm::db::{CacheDB, EmptyDB};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub async fn get_all_pairs(
    factorys: Vec<UniV2>,
    wss_provider: Arc<Provider<Ws>>,
) -> Option<Vec<UniV2Pool>> {
    let multi_progress_bar = MultiProgress::new();
    let current_block = wss_provider.get_block_number().await.unwrap();
    let cache_db: CacheDB<EmptyDB> = CacheDB::new(EmptyDB::default());
    let mut fork_factory = ForkFactory::new_sandbox_factory(wss_provider.clone(), cache_db, None);
    inject_tax_checker_code(&mut fork_factory);

    let mut pairs = Vec::new();

    for factory_data in factorys {
        let progress_bar = create_progress_bar_with_message(
            format!("Processing pools: {}", factory_data.factory),
            &multi_progress_bar,
        );

        let pairs_internal = get_all_pairs_via_batched_calls(
            factory_data.factory,
            wss_provider.clone(),
            progress_bar.clone(),
        )
        .await;

        progress_bar.reset();
        progress_bar.set_message("Getting tax".to_string());
        progress_bar.set_length(pairs_internal.len() as u64);

        let progress_bar_internal =
            create_progress_bar_with_message("Spawning ".to_owned(), &multi_progress_bar);

        progress_bar_internal.set_length(pairs_internal.len() as u64);

        let mut tasks_batch = Vec::new();
        let progress_bar_clone = progress_bar.clone();

        for pool in pairs_internal {
            insert_fake_approval(pool.token0, pool.address, &mut fork_factory);
            let sand_box = fork_factory.new_sandbox_fork();

            if pool.reserve0 < U256::from(1000000) || pool.reserve1 < U256::from(1000000) {
                progress_bar.inc(1);
                progress_bar_internal.inc(1);
                continue;
            }

            let factory_data_clone = factory_data.clone();
            let current_block_clone = current_block;
            let pool_clone = pool.clone();
            let progress_bar_clone = progress_bar_clone.clone();

            let task = tokio::task::spawn(async move {
                let (buy_tax, sell_tax) = match get_tax(
                    pool_clone.token0,
                    pool_clone.address,
                    sand_box,
                    current_block_clone,
                    factory_data_clone.fee,
                )
                .await
                {
                    Some(d) => d,
                    None => {
                        progress_bar_clone.inc(1);
                        return None;
                    }
                };

                if buy_tax > U256::from(9970) || sell_tax > U256::from(9970) {
                    progress_bar_clone.inc(1);
                    return None;
                }

                let mut cloned_pool = pool_clone.clone();
                cloned_pool.fees0 = buy_tax;
                cloned_pool.fees1 = sell_tax;

                progress_bar_clone.inc(1);
                Some(cloned_pool)
            });

            tasks_batch.push(task);
            progress_bar_internal.inc(1);

            if tasks_batch.len() >= 100 {
                for task in &mut tasks_batch {
                    if let Some(pair) = task.await.unwrap() {
                        pairs.push(pair);
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                tasks_batch.clear();
            }
        }

        // Handle any remaining tasks in the batch
        for task in tasks_batch {
            if let Some(pair) = task.await.unwrap() {
                pairs.push(pair);
            }
        }

        progress_bar.reset();
        progress_bar_internal.reset();
    }

    info!("Spawning complete");
    multi_progress_bar.clear().unwrap();

    Some(pairs)
}

pub async fn update_reserves(
    pairs: &mut Vec<UniV2Pool>,
    factories: Vec<UniV2>,
    wss_provider: Arc<Provider<Ws>>,
) {
    let multi_progress_bar = MultiProgress::new();
    let mut pairs_new: HashMap<H160, UniV2Pool> = HashMap::new();

    for factory_data in factories {
        let progress_bar = multi_progress_bar.add(ProgressBar::new(0));
        progress_bar.set_style(
            ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
                .expect("Error when setting progress bar style")
                .progress_chars("##-"),
        );

        progress_bar.set_message(format!("Getting all pools from: {}", factory_data.factory));
        let pairs_internal = get_all_pairs_via_batched_calls(
            factory_data.factory,
            wss_provider.clone(),
            progress_bar.clone(),
        )
        .await;

        progress_bar.finish_and_clear();
        pairs_new.extend(pairs_internal.into_iter().map(|pair| (pair.address, pair)));
    }

    let banned_addresses = [
        address("0xd46ba6d942050d489dbd938a2c909a5d5039a161"),
        address("0x83B04AF7a77C727273B7a582D6Fda65472FCB3f2"),
        address("0x9766d2e3f04AE13e8c2EB018eA51dC640d3f9f1F"),
        address("0x7E3d39398C9574e1B4f9510Fd37aa3a47d602cDD"),
    ];

    pairs.retain(|pair| {
        !banned_addresses.contains(&pair.token0)
            && !banned_addresses.contains(&pair.token1)
            && !banned_addresses.contains(&pair.address)
    });

    for pair in pairs {
        if let Some(new_pair) = pairs_new.get(&pair.address) {
            pair.reserve0 = new_pair.reserve0;
            pair.reserve1 = new_pair.reserve1;
        } else {
            panic!("Not supposed to happen");
        }
    }
}

fn create_progress_bar_with_message(
    message: String,
    multi_progress_bar: &MultiProgress,
) -> ProgressBar {
    let progress_bar = multi_progress_bar.add(ProgressBar::new(0));
    progress_bar.set_style(
        ProgressStyle::with_template("{msg} {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .expect("Error when setting progress bar style")
            .progress_chars("##-"),
    );
    progress_bar.set_message(message);
    progress_bar
}

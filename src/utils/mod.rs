use ethers::prelude::*;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Arc,
};

pub async fn get_state_diffs(
    client: &Arc<Provider<Ws>>,
    meats: &Vec<Transaction>,
    block_num: BlockNumber,
) -> Option<BTreeMap<Address, AccountDiff>> {
    // add statediff trace to each transaction
    let req = meats
        .iter()
        .map(|tx| (tx, vec![TraceType::StateDiff]))
        .collect();

    let block_traces = match client.trace_call_many(req, Some(block_num)).await {
        Ok(x) => x,
        Err(_) => {
            return None;
        }
    };

    let mut merged_state_diffs = BTreeMap::new();

    block_traces
        .into_iter()
        .flat_map(|bt| bt.state_diff.map(|sd| sd.0.into_iter()))
        .flatten()
        .for_each(|(address, account_diff)| {
            match merged_state_diffs.entry(address) {
                Entry::Vacant(entry) => {
                    entry.insert(account_diff);
                }
                Entry::Occupied(_) => {
                    // Do nothing if the key already exists
                    // we only care abt the starting state
                }
            }
        });

    Some(merged_state_diffs)
}

pub async fn get_logs(
    client: &Arc<Provider<Ws>>,
    tx: &Transaction,
    block_num: BlockNumber,
) -> Option<Vec<CallLogFrame>> {
    // add statediff trace to each transaction

    let mut trace_ops = GethDebugTracingCallOptions::default();
    let mut call_config = CallConfig::default();
    call_config.with_log = Some(true);

    trace_ops.tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer));
    trace_ops.tracing_options.tracer_config = Some(
        GethDebugTracerConfig::BuiltInTracer(
            GethDebugBuiltInTracerConfig::CallTracer(
                call_config
            )
        )
    );
    let block_num = Some(BlockId::Number(block_num));

    let call_frame = match client.debug_trace_call(tx, block_num, trace_ops).await {
        Ok(d) => {
            match d {
                GethTrace::Known(d) => {
                    match d {
                        GethTraceFrame::CallTracer(d) => d,
                        _ => return None
                    }
                }
                _ => return None
            }
        },
        Err(_) => {
            return None
        }
    }; 

    let mut logs = Vec::new();
    extract_logs(&call_frame, &mut logs);
    
    
    Some(logs)
}

fn extract_logs(call_frame: &CallFrame, logs: &mut Vec<CallLogFrame>) {
    if let Some(ref logs_vec) = call_frame.logs {
        logs.extend(logs_vec.iter().cloned());
    }

    if let Some(ref calls_vec) = call_frame.calls {
        for call in calls_vec {
            extract_logs(call, logs);
        }
    }
}


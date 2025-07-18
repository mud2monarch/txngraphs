mod types;
use anyhow::Result;
use polars::prelude::*;
use std::{
    sync::Arc,
    collections::{HashMap, HashSet},
    str::FromStr,
};
use types::*;
use alloy_primitives::Address;
use tracing_subscriber;
use tracing::info;
use petgraph::graph::NodeIndex;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "0xa23c6c374b372b6964ef7c1c00916e2b4f5a3629")]
    root_address: String,
    #[arg(short, long, default_value = "10")]
    max_depth: usize,
    #[arg(short, long, default_value = "8610738")]
    block_start: u64,
    #[arg(short, long, default_value = "8625670")]
    block_end: u64,
}

// Dune CSV example
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let root_address: Address = Address::from_str(&args.root_address)?;
    let max_depth: usize = args.max_depth;
    let block_start: u64 = args.block_start;
    let block_end: u64 = args.block_end;
    // Define schema overrides; can't use default i64 for token amounts
    let schema_changes = Schema::from_iter(vec![
        Field::new("token_sold_amount_raw".into(), DataType::String),
        Field::new("token_bought_amount_raw".into(), DataType::String),
        Field::new("block_number".into(), DataType::UInt64),
    ]);

    // Load data from CSV
    let trades = CsvReadOptions::default()
        .with_has_header(true)
        .with_schema_overwrite(Some(Arc::new(schema_changes)))
        .try_into_reader_with_file_path(Some("data/pi_token_trades_dune.csv".into()))?
        .finish()?;

    // Create a new DuneDexTradesDataSource from the loaded data
    let data_source = DuneDexTradesDataSource::new(trades);

    let transfers = data_source.get_transfers(
        &Address::from_str("0xa23c6c374b372b6964ef7c1c00916e2b4f5a3629")?,
        8610738,
        8625670
    )?;

    info!("transfers.length: {}", transfers.len());

    // start the search

    let mut graph = TransferGraph::new(); 
    // stack keeps track of addresses + depth of my BFS
    let mut stack = NodeStack::new();
    // addr_idx_map maps addresses to their node index in the graph so that I can insert edges
    let mut addr_idx_map: HashMap<Address, NodeIndex> = HashMap::new();
    // visited keeps track of addresses that have been visited
    let mut visited: HashSet<Address> = HashSet::new();

    let root_idx = graph.add_node(root_address.clone());
    addr_idx_map.insert(root_address.clone(), root_idx);
    stack.push_back((root_address.clone(), 0));
    visited.insert(root_address.clone());

    while let Some((curr_addr, depth)) = stack.pop_front() {
        if depth >= max_depth {
            continue;
        }

        for transfer in data_source.get_transfers(&curr_addr, block_start, block_end)? {
            let from = transfer.from_address.clone();
            let to = transfer.to_address.clone();

            
        }
    }
    
    Ok(())
}

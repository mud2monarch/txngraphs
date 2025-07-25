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
    #[arg(short = 'd', long, default_value = "10")]
    max_depth: usize,
    #[arg(short = 's', long, default_value = "8610738")]
    block_start: u64,
    #[arg(short = 'e', long, default_value = "8625670")]
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
        if depth > max_depth {
            continue;
        }

        for transfer in data_source.get_transfers(&curr_addr, block_start, block_end)? {
            let from = transfer.from_address.clone();
            let to = transfer.to_address.clone();

            // Check our addr_idx_map to see if we've already seen this address
            // If we have seen this address (i.e., .entry() returns an Entry::Occupied), `.entry().or_insert_with()` will return the existing node index
            // If we haven't seen this address (i.e., .entry() returns an Entry::Vacant), `.or_insert_with()` will
            // (1) add the address as a node in the graph (which returns a NodeIndex),
            // (2) add the NodeIndex to add_idx_map,
            // (3) return a mutable reference to a NodeIndex
            // (4) dereference the mutable reference to get the NodeIndex (required, at least, to avoid maintaining a mutable reference to the NodeIndex in addr_idx_map)
            let from_idx = *addr_idx_map.entry(from.clone()).or_insert_with(|| graph.add_node(from.clone()));
            let to_idx = *addr_idx_map.entry(to.clone()).or_insert_with(|| graph.add_node(to.clone()));

            graph.add_edge(
                from_idx,
                to_idx,
                TransferEdge {
                    amount: transfer.amount,
                    tx_hash: transfer.tx_hash,
                    block_number: transfer.block_number,
                    token: transfer.token,
                },
            );

            // If we haven't visited this address, add it to the stack with depth + 1
            if !visited.contains(&from) {
                stack.push_back((from, depth+1))
            }
        }
    }

    println!("graph: {:?}", graph);
    
    Ok(())
}   

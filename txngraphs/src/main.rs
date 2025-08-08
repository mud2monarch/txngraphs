mod data_sources;
mod traversal;
mod types;
use alloy_primitives::Address;
use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use petgraph::graph::NodeIndex;
use polars::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use tracing::info;
use tracing_subscriber;
use txngraphs::{data_sources::*, reth_source::*, traversal::*, types::*};

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "0x3bc1588f8D987C9ED07B3796AA50ccBF10514326"
    )]
    root_address: String,
    #[arg(
        long,
        default_value = "0x622b6330f226bf08427dcad49c9ea9694604bf2d,0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        value_delimiter = ','
    )]
    token_address: Vec<String>,
    #[arg(short = 'd', long, default_value = "10")]
    max_depth: usize,
    #[arg(short = 's', long, default_value = "20521409")]
    block_start: u64,
    #[arg(short = 'e', long, default_value = "20521410")]
    block_end: u64,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting txngraphs");
    let args = Args::parse();
    let root_address: Address = Address::from_str(&args.root_address)?;
    info!("Root address: {}", root_address);
    let max_depth: usize = args.max_depth;
    let block_start: u64 = args.block_start;
    let block_end: u64 = args.block_end;
    let token_addresses: Vec<Address> = args
        .token_address
        .iter()
        .map(|addr| Address::from_str(addr))
        .collect::<Result<Vec<Address>, _>>()?;
    info!("Token addresses: {:?}", token_addresses);

    let db_path = String::from("/home/gyges/.local/share/reth");

    info!("Initializing RethTransferDataSource");
    let reth_source = RethTransferDataSource::new(db_path);
    info!("Building transfer graph");
    let graph = build_transfer_graph(
        &reth_source,
        root_address,
        block_start,
        block_end,
        &token_addresses,
        max_depth,
    )?;

    info!("Graph built successfully");
    info!(
        "Graph has {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    let output = export_graph_to_dot(&graph);
    info!("Graph exported to dot file: {}", output);

    Ok(())
}

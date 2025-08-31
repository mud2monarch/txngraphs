use alloy_primitives::Address;
use anyhow::Result;
use clap::Parser;
use std::str::FromStr;
use tracing::{info, warn};
use tracing_subscriber;
use txngraphs::{data_sources::*, graph_utils::*, reth_source::*, summary::*, traversal::*};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    root_address: String,
    #[arg(long, value_delimiter = ',')]
    token_address: Vec<String>,
    #[arg(short = 'd', long)]
    max_depth: usize,
    #[arg(short = 's', long)]
    block_start: u64,
    #[arg(short = 'e', long)]
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
    if block_end < block_start {
        warn!("Block end is less than block start. Please check your input.")
    }
    let token_addresses: Vec<Address> = args
        .token_address
        .iter()
        .map(|addr| Address::from_str(addr))
        .collect::<Result<Vec<Address>, _>>()?;
    info!("Token addresses: {:?}", token_addresses);
    info!("Have {} blocks to process.", block_end - block_start);

    let db_path = String::from("/Users/zach.wong/Documents/unichain/unichain");

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

    info!("{:?}", graph);

    let summary: TransferSummary =
        TransferSummary::from_transfer_graph(&graph).with_summary_table();

    info!("{}", summary);

    Ok(())
}

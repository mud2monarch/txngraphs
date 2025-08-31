mod data_sources;
mod traversal;
mod types;
use alloy_primitives::Address;
use anyhow::Result;
use clap::Parser;
use std::str::FromStr;
use tracing::{info, warn};
use tracing_subscriber;
use txngraphs::{data_sources::*, graph_utils::*, reth_source::*, summary::*, traversal::*};

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "0x284F11109359a7e1306C3e447ef14D38400063FF"
    )]
    root_address: String,
    #[arg(
        long,
        default_value = "0x4200000000000000000000000000000000000006",
        value_delimiter = ','
    )]
    token_address: Vec<String>,
    #[arg(short = 'd', long, default_value = "1")]
    max_depth: usize,
    #[arg(short = 's', long, default_value = "8610738")]
    block_start: u64,
    #[arg(short = 'e', long, default_value = "8630738")]
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

    let summary: TransferSummary =
        TransferSummary::from_transfer_graph(&graph).with_summary_table();

    print!("{}", summary);

    // let closed_loops = find_closed_loops(&graph);
    // for x in closed_loops {
    //     let summary2 =
    //         TransferGraphSummary::aggregate_transfers(&x).sort_by_addr_then_transfer_count(true);

    //     print!("Closed loops includes: \n {}", summary2);
    // }

    Ok(())
}

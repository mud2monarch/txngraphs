mod data_sources;
mod traversal;
mod types;
use txngraphs::{data_sources::*, traversal::*, types::*, reth_source::*};
use alloy_primitives::Address;
use anyhow::Result;
use clap::Parser;
use petgraph::graph::NodeIndex;
use polars::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
    path::PathBuf,
};
use tracing::info;
use tracing_subscriber;
use dotenv::dotenv;

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "0xa23c6c374b372b6964ef7c1c00916e2b4f5a3629"
    )]
    root_address: String,
    #[arg(long, default_value = "0x20f17D48646D57764334B6606d85518680D4e276")]
    token_address: String,
    #[arg(short = 'd', long, default_value = "10")]
    max_depth: usize,
    #[arg(short = 's', long, default_value = "8624945")]
    block_start: u64,
    #[arg(short = 'e', long, default_value = "8624974")]
    block_end: u64,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting txngraphs");
    let args = Args::parse();
    let root_address: Address = Address::from_str(&args.root_address)?;
    let max_depth: usize = args.max_depth;
    let block_start: u64 = args.block_start;
    let block_end: u64 = args.block_end;
    let token_address: Address = Address::from_str(&args.token_address)?;

    let db_path = String::from("/Users/zach.wong/Documents/unichain/unichain");

    info!("Initializing RethTransferDataSource");
    let reth_source = RethTransferDataSource::new(db_path);
    info!("Building transfer graph");
    let graph = build_transfer_graph(&reth_source, root_address, block_start, block_end, &token_address, max_depth)?;

    info!("Graph built successfully");
    info!("Graph has {} nodes and {} edges", graph.node_count(), graph.edge_count());

    Ok(())
}

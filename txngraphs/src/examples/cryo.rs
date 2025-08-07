// TODO: Cryo portion is not working; too much filtering maybe? Getting no results.

mod data_sources;
mod traversal;
mod types;
use crate::{data_sources::*, traversal::*, types::*};
use alloy_primitives::Address;
use anyhow::Result;
use clap::Parser;
use petgraph::graph::NodeIndex;
use polars::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
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
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let root_address: Address = Address::from_str(&args.root_address)?;
    let max_depth: usize = args.max_depth;
    let block_start: u64 = args.block_start;
    let block_end: u64 = args.block_end;
    let token_address: Address = Address::from_str(&args.token_address)?;

    // Get RPC URL from environment
    let unichain_rpc_url = std::env::var("UNICHAIN_RPC_URL")
        .expect("UNICHAIN_RPC_URL environment variable not found. Please set it in your .env file");
    
    info!("Testing Cryo connector with:");
    info!("  Root address: {}", root_address);
    info!("  Block range: {} to {}", block_start, block_end);
    info!("  Chain: Unichain (1301)");

    // Use Unichain chain ID (1301)
    let cryo_source = CryoTransferDataSource::new(1301, unichain_rpc_url)?;
    
    let transfers = cryo_source.get_transfers(&root_address, &token_address, &8624945, &8624974)?;
    
    info!("Retrieved {} transfers", transfers.len());
    for (i, transfer) in transfers.iter().take(5).enumerate() {
        info!("Transfer {}: {:?}", i, transfer);
    }

    Ok(())
}

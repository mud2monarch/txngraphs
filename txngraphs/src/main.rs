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

#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "0xa23c6c374b372b6964ef7c1c00916e2b4f5a3629"
    )]
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

    let transfers = data_source.get_transfers(&root_address, &block_start, &block_end)?;

    let graph = build_transfer_graph(
        &data_source,
        root_address,
        block_start,
        block_end,
        max_depth,
    )?;

    println!("graph: {:?}", graph);

    Ok(())
}

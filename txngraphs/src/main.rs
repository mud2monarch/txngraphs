mod types;
use anyhow::Result;
use polars::prelude::*;
use std::sync::Arc;
use types::*;
use alloy_primitives::Address;
use std::str::FromStr;
use tracing_subscriber;
use tracing::info;

// Dune CSV example
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
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

    Ok(())
}

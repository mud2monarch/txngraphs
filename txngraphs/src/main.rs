mod types;
use polars::prelude::*;
use types::*;
use anyhow::Result;
use std::sync::Arc;

// Dune CSV example
fn main() -> Result<()> {

    // Define schema overrides; can't use default i64 for token amounts
    let schema_changes = Schema::from_iter(vec![
        Field::new("token_sold_amount_raw".into(), DataType::String),
        Field::new("token_bought_amount_raw".into(), DataType::String),
    ]);

    // Load data from CSV
    let trades = CsvReadOptions::default()
        .with_has_header(true)
        .with_schema_overwrite(Some(Arc::new(schema_changes)))
        .try_into_reader_with_file_path(Some("data/pi_token_trades_dune.csv".into()))?
        .finish()?;
    
    // Create a new DuneDexTradesDataSource from the loaded data
    let data_source = DuneDexTradesDataSource::new(trades);
    Ok(())
}
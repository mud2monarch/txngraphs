use crate::types::*;
use alloy_primitives::{
    Address, B256,
    aliases::{BlockNumber, TxHash, U256},
};
use anyhow::{Context, Result};
use petgraph::{
    Directed,
    graph::{Graph, NodeIndex},
};
use polars::prelude::*;
use std::str::FromStr;
use std::{collections::VecDeque, fmt::Debug, fmt::Display};
use tracing::info;

/// TransferDataSource
///
/// A generic trait across different data sources.
///
pub trait TransferDataSource {
    fn get_transfers(
        &self,
        address: &Address,
        block_start: &BlockNumber,
        block_end: &BlockNumber,
    ) -> anyhow::Result<Vec<Transfer>>;
}

/// DuneDexTradesDataSource
///
/// An opinionated implementation of a data source based on Dune's dex.trades table.
///
/// A DuneDexTradesDataSource wraps a polars DataFrame.
/// TODO: Implement Option<amount_usd>
///
/// Each row of the DataFrame should be a single DEX trade, and it should have the following columns:
/// - `tx_from`
/// - `tx_to`
/// - `tx_hash`
/// - `block_number`
/// - `token_sold_address`
/// - `token_sold_amount_raw`
/// - `token_bought_address`
/// - `token_bought_amount_raw`
/// - `amount_usd`
///
/// For reference, see dune.com/queries/5488226.
///
pub struct DuneDexTradesDataSource {
    pub dex_trades: polars::prelude::DataFrame,
}

impl DuneDexTradesDataSource {
    pub fn new(dex_trades: polars::prelude::DataFrame) -> Self {
        Self { dex_trades }
    }
}

impl TransferDataSource for DuneDexTradesDataSource {
    fn get_transfers(
        &self,
        address: &Address,
        block_start: &BlockNumber,
        block_end: &BlockNumber,
    ) -> anyhow::Result<Vec<Transfer>> {
        let addr_hex = format!("{address:#x}");

        let filtered_trades = self
            .dex_trades
            .clone()
            .lazy()
            .filter(
                col("tx_from")
                    .eq(lit(addr_hex))
                    .and(col("block_number").gt_eq(lit(*block_start)))
                    .and(col("block_number").lt_eq(lit(*block_end))),
            )
            .collect()?;

        info!("filtered_trades.height(): {}", filtered_trades.height());

        let mut transfers = Vec::with_capacity(filtered_trades.height() as usize);

        let col_tx_hash = filtered_trades.column("tx_hash")?.str()?;
        let col_block_number = filtered_trades.column("block_number")?.u64()?;
        let col_from_address = filtered_trades.column("tx_from")?.str()?;
        let col_to_address = filtered_trades.column("tx_to")?.str()?;
        let col_token = filtered_trades.column("token_sold_address")?.str()?;
        let col_amount = filtered_trades.column("token_sold_amount_raw")?.str()?;

        for row in 0..filtered_trades.height() {
            let trade =
                Transfer::new(
                    B256::from_str(
                        col_tx_hash
                            .get(row)
                            .with_context(|| format!("Failed to get tx_hash for {}", row))?,
                    )?,
                    col_block_number
                        .get(row)
                        .with_context(|| format!("Failed to get block_number for {}", row))?,
                    Address::from_str(
                        col_from_address
                            .get(row)
                            .with_context(|| format!("Failed to get tx_from for {}", row))?,
                    )?,
                    Address::from_str(
                        col_to_address
                            .get(row)
                            .with_context(|| format!("Failed to get tx_to for {}", row))?,
                    )?,
                    Address::from_str(col_token.get(row).with_context(|| {
                        format!("Failed to get token_sold_address for {}", row)
                    })?)?,
                    U256::from_str(col_amount.get(row).with_context(|| {
                        format!("Failed to get token_sold_amount_raw for {}", row)
                    })?)?,
                );
            transfers.push(trade);
        }

        Ok(transfers)
    }
}

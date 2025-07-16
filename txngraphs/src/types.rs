use petgraph::graphmap::DiGraphMap;
use alloy_primitives::{
    Address,
    aliases::{
        TxHash,
        BlockNumber,
        U256,
    }
};
use polars::prelude::*;

///
/// TransferGraph
///
/// The graph is a directed graph where the nodes are addresses and the edges are transfers with certain characteristics.
/// For edges, see `TransferEdge`.
pub type TransferGraph = DiGraphMap<String, TransferEdge>;

///
/// TransferEdge
///
/// The edge is a transfer with certain characteristics.
///
pub struct TransferEdge {
    pub amount: f64,
    pub tx_hash: TxHash,
    pub block_number: BlockNumber,
    pub token: Address,
}

///
/// Transfer
///
/// A transfer is a single token transfer between two addresses.
///
pub struct Transfer {
    pub tx_hash: TxHash,
    pub block_number: BlockNumber,
    pub from_address: Address,
    pub to_address: Address,
    pub token: Address,
    pub amount: U256,
}

///
/// TransferDataSource
///
/// A generic trait across different data sources.
///
pub trait TransferDataSource {
    fn get_transfers_from(
        &self,
        address: &Address,
        block_start: BlockNumber,
        block_end: BlockNumber,
    ) -> Vec<Transfer>;
}

///
/// DuneDexTradesDataSource
///
/// An opinionated implementation of a data source based on Dune's dex.trades table.
/// 
/// A DuneDexTradesDataSource wraps a polars DataFrame.
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
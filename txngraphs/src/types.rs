use alloy_primitives::{
    Address, B256,
    aliases::{BlockNumber, TxHash, U256},
};
use anyhow::{Context, Result};
use petgraph::{
    Directed,
    graph::{Graph, NodeIndex},
};
use std::{collections::VecDeque, fmt::Debug, fmt::Display};
use tracing::info;

///
/// TransferGraph
///
/// The graph is a directed graph where the nodes are addresses, as an &str, and the edges are transfers with certain characteristics.
/// For edges, see `TransferEdge`.
pub type TransferGraph = Graph<Address, TransferEdge, Directed>;

///
/// NodeStack
///
/// A stack of nodes to visit.
///
/// The first element is the address, and the second element is the depth.
///
pub type NodeStack = VecDeque<(Address, usize)>;

///
/// TransferEdge
///
/// The edge is a transfer with certain characteristics.
///
pub struct TransferEdge {
    pub amount: U256,
    pub tx_hash: TxHash,
    pub block_number: BlockNumber,
    pub token: Address,
}

impl Debug for TransferEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TransferEdge {{ amount: {}, tx_hash: {}, block_number: {}, token: {} }}",
            self.amount, self.tx_hash, self.block_number, self.token
        )
    }
}

impl Display for TransferEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TransferEdge {{ amount: {}, tx_hash: {}, block_number: {}, token: {} }}",
            self.amount, self.tx_hash, self.block_number, self.token
        )
    }
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

impl Transfer {
    pub fn new(
        tx_hash: TxHash,
        block_number: BlockNumber,
        from_address: Address,
        to_address: Address,
        token: Address,
        amount: U256,
    ) -> Self {
        Self {
            tx_hash,
            block_number,
            from_address,
            to_address,
            token,
            amount,
        }
    }
}

impl Debug for Transfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transfer {{ tx_hash: {}, block_number: {}, from_address: {}, to_address: {}, token: {}, amount: {} }}",
            self.tx_hash,
            self.block_number,
            self.from_address,
            self.to_address,
            self.token,
            self.amount
        )
    }
}

impl Display for Transfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transfer {{ tx_hash: {}, block_number: {}, from_address: {}, to_address: {}, token: {}, amount: {} }}",
            self.tx_hash,
            self.block_number,
            self.from_address,
            self.to_address,
            self.token,
            self.amount
        )
    }
}

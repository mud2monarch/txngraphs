use crate::{data_sources::*, types::*};
use alloy_primitives::{Address, BlockNumber};
use anyhow::Result;
use petgraph::graph::NodeIndex;
use std::{
    collections::{HashMap, HashSet},
};

pub fn build_transfer_graph<D: TransferDataSource>(
    data_source: &D,
    root_address: Address,
    block_start: BlockNumber,
    block_end: BlockNumber,
    max_depth: usize,
) -> Result<TransferGraph> {
    let transfers = data_source.get_transfers(&root_address, &block_start, &block_end)?;

    // start the search

    let mut graph = TransferGraph::new();
    // stack keeps track of addresses + depth of my BFS
    let mut stack = NodeStack::new();
    // addr_idx_map maps addresses to their node index in the graph so that I can insert edges
    let mut addr_idx_map: HashMap<Address, NodeIndex> = HashMap::new();
    // visited keeps track of addresses that have been visited
    let mut visited: HashSet<Address> = HashSet::new();

    let root_idx = graph.add_node(root_address.clone());
    addr_idx_map.insert(root_address.clone(), root_idx);
    stack.push_back((root_address.clone(), 0));
    visited.insert(root_address.clone());

    while let Some((curr_addr, depth)) = stack.pop_front() {
        if depth > max_depth {
            continue;
        }

        for transfer in data_source.get_transfers(&curr_addr, &block_start, &block_end)? {
            let from = transfer.from_address.clone();
            let to = transfer.to_address.clone();

            // Check our addr_idx_map to see if we've already seen this address
            // If we have seen this address (i.e., .entry() returns an Entry::Occupied), `.entry().or_insert_with()` will return the existing node index
            // If we haven't seen this address (i.e., .entry() returns an Entry::Vacant), `.or_insert_with()` will
            // (1) add the address as a node in the graph (which returns a NodeIndex),
            // (2) add the NodeIndex to add_idx_map,
            // (3) return a mutable reference to a NodeIndex
            // (4) dereference the mutable reference to get the NodeIndex (required, at least, to avoid maintaining a mutable reference to the NodeIndex in addr_idx_map)
            let from_idx = *addr_idx_map
                .entry(from.clone())
                .or_insert_with(|| graph.add_node(from.clone()));
            let to_idx = *addr_idx_map
                .entry(to.clone())
                .or_insert_with(|| graph.add_node(to.clone()));

            graph.add_edge(
                from_idx,
                to_idx,
                TransferEdge {
                    amount: transfer.amount,
                    tx_hash: transfer.tx_hash,
                    block_number: transfer.block_number,
                    token: transfer.token,
                },
            );

            // If we haven't visited this address, add it to the stack with depth + 1
            if !visited.contains(&from) {
                stack.push_back((from, depth + 1))
            }
        }
    }

    Ok(graph)
}

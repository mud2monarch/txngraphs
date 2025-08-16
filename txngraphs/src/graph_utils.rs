use crate::types::{Transfer, TransferGraph};
use alloy_primitives::{
    Address, B256,
    aliases::{BlockNumber, TxHash, U256},
};
use anyhow::{Context, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec, parse,
    printer::PrinterContext,
};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::{fmt::Display, fmt::Write, fs};

// Oops I/Claude didn't realize petgraph had DOT exports already

/// Write TransferGraph into a DOT string for visualization
///
/// Useful for small to medium sized graphs with `https://dreampuf.github.io/GraphvizOnline/?engine=dot`
pub fn write_graph_to_dot(graph: &TransferGraph) -> String {
    let mut dot = String::new();
    writeln!(dot, "digraph TransferGraph {{").unwrap();
    writeln!(dot, "  node [shape=ellipse];").unwrap();
    writeln!(dot, "  edge [dir=forward];").unwrap();
    writeln!(dot).unwrap();

    // Add nodes (addresses)
    for node_idx in graph.node_indices() {
        let address = graph[node_idx];
        writeln!(dot, "  \"{}\" [label=\"{:.36}...\"];", address, address).unwrap();
    }

    writeln!(dot).unwrap();

    // Add edges (transfers)
    for edge_idx in graph.edge_indices() {
        let (from_idx, to_idx) = graph.edge_endpoints(edge_idx).unwrap();
        let from_addr = graph[from_idx];
        let to_addr = graph[to_idx];
        let transfer = &graph[edge_idx];

        let amount_str = transfer.amount.to_string();

        writeln!(
            dot,
            "  \"{}\" -> \"{}\" [label=\"{}\\nBlock {}\" tooltip=\"Tx: {}\"];",
            from_addr, to_addr, amount_str, transfer.block_number, transfer.tx_hash
        )
        .unwrap();
    }

    writeln!(dot, "}}").unwrap();
    dot
}

/// Call write_graph_to_dot() and copy result to clipboard
///
/// Probably only use this for pretty small graphs.
pub fn copy_graph_to_clipboard(graph: &TransferGraph) -> Result<()> {
    let dot_string = write_graph_to_dot(graph);
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to initialize clipboard context")?;
    ctx.set_contents(dot_string)
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to copy DOT string to clipboard")?;
    Ok(())
}

/// Export TransferGraph to SVG file saved in results directory
///
/// Useful for very large graphs if web-based viewing is clunky.
///
/// Requires you to have graphviz installed. `brew install graphviz`
pub fn save_graph_as_svg(graph: &TransferGraph, filename: &str) -> Result<String> {
    let mut dot = String::new();
    writeln!(dot, "digraph TransferGraph {{").unwrap();
    writeln!(dot, "  node [shape=ellipse];").unwrap();
    writeln!(dot, "  edge [dir=forward];").unwrap();
    writeln!(dot).unwrap();

    // Add nodes (addresses)
    for node_idx in graph.node_indices() {
        let address = graph[node_idx];
        writeln!(dot, "  \"{}\" [label=\"{:.36}...\"];", address, address).unwrap();
    }

    writeln!(dot).unwrap();

    // Add edges (transfers)
    for edge_idx in graph.edge_indices() {
        let (from_idx, to_idx) = graph.edge_endpoints(edge_idx).unwrap();
        let from_addr = graph[from_idx];
        let to_addr = graph[to_idx];
        let transfer = &graph[edge_idx];

        let amount_str = transfer.amount.to_string();

        writeln!(
            dot,
            "  \"{}\" -> \"{}\" [label=\"{}\\nBlock {}\" tooltip=\"Tx: {}\"];",
            from_addr, to_addr, amount_str, transfer.block_number, transfer.tx_hash
        )
        .unwrap();
    }

    writeln!(dot, "}}").unwrap();

    // Parse DOT string and convert to SVG
    let graph_ast = parse(&dot).expect("Couldn't parse SVG");

    let svg_data = exec(
        graph_ast,
        &mut PrinterContext::default(),
        vec![CommandArg::Format(Format::Svg)],
    )
    .context("Failed to generate SVG from graph")?;

    // Save to results directory
    let output_path = format!("results/{}", filename);
    fs::write(&output_path, &svg_data)
        .with_context(|| format!("Failed to write SVG to {}", output_path))?;
    Ok(output_path)
}

// TODO: Add support for sum_amount
pub struct AggregatedTransfer {
    pub from: Address,
    pub to: Address,
    pub no_transfers: usize,
}

pub struct TransferGraphSummary {
    pub aggregated_transfers: Vec<AggregatedTransfer>,
}

impl TransferGraphSummary {
    pub fn new(aggregated_transfers: Vec<AggregatedTransfer>) -> Self {
        Self {
            aggregated_transfers,
        }
    }

    // TODO: Nice to have min and max block too

    pub fn aggregate_transfers(graph: &TransferGraph) -> Self {
        let mut acc: HashMap<(Address, Address), usize> = HashMap::new();

        for edge in graph.edge_references() {
            let key = (edge.source(), edge.target());
            *acc.entry((graph[key.0], graph[key.1])).or_insert(0) += 1;
        }

        let mut aggregated_transfers = Vec::new();

        for ((from, to), count) in acc {
            aggregated_transfers.push(AggregatedTransfer {
                from: from,
                to: to,
                no_transfers: count,
            });
        }

        TransferGraphSummary {
            aggregated_transfers: aggregated_transfers,
        }
    }

    pub fn sort_by_transfer_count(mut self, descending: bool) -> Self {
        self.aggregated_transfers.sort_by(|a, b| {
            if descending {
                b.no_transfers.cmp(&a.no_transfers)
            } else {
                a.no_transfers.cmp(&b.no_transfers)
            }
        });
        self
    }

    pub fn sort_by_addr_then_transfer_count(mut self, descending: bool) -> Self {
        self.aggregated_transfers.sort_by(|a, b| {
            a.from.cmp(&b.from).then_with(|| {
                if descending {
                    b.no_transfers.cmp(&a.no_transfers)
                } else {
                    a.no_transfers.cmp(&b.no_transfers)
                }
            })
        });

        self
    }
}

impl Display for TransferGraphSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for transfer in &self.aggregated_transfers {
            write!(
                f,
                "{:.36} -> {:.36} transferred {} times.\n",
                transfer.from, transfer.to, transfer.no_transfers
            )?
        }
        Ok(())
    }
}

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
use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, algo::tarjan_scc};
use std::clone::Clone;
use std::collections::HashMap;
use std::{
    fmt::{Debug, Display, Write},
    fs,
};

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

// TODO: move this to impl TransferGraph
pub fn find_closed_loops(graph: &TransferGraph) -> Vec<TransferGraph> {
    let mut closed_loops = Vec::new();

    let mut tarjan_graph = tarjan_scc(&graph);
    tarjan_graph.retain(|scc| scc.len() > 1);

    for scc in tarjan_graph {
        let mut index_mapping = HashMap::new();
        let mut loop_graph = TransferGraph::new();

        for node in &scc {
            let address = graph[*node];
            let new_idx = loop_graph.add_node(address);
            index_mapping.insert(*node, new_idx);
        }

        for edge in graph.edge_references() {
            let (source, target) = (edge.source(), edge.target());
            if scc.contains(&source) && scc.contains(&target) {
                let new_source_idx = index_mapping[&source];
                let new_target_idx = index_mapping[&target];
                loop_graph.add_edge(new_source_idx, new_target_idx, edge.weight().clone());
            }
        }

        closed_loops.push(loop_graph);
    }

    closed_loops
}

// TODO: feat. add support for sum_amount
#[derive(Debug)]
pub struct SummaryEdge {
    pub no_transfers: usize,
}

impl SummaryEdge {
    pub fn new(no_transfers: usize) -> Self {
        Self {
            no_transfers: no_transfers,
        }
    }
}

impl Display for SummaryEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.no_transfers)
    }
}

// TODO: Add support for sum_amount
#[derive(Debug, Clone)]
pub struct AggregatedTransfer {
    pub from: Address,
    pub to: Address,
    pub no_transfers: usize,
}

impl Display for AggregatedTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.36} -> {:.36} transferred {} times.\n",
            self.from, self.to, self.no_transfers
        );
        Ok(())
    }
}

/// TransferSummary
///
/// A TransferSummary is primarily a graph that aggregates many TransferEdges between nodes.
///
/// It optionally has a tabular representation, a Vec<AggregatedTransfer>, but given this is
/// a graph library it's not required to be created or used.
pub struct TransferSummary {
    pub summary_graph: petgraph::Graph<Address, SummaryEdge, Directed>,
    pub summary_table: Option<Vec<AggregatedTransfer>>,
}

// TODO: Add filtering for >1 transfers
impl TransferSummary {
    pub fn new() -> Self {
        let graph = Graph::<Address, SummaryEdge, Directed>::new();
        Self {
            summary_graph: graph,
            summary_table: None,
        }
    }

    pub fn from_transfer_graph(graph: &TransferGraph) -> Self {
        // Accumulate/count all transfers from the transfer graph
        let mut acc: HashMap<(Address, Address), usize> = HashMap::new();

        for edge in graph.edge_references() {
            let key = (edge.source(), edge.target());
            *acc.entry((graph[key.0], graph[key.1])).or_insert(0) += 1;
        }

        // Add the nodes and edges to the summary graph
        let mut summary_graph = Graph::<Address, SummaryEdge, Directed>::new();
        let mut node_map = HashMap::<Address, NodeIndex>::new();

        for ((from, to), count) in acc {
            let from_index = *node_map
                .entry(from)
                .or_insert_with(|| summary_graph.add_node(from));
            let to_index = *node_map
                .entry(to)
                .or_insert_with(|| summary_graph.add_node(to));

            summary_graph.add_edge(
                from_index,
                to_index,
                SummaryEdge {
                    no_transfers: count,
                },
            );
        }

        TransferSummary {
            summary_graph: summary_graph,
            summary_table: None,
        }
    }

    pub fn with_summary_table(self) -> Self {
        let mut aggregated_transfers = Vec::new();

        self.summary_graph
            .edge_references()
            .into_iter()
            .for_each(|edge| {
                let from = self.summary_graph[edge.source()];
                let to = self.summary_graph[edge.target()];
                let no_transfers = edge.weight().no_transfers;
                aggregated_transfers.push(AggregatedTransfer {
                    from,
                    to,
                    no_transfers,
                });
            });

        Self {
            summary_graph: self.summary_graph,
            summary_table: Some(aggregated_transfers),
        }
    }

    pub fn has_summary_table(&self) -> bool {
        self.summary_table.is_some()
    }
}

impl Display for TransferSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(table) = &self.summary_table {
            let mut sorted_table = table.clone();

            sorted_table.sort_by(|a, b| {
                a.from
                    .cmp(&b.from)
                    .then_with(|| b.no_transfers.cmp(&a.no_transfers))
            });

            for transfer in sorted_table {
                writeln!(
                    f,
                    "{:.36} -> {:.36} for {} transfers",
                    transfer.from, transfer.to, transfer.no_transfers
                )?;
            }
        } else {
            writeln!(f, "Transfer Summary:")?;
            for edge in self.summary_graph.edge_references() {
                let from = self.summary_graph[edge.source()];
                let to = self.summary_graph[edge.target()];
                let no_transfers = edge.weight().no_transfers;
                writeln!(
                    f,
                    "{:.36} -> {:.36} for {} transfers",
                    from, to, no_transfers
                )?;
            }
        }
        Ok(())
    }
}

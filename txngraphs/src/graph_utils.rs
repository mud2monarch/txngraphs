use crate::types::TransferGraph;
use anyhow::{Context, Result};
use copypasta::{ClipboardContext, ClipboardProvider};
use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec, parse,
    printer::PrinterContext,
};
use petgraph::algo::tarjan_scc;
use petgraph::visit::EdgeRef;
use std::clone::Clone;
use std::collections::HashMap;
use std::{fmt::Write, fs};

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

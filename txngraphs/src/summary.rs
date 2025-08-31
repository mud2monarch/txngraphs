use crate::types::TransferGraph;
use alloy_primitives::Address;
use petgraph::Directed;
use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::clone::Clone;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

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
        _ = write!(
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

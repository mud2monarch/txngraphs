# Graph-based transaction tool with reth-DB

This is a tool to find, visualize, and measure token transfers in a graph-based format. It takes advantage of the reth database to unlock blazingly fast exploration that has never been possible with open source software before. [Link to slides from Paradigm Frontiers' hackathon](https://docs.google.com/presentation/d/1j2BoZv-iszDs88wIsYS2kxomsdlOZNqOSjNmQhSGiKk/edit?usp=sharing), which explain the problem, the basic structure, and a POC.

![Graphic showing the transformation of a table of ERC-20 transfers into a directed graph.](https://zachrwong.info/wp-content/uploads/2025/09/Screenshot-2025-09-01-at-7.56.59-PM.png "For token transfers, graphs can be easier to reason about, but finding and formatting about the transfers is challenging today.")

There are no effective, open-source tools for transfer-based investigations today:
- Traditional tabular data like you could find on Dune is ineffective because visualization and measurement tools are not designed for this purpose; a bar chart does not show you if funds are moving between the same ten addresses.
- Commercial services such as TRM Labs, Chainalysis, and Arkham Intelligence provide graph-based tools and visualizations, but they're paid, and appear hard to use programmatically.
- Lastly, searches to assemble transfer graphs from common data sources like Dune, Allium, or RPCs are I/O-limited, and impractically slow and expensive.

This tool is built around reth DB, to solve the I/O problem, and includes visualization and measurement utilities to facilitate explorations.

# Current Status
*last ed. 8/31/25*

I aim to release a v0.1 of this tool. See feature list below. This could take up to 3 weeks of full time work.

**Key Insights:**
- **CPU-bound workload**: 91-98% of time spent in parsing/filtering logic, not database I/O
- **Reth database is incredibly fast**: Only 1-6% of time spent in system calls
- **Linear scaling**: Performance scales predictably with block count
- **Parallelization potential**: High CPU utilization suggests near-linear speedup possible with multiple cores

# Feat. list (WIP)
- ~Reorganize into clear modules~
- Parallelize reads
  - ~Write benchmark script/infra, measure~
  - ~Parallelize get_transfers() reads~
  - Parallelize BFS reads per-tier (need a mutex, so more complicated)
- ~TODO: look into: Why am I passing tokens as an `&[Address]`?~
- Look into SVG rendering perf
- TODO: I'm propagating errors but basically not handling them at all. I think things just crash if there's an issue... need to fix that. Also, is using Anyhow a good idea? I kind of don't think so, it feels 'cheap'. Perhaps I should define my own error types at this point.
- Expand reth source support to at least Superchain and ETH L1
- Get Cryo and CSV connectors working or discard them
- Good docs
- Python API
- ~Write the breadth-first search generic over a data source~
- ~Start with CSV then do RPC then do reth DB~
- Add support for >1 root address (maybe)
- ~Need to add filtering for token. Should be easy with good types.~
- Get Cryo connector working

## Performance Benchmarks

### Baseline - Sequential (commit eb31916)

| Workload | Blocks | Real Time | User (CPU) | Sys (I/O) | CPU % | I/O % |
|----------|--------|-----------|------------|-----------|-------|-------|
| Small    | 5K     | 0.32s     | 0.29s      | 0.02s     | 91%   | 6%    |
| Medium   | 20K    | 1.45s     | 1.42s      | 0.03s     | 98%   | 2%    |
| Large    | 100K   | 20.0s     | 18.8s      | 0.16s     | ---   | ---   |

### Rayon Parallelization (commit 4d12ed1)

Parallelized `get_transfers()` with 5K block chunks using `Rayon::into_par_iter()`:

| Workload | Blocks | Real Time | User (CPU) | Sys (I/O) | Real Speedup | CPU Overhead |
|----------|--------|-----------|------------|-----------|--------------|--------------|
| Small    | 5K     | 0.33s     | 0.30s      | 0.03s     | 1.0x         | 1.0x         |
| Medium   | 20K    | 1.06s     | 3.25s      | 0.04s     | **1.37x**    | 2.3x         |
| Large    | 100K   | 14.66s    | 95.51s     | 0.46s     | **1.36x**    | 5.1x         |

**Key Observations:**
- **25-36% real-time speedup** for medium/large workloads
- **2-5x increase in total CPU time** due to parallel overhead
- **Minimal I/O impact** - sys time remains under 3% of total
- **Thread contention likely** - speedup plateaus despite high CPU usage

# OLD NOTES BELOW
| **Description.** What is it? | A tool to visualize and measure concentration of ERC-20 token transfers amongst a group of addresses |
| --- | --- |
| **Problem.** What problem is this solving? | Given a set of root addresses, build a visual graph of all token transfers to *n* depth and calculate cumulative transfer volumes/other metrics. |
| **Why.** How do we know this is a real problem and worth solving? | We have a lot of tools and services to generate and measure tabular data structures, but nothing/very little that cleanly visualizes and measures token transfers in an efficient way. It's hard to implement iterative searchers in SQL databases, and downloading the entire history of ERC20 transfers is not viable. RPC calls would take days. |
| **Success.** How do we know if we’ve solved this problem? | We can quickly produce a visualization of all of the wallets involved in the PI token honeypot (`0x20f17D48646D57764334B6606d85518680D4e276`) and identify the beginning and ending addresses to which the WETH was transferred. |
| **Audience.** Who are we building for? | Myself and any other data analyst interested in tracking fund transfers onchain |
| **What.** What does this look like in the product? | CLI tool. Inputs: - Root address(es). Token address(es). Starting block number. Ending block number. Output: visual graph + metrics. |

### Outputs
- Visual graph
- Some measures of no. of txns, volume, and centrality
- Try to get <1s for 100k blocks? - [CGPT convo](https://chatgpt.com/share/e/6872c2bc-5358-8013-8a99-291ad6cfa795)
  - Chunk and parallelize reads

### Docs/Reference
- https://github.com/paradigmxyz/reth/blob/3277333df6ba9bd798f059e7a2d43d712e028d5c/crates/storage/db-api/src/lib.rs
- https://github.com/yash-atreya/reth-walk-storage/blob/main/src/main.rs
  - 5m slots in 0.5s
- all tables in reth db: https://github.com/paradigmxyz/reth/blob/3277333df6ba9bd798f059e7a2d43d712e028d5c/crates/storage/db-api/src/tables/mod.rs

### TODO algo
3. Degree distribution (“strandedness” measure)
pub struct DegreeStats {
    pub address: Address,
    pub in_degree: usize,
    pub out_degree: usize,
    pub total_degree: usize,
}

pub struct DegreeDistribution {
    pub min_transfers: usize,
    pub percent_of_nodes: f64,
}

pub fn compute_degree_stats(graph: &TransferGraph) -> Vec<DegreeStats>;

pub fn degree_distribution(stats: &[DegreeStats]) -> Vec<DegreeDistribution>;


compute_degree_stats → per-node stats.

degree_distribution → distribution table like “% of nodes with ≥ N transfers.”

That set covers:

Table 1 via aggregate_transfers.

Table 2 (cycles) via extract_cyclic_subgraph.

Table 3 (strandedness) via compute_degree_stats + degree_distribution.

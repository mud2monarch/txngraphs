# Graph-based transaction tool with reth-DB

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

### Zach notes/Todo
- ~Write the breadth-first search generic over a data source~
- ~Start with CSV then do RPC then do reth DB~
- Add support for >1 root address (maybe)
- ~Need to add filtering for token. Should be easy with good types.~
- Try to get <1s for 100k blocks? - [CGPT convo](https://chatgpt.com/share/e/6872c2bc-5358-8013-8a99-291ad6cfa795)

### Docs/Reference
- https://github.com/paradigmxyz/reth/blob/3277333df6ba9bd798f059e7a2d43d712e028d5c/crates/storage/db-api/src/lib.rs
- https://github.com/yash-atreya/reth-walk-storage/blob/main/src/main.rs
  - 5m slots in 0.5s
- all tables in reth db: https://github.com/paradigmxyz/reth/blob/3277333df6ba9bd798f059e7a2d43d712e028d5c/crates/storage/db-api/src/tables/mod.rs
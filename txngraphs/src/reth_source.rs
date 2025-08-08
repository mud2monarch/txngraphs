use tracing::{info, debug};
use alloy_primitives::{
    b256,
    Address, B256, U256,
    aliases::BlockNumber,
    Log,
};
use anyhow::{Context, Result};
use std::{path::Path, sync::Arc};
// Database components
use reth_db::{open_db_read_only, mdbx::DatabaseArguments, DatabaseEnv};
// Provider components
use reth_provider::{ProviderFactory, BlockBodyIndicesProvider, ReceiptProvider, TransactionsProvider};
use reth_provider::providers::StaticFileProvider;
// Chain specification
use reth_chainspec::ChainSpecBuilder;

// Node types
use reth_node_types::NodeTypesWithDBAdapter;
use reth_node_ethereum::EthereumNode;
use crate::{data_sources::TransferDataSource, types::Transfer};

const ERC20_TRANSFER_EVENT_SIGNATURE: B256 = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

pub struct RethTransferDataSource {
    pub factory: ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>
}

impl RethTransferDataSource {
    pub fn new(db_path: String) -> Self {
        let db_path = Path::new(&db_path);
        let db_env = open_db_read_only(db_path.join("db"), DatabaseArguments::default()).unwrap();
        let spec = ChainSpecBuilder::mainnet().build();

        let factory = ProviderFactory::<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>::new(
            Arc::new(db_env),
            spec.clone().into(),
            StaticFileProvider::read_only(db_path.join("static_files"), true).unwrap(),
        );

        Self { factory }
    }
}

impl TransferDataSource for RethTransferDataSource {
    fn get_transfers(
        &self,
        address: &Address,
        token_address: &Address,
        block_start: &BlockNumber,
        block_end: &BlockNumber,
    ) -> anyhow::Result<Vec<Transfer>> {
        let mut transfers: Vec<Transfer> = Vec::new();
        // I want to collect all txn data without the hash for efficiency so I need
        // this intermediate vector
        let mut txns_no_hash = Vec::new();
        
        let provider = self.factory.provider()?;
        
        for bn in *block_start..=*block_end {
            let txns_in_block = provider.block_body_indices(bn)
                .context("failed to get block body indices")?
                .context(format!("No block body indices found for block {}", bn))?;
            
            info!("Block {} has {} txns", bn, txns_in_block.tx_num_range().count());

            for tx_num in txns_in_block.tx_num_range() {
                let tx_receipt = provider.receipt(tx_num)
                    .context("failed to get tx receipt")?
                    .context(format!("No tx receipt found for tx_num {:?}", tx_num))?;
                
                // check if tx is relevant
                for log in &tx_receipt.logs {

                    // check if the log is from the ERC20 token I'm interested in
                    if log.address != *token_address {
                        continue;
                    }
              
                    // check if the event signature matches
                    debug!("log.topics(): {:?}", log.topics());

                    if log.topics().len() >= 3 && log.topics()[0] == ERC20_TRANSFER_EVENT_SIGNATURE {
                        // if this is a transfer then check the from address
                        let from = Address::from_word(log.topics()[1]);
                        
                        if from == *address {
                            let to = Address::from_word(log.topics()[2]);
                            let amount = U256::from_be_slice(&log.data.data);

                            txns_no_hash.push((tx_num, bn, from, to, amount));
                        }
                    }
                }
            }
        }

        // for matched txns, get the tx hash
        for txn in txns_no_hash {
            let tx_data = provider.transaction_by_id(txn.0)
                .context("failed to get transaction")?
                .context(format!("No transaction found for tx_num {:?}", txn.0))?;
            
            transfers.push(Transfer {
                tx_hash: *tx_data.hash(),
                block_number: txn.1,
                from_address: txn.2,
                to_address: txn.3,
                token: *token_address,
                amount: txn.4,
            })
        }
        Ok(transfers)
    }
}
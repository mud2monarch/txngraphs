use alloy_consensus::TxReceipt;
use alloy_primitives::{Address, B256, U256, aliases::BlockNumber, b256};
use anyhow::{Context, Result};
use std::{path::Path, sync::Arc};
use tracing::info;
// Database components
use reth_db::{DatabaseEnv, mdbx::DatabaseArguments, open_db_read_only};
use reth_op::node::OpNode;
use reth_optimism_chainspec::UNICHAIN_MAINNET;
use reth_provider::providers::StaticFileProvider;
use reth_provider::{
    BlockBodyIndicesProvider, ProviderFactory, ReceiptProvider, TransactionsProvider,
};

// Node types
use crate::{data_sources::TransferDataSource, types::Transfer};
use reth_node_types::NodeTypesWithDBAdapter;

const ERC20_TRANSFER_EVENT_SIGNATURE: B256 =
    b256!("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

pub struct RethTransferDataSource {
    pub factory: ProviderFactory<NodeTypesWithDBAdapter<OpNode, Arc<DatabaseEnv>>>,
}

impl RethTransferDataSource {
    pub fn new(db_path: String) -> Self {
        let db_path = Path::new(&db_path);
        let db_env = open_db_read_only(db_path.join("db"), DatabaseArguments::default()).unwrap();
        let spec = UNICHAIN_MAINNET.clone();

        let factory = ProviderFactory::<NodeTypesWithDBAdapter<OpNode, Arc<DatabaseEnv>>>::new(
            Arc::new(db_env),
            spec.into(),
            StaticFileProvider::read_only(db_path.join("static_files"), true).unwrap(),
        );

        Self { factory }
    }
}

impl TransferDataSource for RethTransferDataSource {
    fn get_transfers(
        &self,
        address: &Address,
        token_addresses: &[Address],
        block_start: &BlockNumber,
        block_end: &BlockNumber,
    ) -> Result<Vec<Transfer>> {
        let mut transfers: Vec<Transfer> = Vec::new();
        // I want to collect all txn data without the hash for efficiency so I need
        // this intermediate vector
        let mut txns_no_hash = Vec::new();

        let provider = self.factory.provider()?;

        for bn in *block_start..=*block_end {
            let txns_in_block = provider
                .block_body_indices(bn)
                .context("failed to get block body indices")?
                .context(format!("No block body indices found for block {}", bn))?;

            if bn % 1000 == 0 {
                info!("Processing block {}", bn);
            }

            for tx_num in txns_in_block.tx_num_range() {
                let tx_receipt = provider
                    .receipt(tx_num)
                    .context("failed to get tx receipt")?
                    .context(format!("No tx receipt found for tx_num {:?}", tx_num))?;

                // check if tx is relevant
                for log in tx_receipt.logs() {
                    if token_addresses.contains(&log.address)
                        && log.topics().len() == 3
                        && log.topics()[0] == ERC20_TRANSFER_EVENT_SIGNATURE
                        && Address::from_word(log.topics()[1]) == *address
                    {
                        let from = Address::from_word(log.topics()[1]);
                        let to = Address::from_word(log.topics()[2]);
                        let amount = U256::from_be_slice(&log.data.data);

                        txns_no_hash.push((tx_num, bn, from, to, amount, log.address));
                        info!(
                            "Pushed onto txns_no_hash: {:?}",
                            txns_no_hash.last().unwrap()
                        );
                    }
                }
            }
        }

        // for matched txns, get the tx hash
        for txn in txns_no_hash {
            let tx_data = provider
                .transaction_by_id(txn.0)
                .context("failed to get transaction")?
                .context(format!("No transaction found for tx_num {:?}", txn.0))?;

            transfers.push(Transfer {
                tx_hash: *tx_data.hash(),
                block_number: txn.1,
                from_address: txn.2,
                to_address: txn.3,
                token: txn.5,
                amount: txn.4,
            });

            info!(
                "Pushed onto transfers, Transfer: {:?}",
                transfers.last().unwrap()
            );
        }
        Ok(transfers)
    }
}

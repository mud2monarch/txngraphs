use tracing::info;
use alloy_primitives::{
    b256,
    Address, B256,
    aliases::{BlockNumber, TxHash, U256},
    Log,
};
use tracing::{debug};
use anyhow::{Context, Result};
use std::{path::PathBuf, sync::Arc};
use reth_db_api::{
    cursor::DbCursorRO,
    database::Database,
    tables,
    transaction::DbTx,
};
use reth_ethereum_primitives::Receipt;
use reth_storage_errors::db::DatabaseError;
use reth_db::{
    open_db_read_only,
    mdbx::DatabaseArguments,
    DatabaseEnv
};
use crate::{data_sources::TransferDataSource, types::Transfer};

const ERC20_TRANSFER_EVENT_SIGNATURE: B256 = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

pub struct RethTransferDataSource {
    pub db: reth_db::DatabaseEnv,
}

impl RethTransferDataSource {
    pub fn new(db_path: PathBuf) -> Self {
        let db = reth_db::open_db_read_only(db_path, DatabaseArguments::default())
            .expect("Failed to initialize a read connection to reth DB");
        Self { db }
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
        
        let db_tx = self.db.tx()?;
        for bn in *block_start..=*block_end {
            let txns_in_block = db_tx.get::<tables::BlockBodyIndices>(bn)
                .context("failed to get block body indices")?
                .context("No block body indices found")?;
            
            info!("Block {} has {} txns", bn, txns_in_block.tx_num_range().count());
            for tx_num in txns_in_block.tx_num_range() {
                let tx_receipt = db_tx.get::<tables::Receipts>(tx_num)
                    .context("failed to get tx receipt")?
                    .context(format!("No tx receipt found for tx_num {:?}", tx_num))?;
                
                // check if tx is relevant
                for log in tx_receipt.logs {

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

        // for matched txns, get the tx hash from tables::Transactions
        for txn in txns_no_hash {
            let tx_data = db_tx.get::<tables::Transactions>(txn.0)
                .context("failed to get tx hash")?
                .context(format!("No tx hash found for tx_num {:?}", txn.0))?;
            let tx_hash = tx_data.hash();
            
            transfers.push(Transfer {
                tx_hash: *tx_hash,
                block_number: txn.1,
                from_address: txn.2,
                to_address: txn.3,
                token: *token_address,
                amount: txn.4,
            })
        }

        db_tx.commit()?;
        Ok(transfers)
    }
}
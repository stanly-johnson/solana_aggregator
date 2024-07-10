//! Module for fetching and processing block data from Solana
use crate::db::{
	insert_or_update_account, insert_or_update_transaction, AccountRecord, TransactionRecord,
};
use log::{error, info};

use rusqlite::Connection;

use solana_client::rpc_client::RpcClient;
use solana_transaction_status::UiConfirmedBlock;
use std::{error::Error, sync::Arc};

use crate::types::Config;
use tokio::sync::Mutex;

pub mod processor;
pub mod retrieval;

use processor::parse_block;
use retrieval::{get_block, get_epoch_info};

/// Fetches and processes blocks for the current epoch.
///
/// This function retrieves the current epoch info, calculates the start and end slots for the
/// epoch, and iterates through each slot to fetch and parse the block data. The transactions within
/// each block are processed and stored in the SQLite database.
///
/// # Arguments
/// * `client` - A shared reference to the `RpcClient` for communicating with the Solana blockchain.
/// * `conn` - A shared, thread-safe reference to the SQLite database connection.
/// * `config` - Configuration parameters for the block aggregation process.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(Box<dyn Error + Send + Sync>)` on failure.
///
/// # Errors
/// This function returns an error if:
/// - The epoch information cannot be fetched.
/// - A block cannot be fetched after the specified number of retry attempts.
/// - The block data cannot be parsed.
/// - A transaction or account record cannot be inserted or updated in the SQLite database.
pub async fn aggregate_blocks(
	client: Arc<RpcClient>,
	conn: Arc<Mutex<Connection>>,
	config: Config,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	// Fetch the current epoch info
	let epoch_info = get_epoch_info(&client)?;
	info!("Epoch Info: {:?}", epoch_info);

	// Calculate the start and end slots for the current epoch
	let start_slot = epoch_info.absolute_slot - epoch_info.slot_index;
	let end_slot = start_slot + epoch_info.slots_in_epoch;

	info!("Fetching blocks from slot {} to {}", start_slot, end_slot);

	for slot in start_slot..=end_slot {
		match get_block_with_retry(&client, slot, config.retry_attempts).await {
			Ok(block) => {
				match parse_block(&block) {
					Ok(parsed_response) => {
						info!("Finished parsing block at slot {:?}", slot);

						let conn = conn.lock().await;
						for transaction in &parsed_response {
							let record = TransactionRecord {
								transaction_id: transaction.0.clone(),
								timestamp: block.block_time.unwrap_or_default(),
								block_height: slot,
								raw_transaction: serde_json::to_string(&transaction.1)?,
							};
							insert_or_update_transaction(&conn, &record)?;

							if let Some(transfer_info) = &transaction.2 {
								// Assuming each transaction has sender and receiver
								for account_id in
									&[transfer_info.sender.clone(), transfer_info.receiver.clone()]
								{
									let account_record = AccountRecord {
										account_id: account_id.clone(),
										estimated_balance: 0,
										related_transactions: vec![transaction.0.clone()],
									};
									insert_or_update_account(&conn, &account_record)?;
								}
							}
						}
					},
					Err(err) => {
						error!("Failed to parse block at slot {}: {:?}", slot, err);
					},
				}
			},
			Err(err) => {
				error!("Failed to fetch block at slot {}: {:?}", slot, err);
			},
		}
	}

	Ok(())
}

/// Fetches a block with retry logic in case of failures.
///
/// This function attempts to fetch a block from the Solana blockchain. If the fetch fails, it will
/// retry up to the specified number of times with an exponential backoff.
///
/// # Arguments
/// * `client` - A reference to the `RpcClient` for communicating with the Solana blockchain.
/// * `slot` - The slot number of the block to fetch.
/// * `retries` - The maximum number of retry attempts.
///
/// # Returns
/// * `Ok(UiConfirmedBlock)` on success, containing the fetched block data.
/// * `Err(Box<dyn Error + Send + Sync>)` on failure, containing the error encountered during the
///   fetch process.
async fn get_block_with_retry(
	client: &RpcClient,
	slot: u64,
	retries: u8,
) -> Result<UiConfirmedBlock, Box<dyn Error + Send + Sync>> {
	let mut attempts = 0;
	let mut wait_time = 2; // initial wait time in seconds

	loop {
		match get_block(client, slot) {
			Ok(block) => return Ok(block),
			Err(err) if attempts < retries => {
				attempts += 1;
				info!("Retry {}/{} for slot {}: {:?}", attempts, retries, slot, err);
				tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
				wait_time *= 2; // exponential backoff
			},
			Err(err) => return Err(err),
		}
	}
}

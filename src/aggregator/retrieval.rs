use crate::types::EpochInfo;
use log::error;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_transaction_status::{UiConfirmedBlock, UiTransactionEncoding};
use std::error::Error;

/// Retrieves epoch information from the RPC client.
pub fn get_epoch_info(client: &RpcClient) -> Result<EpochInfo, Box<dyn Error + Send + Sync>> {
	let epoch_info = client.get_epoch_info().map_err(|e| {
		error!("Failed to get epoch info: {}", e);
		format!("Failed to get epoch info: {}", e)
	})?;
	Ok(EpochInfo {
		absolute_slot: epoch_info.absolute_slot,
		slot_index: epoch_info.slot_index,
		slots_in_epoch: epoch_info.slots_in_epoch,
	})
}

/// Retrieves a confirmed block from the RPC client for a given slot.
pub fn get_block(
	client: &RpcClient,
	slot: u64,
) -> Result<UiConfirmedBlock, Box<dyn Error + Send + Sync>> {
	let block = client
		.get_block_with_config(
			slot,
			RpcBlockConfig {
				encoding: Some(UiTransactionEncoding::JsonParsed),
				transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
				rewards: Some(false),
				commitment: None,
				max_supported_transaction_version: Some(1),
			},
		)
		.map_err(|e| {
			error!("Failed to get block for slot {}: {}", slot, e);
			format!("Failed to get block for slot {}: {}", slot, e)
		})?;
	Ok(block)
}

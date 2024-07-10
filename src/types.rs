use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct EpochInfo {
	pub absolute_slot: u64,
	pub slot_index: u64,
	pub slots_in_epoch: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDetails {
	pub sender: String,
	pub receiver: String,
	pub amount: u64,
	pub timestamp: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferInfo {
	pub source: String,
	pub destination: String,
	pub lamports: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedInstruction {
	pub info: TransferInfo,
	#[serde(rename = "type")]
	pub instruction_type: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
	pub rpc_url: String,
	pub retry_attempts: u8,
	pub server_address: String,
}

impl Config {
	pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let config_content = fs::read_to_string(file_path)?;
		let config: Config = toml::from_str(&config_content)?;
		Ok(config)
	}
}

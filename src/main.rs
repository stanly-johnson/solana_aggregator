mod aggregator;
mod db;
mod server;
mod tests;
pub mod types;

use axum::{routing::get, Extension, Router};
use db::{initialize_db, TransactionRecord};
use log::{error, info};
use rusqlite::Connection;

use solana_client::rpc_client::RpcClient;

use crate::aggregator::aggregate_blocks;
use server::{get_account_handler, get_transaction_handler};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;
use types::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	env_logger::init();

	// Load the configuration file
	let config = Config::from_file("config.toml")?;
	info!("Starting Solana Aggregator with config: {:?}", config);

	let client = Arc::new(RpcClient::new(config.rpc_url.to_string()));

	// Initialize SQLite database
	let conn = Arc::new(Mutex::new(Connection::open("solana.db")?));
	{
		let conn = conn.lock().await;
		initialize_db(&conn)?;
	}

	// Start the block aggregation process
	let client_clone = Arc::clone(&client);
	let conn_clone = Arc::clone(&conn);
	let config_clone = config.clone();

	tokio::spawn(async move {
		if let Err(e) = aggregate_blocks(client_clone, conn_clone, config_clone).await {
			error!("Block aggregation process failed: {:?}", e);
		}
	});

	// Build the API service with Axum
	let app = Router::new()
		.route("/transaction", get(get_transaction_handler))
		.route("/accountid", get(get_account_handler))
		.layer(Extension(Arc::clone(&conn)));

	// Run the Axum server
	axum::Server::bind(&config.server_address.parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}

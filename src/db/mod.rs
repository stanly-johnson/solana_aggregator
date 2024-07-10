use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use std::error::Error;

/// A record representing a transaction.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRecord {
	pub transaction_id: String,
	pub timestamp: i64,
	pub block_height: u64,
	pub raw_transaction: String,
}

/// A record representing an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRecord {
	pub account_id: String,
	pub estimated_balance: u64,
	pub related_transactions: Vec<String>,
}

/// Initializes the database with the required tables.
///
/// This function creates the `transactions` and `accounts` tables if they do not already exist.
///
/// # Arguments
///
/// * `conn` - A reference to a `Connection` object representing the database connection.
///
/// # Returns
///
/// This function returns a `Result` indicating success or failure.
pub fn initialize_db(conn: &Connection) -> Result<()> {
	conn.execute(
		"CREATE TABLE IF NOT EXISTS transactions (
            transaction_id TEXT PRIMARY KEY,
            timestamp INTEGER,
            block_height INTEGER,
            raw_transaction TEXT
        )",
		[],
	)?;

	conn.execute(
		"CREATE TABLE IF NOT EXISTS accounts (
            account_id TEXT PRIMARY KEY,
            estimated_balance INTEGER,
            related_transactions TEXT
        )",
		[],
	)?;

	Ok(())
}

/// Inserts or updates a transaction record in the database.
///
/// This function inserts a new transaction record or updates an existing record with the same
/// transaction ID.
///
/// # Arguments
///
/// * `conn` - A reference to a `Connection` object representing the database connection.
/// * `record` - A reference to a `TransactionRecord` containing the transaction details.
///
/// # Returns
///
/// This function returns a `Result` indicating success or failure.
///
/// # Errors
///
/// This function returns an error if the database operation fails.
pub fn insert_or_update_transaction(
	conn: &Connection,
	record: &TransactionRecord,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	conn.execute(
        "INSERT OR REPLACE INTO transactions (transaction_id, timestamp, block_height, raw_transaction) VALUES (?1, ?2, ?3, ?4)",
        params![
            record.transaction_id,
            record.timestamp,
            record.block_height,
            record.raw_transaction
        ],
    )?;
	Ok(())
}

/// Inserts or updates an account record in the database.
///
/// This function inserts a new account record or updates an existing record with the same account
/// ID.
///
/// # Arguments
///
/// * `conn` - A reference to a `Connection` object representing the database connection.
/// * `record` - A reference to an `AccountRecord` containing the account details.
///
/// # Returns
///
/// This function returns a `Result` indicating success or failure.
///
/// # Errors
///
/// This function returns an error if the database operation fails or if the related transactions
/// cannot be serialized.
pub fn insert_or_update_account(
	conn: &Connection,
	record: &AccountRecord,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	let transactions_json = serde_json::to_string(&record.related_transactions)?;
	conn.execute(
        "INSERT OR REPLACE INTO accounts (account_id, estimated_balance, related_transactions) VALUES (?1, ?2, ?3)",
        params![
            record.account_id,
            record.estimated_balance,
            transactions_json
        ],
    )?;
	Ok(())
}

/// Retrieves a transaction record from the database by transaction ID.
///
/// This function fetches a transaction record matching the given transaction ID.
///
/// # Arguments
///
/// * `conn` - A reference to a `Connection` object representing the database connection.
/// * `tx_id` - A string slice containing the transaction ID.
///
/// # Returns
///
/// This function returns a `Result` containing an `Option<TransactionRecord>`.
/// The `Option` is `Some` if a matching record is found, and `None` otherwise.
///
/// # Errors
///
/// This function returns an error if the database operation fails or if deserialization fails.
pub fn get_transaction(
	conn: &Connection,
	tx_id: &str,
) -> Result<Option<TransactionRecord>, Box<dyn Error + Send + Sync>> {
	let mut stmt = conn.prepare("SELECT transaction_id, timestamp, block_height, raw_transaction FROM transactions WHERE transaction_id = ?1")?;
	let mut rows = stmt.query(params![tx_id])?;

	if let Some(row) = rows.next()? {
		Ok(Some(TransactionRecord {
			transaction_id: row.get(0)?,
			timestamp: row.get(1)?,
			block_height: row.get(2)?,
			raw_transaction: row.get(3)?,
		}))
	} else {
		Ok(None)
	}
}

/// Retrieves an account record from the database by account ID.
///
/// This function fetches an account record matching the given account ID.
///
/// # Arguments
///
/// * `conn` - A reference to a `Connection` object representing the database connection.
/// * `account_id` - A string slice containing the account ID.
///
/// # Returns
///
/// This function returns a `Result` containing an `Option<AccountRecord>`.
/// The `Option` is `Some` if a matching record is found, and `None` otherwise.
///
/// # Errors
///
/// This function returns an error if the database operation fails or if deserialization fails.
pub fn get_account(
	conn: &Connection,
	account_id: &str,
) -> Result<Option<AccountRecord>, Box<dyn Error + Send + Sync>> {
	let mut stmt = conn.prepare("SELECT account_id, estimated_balance, related_transactions FROM accounts WHERE account_id = ?1")?;
	let mut rows = stmt.query(params![account_id])?;

	if let Some(row) = rows.next()? {
		let related_transactions: String = row.get(2)?;
		let related_transactions: Vec<String> = serde_json::from_str(&related_transactions)?;
		Ok(Some(AccountRecord {
			account_id: row.get(0)?,
			estimated_balance: row.get(1)?,
			related_transactions,
		}))
	} else {
		Ok(None)
	}
}

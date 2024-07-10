use crate::types::{ParsedInstruction, TransactionDetails};
use log::{debug, error};

use solana_transaction_status::{
	EncodedTransaction, UiConfirmedBlock, UiInstruction, UiMessage, UiParsedInstruction,
};
use std::error::Error;

/// Parses a confirmed block and extracts transaction details.
///
/// This function takes a reference to a `UiConfirmedBlock` and attempts to parse
/// each transaction within the block. It returns a vector of tuples, where each
/// tuple contains:
/// - A `String` representing the transaction signature.
/// - An `EncodedTransaction` which is the transaction itself.
/// - An `Option<TransactionDetails>` which contains parsed transaction details if the transaction
///   was successfully parsed, or `None` if the transaction was unsupported or failed to parse.
///
/// # Arguments
///
/// * `block` - A reference to a `UiConfirmedBlock` containing transactions to be parsed.
///
/// # Returns
///
/// This function returns a `Result` containing:
/// - `Ok(Vec<(String, EncodedTransaction, Option<TransactionDetails>)>)` on success.
/// - `Err(Box<dyn Error + Send + Sync>)` if there was an error during the parsing process.
pub fn parse_block(
	block: &UiConfirmedBlock,
) -> Result<
	Vec<(String, EncodedTransaction, Option<TransactionDetails>)>,
	Box<dyn Error + Send + Sync>,
> {
	let mut transaction_details = Vec::new();

	if let Some(transactions) = &block.transactions {
		for transaction_with_meta in transactions {
			let tx_signature = get_transaction_signature(&transaction_with_meta.transaction)?;
			match parse_transaction(&transaction_with_meta.transaction, block.block_time) {
				Ok(Some(parsed_transaction)) => {
					transaction_details.push((
						tx_signature,
						transaction_with_meta.transaction.clone(),
						Some(parsed_transaction),
					));
				},
				Ok(None) => {
					debug!("Parsed and not supported tx found");
					transaction_details.push((
						tx_signature,
						transaction_with_meta.transaction.clone(),
						None,
					));
				},
				Err(err) => {
					error!("Failed to parse transaction: {:?}", err);
				},
			}
		}
	}

	Ok(transaction_details)
}

/// Extracts the transaction signature from an encoded transaction.
///
/// This function takes a reference to an `EncodedTransaction` and attempts to retrieve
/// the transaction signature. It supports transactions encoded in JSON format and returns
/// the first signature found.
///
/// # Arguments
///
/// * `transaction` - A reference to an `EncodedTransaction` from which the signature will be
///   extracted.
///
/// # Returns
///
/// This function returns a `Result` containing:
/// - `Ok(String)` with the transaction signature on success.
/// - `Err(Box<dyn Error + Send + Sync>)` if the transaction encoding is unsupported.
///
/// # Errors
///
/// This function will return an error if the transaction encoding is not JSON.
pub fn get_transaction_signature(
	transaction: &EncodedTransaction,
) -> Result<String, Box<dyn Error + Send + Sync>> {
	match transaction {
		EncodedTransaction::Json(ui_transaction) => Ok(ui_transaction.signatures[0].clone()),
		_ => Err("Unsupported transaction encoding".into()),
	}
}

/// Parses an encoded transaction and extracts transaction details if supported.
///
/// This function takes a reference to an `EncodedTransaction` and an optional timestamp,
/// and attempts to parse the transaction to extract details such as sender, receiver,
/// amount, and timestamp. It supports JSON encoded transactions with parsed messages.
///
/// # Arguments
///
/// * `transaction` - A reference to an `EncodedTransaction` to be parsed.
/// * `timestamp` - An optional `i64` timestamp associated with the transaction.
///
/// # Returns
///
/// This function returns a `Result` containing:
/// - `Ok(Some(TransactionDetails))` with the parsed transaction details on success.
/// - `Ok(None)` if the transaction format is supported but no relevant details were found.
/// - `Err(Box<dyn Error + Send + Sync>)` if the transaction encoding or format is unsupported, or
///   if an error occurs during parsing.
///
/// # Errors
///
/// This function will return an error if:
/// - The transaction encoding is unsupported.
/// - The transaction message format is unsupported.
/// - Deserialization of transfer information fails.
pub fn parse_transaction(
	transaction: &EncodedTransaction,
	timestamp: Option<i64>,
) -> Result<Option<TransactionDetails>, Box<dyn Error + Send + Sync>> {
	match transaction {
		EncodedTransaction::Json(ui_transaction) => {
			if let UiMessage::Parsed(parsed_message) = &ui_transaction.message {
				for instruction in &parsed_message.instructions {
					if let UiInstruction::Parsed(parsed_instruction) = instruction {
						if let UiParsedInstruction::Parsed(ref parsed_inst) = parsed_instruction {
							if parsed_inst.program_id == "11111111111111111111111111111111" {
								let transfer_info: ParsedInstruction =
									serde_json::from_value(parsed_inst.parsed.clone()).map_err(
										|e| format!("Failed to deserialize transfer info: {}", e),
									)?;
								return Ok(Some(TransactionDetails {
									sender: transfer_info.info.source,
									receiver: transfer_info.info.destination,
									amount: transfer_info.info.lamports,
									timestamp,
								}))
							}
						}

						// if let UiParsedInstruction::PartiallyDecoded(ref parsed_inst) =
						// parsed_instruction {     todo!();
						// }
					}
				}
			} else {
				return Err("Unsupported transaction message format".into())
			}
		},
		_ => return Err("Unsupported transaction encoding".into()),
	}
	Ok(None)
}

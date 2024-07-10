use axum::{
	body::{boxed, Full},
	extract::Query,
	http::StatusCode,
	response::{IntoResponse, Json, Response},
	Extension,
};
use log::error;
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::{db, TransactionRecord};
use tokio::sync::Mutex;

/// Query parameters for retrieving a transaction.
#[derive(Deserialize)]
pub struct TransactionQuery {
	#[serde(rename = "tx-id")]
	tx_id: String,
}

/// Handler for retrieving a transaction record from the database.
///
/// This asynchronous function takes a transaction query with a transaction ID, and retrieves the
/// corresponding transaction record from the database. It returns the transaction record as JSON
/// or an appropriate error response.
///
/// # Arguments
///
/// * `params` - A `Query` extractor containing the `TransactionQuery`.
/// * `conn` - An `Extension` extractor providing an `Arc<Mutex<Connection>>` to the database.
///
/// # Returns
///
/// This function returns a `Result` containing:
/// - `Ok(Json<TransactionRecord>)` if the transaction is found.
/// - `Err(Response)` with a `NOT_FOUND` status if the transaction is not found.
/// - `Err(Response)` with an `INTERNAL_SERVER_ERROR` status if there is a database query error.
///
/// # Errors
///
/// This function returns an error response if the transaction is not found or if there is a
/// database query error.
pub async fn get_transaction_handler(
	Query(params): Query<TransactionQuery>,
	axum::extract::Extension(conn): axum::extract::Extension<Arc<Mutex<Connection>>>,
) -> Result<Json<TransactionRecord>, Response> {
	let tx_id = params.tx_id;
	let conn = conn.lock().await;
	match db::get_transaction(&conn, &tx_id) {
		Ok(Some(transaction)) => Ok(Json(transaction)),
		Ok(None) => Err(build_error_response(StatusCode::NOT_FOUND, "Transaction not found")),
		Err(err) => {
			error!("Database query error: {:?}", err);
			Err(build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))
		},
	}
}

/// Query parameters for retrieving an account.
#[derive(Deserialize)]
pub struct AccountQuery {
	#[serde(rename = "account-id")]
	account_id: String,
}

/// Handler for retrieving an account record from the database.
///
/// This asynchronous function takes an account query with an account ID, and retrieves the
/// corresponding account record from the database. It returns the account record as JSON
/// or an appropriate error response.
///
/// # Arguments
///
/// * `params` - A `Query` extractor containing the `AccountQuery`.
/// * `conn` - An `Extension` extractor providing an `Arc<Mutex<Connection>>` to the database.
///
/// # Returns
///
/// This function returns an `impl IntoResponse` which can be either:
/// - `Json<AccountRecord>` if the account is found.
/// - An error response with a `NOT_FOUND` status if the account is not found.
/// - An error response with an `INTERNAL_SERVER_ERROR` status if there is a database query error.
///
/// # Errors
///
/// This function returns an error response if the account is not found or if there is a database
/// query error.
pub async fn get_account_handler(
	Query(params): Query<AccountQuery>,
	Extension(conn): Extension<Arc<Mutex<Connection>>>,
) -> impl IntoResponse {
	let account_id = params.account_id;
	let conn = conn.lock().await;
	match db::get_account(&conn, &account_id) {
		Ok(Some(account)) => Json(account).into_response(),
		Ok(None) =>
			build_error_response(StatusCode::NOT_FOUND, "Account not found").into_response(),
		Err(err) => {
			error!("Database query error: {:?}", err);
			build_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
				.into_response()
		},
	}
}

/// Builds an error response with a given status code and message.
///
/// This function takes a status code and a message, and constructs an HTTP response with
/// the given status and a body containing the message.
///
/// # Arguments
///
/// * `status` - A `StatusCode` indicating the HTTP status of the response.
/// * `message` - A string slice containing the error message.
///
/// # Returns
///
/// This function returns a `Response` with the given status and message.
fn build_error_response(status: StatusCode, message: &str) -> Response {
	let message = message.to_string();
	Response::builder().status(status).body(boxed(Full::from(message))).unwrap()
}

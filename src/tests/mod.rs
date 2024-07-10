#![cfg(test)]
#![allow(unused_imports)]
use crate::{
	db, db::AccountRecord, get_account_handler, get_transaction_handler, initialize_db,
	TransactionRecord,
};
use axum::{
	body::Body,
	http::{Request, StatusCode},
	response::Response,
	routing::get,
	Extension, Router,
};
use hyper::body::to_bytes;
use rusqlite::Connection;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

mod aggregator;

mod api;

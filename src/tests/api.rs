use super::*;

// Utility function to setup the router with a mock database connection
async fn setup_router() -> (Router, Arc<Mutex<Connection>>) {
	let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
	{
		let conn = conn.clone();
		let conn = conn.lock().await;
		initialize_db(&conn).unwrap();
	}

	let router = Router::new()
		.route("/transaction", get(get_transaction_handler))
		.route("/accountid", get(get_account_handler))
		.layer(Extension(conn.clone()));

	(router, conn)
}

#[tokio::test]
async fn test_get_transaction_handler_success() {
	let (router, conn) = setup_router().await;

	// Insert mock data
	{
		let conn = conn.lock().await;
		let record = TransactionRecord {
			transaction_id: "tx1".to_string(),
			timestamp: 1622556000,
			block_height: 12345,
			raw_transaction: "raw_data".to_string(),
		};
		db::insert_or_update_transaction(&conn, &record).unwrap();
	}

	let response = router
		.oneshot(Request::builder().uri("/transaction?tx-id=tx1").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body()).await.unwrap();
	let transaction: TransactionRecord = serde_json::from_slice(&body).unwrap();

	assert_eq!(transaction.transaction_id, "tx1");
	assert_eq!(transaction.timestamp, 1622556000);
	assert_eq!(transaction.block_height, 12345);
	assert_eq!(transaction.raw_transaction, "raw_data");
}

#[tokio::test]
async fn test_get_transaction_handler_not_found() {
	let (router, _conn) = setup_router().await;

	let response = router
		.oneshot(
			Request::builder()
				.uri("/transaction?tx-id=nonexistent")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_account_handler_success() {
	let (router, conn) = setup_router().await;

	// Insert mock data
	{
		let conn = conn.lock().await;
		let record = AccountRecord {
			account_id: "acc1".to_string(),
			estimated_balance: 1000,
			related_transactions: vec!["tx1".to_string(), "tx2".to_string()],
		};
		db::insert_or_update_account(&conn, &record).unwrap();
	}

	let response = router
		.oneshot(
			Request::builder()
				.uri("/accountid?account-id=acc1")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body()).await.unwrap();
	let account: AccountRecord = serde_json::from_slice(&body).unwrap();

	assert_eq!(account.account_id, "acc1");
	assert_eq!(account.estimated_balance, 1000);
	assert_eq!(account.related_transactions, vec!["tx1".to_string(), "tx2".to_string()]);
}

#[tokio::test]
async fn test_get_account_handler_not_found() {
	let (router, _conn) = setup_router().await;

	let response = router
		.oneshot(
			Request::builder()
				.uri("/accountid?account-id=nonexistent")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

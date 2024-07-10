## Solana Transaction Aggregator

## Prerequisites

This crate requires rustup and cargo setup in your env

## Setup

1. **Clone the Repository:**

   ```sh
   git clone https://github.com/your-repo/solana-aggregator.git
   cd solana-aggregator
   ```

2. **Install Dependencies and build:**

  ```sh
   cargo b
   ```

## Configuration

Create a configuration file named `config.toml` in the root directory of the project with the following content:

```toml
rpc_url = "https://api.testnet.solana.com"
retry_attempts = 3
server_address="127.0.0.1:3030"
```

Adjust the rpc_url to your Solana RPC endpoint and the server_address to the desired address and port for the API server.


## Running the Application

To run the Solana Aggregator application, execute the following command:

```sh
cargo +nightly run
```

To view detailed logs:

```sh
RUST_LOG=info cargo +nightly t
```

## How to test

```sh
cargo test
```


## Architecture

The crate is designed with a simple architecture : 

Aggregator Module: Handles the block aggregation process.
Database Module: Manages interactions with the SQLite database.
Server Module: Provides API endpoints using Axum.

When the tool is started it starts to sync the blocks from the current epoch, this is moved to a background task to continue syncing. The axum server is started to then serve the api endpoints.

On the DB, since we are targetting transactions and accounts info, the transactions are stored in a table with timestamp/slot/raw_tx etc.. and another table stores the account info linking the transactions to user accounts.

## Possible Improvements

Many features have been ommitted to save time, but ideally these features are next to implement:

- [ ] Decode all types of transactions (right now only native transfers are handled)
- [ ] Optimise the initial harvesting (using threads to calculate and then commit in a single tx)
- [ ] Improve error handling

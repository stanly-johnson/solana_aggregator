use super::*;
use crate::aggregator::processor::{get_transaction_signature, parse_block};
use solana_transaction_status::{
	EncodedTransaction, EncodedTransactionWithStatusMeta, TransactionStatusMeta, UiConfirmedBlock,
	UiInnerInstructions, UiInstruction, UiMessage, UiParsedInstruction, UiTransaction,
};

const MOCK_JSON: &str = r#"{
        "blockHeight": 298414228,
        "blockTime": 1720421680,
        "blockhash": "AZ8jzQjcgFSKYZ47sUVGTn7nR3FowHoyszEo2Nwq8vae",
        "parentSlot": 310175999,
        "previousBlockhash": "6RbXYJiJa8V7K5YJyS8YjkWsWf6Vuh5vGEzww7xSWigf",
        "transactions": [
        {
            "meta": {
                "computeUnitsConsumed": 450,
                "err": null,
                "fee": 5040,
                "innerInstructions": [],
                "logMessages": ["Program ComputeBudget111111111111111111111111111111 invoke [1]", "Program ComputeBudget111111111111111111111111111111 success", "Program ComputeBudget111111111111111111111111111111 invoke [1]", "Program ComputeBudget111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [1]", "Program 11111111111111111111111111111111 success"],
                "postBalances": [771523247926, 1798951577, 1, 1],
                "postTokenBalances": [],
                "preBalances": [771523253933, 1798950610, 1, 1],
                "preTokenBalances": [],
                "rewards": null,
                "status": {
                    "Ok": null
                }
            },
            "transaction": {
                "message": {
                    "accountKeys": [{
                        "pubkey": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g",
                        "signer": true,
                        "source": "transaction",
                        "writable": true
                    }, {
                        "pubkey": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
                        "signer": false,
                        "source": "transaction",
                        "writable": true
                    }, {
                        "pubkey": "11111111111111111111111111111111",
                        "signer": false,
                        "source": "transaction",
                        "writable": false
                    }, {
                        "pubkey": "ComputeBudget111111111111111111111111111111",
                        "signer": false,
                        "source": "transaction",
                        "writable": false
                    }],
                    "instructions": [{
                        "accounts": [],
                        "data": "LKoyXd",
                        "programId": "ComputeBudget111111111111111111111111111111",
                        "stackHeight": null
                    }, {
                        "accounts": [],
                        "data": "3auSnstjHdqH",
                        "programId": "ComputeBudget111111111111111111111111111111",
                        "stackHeight": null
                    }, {
                        "parsed": {
                            "info": {
                                "destination": "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ",
                                "lamports": 967,
                                "source": "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g"
                            },
                            "type": "transfer"
                        },
                        "program": "system",
                        "programId": "11111111111111111111111111111111",
                        "stackHeight": null
                    }],
                    "recentBlockhash": "FF2Z9QfmsehPeSoSC3ekupHCNt3VvxkLrcAZagAUXU85"
                },
                "signatures": ["2xBbzb1SjzSw5VjY92bjRYUB49Exnn45xE7RXRdbgR4XuyKQzJKFkA5kyy98MEDHDCUaQe1qEN4YbyY6jNpUqm1"]
            },
            "version": "legacy"
        }
        
        ]}
    "#;

fn create_mock_ui_confirmed_block() -> UiConfirmedBlock {
	serde_json::from_str(MOCK_JSON).unwrap()
}

#[test]
fn test_get_transaction_signature() {
	let block = create_mock_ui_confirmed_block();
	let encoded_transaction = &block.transactions.as_ref().unwrap()[0].transaction;

	// Test get_transaction_signature function
	let result = get_transaction_signature(encoded_transaction);
	assert!(result.is_ok());
	assert_eq!(
		result.unwrap(),
		"2xBbzb1SjzSw5VjY92bjRYUB49Exnn45xE7RXRdbgR4XuyKQzJKFkA5kyy98MEDHDCUaQe1qEN4YbyY6jNpUqm1"
	);
}

#[test]
fn test_parse_block() {
	let block = create_mock_ui_confirmed_block();

	// Test parse_block function
	let result = parse_block(&block);
	assert!(result.is_ok());
	let transactions = result.unwrap();
	assert_eq!(transactions.len(), 1);
	let (signature, encoded_tx, details) = &transactions[0];
	assert_eq!(
		signature,
		"2xBbzb1SjzSw5VjY92bjRYUB49Exnn45xE7RXRdbgR4XuyKQzJKFkA5kyy98MEDHDCUaQe1qEN4YbyY6jNpUqm1"
	);
	assert!(matches!(encoded_tx, EncodedTransaction::Json(_)));
	let details = details.as_ref().unwrap();
	assert_eq!(details.sender, "tKeYE4wtowRb8yRroZShTipE18YVnqwXjsSAoNsFU6g");
	assert_eq!(details.receiver, "84YKYKo7qN54VHFLn6Eo5uBZMKzUY5Q9qB2t1L3drUeQ");
	assert_eq!(details.amount, 967);
	assert_eq!(details.timestamp, Some(1720421680));
}

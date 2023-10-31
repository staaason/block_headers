use dotenv::dotenv;
use std::env;

extern crate web3;

use web3::Web3;
use web3::transports::Http;
use web3::types::{Block, BlockId, H256, Index};

use serde_json::json;
use tokio::fs;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

fn create_web3_client() -> Result<Web3<Http>, Box<dyn std::error::Error>> {
    // Loading environment variables from .env file
    dotenv().ok();

    let infura_api_key = match env::var("INFURA_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            return Err("Failed to get INFURA_API_KEY value.".into());
        }
    };

    let infura_url = format!("https://mainnet.infura.io/v3/{}", infura_api_key);
    let transport = web3::transports::Http::new(&infura_url).unwrap();
    let web3 = web3::Web3::new(transport);

    Ok(web3)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <start_block> <end_block>", args[0]);
        std::process::exit(1);
    }

    let start_block: u64 = args[1].parse().expect("Invalid start block number");
    let end_block: u64 = args[2].parse().expect("Invalid end block number");

    let web3 = match create_web3_client() {
        Ok(web3) => web3,
        Err(err) => {
            println!("Failed to create web3 client: {}", err);
            return;
        }
    };
    let dir_path = "jsons";
    if let Err(_) = fs::metadata(&dir_path).await {
        fs::create_dir(&dir_path).await.expect("Failed to create 'jsons' directory");
    }
    let file_name = format!("jsons/block_headers_{}-{}.json", start_block, end_block);
    let mut all_blocks_data = Vec::new();
    for block_number in start_block..=end_block {
        let block_id = BlockId::Number(block_number.into());

        let block: Option<Block<H256>> = web3
            .eth()
            .block(block_id)
            .await
            .expect("Could not get block");
        if let Some(block) = block {
            let uncle_count = web3.eth().uncle_count(block_id).await;

            // This was relevant for Ethereum when it was a proof-of-work network,
            // but ommers are not a feature of proof-of-stake Ethereum
            // because precisely one block proposer is selected in each slot.
            // https://ethereum.org/ru/glossary/#section-o
            let mut uncles = Vec::new();

            if let Ok(Some(uncle_count)) = uncle_count {
                for index in 0..uncle_count.low_u64() {
                    let uncle = web3.eth().uncle(block_id, Index::from(index)).await;
                    if let Ok(Some(uncle)) = uncle {
                        uncles.push(uncle);
                    }
                }
            }
            block.number
            let json_data = json!({
                "number": block.number,
                "hash": block.hash,
                "parent_hash": block.parent_hash,
                "uncles_hash": block.uncles_hash,
                 "withdrawals_root": block.withdrawals_root,
                 "blob_gas_used": block.blob_gas_used,
                 "excess_blob_gas": block.excess_blob_gas,
                 "parent_beacon_root": block.parent_beacon_root,
                "author": block.author,
                "state_root": block.state_root,
                "transactions_root": block.transactions_root,
                "receipts_root": block.receipts_root,
                "gas_used": block.gas_used,
                "gas_limit": block.gas_limit,
                "base_fee_per_gas": block.base_fee_per_gas,
                "extra_data": block.extra_data,
                "logs_bloom": block.logs_bloom,
                "timestamp": block.timestamp,
                "difficulty": block.difficulty,
                "mix_hash": block.mix_hash,
                "nonce": block.nonce,
                "uncles": uncles
            });
            all_blocks_data.push(json_data);
        };
    };
    let file = File::create(file_name).await;
    let _ = file.expect("Failed to create file").write_all(
        serde_json::to_string(&all_blocks_data).expect("Failed to serialize JSON").as_bytes()
    ).await;
}


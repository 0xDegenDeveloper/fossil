use ethers::prelude::*;
use ethers::types::{BlockNumber, TxHash};
use futures::future::join_all;
use std::sync::Arc;

use crate::utils::helpers::add_synthetic_blocks;

// Find the block closest to the given timestamp
// @note Need to handle if timestamp >> latest_block_timestamp
pub async fn get_closest_block(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Block<TxHash>, ProviderError> {
    println!("Finding block closest to timestamp: {}", timestamp);

    // Fetch the latest block
    let latest_block = provider.get_block(BlockNumber::Latest).await?.unwrap();
    let latest_block_number = latest_block.number.unwrap().as_u64();
    let latest_block_timestamp = latest_block.timestamp.as_u64();

    // Check if the timestamp is for a block that has not mined yet
    if timestamp >= latest_block_timestamp + 13 {
        panic!("ERROR: Requesting job for the future");
    }

    // Start binary search for block
    let mut lower = 0;
    let mut upper = latest_block_number;

    // Binary search
    while lower <= upper {
        let mid = (lower + upper) / 2;
        let mid_block = provider.get_block(mid).await?.unwrap();
        let mid_timestamp = mid_block.timestamp.as_u64();

        if mid_timestamp == timestamp {
            // Found exact block
            print_block_found(&mid_block);
            return Ok(mid_block);
        } else if mid_timestamp < timestamp {
            // Move lower bound up
            lower = mid + 1;
        } else {
            // Move upper bound down, preventing underflow
            if mid == 0 {
                break;
            }
            upper = mid - 1;
        }
    }

    // After the loop, upper will be the block number where block.timestamp <= T
    let block = provider.get_block(upper).await?.unwrap();
    print_block_found(&block);

    Ok(block)
}

// Gets an array of blocks between from_block and to_block (both inclusive)
pub async fn fetch_blocks(
    from_block: u64,
    to_block: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Vec<Block<TxHash>>, ProviderError> {
    println!("Fetching blocks...");

    // Collect the block numbers into a vector
    let block_numbers: Vec<u64> = (from_block..=to_block).collect();

    // Create a vector of futures for fetching each block
    let block_futures = block_numbers.into_iter().map(|block_number| {
        let provider = provider.clone();
        async move { provider.get_block(block_number).await }
    });

    // Fetch all blocks concurrently
    let blocks_results = join_all(block_futures).await;

    // Push blocks into a vector
    let mut blocks = Vec::new();
    for block_result in blocks_results {
        match block_result {
            Ok(Some(block)) => {
                blocks.push(block);
            }
            Ok(None) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }

    // Spoof extra blocks to stress test the volatility calculation
    let blocks = add_synthetic_blocks(10_000, blocks.clone());

    println!(
        "Fetched {} blocks from [{} to {}]",
        blocks.len(),
        blocks.first().unwrap().number.unwrap(),
        blocks.last().unwrap().number.unwrap(),
    );
    Ok(blocks)
}

// Helper to print the block found in the binary search
fn print_block_found(ref block: &Block<TxHash>) {
    println!(
        "Found block #{} with timestamp: {}",
        block.number.unwrap(),
        block.timestamp
    );
}

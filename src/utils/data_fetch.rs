use crate::utils::helpers::{add_synthetic_blocks, print_block_found};
use ethers::prelude::*;
use ethers::types::{BlockNumber, TxHash};
use futures::future::join_all;
use std::sync::Arc;

const SYNTHETIC_BLOCKS_TO_ADD: u64 = 1_000;

// Gets an array of blocks between from_block and to_block (both inclusive)
pub async fn fetch_blocks(
    from_block: u64,
    to_block: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Vec<Block<TxHash>>, ProviderError> {
    println!("Fetching blocks from {} to {}...\n", from_block, to_block);

    // Fetch blocks concurrently
    let blocks_results = join_all((from_block..=to_block).map(|block_number| {
        let provider = provider.clone();
        async move { provider.get_block(block_number).await }
    }))
    .await;

    // Collect valid blocks, error out on any fetch failure
    let mut blocks = Vec::new();
    for block_result in blocks_results {
        match block_result {
            Ok(Some(block)) => {
                blocks.push(block);
            }
            Ok(None) => {
                continue;
            }
            Err(e) => {
                // Return the error if any fetch fails
                return Err(e);
            }
        }
    }

    // Spoof extra blocks to stress test the volatility calculation
    Ok(add_synthetic_blocks(SYNTHETIC_BLOCKS_TO_ADD, blocks))
}

// Find the block closest to the given timestamp, panics if the timestamp is in the future (> 1 block)
pub async fn get_closest_block(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Block<TxHash>, ProviderError> {
    println!("Finding block closest to timestamp: {}...\n", timestamp);
    // Fetch latest block information
    let latest_block = provider.get_block(BlockNumber::Latest).await?.unwrap();
    let latest_block_number = latest_block.number.unwrap().as_u64();
    let latest_block_timestamp = latest_block.timestamp.as_u64();

    // Handle future requests
    if timestamp >= latest_block_timestamp + 13 {
        panic!("ERROR: Requesting job for the future");
    }

    // Binary search for the block closest to the given timestamp
    let (mut lower, mut upper) = (0, latest_block_number);
    while lower <= upper {
        let mid = (lower + upper) / 2;
        let mid_block = provider.get_block(mid).await?.unwrap();
        let mid_timestamp = mid_block.timestamp.as_u64();

        if mid_timestamp == timestamp {
            print_block_found(&mid_block);
            return Ok(mid_block);
        } else if mid_timestamp < timestamp {
            lower = mid + 1;
        } else {
            upper = mid.saturating_sub(1);
        }
    }

    // Return the last block with timestamp <= `timestamp`
    let block = provider.get_block(upper).await?.unwrap();
    print_block_found(&block);
    Ok(block)
}

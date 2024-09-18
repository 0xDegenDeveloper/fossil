use crate::utils::helpers::{add_synthetic_blocks, print_block_found};
use ethers::prelude::*;
use ethers::types::{BlockNumber, TxHash};
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Gets an array of blocks between from_block and to_block (both inclusive)

pub async fn fetch_blocks(
    from_timestamp: u64,
    to_timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Vec<Block<TxHash>>, ProviderError> {
    // Find the closest blocks to the given timestamps
    let to_block_result = get_closest_block_below(to_timestamp, provider.clone()).await?;
    let from_block_result = get_closest_block_above(from_timestamp, provider.clone()).await?;

    let (from_block, to_block) = (
        from_block_result.number.unwrap().as_u64(),
        to_block_result.number.unwrap().as_u64(),
    );

    println!("Fetching blocks from {} to {}...\n", from_block, to_block);

    let mut blocks = Vec::new();
    let mut current_block = from_block;
    let batch_size = 3;
    let delay_between_batches = 1000;

    // Fetch blocks in batches
    while current_block <= to_block {
        let end_block = (current_block + batch_size as u64).min(to_block); // Define the range of the batch

        // Fetch the current batch of blocks concurrently
        let batch_results = join_all((current_block..=end_block).map(|block_number| {
            let provider = provider.clone();
            async move { provider.get_block(block_number).await }
        }))
        .await;

        // Collect valid blocks from the batch
        for block_result in batch_results {
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

        // Move to the next batch of blocks
        current_block = end_block + 1;

        // Introduce a delay between batches
        if current_block <= to_block {
            println!("...\n",);
            sleep(Duration::from_millis(delay_between_batches)).await;
        }
    }

    // Spoof extra blocks to stress test the volatility calculation
    add_synthetic_blocks(&mut blocks);
    Ok(blocks)
}

pub async fn get_closest_block_below(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Block<TxHash>, ProviderError> {
    println!("Finding block closest below timestamp: {}...\n", timestamp);

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
            lower = mid + 1; // Move search to upper half
        } else {
            upper = mid.saturating_sub(1); // Move search to lower half
        }
    }

    // Return the last block with timestamp <= `timestamp` (closest below)
    let block = provider.get_block(upper).await?.unwrap();
    print_block_found(&block);
    Ok(block)
}

pub async fn get_closest_block_above(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Block<TxHash>, ProviderError> {
    println!("Finding block closest above timestamp: {}...\n", timestamp);

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
        } else if mid_timestamp > timestamp {
            upper = mid.saturating_sub(1); // Move search to lower half
        } else {
            lower = mid + 1; // Move search to upper half
        }
    }

    // Return the first block with timestamp >= `timestamp` (closest above)
    let block = provider.get_block(lower).await?.unwrap();
    print_block_found(&block);
    Ok(block)
}

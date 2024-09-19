use crate::utils::helpers::add_synthetic_blocks;
use dotenv::dotenv;
use ethers::prelude::*;
use ethers::types::{BlockNumber, TxHash};
use futures::future::join_all;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Getting Blocks ///

// Gets an array of blocks between from_block and to_block (both inclusive)
pub async fn fetch_blocks(
    from_timestamp: u64,
    to_timestamp: u64,
) -> Result<Vec<Block<TxHash>>, ProviderError> {
    // Find the blocks closest to the given timestamps without going under/over
    println!("- Finding blocks closest to range...\n");

    let from_block = get_closest_block_above(from_timestamp).await?;
    let from_block_number = from_block.number.unwrap().as_u64();
    let from_block_timestamp = from_block.timestamp.as_u64();

    let to_block = get_closest_block_below(to_timestamp).await?;
    let to_block_number = to_block.number.unwrap().as_u64();
    let to_block_timestamp = to_block.timestamp.as_u64();

    println!(
        "\trange: \t [{}, {}]\n\tbecomes: [{}, {}]\n",
        from_timestamp, to_timestamp, from_block_timestamp, to_block_timestamp
    );

    // Fetch blocks in batches to avoid rate limits
    println!("- Fetching blocks...\n");
    let mut blocks: Vec<Block<TxHash>> = Vec::new();
    let mut current_block_number = from_block_number;
    let batch_size = 5;
    let delay_between_batches = 1000;
    while current_block_number <= to_block_number {
        // Define the range of the batch
        let end_block_number = (current_block_number + batch_size).min(to_block_number);

        // Make batch request
        let batch_results = join_all((current_block_number..=end_block_number).map(
            |block_number| {
                let provider = get_provider();
                async move { provider.get_block(block_number).await }
            },
        ))
        .await;

        // Collect valid blocks from the results
        for block_result in batch_results {
            match block_result {
                Ok(Some(block)) => {
                    blocks.push(block);
                }
                Ok(None) => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Pause between batches to avoid rate limits
        if current_block_number <= to_block_number {
            println!("\t...\n",);
            sleep(Duration::from_millis(delay_between_batches)).await;
        }

        // Move to the next batch
        current_block_number = end_block_number + 1;
    }

    // Add extra blocks for stress testing the volatility calculation
    add_synthetic_blocks(&mut blocks);

    Ok(blocks)
}

/// Helpers ///

// Enum to specify whether to find the closest block above or below the given timestamp
pub enum BlockDirection {
    Above,
    Below,
}

// Gets the latest block below the given timestamp
pub async fn get_closest_block_below(timestamp: u64) -> Result<Block<TxHash>, ProviderError> {
    get_block_closest_to_timestamp(timestamp, BlockDirection::Below).await
}

// Gets the earliest block above the given timestamp
pub async fn get_closest_block_above(timestamp: u64) -> Result<Block<TxHash>, ProviderError> {
    get_block_closest_to_timestamp(timestamp, BlockDirection::Above).await
}

// Gets the block closest to the given timestamp, the direction parameter
// specifies whether to return the block above or below the timestamp
pub async fn get_block_closest_to_timestamp(
    timestamp: u64,
    direction: BlockDirection,
) -> Result<Block<TxHash>, ProviderError> {
    println!("\t...\n");

    // Fetch latest block information
    let (latest_block_number, latest_block_timestamp) =
        get_latest_block_number_and_timestamp().await;

    // Handle future requests
    if timestamp > latest_block_timestamp {
        panic!("ERROR: Requesting job for the future");
    }

    // Binary search for the block closest to the given timestamp
    let (mut lower, mut upper) = (1, latest_block_number);
    while lower <= upper {
        let mid = (lower + upper) / 2;
        let mid_block = get_block_by_number(mid).await?;
        let mid_timestamp = mid_block.timestamp.as_u64();

        // Found block with exact timestamp
        if mid_timestamp == timestamp {
            return Ok(mid_block);
        }
        // Move search to upper half
        else if timestamp > mid_timestamp {
            lower = mid + 1;
        }
        // Move search to lower half
        else {
            upper = mid.saturating_sub(1); // Avoid sub overflow, return 0 if mid == 0
        }
    }

    // Return the correct block closest to `timestamp`
    let closest_block_number = match direction {
        // The earliest block with timestamp t >= given timestamp
        BlockDirection::Above => lower,

        // The latest block with timestamp t <= given timestamp
        BlockDirection::Below => upper,
    };

    get_block_by_number(closest_block_number).await
}

/// RPC ///

// Get RPC provider
fn get_provider() -> Arc<Provider<Http>> {
    // Get RPC URL
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    // Setup provider
    Arc::new(Provider::<Http>::try_from(rpc_url).expect("Failed to create provider"))
}

// Get a block by its number
async fn get_block_by_number(block_number: u64) -> Result<Block<TxHash>, ProviderError> {
    let provider = get_provider();
    let block = provider.get_block(block_number).await?.unwrap();
    Ok(block)
}

// Get the latest block's number and timestamp
async fn get_latest_block_number_and_timestamp() -> (u64, u64) {
    let provider = get_provider();
    let latest_block = provider
        .get_block(BlockNumber::Latest)
        .await
        .unwrap()
        .unwrap();

    let latest_block_number = latest_block.number.unwrap().as_u64();
    let latest_block_timestamp = latest_block.timestamp.as_u64();

    (latest_block_number, latest_block_timestamp)
}

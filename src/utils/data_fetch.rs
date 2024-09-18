use crate::utils::helpers::{add_synthetic_blocks, print_block_found};
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
    // Get the RPC provider
    let provider: Arc<Provider<Http>> = get_provider();

    // Find the blocks closest to the given timestamps
    println!("- Finding blocks closest to range...\n");

    let from_block = get_closest_block_above(from_timestamp, provider.clone()).await?;
    let from_block_number = from_block.number.unwrap().as_u64();
    let from_block_timestamp = from_block.timestamp.as_u64();

    let to_block = get_closest_block_below(to_timestamp, provider.clone()).await?;
    let to_block_number = to_block.number.unwrap().as_u64();
    let to_block_timestamp = to_block.timestamp.as_u64();

    println!(
        "\trange: \t [{}, {}],\n\tbecomes: [{}, {}]\n",
        from_timestamp, to_timestamp, from_block_timestamp, to_block_timestamp
    );

    // Fetch blocks in batches
    println!("- Fetching blocks...\n");
    let mut blocks: Vec<Block<TxHash>> = Vec::new();
    let mut current_block_number = from_block_number;
    let batch_size = 3;
    let delay_between_batches = 1000;
    while current_block_number <= to_block_number {
        let end_block_number = (current_block_number + batch_size).min(to_block_number); // Define the range of the batch

        // Create batch request
        let batch_results = join_all((current_block_number..=end_block_number).map(
            |block_number| {
                let provider = provider.clone();
                async move { provider.get_block(block_number).await }
            },
        ))
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
                    return Err(e);
                }
            }
        }

        // Delay between batches to avoid rate limits
        if current_block_number <= to_block_number {
            println!("\t...\n",);
            sleep(Duration::from_millis(delay_between_batches)).await;
        }

        // Move to the next batch
        current_block_number = end_block_number + 1;
    }

    // Spoof extra blocks to stress test the volatility calculation
    add_synthetic_blocks(&mut blocks);

    Ok(blocks)
}

/// Helpers ///

pub enum BlockDirection {
    Above,
    Below,
}

// Gets the latest block below the given timestamp
pub async fn get_closest_block_below(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Block<TxHash>, ProviderError> {
    println!("\t...\n");
    get_block_closest_to_timestamp(timestamp, provider, BlockDirection::Below).await
}

// Gets the earliest block above the given timestamp
pub async fn get_closest_block_above(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
) -> Result<Block<TxHash>, ProviderError> {
    println!("\t...\n");
    get_block_closest_to_timestamp(timestamp, provider, BlockDirection::Above).await
}

// Gets the block closest to the given timestamp, the direction parameter
// specifies whether to return the block above or below the timestamp
pub async fn get_block_closest_to_timestamp(
    timestamp: u64,
    provider: Arc<Provider<Http>>,
    direction: BlockDirection,
) -> Result<Block<TxHash>, ProviderError> {
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
    let direction = match direction {
        // The earliest block with timestamp t >= given timestamp
        BlockDirection::Above => lower,
        // The latest block with timestamp t <= given timestamp
        BlockDirection::Below => upper,
    };

    let block = provider.get_block(direction).await?.unwrap();
    print_block_found(&block);
    Ok(block)
}

/// RPC ///

// Get ethers.rs provider
fn get_provider() -> Arc<Provider<Http>> {
    // Load env variables
    dotenv().ok();

    // Setup provider
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    Arc::new(Provider::<Http>::try_from(rpc_url).expect("Failed to create provider"))
}

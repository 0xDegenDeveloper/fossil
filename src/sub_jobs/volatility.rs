use crate::utils::data_fetch::{fetch_blocks, get_closest_block};
use ethers::prelude::*;
use std::sync::Arc;

use crate::utils::conversion::wei_to_gwei;

pub async fn calculate_volatility(
    timestamp: u64,
    block_number_range: u64,
    provider: Arc<Provider<Http>>,
) -> Result<f64, ProviderError> {
    // Fetch block closest to this timestamp
    match get_closest_block(timestamp, provider.clone()).await {
        Ok(block) => {
            // Calculate block number range
            let to_block = block.number.unwrap().as_u64();
            let from_block = if to_block >= block_number_range {
                to_block - block_number_range
            } else {
                0
            };

            // Fetch blocks in the range
            let blocks = fetch_blocks(from_block, to_block, provider.clone()).await?;

            // If there are less than 2 blocks, then we cannot calculate returns
            if blocks.len() < 2 {
                return Ok(0.0);
            }

            // Calculate returns
            let mut returns: Vec<f64> = Vec::new();
            for i in 1..blocks.len() {
                let basefee_current = wei_to_gwei(blocks[i].base_fee_per_gas.unwrap_or_default());
                let basefee_previous =
                    wei_to_gwei(blocks[i - 1].base_fee_per_gas.unwrap_or_default());

                if basefee_previous == 0.0 {
                    continue;
                }

                let return_i = (basefee_current / basefee_previous).ln();
                returns.push(return_i);
            }

            // If there are no returns, return 0.0
            if returns.is_empty() {
                return Ok(0.0);
            }

            // Calculate mean return
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

            // Calculate variance
            let variance = returns
                .iter()
                .map(|&r| (r - mean_return).powi(2))
                .sum::<f64>()
                / returns.len() as f64;

            // Calculate volatility
            let volatility = variance.sqrt();

            Ok(volatility)
        }
        Err(e) => {
            eprintln!("Error fetching block: {}", e);
            Ok(0.0)
        }
    }
}

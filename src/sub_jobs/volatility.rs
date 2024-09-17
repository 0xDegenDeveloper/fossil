use crate::utils::data_fetch::{fetch_blocks, get_closest_block};
use ethers::prelude::*;
use std::sync::Arc;

// Returns BPS (i.e., 5001 == 50.01% VOL)
pub async fn calculate_volatility(
    timestamp: u64,
    block_number_range: u64,
    provider: Arc<Provider<Http>>,
) -> Result<u128, ProviderError> {
    if let Ok(block) = get_closest_block(timestamp, provider.clone()).await {
        // Fetch blocks in the range
        let to_block = block.number.unwrap().as_u64();
        let from_block = to_block.saturating_sub(block_number_range);
        let blocks = fetch_blocks(from_block, to_block, provider.clone()).await?;

        // If there are less than 2 blocks, we cannot calculate returns
        if blocks.len() < 2 {
            return Ok(0);
        }

        // Calculate log returns
        let mut returns: Vec<f64> = Vec::new();
        for i in 1..blocks.len() {
            if let (Some(basefee_current), Some(basefee_previous)) =
                (blocks[i].base_fee_per_gas, blocks[i - 1].base_fee_per_gas)
            {
                if basefee_previous.is_zero() {
                    continue;
                }

                let basefee_current_f64 = basefee_current.as_u128() as f64;
                let basefee_previous_f64 = basefee_previous.as_u128() as f64;

                let return_i = (basefee_current_f64 / basefee_previous_f64).ln();
                returns.push(return_i);
            }
        }

        // If there are no returns, return 0
        if returns.is_empty() {
            return Ok(0);
        }

        // Calculate average return
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;

        // Calculate volatility (standard deviation) as a BPS integer
        let volatility_bps = (variance.sqrt() * 10_000.0).round() as u128;
        Ok(volatility_bps)
    } else {
        eprintln!("Error fetching block closest to t= {}", timestamp);
        Ok(0)
    }
}

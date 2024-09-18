use crate::utils::data_fetch::fetch_blocks;

use ethers::prelude::*;

// Volatility is the standard deviation of the variance
// Variaance is the average squared difference between each log return and the mean
// Log return is the natural logarithm of the ratio of the current base fee to the previous base fee
// Returns BPS (i.e., 5001 == 50.01% VOL)
pub async fn calculate_volatility(
    from_timestamp: u64,
    to_timestamp: u64,
) -> Result<u128, ProviderError> {
    // Fetch blocks
    let blocks = fetch_blocks(from_timestamp, to_timestamp).await?;

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

            returns.push((basefee_current_f64 / basefee_previous_f64).ln());
        }
    }

    // If there are no returns the volatility is 0
    if returns.is_empty() {
        return Ok(0);
    }

    // Calculate average log returns
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

    // Calculate variance
    let variance = returns
        .iter()
        .map(|&r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;

    // Calculate volatility (standard deviation) as a BPS integer
    Ok((variance.sqrt() * 10_000.0).round() as u128)
}

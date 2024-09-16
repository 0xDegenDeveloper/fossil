use crate::utils::conversion::{gwei_to_wei, wei_to_gwei};
use ethers::core::types::{Block, U256, U64};
use ethers::types::{BlockNumber, TxHash};
use rand::Rng;

// Add synthetic blocks for stress testing
// @note Each synthetic block's timestamp is 12 seconds after the previous block
// @note Each synthetic block's base_fee_per_gas is between -12.5% and +12.5% of the previous block's (random)
pub fn add_synthetic_blocks(amount: u64, mut blocks: Vec<Block<TxHash>>) -> Vec<Block<TxHash>> {
    for i in 0..amount {
        if let Some(mut last_block) = blocks.last().cloned() {
            // Increment the block number
            // @note If there is no previous block number, set this one's to 1
            if let Some(number) = last_block.number {
                last_block.number = Some(number + U64::one());
            } else {
                last_block.number = Some(U64::one());
            }

            // Increment the timestamp by 12 seconds
            last_block.timestamp = last_block.timestamp + U256::from(12u64);

            // Update the base fee per gas
            // @note If there is no previous base fee, set this one's to 100 gwei
            if let Some(base_fee) = last_block.base_fee_per_gas {
                // Generate a random percentage change between -12.5% and +12.5%
                let mut rng = rand::thread_rng();
                let percentage_change: f64 = rng.gen_range(-0.125..=0.125); // Inclusive range

                // @note Converts wei to gwei to compute, then converts back to wei
                // @note This is done to avoid overflow and precisions issues
                let base_fee_gwei = wei_to_gwei(base_fee);
                let new_base_fee_gwei = base_fee_gwei * (1.0 + percentage_change);
                let new_base_fee_wei = gwei_to_wei(new_base_fee_gwei);

                // Ensure base fee is not 0, if it is, reset to 15 gwei
                if new_base_fee_wei.is_zero() {
                    last_block.base_fee_per_gas =
                        Some(U256::from(15_u128 * 1_000_000_000_u128 * 1_000_000_000_u128));
                } else {
                    last_block.base_fee_per_gas = Some(new_base_fee_wei);
                }
            } else {
                // Set initial base fee per gas to 100 gwei
                let initial_base_fee_wei = 100u128 * 1_000_000_000u128; // 100 gwei in wei
                last_block.base_fee_per_gas = Some(U256::from(initial_base_fee_wei));
            }
            blocks.push(last_block);
        } else {
            println!("No blocks fetched. Cannot generate synthetic blocks.");
        }
    }
    blocks
}

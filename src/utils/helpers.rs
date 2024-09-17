use crate::utils::conversion::to_gwei;
use ethers::core::types::{Block, TxHash, U256, U64};
use rand::Rng;

// Helper to print the block found in the binary search
pub fn print_block_found(ref block: &Block<TxHash>) {
    println!(
        "> found block with timestamp:       {} (#{})\n",
        block.timestamp,
        block.number.unwrap(),
    );
}

// Add synthetic blocks for stress testing, if no blocks are provided, a spoof genesis block is generated
pub fn add_synthetic_blocks(amount: u64, mut blocks: Vec<Block<TxHash>>) -> Vec<Block<TxHash>> {
    println!("Adding {} synthetic blocks...\n", amount);

    // Generate a spoof genesis block if necessary
    if blocks.is_empty() || blocks.last().unwrap().number.is_none() {
        blocks.push(Block {
            number: Some(U64::one()),
            timestamp: U256::from(123456789),
            base_fee_per_gas: Some(U256::from(to_gwei(10.0))),
            ..Default::default()
        });
    }

    let mut last_block = blocks.last().unwrap().clone();
    let mut random = rand::thread_rng();

    for _ in 0..amount {
        // Increment block number and timestamp
        last_block.number = Some(last_block.number.unwrap() + U64::one());
        last_block.timestamp += U256::from(12);

        // Randomly change the base fee +/- 12.5%
        let percentage_change = 1.0 + random.gen_range(-0.125..=0.125);
        let new_base_fee = U256::from(
            (last_block.base_fee_per_gas.unwrap().as_u64() as f64 * percentage_change) as u64,
        );

        // Ensure base fee does not drop below 0.01 gwei, reset to 5 gwei if it does
        last_block.base_fee_per_gas = Some(if new_base_fee < to_gwei(0.01) {
            U256::from(to_gwei(5.0))
        } else {
            new_base_fee
        });

        blocks.push(last_block.clone());
    }

    blocks
}

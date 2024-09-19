use crate::utils::conversion::to_gwei;
use ethers::core::types::{Block, TxHash, U256, U64};
use rand::Rng;

/// ADDING SYNTHETIC BLOCKS ///

const SYNTHETIC_BLOCKS_TO_ADD: u64 = 100;

// Define the types of base fee changes
pub enum BaseFeeChange {
    Random,
    //Custom,
}

// Update a block's base fee
fn update_base_fee(mut block: &mut Block<TxHash>, change_type: BaseFeeChange) {
    match change_type {
        BaseFeeChange::Random => _random_change(&mut block),
        //BaseFeeChange::Custom => _custom_increase(&mut block),
    }
}

fn _custom_change(_block: &mut Block<TxHash>) {
    // < ... > //
}

// Change a block's base fee by a random percentage between -12.5% and +12.5%
fn _random_change(block: &mut Block<TxHash>) {
    if let Some(base_fee) = block.base_fee_per_gas {
        let mut random = rand::thread_rng();
        let percentage_change = 1.0 + random.gen_range(-0.125..=0.125);
        let new_base_fee = U256::from((base_fee.as_u64() as f64 * percentage_change) as u64);

        // Ensure base fee does not trail to 0 gwei
        if new_base_fee < to_gwei(0.01) {
            println!("\t> WARNING: Resetting base fee to 10 gwei\n");
            block.base_fee_per_gas = Some(U256::from(to_gwei(10.0)));
        } else {
            block.base_fee_per_gas = Some(new_base_fee);
        }
    }
}

// Add synthetic blocks to the provided vector
pub fn add_synthetic_blocks(blocks: &mut Vec<Block<TxHash>>) {
    println!("- Adding {} synthetic blocks...\n", SYNTHETIC_BLOCKS_TO_ADD);

    // Generate a spoof genesis block if necessary
    if blocks.is_empty()
        || blocks.last().unwrap().number.is_none()
        || blocks.last().unwrap().base_fee_per_gas.is_none()
    {
        println!("\t> Generating a spoof genesis block...\n");
        blocks.push(Block::<TxHash> {
            number: Some(U64::one()),
            timestamp: U256::from(123456789),
            base_fee_per_gas: Some(U256::from(to_gwei(10.0))),
            ..Default::default()
        });
    }

    // Spoof extra blocks
    let mut last_block = blocks.last().unwrap().clone();
    for _ in 0..SYNTHETIC_BLOCKS_TO_ADD {
        // Increment block number and timestamp
        last_block.number = Some(last_block.number.unwrap() + U64::one());
        last_block.timestamp += U256::from(12);

        // Update base fee
        update_base_fee(&mut last_block, BaseFeeChange::Random);

        blocks.push(last_block.clone());
    }
}

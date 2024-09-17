use ethers::core::types::U256;

pub fn to_gwei(amount: f64) -> U256 {
    U256::from((amount * 1e9) as u64)
}

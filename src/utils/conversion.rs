use ethers::core::types::U256;

pub fn wei_to_gwei(wei: U256) -> f64 {
    match wei.is_zero() {
        true => 0.0,
        false => wei.as_u128() as f64 / 1e9,
    }
}

pub fn gwei_to_wei(gwei: f64) -> U256 {
    U256::from((gwei * 1e9) as u128)
}

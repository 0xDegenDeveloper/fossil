use dotenv::dotenv;
use ethers::prelude::*;
use std::env;
use std::sync::Arc;
mod sub_jobs;
mod utils;

#[tokio::main]
async fn main() {
    println!("\n/// VOLATILITY ///\n");

    // Load environment variables
    dotenv().ok();
    let infura_project_id = env::var("INFURA_PROJECT_ID").expect("INFURA_PROJECT_ID must be set");

    // Setup provider
    let provider = Arc::new(
        Provider::<Http>::try_from(format!(
            "https://mainnet.infura.io/v3/{}",
            infura_project_id
        ))
        .expect("Failed to create provider"),
    );

    // Mock a job request
    let (timestamp, block_number_range) = (1726514877, 11);

    // Call async function directly with `.await`
    match sub_jobs::volatility::calculate_volatility(timestamp, block_number_range, provider).await
    {
        Ok(vol) => println!("> VOL = {:.4}% ({} as u128)\n", vol as f32 / 10_000.0, vol),
        Err(e) => eprintln!(
            "Error calculating volatility for timestamp {}: {}",
            timestamp, e
        ),
    }
}

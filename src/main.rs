use dotenv::dotenv;
use ethers::prelude::*;
use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;
mod sub_jobs;
mod utils;

fn main() {
    // Load environment variables
    dotenv().ok();
    let infura_project_id = env::var("INFURA_PROJECT_ID").expect("INFURA_PROJECT_ID must be set");

    // Setup provider
    let provider_url = format!("https://mainnet.infura.io/v3/{}", infura_project_id);
    let provider = Provider::<Http>::try_from(provider_url).unwrap();
    let provider = Arc::new(provider);

    // Mock job request
    let timestamp = 1726514877; // ...67
    let block_num_range = 11;

    // Fetch block closest to this number
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        match sub_jobs::volatility::calculate_volatility(
            timestamp,
            block_num_range,
            provider.clone(),
        )
        .await
        {
            Ok(vol) => {
                println!("Volatility: {}", vol);
            }
            Err(e) => {
                eprintln!(
                    "Error calculating volatility for timestamp {}: {}",
                    timestamp, e
                );
            }
        }
    });
}

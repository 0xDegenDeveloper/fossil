use dotenv::dotenv;
use ethers::prelude::*;
use std::env;
use std::sync::Arc;
mod sub_jobs;
mod utils;

#[tokio::main]
async fn main() {
    println!("\n/// VOLATILITY ///\n");

    // Mock a job request for "now" spanning 5 blocks
    let now = 1726514877;
    let range = 12 * 5;
    let (from_timestamp, to_timestamp) = (now - range, now);

    // Load environment variables
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    // Setup provider
    let provider =
        Arc::new(Provider::<Http>::try_from(rpc_url).expect("Failed to create provider"));

    // Call async function directly with `.await`
    match sub_jobs::volatility::calculate_volatility(from_timestamp, to_timestamp, provider).await {
        Ok(vol) => println!("> VOL = {:.4}% ({} as u128)\n", vol as f32 / 10_000.0, vol),
        Err(e) => eprintln!(
            "Error calculating volatility over timestamps: [{}, {}]\n\nErr: {}",
            from_timestamp, to_timestamp, e
        ),
    }
}

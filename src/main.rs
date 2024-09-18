use fossil::sub_jobs::volatility::calculate_volatility;

#[tokio::main]
async fn main() {
    // Calculate volatility over 5 blocks (+ 1000 synthetic)
    let to_timestamp = 1726514877;
    let from_timestamp = to_timestamp - (12 * 5); // 5 blocks back in time

    println!("\n---------------VOLATILITY---------------\n");

    //let vol = sub_jobs::volatility::calculate_volatility(from_timestamp, to_timestamp)
    let vol = calculate_volatility(from_timestamp, to_timestamp)
        .await
        .expect("Error calculating volatility");

    println!("----------------------------------------");
    println!("> VOL = {:.4}%, {} as u128", vol as f32 / 10_000.0, vol);
    println!("----------------------------------------");
}

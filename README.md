# Initial Code for Volatility Calculations and Finding Closest Block to a Given Timestamp

## How to Run the Volatility Sub-Job

1. Clone the repository:

```
git clone https://github.com/0xDegenDeveloper/fossil fossil && cd fossil
```

2. Copy `.example.env` into `.env`, adding your infura API key (for ETH mainnet).

3. Install dependencies:

```
cargo build
```

4. Run the sub-job:

```
cargo run
```

# Notes

- We pass in a `timestamp` and `block_number_range` into the `calculate_volatility()` function, located in the [volatility.rs file](./src/sub_jobs/volatility.rs). We use the `get_closest_block(timestamp)` function, located in the [data_fetch.rs file](./src/utils/data_fetch.rs) to calculate the `to_block`. We subtract the `block_number_range` from the `to_block` to calculate the `from_block`. These are the (inclusive) bounds for blocks to fetch data from. Next, this function passes the `from_timestamp` & `to_timestamp` into the `fetch_blocks()` function (also located in the [data_fetch.rs file](./src/utils/data_fetch.rs)).

  - To stress test the logic of the volatility calculation, at the end of the `fetch_blocks()` function, we call the `add_synthetic_blocks()` function in the [helpers.rs file](.src/utils/helpers.rs). This function adds an `amount` of blocks to the block array, with each new block's timestamp 12 seconds after the previous's, and its base fee a random % change from the previous's.

- Once the blocks are fetched, we use their base fees to calculate the volatility.

> Note: Confirm with Finn if calculation is correct. We only need `block.basefee`s ? No `block.timestamp`s for the calculation?

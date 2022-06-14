# Toy Payment Engine
A Toy Payment Engine Written in Rust.  It takes in a series of deposits and transactions and will output final account balances. As well as state of transactions.

## Usage
To directly use the toy app, download the binary from download and an example file from zip and run 
```bash
cargo run -- transactions.csv > accounts.csv

# This follows the format of
# cargo run -- {inputfile}.csv > {outputfile}.csv
```

Files with spaces in their names will not be accepted, and input & output file are delineated by ">"

At the moment output goes to terminal.  If you want to output to file add a `--f` flag before files

Alternatively you can download the repo and run the following from repo root
```bash
cargo build --release
cp src/test/inputs 
cd target/release/
cargo run release -- transactions.csv > accounts.csv

# If you wish to have a cs
cargo run release --f -- transactions.csv > accounts.csv
```

## Testing
Unit tests were made with rusts built in testing.  To run unit tests run `cargo run test`

## Usage
Documentation was made using rust's built in documentation tools

To build & open documents run `cargo run build docs`

## TODO
- Implement amount & transaction math with [rust_decimal](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html) crate 
- Implement parallelization strategy by breaking transactions via account.  The only sequential calculation is required per account, and there are no listed cross account calculations.  Should be able to split up work queues based on account.
- Implement Account & Transaction Hashmap tracking with pointers instead of indices
- Store Account & Transaction in Database for true scalability

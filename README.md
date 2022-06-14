# Toy Payment Engine
A Toy Payment Engine Written in Rust.  It takes in a series of deposits and transactions and will output final account balances. As well as state of transactions.

<p align="center">
  <img src="assets/toypaymentengine.jpeg">
</p>

## Usage
To directly use the toy app, download the binary from download and an example file from zip and run 
```bash
cargo run -- transactions.csv > accounts.csv

# This follows the format of
# cargo run -- {inputfile}.csv > {outputfile}.csv
```

## Testing
Unit tests were made with rusts built in testing.  To run unit tests run 
```
cargo test
```

## Documentation
Documentation was made using rust's built in documentation tools

To build & open documents run 
```
cargo run build docs
```
## Next Steps
- For scalability store Account & Transaction in Database for true scalability.  Use postgres and an orm like [Diesel](https://diesel.rs/) or [SeaORM](https://www.sea-ql.org/SeaORM/)
- Move the Account & Transaction ID hashmap lookup to a key value store.  Use redis and [redis-rs](https://github.com/redis-rs/redis-rs)
- Use [tokio](https://tokio.rs/) to speed up access to above read write to added cache & db
- Implement amount & transaction math with [rust_decimal](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html) crate 


## Notes On Efficiency
- Focus on this implementation was on correctness and speed.  
- It was assumed that transaction id's must be globally unique.  
- New transactions must then check global transaction state for validity and then update global state once written.
- All lookups, insertions, & mutations are O(1) so a sequential read write process is pretty efficient.  Memory usage increased to enable speedup. Generally memory is cheaper that compute.
- From the assignment instructions it was unclear if additional processes like a db or cache could be spawned in the running of the program, so parallelized io with tokio was not used.

## Q & A
### Correctness
*For the cases you are handling are you handling them correctly?*
- They are being handled correctly conceptually.  In practice floating point errors could pop up across repeated transactions or when introducing something like interest calculations on account balances.  With more time I would implement the amount values as [rust_decimal](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html)

*Did you write unit tests for the complicated bits? Or are you using the type system to ensure correctness?*
- TDD was employed so unit tests were heavily relied on.  
- Error enums were made so failure cases can be explicitly tested for.  
- The Rust Type system was used for processing raw inputs to their relevant transaction type.  This added safety and reduced memory footprint.

### Efficiency
*Can you stream values through memory as opposed to loading the entire data set upfront?*
- Streaming was implemented but the process is memory limited see [Notes on Efficiency](#notes-on-efficiency).  If the process is hitting memory issues I would take actions listed in [Next Steps](#next-steps)


*What if your code was bundled in a server, and these CSVs came from thousands of concurrent TCP streams?*
- I would update the code to interact with db, cache, and use tokio as discussed in [Next Steps](#next-steps)

mod account;
mod cli_io;
mod constants;
mod payments_engine;
mod test;
mod transaction;

fn main() {
    let mut payment_engine = payments_engine::PaymentsEngine::new();
    payment_engine.streaming_execute_cli();
}

mod account;
mod cli_io;
mod constants;
mod payments_engine;
mod test;
mod transaction;

fn main() {
    let mut pay_engine = payments_engine::PaymentsEngine::new();
    pay_engine.execute_cli();
}

use crate::account::Account;
use crate::transaction::Transaction;
use std::collections::HashMap;
mod batch_execute;
mod stream_process;
mod transactions;

#[derive(Debug)]
pub struct PaymentsEngine {
    /// List of accounts in order of their creation
    pub accounts: Vec<Account>,
    /// Utility to provide O(1) lookup speed for account Id's
    /// In real scenario would want to check on DB or REDIS client
    acnt_map: HashMap<u16, usize>,

    /// List of accepted transactions in order of their creation
    /// Assignment does not require tracking RefTxn's,
    /// but cool because you can confirm account state from transaction history ¯\_(ツ)_/¯
    /// For a payment engine would want an ACID DB
    processed_txns: Vec<Transaction>,
    /// Utility to provide O(1) lookup speed for account Id's
    /// Will only point to pure transactions as ref txn's aren't given identifiers
    /// In real scenario would want to check on DB or REDIS client
    txn_map: HashMap<u32, usize>,
}

impl PaymentsEngine {
    pub fn new() -> Self {
        Self {
            accounts: vec![],
            acnt_map: HashMap::new(),
            processed_txns: vec![],
            txn_map: HashMap::new(),
        }
    }
}

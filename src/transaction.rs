/// Financial transactions which can affect an accounts held & available amounts
#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Deposit(PureTxn),
    Withdrawal(PureTxn),
    Dispute(RefTxn),
    Resolve(RefTxn),
    Chargeback(RefTxn),
}

/// A transaction which adds or removes an amount
#[derive(Debug, Clone, PartialEq)]
pub struct PureTxn {
    pub txn_id: u32,
    pub acnt_id: u16,
    pub amount: f64,
    pub disputed: bool,
}

/// A transaction which references another transaction
#[derive(Debug, Clone, PartialEq)]
pub struct RefTxn {
    /// Transaction ID which a this transaction refers to, should only refer to pure transactions
    pub ref_id: u32,
    /// Account Id this transaction should affect, should align with the reference transaction
    pub acnt_id: u16,
}

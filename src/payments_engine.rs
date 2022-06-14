use crate::account::Account;
use crate::cli_io::{output_accounts, parse_cli, parse_txns_csv, CliOptions};
use crate::transaction::{PureTxn, RefTxn, Transaction};
use std::{collections::HashMap, io};

#[derive(Debug)]
pub struct PaymentsEngine {
    /// List of accounts in order of their creation
    accounts: Vec<Account>,
    /// Utility to provide O(1) lookup speed for account Id's
    acnt_map: HashMap<u16, usize>,

    /// List of accepted transactions in order of their creation
    /// Assignment does not require tracking RefTxn's,
    /// but cool because you can confirm account state from transaction history ¯\_(ツ)_/¯
    processed_txns: Vec<Transaction>,
    /// Utility to provide O(1) lookup speed for account Id's
    /// Will only point to pure transactions as ref txn's aren't given identifiers
    txn_map: HashMap<u32, usize>,
}

#[derive(PartialEq, Debug)]
enum TxnErrors {
    AccountDoesNotExist,
    AccountFrozen,
    AccountLacksFunds,
    TxnAlreadyDisputed,
    TxnIdAlreadyExists,
    TxnIdDoesNotExist,
    TxnMustBeDisputed,
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

    /// Takes input withdrawl txn and applies it if valid, else returns an error message
    fn process_deposit(&mut self, p_txn: &PureTxn) -> Result<(), TxnErrors> {
        if self.txn_map.get(&p_txn.txn_id).is_some() {
            return Err(TxnErrors::TxnIdAlreadyExists);
        }
        if let Some(acnt_indx) = self.acnt_map.get(&p_txn.acnt_id) {
            if self.accounts[*acnt_indx].frozen {
                return Err(TxnErrors::AccountFrozen);
            }
            self.accounts[*acnt_indx].available += p_txn.amount;
            self.processed_txns
                .push(Transaction::Deposit(p_txn.clone()));
            self.txn_map
                .insert(p_txn.txn_id, self.processed_txns.len() - 1);
        } else {
            let new_account = Account {
                id: p_txn.acnt_id,
                available: p_txn.amount,
                held: 0.0,
                frozen: false,
            };
            self.acnt_map.insert(new_account.id, self.accounts.len());
            self.accounts.push(new_account);
            self.processed_txns
                .push(Transaction::Deposit(p_txn.clone()));
            self.txn_map
                .insert(p_txn.txn_id, self.processed_txns.len() - 1);
        }

        Ok(())
    }

    /// Takes input withdrawl txn and applies it if valid, else returns an error message
    fn process_withdrawl(&mut self, p_txn: &PureTxn) -> Result<(), TxnErrors> {
        if self.txn_map.get(&p_txn.txn_id).is_some() {
            return Err(TxnErrors::TxnIdAlreadyExists);
        }
        if let Some(ii) = self.acnt_map.get(&p_txn.acnt_id) {
            if self.accounts[*ii].available < p_txn.amount {
                return Err(TxnErrors::AccountLacksFunds);
            }
            if self.accounts[*ii].frozen {
                return Err(TxnErrors::AccountFrozen);
            }
            self.accounts[*ii].available -= p_txn.amount;
            self.processed_txns
                .push(Transaction::Withdrawal(p_txn.clone()));
            self.txn_map
                .insert(p_txn.txn_id, self.processed_txns.len() - 1);
        } else {
            return Err(TxnErrors::AccountDoesNotExist);
        }
        Ok(())
    }

    // Returns Account & Transaction Indices or error string
    fn get_ref_txn_indicies(&self, ref_txn: &RefTxn) -> Result<(usize, usize), TxnErrors> {
        let acnt_indx = self.acnt_map.get(&ref_txn.acnt_id);
        if acnt_indx.is_none() {
            return Err(TxnErrors::AccountDoesNotExist);
        }
        let acnt_indx = acnt_indx.unwrap().clone();
        if self.accounts[acnt_indx].frozen {
            return Err(TxnErrors::AccountFrozen);
        }

        let txn_indx = self.txn_map.get(&ref_txn.ref_id);
        if txn_indx.is_none() {
            return Err(TxnErrors::TxnIdDoesNotExist);
        };
        Ok((acnt_indx, txn_indx.unwrap().clone()))
    }

    /// Takes input dispute txn and applies it if valid, else returns an error message
    fn process_dispute(&mut self, ref_txn: &RefTxn) -> Result<(), TxnErrors> {
        let (acnt_indx, txn_indx) = self.get_ref_txn_indicies(ref_txn)?;

        match &mut self.processed_txns[txn_indx] {
            // Assumption can only have referential transactions on withdrawals & deposits
            Transaction::Withdrawal(disputed_txn) | Transaction::Deposit(disputed_txn) => {
                if disputed_txn.disputed {
                    return Err(TxnErrors::TxnAlreadyDisputed);
                }

                self.accounts[acnt_indx].available -= disputed_txn.amount;
                self.accounts[acnt_indx].held += disputed_txn.amount;

                disputed_txn.disputed = true;
                self.processed_txns
                    .push(Transaction::Dispute(ref_txn.clone()))
            }
            _ => panic!("Only indices of PureTxns should be given from get_ref_txn_indicies()"),
        }
        Ok(())
    }

    /// Takes input resolve txn and applies it if valid, else returns an error message
    fn process_resolve(&mut self, ref_txn: &RefTxn) -> Result<(), TxnErrors> {
        let (acnt_indx, txn_indx) = self.get_ref_txn_indicies(ref_txn)?;
        match &mut self.processed_txns[txn_indx] {
            // Assumption can only have referential transactions on withdrawals & deposits
            Transaction::Withdrawal(disputed_txn) | Transaction::Deposit(disputed_txn) => {
                if !disputed_txn.disputed {
                    return Err(TxnErrors::TxnMustBeDisputed);
                }
                self.accounts[acnt_indx].held -= disputed_txn.amount;
                self.accounts[acnt_indx].available += disputed_txn.amount;

                disputed_txn.disputed = false;
                self.processed_txns
                    .push(Transaction::Resolve(ref_txn.clone()))
            }
            _ => panic!("Only indices of PureTxns should be given from get_ref_txn_indicies()"),
        }
        Ok(())
    }

    /// Takes input chargeback txn and applies it if valid, else returns an error message
    fn process_chargeback(&mut self, ref_txn: &RefTxn) -> Result<(), TxnErrors> {
        let (acnt_indx, txn_indx) = self.get_ref_txn_indicies(ref_txn)?;
        // Assumption can only have referential transactions on withdrawals & deposits
        match &mut self.processed_txns[txn_indx] {
            Transaction::Withdrawal(disputed_txn) | Transaction::Deposit(disputed_txn) => {
                if !disputed_txn.disputed {
                    return Err(TxnErrors::TxnMustBeDisputed);
                }
                self.accounts[acnt_indx].held -= disputed_txn.amount;
                self.accounts[acnt_indx].frozen = true;

                disputed_txn.disputed = false;

                self.processed_txns
                    .push(Transaction::Chargeback(ref_txn.clone()))
            }
            _ => panic!("Only indices of PureTxns should be given from get_ref_txn_indicies()"),
        }
        Ok(())
    }

    /// Base level transactions processing function.  Updates account state with transaction info
    /// Returns success or error depending on transaction details & account state
    /// Logging of fails should be handled by outside functionality
    fn process_txn(&mut self, txn: &Transaction) -> Result<(), TxnErrors> {
        match txn {
            Transaction::Deposit(p_txn) => self.process_deposit(p_txn),
            Transaction::Withdrawal(p_txn) => self.process_withdrawl(p_txn),
            Transaction::Dispute(ref_txn) => self.process_dispute(ref_txn),
            Transaction::Resolve(ref_txn) => self.process_resolve(ref_txn),
            Transaction::Chargeback(ref_txn) => self.process_chargeback(ref_txn),
        }
    }

    /// Executes Payments Engine given a cli input
    pub fn execute_cli(&mut self) -> () {
        // Using guard pattern to avoid nested match
        let cli_res = parse_cli();
        if cli_res.is_err() {
            // TODO custom parsing error message
            return;
        }
        let cli_options = cli_res.unwrap();

        match self.execute_cli_str(&cli_options) {
            Ok(_) => {
                // println!("Success!!!!")
            },
            Err(_) => {
                // println!("Fail!!!!")
            },
        }
    }

    /// Executes Payments Engine given a cli input string
    /// Split out from execute_cli to enable easier unit testing
    fn execute_cli_str(&mut self, cli_input: &CliOptions) -> Result<(), io::Error> {
        // Assume files from cli will always have header
        let txns = parse_txns_csv(&cli_input.input_file.as_str(), true)?;
        for txn in txns.iter() {
            match self.process_txn(txn) {
                Ok(_) => {
                    // could do success logging & follow up notifications
                }
                Err(_) => {
                    // could do failure logging & follow up notifications
                }
            }
        }

        output_accounts(&self.accounts, &cli_input.output);

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::account::Account;
    use crate::cli_io::{CliOptions, OutputMethod};
    use crate::payments_engine::{PaymentsEngine, TxnErrors};
    use crate::transaction::Transaction;
    use crate::transaction::{PureTxn, RefTxn};
    use std::path::PathBuf;
    use std::io;

    fn init_test_objects() -> (PaymentsEngine, PureTxn) {
        let payments_engine = PaymentsEngine::new();
        let txn = PureTxn {
            txn_id: 1,
            acnt_id: 1,
            amount: 10.0,
            disputed: false,
        };
        (payments_engine, txn)
    }


    #[test]
    fn tst_process_deposit() {
        let (mut payments_engine, txn) = init_test_objects();
        let res = payments_engine.process_deposit(&txn);
        assert!(res.is_ok(), "Should pass if account doesn't exist");
        assert_eq!(payments_engine.accounts.len(), 1);
        assert_eq!(payments_engine.acnt_map.len(), 1);
        assert_eq!(payments_engine.processed_txns.len(), 1);
        assert_eq!(payments_engine.txn_map.len(), 1);
        assert_eq!(
            payments_engine.accounts[0],
            Account {
                id: 1,
                available: 10.0,
                held: 0.0,
                frozen: false
            },
            "Should get initial values from deposit"
        );

        let res = payments_engine.process_deposit(&txn);
        match res {
            Ok(_) => panic!("Should be invalid deposit due to TxnIdAlreadyExists"),

            Err(e) => assert_eq!(e, TxnErrors::TxnIdAlreadyExists, "Invalid error type"),
        }

        let txn = PureTxn {
            txn_id: 2,
            acnt_id: 1,
            amount: 10.0,
            disputed: false,
        };
        let res = payments_engine.process_deposit(&txn);
        assert!(res.is_ok(), "Should pass if account already exists");
        assert_eq!(payments_engine.accounts.len(), 1);
        assert_eq!(payments_engine.acnt_map.len(), 1);
        assert_eq!(payments_engine.processed_txns.len(), 2);
        assert_eq!(payments_engine.txn_map.len(), 2);
        assert_eq!(
            payments_engine.accounts[0],
            Account {
                id: 1,
                available: 20.0,
                held: 0.0,
                frozen: false
            },
            "Should add to account 1"
        );

        payments_engine.accounts[0].frozen = true;
        let txn = PureTxn {
            txn_id: 3,
            acnt_id: 1,
            amount: 10.0,
            disputed: true,
        };
        let res = payments_engine.process_deposit(&txn);
        match res {
            Ok(_) => {
                panic!("Should be invalid deposit due to AccountFrozen")
            }
            Err(e) => assert_eq!(e, TxnErrors::AccountFrozen, "Invalid error type"),
        }
    }

    #[test]
    fn tst_process_withdrawl() {
        let mut payments_engine = PaymentsEngine::new();
        let mut txn = PureTxn {
            txn_id: 1,
            acnt_id: 1,
            amount: 10.0,
            disputed: false,
        };
        let res = payments_engine.process_withdrawl(&txn);

        match res {
            Ok(_) => panic!("Should err since account dne"),

            Err(e) => assert_eq!(e, TxnErrors::AccountDoesNotExist, "Invalid error type"),
        }

        let _ = payments_engine.process_deposit(&txn);

        let res = payments_engine.process_withdrawl(&txn);
        match res {
            Ok(_) => panic!("Should err since account TxnIdAlreadyExists"),

            Err(e) => assert_eq!(e, TxnErrors::TxnIdAlreadyExists, "Invalid error type"),
        }

        txn.txn_id = 2;
        txn.amount = 20.0;
        let res = payments_engine.process_withdrawl(&txn);
        match res {
            Ok(_) => panic!("Should err since account AccountLacksFunds"),

            Err(e) => assert_eq!(e, TxnErrors::AccountLacksFunds, "Invalid error type"),
        }

        txn.amount = 5.0;
        let res = payments_engine.process_withdrawl(&txn);
        assert!(res.is_ok(), "Should be valid withdrawl");
        assert_eq!(
            5.0,
            payments_engine.accounts[0].get_total(),
            "Should equal 5 'deposit amount - withdrawl' amount"
        );

        payments_engine.accounts[0].frozen = true;
        txn.txn_id = 3;
        txn.amount = 1.0;
        let res = payments_engine.process_deposit(&txn);
        match res {
            Ok(_) => panic!("Should err since account AccountFrozen"),
            Err(e) => assert_eq!(e, TxnErrors::AccountFrozen, "Invalid error type"),
        }
    }

    #[test]
    fn tst_get_ref_txn_indicies() {
        let mut payments_engine = PaymentsEngine::new();
        let txn = PureTxn {
            txn_id: 1,
            acnt_id: 1,
            amount: 10.0,
            disputed: false,
        };
        let _ = payments_engine.process_deposit(&txn);

        let mut ref_txn = RefTxn {
            ref_id: 1,
            acnt_id: 2,
        };
        let res = payments_engine.get_ref_txn_indicies(&ref_txn);
        match res {
            Ok(_) => panic!("Should err since account dne"),
            Err(e) => assert_eq!(e, TxnErrors::AccountDoesNotExist, "Invalid error type"),
        }

        ref_txn.acnt_id = 1;
        payments_engine.accounts[0].frozen = true;
        let res = payments_engine.get_ref_txn_indicies(&ref_txn);
        match res {
            Ok(_) => panic!("Should err since AccountFrozen"),
            Err(e) => assert_eq!(e, TxnErrors::AccountFrozen, "Invalid error type"),
        }

        ref_txn.ref_id = 3;
        payments_engine.accounts[0].frozen = false;
        let res = payments_engine.get_ref_txn_indicies(&ref_txn);
        match res {
            Ok(_) => panic!("Should err since TxnIdDoesNotExist"),
            Err(e) => assert_eq!(e, TxnErrors::TxnIdDoesNotExist, "Invalid error type"),
        }

        ref_txn.ref_id = 1;
        let res = payments_engine.get_ref_txn_indicies(&ref_txn);
        assert!(res.is_ok(), "Should be valid RefTxn");
        assert_eq!(
            (0, 0),
            res.unwrap(),
            "Should be point to acnt & txn indices"
        );
    }

    #[test]
    fn tst_process_dispute_txn() {
        let (mut payments_engine, mut txn) = init_test_objects();
        let _ = payments_engine.process_deposit(&txn);

        let ref_txn = RefTxn {
            ref_id: 1,
            acnt_id: 1,
        };
        let res = payments_engine.process_dispute(&ref_txn);
        assert!(res.is_ok(), "Should be valid RefTxn");
        assert_eq!(
            payments_engine.processed_txns.len(),
            2,
            "Should add to transactions list"
        );
        assert_eq!(
            payments_engine.txn_map.len(),
            1,
            "Should not add to txn lookup"
        );
        txn.disputed = true;
        match payments_engine.processed_txns[0].clone() {
            Transaction::Deposit(processed_txn) => {
                assert_eq!(processed_txn, txn, "Transaction should be disputed")
            }
            _ => panic!("Transaction order should not have changed"),
        }
        assert_eq!(
            payments_engine.accounts[0],
            Account {
                id: 1,
                available: 0.0,
                held: 10.0,
                frozen: false
            },
            "Account should be unfrozen & funds in held"
        );

        let res = payments_engine.process_dispute(&ref_txn);
        match res {
            Ok(_) => panic!("Should err since TxnAlreadyDisputed"),
            Err(e) => assert_eq!(e, TxnErrors::TxnAlreadyDisputed, "Invalid error type"),
        }
    }

    #[test]
    fn tst_process_resolve_txn() {
        let (mut payments_engine, mut txn) = init_test_objects();

        let _ = payments_engine.process_deposit(&txn);

        let ref_txn = RefTxn {
            ref_id: 1,
            acnt_id: 1,
        };
        let res = payments_engine.process_resolve(&ref_txn);
        match res {
            Ok(_) => panic!("Should err since TxnMustBeDisputed"),
            Err(e) => assert_eq!(e, TxnErrors::TxnMustBeDisputed, "Invalid error type"),
        }

        let _ = payments_engine.process_dispute(&ref_txn);

        // Testing successful run
        let res = payments_engine.process_resolve(&ref_txn);
        assert!(res.is_ok(), "Should be valid RefTxn");
        assert_eq!(
            payments_engine.processed_txns.len(),
            3,
            "RefTxns should add to transactions list"
        );
        assert_eq!(
            payments_engine.txn_map.len(),
            1,
            "RefTxns should not add to txn lookup"
        );
        txn.disputed = false;
        match payments_engine.processed_txns[0].clone() {
            Transaction::Deposit(processed_txn) => {
                assert_eq!(processed_txn, txn, "Transaction should be not be disputed")
            }
            _ => panic!("Transaction order should not have changed"),
        }
        assert_eq!(
            payments_engine.accounts[0],
            Account {
                id: 1,
                available: 10.0,
                held: 0.0,
                frozen: false
            },
            "Account should be undisputed & funds in available"
        );
    }

    #[test]
    fn tst_process_chargeback_txn() {
        let (mut payments_engine, mut txn) = init_test_objects();

        let _ = payments_engine.process_deposit(&txn);

        let ref_txn = RefTxn {
            ref_id: 1,
            acnt_id: 1,
        };
        let res = payments_engine.process_chargeback(&ref_txn);
        match res {
            Ok(_) => panic!("Should err since TxnMustBeDisputed"),
            Err(e) => assert_eq!(e, TxnErrors::TxnMustBeDisputed, "Invalid error type"),
        }

        let _ = payments_engine.process_dispute(&ref_txn);

        // Testing successful run
        let res = payments_engine.process_chargeback(&ref_txn);
        assert!(res.is_ok(), "Should be valid RefTxn");
        assert_eq!(
            payments_engine.processed_txns.len(),
            3,
            "RefTxns should add to transactions list"
        );
        assert_eq!(
            payments_engine.txn_map.len(),
            1,
            "RefTxns should not add to txn lookup"
        );
        txn.disputed = false;
        match payments_engine.processed_txns[0].clone() {
            Transaction::Deposit(processed_txn) => {
                assert_eq!(processed_txn, txn, "Transaction should be not be disputed")
            }
            _ => panic!("Transaction order should not have changed"),
        }
        assert_eq!(
            payments_engine.accounts[0],
            Account {
                id: 1,
                available: 0.0,
                held: 0.0,
                frozen: true
            },
            "Account should be frozen, no longer disputed, & funds charged back"
        )
    }

    pub fn execute_on_tst_file(file_root: &str) -> Result<PaymentsEngine, io::Error> {
        let mut f_input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f_input.push(format!("src/test/inputs/{}.csv", file_root));

        let mut f_output = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f_output.push(format!("src/test/outputs/{}_accounts.csv", file_root));

        let mut payments_engine = PaymentsEngine::new();
        let cli_input = CliOptions {
            input_file: f_input.to_str().unwrap().to_string(),
            output: OutputMethod::Csv(f_output.to_str().unwrap().to_string()),
        };
        payments_engine.execute_cli_str(&cli_input);
        Ok(payments_engine)
    }

    #[test]
    fn tst_execute_cli_str() {
        let res = execute_on_tst_file("simple");
        assert!(res.is_ok(), "Error free is the way to be");
        let expected = vec![Account {
            id: 1,
            available: 10.0,
            held: 0.0,
            frozen: false
        }];
        assert_eq!(expected, res.unwrap().accounts);
    }
}

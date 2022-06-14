use crate::account::Account;
use crate::constants::PRECISION;
use crate::transaction::{PureTxn, RefTxn, Transaction};
use csv::Writer;
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::error::Error;
use std::io::{self, ErrorKind};

fn get_specified_precision(val: &f64, decimal_precision: &i32) -> f64 {
    (val * (10.0_f64).powi(*decimal_precision)).floor() / (10.0_f64).powi(*decimal_precision)
}

/// Options and data to export results
pub enum OutputMethod {
    /// Output to csv file.  Used for integration testing.
    _Csv(String),
    /// Output to console
    StdOutput,
}

/// Output a collection of accounts
pub fn output_accounts(accounts: &Vec<Account>, output: &OutputMethod) {
    match output {
        OutputMethod::_Csv(file_path) => {
            let _ = output_accounts_csv(accounts, file_path);
        }
        OutputMethod::StdOutput => {
            println!("client,available,held,total,locked");
            for acnt in accounts.iter() {
                acnt.print_std_out();
            }
        }
    }
}

fn output_accounts_csv(accounts: &Vec<Account>, file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_path)?;
    wtr.write_record(&["client", "available", "held", "total", "locked"])?;
    for acnt in accounts {
        wtr.write_record(&[
            format!("{}", acnt.id),
            format!("{:.*}", PRECISION, acnt.available),
            format!("{:.*}", PRECISION, acnt.held),
            format!("{:.*}", PRECISION, acnt.get_total()),
            format!("{}", acnt.frozen),
        ])?;
    }
    Ok(())
}

pub struct CliOptions {
    pub input_file: String,
    pub output: OutputMethod,
}

pub fn parse_cli() -> Result<CliOptions, io::Error> {
    let input_file = std::env::args().nth(1).expect("Missing Input File");
    let output = OutputMethod::StdOutput;

    let cli_options = CliOptions { input_file, output };
    Ok(cli_options)
}

/// A transaction which adds or removes an amount
#[derive(Debug, Deserialize)]
pub struct RawInputTxn {
    #[serde(rename = "type")]
    txn_type: String,
    #[serde(rename = "client")]
    acnt_id: u16,
    #[serde(rename = "tx")]
    txn_id: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    amount: Option<f64>,
}

impl RawInputTxn {
    pub fn convert_to_txn(self) -> Result<Transaction, InputTxnErr> {
        let type_str = self.txn_type.as_str();
        if type_str == "deposit" || type_str == "withdrawal" {
            if self.amount.is_none() {
                return Err(InputTxnErr::MissingAmount);
            }
            let pure_txn = PureTxn {
                txn_id: self.txn_id,
                acnt_id: self.acnt_id,
                amount: get_specified_precision(&self.amount.unwrap(), &(PRECISION as i32)),
                disputed: false,
            };
            if type_str == "deposit" {
                return Ok(Transaction::Deposit(pure_txn));
            }
            return Ok(Transaction::Withdrawal(pure_txn));
        } else if type_str == "dispute" || type_str == "resolve" || type_str == "chargeback" {
            if self.amount.is_some() {
                return Err(InputTxnErr::ShouldHaveNoAmount);
            }
            let ref_txn = RefTxn {
                ref_id: self.txn_id,
                acnt_id: self.acnt_id,
            };
            if type_str == "dispute" {
                return Ok(Transaction::Dispute(ref_txn));
            } else if type_str == "resolve" {
                return Ok(Transaction::Resolve(ref_txn));
            }
            return Ok(Transaction::Chargeback(ref_txn));
        }
        Err(InputTxnErr::UnsupportedType)
    }
}

#[derive(PartialEq, Debug)]
pub enum InputTxnErr {
    MissingAmount,
    UnsupportedType,
    ShouldHaveNoAmount,
}

pub fn _parse_txns_csv(
    in_file_path: &str,
    has_header: bool,
) -> Result<Vec<Transaction>, io::Error> {
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(has_header)
        .from_path(in_file_path)?;

    let mut txn_vec = vec![];
    for result in rdr.deserialize() {
        let record: RawInputTxn = result?;
        match record.convert_to_txn() {
            Ok(txn) => txn_vec.push(txn),
            Err(_) => return Err(io::Error::from(ErrorKind::InvalidData)),
        }
    }

    Ok(txn_vec)
}

#[cfg(test)]
mod tests {
    use super::{
        get_specified_precision, output_accounts_csv, InputTxnErr, RawInputTxn, _parse_txns_csv,
    };
    use crate::test::utils::_get_test_output_file;
    use crate::{
        account::Account,
        test::utils::_get_test_input_file,
        transaction::{PureTxn, RefTxn, Transaction},
    };
    use csv::ReaderBuilder;

    #[test]
    fn tst_parse_txns_csv() {
        let f = _get_test_input_file("no_header.csv");
        let txns = _parse_txns_csv(f.as_str(), false).unwrap();
        assert_eq!(txns.len(), 1);
        let deposit = Transaction::Deposit(PureTxn {
            txn_id: 1,
            acnt_id: 1,
            amount: 10.0,
            disputed: false,
        });
        assert_eq!(txns[0], deposit);

        let f = _get_test_input_file("simple.csv");
        let txns = _parse_txns_csv(f.as_str(), true).unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0], deposit);

        let f = _get_test_input_file("dep_disp_res.csv");
        let txns = _parse_txns_csv(f.as_str(), true).unwrap();
        assert_eq!(txns.len(), 3);
        let dispute = Transaction::Dispute(RefTxn {
            ref_id: 1,
            acnt_id: 1,
        });
        let resolve = Transaction::Resolve(RefTxn {
            ref_id: 1,
            acnt_id: 1,
        });
        assert_eq!(txns[0], deposit);
        assert_eq!(txns[1], dispute);
        assert_eq!(txns[2], resolve);

        let deposit = Transaction::Deposit(PureTxn {
            txn_id: 1,
            acnt_id: 1,
            amount: 0.1234,
            disputed: false,
        });

        let f = _get_test_input_file("decimal_precision.csv");
        let txns = _parse_txns_csv(f.as_str(), true).unwrap();
        assert_eq!(txns[0], deposit, "Should have dropped to 4 decimal places");
    }

    #[test]
    fn tst_get_specified_precision() {
        let val = 0.12345;
        assert_eq!(0.1234, get_specified_precision(&val, &4));
    }

    #[test]
    fn tst_to_transaction() {
        let in_txn = RawInputTxn {
            txn_type: "unsupportedtype".to_string(),
            acnt_id: 1,
            txn_id: 1,
            amount: Some(10.0),
        };
        match in_txn.convert_to_txn() {
            Ok(_) => panic!("Should error"),
            Err(e) => assert_eq!(e, InputTxnErr::UnsupportedType),
        }

        let in_txn = RawInputTxn {
            txn_type: "dispute".to_string(),
            acnt_id: 1,
            txn_id: 1,
            amount: Some(10.0),
        };
        match in_txn.convert_to_txn() {
            Ok(_) => panic!("Should error"),
            Err(e) => assert_eq!(e, InputTxnErr::ShouldHaveNoAmount),
        }

        let in_txn = RawInputTxn {
            txn_type: "deposit".to_string(),
            acnt_id: 1,
            txn_id: 1,
            amount: None,
        };
        match in_txn.convert_to_txn() {
            Ok(_) => panic!("Should error"),
            Err(e) => assert_eq!(e, InputTxnErr::MissingAmount),
        }

        let in_txn = RawInputTxn {
            txn_type: "dispute".to_string(),
            acnt_id: 1,
            txn_id: 1,
            amount: None,
        };
        match in_txn.convert_to_txn() {
            Ok(txn) => assert_eq!(
                txn,
                Transaction::Dispute(RefTxn {
                    ref_id: 1,
                    acnt_id: 1
                })
            ),
            Err(_) => panic!("Should result"),
        }
    }

    #[test]
    fn tst_output_accounts_csv() {
        let accounts = vec![Account {
            id: 1,
            available: 3.0,
            held: 7.0,
            frozen: false,
        }];

        let f = _get_test_output_file("tst_file_output.csv");
        let res = output_accounts_csv(&accounts, f.as_str());
        assert!(res.is_ok());

        let mut rdr = ReaderBuilder::new()
            .delimiter(b',')
            .from_path(f.as_str())
            .unwrap();

        if let Some(result) = rdr.records().next() {
            let record = result.unwrap();
            assert_eq!(record, vec!["1", "3.0000", "7.0000", "10.0000", "false"]);
        } else {
            panic!("File should be readable")
        }
    }
}

use super::PaymentsEngine;
use crate::cli_io::{output_accounts, parse_cli, CliOptions, _parse_txns_csv};
use std::io;

impl PaymentsEngine {
    /// Executes Payments Engine given a cli input
    #[allow(clippy::single_match)]
    pub fn _execute_cli_batch(&mut self) {
        // Using guard pattern to avoid nested match
        let cli_res = parse_cli();
        if cli_res.is_err() {
            // TODO custom parsing error message
            return;
        }
        let cli_options = cli_res.unwrap();

        match self._batch_execute(&cli_options) {
            Ok(_) => {
                // println!("Success!!!!")
            }
            Err(_) => {
                // println!("Fail!!!!")
            }
        }
    }

    /// Executes Payments Engine given a cli input string
    /// Split out from execute_cli to enable easier unit testing
    #[allow(clippy::single_match)]
    fn _batch_execute(&mut self, cli_input: &CliOptions) -> Result<(), io::Error> {
        // Assume files from cli will always have header
        let txns = _parse_txns_csv(cli_input.input_file.as_str(), true)?;
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
mod test {
    use crate::account::Account;
    use crate::cli_io::{CliOptions, OutputMethod};
    use crate::payments_engine::PaymentsEngine;
    use std::io;
    use std::path::PathBuf;

    pub fn batch_execute_on_tst_file(file_root: &str) -> Result<PaymentsEngine, io::Error> {
        let mut f_input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f_input.push(format!("src/test/inputs/{}.csv", file_root));

        let mut f_output = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f_output.push(format!("src/test/outputs/{}_accounts.csv", file_root));

        let mut payments_engine = PaymentsEngine::new();
        let cli_input = CliOptions {
            input_file: f_input.to_str().unwrap().to_string(),
            output: OutputMethod::_Csv(f_output.to_str().unwrap().to_string()),
        };
        let _ = payments_engine._batch_execute(&cli_input);
        Ok(payments_engine)
    }

    #[test]
    fn tst_batch_execute() {
        let res = batch_execute_on_tst_file("simple");
        assert!(res.is_ok(), "Error free is the way to be");
        let expected = vec![Account {
            id: 1,
            available: 10.0,
            held: 0.0,
            frozen: false,
        }];
        assert_eq!(expected, res.unwrap().accounts);
    }
}

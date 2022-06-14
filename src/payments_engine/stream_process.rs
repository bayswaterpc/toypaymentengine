use super::PaymentsEngine;
use crate::cli_io::RawInputTxn;
use crate::cli_io::{output_accounts, parse_cli, CliOptions};
use csv::{ReaderBuilder, Trim};
use std::io::{self};

impl PaymentsEngine {
    /// Returns error in the event that file cannot be read
    /// Else mutates the payments engine state
    /// Records with correct data format but fail logically given business logic are ignored
    /// Improper csv format or corrupted records are skipped
    #[allow(clippy::single_match)]
    fn stream_process_csv(
        &mut self,
        in_file_path: &str,
        has_header: bool,
    ) -> Result<(), io::Error> {
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .has_headers(has_header)
            .from_path(in_file_path)?;

        for result in rdr.deserialize() {
            if result.is_err() {
                continue;
            }
            let record: RawInputTxn = result?;
            let txn = record.convert_to_txn();
            // Assume individual invalid records can be ignored, continue process file
            if txn.is_err() {
                // Record error logging & fanout
                continue;
            }
            match self.process_txn(&txn.unwrap()) {
                Ok(_) => {
                    // Record success logging & fanout
                }
                Err(_) => {
                    // Record error logging & fanout
                }
            }
        }

        Ok(())
    }

    /// Executes Payments Engine given a cli input
    /// Won't execute if cli fails parsing
    /// Else will output stream data if input file is valid
    pub fn streaming_execute_cli(&mut self) {
        // Using guard pattern to avoid nested match
        let cli_res = parse_cli();
        if cli_res.is_err() {
            // TODO custom parsing error message
            return;
        }
        let cli_options = cli_res.unwrap();

        self.streaming_execute(&cli_options);
    }

    /// Executes Payments Engine given a cli input string
    /// If a failure occurs mid stream will output all valid records up until that point
    #[allow(clippy::single_match)]
    fn streaming_execute(&mut self, cli_input: &CliOptions) {
        match self.stream_process_csv(&cli_input.input_file, true) {
            Ok(_) => {
                // Success logging and follow up
            }
            Err(_) => {
                // Error logging and follow up
            }
        }

        output_accounts(&self.accounts, &cli_input.output);
    }
}

#[cfg(test)]
pub mod tests {
    use crate::account::Account;
    use crate::payments_engine::PaymentsEngine;
    use crate::test::utils::_get_test_input_file;
    use std::io::{self};
    use std::path::PathBuf;

    fn stream_execute_on_tst_file(
        file_root: &str,
        payments_engine: &mut PaymentsEngine,
    ) -> Result<(), io::Error> {
        let mut f_input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f_input.push(format!("src/test/inputs/{}.csv", file_root));
        let f_input = _get_test_input_file(&format!("{}", file_root));

        payments_engine.stream_process_csv(f_input.as_str(), true)
    }

    #[test]
    fn tst_stream_process_csv() {
        let mut payments_engine = PaymentsEngine::new();
        let res = stream_execute_on_tst_file("simple.csv", &mut payments_engine);
        assert!(res.is_ok(), "Error free is the way to be");
        let expected = vec![Account {
            id: 1,
            available: 10.0,
            held: 0.0,
            frozen: false,
        }];
        assert_eq!(expected, payments_engine.accounts);

        let mut payments_engine = PaymentsEngine::new();
        let res = stream_execute_on_tst_file("broke_middle.csv", &mut payments_engine);
        assert!(res.is_ok(), "Error free is the way to be");
        let expected = vec![
            Account {
                id: 1,
                available: 1.0,
                held: 0.0,
                frozen: false,
            },
            Account {
                id: 3,
                available: 3.0,
                held: 0.0,
                frozen: false,
            },
        ];
        assert_eq!(expected, payments_engine.accounts);
    }
}

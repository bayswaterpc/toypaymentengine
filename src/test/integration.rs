/// Testing functionality which spans mods
#[cfg(test)]
mod tests {
    use csv::ReaderBuilder;
    use crate::payments_engine::tests::execute_on_tst_file;
    use std::path::PathBuf;


    fn validate_tst_files(file_root: &str, accounts_str: Vec<Vec<&str>>) {
        let mut f = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        f.push(format!("src/test/outputs/{}_accounts.csv", file_root));
        let mut rdr = ReaderBuilder::new()
            .delimiter(b',')
            .from_path(f.to_str().unwrap())
            .unwrap();

        let mut result = rdr.records().next();
        for accnt in accounts_str.iter() {
            if result.is_none() {
                panic!("File is missing Records");
            }
            let record = result.unwrap();
            assert_eq!(record.unwrap(), *accnt);
            result = rdr.records().next();
        }

        if result.is_some() {
            panic!("File has excess records")
        }
    }

    /// Testing functionality in payments_engine & file io
    #[test]
    fn tst_execute_cli_str() {
        let res = execute_on_tst_file("simple");
        assert!(res.is_ok(), "Error free is the way to be");
        let expected = vec![vec!["1", "10.0000", "0.0000", "10.0000", "false"]];
        validate_tst_files("simple", expected);
    }
}

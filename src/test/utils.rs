use std::path::PathBuf;

pub fn _get_test_input_file(filename: &str) -> String {
    let mut f = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    f.push(format!("src/test/inputs/{}", filename));
    f.to_str().unwrap().to_string()
}

pub fn _get_test_output_file(filename: &str, test_subdir: &str) -> String {
    let mut f = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    f.push(format!("src/test/outputs/{}/{}.csv", test_subdir, filename));
    let parent = f.parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    f.to_str().unwrap().to_string()
}

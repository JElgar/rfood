use std::path::PathBuf;
use std::fs;

use rfood::transform::transformer::{transform_file, TransformType};

fn test_run_transform_example(example_path: PathBuf, example_output_path: PathBuf, direction: TransformType) {
    // Remove the existing output file if it exists
    let _ = fs::remove_file(&example_output_path);
    assert!(!example_output_path.exists());

    // Transform the file
    transform_file(&example_path, &example_output_path, &direction);

    // Assert that the output file exists and that is is not empty
    assert!(example_output_path.exists());
    let file = fs::File::open(&example_output_path).expect("Unable to open file");
    assert!(file.metadata().unwrap().len() > 0);
}

#[test]
fn test_run_transform_example_oop_exp() {
    test_run_transform_example(
        PathBuf::from(r"./src/examples/exp/oop.rs"),
        PathBuf::from(r"./src/examples/exp/oop_output.rs"),
        TransformType::OOPToFP
    )
}

#[test]
fn test_run_transform_example_oop_set() {
    test_run_transform_example(
        PathBuf::from(r"./src/examples/set/oop.rs"),
        PathBuf::from(r"./src/examples/set/oop_output.rs"),
        TransformType::OOPToFP
    )
}

#[test]
fn test_run_transform_example_fp_exp() {
    test_run_transform_example(
        PathBuf::from(r"./src/examples/exp/fp.rs"),
        PathBuf::from(r"./src/examples/exp/fp_output.rs"),
        TransformType::FPToOOP
    )
}

#[test]
fn test_run_transform_example_fp_set() {
    test_run_transform_example(
        PathBuf::from(r"./src/examples/set/fp.rs"),
        PathBuf::from(r"./src/examples/set/fp_output.rs"),
        TransformType::FPToOOP
    )
}

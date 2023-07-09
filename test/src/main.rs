use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const TEST_PACKAGE_NAME: &'static str = "test";

fn main() {
    let output_dir = Path::new("target").join(TEST_PACKAGE_NAME);
    // Prepare the test output directory
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).expect("Failed to remove existing test directory");
    }
    fs::create_dir_all(&output_dir).expect("Failed to create test directory");
    for entry in WalkDir::new("test") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_file() && path.extension() == Some("test".as_ref()) {
            run_test(path, &output_dir);
        }
    }
}

fn run_test(input_path: &Path, output_dir: &Path) {
    // Clean the test package only. We want to keep the builds of the dependencies, but
    // ensure OUT_DIR is removed. It's probably not a bad thing to remove the compilation
    // cache either.
    Command::new("cargo")
        .arg("clean")
        .arg("-p")
        .arg(TEST_PACKAGE_NAME)
        .current_dir(&output_dir)
        .status()
        .expect("failed to clean test package");
    // Prepare the output dir with the files specified in the file at input_path
    if !parse_and_write_files(input_path, &output_dir).is_ok() {
        panic!("Failed to create files for test {}", input_path.display());
    }
    let cargo_run_output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .current_dir(&output_dir)
        .status()
        .expect("failed to run test with 'cargo run'");

    if cargo_run_output.success() {
        println!("***** {} PASS", input_path.display());
    } else {
        println!("***** {} FAIL", input_path.display());
    }
}

fn parse_and_write_files(source_path: &Path, out_prefix: &Path) -> io::Result<()> {
    let source_file = File::open(source_path)?;
    let reader = io::BufReader::new(source_file);

    let mut current_file: Option<File> = None;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("//file:") {
            // Close previous file
            if let Some(mut file) = current_file.take() {
                file.flush()?;
            }
            // Open new file
            let file_name = line.trim_start_matches("//file:").trim();
            if file_name.starts_with("..") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "File path cannot start with '..'",
                ));
            }
            let file_path = out_prefix.join(Path::new(file_name));
            // Create directories if necessary
            let path = PathBuf::from(&file_path);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            // Create the file
            current_file = Some(File::create(&file_path)?);
        } else if let Some(file) = current_file.as_mut() {
            writeln!(file, "{}", line)?;
        }
    }

    // Flush and close the last file
    if let Some(mut file) = current_file.take() {
        file.flush()?;
    }

    Ok(())
}

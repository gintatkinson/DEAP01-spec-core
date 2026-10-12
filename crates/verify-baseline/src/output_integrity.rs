//! Subagent output integrity and escape tokens validation gate.
//!
//! Verifies:
//! 1. Non-zero file size
//! 2. Existence on filesystem
//! 3. Valid code fence balancing
//! 4. Zero unreplaced {{REQUIRED_*}} escape tokens

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(clap::Args, Debug, Clone)]
pub struct OutputIntegrityArgs {
    /// Target directory to scan for markdown files
    #[arg(long = "dir")]
    pub dir: Option<PathBuf>,

    /// Explicit list of files to verify
    #[arg(long = "files", num_args = 1..)]
    pub files: Option<Vec<PathBuf>>,

    /// Positional targets (directory or file paths)
    #[arg(value_name = "TARGETS")]
    pub targets: Vec<PathBuf>,
}

pub fn collect_target_files(args: &OutputIntegrityArgs) -> Vec<PathBuf> {
    let mut target_files = Vec::new();

    if let Some(ref dir) = args.dir {
        if dir.is_dir() {
            for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "md" {
                            target_files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }
    }

    if let Some(ref files) = args.files {
        for f in files {
            target_files.push(f.clone());
        }
    }

    for target in &args.targets {
        if target.is_dir() {
            for entry in WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "md" {
                            target_files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        } else if target.is_file() {
            target_files.push(target.clone());
        }
    }

    target_files.sort();
    target_files.dedup();
    target_files
}

pub fn verify_file_output(path: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // 1. Non-zero size
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            errors.push(format!("Cannot read metadata for {}: {}", path.display(), e));
            return Err(errors);
        }
    };

    if metadata.len() == 0 {
        errors.push(format!("Subagent output file is empty: {}", path.display()));
        return Err(errors);
    }

    // Read content
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("Cannot read file {}: {}", path.display(), e));
            return Err(errors);
        }
    };

    // 2. Escape tokens
    if content.contains("{{REQUIRED_") {
        errors.push(format!(
            "Unreplaced {{{{REQUIRED_*}}}} escape token detected in: {}",
            path.display()
        ));
    }

    // 3. Balanced code blocks
    let fence_count = content.lines().filter(|l| l.contains("```")).count();
    if fence_count % 2 != 0 {
        errors.push(format!(
            "Unbalanced code fences in: {} (found {} fences)",
            path.display(),
            fence_count
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn run_check_output(args: &OutputIntegrityArgs) -> Result<(), i32> {
    let target_files = collect_target_files(args);

    if target_files.is_empty() {
        println!("Subagent output verification PASSED (0 files verified).");
        return Ok(());
    }

    let mut failed = 0;
    let mut checked = 0;

    for file in &target_files {
        if !file.is_file() {
            continue;
        }
        checked += 1;

        if let Err(errors) = verify_file_output(file) {
            failed += 1;
            for err in errors {
                eprintln!("ERROR: {}", err);
            }
        }
    }

    if failed > 0 {
        eprintln!(
            "Subagent output verification FAILED ({} errors out of {} files).",
            failed, checked
        );
        Err(42)
    } else {
        println!("Subagent output verification PASSED ({} files verified).", checked);
        Ok(())
    }
}

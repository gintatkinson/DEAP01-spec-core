//! Native Post-Build CI Secret Leak Scanner (pure lexical token scanning).
//!
//! Enforces zero regex crate usage (Anti-Regex Invariant) while detecting
//! leaked PEM Private Keys and AWS Access Key IDs in post-build assets.

use clap::Args;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Arguments for check-secrets subcommand.
#[derive(Args, Debug, Clone)]
pub struct CheckSecretsArgs {
    /// Target repository root or destination directory (defaults to current directory)
    #[arg(default_value = ".")]
    pub destination: String,

    /// Optional explicit directories to scan (defaults to dist, build/web, build/macos)
    #[arg(long = "dirs", value_delimiter = ' ')]
    pub dirs: Vec<String>,
}

/// Lexical checker for PEM private key headers.
///
/// Matches: `-----BEGIN [TYPE ]PRIVATE KEY-----` case-insensitively,
/// where TYPE contains only uppercase/lowercase alphanumeric characters, underscores, or hyphens.
pub fn is_pem_private_key_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let prefix = "-----begin";
    let suffix = "private key-----";

    let mut start_search = 0;
    while let Some(begin_idx) = lower[start_search..].find(prefix) {
        let abs_begin = start_search + begin_idx;
        let after_begin = abs_begin + prefix.len();
        if after_begin < lower.len() {
            let rem = &lower[after_begin..];
            if let Some(first_char) = rem.chars().next() {
                if first_char == ' ' || first_char == '\t' {
                    if let Some(suffix_idx) = rem.find(suffix) {
                        let middle = &rem[..suffix_idx];
                        let middle_valid = middle.chars().all(|c| {
                            c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '\t'
                        });
                        if middle_valid {
                            return true;
                        }
                    }
                }
            }
        }
        start_search = after_begin;
    }
    false
}

/// Lexical checker for AWS Access Key ID occurrences.
///
/// Matches: `AKIA` followed by exactly 16 uppercase ASCII alphanumeric characters,
/// bounded by non-alphanumeric/non-underscore characters (or string boundaries).
pub fn contains_aws_access_key(text: &str) -> bool {
    let bytes = text.as_bytes();
    let prefix = b"AKIA";
    let mut i = 0;
    while i + 20 <= bytes.len() {
        if &bytes[i..i + 4] == prefix {
            let boundary_before = if i == 0 {
                true
            } else {
                let prev = bytes[i - 1];
                !prev.is_ascii_alphanumeric() && prev != b'_'
            };

            if boundary_before {
                let is_key = bytes[i + 4..i + 20].iter().all(|&b| {
                    b.is_ascii_uppercase() || b.is_ascii_digit()
                });
                if is_key {
                    let boundary_after = if i + 20 == bytes.len() {
                        true
                    } else {
                        let next = bytes[i + 20];
                        !next.is_ascii_alphanumeric() && next != b'_'
                    };
                    if boundary_after {
                        return true;
                    }
                }
            }
        }
        i += 1;
    }
    false
}

/// Execute secret leak scan against target directories.
pub fn run_check_secrets(args: &CheckSecretsArgs) -> Result<(), i32> {
    let repo_root = PathBuf::from(&args.destination);
    let target_dirs: Vec<PathBuf> = if args.dirs.is_empty() {
        vec![
            repo_root.join("dist"),
            repo_root.join("build").join("web"),
            repo_root.join("build").join("macos"),
        ]
    } else {
        args.dirs.iter().map(|d| repo_root.join(d)).collect()
    };

    println!("=== Post-Build CI Secret Leak Scanner ===");

    let mut found_any = false;
    let mut total_errors = 0;

    for dir in &target_dirs {
        let display_name = dir.strip_prefix(&repo_root).unwrap_or(dir).display();
        if dir.is_dir() {
            found_any = true;
            println!("Scanning directory: {}", display_name);

            for entry in WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let path = entry.path();
                let rel_path = path.strip_prefix(&repo_root).unwrap_or(path).display();

                if let Ok(content) = fs::read_to_string(path) {
                    for line in content.lines() {
                        if is_pem_private_key_line(line) {
                            eprintln!("  - Potential PEM Private Key found in {}", rel_path);
                            total_errors += 1;
                            break;
                        }
                        if contains_aws_access_key(line) {
                            eprintln!("  - Potential AWS Access Key ID found in {}", rel_path);
                            total_errors += 1;
                            break;
                        }
                    }
                }
            }
        } else {
            println!("Directory not found (skipped): {}", display_name);
        }
    }

    if !found_any {
        println!("Note: None of the post-build directories exist. Skipping scan.");
        return Ok(());
    }

    if total_errors > 0 {
        eprintln!("[!] CI Security Gate Failure: Potential Secrets or Credentials Leaked.");
        return Err(1);
    }

    println!("Success: No secret leaks detected in post-build assets.");
    Ok(())
}

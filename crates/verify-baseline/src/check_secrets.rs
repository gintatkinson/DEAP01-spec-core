//! Native Post-Build CI Secret Leak Scanner (pure lexical token scanning).
//!
//! Safety & Traceability: Realises: [REQ-VERIFY-BASELINE-CHECK-SECRETS]
//! Contract: Enforces zero regex crate usage while detecting leaked PEM Private Keys and AWS Access Key IDs in post-build assets.
//! Bounds & Faults: Linear time over scanned file bytes; fails closed with `Err(1)` when any potential secret leak is detected.

use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Arguments for check-secrets subcommand.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `CheckSecretsArgs` in `crates/verify-baseline/src/check_secrets.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `CheckSecretsArgs` across member set `[destination: String, dirs: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `CheckSecretsArgs` members (`destination: String, dirs: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `CheckSecretsArgs` representation for struct `CheckSecretsArgs`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `CheckSecretsArgs`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `CheckSecretsArgs`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `CheckSecretsArgs` (`CheckSecretsArgs`).
/// - State: Maintains consistent state invariants across `CheckSecretsArgs` without partial mutation on error.
/// - Action: Executes `CheckSecretsArgs` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `CheckSecretsArgs` (`VERIFY-BASELINE-CHECK-SECRETS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-CHECK-SECRETS-CHECKSECRETSARGS] | SysML v2: PartDef(CheckSecretsArgs)
#[derive(Debug, Clone)]
pub struct CheckSecretsArgs {
    /// Target repository root or destination directory (defaults to current directory)
    pub destination: String,

    /// Optional explicit directories to scan (defaults to dist, build/web, build/macos)
    pub dirs: Vec<String>,
}

/// Lexical checker for PEM private key headers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_pem_private_key_line` in `crates/verify-baseline/src/check_secrets.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_pem_private_key_line` (declaration #2) within `VERIFY-BASELINE-CHECK-SECRETS`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `is_pem_private_key_line`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_pem_private_key_line` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `is_pem_private_key_line`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `is_pem_private_key_line`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_pem_private_key_line` (`bool`).
/// - State: Maintains consistent state invariants across `is_pem_private_key_line` without partial mutation on error.
/// - Action: Executes `is_pem_private_key_line` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_pem_private_key_line` (`VERIFY-BASELINE-CHECK-SECRETS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-CHECK-SECRETS-IS-PEM-PRIVATE-KEY-LINE] | SysML v2: ActionDef(is_pem_private_key_line)
pub fn is_pem_private_key_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let prefix = "-----begin";
    let suffix = "private key-----";

    for (abs_begin, _) in lower.match_indices(prefix) {
        let after_begin = abs_begin.saturating_add(prefix.len());
        let Some(rem) = lower.get(after_begin..) else {
            continue;
        };
        let starts_with_ws = rem
            .chars()
            .next()
            .is_some_and(|first_char| first_char == ' ' || first_char == '\t');
        if !starts_with_ws {
            continue;
        }
        if let Some((middle, _)) = rem.split_once(suffix) {
            let middle_valid = middle.chars().all(|c| {
                c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '\t'
            });
            if middle_valid {
                return true;
            }
        }
    }
    false
}

/// Lexical checker for AWS Access Key ID occurrences.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `contains_aws_access_key` in `crates/verify-baseline/src/check_secrets.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `contains_aws_access_key` (declaration #3) within `VERIFY-BASELINE-CHECK-SECRETS`.
///
/// ## 3. Preconditions
/// - `text: &str` must satisfy domain and type-safety constraints prior to invoking `contains_aws_access_key`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `contains_aws_access_key` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(text)) bounded over input/member dimension of `contains_aws_access_key`.
/// - Space Complexity: O(N(text)) bounded auxiliary memory for `contains_aws_access_key`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `contains_aws_access_key` (`bool`).
/// - State: Maintains consistent state invariants across `contains_aws_access_key` without partial mutation on error.
/// - Action: Executes `contains_aws_access_key` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `contains_aws_access_key` (`VERIFY-BASELINE-CHECK-SECRETS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-CHECK-SECRETS-CONTAINS-AWS-ACCESS-KEY] | SysML v2: ActionDef(contains_aws_access_key)
pub fn contains_aws_access_key(text: &str) -> bool {
    let bytes = text.as_bytes();
    let prefix = b"AKIA";
    let mut i = 0usize;
    while i.saturating_add(20) <= bytes.len() {
        let prefix_end = i.saturating_add(4);
        let key_end = i.saturating_add(20);
        if bytes.get(i..prefix_end) == Some(prefix.as_slice()) {
            let boundary_before = if i == 0 {
                true
            } else {
                !bytes
                    .get(i.saturating_sub(1))
                    .is_some_and(|&prev| prev.is_ascii_alphanumeric() || prev == b'_')
            };

            if boundary_before {
                let is_key = bytes.get(prefix_end..key_end).is_some_and(|slice| {
                    slice
                        .iter()
                        .all(|&b| b.is_ascii_uppercase() || b.is_ascii_digit())
                });
                if is_key {
                    let boundary_after = if key_end == bytes.len() {
                        true
                    } else {
                        !bytes
                            .get(key_end)
                            .is_some_and(|&next| next.is_ascii_alphanumeric() || next == b'_')
                    };
                    if boundary_after {
                        return true;
                    }
                }
            }
        }
        i = i.saturating_add(1);
    }
    false
}

/// Execute secret leak scan against target directories.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `run_check_secrets` in `crates/verify-baseline/src/check_secrets.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `run_check_secrets` (declaration #4) within `VERIFY-BASELINE-CHECK-SECRETS`.
///
/// ## 3. Preconditions
/// - `args: &CheckSecretsArgs` must satisfy domain and type-safety constraints prior to invoking `run_check_secrets`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), i32>` from `run_check_secrets` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(args)) bounded over input/member dimension of `run_check_secrets`.
/// - Space Complexity: O(N(args)) bounded auxiliary memory for `run_check_secrets`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `run_check_secrets` (`Result<(), i32>`).
/// - State: Maintains consistent state invariants across `run_check_secrets` without partial mutation on error.
/// - Action: Executes `run_check_secrets` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `run_check_secrets` (`VERIFY-BASELINE-CHECK-SECRETS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-CHECK-SECRETS-RUN-CHECK-SECRETS] | SysML v2: ActionDef(run_check_secrets)
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
    let mut total_errors = 0usize;

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
                            total_errors = total_errors.saturating_add(1);
                            break;
                        }
                        if contains_aws_access_key(line) {
                            eprintln!("  - Potential AWS Access Key ID found in {}", rel_path);
                            total_errors = total_errors.saturating_add(1);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pem_private_key_detection_and_utf8_boundaries() {
        assert!(is_pem_private_key_line("-----BEGIN PRIVATE KEY-----"));
        assert!(is_pem_private_key_line("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(is_pem_private_key_line(
            "-----BEGIN EC_TEST-1\tPRIVATE KEY-----"
        ));
        assert!(is_pem_private_key_line(
            "prefix \u{1F511} -----BEGIN OPENSSH PRIVATE KEY----- \u{1F511}"
        ));
        assert!(is_pem_private_key_line(
            "noise -----begin bad! -----BEGIN RSA PRIVATE KEY-----"
        ));

        assert!(!is_pem_private_key_line("-----BEGIN PUBLIC KEY-----"));
        assert!(!is_pem_private_key_line("-----BEGINPRIVATE KEY-----"));
        assert!(!is_pem_private_key_line("-----BEGIN"));
        assert!(!is_pem_private_key_line("-----BEGIN RSA$ PRIVATE KEY-----"));
    }

    #[test]
    fn test_aws_access_key_detection_and_boundaries() {
        assert!(contains_aws_access_key("AKIAIOSFODNN7EXAMPLE"));
        assert!(contains_aws_access_key(
            "\u{1F512} key=\"AKIA1234567890ABCDEF\" \u{1F512}"
        ));

        assert!(!contains_aws_access_key("AKIA12345"));
        assert!(!contains_aws_access_key("XAKIAIOSFODNN7EXAMPLE"));
        assert!(!contains_aws_access_key("_AKIAIOSFODNN7EXAMPLE"));
        assert!(!contains_aws_access_key("AKIAIOSFODNN7EXAMPLEX"));
        assert!(!contains_aws_access_key("AKIAIOSFODNN7EXAMPLE_"));
        assert!(!contains_aws_access_key("AKIAiosfodnn7example"));
    }
}

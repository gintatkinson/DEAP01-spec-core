//! Baseline verification library crate (`verify-baseline`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the primary library facade for deterministic repository baseline verification,
//! secret scanning, commit message governance, defect dossier validation, output integrity
//! checks, Git hook provisioning, and specification audit under DO-178C Level A, DO-330 TQL-4,
//! and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exports only deterministic, side-effect-controlled verification entry points and data contracts.
//! - Preserves strict module separation across `audit_code`, `blueprint_gate`, `check_secrets`, `checks`,
//!   `commit_msg`, `defect_dossier`, `output_integrity`, `runner`, `setup_hooks`, and `spec_audit`.
//!
//! ## 3. Preconditions
//! - Callers supply valid workspace paths, CLI options, or in-memory payloads conforming to the
//!   respective submodule argument contracts.
//!
//! ## 4. Postconditions
//! - Exposes a stable, deterministic public API surface for the `verify-baseline` binary and
//!   downstream verification consumers without hidden global state mutations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ module linkage overhead; runtime complexity is bounded by the
//!   invoked submodule entrypoint.
//! - Space Complexity: $O(1)$ crate-level static dispatch overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Submodule verification failures propagate deterministically via explicit `Result` types,
//!   exit codes, or structured violation records without silent suppression.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-LIB], [DO-330-TQL-4-VERIFY-LIB]

pub mod audit_code;
pub mod blueprint_gate;
pub mod check_secrets;
pub mod checks;
pub mod commit_msg;
pub mod defect_dossier;
pub mod ndjson_cli;
pub mod output_integrity;
pub mod runner;
pub mod setup_hooks;
pub mod spec_audit;

pub use check_secrets::{run_check_secrets, CheckSecretsArgs};
pub use commit_msg::{scan_commit_message, CommitMsgArgs, CommitViolation};
pub use defect_dossier::{
    parse_github_repo_slug, resolve_target_repo, validate_defect_dossier,
    validate_defect_dossier_scoped, validate_defect_dossier_with_repo, DefectDossierArgs,
    DefectSchemaValidation,
};
pub use output_integrity::{verify_file_output, OutputIntegrityArgs};
pub use runner::{run, BaselineOptions};
pub use setup_hooks::{run_setup_hooks, SetupHooksArgs};
pub use spec_audit::{
    run_spec_audit, SpecAuditError, SpecAuditFinding, SpecAuditOptions, SpecAuditSummary,
};

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomically persists `bytes` to `path` via a PID- and `AtomicU64`-sequenced sibling temporary
/// file (`.{file_name}.tmp.{pid}.{seq}`), `sync_all()` flush, and rename, unlinking the temp file on error.
/// Realises: [REQ-VERIFY-BASELINE-LIB]
pub(crate) fn write_atomic_file(path: &Path, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    static NEXT_TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

    let Some(raw_file_name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "atomic write target path must include a file name",
        ));
    };

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let file_name = raw_file_name.to_string_lossy();
    let pid = std::process::id();
    let seq = NEXT_TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp_path = path.with_file_name(format!(".{file_name}.tmp.{pid}.{seq}"));

    let tmp_file = std::fs::File::create(&tmp_path)?;
    let persist_res = (|| -> std::io::Result<()> {
        {
            let mut file = tmp_file;
            file.write_all(bytes.as_ref())?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp_path, path)
    })();

    if persist_res.is_err() {
        if let Err(_cleanup_err) = std::fs::remove_file(&tmp_path) {
            // Preserve the primary write, sync, or rename error after best-effort temp cleanup.
        }
    }

    persist_res
}

#[cfg(test)]
mod tests {
    use super::write_atomic_file;
    use std::fs;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct ScopedTempDir {
        path: PathBuf,
    }

    impl ScopedTempDir {
        fn new(label: &str) -> std::io::Result<Self> {
            static DIR_SEQ: AtomicU64 = AtomicU64::new(0);
            let seq = DIR_SEQ.fetch_add(1, Ordering::Relaxed);
            let pid = std::process::id();
            let path =
                std::env::temp_dir().join(format!("verify_baseline_atomic_{label}_{pid}_{seq}"));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    impl Drop for ScopedTempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn count_temp_siblings(dir: &Path) -> std::io::Result<usize> {
        let mut count: usize = 0;
        for entry_res in fs::read_dir(dir)? {
            let entry = entry_res?;
            if entry.file_name().to_string_lossy().contains(".tmp.") {
                count = count.saturating_add(1);
            }
        }
        Ok(count)
    }

    #[test]
    fn test_write_atomic_file_persistence_cleanup_and_concurrency(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scratch = ScopedTempDir::new("persistence")?;
        let nested_dir = scratch.path.join("nested");
        fs::create_dir_all(&nested_dir)?;

        // 1. Persist initial file while preserving unrelated `.tmp` sibling files.
        let sibling_tmp = nested_dir.join("dossier.tmp");
        fs::write(&sibling_tmp, "preserve-sibling-tmp")?;
        let target = nested_dir.join("dossier.md");
        write_atomic_file(&target, "initial-atomic-payload\n")?;
        assert_eq!(fs::read_to_string(&target)?, "initial-atomic-payload\n");
        assert_eq!(fs::read_to_string(&sibling_tmp)?, "preserve-sibling-tmp");
        assert_eq!(count_temp_siblings(&nested_dir)?, 0);

        // 2. Overwrite atomically via rename without mutating an already-open reader handle.
        let mut open_reader = fs::File::open(&target)?;
        write_atomic_file(&target, b"updated-atomic-payload\n")?;
        let mut snapshot = String::new();
        open_reader.read_to_string(&mut snapshot)?;
        assert_eq!(snapshot, "initial-atomic-payload\n");
        assert_eq!(fs::read_to_string(&target)?, "updated-atomic-payload\n");
        assert_eq!(count_temp_siblings(&nested_dir)?, 0);

        // 3. Failure path: renaming over a non-empty directory fails closed and removes temp file.
        let dir_collision = nested_dir.join("dir_target.md");
        fs::create_dir_all(dir_collision.join("child"))?;
        let collision_err = write_atomic_file(&dir_collision, "cannot-overwrite-dir");
        assert!(collision_err.is_err());
        assert_eq!(count_temp_siblings(&nested_dir)?, 0);

        // 4. Reject empty target path without file name cleanly.
        let empty_path_err = write_atomic_file(Path::new(""), "invalid");
        assert_eq!(
            empty_path_err.as_ref().map_err(std::io::Error::kind),
            Err(std::io::ErrorKind::InvalidInput)
        );

        // 5. Concurrent multi-threaded writes to the same path remain collision-free and uncorrupted.
        let concurrent_target = nested_dir.join("concurrent_report.md");
        let thread_results = std::thread::scope(|scope| -> Vec<std::io::Result<()>> {
            let mut handles = Vec::new();
            for tid in 0_u8..8_u8 {
                let target_ref = &concurrent_target;
                let fill_char = char::from(b'A'.saturating_add(tid));
                let payload = fill_char.to_string().repeat(4096);
                handles.push(scope.spawn(move || write_atomic_file(target_ref, payload)));
            }
            let mut results = Vec::with_capacity(handles.len());
            for handle in handles {
                let res = handle.join().unwrap_or_else(|_| {
                    Err(std::io::Error::other("atomic writer thread panicked"))
                });
                results.push(res);
            }
            results
        });
        for res in thread_results {
            assert!(res.is_ok(), "concurrent atomic write failed: {res:?}");
        }
        let final_text = fs::read_to_string(&concurrent_target)?;
        assert_eq!(final_text.len(), 4096);
        let first_ch = final_text
            .chars()
            .next()
            .ok_or("empty concurrent payload")?;
        assert!(final_text.chars().all(|ch| ch == first_ch));
        assert_eq!(count_temp_siblings(&nested_dir)?, 0);

        Ok(())
    }
    #[test]
    fn test_verify_baseline_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        let module_docs: String = src
            .lines()
            .take_while(|line| line.starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");
        assert!(module_docs.contains("## 1. Safety Intent & Regulatory Scope"));
        assert!(module_docs.contains("## 2. Invariants & Mathematical Properties"));
        assert!(module_docs.contains("## 3. Preconditions"));
        assert!(module_docs.contains("## 4. Postconditions"));
        assert!(module_docs.contains("## 5. Algorithmic Complexity Bounds"));
        assert!(module_docs.contains("## 6. Failure Modes & Fault Tolerance"));
        assert!(module_docs.contains("## 7. Realization & Traceability"));
        assert!(
            module_docs.contains("Realises: [REQ-VERIFY-BASELINE-LIB], [DO-330-TQL-4-VERIFY-LIB]")
        );
    }

    #[test]
    fn test_crate_roots_forbid_unconditional_safety_lint_allows() {
        for src in [
            include_str!("../../assemble-conops/src/lib.rs"),
            include_str!("../../assemble-conops/src/main.rs"),
            include_str!("../../compile-sysml/src/lib.rs"),
            include_str!("../../compile-sysml/src/main.rs"),
            include_str!("lib.rs"),
            include_str!("main.rs"),
        ] {
            let prod_src = src.split("#[cfg(test)]").next().unwrap_or(src);
            let has_unconditional_inner_allow = prod_src
                .lines()
                .any(|line| line.trim_start().starts_with("#![allow("));
            assert!(!has_unconditional_inner_allow);
        }
    }
}

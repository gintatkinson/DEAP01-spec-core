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
//! - Preserves strict module separation across `check_secrets`, `checks`, `commit_msg`,
//!   `defect_dossier`, `output_integrity`, `runner`, `setup_hooks`, and `spec_audit`.
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

pub mod check_secrets;
pub mod checks;
pub mod commit_msg;
pub mod defect_dossier;
pub mod output_integrity;
pub mod runner;
pub mod setup_hooks;
pub mod spec_audit;

pub use check_secrets::{run_check_secrets, CheckSecretsArgs};
pub use commit_msg::{scan_commit_message, CommitMsgArgs, CommitViolation};
pub use defect_dossier::{validate_defect_dossier, DefectDossierArgs, DefectSchemaValidation};
pub use output_integrity::{verify_file_output, OutputIntegrityArgs};
pub use runner::{run, BaselineOptions};
pub use setup_hooks::{run_setup_hooks, SetupHooksArgs};
pub use spec_audit::{
    run_spec_audit, SpecAuditError, SpecAuditFinding, SpecAuditOptions, SpecAuditSummary,
};

#[cfg(test)]
mod tests {
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
}

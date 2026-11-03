#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Unit tests for `compile-sysml` docstring synthesis and Tier-B/C compaction (`TD-19`).
//! Safety & Traceability: Realises: [REQ-0501], [REQ-0608], [SC-01]
//! Contract: Verifies Tier-B/C docblock compaction to 2-line summary + `Realises:` form, Tier-A 7-section preservation, and generated-directory exclusion.
//! Bounds & Faults: Executes in ephemeral temporary directories in O(N) time with zero workspace mutation.

use super::*;
use deap_core::rust_cst::codemod::CodemodError;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Auto-cleaned temporary directory for isolated docstring unit tests.
/// Realises: [REQ-0501], [REQ-0608]
struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    /// Creates a unique temporary directory under `std::env::temp_dir()`.
    /// Realises: [REQ-0501], [REQ-0608]
    fn new() -> std::io::Result<Self> {
        let seq = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir_name = format!("deap_docstring_unit_{}_{seq}", std::process::id());
        let path = std::env::temp_dir().join(dir_name);
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// Returns the filesystem path of the temporary directory.
    /// Realises: [REQ-0501], [REQ-0608]
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScopedTempDir {
    /// Removes the temporary directory on scope exit.
    /// Realises: [REQ-0501], [REQ-0608]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies `probe_single_char_literal`, `DocstringExitCode::code`, and empty-input validation smoke invariants.
/// Realises: [REQ-FEAT-42-SINGLE-CHAR-PROBE], [REQ-FEAT-42-DOCSTRING-EXIT-CODE]
#[test]
fn test_docstring_module_smoke_invariants() {
    assert_eq!(probe_single_char_literal("'a'"), Some(3));
    assert_eq!(DocstringExitCode::Clean.code(), 0);
    assert_eq!(DocstringExitCode::Error.code(), 1);
    assert_eq!(DocstringExitCode::IdempotencyOrConflict.code(), 2);
    assert_eq!(
        compact_tier_bc_tree(Path::new(""), "modular_codegen/generated"),
        Err(CodemodError::EmptyInput)
    );
}

/// Verifies that `compact_tier_bc_tree` compacts multi-line Tier-B/C docblocks while preserving summary and `Realises:` lines (`TD-19`).
/// Realises: [REQ-0501], [REQ-0608], [SC-01]
#[test]
fn compacts_tier_bc_docstrings_preserving_summary_and_realises(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new()?;
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir)?;
    let rs_file = src_dir.join("internal_worker.rs");
    let verbose_tier_bc = concat!(
        "/// Computes internal worker step.\r\n",
        "/// ## 1. Safety Intent & Regulatory Scope\r\n",
        "/// Safety & Traceability: Realises: [REQ-0501], [SC-01]\r\n",
        "/// ## 2. Invariants & Mathematical Properties\r\n",
        "/// Contract: Deterministic step evaluation.\r\n",
        "/// ## 6. Failure Modes & Fault Tolerance\r\n",
        "/// Bounds & Faults: O(1) time and space.\r\n",
        "fn run_internal_step() -> usize {\r\n",
        "    7\r\n",
        "}\r\n",
    );
    fs::write(&rs_file, verbose_tier_bc)?;

    let modified_count = compact_tier_bc_tree(tmp.path(), "modular_codegen/generated")?;
    assert_eq!(modified_count, 1);

    let compacted = fs::read_to_string(&rs_file)?;
    assert!(compacted.contains("Computes internal worker step."));
    assert!(compacted.contains("Realises: [REQ-0501], [SC-01]"));
    assert!(!compacted.contains("## 1. Safety Intent"));
    assert!(!compacted.contains("## 2. Invariants"));
    assert!(!compacted.contains('\r'));
    Ok(())
}

/// Verifies that `compact_tier_bc_tree` preserves Tier-A 7-section docblocks on `pub` items and skips `skip_generated_rel` (`TD-19`).
/// Realises: [REQ-0501], [REQ-0608], [SC-01]
#[test]
fn preserves_tier_a_seven_section_docblocks_and_skips_generated_tree(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new()?;
    let src_dir = tmp.path().join("src");
    let gen_dir = src_dir.join("modular_codegen/generated");
    fs::create_dir_all(&gen_dir)?;

    let tier_a_file = src_dir.join("public_api.rs");
    let tier_a_source = concat!(
        "/// [DOC-TIER: A] Executes public safety-critical operation.\n",
        "/// ## 1. Safety Intent & Regulatory Scope\n",
        "/// Safety & Traceability: Realises: [REQ-0501]\n",
        "/// ## 2. Invariants & Mathematical Properties\n",
        "/// Contract: Preserves state invariants.\n",
        "/// ## 3. Preconditions\n",
        "/// ## 4. Postconditions\n",
        "/// ## 5. Algorithmic Complexity Bounds\n",
        "/// ## 6. Failure Modes & Fault Tolerance\n",
        "/// Bounds & Faults: O(1) time.\n",
        "/// ## 7. Realization & Traceability\n",
        "pub fn exported_operation() -> bool {\n",
        "    true\n",
        "}\n",
    );
    fs::write(&tier_a_file, tier_a_source)?;

    let gen_file = gen_dir.join("generated_mod.rs");
    let gen_source = concat!(
        "/// Generated helper function.\n",
        "/// ## 1. Safety Intent & Regulatory Scope\n",
        "/// Safety & Traceability: Realises: [REQ-GEN-01]\n",
        "/// Contract: Generated contract.\n",
        "fn generated_helper() -> u8 {\n",
        "    1\n",
        "}\n",
    );
    fs::write(&gen_file, gen_source)?;

    let modified_count = compact_tier_bc_tree(tmp.path(), "modular_codegen/generated")?;
    assert_eq!(modified_count, 0);
    assert_eq!(fs::read_to_string(&tier_a_file)?, tier_a_source);
    assert_eq!(fs::read_to_string(&gen_file)?, gen_source);
    Ok(())
}

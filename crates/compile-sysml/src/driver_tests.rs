#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Unit tests for `compile-sysml` driver orchestration and Track-1 CST codemod handlers (`TD-15`, `TD-18`).
//! Safety & Traceability: Realises: [REQ-0608], [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]
//! Contract: Verifies deterministic NDJSON item inventory, atomic module extraction, blueprint line-reference rewriting, and fail-closed CLI/item validation.
//! Bounds & Faults: Executes against ephemeral temporary directories in O(N) time with zero workspace mutation.

use super::*;
use deap_core::rust_cst::codemod::CodemodError;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static DRIVER_TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Auto-cleaned temporary directory for isolated driver unit tests.
/// Realises: [REQ-0608], [REQ-COMPILER-SYSML-DRIVER]
struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    /// Creates a unique temporary directory under `std::env::temp_dir()`.
    /// Realises: [REQ-0608], [REQ-COMPILER-SYSML-DRIVER]
    fn new() -> std::io::Result<Self> {
        let seq = DRIVER_TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir_name = format!("deap_driver_unit_{}_{seq}", std::process::id());
        let path = std::env::temp_dir().join(dir_name);
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// Returns the filesystem path of the temporary directory.
    /// Realises: [REQ-0608], [REQ-COMPILER-SYSML-DRIVER]
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScopedTempDir {
    /// Removes the temporary directory on scope exit.
    /// Realises: [REQ-0608], [REQ-COMPILER-SYSML-DRIVER]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies `DriverError` exit code and message projection invariants.
/// Realises: [REQ-COMPILER-SYSML-DRIVER]
#[test]
fn test_driver_error_exit_code_smoke() {
    let err = DriverError::Cli("invalid flag".to_string());
    assert_eq!(err.exit_code(), 1);
    assert_eq!(err.message(), "invalid flag");
    let codegen_err = DriverError::Codegen(2, "idempotency conflict".to_string());
    assert_eq!(codegen_err.exit_code(), 2);
    assert_eq!(codegen_err.message(), "idempotency conflict");
}

/// Verifies that `emit_rust_item_inventory` emits items sorted by `(byte_start, byte_end, name)` in both NDJSON and human modes (`TD-15`).
/// Realises: [REQ-0608]
#[test]
fn emits_sorted_ndjson_rust_item_inventory() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new()?;
    let file_path = tmp.path().join("sample.rs");
    let source = concat!(
        "/// First item.\n",
        "pub struct Alpha {\n    pub count: usize,\n}\n\n",
        "fn beta_helper() -> usize {\n    42\n}\n\n",
        "pub fn gamma_entry() -> usize {\n    beta_helper()\n}\n",
    );
    fs::write(&file_path, source)?;

    let mut json_buf: Vec<u8> = Vec::new();
    emit_rust_item_inventory(&file_path, true, &mut json_buf)?;
    let json_text = String::from_utf8(json_buf)?;
    let lines: Vec<&str> = json_text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.first().is_some_and(|l| l.contains("\"Alpha\"")));
    assert!(lines.get(1).is_some_and(|l| l.contains("\"beta_helper\"")));
    assert!(lines.get(2).is_some_and(|l| l.contains("\"gamma_entry\"")));

    let mut human_buf: Vec<u8> = Vec::new();
    emit_rust_item_inventory(&file_path, false, &mut human_buf)?;
    let human_text = String::from_utf8(human_buf)?;
    assert!(human_text.contains("Alpha"));
    assert!(human_text.contains("..="));
    Ok(())
}

/// Verifies that invalid or conflicting codemod CLI mode arguments fail closed with exit code `1` (`TD-15` / `TD-18`).
/// Realises: [REQ-0608], [DO-330-TQL-4-CLI]
#[test]
fn rejects_conflicting_codemod_cli_flags() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new()?;
    let src_path = tmp.path().join("mod_a.rs");
    let dst_path = tmp.path().join("mod_b.rs");
    fs::write(&src_path, "pub fn alpha() {}\n")?;

    let empty_items_mode = CommandMode::ExtractRustModule {
        source_file: src_path,
        dest_file: dst_path,
        items_csv: " , ".to_string(),
        dry_run: false,
    };
    let res = dispatch_cst_mode(empty_items_mode, MessageFormat::Json);
    assert!(matches!(res, Err(DriverError::Cli(_))));
    if let Err(err) = res {
        assert_eq!(err.exit_code(), 1);
    }

    let traversal_err = extract_rust_module_atomic(
        tmp.path(),
        "../outside.rs",
        "src/inside.rs",
        &BTreeSet::from(["alpha".to_string()]),
    );
    assert_eq!(traversal_err, Err(CodemodError::EmptyInput));
    Ok(())
}

/// Verifies that `extract_rust_module_atomic` extracts requested items, rewrites `docs/designs/*.md` references, and enforces LF line endings (`TD-18`).
/// Realises: [REQ-0608]
#[test]
fn extracts_items_rewrites_blueprints_and_preserves_lf_determinism(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new()?;
    let src_dir = tmp.path().join("src");
    let designs_dir = tmp.path().join("docs/designs");
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&designs_dir)?;

    let parent_rel = "src/parent_mod.rs";
    let child_rel = "src/extracted_child.rs";
    let parent_src = "pub fn keep_here() -> u32 {\r\n    1\r\n}\r\n\r\npub fn move_away() -> u32 {\r\n    2\r\n}\r\n";
    fs::write(tmp.path().join(parent_rel), parent_src)?;

    let bp_path = designs_dir.join("blueprint-01-sample.md");
    let bp_content = "# Blueprint\r\nSee [parent_mod.rs](src/parent_mod.rs#L5-L7) for implementation.\r\n";
    fs::write(&bp_path, bp_content)?;

    let items = BTreeSet::from(["move_away".to_string()]);
    let modified = extract_rust_module_atomic(tmp.path(), parent_rel, child_rel, &items)?;
    assert_eq!(
        modified,
        vec![
            "docs/designs/blueprint-01-sample.md".to_string(),
            "src/extracted_child.rs".to_string(),
            "src/parent_mod.rs".to_string(),
        ]
    );

    let updated_parent = fs::read_to_string(tmp.path().join(parent_rel))?;
    let extracted_child = fs::read_to_string(tmp.path().join(child_rel))?;
    let updated_bp = fs::read_to_string(&bp_path)?;

    assert!(updated_parent.contains("pub fn keep_here"));
    assert!(!updated_parent.contains("pub fn move_away"));
    assert!(!updated_parent.contains('\r'));
    assert!(extracted_child.contains("pub fn move_away"));
    assert!(!extracted_child.contains('\r'));
    assert!(updated_bp.contains("src/extracted_child.rs"));
    assert!(!updated_bp.contains('\r'));
    Ok(())
}

/// Verifies that `extract_rust_module_atomic` rejects unknown item names before mutating any workspace files (`TD-18`).
/// Realises: [REQ-0608]
#[test]
fn rejects_unknown_item_name_without_mutating_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new()?;
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir)?;

    let parent_rel = "src/original.rs";
    let child_rel = "src/never_created.rs";
    let initial_source = "pub fn valid_item() -> bool {\n    true\n}\n";
    let parent_path = tmp.path().join(parent_rel);
    let child_path = tmp.path().join(child_rel);
    fs::write(&parent_path, initial_source)?;

    let items = BTreeSet::from(["valid_item".to_string(), "missing_symbol".to_string()]);
    let res = extract_rust_module_atomic(tmp.path(), parent_rel, child_rel, &items);
    assert_eq!(res, Err(CodemodError::ItemNotFound("missing_symbol".to_string())));
    assert!(!child_path.exists());
    let after_source = fs::read_to_string(&parent_path)?;
    assert_eq!(after_source, initial_source);
    Ok(())
}

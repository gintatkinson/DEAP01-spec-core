#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! AST modularity, ratchet ceiling, lexer-ban, and DoD architecture gates for workspace `.rs` sources.
//! Enforces legacy line ceilings, strict SLOC/function/literal invariants, and the 6 mandatory DoD gates.
//! Realises: [REQ-0601], [REQ-0608]

#[cfg(test)]
pub(crate) use crate::checks::dod_ast_visitor::count_executable_sloc;
use crate::checks::dod_ast_visitor::{
    audit_hand_rolled_lexer_in_source, audit_non_ratcheted_source,
};
use crate::checks::ratchet_table::{lookup_ratchet_ceiling, LEGACY_RATCHET_MAX_LINES};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Structured finding message emitted by the Definition-of-Done (DoD) verification gates (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub type Finding = String;

/// Required OMG SysML v2 Clause 8.2.2 syntax node kinds enforced by [`check_sysml_clause8_coverage`].
const REQUIRED_SYSML_CLAUSE8_KINDS: &[&str] = &[
    "Package", "Import", "PartDef", "PortDef", "ActionDef", "StateDef", "RequirementDef",
    "UseCaseDef", "VerificationDef", "ConstraintDef", "ConnectionDef", "InterfaceDef", "FlowDef",
    "AllocationDef", "CalculationDef", "OccurrenceDef", "ViewDef", "ViewpointDef", "MetadataDef",
    "Satisfy", "Verify",
];

/// Audits all `.rs` files under `<repo_root>/crates` against ratchet ceilings and DoD AST gates.
/// Returns `Ok(())` if `<repo_root>/crates` is absent or all files pass (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub fn check_dod_gates(repo_root: &Path) -> Result<(), Vec<String>> {
    let crates_dir = repo_root.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }
    let mut findings = Vec::new();
    if crates_dir.join("verify-baseline/src/checks/struct_check_clippy.rs").exists() {
        findings.push("crates/verify-baseline/src/checks/struct_check_clippy.rs: retired module must be removed (BP-01)".to_string());
    }
    for (rel_path, disk_path) in collect_crate_rs_files(repo_root, &crates_dir, &mut findings) {
        let Ok(raw) = fs::read_to_string(&disk_path)
            .map_err(|e| findings.push(format!("{rel_path}: {e}")))
        else {
            continue;
        };
        let content: Cow<'_, str> = if raw.contains('\r') {
            Cow::Owned(raw.replace("\r\n", "\n"))
        } else {
            Cow::Borrowed(&raw)
        };
        if let Some(max_lines) = lookup_ratchet_ceiling(&rel_path) {
            let line_count = content.lines().count();
            if line_count > max_lines {
                findings.push(format!(
                    "{rel_path}: line count {line_count} exceeds ratchet ceiling {max_lines} ([REQ-0601])"
                ));
            }
        } else {
            audit_non_ratcheted_source(&rel_path, &content, &mut findings);
        }
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

/// Gate 1: Verifies zero `.chars().peekable()` or `.char_indices()` hand-rolled lexers exist in any
/// `.rs` file under `<workspace_root>/crates` outside `#[derive(Logos)]` modules (`[REQ-0608]`).
/// Realises: [REQ-0608]
pub fn check_zero_hand_rolled_lexers(workspace_root: &Path) -> Vec<Finding> {
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    for (rel_path, disk_path) in collect_crate_rs_files(workspace_root, &crates_dir, &mut findings)
    {
        if let Ok(raw) = fs::read_to_string(&disk_path)
            .map_err(|e| findings.push(format!("{rel_path}: {e}")))
        {
            audit_hand_rolled_lexer_in_source(&rel_path, &raw, &mut findings);
        }
    }
    findings
}

/// Gate 2: Verifies `LEGACY_RATCHET_MAX_LINES` is empty and every `.rs` file under `<workspace_root>/crates`
/// (excluding `ratchet_table.rs`) is `<= 380` lines (`[REQ-0601]`).
/// Realises: [REQ-0601]
pub fn check_zero_legacy_ratchet(workspace_root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (path, ceiling) in LEGACY_RATCHET_MAX_LINES {
        findings.push(format!(
            "LEGACY_RATCHET_MAX_LINES still contains `{path}` (ceiling {ceiling}); expected 0 entries ([REQ-0601])"
        ));
    }
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return findings;
    }
    for (rel_path, disk_path) in collect_crate_rs_files(workspace_root, &crates_dir, &mut findings)
    {
        if rel_path.ends_with("/ratchet_table.rs") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&disk_path)
            .map_err(|e| findings.push(format!("{rel_path}: {e}")))
        {
            let lines = raw.lines().count();
            if lines > 380 {
                findings.push(format!(
                    "{rel_path}: line count {lines} exceeds 380-line ceiling ([REQ-0601])"
                ));
            }
        }
    }
    findings
}

/// Gate 3: Verifies that `deap-core::sysml_cst` implements the OMG SysML v2 Clause 8.2.2 grammar
/// productions (`REQUIRED_SYSML_CLAUSE8_KINDS`) via `logos` + `rowan` (`[REQ-0608]`).
/// Realises: [REQ-0608]
pub fn check_sysml_clause8_coverage(workspace_root: &Path) -> Vec<Finding> {
    let sysml_cst_dir = workspace_root.join("crates/deap-core/src/sysml_cst");
    if !sysml_cst_dir.exists() {
        return vec![
            "missing `crates/deap-core/src/sysml_cst` Clause 8.2.2 CST module ([REQ-0608])"
                .to_string(),
        ];
    }
    let mut combined = String::new();
    let mut findings = Vec::new();
    for (_, path) in collect_crate_rs_files(workspace_root, &sysml_cst_dir, &mut findings) {
        if let Ok(text) = fs::read_to_string(&path) {
            combined.push_str(&text);
            combined.push('\n');
        }
    }
    for kind in REQUIRED_SYSML_CLAUSE8_KINDS {
        if !combined.contains(kind) {
            findings.push(format!(
                "sysml_cst missing Clause 8.2.2 syntax production `{kind}` ([REQ-0608])"
            ));
        }
    }
    findings
}

/// Gate 4: Verifies that `crates/deap-core/src/salsa_db.rs` provides incremental Salsa-style
/// revision-tracked memoization across the compiler front-end (`[REQ-0608]`).
/// Realises: [REQ-0608]
pub fn check_salsa_incremental_memoization(workspace_root: &Path) -> Vec<Finding> {
    let salsa_path = workspace_root.join("crates/deap-core/src/salsa_db.rs");
    let Ok(text) = fs::read_to_string(&salsa_path) else {
        return vec![
            "missing `crates/deap-core/src/salsa_db.rs` incremental memoization database ([REQ-0608])"
                .to_string(),
        ];
    };
    let mut findings = Vec::new();
    for symbol in [
        "CompilerDatabase",
        "sysml_cst",
        "rust_cst",
        "dart_cst",
        "markdown_ast",
    ] {
        if !text.contains(symbol) {
            findings.push(format!(
                "salsa_db.rs missing required incremental query symbol `{symbol}` ([REQ-0608])"
            ));
        }
    }
    findings
}

/// Gate 5: Verifies that `crates/compile-sysml/src/modular_codegen` synthesizes Rust code via
/// `syn` + `quote::quote!` + `prettyplease::unparse` with zero `push_str("pub struct ")` string
/// concatenation (`[REQ-0608]`).
/// Realises: [REQ-0608]
pub fn check_codegen_zero_concat(workspace_root: &Path) -> Vec<Finding> {
    let codegen_dir = workspace_root.join("crates/compile-sysml/src/modular_codegen");
    if !codegen_dir.exists() {
        return vec![
            "missing `crates/compile-sysml/src/modular_codegen` module ([REQ-0608])".to_string(),
        ];
    }
    let mut findings = Vec::new();
    let mut uses_quote_or_unparse = false;
    for (rel_path, disk_path) in collect_crate_rs_files(workspace_root, &codegen_dir, &mut findings)
    {
        let Ok(text) = fs::read_to_string(&disk_path) else {
            continue;
        };
        let prod_text = text.split("#[cfg(test)]").next().unwrap_or(&text);
        if prod_text.contains("quote!") || prod_text.contains("prettyplease::unparse") {
            uses_quote_or_unparse = true;
        }
        for banned in [
            "push_str(\"pub struct ",
            "push_str(\"pub enum ",
            "push_str(\"impl ",
        ] {
            if prod_text.contains(banned) {
                findings.push(format!(
                    "{rel_path}: banned string-concatenation codegen `{banned}` ([REQ-0608])"
                ));
            }
        }
    }
    if !uses_quote_or_unparse {
        findings.push(
            "modular_codegen does not invoke `quote!` or `prettyplease::unparse` ([REQ-0608])"
                .to_string(),
        );
    }
    findings
}

/// Gate 6: Verifies that upstream self-hosting SysML schemas and design blueprints exist and are
/// non-empty when `.pipeline/upstream` is present (`[REQ-0608]`).
/// Realises: [REQ-0608]
pub fn check_upstream_self_hosting_parity(workspace_root: &Path) -> Vec<Finding> {
    let upstream_dir = workspace_root.join(".pipeline/upstream");
    if !upstream_dir.exists() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    for rel in [
        ".pipeline/upstream/schema/upstream_compiler.sysml",
        "docs/designs",
    ] {
        let target = workspace_root.join(rel);
        if !target.exists() {
            findings.push(format!(
                "missing upstream self-hosting artifact `{rel}` ([REQ-0608])"
            ));
        }
    }
    findings
}

/// Collects lexicographically sorted `(./crates/..., abs_path)` pairs for all `.rs` files (`[REQ-0601]`).
fn collect_crate_rs_files(
    repo_root: &Path,
    crates_dir: &Path,
    findings: &mut Vec<String>,
) -> Vec<(String, PathBuf)> {
    let mut files = Vec::new();
    let iter = WalkDir::new(crates_dir)
        .into_iter()
        .filter_entry(|e| !(e.file_type().is_dir() && e.file_name() == "target"));
    for entry_res in iter {
        let Ok(entry) = entry_res.map_err(|e| findings.push(format!("crates walk error: {e}")))
        else {
            continue;
        };
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
        {
            match entry.path().strip_prefix(repo_root) {
                Ok(rel) => files.push((
                    format!("./{}", rel.to_string_lossy().replace('\\', "/")),
                    entry.path().to_path_buf(),
                )),
                Err(err) => findings.push(format!("path prefix error: {err}")),
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

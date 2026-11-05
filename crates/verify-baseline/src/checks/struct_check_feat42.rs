//! Check 34 Layer 1 compiler AST purity gate, FEAT-42 design blueprint check, and blueprint `crates/...` path validator.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE], [REQ-CHECK-34-FEAT42-BLUEPRINT-COMPLIANCE]
//! Contract: Audits `AUDITED_MODULES` for AST purity, validates FEAT-42 design blueprint compliance, and checks `crates/...` path grounding.
//! Bounds & Faults: Single-pass directory and line walks; fails closed with sorted diagnostic vectors.

use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::struct_check_audit::audit_source_purity;
use super::struct_check_grounding::{is_valid_workspace_crate_name, BlueprintLineScanCtx};
use super::struct_check_purity::AUDITED_MODULES;

/// Check 34 Layer 1: Self-hosting compiler AST purity gate.
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE].
pub fn check_compiler_ast_purity(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for module in AUDITED_MODULES {
        let stem_rel = module.workspace_stem;
        let single_file = repo_root.join(format!("{stem_rel}.rs"));
        let module_dir = repo_root.join(stem_rel);
        let has_file = single_file.is_file();
        let has_dir = module_dir.is_dir();

        if has_file {
            audit_mandatory_file_path(&single_file, repo_root, &mut violations);
        }
        if has_dir {
            audit_decomposed_module_dir(&module_dir, repo_root, stem_rel, &mut violations);
        }
        if !has_file && !has_dir {
            violations.push(format!(
                "Check 34: Missing mandatory audited file '{}.rs'",
                stem_rel
            ));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Audits a single mandatory Rust file on disk against all Check 34 Layer 1 purity rules.
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE]
fn audit_mandatory_file_path(file_path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    let rel = file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path)
        .display()
        .to_string();
    match fs::read_to_string(file_path) {
        Ok(content) => {
            if let Err(errs) = audit_source_purity(&content, &rel, true) {
                violations.extend(errs);
            }
        }
        Err(err) => {
            violations.push(format!(
                "Check 34: Unable to read audited file '{}': {}",
                rel, err
            ));
        }
    }
}

/// Audits every `.rs` file inside a decomposed `<module>/` directory, failing closed if empty.
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE]
fn audit_decomposed_module_dir(
    module_dir: &Path,
    repo_root: &Path,
    stem_rel: &str,
    violations: &mut Vec<String>,
) {
    let mut rs_files = Vec::new();
    for entry_res in WalkDir::new(module_dir) {
        match entry_res {
            Ok(entry) => {
                let path = entry.path();
                if entry.file_type().is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                    rs_files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                violations.push(format!(
                    "Check 34: Unable to traverse audited module directory '{}': {}",
                    stem_rel, err
                ));
            }
        }
    }
    rs_files.sort();
    if rs_files.is_empty() {
        violations.push(format!(
            "Check 34: Missing mandatory audited file '{}.rs'",
            stem_rel
        ));
        return;
    }
    for rs_path in &rs_files {
        audit_mandatory_file_path(rs_path, repo_root, violations);
    }
}

/// Verify FEAT-42 design blueprint compliance (refs #459).
/// Realises: [REQ-CHECK-34-FEAT42-BLUEPRINT-COMPLIANCE].
pub fn check_feat42_design_blueprint_compliance(repo_root: &Path) -> Result<(), Vec<String>> {
    let feat42_path = repo_root
        .join("docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
    if !feat42_path.is_file() {
        return Err(vec![format!(
            "Missing mandatory design blueprint: {}",
            feat42_path.display()
        )]);
    }
    let content = match fs::read_to_string(&feat42_path) {
        Ok(c) => c,
        Err(e) => {
            return Err(vec![format!(
                "Failed to read design blueprint {}: {e}",
                feat42_path.display()
            )]);
        }
    };
    let mut violations = Vec::new();
    validate_feat42_sections(&content, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Validates FEAT-42 Section 2.1 and Section 3.2 architectural contracts.
/// Realises: [REQ-CHECK-34-FEAT42-BLUEPRINT-COMPLIANCE]
fn validate_feat42_sections(content: &str, violations: &mut Vec<String>) {
    if content.contains("sysml-core") {
        violations.push(
            "FEAT-42 contains forbidden reference to non-existent 'sysml-core' crate (must reference 'deap-core')"
                .to_string(),
        );
    }
    if !content.contains("ingest-sysml [OPTIONS] [--source <DIR> | --schema <PATH>]") {
        violations.push(
            "FEAT-42 Section 2.1 must specify CLI syntax: ingest-sysml [OPTIONS] [--source <DIR> | --schema <PATH>]"
                .to_string(),
        );
    }
    let has_optional_source = content.contains("| `--source`")
        && (content.contains("| No (mutually exclusive with `--schema`)")
            || content.contains("| Path (`<DIR>`) | No"));
    if !has_optional_source {
        violations.push(
            "FEAT-42 Section 2.1 must declare --source as optional (No), preserving backward compatibility"
                .to_string(),
        );
    }
    if !content.contains("| `--schema`") {
        violations.push(
            "FEAT-42 Section 2.1 must specify --schema in CLI option definitions table".to_string(),
        );
    }
    if !content
        .lines()
        .any(|l| l.contains("--format") && l.contains("`auto`"))
    {
        violations
            .push("FEAT-42 Section 2.1 must specify --format default value as `auto`".to_string());
    }
    if !content.contains("seen_docblocks") {
        violations.push(
            "FEAT-42 Section 3.2 must mandate docstring uniqueness under 'seen_docblocks'"
                .to_string(),
        );
    }
    if !content.contains("Check 34") {
        violations.push("FEAT-42 Section 3.2 must cite Check 34 uniqueness contract".to_string());
    }
    let has_interpolation = (content.contains("interpolate") || content.contains("Interpolate"))
        && (1..=6).all(|n| content.contains(&format!("Section {n}")));
    if !has_interpolation {
        violations.push(
            "FEAT-42 Section 3.2 must mandate per-function AST signature interpolation across Sections 1 through 6"
                .to_string(),
        );
    }
}

/// Extracts and validates concrete `crates/...` path tokens on a blueprint line against disk.
/// Realises: [REQ-CHECK-37-LINE-CRATES-PATHS]
pub(crate) fn check_blueprint_line_crates_paths(
    ctx: BlueprintLineScanCtx<'_>,
    line: &str,
    violations: &mut Vec<String>,
) {
    let bytes = line.as_bytes();
    for (start_idx, _) in line.match_indices("crates/") {
        if start_idx > 0
            && bytes
                .get(start_idx.saturating_sub(1))
                .is_some_and(|&prev| prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'-')
        {
            continue;
        }
        let Some(tail) = line.get(start_idx..) else {
            continue;
        };
        let end_offset = tail
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '`' | '\'' | '"' | '(' | ')' | '[' | ']' | ';' | ',' | '|'
                    )
            })
            .unwrap_or(tail.len());
        let Some(raw_token) = tail.get(..end_offset) else {
            continue;
        };
        validate_extracted_crates_token(ctx, raw_token, violations);
    }
}

/// Validates a single extracted `crates/...` path token against workspace crate and on-disk rules.
/// Realises: [REQ-CHECK-37-LINE-CRATES-PATHS]
fn validate_extracted_crates_token(
    ctx: BlueprintLineScanCtx<'_>,
    raw_token: &str,
    violations: &mut Vec<String>,
) {
    if raw_token.contains(['*', '<', '>', '{', '}', '$']) || raw_token.contains("...") {
        return;
    }
    let without_hash = raw_token.split('#').next().unwrap_or(raw_token);
    let without_colon = without_hash.split(':').next().unwrap_or(without_hash);
    let cleaned = without_colon.trim_end_matches(['.', ',', ':', ';', '/']);
    let Some(after_crates) = cleaned.strip_prefix("crates/") else {
        return;
    };
    let crate_name = after_crates.split('/').next().unwrap_or("");
    if crate_name.is_empty() {
        return;
    }

    let is_downstream_crate = !is_upstream_compiler(ctx.repo_root)
        && ctx.repo_root.join("crates").join(crate_name).is_dir();
    if !is_valid_workspace_crate_name(crate_name) && !is_downstream_crate {
        violations.push(format!(
            "{}:{}: ungrounded crate '{}' in blueprint path '{}' (must be one of deap-core, verify-baseline, compile-sysml, ingest-sysml, assemble-conops, reconcile-backlog)",
            ctx.rel_file, ctx.line_no, crate_name, cleaned
        ));
        return;
    }

    let is_design_proposed_path = ctx.allow_proposed_design_paths
        && (ctx.repo_root.join("crates").join(crate_name).is_dir()
            || (ctx.allow_unbuilt_workspace_paths && is_valid_workspace_crate_name(crate_name)));
    let decomposed_exists = cleaned.strip_suffix(".rs").is_some_and(|stem| {
        ctx.repo_root.join(format!("{stem}/mod.rs")).exists()
            || stem
                .strip_suffix("_emitter")
                .is_some_and(|s| ctx.repo_root.join(format!("{s}/mod.rs")).exists())
    });
    if !is_design_proposed_path && !ctx.repo_root.join(cleaned).exists() && !decomposed_exists {
        violations.push(format!(
            "{}:{}: non-existent workspace path '{}' referenced in architecture blueprint",
            ctx.rel_file, ctx.line_no, cleaned
        ));
    }
}

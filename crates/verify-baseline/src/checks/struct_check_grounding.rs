//! Check 37 architecture and design blueprint grounding verification gate.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-37-BLUEPRINT-GROUNDING]
//! Contract: Verifies canonical architecture blueprints and `docs/designs/blueprint-*.md` against workspace crates and paths.
//! Bounds & Faults: Single-pass file and line scan; fails closed with sorted diagnostic vectors.

use deap_core::rules::CanonicalArchitectureBlueprint;
use deap_core::workspace::{is_upstream_compiler, is_upstream_compiler_workspace};
use std::fs;
use std::path::{Path, PathBuf};

use super::struct_check_feat42::check_blueprint_line_crates_paths;

/// Check 37: Verify architecture blueprint grounding in workspace crates and on-disk paths.
/// Realises: [REQ-CHECK-37-BLUEPRINT-GROUNDING]
pub fn check_blueprint_grounding(repo_root: &Path) -> Result<(), Vec<String>> {
    let blueprints_dir = repo_root
        .join("docs")
        .join("architecture")
        .join("blueprints");
    let designs_dir = repo_root.join("docs").join("designs");
    if !blueprints_dir.is_dir() && !designs_dir.is_dir() && !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();
    if is_upstream_compiler(repo_root) && !blueprints_dir.is_dir() {
        violations.push(
            "Missing docs/architecture/blueprints directory in upstream specification compiler repository."
                .to_string(),
        );
        return Err(violations);
    }

    let mut md_files = Vec::new();
    if blueprints_dir.is_dir() {
        collect_architecture_blueprint_files(
            repo_root,
            &blueprints_dir,
            &mut md_files,
            &mut violations,
        );
    }
    if designs_dir.is_dir() {
        collect_design_blueprint_files(&designs_dir, &mut md_files, &mut violations);
    }

    md_files.sort();
    for md_path in &md_files {
        scan_blueprint_markdown_file(repo_root, md_path, &mut violations);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Collects architecture blueprint markdown files and checks canonical completeness in upstream workspaces.
/// Realises: [REQ-CHECK-37-BLUEPRINT-GROUNDING]
fn collect_architecture_blueprint_files(
    repo_root: &Path,
    blueprints_dir: &Path,
    md_files: &mut Vec<PathBuf>,
    violations: &mut Vec<String>,
) {
    if is_upstream_compiler(repo_root) {
        for bp in CanonicalArchitectureBlueprint::ALL {
            let bp_path = blueprints_dir.join(bp.as_str());
            if !bp_path.is_file() {
                violations.push(format!(
                    "Missing canonical architecture blueprint: docs/architecture/blueprints/{}",
                    bp.as_str()
                ));
            }
        }
    }
    match fs::read_dir(blueprints_dir) {
        Ok(entries) => {
            for entry_res in entries {
                match entry_res {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                            md_files.push(path);
                        }
                    }
                    Err(err) => {
                        violations.push(format!(
                            "docs/architecture/blueprints: directory entry read error: {}",
                            err
                        ));
                    }
                }
            }
        }
        Err(err) => {
            violations.push(format!(
                "docs/architecture/blueprints: unable to read directory: {}",
                err
            ));
        }
    }
}

/// Collects `blueprint-*.md` files from `docs/designs/`.
/// Realises: [REQ-CHECK-37-BLUEPRINT-GROUNDING]
fn collect_design_blueprint_files(
    designs_dir: &Path,
    md_files: &mut Vec<PathBuf>,
    violations: &mut Vec<String>,
) {
    match fs::read_dir(designs_dir) {
        Ok(entries) => {
            for entry_res in entries {
                match entry_res {
                    Ok(entry) => {
                        let path = entry.path();
                        let is_blueprint_md = path.is_file()
                            && path.extension().is_some_and(|ext| ext == "md")
                            && path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .is_some_and(|name| name.starts_with("blueprint-"));
                        if is_blueprint_md {
                            md_files.push(path);
                        }
                    }
                    Err(err) => {
                        violations
                            .push(format!("docs/designs: directory entry read error: {}", err));
                    }
                }
            }
        }
        Err(err) => {
            violations.push(format!("docs/designs: unable to read directory: {}", err));
        }
    }
}

/// Context carrying per-line blueprint scan flags.
/// Realises: [REQ-CHECK-37-LINE-CRATES-PATHS]
#[derive(Clone, Copy)]
pub(crate) struct BlueprintLineScanCtx<'a> {
    pub(crate) repo_root: &'a Path,
    pub(crate) rel_file: &'a str,
    pub(crate) line_no: usize,
    pub(crate) allow_proposed_design_paths: bool,
    pub(crate) allow_unbuilt_workspace_paths: bool,
}

/// Scans a single architecture blueprint Markdown file for Check 37 grounding violations.
/// Realises: [REQ-CHECK-37-SCAN-BLUEPRINT-FILE]
fn scan_blueprint_markdown_file(repo_root: &Path, md_path: &Path, violations: &mut Vec<String>) {
    let rel = md_path
        .strip_prefix(repo_root)
        .unwrap_or(md_path)
        .display()
        .to_string();

    let content = match fs::read_to_string(md_path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!("{}: unable to read blueprint file: {}", rel, err));
            return;
        }
    };

    let is_design_blueprint = rel.starts_with("docs/designs/");
    let is_compiler_workspace = is_upstream_compiler_workspace(repo_root);
    let is_spec_core = is_upstream_compiler(repo_root) && !is_compiler_workspace;
    let allow_proposed_design_paths =
        is_design_blueprint && (is_spec_core || !is_approved_design_blueprint(&content));
    let allow_unbuilt_workspace_paths = is_spec_core;

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        if !is_design_blueprint && line.contains('\u{2014}') {
            violations.push(format!(
                "{}:{}: forbidden Unicode em dash (\\u2014) in architecture blueprint",
                rel, line_no
            ));
        }
        check_blueprint_line_fictional_crates(&rel, line_no, line, violations);
        check_blueprint_line_retired_tools(&rel, line_no, line, violations);
        let ctx = BlueprintLineScanCtx {
            repo_root,
            rel_file: &rel,
            line_no,
            allow_proposed_design_paths,
            allow_unbuilt_workspace_paths,
        };
        check_blueprint_line_crates_paths(ctx, line, violations);
    }
}

/// Determines whether a design blueprint Markdown payload declares an `Approved` lifecycle status.
/// Realises: [REQ-CHECK-37-APPROVED-BLUEPRINT-STATUS]
fn is_approved_design_blueprint(content: &str) -> bool {
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.contains("**Status**") && trimmed.contains('|') {
            let mut seen_status_key = false;
            for cell in trimmed.split('|') {
                let clean = cell.trim().trim_matches(|c: char| c == '`' || c == '*');
                if seen_status_key && !clean.is_empty() {
                    if clean.eq_ignore_ascii_case("approved") {
                        return true;
                    }
                    break;
                }
                if clean.eq_ignore_ascii_case("status") {
                    seen_status_key = true;
                }
            }
        } else if let Some((_, rhs)) = trimmed
            .split_once("**Status**:")
            .or_else(|| trimmed.split_once("Status:"))
        {
            let clean = rhs
                .trim()
                .trim_matches(|c: char| c == '|' || c == '`' || c == '*' || c.is_whitespace());
            if clean.eq_ignore_ascii_case("approved") {
                return true;
            }
        }
    }
    false
}

/// Checks whether `line` contains `token` bounded on both sides by non-identifier characters.
/// Realises: [REQ-CHECK-37-BOUNDED-TOKEN-MATCH]
pub(crate) fn contains_bounded_token(line: &str, token: &str) -> bool {
    let bytes = line.as_bytes();
    let token_len = token.len();
    for (idx, _) in line.match_indices(token) {
        let before_ok = if idx == 0 {
            true
        } else {
            match bytes.get(idx.saturating_sub(1)) {
                Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
                None => true,
            }
        };
        let after_ok = match bytes.get(idx.saturating_add(token_len)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Checks whether a blueprint line states an explicit negative prohibition or retirement rule.
/// Realises: [REQ-CHECK-37-NEGATIVE-RULE-CONTEXT]
fn is_negative_blueprint_rule_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("forbidden")
        || lower.contains("prohibited")
        || lower.contains("retired")
        || lower.contains("nonexistent")
        || lower.contains("non-existent")
        || lower.contains("fictional")
}

/// Scans a blueprint line for references to fictional, non-existent DEAP crate names.
/// Realises: [REQ-CHECK-37-DETECT-FICTIONAL-CRATE]
fn check_blueprint_line_fictional_crates(
    rel_file: &str,
    line_no: usize,
    line: &str,
    violations: &mut Vec<String>,
) {
    if is_negative_blueprint_rule_line(line) {
        return;
    }
    for fictional in [
        "deap-ast",
        "deap-codegen",
        "deap-harness",
        "deap-cli",
        "deap_ast",
        "deap_codegen",
        "deap_harness",
        "deap_cli",
    ] {
        if contains_bounded_token(line, fictional) {
            violations.push(format!(
                "{}:{}: references fictional crate '{}'",
                rel_file, line_no, fictional
            ));
        }
    }
}

/// Scans a blueprint line for references to retired Python or shell tools outside negative rules.
/// Realises: [REQ-CHECK-37-DETECT-RETIRED-TOOL]
fn check_blueprint_line_retired_tools(
    rel_file: &str,
    line_no: usize,
    line: &str,
    violations: &mut Vec<String>,
) {
    if is_negative_blueprint_rule_line(line) {
        return;
    }
    for tool in [
        "verify_downstream",
        "sysmlv2_ingest",
        "reconcile_backlog.py",
        "parity_auditor",
        "setup_git_hooks.sh",
    ] {
        if line.contains(tool) {
            violations.push(format!(
                "{}:{}: references retired tool '{}'",
                rel_file, line_no, tool
            ));
        }
    }
}

/// Checks whether `crate_name` matches one of the 6 canonical workspace crates in `Cargo.toml`.
/// Realises: [REQ-CHECK-37-VALID-WORKSPACE-CRATE]
pub(crate) fn is_valid_workspace_crate_name(crate_name: &str) -> bool {
    matches!(
        crate_name,
        "deap-core"
            | "verify-baseline"
            | "compile-sysml"
            | "ingest-sysml"
            | "assemble-conops"
            | "reconcile-backlog"
    )
}

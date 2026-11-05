//! Checks 21 and 22: Semantic Diagram-to-AST Topology Parity and Physical Invariant Prose Gates.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-21], [REQ-PARITY-GATE-22]
//! Contract: Verifies Mermaid diagram headers/class symbols and prose `[INV-*]` assertions against SysML AST.
//! Bounds & Faults: Deterministic sorted file walking; fails closed on unreadable files or ungrounded symbols.

use super::parity_check_discovery::{discover_sysml_model_text, TargetScanDir};
use super::parity_check_icd_ast::collect_part_def_names;
use super::parity_gates::{has_colon_method_syntax, DiagramType, ParityOutcome};
use deap_core::rules::EXCLUDED_DIRS;
use deap_core::sysml_ast::{
    parse_action_defs, parse_constraint_defs, parse_part_defs, parse_port_defs,
};
use deap_core::workspace::is_upstream_compiler;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Check 21: Semantic Diagram-to-AST Topology Parity Gate.
/// Realises: [REQ-PARITY-GATE-21]
pub fn check_semantic_diagram_ast_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let Some(model_str) = discover_sysml_model_text(repo_root) else {
        return Ok(ParityOutcome::PendingOrClean);
    };

    let mut declared_symbols = BTreeSet::new();
    for p in parse_part_defs(&model_str) {
        collect_part_def_names(&p, &mut declared_symbols);
    }
    for p in parse_port_defs(&model_str) {
        declared_symbols.insert(p.name);
    }
    for a in parse_action_defs(&model_str) {
        declared_symbols.insert(a.name);
    }

    let mut errors = Vec::new();
    let is_upstream = is_upstream_compiler(repo_root);

    for &scan_dir in TargetScanDir::ALL {
        if scan_dir == TargetScanDir::Designs && is_upstream {
            continue;
        }
        let dir = scan_dir.resolve(repo_root);
        if !dir.is_dir() {
            continue;
        }
        let md_files = collect_check21_markdown_files(&dir, &mut errors);
        for path in md_files {
            audit_markdown_file_diagrams(&path, repo_root, &declared_symbols, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Collects sorted Markdown files under `dir` for Check 21 diagram auditing.
/// Realises: [REQ-PARITY-GATE-21]
fn collect_check21_markdown_files(dir: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let mut md_files = Vec::new();
    for entry_res in WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == *ex) && name != "blueprints"
            } else {
                true
            }
        })
    {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!(
                    "Check 21 failed: I/O error walking directory '{}': {}",
                    dir.display(),
                    err
                ));
                continue;
            }
        };
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
            md_files.push(entry.path().to_path_buf());
        }
    }
    md_files.sort();
    md_files
}

/// Audits all fenced Mermaid blocks in a single Markdown file for Check 21 AST parity.
/// Realises: [REQ-PARITY-GATE-21]
fn audit_markdown_file_diagrams(
    path: &Path,
    repo_root: &Path,
    declared_symbols: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!(
                "Check 21 failed: Unable to read file '{}': {}",
                rel, e
            ));
            return;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let Some(&line_str) = lines.get(i) else {
            break;
        };
        if line_str.trim().starts_with("```mermaid") {
            i += 1;
            let mut block_lines = Vec::new();
            while let Some(&inner) = lines.get(i) {
                if inner.trim().starts_with("```") {
                    break;
                }
                block_lines.push(inner);
                i += 1;
            }
            audit_single_mermaid_block(&block_lines, &rel, declared_symbols, errors);
        }
        i += 1;
    }
}

/// Audits one extracted Mermaid block's header and classDiagram lines against `declared_symbols`.
/// Realises: [REQ-PARITY-GATE-21]
fn audit_single_mermaid_block(
    block_lines: &[&str],
    rel: &str,
    declared_symbols: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let Some(&first) = block_lines
        .iter()
        .find(|l| !l.trim().is_empty() && !l.trim().starts_with("%%"))
    else {
        return;
    };
    let trimmed = first.trim();
    let diag_type = DiagramType::parse_header(trimmed);
    if diag_type.is_none() {
        errors.push(format!(
            "Missing mandatory Mermaid diagram type header in {} (got: '{}')",
            rel, trimmed
        ));
    }
    if diag_type != Some(DiagramType::ClassDiagram) {
        return;
    }

    for line in block_lines {
        let s = line.trim();
        if (s.contains('{') || s.contains('}')) && !s.starts_with("class ") && s != "{" && s != "}"
        {
            errors.push(format!(
                "Mermaid syntax violation in {}: Curly braces '{{}}' inside class member line: '{}'.",
                rel, s
            ));
        }
        if has_colon_method_syntax(s) {
            errors.push(format!(
                "Mermaid syntax violation in {}: Colons ':' forbidden in class member line: '{}'. Use '+ReturnType methodName(Type arg)' spacing.",
                rel, s
            ));
        }
        if let Some(stripped) = s.strip_prefix("class ") {
            let class_name = stripped.trim().trim_end_matches('{').trim();
            if !class_name.is_empty()
                && !declared_symbols.contains(class_name)
                && !class_name.starts_with('<')
            {
                errors.push(format!(
                    "Topological drift in {}: Diagram references undeclared hardware/subsystem concept '{}'.",
                    rel, class_name
                ));
            }
        }
    }
}

/// Check 22: Physical Invariant Semantic Prose Gate.
/// Realises: [REQ-PARITY-GATE-22]
pub fn check_semantic_prose_invariants(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let model_text = discover_sysml_model_text(repo_root);
    let has_extracted = repo_root.join("schema").join("extracted").is_dir();

    if model_text.is_none() && !has_extracted {
        return Ok(ParityOutcome::PendingOrClean);
    }
    let model_str = model_text.unwrap_or_default();

    let mut declared_constraints = BTreeSet::new();
    for c in parse_constraint_defs(&model_str) {
        declared_constraints.insert(c.name);
    }

    let mut declared_parts = BTreeSet::new();
    for p in parse_part_defs(&model_str) {
        collect_part_def_names(&p, &mut declared_parts);
    }

    let mut errors = Vec::new();
    let scan_dirs = [
        TargetScanDir::Safety.resolve(repo_root),
        TargetScanDir::Conops.resolve(repo_root),
        TargetScanDir::Features.resolve(repo_root),
    ];

    for dir in &scan_dirs {
        if !dir.is_dir() {
            continue;
        }
        scan_dir_for_prose_invariants(dir, repo_root, &declared_constraints, &mut errors);
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Scans a single directory for Check 22 prose invariant assertions (`[INV-*]`).
/// Realises: [REQ-PARITY-GATE-22]
fn scan_dir_for_prose_invariants(
    dir: &Path,
    repo_root: &Path,
    declared_constraints: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(dir).sort_by_file_name().into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!(
                    "Check 22 failed: I/O error walking directory '{}': {}",
                    dir.display(),
                    err
                ));
                continue;
            }
        };
        if !entry.file_type().is_file() || !entry.path().extension().is_some_and(|e| e == "md") {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!(
                    "Check 22 failed: Unable to read file '{}': {}",
                    rel, e
                ));
                continue;
            }
        };
        audit_file_prose_invariants(&content, &rel, declared_constraints, errors);
    }
}

/// Audits lines of a Markdown file for ungrounded `[INV-*]` invariant assertions.
/// Realises: [REQ-PARITY-GATE-22]
fn audit_file_prose_invariants(
    content: &str,
    rel: &str,
    declared_constraints: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.is_empty() {
            continue;
        }
        if (trimmed.contains("Invariant:") || trimmed.contains("Physical Invariant:"))
            && trimmed.contains("[INV-")
        {
            if let Some(inv_id) = trimmed
                .find("[INV-")
                .and_then(|start| trimmed.get(start.saturating_add(1)..))
                .and_then(|slice| slice.find(']').and_then(|end| slice.get(..end)))
            {
                if !declared_constraints.is_empty() && !declared_constraints.contains(inv_id) {
                    errors.push(format!(
                        "Check 22 failed: Prose invariant assertion '{}' on line {} in '{}' is ungrounded in SysML model.",
                        inv_id, line_no, rel
                    ));
                }
            }
        }
    }
}

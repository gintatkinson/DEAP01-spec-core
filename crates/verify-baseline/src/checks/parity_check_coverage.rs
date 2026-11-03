//! Check 30A: Coverage Digest & Use Case Realization Gate (`check_coverage_digest`) and `use case def` parser.
//!
//! Safety & Traceability: Realises: [REQ-COVERAGE-DIGEST-GATE], [REQ-DUAL-SCHEMA-SSOT-PARITY]
//! Contract: Validates specification coverage references and forward `use case def` realizations in `docs/use-cases/`.
//! Bounds & Faults: Deterministic sorted WalkDir scanning; fails closed on phantom specs or unrealized use cases.

use super::parity_check_discovery::TargetScanDir;
use super::parity_gates::ParityOutcome;
use deap_core::sysml_ast::{find_matching_brace, mask_string_literals, strip_sysml_comments};
use deap_core::workspace::is_upstream_compiler;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Parses SysML v2 `use case def` declarations from source text into `(name, normalized_body)` pairs.
/// Realises: [REQ-COVERAGE-DIGEST-GATE], [REQ-DUAL-SCHEMA-SSOT-PARITY]
pub(crate) fn parse_use_case_defs(content: &str) -> Vec<(String, String)> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let bytes = masked.as_bytes();
    let len = bytes.len();
    let mut out = Vec::new();
    let mut idx = 0;

    while let Some(rel_pos) = masked.get(idx..).and_then(|s| s.find("use")) {
        let pos = idx + rel_pos;
        let before_ok = pos == 0
            || !bytes
                .get(pos - 1)
                .copied()
                .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !before_ok {
            idx = pos + 3;
            continue;
        }
        let Some((name, mut curr)) = parse_use_case_def_header(&clean, &masked, pos + 3) else {
            idx = (pos + 3).min(len);
            continue;
        };

        while curr < len {
            let b = bytes.get(curr).copied().unwrap_or(b';');
            if b == b'{' {
                if let Some(close_pos) = find_matching_brace(&clean, curr) {
                    let raw_body = clean.get(curr + 1..close_pos).unwrap_or("");
                    let norm_body = raw_body.split_whitespace().collect::<Vec<_>>().join(" ");
                    out.push((name, norm_body));
                    idx = close_pos + 1;
                } else {
                    idx = curr + 1;
                }
                break;
            } else if b == b';' {
                out.push((name, String::new()));
                idx = curr + 1;
                break;
            }
            curr += 1;
        }
        if curr >= len {
            break;
        }
    }
    out
}

/// Parses `case def <Name>` following a `use` keyword at byte offset `start`.
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
fn parse_use_case_def_header(clean: &str, masked: &str, start: usize) -> Option<(String, usize)> {
    let mut curr = start;
    let ws1 = masked
        .get(curr..)?
        .bytes()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    if ws1 == 0 {
        return None;
    }
    curr += ws1;
    if !masked.get(curr..)?.starts_with("case") {
        return None;
    }
    curr += 4;
    let ws2 = masked
        .get(curr..)?
        .bytes()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    if ws2 == 0 {
        return None;
    }
    curr += ws2;
    if !masked.get(curr..)?.starts_with("def") {
        return None;
    }
    curr += 3;
    let ws3 = masked
        .get(curr..)?
        .bytes()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    if ws3 == 0 {
        return None;
    }
    curr += ws3;
    let name_len = masked
        .get(curr..)?
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
        .count();
    let name = clean.get(curr..curr + name_len)?.trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some((name, curr + name_len))
}

/// Normalizes a use case identifier or Markdown string for forward realization matching in Check 30A.
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
pub(crate) fn normalize_uc_token(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_sep = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_sep = false;
        } else if !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    out.trim_matches('_').to_string()
}

/// Strips canonical SysML-to-Markdown use case numbering prefixes (`uc_ac_<digits>_` or `uc_<digits>_`).
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
pub(crate) fn strip_uc_numbering_prefix(norm: &str) -> &str {
    let after_prefix = norm
        .strip_prefix("uc_ac_")
        .or_else(|| norm.strip_prefix("uc_"));
    if let Some(rest) = after_prefix {
        let trimmed_digits = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        if trimmed_digits.len() < rest.len() {
            if let Some(slug) = trimmed_digits.strip_prefix('_') {
                if slug.len() >= 4 {
                    return slug;
                }
            }
        }
    }
    norm
}

/// Check 30A: Coverage Digest & Use Case Realization Gate (Gate 28A).
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
pub fn check_coverage_digest(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let (declared_specs, active_uc_files) =
        collect_declared_specs_and_uc_files(repo_root, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    if declared_specs.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    audit_phantom_spec_references(repo_root, &declared_specs, &mut errors);
    if !active_uc_files.is_empty() {
        audit_schema_use_case_realizations(repo_root, &active_uc_files, &mut errors);
    }
    crate::spec_audit::contract_and_schema_parity::audit_ast_node_spec_realizations(
        repo_root,
        &[],
        &repo_root.join(".pipeline/schema.sysml"),
        &mut errors,
    );
    errors.sort();
    errors.dedup();

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Collects declared `features/`, `use-cases/`, and `user-stories/` basenames and active use case paths.
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
fn collect_declared_specs_and_uc_files(
    repo_root: &Path,
    errors: &mut Vec<String>,
) -> (BTreeSet<String>, Vec<PathBuf>) {
    let mut declared_specs = BTreeSet::new();
    let mut active_uc_files = Vec::new();
    for spec_dir_kind in [
        TargetScanDir::Features,
        TargetScanDir::UseCases,
        TargetScanDir::UserStories,
    ] {
        let dir = spec_dir_kind.resolve(repo_root);
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(&dir).sort_by_file_name().into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!(
                        "Check 30A failed: I/O error walking directory '{}': {}",
                        dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") && name != "README.md" {
                    declared_specs.insert(name.to_ascii_lowercase());
                    if spec_dir_kind == TargetScanDir::UseCases && !name.starts_with('_') {
                        active_uc_files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    (declared_specs, active_uc_files)
}

/// Scans reports and agile specs for phantom `feat-*.md`, `uc-*.md`, or `us-*.md` references.
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
fn audit_phantom_spec_references(
    repo_root: &Path,
    declared_specs: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for scan_dir_kind in [
        TargetScanDir::Reports,
        TargetScanDir::Epics,
        TargetScanDir::Features,
        TargetScanDir::UseCases,
        TargetScanDir::UserStories,
    ] {
        let dir = scan_dir_kind.resolve(repo_root);
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(&dir).sort_by_file_name().into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!(
                        "Check 30A failed: I/O error walking directory '{}': {}",
                        dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if !entry.file_type().is_file()
                || !entry.path().extension().is_some_and(|ext| ext == "md")
                || entry.file_name().to_string_lossy() == "README.md"
            {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(repo_root)
                .unwrap_or(entry.path())
                .display()
                .to_string();
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("Check 30A failed: Unable to read '{}': {}", rel, e));
                    continue;
                }
            };
            for word in content.split_whitespace() {
                for part in word.split(['(', ')', '[', ']', '`', '"', '\'']) {
                    let clean = part.trim_matches(|c: char| {
                        !c.is_ascii_alphanumeric() && !matches!(c, '-' | '_' | '.' | '/')
                    });
                    let basename = clean.rsplit('/').next().unwrap_or(clean);
                    let bl = basename.to_ascii_lowercase();
                    if (bl.starts_with("feat-") || bl.starts_with("uc-") || bl.starts_with("us-"))
                        && bl.ends_with(".md")
                        && !declared_specs.contains(&bl)
                    {
                        errors.push(format!(
                            "Check 30A failed: Coverage digest or specification '{}' references undeclared specification file '{}'",
                            rel, basename
                        ));
                    }
                }
            }
        }
    }
}

/// Verifies that every declared SysML `use case def` is realized in `active_uc_files`.
/// Realises: [REQ-COVERAGE-DIGEST-GATE]
fn audit_schema_use_case_realizations(
    repo_root: &Path,
    active_uc_files: &[PathBuf],
    errors: &mut Vec<String>,
) {
    let mut schema_files = Vec::new();
    let pipeline_schema = repo_root.join(".pipeline").join("schema.sysml");
    if pipeline_schema.is_file() {
        schema_files.push(pipeline_schema);
    }
    let schema_dir = repo_root.join("schema");
    if schema_dir.is_dir() {
        for p in deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[
            &schema_dir,
        ]) {
            if p.extension().is_some_and(|ext| ext == "sysml") {
                schema_files.push(p);
            }
        }
    }

    let mut declared_uc_names = BTreeSet::new();
    for sf in &schema_files {
        if let Ok(content) = fs::read_to_string(sf) {
            for (uc_name, _) in parse_use_case_defs(&content) {
                if !uc_name.is_empty() {
                    declared_uc_names.insert(uc_name);
                }
            }
        }
    }
    if declared_uc_names.is_empty() {
        return;
    }

    let normalized_uc_docs: Vec<(String, String)> = active_uc_files
        .iter()
        .map(|uc_path| {
            let stem = uc_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let norm_content = fs::read_to_string(uc_path)
                .map(|c| normalize_uc_token(&c))
                .unwrap_or_default();
            (normalize_uc_token(&stem), norm_content)
        })
        .collect();

    for uc_name in &declared_uc_names {
        let norm = normalize_uc_token(uc_name);
        if norm.is_empty() {
            continue;
        }
        let slug = strip_uc_numbering_prefix(&norm);
        let norm_alnum = norm.replace('_', "");
        let realized = normalized_uc_docs.iter().any(|(stem, body)| {
            stem.contains(&norm)
                || body.contains(&norm)
                || (!norm_alnum.is_empty() && stem.replace('_', "").contains(&norm_alnum))
                || (slug != norm && (strip_uc_numbering_prefix(stem) == slug || body.contains(slug)))
        });
        if !realized {
            errors.push(format!(
                "Check 30A failed: Schema use case def '{}' is not realized in docs/use-cases/",
                uc_name
            ));
        }
    }
}

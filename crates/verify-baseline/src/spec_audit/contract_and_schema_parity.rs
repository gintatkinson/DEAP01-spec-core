//! Contract A/B, 3-Layer Definition of Done, and Multi-File Schema/Spec Realization Parity Gates.
//!
//! Safety & Traceability: Realises: [REQ-SPEC-CONTRACT-AB-DOD-GATE], [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
//! Contract: Fails closed when specs lack Contract A/B or 3-Layer DoD, or when schema files drift or lack spec realizations.

use crate::checks::parity_check_coverage::{
    normalize_uc_token, parse_use_case_defs, strip_uc_numbering_prefix,
};
use crate::checks::parity_check_dual_ssot::{
    check_dual_schema_ssot_parity, collect_qualified_package_bodies,
};
use crate::spec_audit::{
    extract_body_outside_code_blocks, validate_workspace_remote_tracker_projection, MarkdownDoc,
    RemoteTrackerIssue, SpecAuditFinding,
};
use deap_core::sysml_cst::{
    parse_action_defs, parse_part_defs, parse_port_defs, parse_requirement_defs,
    UnifiedSchemaModelExtractor,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Validates that a specification file contains Contract A, Contract B, and 3-Layer Definition of Done.
/// Realises: [REQ-SPEC-CONTRACT-AB-DOD-GATE]
pub fn validate_contract_ab_and_dod(
    spec_type: &str,
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<String> {
    if !matches!(spec_type, "epic" | "feature" | "use_case" | "user_story") {
        return Vec::new();
    }

    let mut errors = Vec::new();
    let prose = extract_body_outside_code_blocks(content);
    let lower_prose = prose.to_ascii_lowercase();
    let lower_headings: Vec<String> = doc
        .headings
        .iter()
        .map(|h| h.text.to_ascii_lowercase())
        .collect();

    let has_contract_a = lower_prose.contains("contract a")
        || lower_headings.iter().any(|h| h.contains("contract a"));
    let has_contract_b = lower_prose.contains("contract b")
        || lower_prose.contains("contract a & b")
        || lower_prose.contains("contract a and b")
        || lower_headings
            .iter()
            .any(|h| h.contains("contract b") || h.contains("contract a & b"));

    if !has_contract_a {
        errors.push(format!(
            "{}: Missing mandatory 'Contract A' design-time specification grounding",
            rel_path
        ));
    }
    if !has_contract_b {
        errors.push(format!(
            "{}: Missing mandatory 'Contract B' runtime verification grounding",
            rel_path
        ));
    }

    let has_3layer_heading = lower_headings.iter().any(|h| {
        h.contains("3-layer definition of done")
            || h.contains("3-layer dod")
            || h.contains("three-layer definition of done")
    });
    let has_all_three_layers = lower_prose.contains("layer 1")
        && lower_prose.contains("layer 2")
        && lower_prose.contains("layer 3");

    if !has_3layer_heading && !has_all_three_layers {
        errors.push(format!(
            "{}: Missing mandatory '3-Layer Definition of Done' (Layer 1, Layer 2, and Layer 3)",
            rel_path
        ));
    }
    errors
}

/// Validates multi-file schema parity (`schema/*.sysml` vs `schema/model.sysml` vs `.pipeline/schema.sysml`) and spec realizations.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
pub fn validate_multi_file_schema_and_spec_realization_parity(repo_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    if let Err(mismatches) = check_dual_schema_ssot_parity(repo_root) {
        errors.extend(mismatches);
    }
    let schema_files = collect_non_empty_schema_sysml_files(repo_root);
    let pipeline_schema = repo_root.join(".pipeline").join("schema.sysml");
    audit_ast_node_spec_realizations(repo_root, &schema_files, &pipeline_schema, &mut errors);
    errors.sort();
    errors.dedup();
    errors
}

/// Collects sorted non-empty `.sysml` files under `schema/`.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
fn collect_non_empty_schema_sysml_files(repo_root: &Path) -> Vec<PathBuf> {
    let schema_dir = repo_root.join("schema");
    if !schema_dir.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> =
        UnifiedSchemaModelExtractor::discover_schema_files(&[&schema_dir])
            .into_iter()
            .filter(|p| {
                p.extension().is_some_and(|ext| ext == "sysml")
                    && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false)
            })
            .collect();
    files.sort();
    files
}

/// Verifies that `schema/model.sysml` is not missing AST definitions declared in sibling `schema/*.sysml` files.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
pub(crate) fn compare_model_sysml_against_combined(
    model_text: &str,
    combined_schema: &str,
    errors: &mut Vec<String>,
) {
    let mut model_bodies = Vec::new();
    collect_qualified_package_bodies(model_text, "", &mut model_bodies);
    let mut comb_bodies = Vec::new();
    collect_qualified_package_bodies(combined_schema, "", &mut comb_bodies);

    let (m_parts, m_actions, m_ucs, m_reqs, m_ports) = extract_scoped_core_sets(&model_bodies);
    let (c_parts, c_actions, c_ucs, c_reqs, c_ports) = extract_scoped_core_sets(&comb_bodies);

    for (label, m_set, c_set) in [
        ("part def", &m_parts, &c_parts),
        ("action def", &m_actions, &c_actions),
        ("use case def", &m_ucs, &c_ucs),
        ("requirement def", &m_reqs, &c_reqs),
        ("port def", &m_ports, &c_ports),
    ] {
        for k in c_set {
            if !m_set.contains(k) {
                errors.push(format!(
                    "Check 31 failed: Split schema drift: {} '{}' is defined in sibling schema/*.sysml or .pipeline/schema.sysml but missing from canonical schema/model.sysml",
                    label, k
                ));
            }
        }
    }
}

/// Core 5-tuple of package-scoped AST name sets (`part`, `action`, `use case`, `requirement`, `port`).
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
type CoreScopedSets = (
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeSet<String>,
);

/// Extracts package-scoped identifier sets for parts, actions, use cases, requirements, and ports.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
fn extract_scoped_core_sets(bodies: &[(String, String)]) -> CoreScopedSets {
    let (mut parts, mut actions, mut ucs, mut reqs, mut ports): CoreScopedSets = Default::default();
    for (pkg, body) in bodies {
        let qual = |name: &str| {
            if pkg.is_empty() {
                name.to_string()
            } else {
                format!("{}::{}", pkg, name)
            }
        };
        for p in parse_part_defs(body) {
            parts.insert(qual(&p.name));
        }
        for a in parse_action_defs(body) {
            actions.insert(qual(&a.name));
        }
        for (u_name, _) in parse_use_case_defs(body) {
            ucs.insert(qual(&u_name));
        }
        for r in parse_requirement_defs(body) {
            reqs.insert(qual(&r.name));
        }
        for p in parse_port_defs(body) {
            ports.insert(qual(&p.name));
        }
    }
    (parts, actions, ucs, reqs, ports)
}

/// Verifies that every `part def`, `use case def`, and `action def` is realized in `docs/{features,use-cases,user-stories}/`.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
pub(crate) fn audit_ast_node_spec_realizations(
    repo_root: &Path,
    schema_files: &[PathBuf],
    pipeline_schema: &Path,
    errors: &mut Vec<String>,
) {
    let mut all_sources = if schema_files.is_empty() {
        collect_non_empty_schema_sysml_files(repo_root)
    } else {
        schema_files.to_vec()
    };
    if pipeline_schema.is_file() {
        all_sources.push(pipeline_schema.to_path_buf());
    }
    let (mut parts, mut ucs, mut actions) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for src in &all_sources {
        if let Ok(content) = fs::read_to_string(src) {
            for p in parse_part_defs(&content) {
                if !p.name.is_empty() {
                    parts.insert(p.name);
                }
            }
            for (u_name, _) in parse_use_case_defs(&content) {
                if !u_name.is_empty() {
                    ucs.insert(u_name);
                }
            }
            for a in parse_action_defs(&content) {
                if !a.name.is_empty() {
                    actions.insert(a.name);
                }
            }
        }
    }
    verify_tier_realization(&repo_root.join("docs/features"), &parts, ("part def", "docs/features/"), errors);
    verify_tier_realization(&repo_root.join("docs/use-cases"), &ucs, ("use case def", "docs/use-cases/"), errors);
    verify_tier_realization(&repo_root.join("docs/user-stories"), &actions, ("action def", "docs/user-stories/"), errors);
}

/// Verifies that all AST symbols in `declared_names` are realized by at least one spec in `tier_dir`.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
fn verify_tier_realization(
    tier_dir: &Path,
    declared_names: &BTreeSet<String>,
    labels: (&str, &str),
    errors: &mut Vec<String>,
) {
    if !tier_dir.is_dir() || declared_names.is_empty() {
        return;
    }
    let mut norm_docs: Vec<(String, String)> = Vec::new();
    for entry in WalkDir::new(tier_dir).sort_by_file_name().into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let fname = entry.file_name().to_string_lossy();
        if !fname.ends_with(".md") || fname == "README.md" || fname.starts_with('_') {
            continue;
        }
        let stem = entry.path().file_stem().map(|s| normalize_uc_token(&s.to_string_lossy())).unwrap_or_default();
        let body = fs::read_to_string(entry.path()).map(|c| normalize_uc_token(&c)).unwrap_or_default();
        norm_docs.push((stem, body));
    }
    if norm_docs.is_empty() {
        return;
    }
    let (kind_label, dir_label) = labels;
    for sym in declared_names {
        let norm = normalize_uc_token(sym);
        if norm.is_empty() {
            continue;
        }
        let slug = strip_uc_numbering_prefix(&norm);
        let norm_alnum = norm.replace('_', "");
        let realized = norm_docs.iter().any(|(stem, body)| {
            stem.contains(&norm)
                || body.contains(&norm)
                || (!norm_alnum.is_empty() && stem.replace('_', "").contains(&norm_alnum))
                || (!norm_alnum.is_empty() && body.replace('_', "").contains(&norm_alnum))
                || (slug != norm && (strip_uc_numbering_prefix(stem) == slug || body.contains(slug)))
        });
        if !realized {
            errors.push(format!("Check 30A failed: Schema {} '{}' is not realized in {}", kind_label, sym, dir_label));
        }
    }
}

/// Runs cross-document schema realization and remote tracker projection gates for `audit_gates`.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE], [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
pub(crate) fn run_extended_cross_doc_gates(repo_root: &Path, findings: &mut Vec<SpecAuditFinding>) {
    for err in validate_multi_file_schema_and_spec_realization_parity(repo_root) {
        findings.push(SpecAuditFinding::new("multi-file-schema-realization-parity", err, "schema", None));
    }
    let snapshot_path = repo_root.join(".pipeline").join("tracker_issues.json");
    let Ok(json_str) = fs::read_to_string(&snapshot_path) else {
        return;
    };
    let Ok(serde_json::Value::Array(items)) = serde_json::from_str::<serde_json::Value>(&json_str) else {
        return;
    };
    let mut remote_map = BTreeMap::new();
    for item in items {
        if let Some(num) = item.get("number").and_then(|v| v.as_u64()) {
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let body = item.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let state = item.get("state").and_then(|v| v.as_str()).unwrap_or("OPEN").to_string();
            let labels = item.get("labels").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|l| l.as_str().or_else(|| l.get("name").and_then(|n| n.as_str())).map(String::from)).collect()
            }).unwrap_or_default();
            remote_map.insert(num, RemoteTrackerIssue { number: num, title, body, state, labels });
        }
    }
    let pref = repo_root.to_string_lossy().contains(".pipeline/upstream").then_some(".pipeline/upstream");
    for err in validate_workspace_remote_tracker_projection(repo_root, &remote_map, (None, "main", pref)) {
        findings.push(SpecAuditFinding::new("remote-tracker-projection-parity", err, "docs", None));
    }
}

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Markdown Specification AST Entity Extraction and Package Merge Helpers for Reverse Sync.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Mitigates dual-schema AST drift (`HAZ-SCHEMA-DRIFT-GATE31`) and
//! non-Agile subdirectory pollution (`Issue #534`, `Issue #535`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Subdirectory Scoping Invariant: Only canonical Agile subdirectories (`epics`, `features`,
//!   `user-stories`, `use-cases`) synthesize structural or behavioral classifiers.
//! - Blueprint Safety Constraint Invariant: `SC-<digits>` rows in `STPA_MATRIX.md` and
//!   `docs/designs/blueprint-*.md` synthesize deduplicated `SC_<digits>_SafetyConstraint` nodes.
//!
//! ## 3. Preconditions
//! - `docs_dir` exists and is readable; `pkg` is the base or default `PackageDef`.
//!
//! ## 4. Postconditions
//! - Merges extracted capabilities, part actions, interactions, use cases, and safety constraints into `pkg`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N_{docs} \cdot L_{doc} + V_{ast})$ across discovered Markdown files.
//! - Space Complexity: $O(V_{ast})$ for intermediate extracted AST node collections.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns `Err(String)` if reading a discovered `.md` specification file fails.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/ENGINE-SYNC]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use deap_core::sysml_cst::{
    CapabilityDef, ConstraintDef, InteractionDef, PackageDef, PartDef, UseCaseDef,
};
use walkdir::WalkDir;

use super::frontmatter::{extract_feature_part_actions, parse_frontmatter, to_pascal_case};

/// Intermediate AST definitions extracted from Markdown specifications prior to merging into `PackageDef`.
/// Realises: [REQ-0604]
#[derive(Default)]
struct ReverseExtractedDefs {
    capabilities: Vec<CapabilityDef>,
    parts: BTreeMap<String, PartDef>,
    interactions: Vec<InteractionDef>,
    use_cases: Vec<UseCaseDef>,
    constraints: Vec<ConstraintDef>,
}

/// Extracts an `InteractionDef` from a `user-stories/*.md` specification.
/// Realises: [REQ-0604]
fn extract_user_story_interaction(
    pkg: &PackageDef,
    fm: &BTreeMap<String, String>,
    path: &Path,
) -> InteractionDef {
    let inter_name = fm
        .get("interaction")
        .or_else(|| fm.get("interaction_def"))
        .map(|s| to_pascal_case(s))
        .unwrap_or_else(|| to_pascal_case(&path.file_stem().unwrap_or_default().to_string_lossy()));
    let default_actor = fm
        .get("actors")
        .or_else(|| fm.get("primary_actor"))
        .map(|s| to_pascal_case(s))
        .or_else(|| {
            pkg.use_case_defs
                .first()
                .and_then(|u| u.actors.first().cloned())
        })
        .unwrap_or_else(|| "SystemActor".to_string());
    let default_subject = fm
        .get("subject")
        .or_else(|| fm.get("subsystem"))
        .or_else(|| fm.get("part"))
        .map(|s| to_pascal_case(s))
        .filter(|s| !s.is_empty())
        .or_else(|| pkg.part_defs.first().map(|p| p.name.clone()))
        .unwrap_or_else(|| to_pascal_case(&pkg.name));

    InteractionDef {
        name: inter_name,
        doc: Some(format!("Interaction from {}", path.display())),
        lifelines: vec![default_actor, default_subject],
        messages: vec!["ExecuteTask".to_string()],
        triggers: vec!["OperationalTrigger".to_string()],
    }
}

/// Extracts a `UseCaseDef` from a `use-cases/*.md` specification.
/// Realises: [REQ-0604]
fn extract_use_case_def(
    pkg: &PackageDef,
    fm: &BTreeMap<String, String>,
    path: &Path,
) -> UseCaseDef {
    let uc_name = fm
        .get("use_case")
        .or_else(|| fm.get("use_case_def"))
        .map(|s| to_pascal_case(s))
        .unwrap_or_else(|| to_pascal_case(&path.file_stem().unwrap_or_default().to_string_lossy()));
    let default_actor = fm
        .get("actors")
        .or_else(|| fm.get("primary_actor"))
        .map(|s| to_pascal_case(s))
        .or_else(|| {
            pkg.use_case_defs
                .first()
                .and_then(|u| u.actors.first().cloned())
        })
        .unwrap_or_else(|| "SystemActor".to_string());
    let subject = fm
        .get("subject")
        .or_else(|| fm.get("subsystem"))
        .or_else(|| fm.get("part"))
        .map(|s| to_pascal_case(s))
        .filter(|s| !s.is_empty())
        .or_else(|| pkg.part_defs.first().map(|p| p.name.clone()))
        .unwrap_or_else(|| to_pascal_case(&pkg.name));
    let objective = fm.get("objective").cloned().unwrap_or_default();

    UseCaseDef {
        name: uc_name,
        doc: Some(objective.clone()),
        actors: vec![default_actor],
        subject: Some(subject),
        objective: Some(objective),
        ..Default::default()
    }
}

/// Extracts deduplicated `SC-*` safety constraints from `STPA_MATRIX.md` or `blueprint-*.md` tables.
/// Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/BLUEPRINT-SC-EXTRACTION]
fn extract_stpa_or_blueprint_constraints(
    doc_type: &str,
    path: &Path,
    content: &str,
    constraints: &mut Vec<ConstraintDef>,
) {
    let is_stpa_or_blueprint = doc_type == "blueprint"
        || path
            .file_name()
            .and_then(|f| f.to_str())
            .map(|name| {
                name == "STPA_MATRIX.md" || (name.starts_with("blueprint-") && name.ends_with(".md"))
            })
            .unwrap_or(false);
    if !is_stpa_or_blueprint {
        return;
    }
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').map(str::trim).collect();
        if let &[_, sc_cell, stmt_cell, _, ..] = cols.as_slice() {
            let raw_sc = sc_cell.trim_matches(|c: char| c == '*' || c == '`' || c.is_whitespace());
            if raw_sc
                .strip_prefix("SC-")
                .is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
            {
                let stmt = stmt_cell.trim();
                if !stmt.is_empty() {
                    let sc_id = raw_sc.replace('-', "_");
                    let constraint_name = format!("{}_SafetyConstraint", sc_id);
                    if !constraints.iter().any(|c| c.name == constraint_name) {
                        constraints.push(ConstraintDef {
                            name: constraint_name,
                            doc: Some(stmt.to_string()),
                            expression: format!("{} == true", sc_id),
                            is_assertion: true,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}

/// Parses a single discovered `.md` specification file and appends its extracted AST definitions to `extracted`.
/// Realises: [REQ-0604]
fn process_markdown_spec_file(
    pkg: &PackageDef,
    docs_dir: &Path,
    path: &Path,
    extracted: &mut ReverseExtractedDefs,
) -> Result<(), String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let (fm, body) = parse_frontmatter(&content);
    let doc_type = fm.get("type").map(String::as_str).unwrap_or("");
    let rel_subdir = path
        .strip_prefix(docs_dir)
        .ok()
        .and_then(|rel| {
            let mut comps = rel.components();
            let first = comps.next()?.as_os_str().to_str()?;
            comps.next().map(|_| first)
        })
        .unwrap_or("");

    match doc_type {
        "epic" if rel_subdir == "epics" => {
            let title = fm.get("title").cloned().unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });
            let subsys = fm.get("subsystem").cloned().unwrap_or_default();
            let name = to_pascal_case(&title);
            extracted.capabilities.push(CapabilityDef {
                name: if name.is_empty() {
                    "OperationalCapability".to_string()
                } else {
                    name
                },
                doc: Some(format!("Capability extracted from {}", path.display())),
                subsystem: Some(subsys),
                description: Some(title),
                ..Default::default()
            });
        }
        "feature" if rel_subdir == "features" => {
            extract_feature_part_actions(&fm, body, path, &mut extracted.parts);
        }
        "user-story" if rel_subdir == "user-stories" => {
            extracted
                .interactions
                .push(extract_user_story_interaction(pkg, &fm, path));
        }
        "use-case" if rel_subdir == "use-cases" => {
            extracted.use_cases.push(extract_use_case_def(pkg, &fm, path));
        }
        _ => {
            extract_stpa_or_blueprint_constraints(
                doc_type,
                path,
                &content,
                &mut extracted.constraints,
            );
        }
    }
    Ok(())
}

/// Scans `docs_dir` for `.md` specifications and merges extracted AST definitions non-destructively into `pkg`.
/// Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/ENGINE-SYNC]
pub(crate) fn extract_markdown_specs_into_package(
    pkg: &mut PackageDef,
    docs_dir: &Path,
) -> Result<(), String> {
    let mut extracted = ReverseExtractedDefs::default();
    for part in pkg.get_all_parts() {
        extracted.parts.insert(part.name.clone(), part);
    }

    for entry in WalkDir::new(docs_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            process_markdown_spec_file(pkg, docs_dir, path, &mut extracted)?;
        }
    }

    for cap in extracted.capabilities {
        if !pkg.capability_defs.iter().any(|c| c.name == cap.name) {
            pkg.capability_defs.push(cap);
        }
    }
    for (_, part) in extracted.parts {
        if let Some(existing) = pkg.find_part_mut(&part.name) {
            for act in part.actions {
                if !existing.actions.iter().any(|a| a.name == act.name) {
                    existing.actions.push(act);
                }
            }
        } else {
            pkg.part_defs.push(part);
        }
    }
    for inter in extracted.interactions {
        if !pkg.interaction_defs.iter().any(|i| i.name == inter.name) {
            pkg.interaction_defs.push(inter);
        }
    }
    for uc in extracted.use_cases {
        if !pkg.use_case_defs.iter().any(|u| u.name == uc.name) {
            pkg.use_case_defs.push(uc);
        }
    }
    for con in extracted.constraints {
        if !pkg.constraint_defs.iter().any(|c| c.name == con.name) {
            pkg.constraint_defs.push(con);
        }
    }
    Ok(())
}

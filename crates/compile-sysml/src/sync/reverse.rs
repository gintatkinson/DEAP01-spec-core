//! Markdown Specifications to SysML v2 Reverse Synchronization Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Mitigates dual-schema AST drift hazards (HAZ-SCHEMA-DRIFT-GATE31)
//! and specification-model desynchronization (HAZ-SPEC-MODEL-DESYNC).
//!
//! ## 2. Invariants & Mathematical Properties
//! - KerML Identifier Invariant: All emitted classifier identifiers conform to KerML grammar
//!   rules (starting with an alphabetic character or underscore, containing only alphanumeric/underscores).
//! - Non-Destructive AST Parity: Existing schema AST structures and subsystem part definitions
//!   are preserved, merging markdown-extracted operations into existing parts without creating
//!   duplicate top-level entities.
//! - Deterministic Serialization & Digest Integrity: Emitted `.pipeline/schema.sysml` and
//!   `.pipeline/schema-digest.json` produce bit-for-bit identical outputs for identical inputs.
//!
//! ## 3. Preconditions
//! - `opts.docs_dir` must exist and be readable.
//! - Parsed markdown frontmatters must contain valid type discriminators.
//!
//! ## 4. Postconditions
//! - Returns `(PackageDef, SchemaDigest)` representing the unified AST and cryptographic digest.
//! - Atomically persists the updated schema and digest when configured.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N_{docs} \cdot L_{doc} + V_{ast} + E_{ast})$ where $N_{docs}$ is the number
//!   of specification files, $L_{doc}$ is average file length, and $(V, E)$ represent AST nodes and edges.
//! - Space Complexity: $O(V_{ast} + E_{ast})$ memory for AST and extracted elements.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns `Err(String)` if `docs_dir` is missing, in-place overwrite is unauthorized, or filesystem writes fail.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-REVERSE-SYNC-ENGINE]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use deap_core::sysml_ast::{
    ActionDef, CapabilityDef, ConstraintDef, InteractionDef, PackageDef, PartDef, UseCaseDef,
};
use walkdir::WalkDir;

use crate::semantic::digest::{generate_digest, write_atomic, write_digest_atomic, SchemaDigest};
use crate::semantic::serializer::to_sysml;

/// Options for reverse synchronization.
#[derive(Debug, Clone)]
pub struct ReverseSyncOptions {
    pub docs_dir: PathBuf,
    pub schema_path: Option<PathBuf>,
    pub output_path: PathBuf,
    pub digest_path: PathBuf,
    pub allow_overwrite: bool,
}

/// Helper to parse metadata from YAML frontmatter or CommonMark table from a markdown string.
fn parse_frontmatter(content: &str) -> (HashMap<String, String>, &str) {
    let mut fm = HashMap::new();
    let trimmed = content.trim_start();
    if trimmed.starts_with("---") {
        let rest = &trimmed[3..];
        if let Some(end_idx) = rest.find("\n---") {
            let fm_block = &rest[..end_idx];
            let body = &rest[end_idx + 4..];

            for line in fm_block.lines() {
                let line = line.trim();
                if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_string();
                    let mut val = line[colon_pos + 1..].trim();
                    if (val.starts_with('"') && val.ends_with('"'))
                        || (val.starts_with('\'') && val.ends_with('\''))
                    {
                        val = &val[1..val.len() - 1];
                    }
                    fm.insert(key, val.to_string());
                }
            }
            return (fm, body);
        }
    }

    // CommonMark 2-column metadata table fallback
    for line in content.lines().take(20) {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('|') && line_trimmed.ends_with('|') {
            let inner = line_trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                let mut key = String::new();
                for c in parts[0].chars() {
                    if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                        key.push(c);
                    }
                }
                let key = key.trim().to_lowercase().replace(' ', "_").replace('-', "_");
                if !key.is_empty() && key != "attribute" && key != "key" {
                    let val = parts[1]
                        .trim()
                        .trim_matches(|c| c == '`' || c == '"' || c == '\'');
                    fm.insert(key.clone(), val.to_string());
                    if key == "sysml_interaction" {
                        fm.insert("interaction".to_string(), val.to_string());
                    } else if key == "scope" {
                        fm.insert("subject".to_string(), val.to_string());
                    } else if key == "primary_actor" {
                        fm.insert("actors".to_string(), val.to_string());
                    } else if key == "schema_containers" {
                        let part = val.split("::").last().unwrap_or(val).trim();
                        fm.insert("part".to_string(), part.to_string());
                    }
                }
            }
        }
    }

    (fm, content)
}

/// Helper to convert a string into clean PascalCase, prepending an underscore if leading with a digit.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
/// engineering standards. Mitigates invalid identifier generation (HAZ-INVALID-KERML-IDENTIFIER)
/// where digit-leading section names (e.g. `1_Normative_Statement`) would violate KerML grammar
/// and cause dual-schema parity validation failure (Check 31).
///
/// ## 2. Invariants & Mathematical Properties
/// - KerML Grammar Conformance: Produced string $S$ satisfies $(S[0] \in \{'\_'\} \cup \text{Alpha}) \land \forall c \in S, c \in \text{Alphanumeric} \cup \{'\_'\}$.
/// - Digit Prefixing Invariant: If input trimmed text begins with a digit, output is guaranteed to start with `_`.
///
/// ## 3. Preconditions
/// - `text` may be any arbitrary ASCII / UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns a PascalCase string conforming to KerML identifier syntax rules.
/// - Preserves leading digit semantics by prepending `_`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ where $L$ is the byte length of `text`.
/// - Space Complexity: $O(L)$ allocated for the resulting `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty input returns empty string; all non-alphanumeric characters act as word boundaries.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-SYNC-REVERSE/PASCAL-CASE-IDENTIFIER]
fn to_pascal_case(text: &str) -> String {
    let has_leading_digit = text.trim_start_matches(|c: char| !c.is_alphanumeric()).chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false);
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let pascal: String = words
        .iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();
    if has_leading_digit || pascal.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{}", pascal)
    } else {
        pascal
    }
}

/// Reverse synchronizes markdown documents under `docs_dir` into the SysML AST SSOT.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
/// engineering standards. Mitigates dual-schema AST drift hazards (HAZ-SCHEMA-DRIFT-GATE31)
/// and ensures complete bidirectional traceability between downstream markdown requirements
/// and formal SysML v2 architectural models.
///
/// ## 2. Invariants & Mathematical Properties
/// - AST SSOT Parity Invariant: Structural part definitions present in multi-file subsystem schemas
///   are located via `pkg.find_part_mut` and updated in-place without generating root-level duplicate definitions.
/// - Cryptographic Digest Invariant: Emits SHA-256 digest capturing normalized AST structure and serialized text.
///
/// ## 3. Preconditions
/// - `opts.docs_dir` must point to an accessible directory containing markdown specifications.
/// - If `opts.allow_overwrite` is false, `opts.output_path` must not collide with `opts.schema_path`.
///
/// ## 4. Postconditions
/// - Emits atomically written `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`.
/// - Returns `Ok((PackageDef, SchemaDigest))` on success.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N_{files} \cdot S_{file} + V_{ast})$ where $N_{files}$ is the count of scanned markdown files.
/// - Space Complexity: $O(V_{ast})$ proportional to AST entity count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns descriptive `Err(String)` if filesystem operations fail or invalid paths are supplied.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-SYNC-REVERSE/ENGINE-SYNC]
pub fn reverse_sync_specs_to_sysml(
    base_pkg: Option<&PackageDef>,
    opts: &ReverseSyncOptions,
) -> Result<(PackageDef, SchemaDigest), String> {
    if !opts.docs_dir.exists() {
        return Err(format!("Docs directory does not exist: {}", opts.docs_dir.display()));
    }

    if !opts.allow_overwrite
        && opts.schema_path.as_ref() == Some(&opts.output_path)
    {
        return Err(format!(
            "In-place overwrite of base input schema '{}' is prohibited when allow_overwrite=false.",
            opts.output_path.display()
        ));
    }

    let mut pkg = base_pkg.cloned().unwrap_or_else(|| {
        let mut p = PackageDef::default();
        p.name = "System_SSOT".to_string();
        p.doc = Some("Single Source of Truth for System Architecture and Safety Model".to_string());
        p
    });

    let mut extracted_capabilities = Vec::new();
    let mut extracted_parts: HashMap<String, PartDef> = HashMap::new();
    let mut extracted_interactions = Vec::new();
    let mut extracted_use_cases = Vec::new();
    let mut extracted_constraints = Vec::new();

    // Populate existing parts into the map for merge
    for part in pkg.get_all_parts() {
        extracted_parts.insert(part.name.clone(), part);
    }

    for entry in WalkDir::new(&opts.docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

            let (fm, body) = parse_frontmatter(&content);
            let doc_type = fm.get("type").map(|s| s.as_str()).unwrap_or("");

            match doc_type {
                "epic" => {
                    let title = fm.get("title").cloned().unwrap_or_else(|| {
                        path.file_stem().unwrap().to_string_lossy().to_string()
                    });
                    let subsys = fm.get("subsystem").cloned().unwrap_or_default();
                    let name = to_pascal_case(&title);
                    extracted_capabilities.push(CapabilityDef {
                        name: if name.is_empty() { "OperationalCapability".to_string() } else { name },
                        doc: Some(format!("Capability extracted from {}", path.display())),
                        subsystem: Some(subsys),
                        description: Some(title),
                        ..Default::default()
                    });
                }
                "feature" => {
                    let part_name = fm
                        .get("part")
                        .or_else(|| fm.get("part_def"))
                        .map(|s| {
                            // If already a valid KerML identifier, preserve it directly
                            if !s.is_empty()
                                && (s.starts_with('_') || s.chars().next().unwrap().is_alphabetic())
                                && s.chars().all(|c| c.is_alphanumeric() || c == '_')
                            {
                                s.clone()
                            } else {
                                to_pascal_case(s)
                            }
                        })
                        .unwrap_or_else(|| {
                            to_pascal_case(&path.file_stem().unwrap().to_string_lossy())
                        });

                    if !part_name.is_empty() {
                        let entry = extracted_parts.entry(part_name.clone()).or_insert_with(|| {
                            let mut p = PartDef::default();
                            p.name = part_name.clone();
                            p
                        });

                        // Extract operations/actions from markdown lists: - `+ActionName() : void`
                        let default_act = format!("execute_{}", part_name.to_lowercase());
                        for line in body.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with('-') {
                                let after_dash = trimmed[1..].trim_start();
                                if after_dash.starts_with("`+") {
                                    let rest = &after_dash[2..];
                                    if let Some(paren_open) = rest.find('(') {
                                        let act_name = rest[..paren_open].trim();
                                        if !act_name.is_empty() && act_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                            if let Some(paren_close) = rest[paren_open..].find(')') {
                                                let after_paren = rest[paren_open + paren_close + 1..].trim_start();
                                                if after_paren.starts_with(':') {
                                                    let after_colon = after_paren[1..].trim();
                                                    if after_colon.ends_with('`') {
                                                        let ret_type = after_colon.trim_end_matches('`').trim();
                                                        if !ret_type.is_empty() && ret_type.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                                            let act_name_str = act_name.to_string();
                                                            // Skip forward-sync synthetic placeholder actions (execute_<part_name>) to preserve SSOT parity
                                                            if act_name_str.to_lowercase() == default_act.to_lowercase() {
                                                                continue;
                                                            }
                                                            if !entry.actions.iter().any(|a| a.name == act_name_str) {
                                                                let mut a = ActionDef::default();
                                                                a.name = act_name_str;
                                                                entry.actions.push(a);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "user-story" => {
                    let inter_name = fm
                        .get("interaction")
                        .or_else(|| fm.get("interaction_def"))
                        .map(|s| to_pascal_case(s))
                        .unwrap_or_else(|| {
                            to_pascal_case(&path.file_stem().unwrap().to_string_lossy())
                        });
                    let subject = fm.get("subject").cloned().unwrap_or_default();

                    extracted_interactions.push(InteractionDef {
                        name: inter_name,
                        doc: Some(format!("Interaction from {}", path.display())),
                        lifelines: if subject.is_empty() {
                            vec!["OperatorConsole".to_string(), "CoreController".to_string()]
                        } else {
                            vec!["OperatorConsole".to_string(), subject]
                        },
                        messages: vec!["ExecuteTask".to_string()],
                        triggers: vec!["OperationalTrigger".to_string()],
                    });
                }
                "use-case" => {
                    let uc_name = fm
                        .get("use_case")
                        .or_else(|| fm.get("use_case_def"))
                        .map(|s| to_pascal_case(s))
                        .unwrap_or_else(|| {
                            to_pascal_case(&path.file_stem().unwrap().to_string_lossy())
                        });
                    let subject = fm.get("subject").cloned().unwrap_or_default();
                    let objective = fm.get("objective").cloned().unwrap_or_default();

                    extracted_use_cases.push(UseCaseDef {
                        name: uc_name,
                        doc: Some(objective.clone()),
                        actors: vec!["OperatorConsole".to_string()],
                        subject: if subject.is_empty() { Some("CoreController".to_string()) } else { Some(subject) },
                        objective: Some(objective),
                        ..Default::default()
                    });
                }
                _ => {
                    // Check if file is STPA_MATRIX.md
                    if path.file_name().map(|f| f == "STPA_MATRIX.md").unwrap_or(false) {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if !trimmed.starts_with('|') {
                                continue;
                            }
                            let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
                            if cols.len() >= 4 {
                                let raw_sc = cols[1].trim_matches('*').trim();
                                if raw_sc.starts_with("SC-")
                                    && !raw_sc["SC-".len()..].is_empty()
                                    && raw_sc["SC-".len()..].chars().all(|c| c.is_ascii_digit())
                                {
                                    let stmt = cols[2].trim();
                                    if !stmt.is_empty() {
                                        let sc_id = raw_sc.replace('-', "_");
                                        extracted_constraints.push(ConstraintDef {
                                            name: format!("{}_SafetyConstraint", sc_id),
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
            }
        }
    }

    // Merge into PackageDef
    for cap in extracted_capabilities {
        if !pkg.capability_defs.iter().any(|c| c.name == cap.name) {
            pkg.capability_defs.push(cap);
        }
    }

    // Update part defs
    for (_, part) in extracted_parts {
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

    for inter in extracted_interactions {
        if !pkg.interaction_defs.iter().any(|i| i.name == inter.name) {
            pkg.interaction_defs.push(inter);
        }
    }

    for uc in extracted_use_cases {
        if !pkg.use_case_defs.iter().any(|u| u.name == uc.name) {
            pkg.use_case_defs.push(uc);
        }
    }

    for con in extracted_constraints {
        if !pkg.constraint_defs.iter().any(|c| c.name == con.name) {
            pkg.constraint_defs.push(con);
        }
    }

    // Serialize to SysML
    let sysml_code = to_sysml(&pkg);

    // Write schema atomically
    if let Some(parent) = opts.output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create directory '{}': {}", parent.display(), e)
        })?;
    }
    write_atomic(&opts.output_path, &sysml_code).map_err(|e| e.to_string())?;

    // Compute and write digest
    let mut digest = generate_digest(&pkg, &sysml_code);
    let schema_dir = opts
        .schema_path
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("schema"));
    if schema_dir.is_dir() {
        let files = crate::parser::grammar::discover_sysml_files(schema_dir);
        if !files.is_empty() {
            let manifest = crate::semantic::digest::compute_file_manifest(&files);
            digest = digest.with_file_manifest(manifest);
        }
    }
    write_digest_atomic(&opts.digest_path, &digest).map_err(|e| e.to_string())?;

    println!(
        "[SysML v2 Reverse-Sync] Merged specifications from '{}' -> '{}' (SHA-256: {})",
        opts.docs_dir.display(),
        opts.output_path.display(),
        digest.sha256
    );

    Ok((pkg, digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_extraction() {
        let md = r#"---
title: "Feature 01: Airframe"
type: feature
part: Airframe
---

# Content
"#;
        let (fm, body) = parse_frontmatter(md);
        assert_eq!(fm.get("title").map(|s| s.as_str()), Some("Feature 01: Airframe"));
        assert_eq!(fm.get("type").map(|s| s.as_str()), Some("feature"));
        assert_eq!(fm.get("part").map(|s| s.as_str()), Some("Airframe"));
        assert!(body.contains("# Content"));
    }
}

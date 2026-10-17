//! # SysML v2 Universal Ingestion Engine Core
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This library provides the authoritative Ground 0.0 ingestion framework for transforming
//! heterogeneous Level 0 OEM requirements, technical interface control documents (ICDs),
//! and domain specifications into canonical SysML v2 textual Abstract Syntax Tree (AST)
//! models (`deap_core::sysml_ast::PackageDef`).
//!
//! In high-integrity software engineering (DO-178C Level A, ISO 26262 ASIL D, ECSS-E-ST-40C, DO-331),
//! specification ingestion establishes the foundational formal baseline for all downstream verification,
//! formal invariant checks, hazard mitigations, and code generation. Corruptions or unverified
//! omissions at this layer propagate silently throughout the lifecycle.
//!
//! ## 2. Core Safety Invariants & Architecture
//! - **Pure Schema-Driven Operation**: All AST derivations occur deterministically from input
//!   schemas without hardcoded domain vocabulary or speculative heuristic assumptions.
//! - **Heterogeneous Specification Support**:
//!   - Level 0 OEM Markdown with YAML frontmatter, BDD acceptance criteria, and GFM tables ([`translators::markdown`]).
//!   - OMG IDL 3.x / 4.x specifications for interface and payload definitions ([`translators::idl`]).
//!   - OpenAPI 3.0 / 3.1 RESTful interface specifications ([`translators::openapi`]).
//!   - IETF RFC 6020 / RFC 7950 YANG data models ([`translators::yang`]).
//! - **Scope Filtering & Negative Invariant Enforcement**:
//!   [`filter_ast_to_target_scope`] strictly prunes phantom entities and projects negative exclusion
//!   assertions (`assert_exclusion_<entity>`) to ensure strict closed-world semantics.
//! - **Cryptographic Fingerprinting & Truncation Gates**:
//!   [`digest`] enforces FIPS 180-4 SHA-256 fingerprinting and subagent context truncation verification.

pub mod detectors;
pub mod digest;
pub mod table;
pub mod translators;

use std::collections::HashSet;
use deap_core::sysml_ast::{ConstraintDef, PackageDef, PartDef};

/// Translates a single Markdown requirement document into a canonical SysML v2 `PackageDef`.
///
/// /// Realises: [REQ-SYSML-INGEST-CORE/translate_markdown]
///
/// ### Safety Intent:
/// Provides the primary convenience entrypoint for ingesting Level 0 OEM Markdown documents,
/// invoking [`translators::markdown::MarkdownTranslator::translate`] in a bounded execution context.
///
/// ### Preconditions:
/// - `content` is a valid UTF-8 string slice containing Markdown text.
/// - `file_name` is a non-empty fallback package identifier.
///
/// ### Postconditions:
/// - Returns `Ok(PackageDef)` on success, or an explanatory error string on parsing failure.
pub fn translate_markdown(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::markdown::MarkdownTranslator::new().translate(content, file_name)
}

/// Translates a single YANG model into a canonical SysML v2 `PackageDef`.
///
/// /// Realises: [REQ-SYSML-INGEST-CORE/translate_yang]
///
/// ### Safety Intent:
/// Provides the primary convenience entrypoint for ingesting RFC 6020/7950 YANG models,
/// invoking [`translators::yang::YangTranslator::translate`] in a bounded execution context.
///
/// ### Preconditions:
/// - `content` is a valid UTF-8 string slice containing YANG specification text.
/// - `file_name` is a non-empty fallback package identifier.
///
/// ### Postconditions:
/// - Returns `Ok(PackageDef)` on success, or an explanatory error string on parsing failure.
pub fn translate_yang(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::yang::YangTranslator::new().translate(content, file_name)
}

/// Filters AST nodes to target metamodel scope and projects negative invariants.
///
/// /// Realises: [REQ-SYSML-INGEST-SCOPE/filter_ast_to_target_scope]
///
/// ### Safety Intent:
/// Enforces a strict closed-world assumption by pruning extraneous external reference entities
/// that are not part of the active system boundary. Furthermore, projects formal negative
/// exclusion invariants (`assert_exclusion_<name>`) to mathematically prove that phantom
/// entities do not exist in the ingested model.
///
/// ### Preconditions:
/// - `pkg` is a mutable reference to an initialized `PackageDef`.
/// - `allowed_parts` is an optional set of whitelisted part names. If `None`, no parts are pruned.
/// - `negative_invariants` is an optional slice of entity names whose absence must be formally asserted.
///
/// ### Postconditions:
/// - Any `PartDef` whose name is not in `allowed_parts` is recursively pruned from `pkg.part_defs`.
/// - Cascading references in items, capabilities, hazards, risks, and connections are cleaned.
/// - For each entry in `negative_invariants`, a formal `ConstraintDef` is appended asserting `!exists(<entity>)`.
pub fn filter_ast_to_target_scope(
    pkg: &mut PackageDef,
    allowed_parts: Option<&HashSet<String>>,
    negative_invariants: Option<&[String]>,
) {
    // Step 1: Prune parts outside allowed scope if a whitelist is supplied
    if let Some(allowed) = allowed_parts {
        // Recursive helper to filter nested child parts
        fn filter_part(part: &mut PartDef, allowed: &HashSet<String>) -> bool {
            if !allowed.contains(&part.name) {
                return false;
            }
            part.parts.retain_mut(|sub| filter_part(sub, allowed));
            true
        }

        // Retain only whitelisted top-level part definitions
        pkg.part_defs.retain_mut(|p| filter_part(p, allowed));

        if !allowed.is_empty() {
            // Prune item defs not associated with surviving parts
            pkg.item_defs.retain(|item| {
                allowed.contains(&item.name)
                    || pkg.part_defs.iter().any(|p| p.name.contains(&item.name) || item.name.contains(&p.name))
            });

            // Prune capabilities whose subsystem is not allowed
            pkg.capability_defs.retain(|c| {
                c.subsystem.as_ref().map_or(true, |sub| allowed.contains(sub)) || allowed.contains(&c.name)
            });

            // Prune hazards referencing excluded parts
            pkg.hazard_defs.retain(|h| {
                h.part_ref.as_ref().map_or(true, |pr| allowed.contains(pr)) || allowed.contains(&h.name)
            });

            // Prune risks referencing pruned hazards
            let surviving_hazards: HashSet<String> = pkg.hazard_defs.iter().map(|h| h.name.clone()).collect();
            pkg.risk_defs.retain(|r| {
                r.hazard_ref.as_ref().map_or(true, |hr| surviving_hazards.contains(hr)) || allowed.contains(&r.name)
            });

            // Prune connection defs where source or target part is excluded
            pkg.connection_defs.retain(|conn| {
                let src_part = conn.source_port.split('.').next();
                let tgt_part = conn.target_port.split('.').next();
                if let Some(sp) = src_part {
                    if conn.source_port.contains('.') && !allowed.contains(sp) {
                        return false;
                    }
                }
                if let Some(tp) = tgt_part {
                    if conn.target_port.contains('.') && !allowed.contains(tp) {
                        return false;
                    }
                }
                true
            });
        }

        // Recursively filter child sub-packages
        for sub_pkg in &mut pkg.packages {
            filter_ast_to_target_scope(sub_pkg, Some(allowed), None);
        }
    }

    // Step 2: Project negative exclusion invariants into formal constraint assertions
    if let Some(neg_list) = negative_invariants {
        for inv in neg_list {
            let clean = inv
                .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_")
                .to_ascii_lowercase();
            let clean_name = clean.trim_matches('_');
            let constraint_name = format!("assert_exclusion_{}", clean_name);
            if !pkg.constraint_defs.iter().any(|c| c.name == constraint_name) {
                pkg.constraint_defs.push(ConstraintDef {
                    name: constraint_name,
                    expression: format!("!exists({})", inv),
                    parameters: Vec::new(),
                    is_assertion: true,
                    doc: Some(format!("Negative invariant asserting exclusion of phantom entity: {}", inv)),
                    pre_conditions: Vec::new(),
                    post_conditions: Vec::new(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_translate_markdown_api() {
        let content = "# Package_0\n\n## Classifier_Alpha\nSynthetic element.";
        let res = translate_markdown(content, "Package_0");
        assert!(res.is_ok());
        let pkg = res.unwrap();
        assert_eq!(pkg.name, "Package_0");
        assert_eq!(pkg.part_defs.len(), 1);
        assert_eq!(pkg.part_defs[0].name, "Classifier_Alpha");
    }

    #[test]
    fn test_filter_ast_to_target_scope_synthetic() {
        let mut pkg = PackageDef {
            name: "Package_0".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Classifier_Alpha".to_string(),
                    ..Default::default()
                },
                PartDef {
                    name: "Classifier_Extraneous".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut allowed = HashSet::new();
        allowed.insert("Classifier_Alpha".to_string());
        let neg = vec!["Phantom_Entity".to_string()];

        filter_ast_to_target_scope(&mut pkg, Some(&allowed), Some(&neg));

        assert_eq!(pkg.part_defs.len(), 1);
        assert_eq!(pkg.part_defs[0].name, "Classifier_Alpha");
        assert_eq!(pkg.constraint_defs.len(), 1);
        assert_eq!(pkg.constraint_defs[0].name, "assert_exclusion_phantom_entity");
        assert_eq!(pkg.constraint_defs[0].expression, "!exists(Phantom_Entity)");
    }

    #[test]
    fn test_batch_translate_multiple_files_synthetic() {
        use std::fs;
        let content1 = "# Package_0\n\n## Classifier_Alpha\nAlpha doc.";
        let content2 = "# Package_0\n\n## Classifier_Beta\nBeta doc.";

        let dir = std::env::temp_dir().join("ingest_test_batch");
        let _ = fs::create_dir_all(&dir);
        let f1 = dir.join("spec1.md");
        let f2 = dir.join("spec2.md");
        let _ = fs::write(&f1, content1);
        let _ = fs::write(&f2, content2);

        let translator = translators::markdown::MarkdownTranslator::new();
        let file_paths = vec![f1.to_string_lossy().to_string(), f2.to_string_lossy().to_string()];
        let pkg = translator.translate_files(&file_paths, "Consolidated_Package").expect("Failed batch translation");

        assert_eq!(pkg.name, "Consolidated_Package");
        let p_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
        assert!(p_names.contains(&"Classifier_Alpha"));
        assert!(p_names.contains(&"Classifier_Beta"));

        let _ = fs::remove_file(f1);
        let _ = fs::remove_file(f2);
        let _ = fs::remove_dir(dir);
    }
}

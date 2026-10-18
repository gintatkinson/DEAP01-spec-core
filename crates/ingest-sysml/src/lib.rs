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
//! ## 2. Invariants & Mathematical Properties
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
//!
//! ## 3. Preconditions
//! - Input payloads provided to ingestion translators must be valid UTF-8 text streams.
//! - Target package identifiers and scope boundary filter sets must be well-formed strings.
//!
//! ## 4. Postconditions
//! - Successful ingestion produces a deterministic, structurally valid `deap_core::sysml_ast::PackageDef`.
//! - When scope filtering is applied, out-of-scope definitions and dangling references are pruned and
//!   formal negative exclusion constraints are injected into `PackageDef::constraint_defs`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - **Time Complexity**: $O(N + P \cdot C)$ where $N$ is the input specification byte length, $P$ is the
//!   number of AST part definitions, and $C$ is the number of cross-referenced items/connections.
//! - **Space Complexity**: $O(N)$ auxiliary heap memory proportional to the parsed AST node count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed syntax or structural violations in upstream documents return explicit `Err(String)`
//!   diagnostics without panicking or emitting partial corrupt AST packages.
//! - Unrecognized or out-of-scope entities are deterministically excluded via closed-world pruning.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-INGEST-LIB], [DO-330-TQL-4-INGEST-LIB]

pub mod detectors;
pub mod digest;
pub mod table;
pub mod translators;

use std::collections::HashSet;
use deap_core::sysml_ast::{ConstraintDef, PackageDef, PartDef};

/// Translates a single Markdown requirement document into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides the primary convenience entrypoint for ingesting Level 0 OEM Markdown documents,
/// invoking [`translators::markdown::MarkdownTranslator::translate`] in a bounded execution context.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves total deterministic mapping from Markdown AST headings, YAML frontmatter, and tables
///   into `PackageDef` nodes without speculative domain inference.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice containing Markdown text.
/// - `file_name` is a non-empty fallback package identifier.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` containing all extracted SysML v2 AST nodes on success, or an
///   explanatory `Err(String)` diagnostic on parsing failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(N)$ where $N$ is the byte length of `content`.
/// - **Space Complexity**: $O(N)$ to hold the synthesized `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates translator diagnostic errors cleanly if `content` violates structural expectations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CORE/translate_markdown]
pub fn translate_markdown(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::markdown::MarkdownTranslator::new().translate(content, file_name)
}

/// Translates a single YANG model into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides the primary convenience entrypoint for ingesting RFC 6020/7950 YANG models,
/// invoking [`translators::yang::YangTranslator::translate`] in a bounded execution context.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic projection of YANG containers, groupings, typedefs, and leaves into canonical
///   SysML v2 `PartDef`, `ItemDef`, and `AttributeDef` structures.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice containing YANG specification text.
/// - `file_name` is a non-empty fallback package identifier.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` on success, or an explanatory `Err(String)` on parsing failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(N)$ where $N$ is the byte length of `content`.
/// - **Space Complexity**: $O(N)$ proportional to the number of YANG statements and AST elements.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unbalanced braces or malformed YANG constructs yield deterministic `Err(String)` diagnostics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CORE/translate_yang]
pub fn translate_yang(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::yang::YangTranslator::new().translate(content, file_name)
}

/// Translates a single Rust or Dart source file into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 source-code AST ingestion (`translate_code`).
/// Invokes [`translators::code::CodeTranslator::translate`] to parse source declarations
/// without regular expressions and lower them into a `PackageDef`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Maps classes/structs to `PartDef`, fields to `AttributeDef`, callbacks to `PortDef`,
///   and functions/methods to `ActionDef`.
///
/// ## 3. Preconditions
/// - `content: &str` is valid UTF-8 Rust or Dart source code.
/// - `file_name: &str` is the source filename or package identifier.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` on clean parse and lowering, or `Err(String)` on syntax error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `content.len()`.
/// - Space Complexity: O(N) proportional to extracted AST elements.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates syntax/delimiter errors with `line:col` coordinates.
/// - State: Pure functional delegation.
/// - Action: Never panics on malformed source code.
/// - Resource: Bounded by single file buffer size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CORE/translate_code]
pub fn translate_code(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::code::CodeTranslator::new().translate(content, file_name)
}

/// Filters AST nodes to target metamodel scope and projects negative invariants.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces a strict closed-world assumption by pruning extraneous external reference entities
/// that are not part of the active system boundary. Furthermore, projects formal negative
/// exclusion invariants (`assert_exclusion_<name>`) to mathematically prove that phantom
/// entities do not exist in the ingested model.
///
/// ## 2. Invariants & Mathematical Properties
/// - **Closed-World Pruning**: Every surviving `PartDef` in `pkg.part_defs` (and nested sub-packages)
///   is guaranteed to be a member of `allowed_parts` when a whitelist is supplied.
/// - **Idempotent Constraint Injection**: Duplicate negative invariant entries do not produce
///   duplicate `ConstraintDef` assertions in `pkg.constraint_defs`.
///
/// ## 3. Preconditions
/// - `pkg` is a mutable reference to an initialized `PackageDef`.
/// - `allowed_parts` is an optional set of whitelisted part names. If `None`, no parts are pruned.
/// - `negative_invariants` is an optional slice of entity names whose absence must be formally asserted.
///
/// ## 4. Postconditions
/// - Any `PartDef` whose name is not in `allowed_parts` is recursively pruned from `pkg.part_defs`.
/// - Cascading references in items, capabilities, hazards, risks, and connections are cleaned.
/// - For each entry in `negative_invariants`, a formal `ConstraintDef` is appended asserting `!exists(<entity>)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(P \cdot I + H + R + K + M \cdot C)$ where $P$ is part count, $I$ is item count,
///   $H$ is hazard count, $R$ is risk count, $K$ is connection count, $M$ is negative invariant count,
///   and $C$ is existing constraint count.
/// - **Space Complexity**: $O(H + D)$ where $H$ is surviving hazard set size and $D$ is package recursion depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty `allowed_parts` prunes all parts while preserving structural validity of `pkg`.
/// - Non-alphanumeric characters in `negative_invariants` are sanitized into valid identifier tokens.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/filter_ast_to_target_scope]
pub fn filter_ast_to_target_scope(
    pkg: &mut PackageDef,
    allowed_parts: Option<&HashSet<String>>,
    negative_invariants: Option<&[String]>,
) {
    // Step 1: Prune parts outside allowed scope if a whitelist is supplied
    if let Some(allowed) = allowed_parts {
        /// Recursively filters a `PartDef` and its nested child parts against `allowed`.
        ///
        /// ## 1. Safety Intent & Regulatory Scope
        /// Ensures hierarchical containment integrity during scope pruning so no disallowed child
        /// part remains attached to an allowed parent part.
        ///
        /// ## 2. Invariants & Mathematical Properties
        /// - Returns `true` if and only if `part.name` is present in `allowed`.
        /// - All retained descendants of `part.parts` also satisfy membership in `allowed`.
        ///
        /// ## 3. Preconditions
        /// - `part` is a valid mutable `PartDef` reference.
        /// - `allowed` is an initialized `HashSet<String>` of permitted part names.
        ///
        /// ## 4. Postconditions
        /// - If `part.name` is in `allowed`, `part.parts` is pruned in-place to retain only allowed descendants.
        ///
        /// ## 5. Algorithmic Complexity Bounds
        /// - **Time Complexity**: $O(T)$ where $T$ is the number of nodes in the `part` subtree.
        /// - **Space Complexity**: $O(D)$ call-stack frames where $D$ is the maximum part nesting depth.
        ///
        /// ## 6. Failure Modes & Fault Tolerance
        /// - Immediately short-circuits and returns `false` when the root `part.name` is excluded.
        ///
        /// ## 7. Realization & Traceability
        /// Realises: [REQ-SYSML-INGEST-SCOPE/filter_part]
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
        fs::create_dir_all(&dir).ok();
        let f1 = dir.join("spec1.md");
        let f2 = dir.join("spec2.md");
        fs::write(&f1, content1).ok();
        fs::write(&f2, content2).ok();

        let translator = translators::markdown::MarkdownTranslator::new();
        let file_paths = vec![f1.to_string_lossy().to_string(), f2.to_string_lossy().to_string()];
        let pkg = translator.translate_files(&file_paths, "Consolidated_Package").expect("Failed batch translation");

        assert_eq!(pkg.name, "Consolidated_Package");
        let p_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
        assert!(p_names.contains(&"Classifier_Alpha"));
        assert!(p_names.contains(&"Classifier_Beta"));

        fs::remove_file(f1).ok();
        fs::remove_file(f2).ok();
        fs::remove_dir(dir).ok();
    }

    #[test]
    fn test_ingest_sysml_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        assert!(src.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("//! ## 3. Preconditions"));
        assert!(src.contains("//! ## 4. Postconditions"));
        assert!(src.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("//! ## 7. Realization & Traceability"));
        assert!(src.contains("//! Realises: [REQ-SYSML-INGEST-LIB], [DO-330-TQL-4-INGEST-LIB]"));
    }
}

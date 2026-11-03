//! # SysML v2 Universal Ingestion Engine Core
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Authoritative Ground 0.0 ingestion framework transforming heterogeneous Level 0 OEM
//! requirements, ICDs, and domain specifications into canonical SysML v2 AST models
//! (`deap_core::PackageDef`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure schema-driven operation across Markdown, IDL, OpenAPI, YANG, and source code ASTs.
//! - Scope filtering and negative invariant projection via [`filter_ast_to_target_scope`].
//! - FIPS 180-4 SHA-256 fingerprinting and truncation detection via [`digest`].
//!
//! ## 3. Preconditions
//! - Input payloads must be valid UTF-8 text streams with well-formed package identifiers.
//!
//! ## 4. Postconditions
//! - Produces deterministic `deap_core::PackageDef` models with closed-world scope pruning.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N + P \cdot C)$ over input byte length $N$, part definitions $P$, and cross-references $C$.
//! - Space Complexity: $O(N)$ auxiliary heap memory proportional to parsed AST node count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed syntax or structural violations return explicit `Err(String)` diagnostics without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-INGEST-LIB], [DO-330-TQL-4-INGEST-LIB]

pub mod cst;
pub mod detectors;
pub mod digest;
pub mod table;

use deap_core::{ConstraintDef, IncrementalAstCache, PackageDef, PartDef, SysmlModel};
use std::collections::BTreeSet;

/// Fail-safe process exit code classification for the `ingest-sysml` CLI engine. Realises: [REQ-SYSML-INGEST-EXIT-CODE/IngestExitCode]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestExitCode {
    /// Clean ingestion, scope verification, and cryptographic digest verification (`0`).
    Clean = 0,
    /// Lexical, syntactic, format classification, or filesystem I/O error (`1`).
    ParseOrIoError = 1,
    /// Architectural scope starvation, negative-invariant violation, or SHA-256 verification failure (`2`).
    ScopeOrCryptoError = 2,
}

impl IngestExitCode {
    /// Converts this [`IngestExitCode`] into its canonical `i32` OS process exit status. Realises: [REQ-SYSML-INGEST-EXIT-CODE/code]
    pub const fn code(self) -> i32 {
        self as i32
    }
}

/// Translates a single Markdown requirement document into a canonical SysML v2 `PackageDef`. Realises: [REQ-SYSML-INGEST-CORE/translate_markdown]
pub fn translate_markdown(content: &str, file_name: &str) -> Result<PackageDef, String> {
    cst::markdown_cst::MarkdownTranslator::new().translate(content, file_name)
}

/// Translates a single YANG model into a canonical SysML v2 `PackageDef`. Realises: [REQ-SYSML-INGEST-CORE/translate_yang]
pub fn translate_yang(content: &str, file_name: &str) -> Result<PackageDef, String> {
    cst::yang_cst::YangTranslator::new().translate(content, file_name)
}

/// Translates a single Rust or Dart source file into a canonical SysML v2 `PackageDef`. Realises: [REQ-SYSML-INGEST-CORE/translate_code]
pub fn translate_code(content: &str, file_name: &str) -> Result<PackageDef, String> {
    cst::code_cst::CodeTranslator::new().translate(content, file_name)
}

/// Filters AST nodes to target metamodel scope and projects negative invariants. Realises: [REQ-SYSML-INGEST-SCOPE/filter_ast_to_target_scope]
pub fn filter_ast_to_target_scope(
    pkg: &mut PackageDef,
    allowed_parts: Option<&BTreeSet<String>>,
    negative_invariants: Option<&[String]>,
) {
    if let Some(allowed) = allowed_parts {
        /// Recursively filters a `PartDef` and its nested child parts against `allowed`. Realises: [REQ-SYSML-INGEST-SCOPE/filter_part]
        fn filter_part(part: &mut PartDef, allowed: &BTreeSet<String>) -> bool {
            if !allowed.contains(&part.name) {
                return false;
            }
            part.parts.retain_mut(|sub| filter_part(sub, allowed));
            true
        }

        pkg.part_defs.retain_mut(|p| filter_part(p, allowed));

        if !allowed.is_empty() {
            pkg.item_defs.retain(|item| {
                allowed.contains(&item.name)
                    || pkg.part_defs.iter().any(|p| p.name.contains(&item.name) || item.name.contains(&p.name))
            });
            pkg.capability_defs.retain(|c| {
                c.subsystem.as_ref().is_none_or(|sub| allowed.contains(sub)) || allowed.contains(&c.name)
            });
            pkg.hazard_defs.retain(|h| {
                h.part_ref.as_ref().is_none_or(|pr| allowed.contains(pr)) || allowed.contains(&h.name)
            });
            let surviving_hazards: BTreeSet<String> = pkg.hazard_defs.iter().map(|h| h.name.clone()).collect();
            pkg.risk_defs.retain(|r| {
                r.hazard_ref.as_ref().is_none_or(|hr| surviving_hazards.contains(hr)) || allowed.contains(&r.name)
            });
            pkg.connection_defs.retain(|conn| {
                let sp_bad = conn.source_port.split('.').next().is_some_and(|sp| conn.source_port.contains('.') && !allowed.contains(sp));
                let tp_bad = conn.target_port.split('.').next().is_some_and(|tp| conn.target_port.contains('.') && !allowed.contains(tp));
                !sp_bad && !tp_bad
            });
        }
        for sub_pkg in &mut pkg.packages {
            filter_ast_to_target_scope(sub_pkg, Some(allowed), None);
        }
    }
    if let Some(neg_list) = negative_invariants {
        for inv in neg_list {
            let clean = inv.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_").to_ascii_lowercase();
            let constraint_name = format!("assert_exclusion_{}", clean.trim_matches('_'));
            if !pkg.constraint_defs.iter().any(|c| c.name == constraint_name) {
                pkg.constraint_defs.push(ConstraintDef {
                    name: constraint_name,
                    expression: format!("!exists({inv})"),
                    is_assertion: true,
                    doc: Some(format!("Negative invariant asserting exclusion of phantom entity: {inv}")),
                    ..Default::default()
                });
            }
        }
    }
}

/// Checks whether a `PackageDef` (or any nested sub-package) contains at least one `PartDef`. Realises: [REQ-SYSML-INGEST-SCOPE/package_has_any_part]
fn package_has_any_part(pkg: &PackageDef) -> bool {
    !pkg.part_defs.is_empty() || pkg.packages.iter().any(package_has_any_part)
}

/// Checks whether any declared entity in `pkg` (or nested packages/parts) matches `entity_name`. Realises: [REQ-SYSML-INGEST-SCOPE/package_contains_entity]
fn package_contains_entity(pkg: &PackageDef, entity_name: &str) -> bool {
    let mut pkg_stack = vec![pkg];
    while let Some(curr_pkg) = pkg_stack.pop() {
        if curr_pkg.item_defs.iter().any(|i| i.name == entity_name)
            || curr_pkg.action_defs.iter().any(|a| a.name == entity_name)
            || curr_pkg.port_defs.iter().any(|port| port.name == entity_name)
            || curr_pkg.state_defs.iter().any(|s| s.name == entity_name)
        {
            return true;
        }
        let mut part_stack: Vec<&PartDef> = curr_pkg.part_defs.iter().collect();
        while let Some(part) = part_stack.pop() {
            if part.name == entity_name
                || part.actions.iter().any(|a| a.name == entity_name)
                || part.ports.iter().any(|port| port.name == entity_name)
                || part.states.iter().any(|s| s.name == entity_name)
            {
                return true;
            }
            part_stack.extend(&part.parts);
        }
        pkg_stack.extend(&curr_pkg.packages);
    }
    false
}

/// Verifies post-filtering architectural scope survival and negative exclusion invariants on `pkg`. Realises: [REQ-SYSML-INGEST-SCOPE/verify_scope_and_negative_invariants]
pub fn verify_scope_and_negative_invariants(
    pkg: &PackageDef,
    allowed_parts: Option<&BTreeSet<String>>,
    negative_invariants: Option<&[String]>,
) -> Result<(), String> {
    if let Some(allowed) = allowed_parts.filter(|a| !a.is_empty() && !package_has_any_part(pkg)) {
        return Err(format!("Architectural scope verification failed: none of the --allowed-parts {:?} matched any extracted PartDef in package '{}'", allowed, pkg.name));
    }
    if let Some(neg_list) = negative_invariants {
        for inv in neg_list.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if package_contains_entity(pkg, inv) {
                return Err(format!("Architectural negative-invariant violation: forbidden entity '{}' exists in package '{}'", inv, pkg.name));
            }
        }
    }
    Ok(())
}

/// Collects all declared `PartDef` names across a `PackageDef` hierarchy into `out`. Realises: [REQ-SYSML-INGEST-SCOPE/collect_declared_part_names], [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
pub fn collect_declared_part_names(pkg: &PackageDef, out: &mut BTreeSet<String>) {
    let mut pkg_stack = vec![pkg];
    while let Some(curr_pkg) = pkg_stack.pop() {
        let mut part_stack: Vec<&PartDef> = curr_pkg.part_defs.iter().collect();
        while let Some(curr_part) = part_stack.pop() {
            if !curr_part.name.trim().is_empty() {
                out.insert(curr_part.name.clone());
            }
            part_stack.extend(&curr_part.parts);
        }
        pkg_stack.extend(&curr_pkg.packages);
    }
}

/// Filters a code-extracted `PackageDef` to retain only architectural `part def` components or realized constructs. Realises: [REQ-SYSML-INGEST-SCOPE/filter_ast_to_realized_or_allowed], [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
pub fn filter_ast_to_realized_or_allowed(pkg: &mut PackageDef, schema_parts: &BTreeSet<String>) {
    /// Recursively prunes `part.parts` and determines whether `part` is retained under realization scoping. Realises: [REQ-SYSML-INGEST-SCOPE/retain_realized_part]
    fn retain_realized_part(part: &mut PartDef, schema_parts: &BTreeSet<String>) -> bool {
        part.parts.retain_mut(|sub| retain_realized_part(sub, schema_parts));
        let has_realises_doc = part
            .doc
            .as_deref()
            .is_some_and(|d| d.contains("Realises:") && !d.contains("Realises: [UNMAPPED]"));
        schema_parts.contains(&part.name)
            || !part.satisfied_requirements.is_empty()
            || has_realises_doc
            || !part.parts.is_empty()
    }

    pkg.part_defs.retain_mut(|p| retain_realized_part(p, schema_parts));
    for sub_pkg in &mut pkg.packages {
        filter_ast_to_realized_or_allowed(sub_pkg, schema_parts);
    }
    pkg.packages.retain(|sub_pkg| {
        !sub_pkg.part_defs.is_empty()
            || !sub_pkg.action_defs.is_empty()
            || !sub_pkg.requirement_defs.is_empty()
            || !sub_pkg.packages.is_empty()
    });
}

/// Loads or parses and caches an ingested `SysmlModel` using `IncrementalAstCache`. Realises: [REQ-SYSML-INGEST-CACHE-MODEL]
pub fn cache_ingested_sysml_model(
    repo_root: &std::path::Path,
    sysml_source: &str,
) -> Result<(SysmlModel, bool), String> {
    let cache = IncrementalAstCache::new(repo_root);
    cache.load_or_parse(sysml_source).map_err(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_translate_markdown_api() -> Result<(), String> {
        let pkg = translate_markdown("# Package_0\n\n## Classifier_Alpha\nSynthetic element.", "Package_0")?;
        assert_eq!(pkg.name, "Package_0");
        assert_eq!(pkg.part_defs.first().map(|p| p.name.as_str()), Some("Classifier_Alpha"));
        Ok(())
    }

    #[test]
    fn test_filter_ast_to_target_scope_synthetic() {
        let mut pkg = PackageDef {
            name: "Package_0".to_string(),
            part_defs: vec![
                PartDef { name: "Classifier_Alpha".to_string(), ..Default::default() },
                PartDef { name: "Classifier_Extraneous".to_string(), ..Default::default() },
            ],
            ..Default::default()
        };
        let allowed = BTreeSet::from(["Classifier_Alpha".to_string()]);
        let neg = vec!["Phantom_Entity".to_string()];
        filter_ast_to_target_scope(&mut pkg, Some(&allowed), Some(&neg));
        assert_eq!(pkg.part_defs.first().map(|p| p.name.as_str()), Some("Classifier_Alpha"));
        assert_eq!(pkg.constraint_defs.first().map(|c| c.name.as_str()), Some("assert_exclusion_phantom_entity"));
        assert_eq!(pkg.constraint_defs.first().map(|c| c.expression.as_str()), Some("!exists(Phantom_Entity)"));
    }

    struct TempTestDir(std::path::PathBuf);
    impl TempTestDir {
        fn new(label: &str) -> Result<Self, std::io::Error> {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!("deap_ingest_test_{label}_{}_{id}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir)?;
            Ok(Self(dir))
        }
    }
    impl Drop for TempTestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_batch_translate_multiple_files_synthetic() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempTestDir::new("batch_synthetic")?;
        let (f1, f2) = (tmp.0.join("spec1.md"), tmp.0.join("spec2.md"));
        std::fs::write(&f1, "# Package_0\n\n## Classifier_Alpha\nAlpha doc.")?;
        std::fs::write(&f2, "# Package_0\n\n## Classifier_Beta\nBeta doc.")?;
        let pkg = cst::markdown_cst::MarkdownTranslator::new().translate_files(
            &[f1.to_string_lossy().to_string(), f2.to_string_lossy().to_string()],
            "Consolidated_Package",
        )?;
        assert_eq!(pkg.name, "Consolidated_Package");
        let p_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
        assert!(p_names.contains(&"Classifier_Alpha") && p_names.contains(&"Classifier_Beta"));
        Ok(())
    }

    #[test]
    fn test_batch_translators_fail_closed_on_empty_and_mixed_missing_inputs() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempTestDir::new("batch_fail_closed")?;
        let valid_md_path = tmp.0.join("valid.md");
        std::fs::write(&valid_md_path, "# Package_0\n\n## Classifier_Alpha\nAlpha doc.")?;
        let valid_md = valid_md_path.to_string_lossy().to_string();
        let missing_md = tmp.0.join("missing.md").to_string_lossy().to_string();
        let md_translator = cst::markdown_cst::MarkdownTranslator::new();
        assert!(md_translator.translate_files(&[], "Pkg").is_err());
        assert!(md_translator.translate_files(&[valid_md, missing_md], "Pkg").is_err());

        let valid_yang_path = tmp.0.join("valid.yang");
        std::fs::write(&valid_yang_path, "module valid-mod { namespace \"urn:test:valid\"; prefix vm; container root { leaf id { type string; } } }")?;
        let valid_yang = valid_yang_path.to_string_lossy().to_string();
        let missing_yang = tmp.0.join("missing.yang").to_string_lossy().to_string();
        let yang_translator = cst::yang_cst::YangTranslator::new();
        assert!(yang_translator.translate_files(&[], "Pkg").is_err());
        assert!(yang_translator.translate_files(std::slice::from_ref(&missing_yang), "Pkg").is_err());
        assert!(yang_translator.translate_files(&[valid_yang, missing_yang], "Pkg").is_err());
        Ok(())
    }

    #[test]
    fn test_ingest_sysml_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        for section in [
            "//! ## 1. Safety Intent & Regulatory Scope",
            "//! ## 2. Invariants & Mathematical Properties",
            "//! ## 3. Preconditions",
            "//! ## 4. Postconditions",
            "//! ## 5. Algorithmic Complexity Bounds",
            "//! ## 6. Failure Modes & Fault Tolerance",
            "//! ## 7. Realization & Traceability",
            "//! Realises: [REQ-SYSML-INGEST-LIB], [DO-330-TQL-4-INGEST-LIB]",
        ] {
            assert!(src.contains(section));
        }
    }

    #[test]
    fn test_cache_ingested_sysml_model_hit_and_invalidation() -> Result<(), Box<dyn std::error::Error>> {
        let tmp_dir = std::env::temp_dir().join(format!("deap_ingest_cache_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir)?;
        let sysml_v1 = "package IngestPkg { part def SensorUnit { attribute rate_hz : Real; } }";
        let (model_first, hit_first) = cache_ingested_sysml_model(&tmp_dir, sysml_v1)?;
        assert!(!hit_first && model_first.parts.len() == 1);
        let (model_second, hit_second) = cache_ingested_sysml_model(&tmp_dir, sysml_v1)?;
        assert!(hit_second && model_second == model_first);
        let sysml_v2 = "package IngestPkg { part def SensorUnit { attribute rate_hz : Real; } part def ActuatorUnit {} }";
        let (model_third, hit_third) = cache_ingested_sysml_model(&tmp_dir, sysml_v2)?;
        assert!(!hit_third && model_third.parts.len() == 2);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(())
    }

    #[test]
    fn test_issue_491_zero_blanket_lint_suppressions() -> Result<(), Box<dyn std::error::Error>> {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let targets = [
            "src/lib.rs", "src/main.rs", "src/cst/markdown_cst.rs", "src/cst/yang_cst.rs",
            "src/cst/code_cst.rs", "tests/code_ingestion_tests.rs", "tests/code_realization_scoping_tests.rs",
        ];
        let crate_allow = concat!("#![", "allow(");
        let crate_clippy = concat!("#![", "allow(", "clippy::");
        let item_clippy = concat!("#[", "allow(", "clippy::");
        let dead_code_attr = concat!("#[", "allow(", "dead_code)]");

        for rel in targets {
            let content = std::fs::read_to_string(manifest_dir.join(rel))?;
            assert!(!content.contains(crate_allow) && !content.contains(crate_clippy));
            assert!(!content.contains(item_clippy) && !content.contains(dead_code_attr));
        }
        assert!(cst::markdown_cst::MarkdownTranslator::new().translate_files(&[], "EmptyBatch").is_err());
        Ok(())
    }
}

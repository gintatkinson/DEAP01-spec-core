//! Unit tests for Contract A/B + 3-Layer DoD, Multi-File Schema/Spec Realization, and Remote Tracker Projection Gates.
//!
//! Safety & Traceability: Realises: [REQ-SPEC-CONTRACT-AB-DOD-GATE], [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE], [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]

use crate::spec_audit::{
    parse_markdown, project_body_for_tracker_with_prefix, validate_contract_ab_and_dod,
    validate_multi_file_schema_and_spec_realization_parity,
    validate_spec_remote_tracker_projection, validate_workspace_remote_tracker_projection,
    RemoteTrackerIssue,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Auto-cleaned temporary directory fixture for isolated unit tests.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Creates a new isolated temporary directory that is removed on drop.
    /// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
    fn new(name: &str) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "verify_contract_proj_{}_{}_{}",
            name, nanos, count
        ));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }

    /// Returns a reference to the temporary directory path.
    /// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    /// Removes the temporary directory tree on drop.
    /// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies that `validate_contract_ab_and_dod` fails closed when Contract A, Contract B, or 3-Layer DoD is missing.
/// Realises: [REQ-SPEC-CONTRACT-AB-DOD-GATE]
#[test]
fn test_contract_ab_and_dod_gate_fails_closed_when_missing_contract_a_or_b_or_3layer_dod(
) -> Result<(), Box<dyn std::error::Error>> {
    let missing_contracts_md = "\
# FEAT-11: GitHub CLI Resilience & Subprocess Bounds

| Field | Value |
| :--- | :--- |
| **Requirement ID** | `FEAT-11` |
| **Issue ID** | `#586` |

## 1. Description
Resilience and subprocess timeout bounds.

## 5. 3-Layer Definition of Done
- [ ] **Layer 1 (Domain State & Data Model)**: Subprocess timeout state.
- [ ] **Layer 2 (Logic & State Management)**: Retry logic.
- [ ] **Layer 3 (Presentation & Actuator Interface Binding)**: CLI output.
";
    let doc1 = parse_markdown(missing_contracts_md)?;
    let errs1 = validate_contract_ab_and_dod(
        "feature",
        &doc1,
        missing_contracts_md,
        "docs/features/FEAT-11-github-cli-resilience.md",
    );
    assert!(errs1.iter().any(|e| e.contains("Contract A")));
    assert!(errs1.iter().any(|e| e.contains("Contract B")));

    let missing_dod_md = "\
# UC-10: Execute 4-Level Hierarchical Codegen Lowering Pipeline

| Field | Value |
| :--- | :--- |
| **Use Case ID** | `UC-10` |
| **Issue ID** | `#582` |

## 1. Overview & Dual-Contract Grounding
- **Contract A (Design-Time AST & Mathematical Formulation)**: AST lowering rules.
- **Contract B (Runtime Verification & Execution Invariants)**: Execution bounds.
";
    let doc2 = parse_markdown(missing_dod_md)?;
    let errs2 = validate_contract_ab_and_dod(
        "use_case",
        &doc2,
        missing_dod_md,
        "docs/use-cases/UC-10-execute-4-level-hierarchical-codegen-lowering-pipeline.md",
    );
    assert!(errs2.iter().any(|e| e.contains("3-Layer Definition of Done")));
    Ok(())
}

/// Verifies that `validate_contract_ab_and_dod` passes when Contract A, Contract B, and 3-Layer DoD are present.
/// Realises: [REQ-SPEC-CONTRACT-AB-DOD-GATE]
#[test]
fn test_contract_ab_and_dod_gate_passes_when_contract_a_b_and_3layer_dod_present(
) -> Result<(), Box<dyn std::error::Error>> {
    let compliant_md = "\
# FEAT-01: Unified SysML v2 AST Extractor

| Field | Value |
| :--- | :--- |
| **Requirement ID** | `FEAT-01` |
| **Issue ID** | `#573` |

## 1. Dual-Contract Grounding (Contract A & Contract B)
- **Contract A (Design-Time AST & Mathematical Formulation)**: Recursive descent parser.
- **Contract B (Runtime Verification & Execution Invariants)**: Bounded O(N) scan.

## 5. 3-Layer Definition of Done
- [ ] **Layer 1 (Domain State & Data Model)**: AST structs.
- [ ] **Layer 2 (Logic & State Management)**: Extractor pipeline.
- [ ] **Layer 3 (Presentation & Actuator Interface Binding)**: Diagnostic CLI output.
";
    let doc = parse_markdown(compliant_md)?;
    let errs = validate_contract_ab_and_dod(
        "feature",
        &doc,
        compliant_md,
        "docs/features/FEAT-01-unified-sysml-v2-ast-extractor.md",
    );
    assert!(errs.is_empty(), "Expected compliant spec to pass, got: {:?}", errs);
    Ok(())
}

/// Verifies that `validate_multi_file_schema_and_spec_realization_parity` fails closed on split schema files.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
#[test]
fn test_multi_file_schema_parity_fails_closed_on_split_model_sysml_or_pipeline_schema_drift(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("split_schema");
    let root = tmp.path();
    fs::create_dir_all(root.join("schema"))?;
    fs::create_dir_all(root.join(".pipeline"))?;

    let model_sysml = "package SysML_Model {\n  part def CoreEngine { attribute version : Integer; }\n}\n";
    let modular_sysml = "package ModularCodegen {\n  part def ModularityBoundsSpec { attribute max_sloc : Integer; }\n}\n";
    let combined = format!("{}\n{}", model_sysml, modular_sysml);
    fs::write(root.join("schema/model.sysml"), model_sysml)?;
    fs::write(root.join("schema/modular_codegen.sysml"), modular_sysml)?;
    fs::write(root.join(".pipeline/schema.sysml"), &combined)?;

    let errs = validate_multi_file_schema_and_spec_realization_parity(root);
    assert!(
        errs.iter()
            .any(|e| e.contains("ModularityBoundsSpec") && e.contains("model.sysml")),
        "Expected failure for split schema/modular_codegen.sysml vs model.sysml, got: {:?}",
        errs
    );
    Ok(())
}

/// Verifies that `validate_multi_file_schema_and_spec_realization_parity` fails closed when AST defs lack spec files.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
#[test]
fn test_multi_file_schema_realization_fails_closed_on_unrealized_part_def_use_case_def_or_action_def(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("unrealized_defs");
    let root = tmp.path();
    fs::create_dir_all(root.join("schema"))?;
    fs::create_dir_all(root.join(".pipeline"))?;
    fs::create_dir_all(root.join("docs/features"))?;
    fs::create_dir_all(root.join("docs/use-cases"))?;
    fs::create_dir_all(root.join("docs/user-stories"))?;

    let sysml = "\
package SysML_Model {
    part def CoreEngine {}
    part def ModularityBoundsSpec {}
    use case def UC_01_Parse_Core {}
    use case def UC_02_Missing_UseCase {}
    action def Act_01_Run_Core {}
    action def ExecuteFourLevelLoweringPipeline {}
}
";
    fs::write(root.join("schema/model.sysml"), sysml)?;
    fs::write(root.join(".pipeline/schema.sysml"), sysml)?;
    fs::write(root.join("docs/features/FEAT-01-core-engine.md"), "# FEAT-01: CoreEngine\nRealizes `CoreEngine`.")?;
    fs::write(root.join("docs/use-cases/UC-01-parse-core.md"), "# UC-01: Parse Core\nRealizes `UC_01_Parse_Core`.")?;
    fs::write(root.join("docs/user-stories/US-01-run-core.md"), "# US-01: Run Core\nRealizes `Act_01_Run_Core`.")?;

    let errs = validate_multi_file_schema_and_spec_realization_parity(root);
    assert!(errs.iter().any(|e| e.contains("ModularityBoundsSpec")));
    assert!(errs.iter().any(|e| e.contains("UC_02_Missing_UseCase")));
    assert!(errs.iter().any(|e| e.contains("ExecuteFourLevelLoweringPipeline")));
    Ok(())
}

/// Verifies that `validate_multi_file_schema_and_spec_realization_parity` passes when unified and realized.
/// Realises: [REQ-MULTI-FILE-SCHEMA-REALIZATION-GATE]
#[test]
fn test_multi_file_schema_realization_passes_when_unified_and_realized(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("unified_realized");
    let root = tmp.path();
    fs::create_dir_all(root.join("schema"))?;
    fs::create_dir_all(root.join(".pipeline"))?;
    fs::create_dir_all(root.join("docs/features"))?;
    fs::create_dir_all(root.join("docs/use-cases"))?;
    fs::create_dir_all(root.join("docs/user-stories"))?;

    let sysml = "package SysML_Model {\n  part def CoreEngine {}\n  use case def UC_01_Parse_Core {}\n  action def Act_01_Run_Core {}\n}\n";
    fs::write(root.join("schema/model.sysml"), sysml)?;
    fs::write(root.join(".pipeline/schema.sysml"), sysml)?;
    fs::write(root.join("docs/features/FEAT-01-core-engine.md"), "# FEAT-01: CoreEngine\nRealizes `CoreEngine`.")?;
    fs::write(root.join("docs/use-cases/UC-01-parse-core.md"), "# UC-01: Parse Core\nRealizes `UC_01_Parse_Core`.")?;
    fs::write(root.join("docs/user-stories/US-01-run-core.md"), "# US-01: Run Core\nRealizes `Act_01_Run_Core`.")?;

    let errs = validate_multi_file_schema_and_spec_realization_parity(root);
    assert!(errs.is_empty(), "Expected unified and realized schema to pass, got: {:?}", errs);
    Ok(())
}

/// Verifies that remote tracker projection parity fails closed on missing ID, `[AUDIT]` collision, drift, and premature resolution.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
#[test]
fn test_remote_tracker_projection_fails_closed_on_missing_issue_audit_collision_title_body_drift_and_premature_fixed_resolved(
) -> Result<(), Box<dyn std::error::Error>> {
    let spec_md = "\
# FEAT-01: Unified SysML v2 AST Extractor

| Field | Value |
| :--- | :--- |
| **Requirement ID** | `FEAT-01` |
| **Issue ID** | `#573` |
| **Parent Epic** | [EPIC-01](../epics/EPIC-01-core.md) |

## 5. 3-Layer Definition of Done
- [ ] **Layer 1**: AST structs.
- [ ] **Layer 2**: Parser logic.
- [ ] **Layer 3**: CLI binding.
";
    let doc = parse_markdown(spec_md)?;
    let empty_map = BTreeMap::new();
    let errs_missing = validate_spec_remote_tracker_projection(
        "docs/features/FEAT-01.md",
        &doc,
        spec_md,
        &empty_map,
        (None, "main", None),
    );
    assert!(errs_missing.iter().any(|e| e.contains("#573")));

    let mut collision_map = BTreeMap::new();
    collision_map.insert(
        573,
        RemoteTrackerIssue {
            number: 573,
            title: "[AUDIT] Unimplemented Feature: FEAT-15".to_string(),
            body: "Audit body".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:feature".to_string(), "status:fixed-resolved".to_string()],
        },
    );
    let errs_collision = validate_spec_remote_tracker_projection(
        "docs/features/FEAT-01.md",
        &doc,
        spec_md,
        &collision_map,
        (Some("org/repo"), "main", None),
    );
    assert!(errs_collision.iter().any(|e| e.contains("[AUDIT]")));
    assert!(errs_collision.iter().any(|e| e.contains("title")));
    assert!(errs_collision.iter().any(|e| e.contains("body")));
    assert!(errs_collision.iter().any(|e| e.contains("status:fixed-resolved")));
    Ok(())
}

/// Verifies that remote tracker projection parity passes when remote title, projected body, and status align.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
#[test]
fn test_remote_tracker_projection_passes_when_title_body_and_status_match(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("tracker_match");
    let root = tmp.path();
    fs::create_dir_all(root.join("docs/features"))?;

    let spec_md = "\
# FEAT-01: Unified SysML v2 AST Extractor

| Field | Value |
| :--- | :--- |
| **Requirement ID** | `FEAT-01` |
| **Issue ID** | `#573` |
| **Parent Epic** | [EPIC-01](../epics/EPIC-01-core.md) |

## 5. 3-Layer Definition of Done
- [ ] **Layer 1**: AST structs.
- [ ] **Layer 2**: Parser logic.
- [ ] **Layer 3**: CLI binding.
";
    fs::write(root.join("docs/features/FEAT-01.md"), spec_md)?;
    let proj_ctx = (Some("org/repo"), "main", None);
    let projected_body = project_body_for_tracker_with_prefix(
        spec_md,
        "docs/features/FEAT-01.md",
        proj_ctx.0,
        proj_ctx.1,
        proj_ctx.2,
    );
    let mut remote_map = BTreeMap::new();
    remote_map.insert(
        573,
        RemoteTrackerIssue {
            number: 573,
            title: "FEAT-01: Unified SysML v2 AST Extractor".to_string(),
            body: projected_body,
            state: "OPEN".to_string(),
            labels: vec!["type:feature".to_string(), "status:needs-triage".to_string()],
        },
    );
    let errs = validate_workspace_remote_tracker_projection(root, &remote_map, proj_ctx);
    assert!(errs.is_empty(), "Expected matching remote projection to pass, got: {:?}", errs);
    Ok(())
}

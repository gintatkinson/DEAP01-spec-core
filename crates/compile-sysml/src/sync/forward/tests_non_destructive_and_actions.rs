#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Action Round-Trip Parity, Parameterized Signatures, and ICD/WBS Compilation Unit Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
//! Verifies `PartDef::actions` and `PartDef::operations` deduplicated rendering across Feature
//! and Epic `classDiagram` blocks (`DEFECT-07`), parameterized `ActionDef` Mermaid and Markdown
//! bullets, logical ICD and WBS compilation, and HTML comment state tracking (`REQ-0604`).
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Temporary directories and in-memory `PackageDef` fixtures are initialized.
//! - **Postconditions**: Operations and actions are deduplicated with zero duplicate signatures.
//! - **Invariants**: All temporary filesystem directories are cleaned up before test return.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Test assertions return `Result<(), Box<dyn std::error::Error>>` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Uses PID-scoped temporary directories and pure in-memory AST structures.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Exercises `reverse_sync_specs_to_sysml`, `generate_class_diagram`, `generate_epic_class_diagram`, `compile_logical_icd_artifacts`, and `compile_wbs_artifacts`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]

use super::*;
use deap_core::sysml_cst::{ActionDef, OperationDef, PartDef};

/// Creates the `DEFECT-07` temporary Feature Markdown fixture and initial `FlightControlSystem` AST.
/// Realises: [REQ-0604]
fn setup_defect_07_roundtrip_fixture(
    tmp_dir: &Path,
) -> Result<PackageDef, Box<dyn std::error::Error>> {
    if tmp_dir.exists() {
        std::fs::remove_dir_all(tmp_dir)?;
    }
    let docs_features_dir = tmp_dir.join("docs").join("features");
    std::fs::create_dir_all(&docs_features_dir)?;

    let feat_md = "| Attribute | Specification Detail |\n\
| :--- | :--- |\n\
| **Issue ID** | #10 |\n\
| **Title** | Feature 01: [Guidance Subsystem] GuidanceController |\n\
| **Type** | feature |\n\
| **Schema Containers** | `GuidanceSubsystem::GuidanceController` |\n\
\n\
# Feature 01: [Guidance Subsystem] GuidanceController\n\
\n\
## Interface Requirements\n\
- `+ComputeTrajectory() : void`\n\
- `+SharedOperation() : Boolean`\n";
    std::fs::write(
        docs_features_dir.join("FEAT-01-GuidanceController.md"),
        feat_md,
    )?;

    let mut root = PackageDef {
        name: "FlightControlSystem".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "GuidanceSubsystem".to_string(),
        parent_package: Some("FlightControlSystem".to_string()),
        ..Default::default()
    };
    let mut controller = PartDef {
        name: "GuidanceController".to_string(),
        ..Default::default()
    };
    controller.operations.push(OperationDef {
        name: "SharedOperation".to_string(),
        return_type: Some("Boolean".to_string()),
        ..Default::default()
    });
    controller.actions.push(ActionDef {
        name: "PreExistingAction".to_string(),
        ..Default::default()
    });
    sub.part_defs.push(controller);
    root.packages.push(sub);
    Ok(root)
}

/// Asserts that both `generate_class_diagram` and `generate_epic_class_diagram` deduplicate `SharedOperation` and include `ComputeTrajectory`.
/// Realises: [REQ-0604]
fn assert_defect_07_class_diagrams(synced_sub: &PackageDef, synced_part: &PartDef) {
    let feat_diag = generate_class_diagram(synced_part);
    assert!(feat_diag.contains("+SharedOperation() Boolean"));
    assert!(feat_diag.contains("+PreExistingAction() void"));
    assert!(feat_diag.contains("+ComputeTrajectory() void"));
    let shared_count_feat = feat_diag
        .lines()
        .filter(|l| l.contains("+SharedOperation("))
        .count();
    assert_eq!(shared_count_feat, 1);

    let epic_diag = generate_epic_class_diagram(synced_sub, "GuidanceController", false);
    assert!(epic_diag.contains("+SharedOperation() Boolean"));
    assert!(epic_diag.contains("+PreExistingAction() void"));
    assert!(epic_diag.contains("+ComputeTrajectory() void"));
    let shared_count_epic = epic_diag
        .lines()
        .filter(|l| l.contains("+SharedOperation("))
        .count();
    assert_eq!(shared_count_epic, 1);
}

/// Asserts root Epic and actions-only subpart classDiagram rendering for `DEFECT-07`.
/// Realises: [REQ-0604]
fn assert_defect_07_root_and_actions_only_diagrams(
    synced_part: &PartDef,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root_with_part = PackageDef {
        name: "GuidanceSubsystem".to_string(),
        part_defs: vec![synced_part.clone()],
        ..Default::default()
    };
    let mut actions_only_subpart = PartDef {
        name: "ActuatorUnit".to_string(),
        ..Default::default()
    };
    actions_only_subpart.actions.push(ActionDef {
        name: "DeflectFin".to_string(),
        ..Default::default()
    });
    root_with_part.part_defs.push(actions_only_subpart);

    let root_epic_diag = generate_epic_class_diagram(&root_with_part, "GuidanceController", true);
    assert!(root_epic_diag.contains("+SharedOperation() Boolean"));
    assert!(root_epic_diag.contains("+ComputeTrajectory() void"));
    assert!(root_epic_diag.contains("+DeflectFin() void"));

    let actions_only_feat_diag = generate_class_diagram(
        root_with_part
            .part_defs
            .get(1)
            .ok_or("expected ActuatorUnit subpart")?,
    );
    assert!(actions_only_feat_diag.contains("class ActuatorUnit {"));
    assert!(actions_only_feat_diag.contains("+DeflectFin() void"));
    Ok(())
}

/// Verifies `PartDef::actions` (including actions populated by `reverse_sync_specs_to_sysml`) render in `generate_class_diagram` and `generate_epic_class_diagram` without duplicating `part.operations`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
#[test]
fn test_defect_07_forward_reverse_sync_actions_roundtrip_parity(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::sync::reverse::{reverse_sync_specs_to_sysml, ReverseSyncOptions};

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_defect_07_sync_roundtrip_{}",
        std::process::id()
    ));
    let root = setup_defect_07_roundtrip_fixture(&tmp_dir)?;

    let rev_opts = ReverseSyncOptions {
        docs_dir: tmp_dir.join("docs"),
        schema_path: None,
        output_path: tmp_dir.join(".pipeline").join("schema.sysml"),
        digest_path: tmp_dir.join(".pipeline").join("schema-digest.json"),
        allow_overwrite: true,
    };

    let (synced_root, _digest) = reverse_sync_specs_to_sysml(Some(&root), &rev_opts)?;
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }

    let synced_sub = synced_root
        .packages
        .first()
        .ok_or("expected GuidanceSubsystem package")?;
    let synced_part = synced_sub
        .part_defs
        .first()
        .ok_or("expected GuidanceController part")?;

    assert!(synced_part
        .actions
        .iter()
        .any(|a| a.name == "ComputeTrajectory"));
    assert_defect_07_class_diagrams(synced_sub, synced_part);
    assert_defect_07_root_and_actions_only_diagrams(synced_part)?;

    let fwd_opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&synced_root, &fwd_opts)?;
    let gen_feat = generated
        .iter()
        .find(|(k, _)| k.contains("FEAT-") && k.contains("GuidanceController"))
        .map(|(_, v)| v)
        .ok_or("expected generated FEAT-01-GuidanceController.md")?;
    let gen_epic = generated
        .iter()
        .find(|(k, _)| k.contains("EPIC-") && k.contains("GuidanceSubsystem"))
        .map(|(_, v)| v)
        .ok_or("expected generated EPIC for GuidanceSubsystem")?;

    assert!(gen_feat.contains("+ComputeTrajectory() void"));
    assert!(gen_epic.contains("+ComputeTrajectory() void"));
    Ok(())
}

/// Builds the `GncSystem` fixture for `REQ-0604` ICD and WBS compilation tests.
/// Realises: [REQ-0604]
fn build_req_0604_gnc_model(act: ActionDef) -> PackageDef {
    use deap_core::sysml_cst::{ConnectionDef, PortDef};

    let mut pkg = PackageDef {
        name: "GncSystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "AttitudeSensor".to_string(),
        ports: vec![PortDef {
            name: "q_out".to_string(),
            type_name: "Quaternion".to_string(),
            direction: "out".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    pkg.part_defs.push(PartDef {
        name: "AttitudeController".to_string(),
        ports: vec![PortDef {
            name: "q_in".to_string(),
            type_name: "Quaternion".to_string(),
            direction: "in".to_string(),
            ..Default::default()
        }],
        actions: vec![act],
        ..Default::default()
    });
    pkg.connection_defs.push(ConnectionDef {
        name: "QuatBus".to_string(),
        source_part: Some("AttitudeSensor".to_string()),
        source_port: "q_out".to_string(),
        target_part: Some("AttitudeController".to_string()),
        target_port: "q_in".to_string(),
        item_flow_ref: Some("Quaternion".to_string()),
        is_flow: true,
        ..Default::default()
    });
    pkg
}

/// Verifies `REQ-0604` (`BP-04-FORWARD-SYNC-ICD-WBS`) `AC-01` through `AC-05`.
/// Realises: [REQ-0604]
#[test]
fn test_req_0604_forward_sync_parameterized_actions_icd_wbs_and_html_comment_guards(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_cst::AttributeDef;

    let act = ActionDef {
        name: "SolveAttitude".to_string(),
        in_params: vec![AttributeDef {
            name: "quat_in".to_string(),
            type_name: "Quaternion".to_string(),
            ..Default::default()
        }],
        out_params: vec![AttributeDef {
            name: "torque_cmd".to_string(),
            type_name: "TorqueVector".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let sig = format_mermaid_operation_signature(&act);
    assert_eq!(
        sig,
        "+SolveAttitude(in_quat_in_Quaternion, out_torque_cmd_TorqueVector) TorqueVector"
    );
    let bullet = format_markdown_operation_bullet(&act);
    assert_eq!(
        bullet,
        "- `+SolveAttitude(in quat_in: Quaternion, out torque_cmd: TorqueVector) : TorqueVector`"
    );
    assert_eq!(parse_markdown_operation_bullet(&bullet), Some(act.clone()));
    assert_eq!(parse_markdown_operation_bullet("invalid"), None);

    let pkg = build_req_0604_gnc_model(act);
    let (icd01, icd02) = compile_logical_icd_artifacts(&pkg);
    assert!(icd01.contains("| **AttitudeController** | --- |"));
    assert!(icd01.contains("| **AttitudeSensor** | `Quaternion` | --- |"));
    assert!(icd02.contains("Quaternion"));

    let wbs = compile_wbs_artifacts(&pkg);
    assert!(wbs.wbs_01_md.contains("01.00.00"));
    assert!(wbs.wbs_02_md.contains("FEAT-01"));

    let mut in_comment = true;
    assert!(advance_html_comment_state(
        "close --> mid <!-- reopen",
        &mut in_comment
    ));
    assert!(in_comment);

    let specs = vec![("epics/EPIC-02-ConOps.md".to_string(), String::new())];
    assert_eq!(
        count_total_epics_inclusive("epics/EPIC-01-GncSystem.md", true, &specs),
        2
    );
    Ok(())
}

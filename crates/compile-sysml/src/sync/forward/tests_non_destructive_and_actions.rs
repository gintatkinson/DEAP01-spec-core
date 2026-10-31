//! Reverse-to-forward action roundtrip deduplication, ConOps behavioral projection, and non-destructive merge/cleanup unit tests.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
//! Contract: Verifies `PartDef::actions` and `PartDef::operations` deduplicated rendering across Feature and Epic `classDiagram` blocks, `ConOps` and standalone `InteractionDef` Use Case / User Story projection, `!opts.force` authored prose and `- [x]` checkbox preservation, and stale colliding prefix file removal.
//! Bounds & Faults: Isolates filesystem fixtures inside unique auto-cleaned temporary directories returning `Result<(), Box<dyn std::error::Error>>` with zero panics.

use super::*;
use deap_core::sysml_ast::PartDef;

/// Verifies `PartDef::actions` (including actions populated by `reverse_sync_specs_to_sysml`) render in `generate_class_diagram` and `generate_epic_class_diagram` without duplicating `part.operations`.
///
/// Realises: [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
#[test]
fn test_defect_07_forward_reverse_sync_actions_roundtrip_parity(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::sync::reverse::{reverse_sync_specs_to_sysml, ReverseSyncOptions};
    use deap_core::sysml_ast::{ActionDef, OperationDef};

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_defect_07_sync_roundtrip_{}",
        std::process::id()
    ));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
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

    assert!(
        synced_part
            .actions
            .iter()
            .any(|a| a.name == "ComputeTrajectory"),
        "reverse_sync_specs_to_sysml must populate ComputeTrajectory in PartDef::actions"
    );

    let feat_diag = generate_class_diagram(synced_part);
    assert!(
        feat_diag.contains("+SharedOperation() Boolean"),
        "generate_class_diagram must render operation signature, got:\n{}",
        feat_diag
    );
    assert!(
        feat_diag.contains("+PreExistingAction() void"),
        "generate_class_diagram must render pre-existing PartDef::actions, got:\n{}",
        feat_diag
    );
    assert!(
        feat_diag.contains("+ComputeTrajectory() void"),
        "generate_class_diagram must render reverse-synced PartDef::actions, got:\n{}",
        feat_diag
    );
    let shared_count_feat = feat_diag
        .lines()
        .filter(|l| l.contains("+SharedOperation("))
        .count();
    assert_eq!(
        shared_count_feat, 1,
        "SharedOperation must not be duplicated when present in both operations and actions"
    );

    let epic_diag = generate_epic_class_diagram(synced_sub, "GuidanceController", false);
    assert!(
        epic_diag.contains("+SharedOperation() Boolean"),
        "generate_epic_class_diagram must render operation signature, got:\n{}",
        epic_diag
    );
    assert!(
        epic_diag.contains("+PreExistingAction() void"),
        "generate_epic_class_diagram must render pre-existing PartDef::actions, got:\n{}",
        epic_diag
    );
    assert!(
        epic_diag.contains("+ComputeTrajectory() void"),
        "generate_epic_class_diagram must render reverse-synced PartDef::actions, got:\n{}",
        epic_diag
    );
    let shared_count_epic = epic_diag
        .lines()
        .filter(|l| l.contains("+SharedOperation("))
        .count();
    assert_eq!(
        shared_count_epic, 1,
        "SharedOperation must not be duplicated in generate_epic_class_diagram"
    );

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
    assert!(
        root_epic_diag.contains("+SharedOperation() Boolean"),
        "root generate_epic_class_diagram must render root_part operations, got:\n{}",
        root_epic_diag
    );
    assert!(
        root_epic_diag.contains("+ComputeTrajectory() void"),
        "root generate_epic_class_diagram must render root_part actions, got:\n{}",
        root_epic_diag
    );
    assert!(
        root_epic_diag.contains("+DeflectFin() void"),
        "root generate_epic_class_diagram must render actions-only subpart actions, got:\n{}",
        root_epic_diag
    );

    let actions_only_feat_diag = generate_class_diagram(
        root_with_part
            .part_defs
            .get(1)
            .ok_or("expected ActuatorUnit subpart")?,
    );
    assert!(
        actions_only_feat_diag.contains("class ActuatorUnit {"),
        "generate_class_diagram must open class block when only PartDef::actions is non-empty, got:\n{}",
        actions_only_feat_diag
    );
    assert!(
        actions_only_feat_diag.contains("+DeflectFin() void"),
        "generate_class_diagram must render action in actions-only PartDef, got:\n{}",
        actions_only_feat_diag
    );

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

    assert!(
        gen_feat.contains("+ComputeTrajectory() void"),
        "Forward-synced Feature markdown must preserve reverse-synced action in classDiagram"
    );
    assert!(
        gen_epic.contains("+ComputeTrajectory() void"),
        "Forward-synced Epic markdown must preserve reverse-synced action in classDiagram"
    );

    Ok(())
}

/// Verifies Issue #477: `ConOps` + `part.actions` + `p.interaction_defs` UC/US projection, `!opts.force` authored prose & `- [x]` checkbox preservation, and colliding prefix cleanup.
///
/// Realises: [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
#[test]
fn test_issue_477_conops_part_behavior_projection_and_non_destructive_sync(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{ActionDef, InteractionDef, PartDef, UseCaseDef};

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_issue_477_forward_sync_{}",
        std::process::id()
    ));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    let docs_dir = tmp_dir.join("docs");
    std::fs::create_dir_all(docs_dir.join("epics"))?;
    std::fs::create_dir_all(docs_dir.join("features"))?;
    std::fs::create_dir_all(docs_dir.join("use-cases"))?;
    std::fs::create_dir_all(docs_dir.join("user-stories"))?;

    let existing_epic = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #2 |
| **Title** | Epic 02: Concept of Operations Operational Activities |
| **Type** | epic |

# Epic 02: Concept of Operations Operational Activities

## 1. Context
Custom authored ConOps context narrative that must not be clobbered.

## 2. Requirements & Checklist
- [x] #12 - [Feature 01: [Con Ops] MissionOperator](../features/FEAT-01-MissionOperator.md) (conops.sysml)

### Associated Use Cases & User Stories

#### Associated Use Cases
- [x] #42 - [Use Case 01: Mission Planning](../use-cases/UC-01-MissionPlanning.md)

#### Associated User Stories
- [ ] TBD - [User Story 01: Execute Mission Plan Action](../user-stories/US-01-ExecuteMissionPlanAction.md)

## 3. Architecture
Custom authored architecture narrative with detailed telemetry buses.

## 4. Operational Considerations
Custom authored operational considerations for mission timing bounds.

## 5. Security & Governance
Custom authored security and zero-trust governance controls.

## 6. Source References
- Custom source ref
"#;
    std::fs::write(docs_dir.join("epics/EPIC-02-ConOps.md"), existing_epic)?;

    let existing_feat = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #12 |
| **Title** | Feature 01: [Con Ops] MissionOperator |
| **Type** | feature |

# Feature 01: [Con Ops] MissionOperator

## Interface Requirements
### 1. Test Data Shape
```json
{}
```

### 2. Validation & Constraints
Custom authored validation constraints for MissionOperator.

### 3. Visual Layout & Arrangement
Custom authored logical containment and structural arrangement rules.

### 4. Interactive Flow & States
Custom authored interactive flow and operator state transitions.

### Layer 1: Domain State & Signal Model
Custom authored Layer 1 domain state model.

### Layer 2: Logic & Safety State Management
Custom authored Layer 2 safety state management logic.

### Layer 3: Presentation & Actuator Interface Binding
Custom authored Layer 3 presentation actuator bindings.

## Acceptance Criteria (BDD)
- [x] AC-01: Given nominal MissionOperator domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds.
"#;
    std::fs::write(
        docs_dir.join("features/FEAT-01-MissionOperator.md"),
        existing_feat,
    )?;

    let stale_uc = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #42 |
| **Title** | Use Case 01: Old Obsolete Slug |
"#;
    let stale_uc_path = docs_dir.join("use-cases/UC-01-OldObsoleteSlug.md");
    std::fs::write(&stale_uc_path, stale_uc)?;

    let mut root = PackageDef {
        name: "SysML_Model".to_string(),
        ..Default::default()
    };

    let mut conops = PackageDef {
        name: "ConOps".to_string(),
        ..Default::default()
    };
    conops.part_defs.push(PartDef {
        name: "MissionOperator".to_string(),
        ..Default::default()
    });
    conops.action_defs.push(ActionDef {
        name: "ExecuteMissionPlanAction".to_string(),
        ..Default::default()
    });
    conops.use_case_defs.push(UseCaseDef {
        name: "UC_01_MissionPlanning".to_string(),
        ..Default::default()
    });
    root.packages.push(conops);

    let mut sensor_pkg = PackageDef {
        name: "SensorSubsystem".to_string(),
        ..Default::default()
    };
    let mut radar_part = PartDef {
        name: "RadarSensor".to_string(),
        ..Default::default()
    };
    radar_part.actions.push(ActionDef {
        name: "CalibrateBeamAction".to_string(),
        ..Default::default()
    });
    sensor_pkg.part_defs.push(radar_part);
    sensor_pkg.use_case_defs.push(UseCaseDef {
        name: "UC_02_CalibrateSensor".to_string(),
        ..Default::default()
    });
    root.packages.push(sensor_pkg);

    let mut comms_pkg = PackageDef {
        name: "CommsSubsystem".to_string(),
        ..Default::default()
    };
    comms_pkg.part_defs.push(PartDef {
        name: "RadioLink".to_string(),
        ..Default::default()
    });
    comms_pkg.interaction_defs.push(InteractionDef {
        name: "EstablishLinkInteraction".to_string(),
        lifelines: vec![
            "op : MissionOperator".to_string(),
            "radio : RadioLink".to_string(),
        ],
        messages: vec![
            "OpenChannel from op to radio".to_string(),
            "LinkEstablished from radio to op".to_string(),
        ],
        ..Default::default()
    });
    root.packages.push(comms_pkg);

    let opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema/model.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: None,
        dry_run: false,
        force: false,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;

    assert!(
        generated.contains_key("use-cases/UC-01-MissionPlanning.md"),
        "ConOps use case must be projected into UC-01-MissionPlanning.md"
    );
    assert!(
        generated.contains_key("user-stories/US-01-ExecuteMissionPlanAction.md"),
        "ConOps action must be projected into US-01-ExecuteMissionPlanAction.md"
    );
    assert!(
        generated.contains_key("user-stories/US-02-CalibrateBeamAction.md"),
        "PartDef::actions on RadarSensor must be projected into US-02-CalibrateBeamAction.md"
    );
    let uc02 = generated
        .get("use-cases/UC-02-CalibrateSensor.md")
        .ok_or("expected UC-02-CalibrateSensor.md")?;
    assert!(
        uc02.contains("US-02-CalibrateBeamAction.md"),
        "UC-02 realization matrix must link to US-02-CalibrateBeamAction.md"
    );
    assert!(
        generated.contains_key("user-stories/US-03-EstablishLinkInteraction.md"),
        "Standalone interaction_defs on CommsSubsystem must be projected into US-03-EstablishLinkInteraction.md"
    );

    let epic02 = generated
        .get("epics/EPIC-02-ConOps.md")
        .ok_or("expected epics/EPIC-02-ConOps.md")?;
    assert!(epic02.contains("Custom authored ConOps context narrative that must not be clobbered."));
    assert!(
        epic02.contains("Custom authored architecture narrative with detailed telemetry buses.")
    );
    assert!(
        epic02.contains("Custom authored operational considerations for mission timing bounds.")
    );
    assert!(epic02.contains("Custom authored security and zero-trust governance controls."));
    assert!(epic02.contains("- [x] #12 - [Feature 01: [Con Ops] MissionOperator]"));
    assert!(epic02.contains("- [x] #42 - [Use Case 01: Mission Planning]"));
    assert!(!epic02.contains("CalibrateBeamAction"));

    let feat01 = generated
        .get("features/FEAT-01-MissionOperator.md")
        .ok_or("expected features/FEAT-01-MissionOperator.md")?;
    assert!(feat01.contains("Custom authored validation constraints for MissionOperator."));
    assert!(
        feat01.contains("Custom authored logical containment and structural arrangement rules.")
    );
    assert!(feat01.contains("Custom authored interactive flow and operator state transitions."));
    assert!(feat01.contains("Custom authored Layer 1 domain state model."));
    assert!(feat01.contains("Custom authored Layer 2 safety state management logic."));
    assert!(feat01.contains("Custom authored Layer 3 presentation actuator bindings."));
    assert!(feat01.contains("- [x] AC-01:"));

    assert!(
        !stale_uc_path.exists(),
        "Stale colliding file UC-01-OldObsoleteSlug.md must be cleaned up"
    );
    assert!(
        docs_dir.join("use-cases/UC-01-MissionPlanning.md").exists(),
        "Active UC-01-MissionPlanning.md must exist on disk"
    );

    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    Ok(())
}

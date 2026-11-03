#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # ConOps Behavioral Projection and Non-Destructive Merge/Cleanup Unit Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
//! Verifies `ConOps`, `PartDef::actions`, and standalone `InteractionDef` Use Case / User Story
//! projection, `!opts.force` authored prose and `- [x]` checkbox preservation, and stale
//! colliding prefix file removal (`Issue #477`).
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Temporary directory fixtures with pre-authored Epic/Feature files exist.
//! - **Postconditions**: Authored prose and checked `- [x]` items are preserved across sync.
//! - **Invariants**: Stale colliding prefix files (`UC-01-OldObsoleteSlug.md`) are removed.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Test assertions return `Result<(), Box<dyn std::error::Error>>` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Uses isolated PID-scoped temporary directories cleaned up prior to exit.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Exercises `forward_sync_sysml_to_specs` with `force: false, dry_run: false`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]

use super::*;
use deap_core::sysml_cst::{ActionDef, InteractionDef, PartDef, UseCaseDef};

/// Writes the pre-existing `EPIC-02-ConOps.md` fixture to `docs_dir`.
/// Realises: [REQ-0604]
fn write_issue_477_existing_epic_fixture(
    docs_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

/// Writes pre-existing `FEAT-01-MissionOperator.md` and `UC-01-OldObsoleteSlug.md` fixtures to `docs_dir`.
/// Realises: [REQ-0604]
fn write_issue_477_existing_feat_and_stale_uc_fixtures(
    docs_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    write_issue_477_existing_epic_fixture(docs_dir)?;
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
    Ok(stale_uc_path)
}

/// Builds the `SysML_Model` with `ConOps`, `SensorSubsystem`, and `CommsSubsystem` packages for `Issue #477`.
/// Realises: [REQ-0604]
fn build_issue_477_conops_and_subsystems_model() -> PackageDef {
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
    root
}

/// Asserts that `EPIC-02-ConOps.md` and `FEAT-01-MissionOperator.md` preserve custom authored prose and `- [x]` checkboxes.
/// Realises: [REQ-0604]
fn assert_issue_477_preserved_prose_and_checklists(
    generated: &BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

/// Verifies Issue #477: `ConOps` + `part.actions` + `p.interaction_defs` UC/US projection, `!opts.force` authored prose & `- [x]` checkbox preservation, and colliding prefix cleanup.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
#[test]
fn test_issue_477_conops_part_behavior_projection_and_non_destructive_sync(
) -> Result<(), Box<dyn std::error::Error>> {
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

    let stale_uc_path = write_issue_477_existing_feat_and_stale_uc_fixtures(&docs_dir)?;
    let root = build_issue_477_conops_and_subsystems_model();

    let opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema/model.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: None,
        dry_run: false,
        force: false,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;

    assert!(generated.contains_key("use-cases/UC-01-MissionPlanning.md"));
    assert!(generated.contains_key("user-stories/US-01-ExecuteMissionPlanAction.md"));
    assert!(generated.contains_key("user-stories/US-02-CalibrateBeamAction.md"));
    let uc02 = generated
        .get("use-cases/UC-02-CalibrateSensor.md")
        .ok_or("expected UC-02-CalibrateSensor.md")?;
    assert!(uc02.contains("US-02-CalibrateBeamAction.md"));
    assert!(generated.contains_key("user-stories/US-03-EstablishLinkInteraction.md"));

    assert_issue_477_preserved_prose_and_checklists(&generated)?;

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

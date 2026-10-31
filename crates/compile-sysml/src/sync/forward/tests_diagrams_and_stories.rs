//! Feature BDD derivation, Use Case prefix stripping, Mermaid diagram generation, User Story lifeline binding, and root package projection unit tests.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
//! Contract: Verifies positive/negative BDD obligation derivation from `PartDef` and `RequirementDef` constraints, iterative Use Case prefix stripping, backtick-free Mermaid `graph TD`/`stateDiagram-v2`/`classDiagram` rendering, User Story lifeline and dynamic Specification Source binding, and root package behavioral checklist linkage.
//! Bounds & Faults: Executes deterministic in-memory unit assertions returning `Result<(), Box<dyn std::error::Error>>` with zero panics or external side effects.

use super::*;
use deap_core::sysml_ast::PartDef;

/// Verifies `generate_feature_bdd` derives positive and negative BDD acceptance criteria from `PartDef` constraints and `RequirementDef` obligations within `3..=10` items.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_feature_bdd_derived_from_constraints_and_requirements(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{ConstraintDef, RequirementDef};

    let mut part = PartDef {
        name: "TelemetryProcessor".to_string(),
        ..Default::default()
    };
    part.constraints.push(ConstraintDef {
        name: "FrameRateLimit".to_string(),
        expression: "frame_rate <= 60".to_string(),
        ..Default::default()
    });

    let req = RequirementDef {
        name: "LatencyBoundRequirement".to_string(),
        req_id: "REQ-0100".to_string(),
        constraints: vec![ConstraintDef {
            name: "MaxLatencyInvariant".to_string(),
            expression: "latency_ms < 5".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let bdd = generate_feature_bdd(&part, &[&req]);

    assert!(
        !bdd.contains("operational_invariants"),
        "Must never fabricate 'operational_invariants', got:\n{}",
        bdd
    );
    assert!(
        bdd.contains(
            "evaluates invariant `FrameRateLimit`, Then the invariant holds and verified state is committed."
        ),
        "Must contain positive satisfaction criterion for FrameRateLimit, got:\n{}",
        bdd
    );
    assert!(
        bdd.contains(
            "violate `FrameRateLimit`, When `TelemetryProcessor` evaluates invariant `FrameRateLimit`, Then `TelemetryProcessor` rejects"
        ),
        "Must contain negative violation/rejection criterion for FrameRateLimit, got:\n{}",
        bdd
    );
    assert!(
        bdd.contains("MaxLatencyInvariant") && bdd.contains("REQ-0100"),
        "Must contain requirement constraint MaxLatencyInvariant and REQ-0100 trace, got:\n{}",
        bdd
    );
    assert!(
        bdd.contains(
            "When `TelemetryProcessor` executes structural interface processing, Then state and output ports conform to declared schema types."
        ),
        "Must contain structural interface conformance criterion, got:\n{}",
        bdd
    );

    let lines: Vec<&str> = bdd
        .lines()
        .filter(|l| l.trim().starts_with("- [ ] AC-"))
        .collect();
    assert!(
        (3..=10).contains(&lines.len()),
        "Criteria count must be between 3 and 10, got: {}",
        lines.len()
    );
    Ok(())
}

/// Verifies `strip_use_case_prefix` safely strips simple and composite prefixes (`UC_AC_01_`, `UC_01_`, `UC-02-`, `UC03_`) using `.get(..)` bounds-checked slicing.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_strip_use_case_prefix_variations() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        strip_use_case_prefix("UC_01_System_Vision_and_Foundational_Invariants"),
        "System_Vision_and_Foundational_Invariants"
    );
    assert_eq!(
        strip_use_case_prefix("UC-02-Universal_Schema_Ingestion_Workflow"),
        "Universal_Schema_Ingestion_Workflow"
    );
    assert_eq!(
        strip_use_case_prefix("UC03_Node_Arena_Memory_Management_Workflow"),
        "Node_Arena_Memory_Management_Workflow"
    );
    assert_eq!(
        strip_use_case_prefix("UC_AC_01_Rustc_E0453_Forbid_Direct_Override"),
        "Rustc_E0453_Forbid_Direct_Override"
    );
    assert_eq!(strip_use_case_prefix("UC_01_Login"), "Login");
    assert_eq!(
        strip_use_case_prefix("UC_Alpha_Workflow"),
        "UC_Alpha_Workflow"
    );
    Ok(())
}

/// Verifies `generate_use_case_diagram` strips inline backticks before truncation and emits `<<include>>`.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_generate_use_case_diagram_model_integrity() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = vec![
        "1. Actor `Operator` initiates task".to_string(),
        "2. `StateMachineSolverEngine` validates input schema structure and precondition guards for `State Machine Solver Workflow`.".to_string(),
    ];
    let diag = generate_use_case_diagram(
        "CIContinuousIntegrationRunner",
        "StateMachineSolverEngine",
        "State Machine Solver Workflow",
        &scenario,
    );

    assert!(!diag.contains('`'));
    assert!(diag.contains("graph TD"));
    assert!(diag.contains("subgraph \"StateMachineSolverEngine System Boundary\""));
    assert!(diag.contains("UC_Main([\"State Machine Solver Workflow\"])"));
    assert!(diag.contains("UC_Step01([\"Actor Operator initiates task\"])"));
    assert!(diag
        .contains("UC_Step02([\"StateMachineSolverEngine validates input schema structure...\"])"));
    assert!(diag.contains("end"));
    assert!(diag.contains(
        "CIContinuousIntegrationRunner((\"CIContinuousIntegrationRunner\")) --- UC_Main"
    ));
    assert!(diag.contains("UC_Main -. \"<<include>>\" .-> UC_Step01"));
    Ok(())
}

/// Verifies `generate_epic_statechart` and `generate_epic_class_diagram` without emitting phantom package classes.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_epic_statechart_and_class_diagram_generation() -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{AttributeDef, PortDef, StateDef, TransitionDef};

    let mut pkg = PackageDef {
        name: "SpatioTemporalStateSolvers".to_string(),
        parent_package: Some("SystemModel".to_string()),
        ..Default::default()
    };
    let mut st_def = StateDef {
        name: "StateSolverExecutionStatechart".to_string(),
        ..Default::default()
    };
    st_def.transitions.push(TransitionDef {
        name: Some("InitialToIngest first entry then IngestingStates".to_string()),
        ..Default::default()
    });
    st_def.transitions.push(TransitionDef {
        name: Some("IngestToSolve first IngestingStates then Solved".to_string()),
        ..Default::default()
    });
    pkg.state_defs.push(st_def);

    let mut engine = PartDef {
        name: "StateMachineSolverEngine".to_string(),
        ..Default::default()
    };
    engine.ports.push(PortDef {
        name: "stream_in".to_string(),
        direction: "in".to_string(),
        type_name: "ASTStreamPort".to_string(),
        ..Default::default()
    });
    engine.attributes.push(AttributeDef {
        name: "recursion_depth".to_string(),
        type_name: "Integer".to_string(),
        ..Default::default()
    });
    pkg.part_defs.push(engine);

    let helper = PartDef {
        name: "StateHierarchyFlattener".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(helper);

    let statechart = generate_epic_statechart(&pkg);
    assert!(statechart.contains("stateDiagram-v2"));
    assert!(statechart.contains("[*] --> IngestingStates"));
    assert!(statechart.contains("IngestingStates --> Solved"));
    assert!(statechart.contains("Solved --> [*]"));

    let class_diag = generate_epic_class_diagram(&pkg, "StateMachineSolverEngine", false);
    assert!(class_diag.contains("classDiagram"));
    assert!(class_diag.contains("class StateMachineSolverEngine {"));
    assert!(class_diag.contains("+ASTStreamPort stream_in (in)"));
    assert!(class_diag.contains("+Integer recursion_depth"));
    assert!(
        !class_diag.contains("SystemModel"),
        "Undeclared parent_package must not be emitted as phantom class"
    );
    assert!(
        class_diag.contains("StateMachineSolverEngine --> StateHierarchyFlattener : coordinates")
    );
    Ok(())
}

/// Verifies User Story subsystem Parent Epic linkage, declared `PartDef` lifeline binding, and dynamic `Specification Source` selection.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_user_story_subsystem_parent_epic_and_dynamic_spec_source(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{ActionDef, InteractionDef};

    let mut root = PackageDef {
        name: "DEAP_Compiler_System".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "SpatioTemporalStateSolvers".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "GrammarLoweringEngine".to_string(),
        ..Default::default()
    });
    sub.part_defs.push(PartDef {
        name: "StateMachineSolverEngine".to_string(),
        ..Default::default()
    });
    sub.action_defs.push(ActionDef {
        name: "SolveStateDynamicsAction".to_string(),
        ..Default::default()
    });
    sub.interaction_defs.push(InteractionDef {
        name: "StateSolverInteraction".to_string(),
        lifelines: vec![
            "grammar : GrammarLoweringEngine".to_string(),
            "solver : StateMachineSolverEngine".to_string(),
        ],
        messages: vec![
            "SubmitModel from grammar to solver".to_string(),
            "StateOutput from solver to grammar".to_string(),
        ],
        ..Default::default()
    });
    root.packages.push(sub);

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;
    let us = generated
        .iter()
        .find(|(k, _)| k.contains("US-") && k.contains("SolveStateDynamicsAction"))
        .map(|(_, v)| v)
        .ok_or("expected value is absent")?;

    assert!(
        us.contains("[schema/model.sysml](../../schema/model.sysml)"),
        "Specification Source must bind to schema/model.sysml when modular schema dirs are absent"
    );
    assert!(
        us.contains(
            "[EPIC-02-SpatioTemporalStateSolvers](../epics/EPIC-02-SpatioTemporalStateSolvers.md)"
        ),
        "Parent Epic must bind to EPIC-02-SpatioTemporalStateSolvers instead of EPIC-02-ConOps"
    );
    assert!(
        us.contains("GrammarLoweringEngine->>StateMachineSolverEngine: SolveStateDynamicsAction()"),
        "Sequence diagram must use interaction lifelines"
    );
    assert!(
        us.contains("StateMachineSolverEngine-->>GrammarLoweringEngine: StateOutput"),
        "Sequence diagram must return reply message from interaction def"
    );
    assert!(
        us.contains("`StateSolverInteraction`"),
        "SysML Interaction must match interaction name"
    );
    Ok(())
}

/// Verifies Issue `#442` zero compiler-domain contamination across generated downstream specs.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_issue_442_forward_sync_zero_compiler_contamination_in_specs(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::ActionDef;

    let mut root = PackageDef {
        name: "Satellite_System".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "Attitude_Control".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "AttitudeController".to_string(),
        ..Default::default()
    });
    sub.action_defs.push(ActionDef {
        name: "OrientSolarArraysAction".to_string(),
        ..Default::default()
    });
    root.packages.push(sub);

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;
    for (path, content) in &generated {
        assert!(
            !content.contains("DEAPCompilerSystem"),
            "File '{}' contains contaminated class DEAPCompilerSystem",
            path
        );
        assert!(
            !content.contains("Compiler Primacy"),
            "File '{}' contains contaminated 'Compiler Primacy' title",
            path
        );
        assert!(
            !content.contains("actor HumanEngineer"),
            "File '{}' contains contaminated 'actor HumanEngineer'",
            path
        );
        assert!(
            !content.contains("FEAT-01-HumanEngineer.md"),
            "File '{}' contains contaminated 'FEAT-01-HumanEngineer.md'",
            path
        );
        assert!(
            !content.contains("initialize_compiler"),
            "File '{}' contains contaminated statechart transition 'initialize_compiler'",
            path
        );
        assert!(
            !content.contains("IngestOEMArtifacts"),
            "File '{}' contains contaminated statechart 'IngestOEMArtifacts'",
            path
        );
    }
    Ok(())
}

/// Verifies Issue `#447` root-package specification emission, behavioral action stories, and linked Epic checklists.
///
/// Realises: [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_issue_447_forward_sync_root_package_specs_actions_and_linked_checklists(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{ActionDef, PartDef, UseCaseDef};

    let mut root = PackageDef {
        name: "SysML_Model".to_string(),
        ..Default::default()
    };
    root.part_defs.push(PartDef {
        name: "TrackingRadar".to_string(),
        ..Default::default()
    });
    root.part_defs.push(PartDef {
        name: "MitigationEffector".to_string(),
        ..Default::default()
    });
    root.action_defs.push(ActionDef {
        name: "DetectTrackAction".to_string(),
        ..Default::default()
    });
    root.action_defs.push(ActionDef {
        name: "EngageThreatAction".to_string(),
        ..Default::default()
    });
    root.use_case_defs.push(UseCaseDef {
        name: "DetectTargetUseCase".to_string(),
        ..Default::default()
    });

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/cuas_coa.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;

    let has_us01 = generated
        .keys()
        .any(|k| k.contains("US-01") && k.contains("DetectTrackAction"));
    let has_us02 = generated
        .keys()
        .any(|k| k.contains("US-02") && k.contains("EngageThreatAction"));
    assert!(
        has_us01,
        "User Story 01 for root package action must be generated"
    );
    assert!(
        has_us02,
        "User Story 02 for root package second action must be generated without truncation"
    );

    let has_uc01 = generated
        .keys()
        .any(|k| k.contains("UC-01") && k.contains("DetectTargetUseCase"));
    assert!(
        has_uc01,
        "Use Case 01 for root package use case def must be generated"
    );

    let epic_content = generated
        .iter()
        .find(|(k, _)| k.contains("EPIC-01") && k.contains("SysML_Model"))
        .map(|(_, v)| v)
        .ok_or("EPIC-01-SysML_Model.md must be generated")?;

    assert!(
        !epic_content
            .contains("Feature 01: Subsystem capability implementation for System Architecture."),
        "Epic must not emit degenerate unlinked placeholder feature checklist"
    );
    assert!(
        epic_content.contains(
            "[Feature 01: [System Architecture] TrackingRadar](../features/FEAT-01-TrackingRadar.md)"
        ),
        "Epic feature checklist must contain linked Feature 01"
    );
    assert!(
        epic_content.contains(
            "[Feature 02: [System Architecture] MitigationEffector](../features/FEAT-02-MitigationEffector.md)"
        ),
        "Epic feature checklist must contain linked Feature 02"
    );
    assert!(
        epic_content.contains(
            "[User Story 01: Detect Track Action](../user-stories/US-01-DetectTrackAction.md)"
        ),
        "Epic must associate generated User Story 01"
    );
    assert!(
        epic_content.contains(
            "[Use Case 01: Detect Target Use Case](../use-cases/UC-01-DetectTargetUseCase.md)"
        ),
        "Epic must associate generated Use Case 01"
    );

    Ok(())
}

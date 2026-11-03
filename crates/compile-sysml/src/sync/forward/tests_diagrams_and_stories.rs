#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Feature BDD Derivation, Mermaid Diagrams, and User Story Lifeline Unit Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
//! Verifies positive/negative BDD obligation derivation from `PartDef` and `RequirementDef`
//! constraints, iterative Use Case prefix stripping, backtick-free Mermaid `graph TD`,
//! `stateDiagram-v2`, and `classDiagram` rendering, and User Story lifeline binding.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: In-memory AST fixtures are initialized.
//! - **Postconditions**: Generated BDD criteria, diagrams, and User Stories satisfy schema contracts.
//! - **Invariants**: Feature BDD criteria count is bounded within `3..=10`.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Test assertions return `Result<(), Box<dyn std::error::Error>>` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure in-memory unit test execution.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Exercises `generate_feature_bdd`, `strip_use_case_prefix`, `generate_use_case_diagram`, `generate_epic_statechart`, `generate_epic_class_diagram`, and `forward_sync_sysml_to_specs`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]

use super::*;
use deap_core::sysml_cst::PartDef;

/// Verifies `generate_feature_bdd` derives positive and negative BDD acceptance criteria from `PartDef` constraints and `RequirementDef` obligations within `3..=10` items.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_feature_bdd_derived_from_constraints_and_requirements(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_cst::{ConstraintDef, RequirementDef};

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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_epic_statechart_and_class_diagram_generation() -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_cst::{AttributeDef, PortDef, StateDef, TransitionDef};

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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_user_story_subsystem_parent_epic_and_dynamic_spec_source(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_cst::{ActionDef, InteractionDef};

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

    assert!(us.contains("[schema/model.sysml](../../schema/model.sysml)"));
    assert!(us.contains(
        "[EPIC-02-SpatioTemporalStateSolvers](../epics/EPIC-02-SpatioTemporalStateSolvers.md)"
    ));
    assert!(us.contains("GrammarLoweringEngine->>StateMachineSolverEngine: SolveStateDynamicsAction()"));
    assert!(us.contains("StateMachineSolverEngine-->>GrammarLoweringEngine: StateOutput"));
    assert!(us.contains("`StateSolverInteraction`"));
    Ok(())
}

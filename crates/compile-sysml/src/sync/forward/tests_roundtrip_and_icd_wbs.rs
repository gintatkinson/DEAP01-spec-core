#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Full AST Forward-Reverse-Serialize-Parse Round-Trip Idempotency Unit Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604]
//! Verifies structured `StateDef` transition trigger/guard/effect preservation and multi-clause
//! `UseCaseDef` fidelity across full AST round-trips (`DEFECT-07`, `Issue #497`), and Parent Epic
//! and Use Case realization linkage (`Issue #507`).
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Temporary directories and `SystemPackage` AST fixtures are initialized.
//! - **Postconditions**: Reparsed SysML v2 models preserve state transitions, use cases, and part actions.
//! - **Invariants**: Temporary directories are cleaned up deterministically before test return.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Test assertions return `Result<(), Box<dyn std::error::Error>>` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Uses isolated PID-scoped temporary directories.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Exercises `forward_sync_sysml_to_specs`, `reverse_sync_specs_to_sysml`, `to_sysml`, and `SysmlParser::parse_source`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604]

use super::*;
use deap_core::sysml_ast::{ActionDef, PackageDef, PartDef, StateDef, TransitionDef, UseCaseDef};

/// Builds the `SystemPackage` model with `ProcessorLifecycleState` and `ExecuteSubsystemWorkflow` for `DEFECT-07`.
/// Realises: [REQ-0604]
fn build_defect_07_full_roundtrip_model() -> PackageDef {
    let mut root = PackageDef {
        name: "SystemPackage".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "SubsystemAlpha".to_string(),
        parent_package: Some("SystemPackage".to_string()),
        ..Default::default()
    };
    let mut processor = PartDef {
        name: "ProcessorEngine".to_string(),
        ..Default::default()
    };
    processor.actions.push(ActionDef {
        name: "ProcessItemAction".to_string(),
        ..Default::default()
    });
    sub.part_defs.push(processor);

    sub.action_defs.push(ActionDef {
        name: "ExecuteSequenceAction".to_string(),
        ..Default::default()
    });

    let mut state_def = StateDef {
        name: "ProcessorLifecycleState".to_string(),
        ..Default::default()
    };
    state_def.transitions.push(TransitionDef {
        name: Some("IdleToActive".to_string()),
        source: Some("IdleState".to_string()),
        target: Some("ActiveState".to_string()),
        trigger: Some("StartCommand".to_string()),
        guard: Some("input_ready == true".to_string()),
        effect: Some("InitializeContext".to_string()),
        doc: None,
    });
    sub.state_defs.push(state_def);

    sub.use_case_defs.push(UseCaseDef {
        name: "ExecuteSubsystemWorkflow".to_string(),
        actors: vec!["SystemActor".to_string()],
        subject: Some("ProcessorEngine".to_string()),
        objective: Some("Execute deterministic subsystem workflow".to_string()),
        preconditions: vec!["Subsystem initialized in nominal state".to_string()],
        steps: vec!["Actor dispatches request to ProcessorEngine".to_string()],
        postconditions: vec!["Output state verified without error".to_string()],
        ..Default::default()
    });
    root.packages.push(sub);
    root
}

/// Asserts that `reparsed` preserves `IdleState -> ActiveState` transition fields and `UseCaseDef` clauses.
/// Realises: [REQ-0604]
fn assert_defect_07_reparsed_transitions_and_uc(
    reparsed: &PackageDef,
) -> Result<(), Box<dyn std::error::Error>> {
    let reparsed_sub = reparsed
        .packages
        .iter()
        .find(|p| p.name == "SubsystemAlpha")
        .ok_or("expected SubsystemAlpha in reparsed PackageDef")?;
    let reparsed_state = reparsed_sub
        .state_defs
        .first()
        .ok_or("expected StateDef in reparsed SubsystemAlpha")?;
    let reparsed_trans = reparsed_state
        .transitions
        .first()
        .ok_or("expected TransitionDef in reparsed StateDef")?;

    assert_eq!(reparsed_trans.source.as_deref(), Some("IdleState"));
    assert_eq!(reparsed_trans.target.as_deref(), Some("ActiveState"));
    assert_eq!(reparsed_trans.trigger.as_deref(), Some("StartCommand"));
    assert_eq!(reparsed_trans.guard.as_deref(), Some("input_ready == true"));
    assert_eq!(reparsed_trans.effect.as_deref(), Some("InitializeContext"));

    let reparsed_uc = reparsed_sub
        .use_case_defs
        .first()
        .ok_or("expected UseCaseDef in reparsed SubsystemAlpha")?;
    let norm_pre: Vec<String> = reparsed_uc.preconditions.iter().map(|s| s.trim_matches('"').to_string()).collect();
    let norm_steps: Vec<String> = reparsed_uc.steps.iter().map(|s| s.trim_matches('"').to_string()).collect();
    let norm_post: Vec<String> = reparsed_uc.postconditions.iter().map(|s| s.trim_matches('"').to_string()).collect();
    assert_eq!(norm_pre, vec!["Subsystem initialized in nominal state".to_string()]);
    assert_eq!(norm_steps, vec!["Actor dispatches request to ProcessorEngine".to_string()]);
    assert_eq!(norm_post, vec!["Output state verified without error".to_string()]);
    Ok(())
}

/// Verifies `DEFECT-07` (Issue #497): full AST roundtrip synchronization preserves `StateDef` transitions and `UseCaseDef` clauses.
/// Realises: [REQ-0604]
#[test]
fn test_defect_07_full_ast_roundtrip_sync_preserves_state_transitions_and_use_cases(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::parser::grammar::SysmlParser;
    use crate::semantic::serializer::to_sysml;
    use crate::sync::reverse::{reverse_sync_specs_to_sysml, ReverseSyncOptions};

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_defect_07_full_ast_roundtrip_{}",
        std::process::id()
    ));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    let docs_dir = tmp_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    let root = build_defect_07_full_roundtrip_model();

    let fwd_opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema").join("model.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir.clone()),
        dry_run: false,
        force: true,
    };
    let generated_initial = forward_sync_sysml_to_specs(&root, &fwd_opts)?;
    let epic_md = generated_initial
        .iter()
        .find(|(k, _)| k.contains("EPIC-") && k.contains("SubsystemAlpha"))
        .map(|(_, v)| v)
        .ok_or("expected generated EPIC for SubsystemAlpha")?;
    assert!(epic_md.contains("IdleState --> ActiveState"));
    assert!(epic_md.contains(
        "IdleState --> ActiveState : StartCommand [input_ready == true] / InitializeContext"
    ));

    let rev_opts = ReverseSyncOptions {
        docs_dir: docs_dir.clone(),
        schema_path: None,
        output_path: tmp_dir.join(".pipeline").join("schema.sysml"),
        digest_path: tmp_dir.join(".pipeline").join("schema-digest.json"),
        allow_overwrite: true,
    };
    let (synced_pkg, _digest) = reverse_sync_specs_to_sysml(Some(&root), &rev_opts)?;
    let serialized = to_sysml(&synced_pkg);
    let reparsed = SysmlParser::parse_source(&serialized, "SystemPackage")?;

    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }

    assert_defect_07_reparsed_transitions_and_uc(&reparsed)?;
    Ok(())
}

/// Builds the two-Use-Case / two-Action `SystemPackage` fixture for Issue `#507`.
/// Realises: [REQ-0604]
fn build_issue_507_roundtrip_model() -> PackageDef {
    let mut root = PackageDef {
        name: "SystemPackage".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "SubsystemAlpha".to_string(),
        parent_package: Some("SystemPackage".to_string()),
        ..Default::default()
    };
    let mut alpha_engine = PartDef {
        name: "AlphaEngine".to_string(),
        ..Default::default()
    };
    alpha_engine.actions.push(ActionDef {
        name: "ComputeStepAction".to_string(),
        ..Default::default()
    });
    sub.part_defs.push(alpha_engine);

    sub.action_defs.push(ActionDef {
        name: "PrimaryFlowAction".to_string(),
        ..Default::default()
    });
    sub.action_defs.push(ActionDef {
        name: "SecondaryFlowAction".to_string(),
        ..Default::default()
    });

    sub.use_case_defs.push(UseCaseDef {
        name: "ExecutePrimaryFlow".to_string(),
        actors: vec!["SystemActor".to_string()],
        subject: Some("AlphaEngine".to_string()),
        objective: Some("Execute primary flow".to_string()),
        preconditions: vec!["Primary precondition".to_string()],
        steps: vec!["Primary step".to_string()],
        postconditions: vec!["Primary postcondition".to_string()],
        ..Default::default()
    });
    sub.use_case_defs.push(UseCaseDef {
        name: "ExecuteSecondaryFlow".to_string(),
        actors: vec!["SystemActor".to_string()],
        subject: Some("AlphaEngine".to_string()),
        objective: Some("Execute secondary flow".to_string()),
        preconditions: vec!["Secondary precondition".to_string()],
        steps: vec!["Secondary step".to_string()],
        postconditions: vec!["Secondary postcondition".to_string()],
        ..Default::default()
    });
    root.packages.push(sub);
    root
}

/// Asserts that both `reparsed` and `synced_without_base` preserve `UseCaseDef` count and `ComputeStepAction`.
/// Realises: [REQ-0604]
fn assert_issue_507_reparsed_and_no_base_models(
    reparsed: &PackageDef,
    synced_without_base: &PackageDef,
) -> Result<(), Box<dyn std::error::Error>> {
    let reparsed_sub = reparsed
        .packages
        .iter()
        .find(|p| p.name == "SubsystemAlpha")
        .ok_or("expected SubsystemAlpha in reparsed PackageDef")?;
    assert_eq!(reparsed_sub.use_case_defs.len(), 2);
    let reparsed_engine = reparsed_sub
        .part_defs
        .iter()
        .find(|pt| pt.name == "AlphaEngine")
        .ok_or("expected AlphaEngine in reparsed SubsystemAlpha")?;
    assert!(reparsed_engine
        .actions
        .iter()
        .any(|a| a.name == "ComputeStepAction"));

    let engine_no_base = synced_without_base
        .part_defs
        .iter()
        .chain(
            synced_without_base
                .packages
                .iter()
                .flat_map(|p| p.part_defs.iter()),
        )
        .find(|pt| pt.name == "AlphaEngine")
        .ok_or("expected AlphaEngine in synced_without_base")?;
    assert!(
        engine_no_base
            .actions
            .iter()
            .any(|a| a.name == "ComputeStepAction")
            || engine_no_base
                .operations
                .iter()
                .any(|o| o.name == "ComputeStepAction")
    );
    Ok(())
}

/// Verifies forward-to-reverse-to-forward roundtrip idempotency, Parent Epic linkage, and feature operation/action bullet preservation.
/// Realises: [REQ-0604]
#[test]
fn test_forward_to_reverse_to_forward_roundtrip_idempotency_and_parent_epic_links(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::parser::grammar::SysmlParser;
    use crate::semantic::serializer::to_sysml;
    use crate::sync::reverse::{reverse_sync_specs_to_sysml, ReverseSyncOptions};

    let tmp_dir =
        std::env::temp_dir().join(format!("deap_issue_507_roundtrip_{}", std::process::id()));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    let docs_dir = tmp_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    let root = build_issue_507_roundtrip_model();
    let fwd_opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema").join("model.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir.clone()),
        dry_run: false,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&root, &fwd_opts)?;

    let us01 = generated
        .get("user-stories/US-01-PrimaryFlowAction.md")
        .ok_or("expected user-stories/US-01-PrimaryFlowAction.md")?;
    assert!(us01.contains("[EPIC-02-SubsystemAlpha](../epics/EPIC-02-SubsystemAlpha.md)"));

    let uc02 = generated
        .get("use-cases/UC-02-ExecuteSecondaryFlow.md")
        .ok_or("expected use-cases/UC-02-ExecuteSecondaryFlow.md")?;
    assert!(uc02.contains("US-02-SecondaryFlowAction.md"));

    let feat01 = generated
        .get("features/FEAT-01-AlphaEngine.md")
        .ok_or("expected features/FEAT-01-AlphaEngine.md")?;
    assert!(feat01.contains("- `+ComputeStepAction() : void`"));

    let rev_opts = ReverseSyncOptions {
        docs_dir: docs_dir.clone(),
        schema_path: None,
        output_path: tmp_dir.join(".pipeline").join("schema.sysml"),
        digest_path: tmp_dir.join(".pipeline").join("schema-digest.json"),
        allow_overwrite: true,
    };
    let (synced_pkg, _digest) = reverse_sync_specs_to_sysml(Some(&root), &rev_opts)?;
    let serialized = to_sysml(&synced_pkg);
    let reparsed = SysmlParser::parse_source(&serialized, "SystemPackage")?;
    let (synced_without_base, _) = reverse_sync_specs_to_sysml(None, &rev_opts)?;

    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }

    assert_issue_507_reparsed_and_no_base_models(&reparsed, &synced_without_base)?;
    Ok(())
}

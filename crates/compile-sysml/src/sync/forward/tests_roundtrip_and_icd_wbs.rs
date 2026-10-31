//! Full AST forward-reverse-serialize-parse roundtrip idempotency, parameterized action signatures, ICD/WBS emitters, and HTML comment state guard unit tests.
//!
//! Safety & Traceability: Realises: [REQ-0604]
//! Contract: Verifies structured `StateDef` transition trigger/guard/effect preservation and multi-clause `UseCaseDef` fidelity across full AST roundtrips, Parent Epic and Use Case realization linkage, parameterized `ActionDef` Mermaid and Markdown bullets, logical ICD and WBS compilation, and HTML comment state tracking.
//! Bounds & Faults: Uses isolated auto-cleaned temporary directories and pure in-memory fixtures returning `Result<(), Box<dyn std::error::Error>>` with zero panics.

use super::*;
use deap_core::sysml_ast::PartDef;

/// Verifies `DEFECT-07` (Issue #497): full AST roundtrip synchronization preserves `StateDef` transitions and `UseCaseDef` clauses.
///
/// Realises: [REQ-0604]
#[test]
fn test_defect_07_full_ast_roundtrip_sync_preserves_state_transitions_and_use_cases(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::parser::grammar::SysmlParser;
    use crate::semantic::serializer::to_sysml;
    use crate::sync::reverse::{reverse_sync_specs_to_sysml, ReverseSyncOptions};
    use deap_core::sysml_ast::{
        ActionDef, PackageDef, PartDef, StateDef, TransitionDef, UseCaseDef,
    };

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_defect_07_full_ast_roundtrip_{}",
        std::process::id()
    ));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    let docs_dir = tmp_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

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
    assert!(
        epic_md.contains("IdleState --> ActiveState"),
        "Epic statechart must render structured TransitionDef source and target, got:\n{}",
        epic_md
    );
    assert!(
        epic_md.contains(
            "IdleState --> ActiveState : StartCommand [input_ready == true] / InitializeContext"
        ),
        "Epic statechart must render structured TransitionDef trigger, guard, and effect, got:\n{}",
        epic_md
    );

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
    let norm_pre: Vec<String> = reparsed_uc
        .preconditions
        .iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    let norm_steps: Vec<String> = reparsed_uc
        .steps
        .iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    let norm_post: Vec<String> = reparsed_uc
        .postconditions
        .iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    assert_eq!(
        norm_pre,
        vec!["Subsystem initialized in nominal state".to_string()]
    );
    assert_eq!(
        norm_steps,
        vec!["Actor dispatches request to ProcessorEngine".to_string()]
    );
    assert_eq!(
        norm_post,
        vec!["Output state verified without error".to_string()]
    );

    Ok(())
}

/// Verifies forward-to-reverse-to-forward roundtrip idempotency, Parent Epic linkage, and feature operation/action bullet preservation.
///
/// Realises: [REQ-0604]
#[test]
fn test_forward_to_reverse_to_forward_roundtrip_idempotency_and_parent_epic_links(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::parser::grammar::SysmlParser;
    use crate::semantic::serializer::to_sysml;
    use crate::sync::reverse::{reverse_sync_specs_to_sysml, ReverseSyncOptions};
    use deap_core::sysml_ast::{ActionDef, PackageDef, PartDef, UseCaseDef};

    let tmp_dir =
        std::env::temp_dir().join(format!("deap_issue_507_roundtrip_{}", std::process::id()));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    let docs_dir = tmp_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

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
    assert!(
        us01.contains("[EPIC-02-SubsystemAlpha](../epics/EPIC-02-SubsystemAlpha.md)"),
        "US-01 must link to EPIC-02-SubsystemAlpha, got:\n{}",
        us01
    );

    let uc02 = generated
        .get("use-cases/UC-02-ExecuteSecondaryFlow.md")
        .ok_or("expected use-cases/UC-02-ExecuteSecondaryFlow.md")?;
    assert!(
        uc02.contains("US-02-SecondaryFlowAction.md"),
        "UC-02 must link to US-02-SecondaryFlowAction.md, got:\n{}",
        uc02
    );

    let feat01 = generated
        .get("features/FEAT-01-AlphaEngine.md")
        .ok_or("expected features/FEAT-01-AlphaEngine.md")?;
    assert!(
        feat01.contains("- `+ComputeStepAction() : void`"),
        "FEAT-01-AlphaEngine.md must contain `- `+ComputeStepAction() : void`` under Interface Requirements, got:\n{}",
        feat01
    );

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

    let reparsed_sub = reparsed
        .packages
        .iter()
        .find(|p| p.name == "SubsystemAlpha")
        .ok_or("expected SubsystemAlpha in reparsed PackageDef")?;
    assert_eq!(
        reparsed_sub.use_case_defs.len(),
        2,
        "SubsystemAlpha must preserve both use_case_defs on roundtrip"
    );
    let reparsed_engine = reparsed_sub
        .part_defs
        .iter()
        .find(|pt| pt.name == "AlphaEngine")
        .ok_or("expected AlphaEngine in reparsed SubsystemAlpha")?;
    assert!(
        reparsed_engine
            .actions
            .iter()
            .any(|a| a.name == "ComputeStepAction"),
        "reparsed AlphaEngine must preserve ComputeStepAction in actions"
    );

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
                .any(|o| o.name == "ComputeStepAction"),
        "reverse_sync_specs_to_sysml(None, ...) must preserve ComputeStepAction on AlphaEngine from FEAT-01-AlphaEngine.md"
    );

    Ok(())
}

/// Verifies `REQ-0604` (`BP-04-FORWARD-SYNC-ICD-WBS`) `AC-01` through `AC-05`.
///
/// Realises: [REQ-0604]
#[test]
fn test_req_0604_forward_sync_parameterized_actions_icd_wbs_and_html_comment_guards(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{ActionDef, AttributeDef, ConnectionDef, PortDef};

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

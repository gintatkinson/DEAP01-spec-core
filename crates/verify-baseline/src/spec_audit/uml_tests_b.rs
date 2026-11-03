//! Unit tests for `spec_audit::uml_validator` (compliance, alignment, and coverage gates).
//! Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR]

use std::collections::BTreeMap;

use deap_core::sysml_cst::{ActionDef, OperationDef, PartDef, StateDef, SysmlModel};

use crate::spec_audit::diagram_ast::{StateDefAst, StateDiagramAst, StateTransitionAst};
use crate::spec_audit::markdown_ast::{MarkdownDoc, MermaidBlock, ProseSegment};
use crate::spec_audit::uml_validator::{
    validate_epic_statechart_heterogeneity, validate_feature_operation_coverage,
    validate_parent_epic_alignment, validate_state_diagram, validate_use_case_diagram_compliance,
    validate_user_story_test_cases, SysmlSymbolIndex,
};

#[test]
fn test_compliant_use_case_diagram_passes() {
    let compliant_raw = r#"graph TD
    subgraph "SystemVisionEngine System Boundary"
        UC_Main(["System Vision and Foundational Invariants"])
        UC_Step01(["Operator dispatches command"])
        UC_Step02(["Engine boots invariants"])
    end
    HumanEngineer(("HumanEngineer")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
"#;
    let compliant_doc = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([("primary_actor".to_string(), "HumanEngineer".to_string())]),
        prose: vec![],
        mermaid_blocks: vec![MermaidBlock {
            raw: compliant_raw.to_string(),
            header: "graph TD".to_string(),
            start_line: 10,
            end_line: 18,
        }],
        unresolved_placeholders: vec![],
    };

    let clean_errs = validate_use_case_diagram_compliance(&compliant_doc, "docs/use-cases/uc-01.md");
    assert!(clean_errs.is_empty(), "Compliant diagram should have 0 errors: {:?}", clean_errs);
}

#[test]
fn test_epic_statechart_heterogeneity_gate() {
    let canned_raw = r#"stateDiagram-v2
    [*] --> Bootstrapping
    Bootstrapping --> Ingesting : dispatch
    Ingesting --> Compiling : parse_complete
    Compiling --> Verifying : ast_lowered
    Verifying --> EmitSuccess : pass
    EmitSuccess --> [*]
"#;
    let epic1 = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::new(),
        prose: vec![],
        mermaid_blocks: vec![MermaidBlock {
            raw: canned_raw.to_string(),
            header: "stateDiagram-v2".to_string(),
            start_line: 1,
            end_line: 8,
        }],
        unresolved_placeholders: vec![],
    };
    let epic2 = epic1.clone();
    let epics = vec![("docs/epics/epic-03.md", &epic1), ("docs/epics/epic-04.md", &epic2)];
    let errs = validate_epic_statechart_heterogeneity(&epics);
    assert!(!errs.is_empty(), "Identical canned statecharts must be flagged");
    assert!(errs
        .iter()
        .any(|e| e.contains("share identical canned pipeline lifecycle statecharts")));

    let custom_raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> IngestionActive : start
    IngestionActive --> Finalized : done
    Finalized --> [*]
"#;
    let epic3 = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::new(),
        prose: vec![],
        mermaid_blocks: vec![MermaidBlock {
            raw: custom_raw.to_string(),
            header: "stateDiagram-v2".to_string(),
            start_line: 1,
            end_line: 6,
        }],
        unresolved_placeholders: vec![],
    };
    let heterogenous = vec![("docs/epics/epic-03.md", &epic1), ("docs/epics/epic-04.md", &epic3)];
    let clean_errs = validate_epic_statechart_heterogeneity(&heterogenous);
    assert!(clean_errs.is_empty(), "Heterogeneous epics must pass: {:?}", clean_errs);
}

#[test]
fn test_parent_epic_alignment_gate() {
    let feat06 = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([
            ("title".to_string(), "Feature 06: [System Vision] SystemVisionEngine".to_string()),
            ("parent_epic".to_string(), "[EPIC-03-SystemVision](../epics/EPIC-03-SystemVision.md)".to_string()),
        ]),
        prose: vec![],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };
    let epic03 = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::new(),
        prose: vec![ProseSegment {
            text: "- [ ] [User Story 01: Bootstrap System Vision Action](../user-stories/US-01-BootstrapSystemVisionAction.md)".to_string(),
            line: 20,
        }],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };
    let mismatched_story = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([
            ("title".to_string(), "User Story 01: Bootstrap System Vision Action".to_string()),
            ("parent_epic".to_string(), "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)".to_string()),
            ("specification_source".to_string(), "[schema/subsystems/system_vision/behavior.sysml]".to_string()),
        ]),
        prose: vec![ProseSegment {
            text: "## Required Features\n- [ ] [Feature 06: [System Vision] SystemVisionEngine](../features/FEAT-06-SystemVisionEngine.md)".to_string(),
            line: 30,
        }],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };

    let stories = vec![("docs/user-stories/US-01-BootstrapSystemVisionAction.md", &mismatched_story)];
    let features = vec![("docs/features/FEAT-06-SystemVisionEngine.md", &feat06)];
    let epics = vec![("docs/epics/EPIC-03-SystemVision.md", &epic03)];

    let errs = validate_parent_epic_alignment(&stories, &features, &epics);
    assert_eq!(errs.len(), 1);
    assert!(errs.first().is_some_and(|e| e.contains("Parent Epic Alignment Gate violation")));
    assert!(errs.first().is_some_and(|e| e.contains("declared parent Epic 'EPIC-02-ConOps'")));
    assert!(errs.first().is_some_and(|e| e.contains("EPIC-03-SystemVision")));

    let aligned_story = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([
            ("title".to_string(), "User Story 01: Bootstrap System Vision Action".to_string()),
            ("parent_epic".to_string(), "[EPIC-03-SystemVision](../epics/EPIC-03-SystemVision.md)".to_string()),
            ("specification_source".to_string(), "[schema/subsystems/system_vision/behavior.sysml]".to_string()),
        ]),
        prose: vec![ProseSegment {
            text: "## Required Features\n- [ ] [Feature 06: [System Vision] SystemVisionEngine](../features/FEAT-06-SystemVisionEngine.md)".to_string(),
            line: 30,
        }],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };
    let clean_stories =
        vec![("docs/user-stories/US-01-BootstrapSystemVisionAction.md", &aligned_story)];
    let clean_errs = validate_parent_epic_alignment(&clean_stories, &features, &epics);
    assert!(clean_errs.is_empty(), "Aligned story should have 0 errors: {:?}", clean_errs);
}

#[test]
fn test_feature_operation_coverage_scoping_ignores_prose_mentions() {
    let mut model = SysmlModel::default();
    let mut platform = PartDef { name: "MockPlatform".to_string(), ..Default::default() };
    platform.actions.push(ActionDef { name: "ExecuteAction".to_string(), ..Default::default() });
    platform
        .operations
        .push(OperationDef { name: "VerifyInterface".to_string(), ..Default::default() });

    let mut payload = PartDef { name: "MockPayload".to_string(), ..Default::default() };
    payload.actions.push(ActionDef { name: "ProcessData".to_string(), ..Default::default() });

    model.parts.push(platform);
    model.parts.push(payload);
    let index = SysmlSymbolIndex::from_model(&model);

    let feat102_content = "| **Schema Containers** | `MockSystem::MockPayload` |\n# Feature 102\nThis module is nested under part def MockPlatform.\nProcessData\n";
    let feat102_doc = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([(
            "schema_containers".to_string(),
            "MockSystem::MockPayload".to_string(),
        )]),
        prose: vec![ProseSegment {
            text: "This module is nested under part def MockPlatform.".to_string(),
            line: 6,
        }],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };
    let findings = validate_feature_operation_coverage(
        &feat102_doc,
        feat102_content,
        "docs/features/FEAT-102-MockPayload.md",
        &index,
    );
    assert!(findings.is_empty());

    let feat101_content =
        "| **Schema Containers** | `MockSystem::MockPlatform` |\n# Feature 101\nExecuteAction\n";
    let feat101_doc = MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([(
            "schema_containers".to_string(),
            "MockSystem::MockPlatform".to_string(),
        )]),
        prose: vec![],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };
    let findings_101 = validate_feature_operation_coverage(
        &feat101_doc,
        feat101_content,
        "docs/features/FEAT-101-MockPlatform.md",
        &index,
    );
    assert_eq!(findings_101.len(), 1);
    assert_eq!(findings_101.first().map(|f| f.rule_id.as_str()), Some("feature-operation-missing"));
    assert!(findings_101.first().is_some_and(|f| f.message.contains("VerifyInterface")));
}

#[test]
fn test_user_story_test_case_accepts_assert_constraint_ids() {
    let mut model = SysmlModel::default();
    for name in ["Assert_UCA_33", "Assert_UCA_34"] {
        model.constraints.push(deap_core::sysml_cst::ConstraintDef {
            name: name.to_string(),
            is_assertion: true,
            ..Default::default()
        });
    }
    model.test_cases.push(deap_core::sysml_cst::TestCaseDef {
        name: "TC_UCA_33_Verification".to_string(),
        verified_requirements: vec!["Assert_UCA_33".to_string(), "Assert_UCA_34".to_string()],
        ..Default::default()
    });
    model.test_cases.push(deap_core::sysml_cst::TestCaseDef {
        name: "TC_Invalid_Target".to_string(),
        verified_requirements: vec!["NonExistentRequirement_99".to_string()],
        ..Default::default()
    });

    let index = SysmlSymbolIndex::from_model(&model);
    let make_doc = |tc: &str| MarkdownDoc {
        headings: vec![],
        frontmatter: None,
        table_metadata: BTreeMap::from([("sysml_test_case".to_string(), tc.to_string())]),
        prose: vec![],
        mermaid_blocks: vec![],
        unresolved_placeholders: vec![],
    };
    let valid_story_doc = make_doc("TC_UCA_33_Verification");
    assert!(validate_user_story_test_cases(&valid_story_doc, "SysML Test Case: TC_UCA_33_Verification", "docs/user-stories/US-201-Valid.md", &index).is_empty());

    let invalid_story_doc = make_doc("TC_Invalid_Target");
    let findings_invalid = validate_user_story_test_cases(&invalid_story_doc, "SysML Test Case: TC_Invalid_Target", "docs/user-stories/US-203-Invalid.md", &index);
    assert_eq!(findings_invalid.len(), 1);
    assert_eq!(findings_invalid.first().map(|f| f.rule_id.as_str()), Some("test-case-verify-requirement-invalid"));

    let ghost_story_doc = make_doc("TC_Ghost");
    let findings_ghost = validate_user_story_test_cases(&ghost_story_doc, "SysML Test Case: TC_Ghost", "docs/user-stories/US-204-Ghost.md", &index);
    assert_eq!(findings_ghost.len(), 1);
    assert_eq!(findings_ghost.first().map(|f| f.rule_id.as_str()), Some("user-story-test-case-not-in-ast"));
}

#[test]
fn test_index_state_registers_transition_names_triggers_and_states(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut model = SysmlModel::default();
    model.parts.push(PartDef {
        name: "ParserEngine".to_string(),
        states: vec![StateDef {
            name: "ParserLifecycle".to_string(),
            sub_states: vec![
                StateDef { name: "Uninitialized".to_string(), ..Default::default() },
                StateDef { name: "Active".to_string(), ..Default::default() },
            ],
            transitions: vec![deap_core::sysml_cst::TransitionDef {
                name: Some("Uninitialized_to_Active_on_parse_token".to_string()),
                source: Some("Uninitialized".to_string()),
                target: Some("Active".to_string()),
                trigger: Some("parse_token".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    });

    let index = SysmlSymbolIndex::from_model(&model);
    assert!(index.state_names.contains("Uninitialized"));
    assert!(index.state_names.contains("Active"));
    assert!(index.action_names.contains("parse_token"));
    assert!(index.action_names.contains("Uninitialized_to_Active_on_parse_token"));

    let state_diag = StateDiagramAst {
        states: vec![
            StateDefAst { id: "Uninitialized".to_string(), label: None, is_composite: false, sub_states: Vec::new(), line: 1 },
            StateDefAst { id: "Active".to_string(), label: None, is_composite: false, sub_states: Vec::new(), line: 2 },
        ],
        transitions: vec![StateTransitionAst {
            source: "Uninitialized".to_string(),
            target: "Active".to_string(),
            event: Some("parse_token".to_string()),
            guard: None,
            action: Some("Uninitialized_to_Active_on_parse_token".to_string()),
            raw_label: Some("parse_token / Uninitialized_to_Active_on_parse_token".to_string()),
            line: 3,
        }],
        notes: Vec::new(),
    };
    assert!(validate_state_diagram(&state_diag, &index).is_ok());
    Ok(())
}

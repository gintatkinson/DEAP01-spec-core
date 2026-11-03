#![cfg(test)]

use super::*;
use crate::spec_audit::audit_runner::resolve_behavioral_triggers_path;
use std::fs;

#[test]
fn test_infer_spec_type() {
    assert_eq!(infer_spec_type("docs/epics/epic-01-sysml.md"), "epic");
    assert_eq!(infer_spec_type("docs/features/feat-01-human.md"), "feature");
    assert_eq!(
        infer_spec_type("docs/user-stories/us-01-telemetry.md"),
        "user_story"
    );
    assert_eq!(
        infer_spec_type("docs/use-cases/uc-01-mission.md"),
        "use_case"
    );
    assert_eq!(infer_spec_type("feat-02-foo.md"), "feature");
    assert_eq!(infer_spec_type("other/notes.md"), "generic");
}

#[test]
fn test_active_gates_filter() -> Result<(), Box<dyn std::error::Error>> {
    let all = ActiveGates::parse(None)?;
    assert!(all.uml && all.sections && all.metadata && all.behavioral);

    let uml = ActiveGates::parse(Some("uml"))?;
    assert!(uml.uml && !uml.sections && !uml.metadata);

    let sections = ActiveGates::parse(Some("sections"))?;
    assert!(sections.sections && !sections.uml);

    let invalid = ActiveGates::parse(Some("invalid-gate"));
    assert!(invalid.is_err());
    Ok(())
}

#[test]
fn test_empty_workspace_allow_missing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!("spec_audit_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);

    let options = SpecAuditOptions {
        repo_root: temp_dir.clone(),
        only_file: None,
        gate_filter: None,
        allow_missing_specs: true,
        strict: false,
    };

    let summary = run_spec_audit(&options)?;
    assert_eq!(summary.total_files_audited, 0);
    assert_eq!(summary.total_findings, 0);
    assert!(summary.passed);

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_empty_workspace_disallow_missing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir =
        std::env::temp_dir().join(format!("spec_audit_test_strict_{}", std::process::id()));
    let rules_dir = temp_dir.join("rules");
    let _ = fs::create_dir_all(&rules_dir);
    let _ = fs::write(rules_dir.join("behavioral_triggers.json"), "[]");

    let options = SpecAuditOptions {
        repo_root: temp_dir.clone(),
        only_file: None,
        gate_filter: None,
        allow_missing_specs: false,
        strict: true,
    };

    let summary = run_spec_audit(&options)?;
    assert_eq!(summary.total_files_audited, 0);
    assert_eq!(summary.total_findings, 1);
    assert!(!summary.passed);

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_gate_anti_stutter_detection() {
    assert!(check_title_anti_stutter("Use Case 01: UC 01 System Vision").is_some());
    assert!(check_title_anti_stutter("UC_XX_Workflow").is_some());
    assert!(
        check_title_anti_stutter("Use Case 01: System Vision and Foundational Invariants")
            .is_none()
    );

    assert!(check_filename_anti_stutter("UC-01-UC_01_System_Vision.md").is_some());
    assert!(
        check_filename_anti_stutter("UC-01-System_Vision_and_Foundational_Invariants.md").is_none()
    );
}

#[test]
fn test_gate_use_case_anti_boilerplate() -> Result<(), Box<dyn std::error::Error>> {
    let bad_content = r#"| Metadata Field | Value |
|---|---|
| Title | Use Case 01: System Vision |
| Type | use_case |

## Main Success Scenario
1. Step one.
2. Step two.

## Alternate and Exception Flows
5a. Condition fails.
"#;
    let bad_doc = parse_markdown(bad_content)?;
    let errors =
        validate_use_case_anti_boilerplate(&bad_doc, bad_content, "docs/use-cases/uc-01.md");
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.contains("fewer than 5 steps")));
    assert!(errors
        .iter()
        .any(|e| e.contains("fewer than 3 exception cases")));
    Ok(())
}

#[test]
fn test_gate_use_case_diagram_compliance() -> Result<(), Box<dyn std::error::Error>> {
    let bad_diagram = r#"| Metadata Field | Value |
|---|---|
| Title | Use Case 01: System Vision |
| Type | use_case |

```mermaid
flowchart LR
    Actor --> Action
```
"#;
    let doc = parse_markdown(bad_diagram)?;
    let errors = validate_use_case_diagram_compliance(&doc, "docs/use-cases/uc-01.md");
    assert!(!errors.is_empty());
    assert!(errors
        .iter()
        .any(|e| e.contains("lacks a system boundary subgraph")));
    Ok(())
}

#[test]
fn test_gate_epic_statechart_heterogeneity() -> Result<(), Box<dyn std::error::Error>> {
    let epic1_content = r#"```mermaid
stateDiagram-v2
    [*] --> Bootstrapping
    Bootstrapping --> Ingesting
    Ingesting --> Compiling
    Compiling --> Verifying
    Verifying --> [*]
```"#;
    let epic2_content = r#"```mermaid
stateDiagram-v2
    [*] --> Bootstrapping
    Bootstrapping --> Ingesting
    Ingesting --> Compiling
    Compiling --> Verifying
    Verifying --> [*]
```"#;
    let doc1 = parse_markdown(epic1_content)?;
    let doc2 = parse_markdown(epic2_content)?;
    let epics = vec![
        ("docs/epics/epic-01.md", &doc1),
        ("docs/epics/epic-02.md", &doc2),
    ];
    let errors = validate_epic_statechart_heterogeneity(&epics);
    assert!(!errors.is_empty());
    assert!(errors
        .iter()
        .any(|e| e.contains("share identical canned pipeline lifecycle statecharts")));
    Ok(())
}

#[test]
fn test_gate_parent_epic_alignment() -> Result<(), Box<dyn std::error::Error>> {
    let story_content = r#"| Metadata Field | Value |
|---|---|
| Title | US 01: Invariant Checks |
| Type | user_story |
| Parent Epic | EPIC-02-ConOps |

Links to [FEAT-06-SystemVisionEngine.md](../features/feat-06.md).
"#;
    let feat_content = r#"| Metadata Field | Value |
|---|---|
| Title | Feature 06: [System Vision] SystemVisionEngine |
| Type | feature |
| Parent Epic | EPIC-03-SystemVision |
"#;
    let epic_content = r#"| Metadata Field | Value |
|---|---|
| Title | Epic 03: System Vision |
| Type | epic |
"#;
    let s_doc = parse_markdown(story_content)?;
    let f_doc = parse_markdown(feat_content)?;
    let e_doc = parse_markdown(epic_content)?;

    let stories = vec![("docs/user-stories/us-01.md", &s_doc)];
    let features = vec![("docs/features/feat-06.md", &f_doc)];
    let epics = vec![("docs/epics/epic-03.md", &e_doc)];

    let errors = validate_parent_epic_alignment(&stories, &features, &epics);
    assert!(!errors.is_empty());
    assert!(errors
        .iter()
        .any(|e| e.contains("Parent Epic Alignment Gate violation")));
    Ok(())
}

#[test]
fn test_load_sysml_symbol_index_with_heterogeneous_schemas() -> Result<(), Box<dyn std::error::Error>>
{
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("test_schema_hetero_{}", ts));
    let schema_dir = temp_dir.join("schema");
    fs::create_dir_all(&schema_dir)?;

    let yaml_file = schema_dir.join("model.yaml");
    fs::write(
        &yaml_file,
        "subsystems:\n  - FlightComputer\n  - NavigationSensor\n",
    )?;

    let json_file = schema_dir.join("components.json");
    fs::write(
        &json_file,
        r#"{"components": [{"name": "TelemetryGateway"}]}"#,
    )?;

    let index = load_sysml_symbol_index(&temp_dir)?;
    assert!(index.part_names.contains("FlightComputer"));
    assert!(index.part_names.contains("NavigationSensor"));
    assert!(index.part_names.contains("TelemetryGateway"));

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_resolve_behavioral_triggers_path_falls_back_to_ancestor_workspace(
) -> Result<(), Box<dyn std::error::Error>> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("test_ancestor_triggers_{}", ts));
    let rules_dir = temp_root.join("rules");
    let nested_root = temp_root.join(".pipeline").join("upstream");
    fs::create_dir_all(&rules_dir)?;
    fs::create_dir_all(&nested_root)?;

    let triggers_file = rules_dir.join("behavioral_triggers.json");
    fs::write(&triggers_file, "[]")?;

    let resolved = resolve_behavioral_triggers_path(&nested_root);
    assert_eq!(resolved.as_deref(), Some(triggers_file.as_path()));

    let _ = fs::remove_dir_all(&temp_root);
    Ok(())
}

#![cfg(test)]

use super::*;

fn create_test_trigger() -> BehavioralTrigger {
    BehavioralTrigger {
        name: "High-Frequency Stream Data Trigger".to_string(),
        trigger_nodes: vec!["stream-data".to_string(), "sensor-stream".to_string()],
        rules: vec![
            BehavioralRule {
                target_type: "user-story".to_string(),
                requires_mermaid_block: Some("sequenceDiagram".to_string()),
                match_terms_in_mermaid: vec![
                    "OffThreadHandler".to_string(),
                    "ConcurrencyGuard".to_string(),
                ],
                match_terms_in_body: Vec::new(),
                match_terms_in_body_secondary: Vec::new(),
                error_message:
                    "User Story diagrams must illustrate off-thread computations (OffThreadHandler)."
                        .to_string(),
            },
            BehavioralRule {
                target_type: "use-case".to_string(),
                requires_mermaid_block: None,
                match_terms_in_mermaid: Vec::new(),
                match_terms_in_body: vec!["temporal-context".to_string(), "playback".to_string()],
                match_terms_in_body_secondary: Vec::new(),
                error_message: "Use Cases must detail the temporal context state machine."
                    .to_string(),
            },
        ],
    }
}

#[test]
fn test_inactive_trigger_imposes_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let trigger = create_test_trigger();
    let index = SysmlSymbolIndex::new();
    let specs = Vec::new();

    let findings = validate_behavioral_triggers(&[trigger], &index, &specs)?;
    assert!(
        findings.is_empty(),
        "Inactive trigger should impose nothing"
    );
    Ok(())
}

#[test]
fn test_active_trigger_missing_coverage_fails() -> Result<(), Box<dyn std::error::Error>> {
    let trigger = create_test_trigger();
    let mut index = SysmlSymbolIndex::new();
    index.part_names.insert("stream-data".to_string());

    let specs = Vec::new();

    let findings = validate_behavioral_triggers(&[trigger], &index, &specs)?;
    assert_eq!(
        findings.len(),
        2,
        "Expected 2 findings (one per rule for stream-data)"
    );
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("behavioral-trigger-node-must-be-covered-by-a-specification")
    );
    assert_eq!(
        findings.get(1).map(|f| f.rule_id.as_str()),
        Some("behavioral-trigger-node-must-be-covered-by-a-specification")
    );
    Ok(())
}

#[test]
fn test_active_trigger_user_story_passes_with_required_mermaid(
) -> Result<(), Box<dyn std::error::Error>> {
    let trigger = create_test_trigger();
    let mut index = SysmlSymbolIndex::new();
    index.part_names.insert("stream-data".to_string());

    let user_story_content = r#"# US-01 Ingest Sensor Data

The system ingests high-frequency stream-data from telemetry sensors.

```mermaid
sequenceDiagram
    participant S as Sensor
    participant O as OffThreadHandler
    S->>O: transmit(stream_data)
```
"#;

    let use_case_content = r#"# UC-01 Stream Data Playback

The operator reviews recorded stream-data.

## 4. Main Success Scenario
1. Operator requests temporal-context playback.
"#;

    let specs = vec![
        AuditedSpec::from_content(
            "docs/user-stories/us-01.md",
            "user-story",
            user_story_content,
        )?,
        AuditedSpec::from_content("docs/use-cases/uc-01.md", "use-case", use_case_content)?,
    ];

    let findings = validate_behavioral_triggers(&[trigger], &index, &specs)?;
    assert!(
        findings.is_empty(),
        "Expected 0 findings for passing specs, got: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_active_trigger_fails_when_mermaid_lacks_match_terms(
) -> Result<(), Box<dyn std::error::Error>> {
    let trigger = create_test_trigger();
    let mut index = SysmlSymbolIndex::new();
    index.part_names.insert("stream-data".to_string());

    let invalid_user_story = r#"# US-01 Ingest Sensor Data

The system ingests high-frequency stream-data from telemetry sensors.

```mermaid
sequenceDiagram
    participant S as Sensor
    participant B as Buffer
    S->>B: transmit(stream_data)
```
"#;

    let valid_use_case = r#"# UC-01 Stream Data Playback

The operator reviews recorded stream-data.

## 4. Main Success Scenario
1. Operator requests playback mode.
"#;

    let specs = vec![
        AuditedSpec::from_content(
            "docs/user-stories/us-01.md",
            "user-story",
            invalid_user_story,
        )?,
        AuditedSpec::from_content("docs/use-cases/uc-01.md", "use-case", valid_use_case)?,
    ];

    let findings = validate_behavioral_triggers(&[trigger], &index, &specs)?;
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("behavioral-trigger-rule-must-be-satisfied-by-the-specification")
    );
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("OffThreadHandler")));
    Ok(())
}

#[test]
fn test_active_trigger_fails_when_use_case_lacks_body_terms(
) -> Result<(), Box<dyn std::error::Error>> {
    let trigger = create_test_trigger();
    let mut index = SysmlSymbolIndex::new();
    index.part_names.insert("stream-data".to_string());

    let valid_user_story = r#"# US-01 Ingest Sensor Data

The system ingests high-frequency stream-data from telemetry sensors.

```mermaid
sequenceDiagram
    participant S as Sensor
    participant O as OffThreadHandler
    S->>O: transmit(stream_data)
```
"#;

    let invalid_use_case = r#"# UC-01 Stream Data Monitor

The operator reviews recorded stream-data.

## 4. Main Success Scenario
1. Operator requests direct feed view without any temporal mechanisms.
"#;

    let specs = vec![
        AuditedSpec::from_content("docs/user-stories/us-01.md", "user-story", valid_user_story)?,
        AuditedSpec::from_content("docs/use-cases/uc-01.md", "use-case", invalid_use_case)?,
    ];

    let findings = validate_behavioral_triggers(&[trigger], &index, &specs)?;
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("behavioral-trigger-rule-must-be-satisfied-by-the-specification")
    );
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("temporal context state machine")));
    Ok(())
}

#[test]
fn test_contains_token_with_word_boundary() {
    assert!(contains_token_with_word_boundary(
        "stream-data payload",
        "stream-data"
    ));
    assert!(contains_token_with_word_boundary(
        "The `stream-data` node",
        "stream-data"
    ));
    assert!(!contains_token_with_word_boundary(
        "mainstream-database",
        "stream-data"
    ));
    assert!(!contains_token_with_word_boundary(
        "stream-dataset",
        "stream-data"
    ));
}

#[test]
fn test_extract_body_outside_code_blocks() {
    let content = "Heading 1\n```mermaid\nsequenceDiagram\n```\nParagraph text\n";
    let body = extract_body_outside_code_blocks(content);
    assert!(body.contains("Heading 1"));
    assert!(body.contains("Paragraph text"));
    assert!(!body.contains("sequenceDiagram"));
}

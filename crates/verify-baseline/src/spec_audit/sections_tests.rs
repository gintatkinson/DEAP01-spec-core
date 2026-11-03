#![cfg(test)]

use super::*;
use crate::spec_audit::markdown_ast::parse_markdown;

#[test]
fn test_normalize_tokens() {
    assert_eq!(normalize_tokens("## 1. Context"), vec!["1", "context"]);
    assert_eq!(
        normalize_tokens("2. Requirements & Checklist"),
        vec!["2", "requirements", "and", "checklist"]
    );
    assert_eq!(
        normalize_tokens("5. Alternate and Exception Flows"),
        vec!["5", "alternate", "and", "exception", "flows"]
    );
    assert_eq!(
        normalize_tokens("5. Alternate & Exception Flows"),
        vec!["5", "alternate", "and", "exception", "flows"]
    );
    assert_eq!(
        normalize_tokens("System-Level UML Class Diagram"),
        vec!["system", "level", "uml", "class", "diagram"]
    );
    assert_eq!(
        normalize_tokens("Special: / Path-With_Delimiters & More"),
        vec!["special", "path", "with", "delimiters", "and", "more"]
    );
}

#[test]
fn test_section_rule_from_pattern() {
    let rule = SectionRule::from_pattern(r"(?i)##\s+1\.\s+Context", "## 1. Context");
    assert_eq!(rule.level, 2);
    assert_eq!(rule.normalized_tokens, vec!["1", "context"]);
    assert_eq!(rule.display_name, "## 1. Context");

    let rule3 =
        SectionRule::from_pattern(r"###\s+1\.\s+Payload\s+Schema", "### 1. Payload Schema");
    assert_eq!(rule3.level, 3);
    assert_eq!(rule3.normalized_tokens, vec!["1", "payload", "schema"]);
}

#[test]
fn test_epic_missing_sections() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Epic Title
## 1. Context
Prose here.
"#;
    let doc = parse_markdown(content)?;
    let errors = validate_sections("epic", &doc, "docs/epics/epic-01-test.md");
    assert!(!errors.is_empty(), "Should report missing epic sections");
    assert!(errors
        .iter()
        .any(|e| e.contains("2. Requirements & Checklist")));
    Ok(())
}

#[test]
fn test_epic_complete_sections_passes() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Epic Complete
## 1. Context
## 2. Requirements & Checklist
### Associated Use Cases & User Stories
#### Associated Use Cases
#### Associated User Stories
## 3. Architecture
## 4. Operational Considerations
## 5. Security & Governance
## 6. Source References
## System-Level UML Class Diagram
## System State Machine Diagram
"#;
    let doc = parse_markdown(content)?;
    let errors = validate_sections("epic", &doc, "docs/epics/epic-01-complete.md");
    assert!(
        errors.is_empty(),
        "Complete epic should have 0 section errors, got: {:?}",
        errors
    );
    Ok(())
}

#[test]
fn test_feature_ui_sections() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Interface Type** | UI |

## UML Class Diagram
## Interface Requirements
### 1. Test Data Shape
### 3. Visual Layout & Arrangement
### 4. Interactive Flow & States
"#;
    let doc = parse_markdown(content)?;
    let errors = validate_sections("feature", &doc, "docs/features/feat-01-ui.md");
    assert!(
        errors.is_empty(),
        "Complete UI feature should have 0 section errors, got: {:?}",
        errors
    );
    Ok(())
}

#[test]
fn test_heading_level_mismatch_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Epic Title
### 1. Context
## 2. Requirements & Checklist
## 3. Architecture
## 4. Operational Considerations
## 5. Security & Governance
## 6. Source References
## System-Level UML Class Diagram
## System State Machine Diagram
"#;
    let doc = parse_markdown(content)?;
    let errors = validate_sections("epic", &doc, "docs/epics/epic-01-mismatch.md");
    assert!(
        errors.iter().any(|e| e.contains("1. Context")),
        "H3 should not satisfy H2 requirement for 1. Context"
    );
    Ok(())
}

#[test]
fn test_anti_boilerplate_gate() -> Result<(), Box<dyn std::error::Error>> {
    let canned_content = r#"# Use Case 01: System Vision
## 4. Main Success Scenario
1. Actor `HumanEngineer` initiates workflow execution.
2. System `DEAPCompilerSystem` parses input model definitions.
3. Component `SystemVisionEngine` processes structural elements and validates semantic constraints.
4. System verifies invariant satisfaction.

## 5. Alternate and Exception Flows
- 5a. Syntax error detected: System logs error.
"#;
    let doc_canned = parse_markdown(canned_content)?;
    let errors = validate_use_case_anti_boilerplate(
        &doc_canned,
        canned_content,
        "docs/use-cases/uc-01.md",
    );
    assert!(errors
        .iter()
        .any(|e| e.contains("initiates workflow execution")));
    assert!(errors.iter().any(
        |e| e.contains("processes structural elements and validates semantic constraints")
    ));
    assert!(errors
        .iter()
        .any(|e| e.contains("Main Success Scenario has fewer than 5 steps")));
    assert!(errors
        .iter()
        .any(|e| e.contains("Alternate and Exception Flows has fewer than 3 exception cases")));

    let valid_content = r#"# Use Case 01: System Vision
## 4. Main Success Scenario
1. Operator dispatches compilation command with workspace schema root.
2. SystemVisionEngine initializes compiler namespace UUID constant NS_DEAP.
3. InvariantBootstrapEngine asserts zero hardcoded domain concept rules across runtime context.
4. NamespaceUuidGenerator pre-computes topological UUIDv5 namespace roots.
5. DomainVocabularyValidator inspects grammar vocabulary tokens against prohibited domain ontologies.
6. SystemVisionEngine emits immutable CompilerRulePort descriptor to downstream ingestion engines.

## 5. Alternate and Exception Flows
- 5a. Prohibited domain term detected: SystemVisionEngine halts pipeline and emits diagnostic E0100.
- 5b. Namespace UUID collision: SystemVisionEngine regenerates topological anchor.
- 5c. Malformed configuration flags: Compiler logs invalid argument diagnostics.
- 5d. File permission error on workspace: Execution halts safely.
"#;
    let doc_valid = parse_markdown(valid_content)?;
    let errors_valid = validate_use_case_anti_boilerplate(
        &doc_valid,
        valid_content,
        "docs/use-cases/uc-01.md",
    );
    assert!(
        errors_valid.is_empty(),
        "Valid use case should have 0 boilerplate errors: {:?}",
        errors_valid
    );
    Ok(())
}

#[test]
fn test_issue_449_epic_associated_use_cases_and_user_stories_sections_required(
) -> Result<(), Box<dyn std::error::Error>> {
    let missing_assoc_content = r#"# Epic Complete
## 1. Context
## 2. Requirements & Checklist
## 3. Architecture
## 4. Operational Considerations
## 5. Security & Governance
## 6. Source References
## System-Level UML Class Diagram
## System State Machine Diagram
"#;
    let doc = parse_markdown(missing_assoc_content)?;
    let errors = validate_sections("epic", &doc, "docs/epics/epic-01-missing-assoc.md");
    assert!(
        errors.iter().any(|e| e.contains("Associated Use Cases & User Stories")),
        "Epic missing '### Associated Use Cases & User Stories' must produce section validation error, got: {:?}",
        errors
    );
    assert!(
        errors.iter().any(|e| e.contains("Associated Use Cases")),
        "Epic missing '#### Associated Use Cases' must produce section validation error, got: {:?}",
        errors
    );
    assert!(
        errors.iter().any(|e| e.contains("Associated User Stories")),
        "Epic missing '#### Associated User Stories' must produce section validation error, got: {:?}",
        errors
    );
    Ok(())
}

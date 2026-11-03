#![cfg(test)]

use super::*;
use crate::spec_audit::markdown_ast::parse_markdown;

#[test]
fn test_iso_date_validation() {
    assert!(is_iso_date("2026-10-10"));
    assert!(is_iso_date("2024-02-29"));
    assert!(!is_iso_date("2026-13-01"));
    assert!(!is_iso_date("10/10/2026"));
    assert!(!is_iso_date("invalid-date"));
    assert!(!is_iso_date("2026-00-10"));
    assert!(!is_iso_date("2026-10-00"));
    assert!(!is_iso_date("2026-10-32"));
    assert!(!is_iso_date("1969-12-31"));
    assert!(!is_iso_date("2101-01-01"));
}

#[test]
fn test_semver_validation() {
    assert!(is_semver("1.0.0"));
    assert!(is_semver("v2.1.0"));
    assert!(is_semver("0.1.0-alpha.1"));
    assert!(is_semver("v1.0.0+20130313144700"));
    assert!(is_semver("1.2"));
    assert!(!is_semver("1"));
    assert!(!is_semver("v1"));
    assert!(!is_semver("release-1.0"));
    assert!(!is_semver("1.0.0.0"));
    assert!(!is_semver("v"));
}

#[test]
fn test_has_concatenated_title_metadata() {
    assert!(has_concatenated_title_metadata(
        "Core System Architecture v1.0.0"
    ));
    assert!(has_concatenated_title_metadata("Subsystem Spec 2026-10-10"));
    assert!(has_concatenated_title_metadata(
        "Navigation Module (DOC-NAV-001)"
    ));
    assert!(has_concatenated_title_metadata("Version: 2 Architecture"));
    assert!(!has_concatenated_title_metadata("Core System Architecture"));
}

#[test]
fn test_normalize_title() {
    assert_eq!(
        normalize_title("EPIC-001: Core System Architecture"),
        "core system architecture"
    );
    assert_eq!(
        normalize_title("feat-02: User Authentication Flow"),
        "user authentication flow"
    );
    assert_eq!(
        normalize_title("US-03 - Execute Telemetry Check!"),
        "execute telemetry check"
    );
    assert_eq!(
        normalize_title("Feature: User Authentication"),
        "user authentication"
    );
    assert_eq!(
        normalize_title("UC 04: Emergency Landing"),
        "emergency landing"
    );
    assert_eq!(
        normalize_title("User Story 10 - Sensor Telemetry"),
        "sensor telemetry"
    );
    assert_eq!(
        normalize_title("epics-12: System Overview"),
        "system overview"
    );
    assert_eq!(
        normalize_title("Feature 01: [ConOps] Human_Engineer_Interface"),
        "conops human engineer interface"
    );
    assert_eq!(
        normalize_title("Feature_01_Human_Engineer"),
        "human engineer"
    );
    assert_eq!(
        normalize_title("Epic_02: Radar_Tracker_Architecture"),
        "radar tracker architecture"
    );
    assert_eq!(
        normalize_title("User_Story_05: Ingest_Telemetry_Stream"),
        "ingest telemetry stream"
    );
    assert_eq!(
        normalize_title("Use_Case_03: Process_Radar_Tracks"),
        "process radar tracks"
    );
}

#[test]
fn test_title_uniqueness_detected() -> Result<(), Box<dyn std::error::Error>> {
    let content1 = "# EPIC-01: Core Architecture\n";
    let content2 = "# EPIC-02: Core Architecture\n";
    let doc1 = parse_markdown(content1)?;
    let doc2 = parse_markdown(content2)?;

    let docs = vec![
        (
            "epic".to_string(),
            "docs/epics/epic-01.md".to_string(),
            doc1,
        ),
        (
            "epic".to_string(),
            "docs/epics/epic-02.md".to_string(),
            doc2,
        ),
    ];

    let errors = validate_title_uniqueness(&docs);
    assert_eq!(errors.len(), 1);
    assert!(errors
        .first()
        .is_some_and(|e| e.contains("duplicate epic title")));
    Ok(())
}

#[test]
fn test_filename_conventions() {
    let clean_files = ["feat-01-login.md", "feat-02-logout.md"];
    let errors = validate_filenames("feature", &clean_files, "docs/features");
    assert!(errors.is_empty());

    let duplicate_ordinal = ["feat-01-login.md", "feat-01-duplicate.md"];
    let errors = validate_filenames("feature", &duplicate_ordinal, "docs/features");
    assert_eq!(errors.len(), 1);
    assert!(errors
        .first()
        .is_some_and(|e| e.contains("duplicate ordinal")));

    let mixed_padding = ["feat-01-login.md", "feat-002-logout.md"];
    let errors = validate_filenames("feature", &mixed_padding, "docs/features");
    assert_eq!(errors.len(), 1);
    assert!(errors
        .first()
        .is_some_and(|e| e.contains("inconsistent ordinal padding width")));
}

#[test]
fn test_anti_stutter_gate_title() {
    assert!(check_title_anti_stutter("Use Case 01: UC 01 System Vision").is_some());
    assert!(check_title_anti_stutter("Use Case 01: UC-01 Foundational Invariants").is_some());
    assert!(check_title_anti_stutter("Use Case 01: UC_01 Invariants").is_some());
    assert!(check_title_anti_stutter("UC_XX_Workflow").is_some());
    assert!(check_title_anti_stutter("UC_01_System_Vision").is_some());
    assert!(check_title_anti_stutter("Epic 01: EPIC 01 SysML").is_some());
    assert!(check_title_anti_stutter("Feature 01: FEAT 01 Engine").is_some());
    assert!(check_title_anti_stutter("User Story 01: US 01 Bootstrap").is_some());
    assert!(check_title_anti_stutter("Use Case 01: Use Case 01 Workflow").is_some());

    assert!(
        check_title_anti_stutter("Use Case 01: System Vision and Foundational Invariants")
            .is_none()
    );
    assert!(check_title_anti_stutter("Epic 01: SysML Model").is_none());
    assert!(check_title_anti_stutter("Feature 06: [System Vision] SystemVisionEngine").is_none());
    assert!(check_title_anti_stutter("User Story 01: Bootstrap System Vision Action").is_none());
}

#[test]
fn test_anti_stutter_gate_filename() {
    assert!(check_filename_anti_stutter("UC-01-UC_01_System_Vision.md").is_some());
    assert!(check_filename_anti_stutter("UC-01-UC-01-System_Vision.md").is_some());
    assert!(check_filename_anti_stutter("uc-01-uc_01_workflow.md").is_some());
    assert!(check_filename_anti_stutter("uc-01-uc_xx_workflow.md").is_some());
    assert!(check_filename_anti_stutter("feat-01-feat_01_engine.md").is_some());
    assert!(check_filename_anti_stutter("us-01-us_01_action.md").is_some());
    assert!(check_filename_anti_stutter("epic-01-epic_01_model.md").is_some());

    assert!(
        check_filename_anti_stutter("UC-01-System_Vision_and_Foundational_Invariants.md").is_none()
    );
    assert!(check_filename_anti_stutter("FEAT-06-SystemVisionEngine.md").is_none());
    assert!(check_filename_anti_stutter("US-01-BootstrapSystemVisionAction.md").is_none());
    assert!(check_filename_anti_stutter("EPIC-03-SystemVision.md").is_none());
}

#[test]
fn test_validate_metadata_generation_mode_variants() -> Result<(), Box<dyn std::error::Error>> {
    let template = |key: &str, val: &str| {
        format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #10 |
| **Title** | Feature 01 Telemetry Stream |
| **Type** | feature |
| **Parent Epic** | [EPIC-01](../epics/EPIC-01.md) |
| **Package** | FlightControl |
| **Interface Type** | programmatic |
| **Schema Containers** | FlightPackage |
| **{}** | {} |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 01: Telemetry Stream
"#,
            key, val
        )
    };

    let variants = [
        ("Generation Mode", "subagent"),
        ("generation-mode", "subagent"),
        ("subagent_drafted", "true"),
        ("subagent", "autonomous"),
        ("subagent_mode", "active"),
    ];

    for (k, v) in variants {
        let md = template(k, v);
        let doc = parse_markdown(&md)?;
        let errors =
            validate_metadata("feature", &doc, &md, "docs/features/feat-01-telemetry.md");
        assert!(
            errors.is_empty(),
            "Variant '{}' failed with errors: {:?}",
            k,
            errors
        );
    }
    Ok(())
}

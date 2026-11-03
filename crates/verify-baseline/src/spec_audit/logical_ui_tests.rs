#![cfg(test)]

use super::*;

#[test]
fn test_valid_lumi_spec_passes() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"---
title: "Telemetry Viewport Feature"
interface_type: "ui"
---

# Feat-01 Telemetry Viewport

## 1. Context
Visual telemetry monitoring interface.

## 3. Visual Layout & Arrangement
The UI uses responsive split panes with @container isolation rules.

### Layer 1: Domain State & Signal Model
- SensorTelemetryStream
- TelemetryBuffer

### Layer 2: Logic & Safety State Management
- TelemetryViewModel
- SafetyStateController

### Layer 3: Presentation & Actuator Interface Binding
- TelemetryViewportWidget
- ActuatorControlPanel
"#;
    let findings = validate_logical_ui_content(content, "docs/features/feat-01.md")?;
    assert!(
        findings.is_empty(),
        "Expected valid LUMI spec to produce 0 findings, got: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_lumi_missing_layer_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"---
title: "Telemetry Viewport Feature"
interface_type: "ui"
---

# Feat-01 Telemetry Viewport

## 3. Visual Layout & Arrangement
The UI uses responsive split panes with @container isolation rules.

### Layer 1: Domain State & Signal Model
- SensorTelemetryStream

### Layer 3: Presentation & Actuator Interface Binding
- TelemetryViewportWidget
"#;
    let findings = validate_logical_ui_content(content, "docs/features/feat-01.md")?;
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("lumi-3-layer-chain-incomplete")
    );
    assert!(findings.first().is_some_and(|f| f
        .message
        .contains("Layer 2: Logic & Safety State Management")));
    Ok(())
}

#[test]
fn test_lumi_split_panes_missing_container_query_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"---
title: "Telemetry Viewport Feature"
interface_type: "ui"
---

# Feat-01 Telemetry Viewport

## 3. Visual Layout & Arrangement
The UI features a resizable split pane between telemetry and controls.

### Layer 1: Domain State & Signal Model
- TelemetryModel

### Layer 2: Logic & Safety State Management
- TelemetryViewModel

### Layer 3: Presentation & Actuator Interface Binding
- ViewportWidget
"#;
    let findings = validate_logical_ui_content(content, "docs/features/feat-01.md")?;
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("lumi-missing-container-query")
    );
    Ok(())
}

#[test]
fn test_platform_independence_dom_attribute_leak_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# US-01 User Authentication

The user inputs credentials in the form.

The input field carries aria-label="Username" and role="textbox" with tabindex="0".
"#;
    let findings = validate_logical_ui_content(content, "docs/user-stories/us-01.md")?;
    assert_eq!(findings.len(), 3);
    for f in &findings {
        assert_eq!(f.rule_id, "platform-independence-dom-attribute-leak");
    }
    Ok(())
}

#[test]
fn test_platform_independence_hardcoded_pixels_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feat-02 Status Card

The card width is fixed at 450px and height at 120px.
Maximum bound is 1000px.
"#;
    let findings = validate_logical_ui_content(content, "docs/features/feat-02.md")?;
    assert_eq!(findings.len(), 3);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("platform-independence-hardcoded-pixel-dimension")
    );
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("450px")));
    assert!(findings.get(1).is_some_and(|f| f.message.contains("120px")));
    assert!(findings
        .get(2)
        .is_some_and(|f| f.message.contains("1000px")));
    Ok(())
}

#[test]
fn test_platform_independence_framework_leak_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feat-03 Flight Display

Acceptance Criteria:
- Built with Flutter BLoC pattern.
- State rendered using React component.
- Supports Electron desktop packaging.
"#;
    let findings = validate_logical_ui_content(content, "docs/features/feat-03.md")?;
    assert_eq!(findings.len(), 3);
    for f in &findings {
        assert_eq!(f.rule_id, "platform-independence-framework-leak");
    }
    Ok(())
}

#[test]
fn test_english_verb_react_to_is_not_flagged() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# US-02 Emergency Response

The vehicle flight computer must react to emergency abort signals within 10 milliseconds.
The system will react with an automatic parachute deployment.
"#;
    let findings = validate_logical_ui_content(content, "docs/user-stories/us-02.md")?;
    assert!(
        findings.is_empty(),
        "English verb 'react to' / 'react with' should not be flagged as framework leak: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_pixel_detection_boundary_cases() {
    assert_eq!(
        find_pixel_dimensions_in_line("width: 120px;"),
        vec!["120px"]
    );
    assert_eq!(
        find_pixel_dimensions_in_line("margin: 2.5px;"),
        vec!["2.5px"]
    );
    assert!(find_pixel_dimensions_in_line("complex rendering").is_empty());
    assert!(find_pixel_dimensions_in_line("px-to-rem").is_empty());
    assert!(find_pixel_dimensions_in_line("approx 10").is_empty());
}

#[test]
fn test_layout_guidelines_raw_na_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feat-04 Navigation Pane

| Channel | Component | Container |
| :--- | :--- | :--- |
| GUI | NavigationTree | N/A |
"#;
    let findings = validate_logical_ui_content(content, "docs/features/feat-04.md")?;
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("lumi-prohibit-raw-na-fallback")
    );
    Ok(())
}

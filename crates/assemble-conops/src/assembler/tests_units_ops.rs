//! Unit tests for ConOps Unit 01 (metadata/overview), Unit 10 (maintenance/GSE), and Unit 09 (scenarios/timelines).

use std::fs;
use std::path::PathBuf;

use super::*;
use crate::params::SysMLParameterBindingEngine;

#[test]
fn test_canonical_unit_01_metadata_and_overview_integrity() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let unit_01_path = repo_root.join(
        "skills/spec-conops-engineering/resources/units/conops/01_METADATA_AND_OVERVIEW.md",
    );

    if !unit_01_path.is_file() {
        return Ok(());
    }

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&unit_01_path), Some(&engine));
    assert!(is_valid, "Unit 01 integrity validation failed: {errors:?}");
    assert!(errors.is_empty());

    let raw = fs::read_to_string(&unit_01_path)?;
    for required in [
        "### 1.3.1 Parametric Coordinate Reference Frames Math",
        "Primary Global Geodetic Frame",
        "Local Tangent Plane / Navigation Frame",
        "Body-Fixed Geometric Frame",
        "### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table",
        "### 1.3.3 Master Physical Limits Table",
        "### 1.4 Abstract UAF Context Diagram",
        "flowchart TB",
        "subgraph SupportSegment[\"Support & Ground Support Equipment (GSE) Segment\"]",
        "subgraph ControlSegment",
        "subgraph OperationalRoles",
        "subgraph ExternalAuthorities",
        "### 1.5 Normative Standards & Regulatory Baseline",
        "ISO/IEC/IEEE 29148:2018",
        "ISO/IEC/IEEE 15288:2023",
    ] {
        assert!(raw.contains(required), "Unit 01 must contain '{required}'");
    }
    assert!(raw.contains("{{TOTAL_MASS_KG}}") || raw.contains("{{TOTAL_MTOW_KG}}"));
    assert!(raw.contains("{{M_MAX_LIMIT_KG}}") || raw.contains("{{MTOW_MAX_KG}}"));
    assert!(!raw.contains("subgraph AuxSupport"));
    assert!(!raw.contains('\u{2014}'));
    assert!(!raw.contains('\u{2013}'));
    Ok(())
}

#[test]
fn test_canonical_unit_10_maintenance_and_gse_support_integrity(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let unit_10_path = repo_root
        .join("skills/spec-conops-engineering/resources/units/conops/10_MAINTENANCE_AND_GSE_SUPPORT.md");

    if !unit_10_path.is_file() {
        return Ok(());
    }

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&unit_10_path), Some(&engine));
    assert!(is_valid, "Unit 10 integrity validation failed: {errors:?}");
    assert!(errors.is_empty());

    let raw = fs::read_to_string(&unit_10_path)?;
    for card in ["MTC-01", "MTC-01A", "MTC-02", "MTC-03", "MTC-04", "MTC-05"] {
        assert!(raw.contains(&format!("**{card}**")), "Unit 10 must contain {card}");
    }
    for required in [
        "#### 10.1.2 Rapid Sortie Turnaround Workflow & 7-Step Protocol",
        "flowchart TD",
        "The 7-step turnaround sequence encompasses:",
        "- Step 1: System Ingress & Safe-State Disarm",
        "- Step 2: Telemetry Data Offload & Log Audit",
        "- Step 3: Rapid Energy Module Hot-Swap",
        "- Step 4: Rapid Visual & Structural Inspection",
        "- Step 5: Mission Re-Tasking & Key Injection",
        "- Step 6: Automated PBIT Diagnostics",
        "- Step 7: Final Arming & Sortie Release",
        "- Contingency: Abort Turnaround & Route to MTC-04 LRU Swap",
        "Step 1: System Ingress &<br/>Safe-State Disarm",
        "Step 7: Final Arming &<br/>Sortie Release",
        "Abort Turnaround &<br/>Route to MTC-04 LRU Swap",
        "### 10.2 Support Equipment (SE) Taxonomy",
    ] {
        assert!(raw.contains(required), "Unit 10 must contain '{required}'");
    }
    assert!(!raw.contains("Step 1: System Ingress and<br/>Safe-State Disarm"));
    for se_id in ["SE-01", "SE-02", "SE-03", "SE-04", "SE-05", "SE-06"] {
        assert!(raw.contains(&format!("**{se_id}**")), "Unit 10 must contain {se_id}");
    }
    assert!(!raw.contains('\u{2014}'));
    assert!(!raw.contains('\u{2013}'));
    Ok(())
}

#[test]
fn test_canonical_unit_09_scenarios_and_timelines_integrity(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let unit_09_path = repo_root.join(
        "skills/spec-conops-engineering/resources/units/conops/09_SCENARIOS_AND_TIMELINES.md",
    );

    if !unit_09_path.is_file() {
        return Ok(());
    }

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&unit_09_path), Some(&engine));
    assert!(
        is_valid,
        "Unit 09 integrity validation failed: {:?}",
        errors
    );
    assert!(errors.is_empty());

    let raw = fs::read_to_string(&unit_09_path)?;

    assert!(raw.contains("### 9.1 Scenario SCN-01: Normal Operational Cycle"));
    assert!(raw.contains("### 9.2 Scenario SCN-02: Boundary Excursion & Auto-Recovery"));
    assert!(raw.contains(
        "### 9.3 Scenario SCN-03: Degraded C2 Lost-Link & Autonomous Fallback Return"
    ));
    assert!(raw.contains("### 9.4 Scenario SCN-04: Emergency State Safing"));
    assert!(raw.contains(
        "### 9.5 Scenario SCN-05: Sensor Stream Correlation & State Synchronization"
    ));

    assert!(raw.contains("sequenceDiagram"));
    assert!(raw.contains("stateDiagram-v2"));

    assert!(
        !raw.contains("Phase_Ingress_StationKeeping"),
        "Unit 09 must not contain Phase_Ingress_StationKeeping"
    );
    assert!(
        !raw.contains("Phase_PID_InterlockCheck"),
        "Unit 09 must not contain Phase_PID_InterlockCheck"
    );
    assert!(
        !raw.contains("Phase_DualConsent_ArmingExecution"),
        "Unit 09 must not contain Phase_DualConsent_ArmingExecution"
    );
    assert!(
        !raw.contains("Phase_PostAction_AssessmentDump"),
        "Unit 09 must not contain Phase_PostAction_AssessmentDump"
    );

    assert!(raw.contains("Phase_Stream_Ingress"));
    assert!(raw.contains("Phase_Feature_Correlation"));
    assert!(raw.contains("Phase_State_Synchronization"));

    assert!(
        !raw.contains('\u{2014}'),
        "Unit 09 must not contain Unicode em dash"
    );
    assert!(
        !raw.contains('\u{2013}'),
        "Unit 09 must not contain Unicode en dash"
    );
    Ok(())
}

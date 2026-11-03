//! Reproducer test helpers for GitHub Issue #695 (`compile-sysml` domain-neutral safety suite).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_d_tests.rs` (refs #695).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Non-UAV `PackageDef` models with zero `CalcDef` and zero `ConstraintDef` declarations emit
//!   `06_REGULATORY_OBJECTIVES_ASSESSMENT.md` (never `06_SORA_SAIL_ASSESSMENT.md`), zero UAV
//!   aerodynamic `PROOF_TEMPLATES`, zero drone SORA/SAIL/OSO substrings, and zero `sldv.assert(`
//!   statements in `SLDV_FORMAL_PROOFS.m`.
//! - Models declaring `ConstraintDef`, `CalcDef`, and `AttributeDef` regulatory/integrity items
//!   derive RTA display-math proofs, `sldv.assert(` obligations, and regulatory objective tables
//!   purely from the AST without fabricating unmodeled `OSO-*` or `OBJ-04`..`OBJ-10` rows.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All temporary directories are cleaned up deterministically and all assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(A \cdot L)$ where $A = 10$ is the safety artifact count.
//! - Space Complexity: $O(A \cdot L)$ for the in-memory `BTreeMap<String, String>` suite.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `transpile_safety_suite` and `emit_safety_suite`.
//! - State: Pure in-memory and isolated `TempDirGuard` state.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims ephemeral files on drop.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-STPA-SAFETY-SUITE-NO-HARDCODED-UAV-PROOFS], [REQ-STPA-AST-DERIVED-REGULATORY-OBJECTIVES] | SysML v2: VerificationCaseDef(reproducer_d_tests)

use std::collections::BTreeMap;

use deap_core::sysml_cst::{
    ActionDef, AttributeDef, CalcDef, ConstraintDef, PackageDef, PartDef,
};

use crate::reproducer_tests::TempDirGuard;

const FORBIDDEN_UAV_PHYSICS: &[&str] = &[
    "LiftToDragRatio",
    "SinkRate",
    "StallSpeed",
    "V_stall",
    "Aerodynamic drag",
    "PropulsionPower",
    "AvionicsPower",
    "GlideDistance",
    "WellClearRadius",
    "HorizontalSeparationAtCPA",
    "Terminal dive mass",
    "Aeroelastic dynamic pressure",
    "Kinetic Energy Dissipation Bound",
    "Containment Reach Bound",
    "Energy Balance Separation Bound",
];

const FORBIDDEN_SORA_SUBSTRINGS: &[&str] = &[
    "SORA",
    "Ground Risk Class",
    "GRC",
    "Air Risk Class",
    "ARC",
    "Specific Assurance and Integrity Level",
    "| SAIL |",
    "| OSO ID |",
    "Operational Safety Objectives",
    "SAE ARP4761",
];

fn assert_zero_uav_physics(suite: &BTreeMap<String, String>) {
    for (artifact_name, content) in suite {
        for needle in FORBIDDEN_UAV_PHYSICS {
            assert!(
                !content.contains(needle),
                "Safety artifact {} must contain zero hardcoded UAV flight-physics substring {:?}:\n{}",
                artifact_name,
                needle,
                content
            );
        }
    }
}

/// Executes Issue #695 checks verifying non-UAV safety suites omit UAV proofs and SORA tables.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all 10 artifacts are domain-neutral.
///
/// # Complexity
/// * Time: $O(A \cdot L)$ over the 10 safety artifacts. Space: $O(A \cdot L)$.
///
/// # Failure Modes
/// * Propagates any I/O failure and cleans up via `TempDirGuard`.
pub fn run_issue_695_non_uav_safety_suite_checks() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "DatabaseTransactionPkg".to_string(),
        part_defs: vec![PartDef {
            name: "TransactionCoordinator".to_string(),
            ..Default::default()
        }],
        action_defs: vec![ActionDef {
            name: "CommitWriteBatch".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let temp = TempDirGuard::new("deap_issue_695_non_uav_safety_suite")?;
    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    compile_sysml::stpa::safety_suite::emit_safety_suite(&pkg, temp.path(), None)?;

    assert_eq!(
        suite.len(),
        10,
        "transpile_safety_suite must emit 10 safety artifacts, got {}: {:?}",
        suite.len(),
        suite.keys().collect::<Vec<_>>()
    );
    assert!(
        !suite.contains_key("06_SORA_SAIL_ASSESSMENT.md")
            && !temp.path().join("06_SORA_SAIL_ASSESSMENT.md").exists(),
        "Safety suite must not contain or write drone-specific 06_SORA_SAIL_ASSESSMENT.md"
    );
    assert!(
        suite.contains_key("06_REGULATORY_OBJECTIVES_ASSESSMENT.md")
            && temp
                .path()
                .join("06_REGULATORY_OBJECTIVES_ASSESSMENT.md")
                .is_file(),
        "Safety suite must contain and write domain-neutral 06_REGULATORY_OBJECTIVES_ASSESSMENT.md"
    );

    assert_zero_uav_physics(&suite);
    for (artifact_name, content) in &suite {
        for needle in FORBIDDEN_SORA_SUBSTRINGS {
            assert!(
                !content.contains(needle),
                "Non-UAV safety artifact {} must contain zero hardcoded drone SORA substring {:?}:\n{}",
                artifact_name,
                needle,
                content
            );
        }
    }

    let sldv_content = suite
        .get("SLDV_FORMAL_PROOFS.m")
        .ok_or("Missing SLDV_FORMAL_PROOFS.m in transpiled safety suite")?;
    assert!(
        !sldv_content.contains("sldv.assert("),
        "SLDV_FORMAL_PROOFS.m must contain zero sldv.assert( calls when PackageDef has zero CalcDef and zero ConstraintDef nodes:\n{}",
        sldv_content
    );
    Ok(())
}

fn build_issue_695_query_safety_pkg() -> PackageDef {
    let supervisor = PartDef {
        name: "QuerySupervisor".to_string(),
        attributes: vec![
            AttributeDef {
                name: "IntegrityLevel_Service_Tier".to_string(),
                default_value: Some("SIL-3".to_string()),
                ..Default::default()
            },
            AttributeDef {
                name: "OBJ01_Objective".to_string(),
                default_value: Some("Deterministic rollback under arena exhaustion".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    PackageDef {
        name: "QuerySafetyPkg".to_string(),
        part_defs: vec![supervisor],
        constraint_defs: vec![ConstraintDef {
            name: "LatencyCeiling".to_string(),
            expression: "queryLatencyMs <= 50".to_string(),
            ..Default::default()
        }],
        calc_defs: vec![CalcDef {
            name: "HeadroomMargin".to_string(),
            expression: Some("quotaMb - activeUsageMb".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn assert_issue_695_regulatory_docs(reg_md: &str, stpa_md: &str) {
    assert!(
        stpa_md.contains("## 8. Domain Regulatory Objectives & Integrity Level Mapping"),
        "STPA_MATRIX.md must contain '## 8. Domain Regulatory Objectives & Integrity Level Mapping':\n{}",
        stpa_md
    );

    for (doc_name, doc_content) in [
        ("06_REGULATORY_OBJECTIVES_ASSESSMENT.md", reg_md),
        ("STPA_MATRIX.md", stpa_md),
    ] {
        for expected in [
            "Service Tier",
            "SIL-3",
            "OBJ-01",
            "Deterministic rollback under arena exhaustion",
        ] {
            assert!(
                doc_content.contains(expected),
                "{} must contain AST-derived regulatory mapping substring {:?}:\n{}",
                doc_name,
                expected,
                doc_content
            );
        }
        assert!(
            !doc_content.contains("OSO-") && !doc_content.contains("| OSO ID |"),
            "{} must not contain hardcoded drone SORA OSO rows:\n{}",
            doc_name,
            doc_content
        );
        for unmodeled_obj in [
            "OBJ-04", "OBJ-05", "OBJ-06", "OBJ-07", "OBJ-08", "OBJ-09", "OBJ-10",
        ] {
            assert!(
                !doc_content.contains(unmodeled_obj),
                "{} must not fabricate unmodeled objective row {:?} beyond declared AST items:\n{}",
                doc_name,
                unmodeled_obj,
                doc_content
            );
        }
    }
}

/// Executes Issue #695 checks verifying RTA proofs, SLDV obligations, and regulatory objectives derive from AST.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all AST-derived proof and regulatory checks pass.
///
/// # Complexity
/// * Time: $O(A \cdot L)$ over the 10 generated safety artifacts. Space: $O(A \cdot L)$.
///
/// # Failure Modes
/// * Propagates missing-key errors or panics with assertion diagnostics on regression.
pub fn run_issue_695_ast_derived_rta_sldv_reg_checks() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = build_issue_695_query_safety_pkg();
    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    assert_zero_uav_physics(&suite);

    let rta_md = suite
        .get("07_RTA_ARCHITECTURE.md")
        .ok_or("Missing 07_RTA_ARCHITECTURE.md in transpiled safety suite")?;
    let stpa_md = suite
        .get("STPA_MATRIX.md")
        .ok_or("Missing STPA_MATRIX.md in transpiled safety suite")?;

    for (doc_name, doc_content) in [
        ("07_RTA_ARCHITECTURE.md", rta_md.as_str()),
        ("STPA_MATRIX.md", stpa_md.as_str()),
    ] {
        assert!(
            doc_content.contains("$$\n\\begin{aligned}")
                && doc_content.contains("\\end{aligned}\n$$"),
            "{} must emit isolated $$\\n\\begin{{aligned}} .. \\end{{aligned}}\\n$$ display math blocks:\n{}",
            doc_name,
            doc_content
        );
        assert!(
            doc_content.contains("HeadroomMargin") || doc_content.contains("HEADROOMMARGIN"),
            "{} must reference AST CalcDef HeadroomMargin:\n{}",
            doc_name,
            doc_content
        );
        assert!(
            doc_content.contains("LatencyCeiling") || doc_content.contains("LATENCYCEILING"),
            "{} must reference AST ConstraintDef LatencyCeiling:\n{}",
            doc_name,
            doc_content
        );
    }

    let sldv_m = suite
        .get("SLDV_FORMAL_PROOFS.m")
        .ok_or("Missing SLDV_FORMAL_PROOFS.m in transpiled safety suite")?;
    assert!(
        sldv_m.contains("sldv.assert(") && sldv_m.contains("LATENCYCEILING"),
        "SLDV_FORMAL_PROOFS.m must derive sldv.assert( proof obligations referencing LATENCYCEILING from AST ConstraintDef:\n{}",
        sldv_m
    );

    let reg_md = suite
        .get("06_REGULATORY_OBJECTIVES_ASSESSMENT.md")
        .ok_or("Missing 06_REGULATORY_OBJECTIVES_ASSESSMENT.md in transpiled safety suite")?;
    assert_issue_695_regulatory_docs(reg_md, stpa_md);
    Ok(())
}

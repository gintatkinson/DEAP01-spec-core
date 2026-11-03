//! Reproducer test helpers for GitHub Issue #697 (`compile-sysml` UCA/loss scenario/hazard log AST derivation).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_f_tests.rs` (refs #697).
//!
//! ## 2. Invariants & Mathematical Properties
//! - `expand_cartesian_stpa` resolves UCA `controller` to the declared AST `PartDef`
//!   (`TransactionSupervisor`) or enclosing `PackageDef` (`SysML_Model`) and never emits
//!   `DEAPCompilerSystem` or `PENDING_PARAMETER`.
//! - `transpile_safety_suite` synthesizes Class-1/Class-2 loss scenarios (`03_LOSS_SCENARIOS.md`
//!   and `STPA_MATRIX.md`) and `H-1`/`H-2` hazard log rows (`HAZARD_LOG.md`) purely from the AST
//!   with zero skeleton placeholders across both synthetic fixtures and `.pipeline/upstream/schema/model.sysml`.
//!
//! ## 3. Preconditions
//! - `.pipeline/upstream/schema/model.sysml` exists relative to `CARGO_MANIFEST_DIR`.
//!
//! ## 4. Postconditions
//! - All UCA, loss scenario, hazard log, and upstream schema assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(U + A \cdot L)$ over UCA count $U$ and safety artifacts $A = 10$.
//! - Space Complexity: $O(U + A \cdot L)$ for in-memory AST and safety suite buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `expand_cartesian_stpa`, `transpile_safety_suite`, and `parse_sysml`.
//! - State: Pure in-memory evaluation and read-only upstream model load.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaimed at scope exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-STPA-AST-DERIVED-UCA-LOSS-SCENARIOS], [REQ-STPA-UPSTREAM-MODEL-ZERO-PENDING-PARAMETER] | SysML v2: VerificationCaseDef(reproducer_f_tests)

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use deap_core::sysml_cst::{
    ActionDef, AttributeDef, CalcDef, ConstraintDef, PackageDef, PartDef, PortDef, RequirementDef,
    StateDef,
};

fn build_issue_697_fixture_a_pkg() -> PackageDef {
    let supervisor = PartDef {
        name: "TransactionSupervisor".to_string(),
        ports: vec![
            PortDef { name: "cmdIngress".to_string(), ..Default::default() },
            PortDef { name: "ackEgress".to_string(), ..Default::default() },
        ],
        states: vec![
            StateDef { name: "Standby".to_string(), ..Default::default() },
            StateDef { name: "Committing".to_string(), ..Default::default() },
        ],
        attributes: vec![
            AttributeDef { name: "maxCommitJitterMs".to_string(), type_name: "Integer".to_string(), default_value: Some("15".to_string()), doc: None },
            AttributeDef { name: "IntegrityLevel_Tier".to_string(), type_name: "String".to_string(), default_value: Some("SIL-3".to_string()), doc: None },
        ],
        ..Default::default()
    };

    PackageDef {
        name: "SysML_Model".to_string(),
        part_defs: vec![supervisor],
        action_defs: vec![
            ActionDef { name: "PrepareWalBatch".to_string(), ..Default::default() },
            ActionDef { name: "AbortStaleEpoch".to_string(), ..Default::default() },
        ],
        constraint_defs: vec![
            ConstraintDef {
                name: "WalCommitBound".to_string(),
                expression: "commitJitterMs <= 15".to_string(),
                is_assertion: true,
                parameters: vec!["unmatchedParam: Integer".to_string()],
                ..Default::default()
            },
            ConstraintDef { name: "QuorumGate".to_string(), expression: String::new(), ..Default::default() },
        ],
        calc_defs: vec![CalcDef {
            name: "EpochHeadroom".to_string(),
            expression: None,
            in_params: vec![AttributeDef { name: "windowTicks".to_string(), type_name: "Integer".to_string(), default_value: None, doc: None }],
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "ReqAtomicWal".to_string(),
            req_id: "REQ-WAL-01".to_string(),
            text: "Atomic WAL durability".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn assert_issue_697_fixture_a_suite(
    suite: &BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(suite.len(), 10, "transpile_safety_suite must emit 10 safety artifacts");
    let forbidden_placeholders = [
        "PENDING_PARAMETER",
        "DEAPCompilerSystem",
        "Loss scenario skeleton",
        "Skeleton loss entry",
        "Compound hazard skeleton",
    ];
    for (artifact_name, content) in suite {
        for forbidden in forbidden_placeholders {
            assert!(
                !content.contains(forbidden),
                "Safety artifact {} must contain zero placeholder {:?}:\n{}",
                artifact_name,
                forbidden,
                content
            );
        }
    }

    let loss_md = suite.get("03_LOSS_SCENARIOS.md").ok_or("Missing 03_LOSS_SCENARIOS.md")?;
    let stpa_md = suite.get("STPA_MATRIX.md").ok_or("Missing STPA_MATRIX.md")?;
    for (doc_name, doc_content) in [("03_LOSS_SCENARIOS.md", loss_md.as_str()), ("STPA_MATRIX.md", stpa_md.as_str())] {
        for required in ["Class-1", "Class-2", "TransactionSupervisor", "WalCommitBound", "SC-001"] {
            assert!(doc_content.contains(required), "{} missing {:?}:\n{}", doc_name, required, doc_content);
        }
        assert!(
            (doc_content.contains("cmdIngress") || doc_content.contains("ackEgress"))
                && (doc_content.contains("Standby") || doc_content.contains("Committing")),
            "{} must reference AST PortDef and StateDef in loss scenarios:\n{}",
            doc_name,
            doc_content
        );
    }

    let hazard_log_md = suite.get("HAZARD_LOG.md").ok_or("Missing HAZARD_LOG.md")?;
    for row_id in ["H-1", "H-2"] {
        assert!(
            hazard_log_md.lines().any(|line| line.starts_with('|') && line.contains(row_id)),
            "HAZARD_LOG.md must contain an {:?} table row:\n{}",
            row_id,
            hazard_log_md
        );
    }
    assert!(
        hazard_log_md.contains("Resolution Authority") && !hazard_log_md.contains("PENDING_PARAMETER"),
        "HAZARD_LOG.md must include a 'Resolution Authority' entry without 'PENDING_PARAMETER':\n{}",
        hazard_log_md
    );
    Ok(())
}

fn assert_issue_697_fixture_b_minimal() {
    let minimal_pkg = PackageDef {
        name: "SysML_Model".to_string(),
        part_defs: Vec::new(),
        attribute_defs: Vec::new(),
        action_defs: vec![ActionDef {
            name: "DispatchFrame".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let minimal_ucas = compile_sysml::stpa::uca::expand_cartesian_stpa(&minimal_pkg);
    assert!(
        !minimal_ucas.is_empty(),
        "expand_cartesian_stpa on minimal PackageDef with 1 ActionDef must return non-empty UCAs"
    );
    for uca in &minimal_ucas {
        assert_eq!(
            uca.controller, "SysML_Model",
            "Minimal PackageDef without PartDefs must assign controller == 'SysML_Model': {:?}",
            uca
        );
        assert!(
            !uca.controller.contains("DEAPCompilerSystem"),
            "Minimal PackageDef UCA controller must never contain 'DEAPCompilerSystem': {:?}",
            uca
        );
    }

    let minimal_suite =
        compile_sysml::stpa::safety_suite::transpile_safety_suite(&minimal_pkg, None);
    assert_eq!(minimal_suite.len(), 10);
    for (artifact_name, content) in &minimal_suite {
        assert!(
            !content.contains("PENDING_PARAMETER") && !content.contains("DEAPCompilerSystem"),
            "Minimal PackageDef safety artifact {} must contain zero 'PENDING_PARAMETER' or 'DEAPCompilerSystem':\n{}",
            artifact_name,
            content
        );
    }
}

/// Executes Issue #697 checks for UCA controller resolution, Class-1/Class-2 loss scenarios, and hazard log rows.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when both Fixture A and Fixture B pass.
///
/// # Complexity
/// * Time: $O(U + A \cdot L)$. Space: $O(U + A \cdot L)$.
///
/// # Failure Modes
/// * Propagates missing-key errors or panics with diagnostic context on placeholder regression.
pub fn run_issue_697_uca_loss_and_hazard_log_checks() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = build_issue_697_fixture_a_pkg();
    let ucas = compile_sysml::stpa::uca::expand_cartesian_stpa(&pkg);
    assert_eq!(
        ucas.len(),
        8,
        "expand_cartesian_stpa must return 8 UCAs for 2 package-level ActionDefs, got {}: {:?}",
        ucas.len(),
        ucas
    );
    for uca in &ucas {
        assert_eq!(
            uca.controller, "TransactionSupervisor",
            "UCA controller must resolve to AST PartDef 'TransactionSupervisor': {:?}",
            uca
        );
        assert!(
            !uca.controller.contains("DEAPCompilerSystem")
                && !uca.severity.contains("PENDING_PARAMETER")
                && !uca.sail.contains("PENDING_PARAMETER")
                && uca.sail.contains("SIL-3"),
            "UCA must derive SIL-3 without PENDING_PARAMETER or DEAPCompilerSystem: {:?}",
            uca
        );
    }

    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    assert_issue_697_fixture_a_suite(&suite)?;
    assert_issue_697_fixture_b_minimal();
    Ok(())
}

/// Executes Issue #697 checks verifying `.pipeline/upstream/schema/model.sysml` transpiles with zero placeholders.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when upstream `model.sysml` emits 10 clean artifacts.
///
/// # Complexity
/// * Time: $O(|S| + A \cdot L)$. Space: $O(|S| + A \cdot L)$.
///
/// # Failure Modes
/// * Propagates file read or SysML parse errors via `Result`.
pub fn run_issue_697_upstream_model_zero_pending_checks() -> Result<(), Box<dyn std::error::Error>>
{
    let model_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.pipeline/upstream/schema/model.sysml");
    let model_src = fs::read_to_string(&model_path)?;
    let pkg = compile_sysml::parse_sysml(&model_src)
        .map_err(|err| format!("Failed to parse {}: {:?}", model_path.display(), err))?;

    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    assert_eq!(
        suite.len(),
        10,
        "transpile_safety_suite on upstream model.sysml package {} must emit 10 safety artifacts, got {}: {:?}",
        pkg.name,
        suite.len(),
        suite.keys().collect::<Vec<_>>()
    );

    for (artifact_name, content) in &suite {
        assert!(
            !content.contains("PENDING_PARAMETER") && !content.contains("DEAPCompilerSystem"),
            "Upstream model.sysml safety artifact {} (package {}) must contain zero 'PENDING_PARAMETER' or 'DEAPCompilerSystem':\n{}",
            artifact_name,
            pkg.name,
            content
        );
    }
    Ok(())
}

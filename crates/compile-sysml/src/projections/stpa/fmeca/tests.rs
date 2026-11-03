#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Unit Verification Suite for FMECA Criticality Matrix Generation and Negation Filtering.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! verification objectives for module `crates/compile-sysml/src/stpa/fmeca/tests.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Verifies `>= 15` FMECA row coverage, `rpn == severity * occurrence * detection` conservation,
//!   4-dimension completeness, and exclusion of attribute-negated and `!Ident`-negated AST components.
//!
//! ## 3. Preconditions
//! - Invoked by the Rust test harness over synthetic in-memory `PackageDef` fixtures.
//!
//! ## 4. Postconditions
//! - Returns `Result<(), Box<dyn std::error::Error>>` with zero filesystem side effects.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) bounded over small fixed-size test fixtures.
//! - Space Complexity: O(1) bounded in-memory row vectors.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Exercises `generate_fmeca_matrix(&pkg, None)` directly.
//! - State: Isolated per-test fixtures with zero shared state.
//! - Action: Zero `.unwrap()`, `.expect()`, or unchecked slice indexing.
//! - Resource: Reclaimed immediately on test completion.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-FMECA-TESTS] | SysML v2: VerificationCaseDef(fmeca_tests)

use deap_core::sysml_cst::{PackageDef, PartDef};

use super::generate_fmeca_matrix;

#[test]
fn test_fmeca_matrix_generation_and_rpn() -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef::default();
    let p1 = PartDef {
        name: "CoreStructuralFrame".to_string(),
        ..Default::default()
    };
    let p2 = PartDef {
        name: "PrimaryComputeSubsystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(p1);
    pkg.part_defs.push(p2);

    let rows = generate_fmeca_matrix(&pkg, None);

    // 2 parts * 4 dimensions = 8 initial rows, padded to >= 15
    assert!(rows.len() >= 15, "Expected >= 15 rows, got {}", rows.len());

    for row in &rows {
        assert!(row.severity >= 1 && row.severity <= 10);
        assert!(row.occurrence >= 1 && row.occurrence <= 10);
        assert!(row.detection >= 1 && row.detection <= 10);
        assert_eq!(row.rpn, row.severity * row.occurrence * row.detection);
        assert!(row.id.starts_with("FMECA-"));
        assert!(!row.component.is_empty());
        assert!(!row.failure_mode.is_empty());
    }

    // Verify all 4 dimensions are represented
    let dims: Vec<_> = rows.iter().map(|r| r.dimension.as_str()).collect();
    assert!(dims.contains(&"Interface"));
    assert!(dims.contains(&"State"));
    assert!(dims.contains(&"Action"));
    assert!(dims.contains(&"Resource"));
    Ok(())
}

#[test]
fn test_fmeca_matrix_excludes_negated_components() -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef::default();
    let p1 = PartDef {
        name: "PrimaryComputeSubsystem".to_string(),
        ..Default::default()
    };
    let p2 = PartDef {
        name: "AuxiliaryBackupUnit".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(p1);
    pkg.part_defs.push(p2);

    // Explicit negative constraint: backupEnabled = 0; backupSystem = "No";
    pkg.attribute_defs.push(deap_core::sysml_cst::AttributeDef {
        name: "backupEnabled".to_string(),
        default_value: Some("0".to_string()),
        ..Default::default()
    });
    pkg.attribute_defs.push(deap_core::sysml_cst::AttributeDef {
        name: "backupSystem".to_string(),
        default_value: Some("\"No\"".to_string()),
        ..Default::default()
    });

    let rows = generate_fmeca_matrix(&pkg, None);
    let components: Vec<_> = rows.iter().map(|r| r.component.as_str()).collect();
    assert!(components.contains(&"PrimaryComputeSubsystem"));
    assert!(
        !components.contains(&"AuxiliaryBackupUnit"),
        "Negated AuxiliaryBackupUnit must NOT be present in FMECA rows"
    );
    Ok(())
}

#[test]
fn test_arch_07_fmeca_matrix_precomputes_bang_negated_components_in_btreeset(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef::default();
    pkg.part_defs.push(PartDef {
        name: "PrimaryComputeSubsystem".to_string(),
        ..Default::default()
    });
    pkg.part_defs.push(PartDef {
        name: "AuxiliaryBackupUnit".to_string(),
        ..Default::default()
    });
    pkg.constraint_defs
        .push(deap_core::sysml_cst::ConstraintDef {
            name: "ExcludeBackupUnit".to_string(),
            expression: "!AuxiliaryBackupUnit".to_string(),
            is_assertion: true,
            ..Default::default()
        });

    let rows = generate_fmeca_matrix(&pkg, None);
    let components: Vec<_> = rows.iter().map(|r| r.component.as_str()).collect();
    assert!(components.contains(&"PrimaryComputeSubsystem"));
    assert!(
        !components.contains(&"AuxiliaryBackupUnit"),
        "Direct !AuxiliaryBackupUnit negation must be precomputed into BTreeSet and excluded from FMECA matrix"
    );
    Ok(())
}

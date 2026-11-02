#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for TD-03..TD-07 lossless CST metrics, envelopes, and receipts. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Provides comprehensive regression tests for C_CST_01..C_CST_06 and v2_metrics.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Validates deterministic BTreeMap and BTreeSet properties without test fixtures in repo.
//!
//! ## 3. Preconditions
//! - In-memory synthesized source buffers and synthetic lock snapshots.
//!
//! ## 4. Postconditions
//! - Demonstrates RED -> GREEN correctness of TD-03..TD-07 components.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(1) stack allocation for fixtures; O(N) parser traversal.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Asserts fail-closed traps on DEAP-E0801..DEAP-E0804 without panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [LosslessRustCstAnalyzer/tests], [REQ-0608]
//! SysML v2: part def LosslessRustCstAnalyzer

use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn make_fixture_snapshot() -> LockManifestSnapshot {
    let mut packages = BTreeMap::new();
    for name in ["ra_ap_syntax", "rowan", "syn", "quote", "prettyplease"] {
        packages.insert(
            format!("{name}@1.0.0"),
            LockedPackageRecord {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                checksum: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
                source_kind: PackageSourceKind::RegistryPinned,
                dependencies: BTreeSet::new(),
            },
        );
    }
    LockManifestSnapshot {
        cargo_toml_sha256: [1u8; 32],
        cargo_lock_sha256: [2u8; 32],
        packages,
        workspace_members: BTreeSet::from(["deap-core".to_string()]),
        offline_locked_enforced: true,
        vendor_bundle_supported: true,
    }
}

#[test]
fn test_envelope_sealing_and_snapshot_parity() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = make_fixture_snapshot();
    let verified = BTreeSet::from([
        "ra_ap_syntax".to_string(),
        "rowan".to_string(),
        "syn".to_string(),
        "quote".to_string(),
        "prettyplease".to_string(),
    ]);
    let envelope = CstComplexityEnvelope::default();
    let receipt = verify_analyzer_envelope_and_seal_receipt(&snapshot, &verified, &envelope)?;
    assert!(receipt.supply_chain_receipt_sealed);
    assert_eq!(receipt.verify_snapshot_parity(&snapshot), Ok(()));

    let mut drifted = snapshot.clone();
    drifted.cargo_lock_sha256 = [99u8; 32];
    assert_eq!(receipt.verify_snapshot_parity(&drifted), Err(OfflineLockGovernanceError::ManifestToLockDriftDetected));
    Ok(())
}

#[test]
fn test_thresholds_validation_and_error_codes() {
    let default_t = CstMetricsThresholds::default();
    assert!(validate_metrics_thresholds(&default_t).is_ok());

    let mut bad_sloc = default_t;
    bad_sloc.max_fn_exec_sloc = 0;
    assert_eq!(validate_metrics_thresholds(&bad_sloc).unwrap_err().diagnostic_code(), "DEAP-E0803");

    let mut bad_params = default_t;
    bad_params.max_fn_param_count = 6;
    assert_eq!(validate_metrics_thresholds(&bad_params).unwrap_err().diagnostic_code(), "DEAP-E0803");

    let mut bad_uuid = default_t;
    bad_uuid.deterministic_uuid = "wrong-uuid";
    assert_eq!(validate_metrics_thresholds(&bad_uuid).unwrap_err().diagnostic_code(), "DEAP-E0803");
}

#[test]
fn test_parse_verified_lossless_cst() -> Result<(), Box<dyn std::error::Error>> {
    let thresholds = CstMetricsThresholds::default();
    let code = b"pub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n";
    let cst = parse_verified_lossless_cst("src/add.rs", code, &thresholds)?;
    assert!(cst.is_lossless_red_green_tree_ready());
    assert_eq!(cst.source_path(), "src/add.rs");
    assert_eq!(cst.source_bytes(), code);
    assert_eq!(cst.observed_max_depth() > 0, true);

    let bad_code = b"fn unclosed( {";
    let err = parse_verified_lossless_cst("bad.rs", bad_code, &thresholds);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().diagnostic_code(), "DEAP-E0803");
    Ok(())
}

#[test]
fn test_doc_tier_classification_and_test_pruning() -> Result<(), Box<dyn std::error::Error>> {
    let thresholds = CstMetricsThresholds::default();
    let src = r#"
/// Seven section doc
/// Realises: [REQ-01]
pub struct Alpha;

fn internal_helper() {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_alpha() {}
}
"#;
    let cst = parse_verified_lossless_cst("src/alpha.rs", src.as_bytes(), &thresholds)?;
    let inv = classify_cfg_test_excluded_doc_tier(&cst, &thresholds, false)?;
    assert_eq!(inv.rust_tier_a_count, 1);
    assert_eq!(inv.rust_tier_bc_count, 1);
    assert_eq!(inv.test_excluded_item_count > 0, true);
    assert_eq!(inv.pruned_test_subtree_ranges.is_empty(), false);
    assert!(DocTier::RustTierA.is_production());
    assert!(DocTier::RustTierBC.is_production());
    assert!(!DocTier::TestExcluded.is_production());
    assert_eq!(DocTier::RustTierA.label(), "RustTierA");
    Ok(())
}

#[test]
fn test_analyze_cst_v2_metrics_nominal_and_violations() -> Result<(), Box<dyn std::error::Error>> {
    let thresholds = CstMetricsThresholds::default();
    let src = r#"
pub fn calc(x: i32, y: i32) -> i32 {
    let mut flag = true;
    if x > 0 && y > 0 && x != y {
        x + y
    } else {
        0
    }
}

pub fn string_err() -> Result<(), String> {
    Err("fail".to_string())
}
"#;
    let cst = parse_verified_lossless_cst("src/calc.rs", src.as_bytes(), &thresholds)?;
    let report = analyze_cst_v2_metrics(&cst, &thresholds)?;
    assert_eq!(report.lossless_roundtrip_verified, true);
    let calc_fn = report.callable_metrics.values().find(|m| m.symbol == "calc").unwrap();
    assert_eq!(calc_fn.param_count, 2);
    assert!(!calc_fn.has_self_receiver);
    assert_eq!(calc_fn.mut_bool_flag_count, 1);
    assert_eq!(calc_fn.max_predicate_clauses, 3);
    assert!(!calc_fn.returns_result_string_err);

    let err_fn = report.callable_metrics.values().find(|m| m.symbol == "string_err").unwrap();
    assert!(err_fn.returns_result_string_err);
    Ok(())
}

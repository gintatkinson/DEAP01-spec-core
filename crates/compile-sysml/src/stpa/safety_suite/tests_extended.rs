#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Extended unit verification suite for atomic safety emission, Check-23 hazard wording, and UTF-8 boundary handling.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! verification objectives for module `crates/compile-sysml/src/stpa/safety_suite/tests_extended.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Verifies atomic file emission leaving zero `.tmp.*` files, `"Enforced by "` wording in `HAZARD_LOG.md`,
//!   and multi-byte UTF-8 boundary safety in `collect_parameter_tokens` and `compile_stpa_to_constraints_filtered`.
//!
//! ## 3. Preconditions
//! - Invoked by the Rust test harness using PID-scoped temporary directories and UTF-8 test fixtures.
//!
//! ## 4. Postconditions
//! - Cleans up temporary test directories and returns `Result<(), Box<dyn std::error::Error>>`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) bounded over small fixed-size fixtures.
//! - Space Complexity: O(1) bounded in-memory and temporary file artifacts.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Exercises `emit_safety_suite`, `transpile_safety_suite`, and `collect_parameter_tokens`.
//! - State: Uses isolated PID-scoped temporary directory paths cleaned up on exit.
//! - Action: Zero `.unwrap()`, `.expect()`, or unchecked slice indexing.
//! - Resource: Reclaimed immediately upon test completion.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-TESTS] | SysML v2: VerificationCaseDef(safety_suite_tests_extended)

#[cfg(test)]
mod extended_suite_tests {
    use super::super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    /// Verifies `ARCH-04`: `emit_safety_suite` delegates persistence to `write_atomic`
    /// and leaves zero `.tmp.*` files in the output directory.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-EMIT-ATOMIC]
    #[test]
    fn test_arch_04_emit_safety_suite_writes_atomically() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut pkg = PackageDef {
            name: "AtomicSafetyPkg".to_string(),
            ..Default::default()
        };
        pkg.part_defs.push(PartDef {
            name: "SafetyController".to_string(),
            ..Default::default()
        });

        let out_dir = std::env::temp_dir().join(format!(
            "deap_test_emit_safety_suite_atomic_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&out_dir);
        emit_safety_suite(&pkg, &out_dir, None)?;

        let mut count = 0usize;
        for entry in fs::read_dir(&out_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let fname = entry.file_name().to_string_lossy().to_string();
            assert!(
                !fname.contains(".tmp."),
                "No temporary file may remain after emit_safety_suite: {}",
                fname
            );
            count = count.saturating_add(1);
        }
        assert_eq!(
            count, 10,
            "emit_safety_suite must write all 10 safety suite artifacts"
        );

        let _ = fs::remove_dir_all(&out_dir);
        Ok(())
    }

    /// Verifies that `HAZARD_LOG.md` renders safety constraint bindings using `"Enforced by "`
    /// rather than `"Controlled via "` so Check 23 protocol extraction never collides with `SC-*` IDs.
    /// Realises: [REQ-STPA-HAZARD-LOG-ENFORCED-BY]
    #[test]
    fn test_hazard_log_uses_enforced_by_without_controlled_via(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut fallback_pkg = PackageDef {
            name: "BaselineEnvelopePkg".to_string(),
            ..Default::default()
        };
        let fallback_suite = transpile_safety_suite(&fallback_pkg, None);
        let fallback_log = fallback_suite
            .get("HAZARD_LOG.md")
            .ok_or("HAZARD_LOG.md missing in fallback suite")?;
        assert!(fallback_log.contains("Enforced by "));
        assert!(!fallback_log.contains("Controlled via "));

        let mut controller = PartDef {
            name: "SupervisoryController".to_string(),
            ..Default::default()
        };
        controller.actions.push(ActionDef {
            name: "DispatchCommand".to_string(),
            ..Default::default()
        });
        fallback_pkg.part_defs.push(controller);

        let uca_suite = transpile_safety_suite(&fallback_pkg, None);
        let uca_log = uca_suite
            .get("HAZARD_LOG.md")
            .ok_or("HAZARD_LOG.md missing in UCA suite")?;
        assert!(uca_log.contains("Enforced by "));
        assert!(!uca_log.contains("Controlled via "));
        Ok(())
    }

    /// Verifies multi-byte UTF-8 boundary safety in `collect_parameter_tokens`, `transpile_safety_suite`,
    /// and `compile_stpa_to_constraints_filtered`, including non-digit `SC-` prefixes and malformed rows.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-UTF8-BOUNDARIES]
    #[test]
    fn test_utf8_boundaries_cycled_lookups_and_checked_sc_id_slicing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef {
            name: "航空安全_Sys".to_string(),
            constraint_defs: vec![ConstraintDef {
                name: "ThermalAndJitterEnvelope".to_string(),
                expression: "/* Δ */ThermalMargin <= 85; ΩCycleJitter <= 12; 🛰RecoveryBound = 3"
                    .to_string(),
                is_assertion: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        let tokens = collect_parameter_tokens(&pkg);
        assert_eq!(tokens.get("ThermalMargin").map(String::as_str), Some("85"));
        assert_eq!(tokens.get("CycleJitter").map(String::as_str), Some("12"));
        assert_eq!(tokens.get("RecoveryBound").map(String::as_str), Some("3"));

        let empty_member_suite = transpile_safety_suite(&pkg, None);
        assert_eq!(empty_member_suite.len(), 10);

        let mut utf8_ctrl = PartDef {
            name: "軌道制御_Controller".to_string(),
            ..Default::default()
        };
        utf8_ctrl.actions.push(ActionDef {
            name: "ExecuteManeuver".to_string(),
            ..Default::default()
        });
        pkg.part_defs.push(utf8_ctrl);
        let populated_suite = transpile_safety_suite(&pkg, None);
        assert_eq!(populated_suite.len(), 10);

        let stpa_md = r#"|
|||
| UCA-001 | 軌道制御_Controller | ThrustGate | Providing | Δv ≤ 60 m/s | H-1 | SC-none / 🛰ΔΩ SC-042 (SIL-3) | High | SIL-3 |
| UCA-002 | 軌道制御_Controller | BadConstraint | Providing | Δv | H-2 | SC- | High | SIL-3 |
| FMECA-001 | 軌道制御_Controller | SensorDrift_Ω | Loss of Δv | 8 | 2 | 2 | 32 | Interlock | SSOT |
| FMECA-999 | MissingColsOnly |
"#;
        let compiled =
            compile_stpa_to_constraints_filtered(stpa_md, "UTF8ConstraintsPkg", None, None);
        let names: Vec<&str> = compiled
            .constraint_defs
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "SC_042_ThrustGate",
                "FMECA_001_軌道制御_Controller_Integrity"
            ]
        );
        Ok(())
    }
}

//! Unit verification suite for STPA 10-pillar safety artifact transpilation, atomic emission, and constraint compilation.
//!
//! Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-TESTS]
//! Contract: Verifies 10-pillar domain-agnostic artifact generation, UCA/FMECA Markdown-to-SysML constraint compilation, negative-invariant phantom-component suppression, atomic file emission, Check-23 hazard-log wording, and multi-byte UTF-8 boundary handling.
//! Bounds & Faults: Executes deterministic in-memory and PID-scoped temporary-directory assertions returning `Result<(), Box<dyn std::error::Error>>` with zero panics or unchecked slice indexing.

#[cfg(test)]
mod suite_tests {
    use super::super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    /// Verifies that `transpile_safety_suite` emits the exact 10 domain-agnostic safety artifacts,
    /// renders isolated KaTeX aligned blocks from AST `ConstraintDef` nodes, and builds closed-loop
    /// STPA control structure topology diagrams.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-ALL-10-ARTIFACTS]
    #[test]
    fn test_transpile_safety_suite_produces_all_10_artifacts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef {
            name: "TestSystem".to_string(),
            ..Default::default()
        };

        let mut p = PartDef {
            name: "PrimaryController".to_string(),
            ..Default::default()
        };
        let act = ActionDef {
            name: "ExecuteProcess".to_string(),
            ..Default::default()
        };
        p.actions.push(act);
        pkg.part_defs.push(p);
        pkg.constraint_defs.push(ConstraintDef {
            name: "ExecutionDeadlineBound".to_string(),
            expression: "cycleLatencyMs <= 25".to_string(),
            ..Default::default()
        });

        let suite = transpile_safety_suite(&pkg, None);
        assert_eq!(suite.len(), 10);
        assert!(suite.contains_key("01_LOSSES_HAZARDS_TOPOLOGY.md"));
        assert!(suite.contains_key("02_UCA_COMBINATORIAL_MATRIX.md"));
        assert!(suite.contains_key("03_LOSS_SCENARIOS.md"));
        assert!(suite.contains_key("04_SAFETY_CONSTRAINTS.md"));
        assert!(suite.contains_key("05_FMECA_MATRIX.md"));
        assert!(suite.contains_key("06_REGULATORY_OBJECTIVES_ASSESSMENT.md"));
        assert!(!suite.contains_key("06_SORA_SAIL_ASSESSMENT.md"));
        assert!(suite.contains_key("07_RTA_ARCHITECTURE.md"));
        assert!(suite.contains_key("STPA_MATRIX.md"));
        assert!(suite.contains_key("HAZARD_LOG.md"));
        assert!(suite.contains_key("SLDV_FORMAL_PROOFS.m"));

        // Verify RTA architecture contains KaTeX aligned math from AST ConstraintDef
        let rta = suite
            .get("07_RTA_ARCHITECTURE.md")
            .ok_or("expected value is absent")?;
        assert!(rta.contains("$$\n\\begin{aligned}"));
        assert!(rta.contains("\\end{aligned}\n$$"));
        assert!(rta.contains("ExecutionDeadlineBound"));

        // Verify closed-loop STPA control structure topology
        let topo = suite
            .get("01_LOSSES_HAZARDS_TOPOLOGY.md")
            .ok_or("expected value is absent")?;
        assert!(topo.contains("subgraph \"Hierarchical Safety Control Structure\""));
        assert!(
            topo.contains("CompilerController[\"PrimaryController<br/>(Supervisory Controller)\"]")
        );
        assert!(topo.contains("ActuatorEngine[\"TestSystem_Actuator<br/>(Actuator Engine)\"]"));
        assert!(
            topo.contains("ControlledProcess[\"TestSystem_Process<br/>(Execution Environment)\"]")
        );
        assert!(topo.contains("DiagnosticSensors[\"TestSystem_Sensors<br/>(Diagnostic Sensors)\"]"));
        assert!(topo.contains("DiagnosticSensors -->|\"sensor feedback: diagnostic_errors - verification_metrics\"| CompilerController"));
        Ok(())
    }

    /// Verifies that `compile_stpa_to_constraints` parses UCA Markdown tables with the
    /// `Integrity Level` column into formal SysML `ConstraintDef` assertions.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-COMPILE-CONSTRAINTS]
    #[test]
    fn test_compile_stpa_markdown_to_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let md = "\n| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | Integrity Level |\n| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n| UCA-001 | PrimaryProcessor | ComputeControlOutput | Not providing | Operational | H-1 | SC-001 | High | SIL-3 |\n";
        let pkg = compile_stpa_to_constraints(md, "SafetyConstraintsPackage");
        assert_eq!(pkg.name, "SafetyConstraintsPackage");
        assert_eq!(pkg.constraint_defs.len(), 1);
        assert_eq!(
            pkg.constraint_defs
                .first()
                .ok_or("missing first element of pkg.constraint_defs")?
                .name,
            "SC_001_ComputeControlOutput"
        );
        assert!(
            pkg.constraint_defs
                .first()
                .ok_or("missing first element of pkg.constraint_defs")?
                .is_assertion
        );
        Ok(())
    }

    /// Verifies that `compile_stpa_to_constraints` suppresses FMECA constraint synthesis for
    /// components negated by negative safety model invariants.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-NEGATED-COMPONENT-FILTER]
    #[test]
    fn test_issue_249_compile_stpa_rejects_phantom_component_when_negated(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let md = r#"
| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | Integrity Level |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| UCA-001 | PrimaryProcessor | ComputeControlOutput | Not providing | Operational | H-1 | SC-001 | High | SIL-3 |

Negative safety model invariant: backupEnabled = 0, backupSystem = "No"

| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FMECA-001 | PrimaryProcessor | ActuationFault | Loss of Control Output | 8 | 2 | 2 | 32 | Hardware interlock | SSOT |
| FMECA-002 | AuxiliaryBackupUnit | ActuationFault | Loss of Redundancy | 9 | 2 | 3 | 54 | Hardware interlock | Derived |
"#;
        let pkg = compile_stpa_to_constraints(md, "SafetyConstraintsPackage");
        let con_names: Vec<String> = pkg.constraint_defs.iter().map(|c| c.name.clone()).collect();
        assert!(
            con_names.iter().any(|c| c.contains("PrimaryProcessor")),
            "PrimaryProcessor constraint must be present"
        );
        assert!(
            !con_names.iter().any(|c| c.contains("AuxiliaryBackupUnit")),
            "AuxiliaryBackupUnit constraint must NOT be synthesized when negated by negative AST constraints"
        );
        Ok(())
    }

    /// Verifies that `compile_stpa_to_constraints_filtered` restricts FMECA integrity constraints
    /// exclusively to components present in `valid_parts`.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-VALID-PARTS-FILTER]
    #[test]
    fn test_issue_249_compile_stpa_filtered_by_valid_parts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let md = r#"
| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FMECA-001 | PrimaryProcessor | ActuationFault | Loss of Control Output | 8 | 2 | 2 | 32 | Hardware interlock | SSOT |
| FMECA-002 | PhantomSubsystem | Ghost Logic Fail | None | 5 | 2 | 2 | 20 | None | Derived |
"#;
        let mut valid_parts = BTreeSet::new();
        valid_parts.insert("PrimaryProcessor".to_string());

        let pkg = compile_stpa_to_constraints_filtered(
            md,
            "SafetyConstraintsPackage",
            Some(&valid_parts),
            None,
        );
        let con_names: Vec<String> = pkg.constraint_defs.iter().map(|c| c.name.clone()).collect();
        assert!(
            con_names.iter().any(|c| c.contains("PrimaryProcessor")),
            "PrimaryProcessor constraint must be present"
        );
        assert!(
            !con_names.iter().any(|c| c.contains("PhantomSubsystem")),
            "PhantomSubsystem constraint must NOT be synthesized when missing from valid_parts"
        );
        Ok(())
    }

    /// Verifies that `STPA_MATRIX.md` integrates all unified STPA pillars including domain-neutral
    /// regulatory objectives and integrity level mappings and ASTM F3269-17 RTA synthesis hooks.
    /// Realises: [REQ-STPA-SAFETY-SUITE-TEST-UNIFIED-8-PILLARS]
    #[test]
    fn test_defect_333_stpa_matrix_embodies_unified_8_pillars(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef {
            name: "CoreControlSystemSafety".to_string(),
            ..Default::default()
        };

        let mut p = PartDef {
            name: "SupervisoryController".to_string(),
            ..Default::default()
        };
        let act = ActionDef {
            name: "ExecuteProcess".to_string(),
            ..Default::default()
        };
        p.actions.push(act);
        pkg.part_defs.push(p);

        let suite = transpile_safety_suite(&pkg, None);
        let stpa = suite
            .get("STPA_MATRIX.md")
            .ok_or("expected value is absent")?;

        assert!(
            stpa.contains("System Losses"),
            "STPA_MATRIX.md must contain Pillar 1 (System Losses)"
        );
        assert!(
            stpa.contains("System Hazards"),
            "STPA_MATRIX.md must contain Pillar 2 (System Hazards)"
        );
        assert!(
            stpa.contains("Control Structure Topology")
                || stpa.contains("Hierarchical Safety Control Structure"),
            "STPA_MATRIX.md must contain Pillar 3 (Topology)"
        );
        assert!(
            stpa.contains("Unsafe Control Actions") || stpa.contains("UCA ID"),
            "STPA_MATRIX.md must contain Pillar 4 (UCAs)"
        );
        assert!(
            stpa.contains("Loss Scenarios"),
            "STPA_MATRIX.md must contain Pillar 5 (Loss Scenarios)"
        );
        assert!(
            stpa.contains("Safety Constraints"),
            "STPA_MATRIX.md must contain Pillar 6 (Safety Constraints)"
        );
        assert!(
            stpa.contains("FMECA Criticality Matrix") || stpa.contains("FMECA"),
            "STPA_MATRIX.md must contain Pillar 7 (FMECA Criticality Matrix)"
        );
        assert!(
            stpa.contains("## 8. Domain Regulatory Objectives & Integrity Level Mapping"),
            "STPA_MATRIX.md must contain Pillar 8 (Domain Regulatory Objectives & Integrity Level Mapping)"
        );
        assert!(
            stpa.contains("ASTM F3269-17") || stpa.contains("Stateflow"),
            "STPA_MATRIX.md must contain ASTM F3269-17 RTA / Stateflow hooks"
        );
        Ok(())
    }

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
        assert!(
            fallback_log.contains("Enforced by "),
            "Fallback HAZARD_LOG.md must use 'Enforced by ', got:\n{}",
            fallback_log
        );
        assert!(
            !fallback_log.contains("Controlled via "),
            "Fallback HAZARD_LOG.md must not contain 'Controlled via '"
        );

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
        assert!(
            uca_log.contains("Enforced by "),
            "UCA HAZARD_LOG.md must use 'Enforced by ', got:\n{}",
            uca_log
        );
        assert!(
            !uca_log.contains("Controlled via "),
            "UCA HAZARD_LOG.md must not contain 'Controlled via '"
        );
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
        assert_eq!(
            tokens.get("ThermalMargin").map(String::as_str),
            Some("85"),
            "ASCII-delimited parameter following UTF-8 comment must extract cleanly"
        );
        assert_eq!(
            tokens.get("CycleJitter").map(String::as_str),
            Some("12"),
            "Identifier immediately preceded by 2-byte UTF-8 codepoint must not slice mid-codepoint"
        );
        assert_eq!(
            tokens.get("RecoveryBound").map(String::as_str),
            Some("3"),
            "Identifier immediately preceded by 4-byte UTF-8 codepoint must not slice mid-codepoint"
        );

        // Exercise empty-member and populated multi-byte UTF-8 safety-suite rendering without panics.
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

        // Exercise checked SC-### scanning across multi-byte UTF-8 prose, non-digit SC- tokens, and short pipe lines.
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

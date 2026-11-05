#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Unit verification suite for STPA 10-pillar safety artifact transpilation and constraint compilation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! verification objectives for module `crates/compile-sysml/src/stpa/safety_suite/tests.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Verifies 10-pillar domain-agnostic artifact generation, UCA/FMECA Markdown-to-SysML constraint compilation,
//!   negative-invariant phantom-component suppression, and unified 8-pillar `STPA_MATRIX.md` coverage.
//!
//! ## 3. Preconditions
//! - Invoked by the Rust test harness over synthetic in-memory `PackageDef` fixtures.
//!
//! ## 4. Postconditions
//! - Returns `Result<(), Box<dyn std::error::Error>>` with zero filesystem side effects.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) bounded over small fixed-size test fixtures.
//! - Space Complexity: O(1) bounded in-memory artifact maps.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Exercises `transpile_safety_suite`, `compile_stpa_to_constraints`, and `compile_stpa_to_constraints_filtered`.
//! - State: Isolated per-test fixtures with zero shared state.
//! - Action: Zero `.unwrap()`, `.expect()`, or unchecked slice indexing.
//! - Resource: Reclaimed immediately on test completion.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-TESTS] | SysML v2: VerificationCaseDef(safety_suite_tests)

#[cfg(test)]
mod suite_tests {
    use super::super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    /// Verifies that `transpile_safety_suite` emits the exact 10 domain-agnostic safety artifacts,
    /// renders isolated KaTeX aligned blocks, and builds closed-loop STPA control structure topology diagrams.
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
        p.actions.push(ActionDef {
            name: "ExecuteProcess".to_string(),
            ..Default::default()
        });
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

        let rta = suite
            .get("07_RTA_ARCHITECTURE.md")
            .ok_or("expected value is absent")?;
        assert!(rta.contains("$$\n\\begin{aligned}"));
        assert!(rta.contains("\\end{aligned}\n$$"));
        assert!(rta.contains("ExecutionDeadlineBound"));

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
        p.actions.push(ActionDef {
            name: "ExecuteProcess".to_string(),
            ..Default::default()
        });
        pkg.part_defs.push(p);

        let suite = transpile_safety_suite(&pkg, None);
        let stpa = suite
            .get("STPA_MATRIX.md")
            .ok_or("expected value is absent")?;

        assert!(stpa.contains("System Losses"));
        assert!(stpa.contains("System Hazards"));
        assert!(
            stpa.contains("Control Structure Topology")
                || stpa.contains("Hierarchical Safety Control Structure")
        );
        assert!(stpa.contains("Unsafe Control Actions") || stpa.contains("UCA ID"));
        assert!(stpa.contains("Loss Scenarios"));
        assert!(stpa.contains("Safety Constraints"));
        assert!(stpa.contains("FMECA Criticality Matrix") || stpa.contains("FMECA"));
        assert!(stpa.contains("## 8. Domain Regulatory Objectives & Integrity Level Mapping"));
        assert!(stpa.contains("ASTM F3269-17") || stpa.contains("Stateflow"));
        Ok(())
    }
}

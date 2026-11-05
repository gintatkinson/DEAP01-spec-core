#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for SysML v2 canonical serialization of packages, ports, requirements, constraints, imports, and states.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification objectives.
//! ## 2. Invariants & Mathematical Properties
//! Verifies deterministic formatting and parser round-trip equivalence across core SysML v2 definitions.
//! ## 3. Preconditions
//! Executed under standard `cargo test` harness.
//! ## 4. Postconditions
//! Asserts exact textual output and lossless AST round-tripping for `RequirementDef` and `StateDef`.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over synthetic AST fixtures. Space Complexity: O(N) serialized string output.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Zero `.unwrap()` or `.expect()` calls across all test functions.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-REQUIREMENTDEF-TO-SYSML, REQ-SEMANTIC-SERIALIZER-STATEDEF-SUBSTATES-ROUNDTRIP]

use super::serializer::SysmlSerializable;
use deap_core::sysml_ast::{
    AttributeDef, ConstraintDef, ImportDef, PackageDef, PartDef, PortDef, RequirementDef, StateDef,
    TransitionDef,
};

#[test]
fn test_serialize_package_roundtrip() {
    let pkg = PackageDef {
        name: "AutonomousPlatform".to_string(),
        doc: Some("Simple autonomous platform".to_string()),
        attribute_defs: vec![AttributeDef {
            name: "massKg".to_string(),
            type_name: "Real".to_string(),
            default_value: Some("15.5".to_string()),
            doc: None,
        }],
        part_defs: vec![PartDef {
            name: "StructuralFrame".to_string(),
            ports: vec![PortDef {
                name: "bus".to_string(),
                direction: "out".to_string(),
                type_name: "CAN".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let sysml_text = pkg.to_sysml(0);
    assert!(sysml_text.contains("package AutonomousPlatform {"));
    assert!(sysml_text.contains("doc /* Simple autonomous platform */"));
    assert!(sysml_text.contains("attribute massKg : Real = 15.5;"));
    assert!(sysml_text.contains("part def StructuralFrame {"));
    assert!(sysml_text.contains("port out bus : CAN;"));
}

#[test]
fn test_serialize_port_def_dataport_no_empty_body() {
    let port = PortDef {
        name: "rules_out".to_string(),
        type_name: "CompilerRulePort".to_string(),
        direction: "out".to_string(),
        port_category: "DataPort".to_string(),
        ..Default::default()
    };
    let sysml = port.to_sysml(4);
    assert_eq!(sysml, "    port out rules_out : CompilerRulePort;");
}

#[test]
fn test_serialize_port_def_with_body_standard_syntax() {
    let port = PortDef {
        name: "rules_out".to_string(),
        type_name: "CompilerRulePort".to_string(),
        direction: "out".to_string(),
        protocol_family: "CAN".to_string(),
        ..Default::default()
    };
    let sysml = port.to_sysml(4);
    assert!(sysml.starts_with("    port out rules_out : CompilerRulePort {"));
    assert!(sysml.contains("attribute protocol_family : String = \"CAN\";"));
    assert!(sysml.ends_with("    }"));
}

#[test]
fn test_serialize_requirement_def_with_attributes() {
    let req = RequirementDef {
        name: "REQ_0001_Test".to_string(),
        req_id: "REQ-0001".to_string(),
        text: "The system shall perform verification.".to_string(),
        attributes: vec![
            AttributeDef {
                name: "uuidv5".to_string(),
                type_name: "String".to_string(),
                default_value: Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"".to_string()),
                doc: None,
            },
            AttributeDef {
                name: "ac_01_test".to_string(),
                type_name: "String".to_string(),
                default_value: Some("\"Given: X. When: Y. Then: Z.\"".to_string()),
                doc: None,
            },
        ],
        verified_by: vec!["AC_01_Test".to_string()],
        satisfied_by: vec!["TestEngine".to_string()],
        ..Default::default()
    };

    let sysml = req.to_sysml(0);
    assert!(sysml.contains("requirement def REQ_0001_Test {"));
    assert!(sysml.contains("    id = \"REQ-0001\";"));
    assert!(sysml.contains("    text = \"The system shall perform verification.\";"));
    assert!(
        sysml.contains("    attribute uuidv5 : String = \"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\";")
    );
    assert!(sysml.contains("    attribute ac_01_test : String = \"Given: X. When: Y. Then: Z.\";"));
    assert!(sysml.contains("    verify by AC_01_Test;"));
    assert!(sysml.contains("    satisfy by TestEngine;"));
}

#[test]
fn test_serialize_requirement_def_with_assumes_requires_derived_from(
) -> Result<(), Box<dyn std::error::Error>> {
    let req = RequirementDef {
        name: "REQ_0032_Test".to_string(),
        req_id: "REQ-0032".to_string(),
        text: "Test text".to_string(),
        assumes: vec!["ValidInput".to_string()],
        requires: vec!["Invariant_Alpha".to_string(), "Invariant_Beta".to_string()],
        derived_from: vec!["REQ_0031_Parent".to_string()],
        derives: vec!["REQ_0033_Child".to_string()],
        verified_by: vec!["AC_01".to_string()],
        satisfied_by: vec!["SubsystemEngine".to_string()],
        ..Default::default()
    };
    let pkg = PackageDef {
        name: "ReqTracePkg".to_string(),
        requirement_defs: vec![req],
        ..Default::default()
    };
    let sysml = pkg.to_sysml(0);
    assert!(sysml.contains("        assume ValidInput;"));
    assert!(sysml.contains("        require Invariant_Alpha;"));
    assert!(sysml.contains("        require Invariant_Beta;"));
    assert!(sysml.contains("        derived from REQ_0031_Parent;"));
    assert!(sysml.contains("        derives REQ_0033_Child;"));
    let reparsed = crate::parser::grammar::SysmlParser::parse_source(&sysml, "ReqTracePkg")?;
    assert_eq!(reparsed, pkg);
    Ok(())
}

#[test]
fn test_serialize_constraint_def_single_line_when_empty_body_and_params() {
    let c1 = ConstraintDef {
        name: "StrictTotalOrdering".to_string(),
        is_assertion: true,
        ..Default::default()
    };
    assert_eq!(c1.to_sysml(4), "    assert constraint StrictTotalOrdering;");

    let c2 = ConstraintDef {
        name: "Invariant_Alpha".to_string(),
        is_assertion: false,
        ..Default::default()
    };
    assert_eq!(c2.to_sysml(0), "constraint def Invariant_Alpha;");
}

#[test]
fn test_serialize_requirement_def_with_encapsulated_constraints() {
    let req = RequirementDef {
        name: "REQ_0001_Test".to_string(),
        req_id: "REQ-0001".to_string(),
        text: "Testing constraints inside requirement def.".to_string(),
        constraints: vec![
            ConstraintDef {
                name: "Invariant_Alpha".to_string(),
                doc: Some("Formula Alpha".to_string()),
                is_assertion: false,
                ..Default::default()
            },
            ConstraintDef {
                name: "Invariant_Beta".to_string(),
                expression: "x > 0".to_string(),
                is_assertion: true,
                ..Default::default()
            },
        ],
        requires: vec!["Invariant_Alpha".to_string()],
        ..Default::default()
    };
    let sysml = req.to_sysml(0);
    assert!(sysml.contains("requirement def REQ_0001_Test {"));
    assert!(sysml.contains("    doc /* Formula Alpha */\n    constraint def Invariant_Alpha;"));
    assert!(sysml.contains("    assert constraint Invariant_Beta {\n        x > 0;\n    }"));
    assert!(sysml.contains("    require Invariant_Alpha;"));
}

#[test]
fn test_serialize_package_with_imports() {
    let pkg = PackageDef {
        name: "SubsystemWithImports".to_string(),
        imports: vec![
            ImportDef {
                path: "ScalarValues".to_string(),
                is_wildcard: true,
                is_recursive: false,
                doc: None,
            },
            ImportDef {
                path: "Base::Types::*".to_string(),
                is_wildcard: true,
                is_recursive: false,
                doc: None,
            },
            ImportDef {
                path: "NestedSys".to_string(),
                is_wildcard: true,
                is_recursive: true,
                doc: None,
            },
            ImportDef {
                path: "Specific::Item".to_string(),
                is_wildcard: false,
                is_recursive: false,
                doc: Some("Specific imported element".to_string()),
            },
        ],
        ..Default::default()
    };

    let sysml_text = pkg.to_sysml(0);
    assert!(sysml_text.contains("package SubsystemWithImports {"));
    assert!(sysml_text.contains("    import ScalarValues::*;"));
    assert!(sysml_text.contains("    import Base::Types::*;"));
    assert!(sysml_text.contains("    import NestedSys::**;"));
    assert!(
        sysml_text.contains("    doc /* Specific imported element */\n    import Specific::Item;")
    );
}

#[test]
fn test_serialize_part_def_and_package_def_with_satisfied_requirements() {
    let part = PartDef {
        name: "DisplayUnit".to_string(),
        satisfied_requirements: vec!["REQ-UI-01".to_string(), "REQ-UI-02".to_string()],
        ..Default::default()
    };
    let part_sysml = part.to_sysml(0);
    assert!(part_sysml.contains("    satisfy requirement REQ-UI-01;"));
    assert!(part_sysml.contains("    satisfy requirement REQ-UI-02;"));

    let pkg = PackageDef {
        name: "DisplayPackage".to_string(),
        satisfied_requirements: vec!["REQ-SYS-001".to_string()],
        part_defs: vec![part],
        ..Default::default()
    };
    let pkg_sysml = pkg.to_sysml(0);
    assert!(pkg_sysml.contains("    satisfy requirement REQ-SYS-001;"));
    assert!(pkg_sysml.contains("        satisfy requirement REQ-UI-01;"));
    assert!(pkg_sysml.contains("        satisfy requirement REQ-UI-02;"));
}

fn assert_flight_state_machine_ast(parsed: &PackageDef) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(parsed.state_defs.len(), 1);
    if let Some(root_state) = parsed.state_defs.first() {
        assert_eq!(
            root_state.doc.as_deref(),
            Some("Root flight mode state machine")
        );
        assert_eq!(root_state.sub_states.len(), 2);
        assert_eq!(
            root_state.sub_states.first().and_then(|s| s.doc.as_deref()),
            Some("Ground idle leaf state")
        );
        let s1 = root_state.sub_states.get(1).ok_or("missing sub_state 1")?;
        assert_eq!(s1.doc.as_deref(), Some("Airborne composite state"));
        assert_eq!(
            s1.transitions.first().and_then(|t| t.doc.as_deref()),
            Some("Climb to cruise transition")
        );
    }
    Ok(())
}

#[test]
fn test_serialize_state_def_sub_states_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let src = "\
doc /* Flight state machine package */
package FlightStateMachine {
    state def FlightMode {
        entry initFlight;
        do monitorFlight;
        exit shutdownFlight;
        doc = \"Root flight mode state machine\";
        doc /* Ground idle leaf state */
        state GroundIdle;
        doc /* Airborne composite state */
        state def Airborne {
            entry engageAutopilot;
            do trackWaypoints;
            exit disengageAutopilot;
            state Climb;
            state Cruise;
            doc /* Climb to cruise transition */
            transition Climb_to_Cruise first Climb accept AltitudeReached if climbComplete do levelOff then Cruise;
        }
        transition GroundIdle_to_Airborne first GroundIdle then Airborne;
    }
}";
    let parsed = crate::parser::grammar::SysmlParser::parse_source(src, "FlightStateMachine")?;
    assert_flight_state_machine_ast(&parsed)?;

    let serialized = parsed.to_sysml(0);
    assert!(serialized.contains("state GroundIdle;"));
    assert!(serialized.contains("state def Airborne {"));
    assert!(serialized.contains("state Climb;"));
    assert!(serialized.contains("state Cruise;"));
    assert!(serialized.contains(
        "transition Climb_to_Cruise first Climb accept AltitudeReached if climbComplete do levelOff then Cruise;"
    ));
    assert!(serialized.contains("transition GroundIdle_to_Airborne first GroundIdle then Airborne;"));

    let reparsed =
        crate::parser::grammar::SysmlParser::parse_source(&serialized, "FlightStateMachine")?;
    assert_eq!(reparsed, parsed);

    let empty_transition_state = StateDef {
        name: "MinimalState".to_string(),
        transitions: vec![TransitionDef::default()],
        ..Default::default()
    };
    let empty_serialized = empty_transition_state.to_sysml(0);
    assert!(empty_serialized.contains("    transition;"));
    assert!(!empty_serialized.contains("transition transition;"));
    Ok(())
}

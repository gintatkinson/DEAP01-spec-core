//! Core semantic validator unit tests for connections, safety annotations, cross-package imports, and nested subpackage ports.
//!
//! Safety & Traceability: Realises: [REQ-SEMANTIC-VALIDATOR-TESTS]
//! Contract: Verifies valid and inverted port connections, dangling `@SafetyRealises` references, cross-package import resolution, and nested subpackage dangling port detection.
//! Bounds & Faults: Executes deterministic in-memory AST validation assertions with zero panics.

use super::*;

#[test]
fn test_valid_connection_passes() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "TestSys".to_string(),
        part_defs: vec![
            PartDef {
                name: "Sensor".to_string(),
                ports: vec![PortDef {
                    name: "out_data".to_string(),
                    direction: "out".to_string(),
                    type_name: "RawData".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            PartDef {
                name: "Computer".to_string(),
                ports: vec![PortDef {
                    name: "in_data".to_string(),
                    direction: "in".to_string(),
                    type_name: "RawData".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        connection_defs: vec![ConnectionDef {
            name: "SensorToComputer".to_string(),
            source_port: "Sensor.out_data".to_string(),
            target_port: "Computer.in_data".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(!validator.has_errors(), "Expected no errors, got: {:?}", validator.errors());
    Ok(())
}

#[test]
fn test_mismatched_connection_direction_fails() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "TestSys".to_string(),
        part_defs: vec![
            PartDef {
                name: "Sensor".to_string(),
                ports: vec![PortDef {
                    name: "in_data".to_string(),
                    direction: "in".to_string(),
                    type_name: "RawData".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            PartDef {
                name: "Computer".to_string(),
                ports: vec![PortDef {
                    name: "out_data".to_string(),
                    direction: "out".to_string(),
                    type_name: "RawData".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        connection_defs: vec![ConnectionDef {
            name: "InvertedFlow".to_string(),
            source_port: "Sensor.in_data".to_string(),
            target_port: "Computer.out_data".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(validator.has_errors());
    let errors = validator.errors();
    assert_eq!(errors.len(), 1);
    assert!(errors.first().ok_or("missing error")?.message.contains("Inverted flow direction"));
    Ok(())
}

#[test]
fn test_dangling_safety_realises_fails() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "SafetySys".to_string(),
        requirement_defs: vec![RequirementDef {
            name: "Req_Safety_1".to_string(),
            text: "Shall maintain velocity limit. @SafetyRealises(UCA-99)".to_string(),
            ..Default::default()
        }],
        constraint_defs: vec![ConstraintDef {
            name: "UCA_01_Overspeed".to_string(),
            expression: "v <= 60.0".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(validator.has_errors());
    let errors = validator.errors();
    assert!(errors
        .first()
        .ok_or("missing error")?
        .message
        .contains("references undefined constraint/UCA 'UCA-99'"));
    Ok(())
}

#[test]
fn test_validate_part_defs_with_cross_package_import() -> Result<(), Box<dyn std::error::Error>> {
    let pkg_a = PackageDef {
        name: "Subsystem_1".to_string(),
        part_defs: vec![PartDef {
            name: "SystemVisionEngine".to_string(),
            is_def: true,
            ..Default::default()
        }],
        ..Default::default()
    };
    let pkg_b = PackageDef {
        name: "Subsystem_2".to_string(),
        imports: vec![ImportDef {
            path: "Subsystem_1".to_string(),
            is_wildcard: true,
            ..Default::default()
        }],
        part_defs: vec![PartDef {
            name: "v".to_string(),
            is_def: false,
            type_name: Some("SystemVisionEngine".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let root = PackageDef {
        name: "".to_string(),
        packages: vec![pkg_a, pkg_b],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&root);
    validator.validate();
    assert!(
        !validator.has_errors(),
        "Expected no validation errors with imported type, got: {:?}",
        validator.errors()
    );
    Ok(())
}

#[test]
fn test_validate_part_defs_missing_type_fails() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "Subsystem_2".to_string(),
        part_defs: vec![PartDef {
            name: "v".to_string(),
            is_def: false,
            type_name: Some("SystemVisionEngine".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(validator.has_errors());
    let errors = validator.errors();
    assert!(errors
        .iter()
        .any(|e| e.message.contains("references undefined type 'SystemVisionEngine'")));
    Ok(())
}

#[test]
fn test_validate_requirements_with_cross_package_import() -> Result<(), Box<dyn std::error::Error>>
{
    let pkg_a = PackageDef {
        name: "Subsystem_1".to_string(),
        part_defs: vec![PartDef {
            name: "SafetyController".to_string(),
            is_def: true,
            ..Default::default()
        }],
        ..Default::default()
    };
    let pkg_b = PackageDef {
        name: "Subsystem_2".to_string(),
        imports: vec![ImportDef {
            path: "Subsystem_1".to_string(),
            is_wildcard: true,
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "Req_Ctrl".to_string(),
            text: "Shall engage safety controller.".to_string(),
            satisfied_by: vec!["SafetyController".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let root = PackageDef {
        name: "".to_string(),
        packages: vec![pkg_a, pkg_b],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&root);
    validator.validate();
    assert!(
        !validator.has_errors(),
        "Expected no validation errors, got: {:?}",
        validator.errors()
    );
    Ok(())
}

#[test]
fn test_subpackage_connections_validated_for_dangling_ports(
) -> Result<(), Box<dyn std::error::Error>> {
    let subpkg = PackageDef {
        name: "NestedSubsystem".to_string(),
        part_defs: vec![PartDef {
            name: "Worker".to_string(),
            ports: vec![PortDef {
                name: "out_stream".to_string(),
                direction: "out".to_string(),
                type_name: "StreamData".to_string(),
                ..Default::default()
            }],
            connections: vec![ConnectionDef {
                name: "DanglingSubpartConn".to_string(),
                source_port: "out_stream".to_string(),
                target_port: "Worker.undefined_target_port".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        connection_defs: vec![ConnectionDef {
            name: "DanglingPkgConn".to_string(),
            source_port: "Worker.out_stream".to_string(),
            target_port: "UndefinedPart.in_stream".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let root = PackageDef {
        name: "RootSystem".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&root);
    validator.validate();
    assert!(validator.has_errors());
    let errors = validator.errors();
    assert!(errors.iter().any(|e| e
        .message
        .contains("references undefined target port 'Worker.undefined_target_port'")));
    assert!(errors.iter().any(|e| e
        .message
        .contains("references undefined target port 'UndefinedPart.in_stream'")));
    Ok(())
}

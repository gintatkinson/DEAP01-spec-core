//! Core fixture, contextual identifier, assertion constraint, and array-suffix grammar unit tests.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies deterministic AST parsing across ground-truth system fixtures, contextual SysML v2 / KerML keywords in identifier position, part-level assertion constraints, and `Type[]` multiplicity suffixes.
//! Bounds & Faults: Executes deterministic in-memory parsing assertions returning `Result<(), Box<dyn std::error::Error>>` with zero panics or external side effects.

#[cfg(test)]
mod core_fixture_tests {
    use super::super::*;

    #[test]
    fn test_parse_ground_truth_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = r#"
package SystemModel_SSOT {
    doc /* SSOT for Generic System Architecture and Safety Model */

    attribute channelCount : Integer = 4;
    attribute topologyConfiguration : String = "Symmetric";
    attribute primaryOperatingLimit : Real = 12.0;
    attribute secondaryOperatingLimit : Real = 12.0;

    part def StructuralFrame {
        attribute nominalMass : Real = 25.0;
    }

    part def PrimaryControlUnit {
        port command_bus : CommandBus;
        port telemetry : TelemetryLink;
    }
}
"#;
        let pkg = SysmlParser::parse_source(fixture, "TestModel")?;

        assert_eq!(pkg.name, "SystemModel_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("SSOT for Generic System Architecture and Safety Model")
        );
        assert_eq!(pkg.attribute_defs.len(), 4);
        let channel_count = pkg.attribute_defs.first().ok_or("missing attribute 0")?;
        assert_eq!(channel_count.name, "channelCount");
        assert_eq!(channel_count.type_name, "Integer");
        assert_eq!(channel_count.default_value.as_deref(), Some("4"));

        assert_eq!(pkg.part_defs.len(), 2);
        let frame = pkg.part_defs.first().ok_or("missing part 0")?;
        assert_eq!(frame.name, "StructuralFrame");
        assert_eq!(frame.attributes.len(), 1);
        let mass = frame
            .attributes
            .first()
            .ok_or("missing frame attribute 0")?;
        assert_eq!(mass.name, "nominalMass");
        assert_eq!(mass.default_value.as_deref(), Some("25.0"));

        let control_unit = pkg.part_defs.get(1).ok_or("missing part 1")?;
        assert_eq!(control_unit.name, "PrimaryControlUnit");
        assert_eq!(control_unit.ports.len(), 2);
        let command_bus = control_unit.ports.first().ok_or("missing port 0")?;
        assert_eq!(command_bus.name, "command_bus");
        assert_eq!(command_bus.type_name, "CommandBus");
        let telemetry = control_unit.ports.get(1).ok_or("missing port 1")?;
        assert_eq!(telemetry.name, "telemetry");
        assert_eq!(telemetry.type_name, "TelemetryLink");
        Ok(())
    }

    #[test]
    fn test_parse_grounded_limits_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture =
            include_str!("../../../../../tests/fixtures/safety/grounded_limits_model.sysml");
        let pkg = SysmlParser::parse_source(fixture, "TestLimits")?;

        assert_eq!(pkg.name, "GroundedLimits_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("Ground truth limits for flight envelope verification")
        );
        assert_eq!(pkg.attribute_defs.len(), 5);
        assert_eq!(
            pkg.attribute_defs
                .first()
                .ok_or("missing first element of pkg.attribute_defs")?
                .name,
            "catapultPressureLimitBar"
        );
        assert_eq!(
            pkg.attribute_defs
                .first()
                .ok_or("missing first element of pkg.attribute_defs")?
                .default_value
                .as_deref(),
            Some("15.0")
        );
        Ok(())
    }

    #[test]
    fn test_parse_complex_system_model() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package FlightControl_SSOT {
    doc /* Primary Flight Control and Mission Computer Architecture */

    attribute maxRollRateDps : Real = 45.0;
    attribute maxPitchAngleDeg : Real = 30.0;

    part def NavigationSubsystem {
        doc /* GPS, IMU and Sensor Fusion Unit */
        port out nav_solution : NavData;
        port in correction_stream : ~NavCorrection;

        action def ComputeNavigationState {
            in rawImu : ImuPacket;
            out filteredState : NavState;
        }

        state def NavFilterState {
            entry InitINS;
            do RunEKF;
            exit ResetCovariance;
        }
    }

    part def ActuatorController {
        port in servo_cmd : ~ServoCommand;
        port out status : ActuatorStatus;

        assert constraint DeflectionBounds {
            deflectionDeg >= -25.0 && deflectionDeg <= 25.0;
        }
    }

    connection def NavToActuatorBus {
        doc /* High speed deterministic bus connection */
        connect NavigationSubsystem.nav_solution to ActuatorController.servo_cmd;
        attribute protocol : String = "CANopen";
        attribute latency_ms : Real = 2.5;
    }

    requirement def Req_Nav_001 {
        id = "REQ-NAV-001";
        text = "Navigation system shall compute attitude at minimum 100 Hz.";
        require latency <= 10.0;
        verify by HardwareInLoopTest;
        satisfy by NavigationSubsystem;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.name, "FlightControl_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("Primary Flight Control and Mission Computer Architecture")
        );
        assert_eq!(pkg.attribute_defs.len(), 2);
        assert_eq!(pkg.part_defs.len(), 2);

        let nav = pkg
            .part_defs
            .first()
            .ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(nav.name, "NavigationSubsystem");
        assert_eq!(nav.doc.as_deref(), Some("GPS, IMU and Sensor Fusion Unit"));
        assert_eq!(nav.ports.len(), 2);
        assert_eq!(
            nav.ports
                .first()
                .ok_or("missing first element of nav.ports")?
                .direction,
            "out"
        );
        assert_eq!(
            nav.ports
                .first()
                .ok_or("missing first element of nav.ports")?
                .name,
            "nav_solution"
        );
        assert_eq!(
            nav.ports
                .get(1)
                .ok_or("missing element 1 of nav.ports")?
                .direction,
            "in"
        );
        assert!(
            nav.ports
                .get(1)
                .ok_or("missing element 1 of nav.ports")?
                .is_conjugated
        );
        assert_eq!(
            nav.ports
                .get(1)
                .ok_or("missing element 1 of nav.ports")?
                .name,
            "correction_stream"
        );
        assert_eq!(nav.actions.len(), 1);
        assert_eq!(
            nav.actions
                .first()
                .ok_or("missing first element of nav.actions")?
                .name,
            "ComputeNavigationState"
        );
        assert_eq!(nav.states.len(), 1);
        assert_eq!(
            nav.states
                .first()
                .ok_or("missing first element of nav.states")?
                .name,
            "NavFilterState"
        );
        assert_eq!(
            nav.states
                .first()
                .ok_or("missing first element of nav.states")?
                .entry_action
                .as_deref(),
            Some("InitINS")
        );
        assert_eq!(
            nav.states
                .first()
                .ok_or("missing first element of nav.states")?
                .do_action
                .as_deref(),
            Some("RunEKF")
        );

        let actuator = pkg
            .part_defs
            .get(1)
            .ok_or("missing element 1 of pkg.part_defs")?;
        assert_eq!(actuator.name, "ActuatorController");
        assert_eq!(actuator.constraints.len(), 1);
        assert_eq!(
            actuator
                .constraints
                .first()
                .ok_or("missing first element of actuator.constraints")?
                .name,
            "DeflectionBounds"
        );
        assert!(
            actuator
                .constraints
                .first()
                .ok_or("missing first element of actuator.constraints")?
                .is_assertion
        );
        assert!(actuator
            .constraints
            .first()
            .ok_or("missing first element of actuator.constraints")?
            .expression
            .contains("deflectionDeg"));

        assert_eq!(pkg.connection_defs.len(), 1);
        let conn = pkg
            .connection_defs
            .first()
            .ok_or("missing first element of pkg.connection_defs")?;
        assert_eq!(conn.name, "NavToActuatorBus");
        assert_eq!(conn.source_port, "NavigationSubsystem.nav_solution");
        assert_eq!(conn.target_port, "ActuatorController.servo_cmd");
        assert_eq!(conn.source_part.as_deref(), Some("NavigationSubsystem"));
        assert_eq!(conn.target_part.as_deref(), Some("ActuatorController"));
        assert_eq!(conn.protocol.as_deref(), Some("CANopen"));
        assert_eq!(conn.latency_ms, Some(2.5));

        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg
            .requirement_defs
            .first()
            .ok_or("missing first element of pkg.requirement_defs")?;
        assert_eq!(req.name, "Req_Nav_001");
        assert_eq!(req.req_id, "REQ-NAV-001");
        assert_eq!(
            req.text,
            "Navigation system shall compute attitude at minimum 100 Hz."
        );
        assert_eq!(req.verified_by, vec!["HardwareInLoopTest"]);
        assert_eq!(req.satisfied_by, vec!["NavigationSubsystem"]);
        Ok(())
    }

    #[test]
    fn test_parse_contextual_keywords_as_identifiers() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package TestPkg {
    part def Component {
        attribute text : String = "hello";
        attribute doc : String = "world";
        attribute item : String = "val";
        attribute state : String = "ready";
        attribute action : String = "run";
        attribute port : String = "port1";
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.name, "TestPkg");
        let comp = pkg
            .part_defs
            .first()
            .ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(comp.name, "Component");
        assert_eq!(comp.attributes.len(), 6);
        assert_eq!(
            comp.attributes
                .first()
                .ok_or("missing first element of comp.attributes")?
                .name,
            "text"
        );
        assert_eq!(
            comp.attributes
                .get(1)
                .ok_or("missing element 1 of comp.attributes")?
                .name,
            "doc"
        );
        assert_eq!(
            comp.attributes
                .get(2)
                .ok_or("missing element 2 of comp.attributes")?
                .name,
            "item"
        );
        assert_eq!(
            comp.attributes
                .get(3)
                .ok_or("missing element 3 of comp.attributes")?
                .name,
            "state"
        );
        assert_eq!(
            comp.attributes
                .get(4)
                .ok_or("missing element 4 of comp.attributes")?
                .name,
            "action"
        );
        assert_eq!(
            comp.attributes
                .get(5)
                .ok_or("missing element 5 of comp.attributes")?
                .name,
            "port"
        );
        Ok(())
    }

    #[test]
    fn test_parse_part_with_assert_constraint() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package SubsystemPkg {
    part def Engine {
        assert constraint Invariant_Alpha;
        assert constraint Invariant_Beta;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.part_defs.len(), 1);
        let engine = pkg
            .part_defs
            .first()
            .ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(engine.constraints.len(), 2);
        assert_eq!(
            engine
                .constraints
                .first()
                .ok_or("missing first element of engine.constraints")?
                .name,
            "Invariant_Alpha"
        );
        assert!(
            engine
                .constraints
                .first()
                .ok_or("missing first element of engine.constraints")?
                .is_assertion
        );
        assert_eq!(
            engine
                .constraints
                .get(1)
                .ok_or("missing element 1 of engine.constraints")?
                .name,
            "Invariant_Beta"
        );
        assert!(
            engine
                .constraints
                .get(1)
                .ok_or("missing element 1 of engine.constraints")?
                .is_assertion
        );
        Ok(())
    }

    #[test]
    fn test_parse_attribute_decl_accepts_array_bracket_type_suffix() {
        let source = r#"
package ArrayModel {
    attribute labelSet : String[];
    attribute scalarValue : Integer = 3;
    part def Holder {
        attribute counts : Integer[];
        attribute weight : Real = 1.5;
    }
}
"#;
        let parsed = SysmlParser::parse_source(source, "ArrayModel");
        assert!(
            parsed.is_ok(),
            "Type[] attribute failed to parse: {:?}",
            parsed.err()
        );
        let pkg = parsed.unwrap_or_default();
        let label_set = pkg.attribute_defs.iter().find(|a| a.name == "labelSet");
        assert_eq!(label_set.map(|a| a.type_name.as_str()), Some("String[]"));
        assert_eq!(label_set.and_then(|a| a.default_value.clone()), None);
        let scalar = pkg.attribute_defs.iter().find(|a| a.name == "scalarValue");
        assert_eq!(scalar.map(|a| a.type_name.as_str()), Some("Integer"));
        let holder = pkg.part_defs.iter().find(|p| p.name == "Holder");
        let counts = holder.and_then(|p| p.attributes.iter().find(|a| a.name == "counts"));
        assert_eq!(counts.map(|a| a.type_name.as_str()), Some("Integer[]"));
        let weight = holder.and_then(|p| p.attributes.iter().find(|a| a.name == "weight"));
        assert_eq!(weight.map(|a| a.type_name.as_str()), Some("Real"));
    }

    #[test]
    fn test_parse_attribute_decl_rejects_unclosed_array_bracket() {
        let source = "package BadArray {\n    attribute labelSet : String[;\n}\n";
        let parsed = SysmlParser::parse_source(source, "BadArray");
        assert!(
            parsed.is_err(),
            "unclosed '[' must be a parse error, got {:?}",
            parsed.ok()
        );
    }
}

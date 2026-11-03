#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for SysML v2 bijective round-tripping, doc comment escaping, and depth-64 stack bounds.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification objectives.
//! ## 2. Invariants & Mathematical Properties
//! Verifies `parse(serialize(ast)) == ast` across use cases, ports, hazards, risks, connections, and deep AST trees.
//! ## 3. Preconditions
//! Executed under standard `cargo test` harness with bounded stack threads.
//! ## 4. Postconditions
//! Asserts lossless round-trip serialization and safe truncation at `MAX_AST_DEPTH` (64).
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over synthetic AST fixtures. Space Complexity: O(N) heap work-stack.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Zero `.unwrap()` or `.expect()` calls across all test functions.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-USECASEDEF-ROUNDTRIP, REQ-SEMANTIC-SERIALIZER-ROUNDTRIP-505]

use super::serializer::{format_doc_comment, SysmlSerializable, MAX_AST_DEPTH};
use deap_core::sysml_ast::{PackageDef, PartDef, StateDef, UseCaseDef};

fn assert_unquoted_use_case_clauses() {
    let unquoted_uc = UseCaseDef {
        name: "ExecuteContingencySortie".to_string(),
        subject: Some("Vehicle : AutonomousPlatform".to_string()),
        actors: vec!["MissionOperator".to_string()],
        objective: Some("Recover vehicle under degraded navigation".to_string()),
        preconditions: vec!["VehiclePreFlightCheckComplete".to_string(), "Navigation link degraded".to_string()],
        includes: vec!["use case ValidateFlightPlan".to_string(), "use case Verify inertial fallback".to_string()],
        extends: vec!["EmergencyAbortSortie".to_string(), "Execute emergency landing".to_string()],
        steps: vec!["Switch to inertial navigation".to_string(), "\"Return to recovery zone\"".to_string()],
        postconditions: vec!["TelemetryLogPersisted".to_string(), "Vehicle safely recovered".to_string()],
        ..Default::default()
    };
    let uc_sysml = unquoted_uc.to_sysml(0);
    assert!(uc_sysml.contains("objective \"Recover vehicle under degraded navigation\";"));
    assert!(uc_sysml.contains("precondition VehiclePreFlightCheckComplete;"));
    assert!(uc_sysml.contains("precondition \"Navigation link degraded\";"));
    assert!(uc_sysml.contains("include ValidateFlightPlan;"));
    assert!(uc_sysml.contains("include \"Verify inertial fallback\";"));
    assert!(uc_sysml.contains("extend EmergencyAbortSortie;"));
    assert!(uc_sysml.contains("extend \"Execute emergency landing\";"));
    assert!(uc_sysml.contains("step \"Switch to inertial navigation\";"));
    assert!(uc_sysml.contains("step \"Return to recovery zone\";"));
    assert!(uc_sysml.contains("postcondition TelemetryLogPersisted;"));
    assert!(uc_sysml.contains("postcondition \"Vehicle safely recovered\";"));
}

#[test]
fn test_serialize_use_case_def_full_fields_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let src = "\
package MissionUseCases {
    use case def ExecuteAutonomousSortie {
        subject Vehicle : AutonomousPlatform;
        actor MissionOperator;
        actor SafetySupervisor;
        objective \"Execute autonomous reconnaissance sortie safely\";
        precondition VehiclePreFlightCheckComplete;
        include ValidateFlightPlan;
        extend EmergencyAbortSortie;
        step \"Initialize navigation sensors\";
        step \"Follow mission waypoints\";
        postcondition TelemetryLogPersisted;
    }
}";
    let parsed = crate::parser::grammar::SysmlParser::parse_source(src, "MissionUseCases")?;
    assert_eq!(parsed.use_case_defs.len(), 1);
    if let Some(uc) = parsed.use_case_defs.first() {
        assert_eq!(uc.preconditions, vec!["VehiclePreFlightCheckComplete".to_string()]);
        assert_eq!(uc.includes, vec!["ValidateFlightPlan".to_string()]);
        assert_eq!(uc.extends, vec!["EmergencyAbortSortie".to_string()]);
        assert_eq!(uc.postconditions, vec!["TelemetryLogPersisted".to_string()]);
    }

    let serialized = parsed.to_sysml(0);
    assert!(serialized.contains("precondition VehiclePreFlightCheckComplete;"));
    assert!(serialized.contains("include ValidateFlightPlan;"));
    assert!(serialized.contains("extend EmergencyAbortSortie;"));
    assert!(serialized.contains("postcondition TelemetryLogPersisted;"));

    let reparsed = crate::parser::grammar::SysmlParser::parse_source(&serialized, "MissionUseCases")?;
    assert_eq!(reparsed, parsed);
    assert_unquoted_use_case_clauses();
    Ok(())
}

#[test]
fn test_format_doc_comment_escapes_nested_block_comment_open_and_close() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Some("Scans crates/*/src/**/*.rs and excludes crates/*/tests/*.rs".to_string());
    let formatted = format_doc_comment(&doc, 4);
    assert!(formatted.contains("crates/ * /src/ ** / *.rs"));
    assert!(formatted.contains("crates/ * /tests/ *.rs"));
    let sysml_src = format!("package BlueprintPkg {{\n{}    part def Guard;\n}}\n", formatted);
    let parsed = crate::parser::grammar::SysmlParser::parse_source(&sysml_src, "BlueprintPkg")?;
    assert_eq!(parsed.part_defs.len(), 1);
    Ok(())
}

#[test]
fn test_port_def_with_body_and_item_flow_syntax_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let src = "\
package PortPkg {
    port def TelemetryPort {
        attribute protocol_family : String = \"CAN_FD\";
        in item cmd : CommandFrame;
        out item status : StatusFrame;
        inout item diag : DiagFrame;
        flow inout bus : BusPayload;
    }
}";
    let parsed = crate::parser::grammar::SysmlParser::parse_source(src, "PortPkg")?;
    assert_eq!(parsed.port_defs.len(), 1);
    if let Some(port) = parsed.port_defs.first() {
        assert_eq!((port.name.as_str(), port.type_name.as_str(), port.item_flows.len()), ("TelemetryPort", "TelemetryPort", 4));
        let f0 = port.item_flows.first().ok_or("f0")?;
        assert_eq!((f0.direction.as_str(), f0.name.as_str()), ("in", "cmd"));
        let f1 = port.item_flows.get(1).ok_or("f1")?;
        assert_eq!((f1.direction.as_str(), f1.name.as_str()), ("out", "status"));
        let f2 = port.item_flows.get(2).ok_or("f2")?;
        assert_eq!((f2.direction.as_str(), f2.name.as_str()), ("inout", "diag"));
        let f3 = port.item_flows.get(3).ok_or("f3")?;
        assert_eq!((f3.direction.as_str(), f3.name.as_str()), ("inout", "bus"));
    }
    let serialized = parsed.to_sysml(0);
    assert!(serialized.contains("port def TelemetryPort {"));
    assert!(!serialized.contains("port inout TelemetryPort : TelemetryPort"));
    let reparsed = crate::parser::grammar::SysmlParser::parse_source(&serialized, "PortPkg")?;
    assert_eq!(reparsed.port_defs, parsed.port_defs);
    Ok(())
}

#[test]
fn test_use_case_def_include_and_extend_use_case_keyword_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let src = "\
package MissionUseCases {
    use case def ExecuteSortie {
        include use case ValidateFlightPlan;
        extend use case EmergencyAbortSortie;
    }
}";
    let parsed = crate::parser::grammar::SysmlParser::parse_source(src, "MissionUseCases")?;
    assert_eq!(parsed.use_case_defs.len(), 1);
    if let Some(uc) = parsed.use_case_defs.first() {
        assert_eq!(uc.includes, vec!["ValidateFlightPlan".to_string()]);
        assert_eq!(uc.extends, vec!["EmergencyAbortSortie".to_string()]);
    }
    let serialized = parsed.to_sysml(0);
    assert!(serialized.contains("include ValidateFlightPlan;") && serialized.contains("extend EmergencyAbortSortie;"));
    assert!(!serialized.contains("include use case") && !serialized.contains("extend use case"));
    let reparsed = crate::parser::grammar::SysmlParser::parse_source(&serialized, "MissionUseCases")?;
    assert_eq!(reparsed, parsed);
    Ok(())
}

fn assert_parsed_roundtrip_505_fields(parsed: &PackageDef) -> Result<(), Box<dyn std::error::Error>> {
    let act0 = parsed.action_defs.first().ok_or("act0")?;
    assert_eq!(act0.in_params.first().and_then(|p| p.default_value.as_deref()), Some("10.0"));
    assert_eq!(act0.out_params.first().and_then(|p| p.default_value.as_deref()), Some("0.0"));
    assert_eq!(act0.parameters.len(), 1);
    assert_eq!(parsed.connection_defs.first().ok_or("c0")?.latency_ms, Some(0.25));
    let h0 = parsed.hazard_defs.first().ok_or("h0")?;
    assert_eq!((h0.source_port.as_deref(), h0.target_port.as_deref()), (Some("Bus.pwrOut"), Some("Actuator.pwrIn")));
    let r0 = parsed.risk_defs.first().ok_or("r0")?;
    assert_eq!((r0.source_port.as_deref(), r0.target_port.as_deref()), (Some("Bus.pwrOut"), Some("Actuator.pwrIn")));
    let s0 = parsed.state_defs.first().ok_or("s0")?;
    assert_eq!(s0.doc.as_deref(), Some("Supervisor state property doc"));
    assert_eq!(s0.sub_states.first().ok_or("sub0")?.doc, None);
    Ok(())
}

#[test]
fn test_issue_505_hazard_risk_action_connection_state_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let src = "\
package Roundtrip505 {
    port def BusPort {
        attribute protocol_family : String = \"CAN_FD\";
        attribute voltage = \"5V\";
    }
    action def ComputeControl(in setpoint : Real = 10.0, out command : Real = 0.0) {
        attribute gain : Real = 0.25;
        perform ControlLoop;
        step ApplyPid;
    }
    connection def SensorLink {
        connect Sensor.outPort to Controller.inPort;
        attribute source_part : String = \"SensorNode\";
        attribute target_part : String = \"ControlNode\";
        attribute item_flow_ref : String = \"TelemetryFrame\";
        attribute severity : Integer = 3;
        attribute protocol : String = \"SPI\";
        attribute latency_ms : Real = 0.25;
    }
    hazard def HZ_Overcurrent {
        attribute severity : Integer = 4;
        attribute source_port : String = \"Bus.pwrOut\";
        attribute target_port : String = \"Actuator.pwrIn\";
        attribute part_ref : String = \"PowerStage\";
    }
    risk def RSK_Overcurrent {
        attribute severity : Integer = 3;
        attribute hazard_ref : String = \"HZ_Overcurrent\";
        attribute source_port : String = \"Bus.pwrOut\";
        attribute target_port : String = \"Actuator.pwrIn\";
    }
    state def SupervisorState {
        doc = \"Supervisor state property doc\";
        state ActiveSubState;
    }
}";
    let parsed = crate::parser::grammar::SysmlParser::parse_source(src, "Roundtrip505")?;
    assert_parsed_roundtrip_505_fields(&parsed)?;

    let serialized = parsed.to_sysml(0);
    assert!(!serialized.contains("\"\"") && serialized.contains("attribute latency_ms : Real = 0.25;"));
    let reparsed = crate::parser::grammar::SysmlParser::parse_source(&serialized, "Roundtrip505")?;
    assert_eq!(reparsed, parsed);
    Ok(())
}

fn build_deep_chains(total_levels: usize) -> (PackageDef, PartDef, StateDef) {
    let mut curr_pkg = PackageDef { name: format!("Pkg_{}", total_levels.saturating_sub(1)), ..Default::default() };
    let mut curr_part = PartDef { name: format!("Part_{}", total_levels.saturating_sub(1)), ..Default::default() };
    let mut curr_state = StateDef { name: format!("State_{}", total_levels.saturating_sub(1)), ..Default::default() };
    for idx in (0..total_levels.saturating_sub(1)).rev() {
        curr_pkg = PackageDef { name: format!("Pkg_{idx}"), packages: vec![curr_pkg], ..Default::default() };
        curr_part = PartDef { name: format!("Part_{idx}"), parts: vec![curr_part], ..Default::default() };
        curr_state = StateDef { name: format!("State_{idx}"), sub_states: vec![curr_state], ..Default::default() };
    }
    (curr_pkg, curr_part, curr_state)
}

#[test]
fn test_serialize_package_part_state_depth_bound_64_and_65() {
    let (curr_pkg, curr_part, curr_state) = build_deep_chains(65);
    let pkg_out = curr_pkg.to_sysml(0);
    assert!(pkg_out.contains("package Pkg_0 {") && pkg_out.contains("package Pkg_63 {") && !pkg_out.contains("Pkg_64"));
    let part_out = curr_part.to_sysml(0);
    assert!(part_out.contains("part def Part_0 {") && part_out.contains("part def Part_63 {") && !part_out.contains("Part_64"));
    let state_out = curr_state.to_sysml(0);
    assert!(state_out.contains("state def State_0 {") && state_out.contains("state def State_63 {") && !state_out.contains("State_64"));
}

#[test]
fn test_serialize_deep_nesting_bounded_stack_and_synthetic_forest_root() -> Result<(), String> {
    assert_eq!(MAX_AST_DEPTH, 64);
    let deep_levels: usize = 1_500;
    let (deep_pkg, deep_part, deep_state) = build_deep_chains(deep_levels);

    let mut deep_synthetic = PackageDef { name: "LeafNamedPkg".to_string(), ..Default::default() };
    for _ in 0..deep_levels {
        deep_synthetic = PackageDef { name: String::new(), packages: vec![deep_synthetic], ..Default::default() };
    }

    let (pkg_out, synth_out, part_out, state_out) =
        std::thread::scope(|s| -> Result<(String, String, String, String), String> {
            let handle = std::thread::Builder::new()
                .name("bounded_serializer_1500".to_string())
                .stack_size(512 * 1024)
                .spawn_scoped(s, || (deep_pkg.to_sysml(0), deep_synthetic.to_sysml(0), deep_part.to_sysml(0), deep_state.to_sysml(0)))
                .map_err(|e| format!("failed to spawn scoped thread: {e}"))?;
            handle.join().map_err(|_| "bounded-stack serializer thread panicked".to_string())
        })?;

    assert!(pkg_out.contains("package Pkg_63 {") && !pkg_out.contains("Pkg_64") && synth_out.is_empty());
    assert!(part_out.contains("part def Part_63 {") && !part_out.contains("Part_64"));
    assert!(state_out.contains("state def State_63 {") && !state_out.contains("State_64"));

    let mut pkg_drain = vec![deep_pkg, deep_synthetic];
    while let Some(mut node) = pkg_drain.pop() {
        pkg_drain.append(&mut node.packages);
    }
    let mut part_drain = vec![deep_part];
    while let Some(mut node) = part_drain.pop() {
        part_drain.append(&mut node.parts);
    }
    let mut state_drain = vec![deep_state];
    while let Some(mut node) = state_drain.pop() {
        state_drain.append(&mut node.sub_states);
    }
    Ok(())
}

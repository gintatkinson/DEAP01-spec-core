//! Unit tests for SysML v2 AST connectors, interfaces, expressions, and packages. [DOC-TIER: A]
//! Safety & Traceability: Realises: [REQ-0101], [REQ-0801]
//! Contract: Deterministic SysML v2 element parsing and model conversion unit tests.
//! Bounds & Faults: Bounded O(N) test execution; fails closed on assertion or parse mismatches.

#[cfg(test)]
mod tests {
    use super::super::*;

const MAX_AST_DEPTH: usize = 64;

#[test]
fn parse_connection_defs_extracts_connections_flows_and_owner_parts() -> Result<(), Box<dyn std::error::Error>> {
    let src = "doc /* Redundant avionics bus link */\nconnection def SensorToController { connect sensor.out_port to ctrl.in_port; }\nconnection inline_bus connect nav_out to guidance_in;\nflow def TelemetryStream { flow from recorder.tx_port to downlink.rx_port; }\nflow from orphan_src to orphan_tgt;\nconnection broken_unclosed {";
    let conns = parse_connection_defs(src);
    assert_eq!(conns.len(), 3);
    let c0 = conns.first().ok_or("missing SensorToController")?;
    assert_eq!((c0.name.as_str(), c0.doc.as_deref()), ("SensorToController", Some("Redundant avionics bus link")));
    assert_eq!((c0.source_port.as_str(), c0.target_port.as_str()), ("sensor.out_port", "ctrl.in_port"));
    assert_eq!((c0.source_part.as_deref(), c0.target_part.as_deref()), (Some("sensor"), Some("ctrl")));
    assert!(!c0.is_flow && c0.severity == 1);
    let c1 = conns.get(1).ok_or("missing inline_bus")?;
    assert_eq!((c1.name.as_str(), c1.source_port.as_str(), c1.target_port.as_str()), ("inline_bus", "nav_out", "guidance_in"));
    assert!(c1.source_part.is_none() && c1.target_part.is_none() && !c1.is_flow);
    let c2 = conns.get(2).ok_or("missing TelemetryStream")?;
    assert_eq!((c2.name.as_str(), c2.source_port.as_str(), c2.target_port.as_str()), ("TelemetryStream", "recorder.tx_port", "downlink.rx_port"));
    assert_eq!((c2.source_part.as_deref(), c2.target_part.as_deref(), c2.is_flow), (Some("recorder"), Some("downlink"), true));
    Ok(())
}

#[test]
fn parse_port_defs_extracts_conjugation_attributes_and_item_flows() -> Result<(), Box<dyn std::error::Error>> {
    let src = "doc /* Primary ARINC 429 receiver port */\nin port rx_bus : ~Arinc429Port {\n    attribute protocol_family : String = \"ARINC-429\";\n    attribute port_category : String = \"AvionicsBus\";\n    attribute impedance_ohm : Real = 75.0;\n    in item nav_frame : NavPacket;\n    flow out ack_signal : AckPulse;\n    inout status_word;\n}\nport ~conj_inline;\nport unclosed_port : BrokenType {";
    let ports = parse_port_defs(src);
    assert_eq!(ports.len(), 2);
    let p0 = ports.first().ok_or("missing rx_bus")?;
    assert_eq!((p0.name.as_str(), p0.type_name.as_str(), p0.direction.as_str(), p0.is_conjugated), ("rx_bus", "Arinc429Port", "in", true));
    assert_eq!((p0.doc.as_deref(), p0.protocol_family.as_str(), p0.port_category.as_str()), (Some("Primary ARINC 429 receiver port"), "ARINC-429", "AvionicsBus"));
    assert_eq!(p0.electrical_attributes.get("impedance_ohm").map(String::as_str), Some("75.0"));
    let flows: Vec<(&str, &str, &str)> = p0.item_flows.iter().map(|f| (f.name.as_str(), f.direction.as_str(), f.item_type.as_str())).collect();
    assert_eq!(flows, vec![("nav_frame", "in", "NavPacket"), ("ack_signal", "out", "AckPulse"), ("status_word", "inout", "Item")]);
    let p1 = ports.get(1).ok_or("missing conj_inline")?;
    assert_eq!((p1.name.as_str(), p1.type_name.as_str(), p1.direction.as_str(), p1.is_conjugated), ("conj_inline", "conj_inline", "inout", true));
    Ok(())
}

#[test]
fn interface_def_roundtrips_json_with_optional_doc() -> Result<(), serde_json::Error> {
    let iface = InterfaceDef {
        name: "NavToFlightCtrl".to_string(),
        source_port: "nav.state_out".to_string(),
        target_port: "fcu.state_in".to_string(),
        doc: Some("Deterministic state bus".to_string()),
    };
    let encoded = serde_json::to_string(&iface)?;
    assert_eq!(serde_json::from_str::<InterfaceDef>(&encoded)?, iface);
    let minimal: InterfaceDef = serde_json::from_str(r#"{"name":"Bus","source_port":"a","target_port":"b"}"#)?;
    assert!(minimal.doc.is_none());
    Ok(())
}

#[test]
fn parse_attribute_defs_handles_typed_assigned_and_malformed_variants() -> Result<(), Box<dyn std::error::Error>> {
    let src = "doc /* Nominal bus voltage */\nattribute def BusVoltage : ScalarValues::Real = 28.0;\nattribute cycleCount := 100;\nattribute statusLabel : String;\nattribute bareUntyped;\nattribute missingColonType : = 5;";
    let attrs = parse_attribute_defs(src);
    assert_eq!(attrs.len(), 3);
    let a0 = attrs.first().ok_or("missing BusVoltage")?;
    assert_eq!((a0.name.as_str(), a0.type_name.as_str()), ("BusVoltage", "ScalarValues::Real"));
    assert_eq!(a0.default_value.as_deref(), Some("28.0"));
    assert_eq!(a0.doc.as_deref(), Some("Nominal bus voltage"));
    let a1 = attrs.get(1).ok_or("missing cycleCount")?;
    assert_eq!((a1.name.as_str(), a1.type_name.as_str(), a1.default_value.as_deref()), ("cycleCount", "String", Some("100")));
    let a2 = attrs.get(2).ok_or("missing statusLabel")?;
    assert_eq!((a2.name.as_str(), a2.type_name.as_str(), a2.default_value.as_deref()), ("statusLabel", "String", None));
    Ok(())
}

#[test]
fn parse_part_defs_isolates_nested_port_and_subpart_attributes() -> Result<(), Box<dyn std::error::Error>> {
    let src = "part def FlightComputer {\n    attribute parentRateHz : Real = 200.0;\n    in port sensor_in : SensorPort {\n        attribute protocol_family : String = \"SPI\";\n        attribute portInternalGain : Real = 1.5;\n    }\n    part watchdog : WatchdogTimer {\n        attribute childTimeoutMs : Integer = 25;\n    }\n}";
    let parts = parse_part_defs(src);
    assert_eq!(parts.len(), 1);
    let parent = parts.first().ok_or("missing FlightComputer")?;
    assert_eq!(parent.attributes.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["parentRateHz"]);
    assert_eq!(parent.ports.len(), 1);
    let child = parent.parts.first().ok_or("missing watchdog")?;
    assert_eq!((child.name.as_str(), child.is_def, child.type_name.as_deref()), ("watchdog", false, Some("WatchdogTimer")));
    assert_eq!(child.attributes.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["childTimeoutMs"]);
    Ok(())
}

#[test]
fn parse_part_defs_enforces_max_ast_depth_and_plus_one_bounds() -> Result<(), String> {
    let mut nested_sysml = format!("part Leaf_{};", MAX_AST_DEPTH.saturating_add(1));
    for level in (0..=MAX_AST_DEPTH).rev() {
        nested_sysml = format!("part def Node_{level} {{ {nested_sysml} }}");
    }
    let parsed = parse_part_defs(&nested_sysml);
    let mut cursor = parsed.first().ok_or("missing root Node_0")?;
    for expected_level in 0..=MAX_AST_DEPTH {
        assert_eq!(cursor.name, format!("Node_{expected_level}"));
        if expected_level < MAX_AST_DEPTH {
            cursor = cursor.parts.first().ok_or_else(|| format!("missing child {expected_level}"))?;
        }
    }
    assert!(cursor.parts.is_empty());
    let mut very_deep = String::from("part DeepLeaf;");
    for level in (0..160_usize).rev() {
        very_deep = format!("part def Deep_{level} {{ {very_deep} }}");
    }
    assert_eq!(parse_part_defs(&very_deep).len(), 1);
    Ok(())
}

#[test]
fn package_def_iterative_queries_preserve_preorder_across_deep_tree() -> Result<(), Box<dyn std::error::Error>> {
    let mk_part = |name: &str, is_def: bool, port: &str, parts: Vec<PartDef>| PartDef {
        name: name.to_string(),
        is_def,
        ports: vec![PortDef { name: port.to_string(), ..Default::default() }],
        parts,
        ..Default::default()
    };
    let leaf = mk_part("sensorUsage", false, "nested_port_leaf", Vec::new());
    let subdef = mk_part("NestedSubDef", true, "nested_port_def", vec![leaf]);
    let mut current_pkg = PackageDef {
        name: "Pkg_128".to_string(),
        part_defs: vec![mk_part("TopPart_128", true, "top_port", vec![subdef])],
        ..Default::default()
    };
    for depth in (0..128_usize).rev() {
        let sibling = PackageDef { name: format!("SiblingPkg_{depth}"), ..Default::default() };
        current_pkg = PackageDef { name: format!("Pkg_{depth}"), packages: vec![sibling, current_pkg], ..Default::default() };
    }
    let all_pkgs = current_pkg.get_all_packages();
    assert_eq!(all_pkgs.len(), 257);
    assert_eq!(all_pkgs.first().map(|p| p.name.as_str()), Some("Pkg_0"));
    assert_eq!(all_pkgs.get(1).map(|p| p.name.as_str()), Some("SiblingPkg_0"));
    assert_eq!(all_pkgs.last().map(|p| p.name.as_str()), Some("Pkg_128"));
    let all_parts = current_pkg.get_all_parts();
    assert_eq!(all_parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["TopPart_128", "NestedSubDef"]);
    let nested_ports = current_pkg.get_nested_ports();
    assert!(nested_ports.contains("nested_port_def") && nested_ports.contains("nested_port_leaf") && !nested_ports.contains("top_port"));
    current_pkg.find_part_mut("sensorUsage").ok_or("missing sensorUsage")?.doc = Some("Updated in place".to_string());
    assert!(current_pkg.find_part_mut("UnknownPart").is_none());
    Ok(())
}

#[test]
fn sysml_model_and_package_def_roundtrip_conversions() {
    let pkg_default = PackageDef::from(SysmlModel::default());
    assert_eq!(pkg_default.name, "SysML_Model");
    assert_eq!(SysmlModel::from(pkg_default).package_name.as_deref(), Some("SysML_Model"));
    let attr = AttributeDef { name: "hydraulicSystem".to_string(), type_name: "Boolean".to_string(), default_value: Some("false".to_string()), doc: None };
    let pkg = PackageDef { name: "VehicleConfig".to_string(), attribute_defs: vec![attr], ..Default::default() };
    assert_eq!(pkg.extract_negated_components(), vec!["hydraulic".to_string()]);
    assert!(pkg.is_component_negated("Hydraulic Actuator") && !pkg.is_component_negated("Electric Motor"));
}

#[test]
fn constraint_and_calc_def_serde_roundtrip() -> Result<(), serde_json::Error> {
    let constraint = ConstraintDef {
        name: "EnvelopeLimit".to_string(),
        expression: "airspeed <= 350.0".to_string(),
        parameters: vec!["airspeed : Real".to_string()],
        is_assertion: true,
        doc: Some("Enforces Vne limit.".to_string()),
        pre_conditions: vec!["sensorValid == true".to_string()],
        post_conditions: vec!["latchArmed == false".to_string()],
    };
    let c_json = serde_json::to_string(&constraint)?;
    assert_eq!(serde_json::from_str::<ConstraintDef>(&c_json)?, constraint);
    let calc_min: CalcDef = serde_json::from_str(r#"{"name":"ComputeLift","doc":null}"#)?;
    assert_eq!(calc_min.name, "ComputeLift");
    assert!(calc_min.in_params.is_empty() && calc_min.return_type.is_none() && calc_min.expression.is_none());
    Ok(())
}

#[test]
fn parse_constraint_defs_extracts_assertions_docs_and_skips_malformed() -> Result<(), Box<dyn std::error::Error>> {
    let src = "/* Ignore constraint def CommentedOut { x > 0; } */\nattribute note : String = \"constraint def FakeInString { a == b; }\";\ndoc /* Formal rate bound */\nconstraint def RateBound(omega : Real, maxOmega : Real) { omega <= maxOmega; }\ndoc /* Asserted safety invariant */\nassert constraint ThermalMarginInvariant { tempK < 390.0; }\nassert constraint SemicolonAssertion;\nconstraint def ;\nconstraint BrokenParen(x : Real;\nconstraint NoTerminator\nconstraint UnclosedBrace { x > 0;";
    let parsed = parse_constraint_defs(src);
    assert_eq!(parsed.len(), 3);
    let c0 = parsed.first().ok_or("missing RateBound")?;
    assert_eq!((c0.name.as_str(), c0.is_assertion, c0.doc.as_deref()), ("RateBound", false, Some("Formal rate bound")));
    assert_eq!(c0.parameters, vec!["omega : Real", "maxOmega : Real"]);
    assert_eq!(c0.expression, "omega <= maxOmega");
    let c1 = parsed.get(1).ok_or("missing ThermalMarginInvariant")?;
    assert_eq!((c1.name.as_str(), c1.is_assertion, c1.doc.as_deref(), c1.expression.as_str()), ("ThermalMarginInvariant", true, Some("Asserted safety invariant"), "tempK < 390.0"));
    let c2 = parsed.get(2).ok_or("missing SemicolonAssertion")?;
    assert!(c2.name == "SemicolonAssertion" && c2.is_assertion && c2.expression.is_empty());
    Ok(())
}

#[test]
fn parse_calc_defs_extracts_signature_and_body_variants_and_skips_malformed() -> Result<(), Box<dyn std::error::Error>> {
    let src = "/* Ignore calc def CommentedCalc; */\nattribute note : String = \"calc def FakeCalcInString;\";\ndoc /* Dynamic pressure calculation */\ncalc def DynamicPressure(in attribute rho : ScalarValues::Real = 1.225, v = 80.0) : ScalarValues::Real { return 0.5 * rho * v * v; }\ncalc def BodyReturnTypeCalc {\n    in x : Real;\n    attribute gain : Real;\n    return type Real;\n    x * gain;\n}\ncalc StubCalc;\ncalc def ;\ncalc UnclosedParen(in x : Real;\ncalc MissingTerminator";
    let calcs = parse_calc_defs(src);
    assert_eq!(calcs.len(), 3);
    let dp = calcs.first().ok_or("missing DynamicPressure")?;
    assert_eq!((dp.name.as_str(), dp.doc.as_deref(), dp.return_type.as_deref(), dp.expression.as_deref()), ("DynamicPressure", Some("Dynamic pressure calculation"), Some("ScalarValues::Real"), Some("0.5 * rho * v * v")));
    assert_eq!(dp.in_params.len(), 2);
    let p0 = dp.in_params.first().ok_or("missing rho")?;
    assert_eq!((p0.name.as_str(), p0.type_name.as_str(), p0.default_value.as_deref()), ("rho", "ScalarValues::Real", Some("1.225")));
    let p1 = dp.in_params.get(1).ok_or("missing v")?;
    assert_eq!((p1.name.as_str(), p1.type_name.as_str(), p1.default_value.as_deref()), ("v", "ScalarValues::Real", Some("80.0")));
    let br = calcs.get(1).ok_or("missing BodyReturnTypeCalc")?;
    assert_eq!((br.name.as_str(), br.in_params.len(), br.return_type.as_deref(), br.expression.as_deref()), ("BodyReturnTypeCalc", 2, Some("Real"), Some("x * gain")));
    let stub = calcs.get(2).ok_or("missing StubCalc")?;
    assert!(stub.name == "StubCalc" && stub.in_params.is_empty() && stub.return_type.is_none() && stub.expression.is_none());
    Ok(())
}
}


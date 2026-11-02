//! Unit tests for SysML v2 AST parsing, extraction, slicing, and persistence. [DOC-TIER: A]
//! Safety & Traceability: Realises: [REQ-0101], [REQ-0801]
//! Contract: Deterministic SysML v2 AST unit-test suite covering grammar, queries, and digests.
//! Bounds & Faults: Bounded O(N) test execution; fails closed on assertion or parse mismatches.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::sysml_cst::{SourceProgram, SysmlCompilerDb};

    #[test]
    fn test_parse_part_defs_and_ports() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = "package SystemModel_SSOT {\n    doc /* SSOT for Autonomous System Architecture and Safety Model */\n    attribute actuatorCount : Integer = 4;\n    attribute topologyConfiguration : String = \"SymmetricTopology\";\n    attribute transientLoadLimit : Real = 12.0;\n    attribute maxLoadFactor : Real = 12.0;\n    part def StructuralFrame {\n        attribute massKg : Real = 25.0;\n    }\n    part def PrimaryControlUnit {\n        port ctrl_bus : SyncBus;\n        port telemetry : TelemetryLink;\n    }\n}";
        let model = parse_sysml(sysml_source);
        assert_eq!((model.package_name.as_deref(), model.parts.len()), (Some("SystemModel_SSOT"), 2));
        let (frame, ctrl) = (model.parts.first().ok_or("frame")?, model.parts.get(1).ok_or("ctrl")?);
        let attr0 = frame.attributes.first().ok_or("attr0")?;
        assert_eq!((frame.name.as_str(), frame.attributes.len(), attr0.name.as_str(), attr0.type_name.as_str(), attr0.default_value.as_deref()), ("StructuralFrame", 1, "massKg", "Real", Some("25.0")));
        let (port0, port1) = (ctrl.ports.first().ok_or("port0")?, ctrl.ports.get(1).ok_or("port1")?);
        assert_eq!((ctrl.name.as_str(), ctrl.ports.len(), port0.name.as_str(), port0.type_name.as_str(), port1.name.as_str(), port1.type_name.as_str()), ("PrimaryControlUnit", 2, "ctrl_bus", "SyncBus", "telemetry", "TelemetryLink"));
        let mut db = SysmlCompilerDb::new();
        assert_eq!(lower_to_hir_db(&mut db, &SourceProgram::new("schema/model.sysml", sysml_source)), model);
        Ok(())
    }
    #[test]
    fn test_parse_actions_and_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = "action def ComputeStateTransition {\n    in attribute currentState : Vector3;\n    out attribute nextState : Vector3;\n}\nconstraint def RateLimitConstraint(v : Real) {\n    v <= 60.0;\n}";
        let actions = parse_action_defs(sysml_source);
        let act0 = actions.first().ok_or("act0")?;
        assert_eq!((actions.len(), act0.name.as_str(), act0.in_params.first().ok_or("in0")?.name.as_str(), act0.out_params.first().ok_or("out0")?.name.as_str()), (1, "ComputeStateTransition", "currentState", "nextState"));
        let constraints = parse_constraint_defs(sysml_source);
        let con0 = constraints.first().ok_or("con0")?;
        assert_eq!((constraints.len(), con0.name.as_str(), con0.parameters.as_slice(), con0.expression.as_str()), (1, "RateLimitConstraint", &["v : Real".to_string()][..], "v <= 60.0"));
        Ok(())
    }
    #[test]
    fn test_issue_428_parse_action_defs_nested_blocks_and_docblocks() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = "/* Top-level license comment */\npackage SystemControl {\n    attribute configStr : String = \"action def FakeActionInStringLiteral { in a : Int; }\";\n    /* Unrelated model comment */\n    doc /* Primary path planning action */\n    action def PlanExecutionSequence {\n        in attribute currentState : Vector3;\n        if (check_bounds) {\n            // Nested compound block with braces\n            sub_routine();\n        }\n        out attribute targetState : Vector3;\n        perform ControlProcessor;\n        step computeSequence;\n        step validateTransition;\n    }\n    doc /* Semicolon terminated action */\n    action def EmergencyStop(in code : Integer);\n    /* Comment before action usage */\n    action executeTransition;\n}";
        let actions = parse_action_defs(sysml_source);
        assert!(!actions.iter().any(|a| a.name == "FakeActionInStringLiteral") && actions.len() == 3);
        let plan = actions.iter().find(|a| a.name == "PlanExecutionSequence").ok_or("PlanExecutionSequence")?;
        assert_eq!((plan.doc.as_deref(), plan.is_def, plan.in_params.first().ok_or("in0")?.name.as_str(), plan.out_params.first().ok_or("out0")?.name.as_str(), plan.performer.as_deref()), (Some("Primary path planning action"), true, "currentState", "targetState", Some("ControlProcessor")));
        assert_eq!(plan.steps, vec!["computeSequence", "validateTransition"]);
        let estop = actions.iter().find(|a| a.name == "EmergencyStop").ok_or("EmergencyStop")?;
        assert_eq!((estop.doc.as_deref(), estop.is_def, estop.in_params.first().ok_or("in0")?.name.as_str()), (Some("Semicolon terminated action"), true, "code"));
        let maneuver = actions.iter().find(|a| a.name == "executeTransition").ok_or("executeTransition")?;
        assert!(maneuver.doc.is_none() && !maneuver.is_def);
        Ok(())
    }
    #[test]
    fn test_parse_connections_and_states() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = "part def CoreSystem {\n    state def OperationalState {\n        entry InitializeState;\n        do UpdateStateEstimator;\n        exit ShutdownSensors;\n    }\n    connection def SensorToController {\n        connect sensor.out_port to ctrl.in_port;\n    }\n}";
        let parts = parse_part_defs(sysml_source);
        let core = parts.first().ok_or("core")?;
        let (st0, conn0) = (core.states.first().ok_or("st0")?, core.connections.first().ok_or("conn0")?);
        assert_eq!((parts.len(), core.name.as_str(), st0.name.as_str(), st0.entry_action.as_deref(), conn0.name.as_str(), conn0.source_port.as_str(), conn0.target_port.as_str()), (1, "CoreSystem", "OperationalState", Some("InitializeState"), "SensorToController", "sensor.out_port", "ctrl.in_port"));
        Ok(())
    }
    #[test]
    fn test_parse_ground_truth_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = "package SystemModel_SSOT {\n    doc /* Ground truth reference model for structural parsing */\n    attribute redundancyLevel : Integer = 2;\n    part def PrimaryControlUnit {\n        port ctrl_in : CommandPort;\n        attribute cycleRateHz : Real = 50.0;\n    }\n    part def StateEstimator {\n        port sync_pulse_out : SyncPort;\n    }\n}";
        let model = parse_sysml(fixture);
        assert_eq!((model.package_name.as_deref(), model.parts.len(), model.parts.first().ok_or("p0")?.name.as_str(), model.parts.get(1).ok_or("p1")?.name.as_str()), (Some("SystemModel_SSOT"), 2, "PrimaryControlUnit", "StateEstimator"));
        Ok(())
    }
    #[test]
    fn test_parse_port_defs_syntax_variants() -> Result<(), Box<dyn std::error::Error>> {
        let text = "out port rules_out : CompilerRulePort {\n}\nport out token_out : TokenStreamPort;\nin port schema_in : RawSchemaStreamPort {\n    attribute protocol_family : String = \"ControlBus\";\n}\nport in data_in : DataPortType;\nport cmd : Command;";
        let ports = parse_port_defs(text);
        let actual: Vec<(&str, &str, &str)> = ports.iter().map(|p| (p.name.as_str(), p.direction.as_str(), p.type_name.as_str())).collect();
        assert_eq!(actual, vec![("rules_out", "out", "CompilerRulePort"), ("token_out", "out", "TokenStreamPort"), ("schema_in", "in", "RawSchemaStreamPort"), ("data_in", "in", "DataPortType"), ("cmd", "inout", "Command")]);
        Ok(())
    }
    #[test]
    fn test_parse_requirement_defs_with_attributes() -> Result<(), Box<dyn std::error::Error>> {
        let text = "requirement def REQ_0001_Test {\n    id = \"REQ-0001\";\n    text = \"System shall do something.\";\n    attribute uuidv5 : String = \"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\";\n    attribute complexity_class : String = \"Class P\";\n}";
        let reqs = parse_requirement_defs(text);
        let r0 = reqs.first().ok_or("r0")?;
        let (a0, a1) = (r0.attributes.first().ok_or("a0")?, r0.attributes.get(1).ok_or("a1")?);
        assert_eq!((reqs.len(), r0.req_id.as_str(), r0.attributes.len(), a0.name.as_str(), a0.default_value.as_deref(), a1.name.as_str(), a1.default_value.as_deref()), (1, "REQ-0001", 2, "uuidv5", Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\""), "complexity_class", Some("\"Class P\"")));
        Ok(())
    }
    #[test]
    fn test_parse_requirement_defs_constraints_assumptions_derivations() -> Result<(), Box<dyn std::error::Error>> {
        let text = "requirement def REQ_0032_Test {\n    id = \"REQ-0032\";\n    text = \"Requirements shall track assumptions, constraints, and derivations.\";\n    assume ValidSchemaInput;\n    require Invariant_REQ_0032_StrictTotalOrdering;\n    require Invariant_REQ_0032_Determinism;\n    derived from REQ_0031_BaseEmission;\n    derived from REQ_0019_IdentifierSanitization;\n}";
        let reqs = parse_requirement_defs(text);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!((reqs.len(), r0.assumes.as_slice(), r0.requires.as_slice(), r0.derived_from.as_slice()), (1, &["ValidSchemaInput".to_string()][..], &["Invariant_REQ_0032_StrictTotalOrdering".to_string(), "Invariant_REQ_0032_Determinism".to_string()][..], &["REQ_0031_BaseEmission".to_string(), "REQ_0019_IdentifierSanitization".to_string()][..]));
        Ok(())
    }
    #[test]
    fn test_parse_requirement_defs_with_encapsulated_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let text = "requirement def REQ_0001_Universal_Grounding {\n    id = \"REQ-0001\";\n    text = \"Compiler shall accept schema directory.\";\n    attribute uuidv5 : String = \"d4865863-718a-53a8-bcf6-e070e176378e\";\n    doc /* \\forall \\sigma \\in Symbols(A), \\sigma \\in Tokens(S) */\n    constraint def Invariant_Universal_Symbol_Grounding;\n    assert constraint Invariant_Zero_Hardcoded_Domain_Concepts;\n    require Invariant_Universal_Symbol_Grounding;\n    require Invariant_Zero_Hardcoded_Domain_Concepts;\n}";
        let reqs = parse_requirement_defs(text);
        let r0 = reqs.first().ok_or("r0")?;
        let (c0, c1) = (r0.constraints.first().ok_or("c0")?, r0.constraints.get(1).ok_or("c1")?);
        assert_eq!((r0.constraints.len(), c0.name.as_str(), c0.doc.as_deref(), c0.is_assertion, c1.name.as_str(), c1.is_assertion), (2, "Invariant_Universal_Symbol_Grounding", Some("\\forall \\sigma \\in Symbols(A), \\sigma \\in Tokens(S)"), false, "Invariant_Zero_Hardcoded_Domain_Concepts", true));
        Ok(())
    }
    #[test]
    fn test_package_def_recursive_ast_collectors() -> Result<(), Box<dyn std::error::Error>> {
        let nested = PackageDef { name: "Nested".to_string(), part_defs: vec![PartDef { name: "NestedPart".to_string(), ..Default::default() }], use_case_defs: vec![UseCaseDef { name: "NestedUC".to_string(), ..Default::default() }], interaction_defs: vec![InteractionDef { name: "NestedInter".to_string(), ..Default::default() }], constraint_defs: vec![ConstraintDef { name: "NestedConstraint".to_string(), ..Default::default() }], ..Default::default() };
        let sub = PackageDef { name: "Sub".to_string(), capability_defs: vec![CapabilityDef { name: "SubCap".to_string(), ..Default::default() }], action_defs: vec![ActionDef { name: "SubAction".to_string(), ..Default::default() }], packages: vec![nested], ..Default::default() };
        let root = PackageDef { name: "Root".to_string(), packages: vec![sub], ..Default::default() };
        assert_eq!(root.get_all_packages().len(), 3);
        assert_eq!(root.get_all_capabilities().first().ok_or("cap0")?.name, "SubCap");
        assert_eq!(root.get_all_parts().first().ok_or("part0")?.name, "NestedPart");
        assert_eq!(root.get_all_use_cases().first().ok_or("uc0")?.name, "NestedUC");
        assert_eq!(root.get_all_interactions().first().ok_or("inter0")?.name, "NestedInter");
        assert_eq!(root.get_all_actions().first().ok_or("act0")?.name, "SubAction");
        assert_eq!(root.get_all_constraints().first().ok_or("con0")?.name, "NestedConstraint");
        Ok(())
    }
    #[test]
    fn test_parse_test_case_defs_with_assert_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let content = "test case def TC_UCA_33_Verification {\n    subject SafetyController;\n    verify requirement Assert_UCA_33, Assert_UCA_34;\n    objective \"Verify safety interlock per UCA-33\";\n    step trigger_interlock;\n}";
        let test_cases = parse_test_case_defs(content);
        let tc0 = test_cases.first().ok_or("tc0")?;
        assert_eq!((tc0.name.as_str(), tc0.subject_part.as_deref(), tc0.objective.as_deref()), ("TC_UCA_33_Verification", Some("SafetyController"), Some("Verify safety interlock per UCA-33")));
        assert_eq!((tc0.verified_requirements.as_slice(), tc0.test_steps.as_slice()), (&["Assert_UCA_33".to_string(), "Assert_UCA_34".to_string()][..], &["trigger_interlock".to_string()][..]));
        Ok(())
    }
    #[test]
    fn test_component_matches_and_clean_names() {
        for (a, b) in [("Primary Transducer", "Primary_Transducer"), ("PrimaryController", "PrimaryController"), ("primary_controller", "PrimaryController"), ("StateEstimator", "state_estimator"), ("[PrimaryController]", "PrimaryController"), ("**PrimaryController**", "PrimaryController"), ("`PrimaryController`", "PrimaryController")] {
            assert!(component_matches(a, b));
        }
        assert!(!component_matches("PhantomSubsystem", "PrimaryTransducer") && !component_matches("GhostUnit", "PrimaryController"));
    }
    #[test]
    fn test_extract_negated_components_detection() {
        let backup = vec!["backup".to_string()];
        for sample in [r#"attribute backupEnabled = 0;"#, r#"attribute backup_system = "none";"#, r#"TestPlatform.backupSystem = "No""#, "Backup subsystem: No", "No backup unit installed", "!exists(Backup)"] {
            assert_eq!(extract_negated_components(sample), backup);
        }
        for pos in [r#"attribute backupEnabled = 1;"#, r#"attribute backupSystem = "Yes";"#, "Nominal operations", "Retry no more than 3 times"] {
            assert!(extract_negated_components(pos).is_empty());
        }
    }
    #[test]
    fn test_is_component_negated_detection() -> Result<(), Box<dyn std::error::Error>> {
        let model = "attribute backupEnabled = 0;\nattribute backupSystem = \"No\";";
        assert!(is_component_negated("AuxiliaryBackupUnit", model) && is_component_negated("`AuxiliaryBackupUnit`", model));
        assert!(!is_component_negated("PrimaryController", model) && !is_component_negated("StateEstimator", model) && !is_component_negated("", model));
        let sub = PackageDef { name: "Subsystems".to_string(), doc: Some("Redundant module: none".to_string()), constraint_defs: vec![ConstraintDef { name: "DiagnosticsAbsent".to_string(), expression: "!exists(DiagnosticsService)".to_string(), ..Default::default() }], ..Default::default() };
        let pkg = PackageDef { name: "SystemModel_SSOT".to_string(), attribute_defs: vec![AttributeDef { name: "backupEnabled".to_string(), default_value: Some("0".to_string()), ..Default::default() }], packages: vec![sub], ..Default::default() };
        let stems = pkg.extract_negated_components();
        assert_eq!(stems, vec!["backup", "diagnosticsservice", "redundant"]);
        assert!(pkg.is_component_negated("AuxiliaryBackupUnit") && pkg.is_component_negated("RedundantModule") && pkg.is_component_negated("DiagnosticsService") && !pkg.is_component_negated("PrimaryController") && stems.first().ok_or("stem0")?.len() >= 3);
        Ok(())
    }
    #[test]
    fn test_numeric_state_attributes_do_not_negate_components() {
        let model = "attribute faultCount = 0;\nattribute retryLimit = 0;\nattribute hashSeed = 0;\nattribute timeUnit = 0;";
        assert!(extract_negated_components(model).is_empty());
        assert!(!is_component_negated("FaultMonitor", model) && !is_component_negated("RetryHandler", model) && !is_component_negated("HashSeedGenerator", model) && !is_component_negated("TimeSource", model));
        let control = "attribute hasFaultMonitor = false;";
        assert_eq!(extract_negated_components(control), vec!["faultmonitor".to_string()]);
        assert!(is_component_negated("FaultMonitor", control) && !is_component_negated("RetryHandler", control));
        assert_eq!(extract_negated_components("attribute has_backup_module = false;"), vec!["backup".to_string()]);
    }
    #[test]
    fn test_multi_clause_prose_negation_extracts_every_clause() {
        let prose = "Negative safety model invariant: backupEnabled = 0, redundantSystem = \"No\"";
        let stems = extract_negated_components(prose);
        assert!(stems.contains(&"backup".to_string()) && stems.contains(&"redundant".to_string()));
        assert!(is_component_negated("AuxiliaryBackupUnit", prose) && !is_component_negated("PrimaryProcessor", prose));
        let dotted = extract_negated_components("Invariant: Owner.auxSystem = 0, redundantSystem = \"No\"");
        assert!(dotted.contains(&"aux".to_string()) && !dotted.contains(&"owneraux".to_string()));
    }
    #[test]
    fn test_unified_schema_model_extractor_all_formats() {
        let sysml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", "/* comment */\npackage SampleSystem {\n    part def ControlProcessor;\n    part StateEstimator :> SensorBase;\n}");
        let yaml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yaml", "subsystems:\n  - ControlProcessor\n  - SequencingSystem\ncomponents:\n  - name: PowerDistributionUnit\n    id: PDU-01\n");
        let json_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", "{\"schema_nodes\":[\"part:TelemetryTransceiver\",\"port:TX\"],\"components\":[{\"name\":\"ThermalController\",\"id\":\"TCU\"}]}");
        let md_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", "# System Architecture\n### 4.3 Subsystem Architecture\n| Subsystem | Function | Allocation |\n|---|---|---|\n| RedundantModule | Core processing | Segment A |\n| ActuatorControllerUnit | Output control | Segment B |\n");
        let proto_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".proto", "syntax = \"proto3\";\npackage telemetry_pkg;\nservice DiagnosticsService {\n    rpc GetState (Empty) returns (State);\n}\nmessage SensorData {\n    int64 timestamp = 1;\n}");
        let idl_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".idl", "module SystemBus {\n    interface BusController {\n        void transmit();\n    };\n};");
        let arxml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".arxml", "<AUTOSAR>\n    <SW-COMPONENT-PROTOTYPE>\n        <SHORT-NAME>ProcessController</SHORT-NAME>\n    </SW-COMPONENT-PROTOTYPE>\n</AUTOSAR>");
        assert!(sysml_parts.contains("ControlProcessor") && sysml_parts.contains("StateEstimator"));
        assert!(yaml_parts.contains("ControlProcessor") && yaml_parts.contains("SequencingSystem") && yaml_parts.contains("PowerDistributionUnit"));
        assert!(json_parts.contains("TelemetryTransceiver") && json_parts.contains("ThermalController"));
        assert!(md_parts.contains("RedundantModule") && md_parts.contains("ActuatorControllerUnit"));
        assert!(proto_parts.contains("DiagnosticsService") && proto_parts.contains("SensorData"));
        assert!(idl_parts.contains("BusController") && idl_parts.contains("SystemBus") && arxml_parts.contains("ProcessController"));
    }
    #[test]
    fn test_unified_schema_model_extractor_yang_container_grouping_list() {
        let yang = "module sample_module {\n    yang-version 1.1;\n    prefix sm;\n    grouping EndpointSettings {\n        leaf label { type string; }\n    }\n    container NodeRegistry {\n        list NodeEntry {\n            key \"label\";\n            uses EndpointSettings;\n        }\n    }\n    typedef counter_kind { type uint32; }\n    container access-table { leaf size { type uint32; } }\n}";
        let yang_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yang", yang);
        assert!(yang_parts.contains("EndpointSettings") && yang_parts.contains("NodeRegistry") && yang_parts.contains("NodeEntry"));
        assert!(!yang_parts.contains("sample_module") && !yang_parts.contains("counter_kind") && !yang_parts.contains("label"));
        assert_eq!(UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".YANG", yang), yang_parts);
        assert!(UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS.contains(&".yang"));
    }
    #[test]
    fn test_parse_calc_defs_syntax_variants() -> Result<(), Box<dyn std::error::Error>> {
        let content = "doc /* Calculates kinetic energy */\ncalc def KineticEnergy(in massKg : Real, in speedMps : Real) : Real {\n    0.5 * massKg * speedMps * speedMps\n}\ncalc def VelocityCalc {\n    in v0 : Real;\n    in a : Real;\n    in t : Real;\n    return v : Real;\n    v0 + a * t;\n}\ncalc MarginCalculation;";
        let calcs = parse_calc_defs(content);
        let (ke, vc, mc) = (calcs.first().ok_or("ke")?, calcs.get(1).ok_or("vc")?, calcs.get(2).ok_or("mc")?);
        assert_eq!((calcs.len(), ke.name.as_str(), ke.doc.as_deref(), ke.in_params.len(), ke.return_type.as_deref(), ke.expression.as_deref()), (3, "KineticEnergy", Some("Calculates kinetic energy"), 2, Some("Real"), Some("0.5 * massKg * speedMps * speedMps")));
        assert_eq!((vc.name.as_str(), vc.in_params.len(), vc.return_type.as_deref(), vc.expression.as_deref()), ("VelocityCalc", 3, Some("Real"), Some("v0 + a * t")));
        assert!(mc.name == "MarginCalculation" && mc.in_params.is_empty() && mc.return_type.is_none() && mc.expression.is_none());
        Ok(())
    }
    #[test]
    fn test_package_def_queries_and_nested_ports() -> Result<(), Box<dyn std::error::Error>> {
        let sub_part = PartDef { name: "NestedSensor".to_string(), ports: vec![PortDef { name: "sync_pulse_out".to_string(), ..Default::default() }], ..Default::default() };
        let top_part = PartDef { name: "ControlProcessor".to_string(), ports: vec![PortDef { name: "ctrl_bus".to_string(), ..Default::default() }], parts: vec![sub_part], calcs: vec![CalcDef { name: "InnerCalc".to_string(), ..Default::default() }], ..Default::default() };
        let root = PackageDef { name: "RootSystem".to_string(), calc_defs: vec![CalcDef { name: "RootCalc".to_string(), ..Default::default() }], port_defs: vec![PortDef { name: "root_port".to_string(), ..Default::default() }], hazard_defs: vec![HazardDef { name: "H-01".to_string(), severity: 3, ..Default::default() }], risk_defs: vec![RiskDef { name: "R-01".to_string(), severity: 3, ..Default::default() }], state_defs: vec![StateDef { name: "Nominal".to_string(), ..Default::default() }], connection_defs: vec![ConnectionDef { name: "C-01".to_string(), source_port: "p1".to_string(), target_port: "p2".to_string(), ..Default::default() }], part_defs: vec![top_part], ..Default::default() };
        assert_eq!((root.get_all_calcs().len(), root.get_all_ports().len()), (2, 3));
        assert_eq!(root.get_all_hazards().first().ok_or("h0")?.name, "H-01");
        assert_eq!(root.get_all_risks().first().ok_or("r0")?.name, "R-01");
        assert_eq!(root.get_all_states().first().ok_or("s0")?.name, "Nominal");
        assert_eq!(root.get_all_connections().first().ok_or("c0")?.name, "C-01");
        let nested_ports = root.get_nested_ports();
        assert!(nested_ports.len() == 1 && nested_ports.contains("sync_pulse_out"));
        Ok(())
    }
    #[test]
    fn test_sysml_ast_reexports_and_identifier_primitives_roundtrip() -> Result<(), String> {
        let masked = mask_string_literals("part def FlightCtrl { doc /* \"inner\" */ }");
        let open_idx = masked.find('{').ok_or("missing opening brace")?;
        let close_idx = find_matching_brace(&masked, open_idx).ok_or("missing closing brace")?;
        assert_eq!(masked.as_bytes().get(close_idx).copied(), Some(b'}'));
        assert!(is_valid_subsystem_identifier("FlightCtrl") && !is_valid_subsystem_identifier("package"));
        Ok(())
    }
    #[test]
    fn test_supported_schema_extension_enum_cardinality_order_and_catalog_parity() {
        let expected = vec![".sysml", ".yaml", ".yml", ".json", ".proto", ".idl", ".arxml", ".md", ".markdown", ".yang"];
        let via_enum: Vec<&'static str> = SupportedSchemaExtension::ALL.iter().map(|x| x.as_str()).collect();
        let catalog = UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS;
        let via_catalog: Vec<&'static str> = catalog.iter().collect();
        assert_eq!((&via_enum, &via_catalog), (&expected, &expected));
        for x in SupportedSchemaExtension::ALL {
            assert!(x.as_str_ref() == x.as_str() && catalog.contains(&x.as_str()));
        }
        assert!(!catalog.contains(&".txt") && !catalog.contains(&"sysml") && !catalog.contains(&".SYSML") && !catalog.contains(&""));
    }
    #[test]
    fn test_supported_extensions_catalog_exact_size_and_double_ended_iter() {
        let mut iter = UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS.iter();
        assert_eq!((iter.len(), iter.next(), iter.next_back(), iter.len()), (10, Some(".sysml"), Some(".yang"), 8));
    }
    #[test]
    fn test_parse_satisfied_requirements_and_part_defs_extraction() {
        let snippet = "part def SensorController {\n    satisfy requirement REQ-CTRL-001;\n    satisfy requirement \"REQ-CTRL-002\";\n    satisfy REQ-PWR-03 by PowerBus;\n    satisfy requirement REQ-CTRL-001;\n    satisfy /* c1 */ requirement REQ-COM-04;\n    satisfy/*c2*/requirement REQ-COM-05;\n    satisfy requirement /*c3*/ REQ-COM-06;\n    satisfy requirement REQ-COM-07 /*c4*/;\n    satisfy requirement REQ-COM-08 /* c1 */ by /* c2 */ PowerBus /* c3 */;\n}";
        let expected: Vec<String> = ["REQ-CTRL-001", "REQ-CTRL-002", "REQ-PWR-03", "REQ-COM-04", "REQ-COM-05", "REQ-COM-06", "REQ-COM-07", "REQ-COM-08"].into_iter().map(String::from).collect();
        assert_eq!(parse_satisfied_requirements(snippet), expected);
        let parts = parse_part_defs(snippet);
        assert_eq!(parts.first().map(|p| (&p.name[..], &p.satisfied_requirements)), Some(("SensorController", &expected)));
    }
    #[test]
    fn test_sysml_ast_attribute_and_flow_serde_defaults_roundtrip() -> Result<(), String> {
        let attr: AttributeDef = serde_json::from_str(r#"{"name":"pressure","type_name":"Real"}"#).map_err(|e| e.to_string())?;
        let flow: FlowDef = serde_json::from_str(r#"{"name":"telemetry_tx"}"#).map_err(|e| e.to_string())?;
        assert!(attr.name == "pressure" && attr.default_value.is_none() && flow.direction == "out" && flow.item_type == "Item");
        Ok(())
    }
    #[test]
    fn test_parse_triggers_hazard_safety_realises_and_layer_annotations() -> Result<(), String> {
        let sysml = "package SafetyPkg {\n    part def FlightActuator {\n        @TriggersHazard { hazard = \"H-01\"; }\n        @TriggersHazard(\"H-02\")\n        @SafetyRealises { requirement = \"SC-01\"; }\n        @SafetyRealises(\"SC-02\")\n        @Layer { name = \"Domain\"; }\n    }\n    requirement def SC-01 {\n        @TriggersHazard(\"H-01\")\n        @SafetyRealises { requirement = \"SYS-REQ-01\"; }\n        @Layer(\"Safety\")\n    }\n}";
        let model = parse_sysml_model(sysml);
        let (part, req) = (model.parts.first().ok_or("Expected FlightActuator")?, model.requirements.first().ok_or("Expected SC-01")?);
        assert_eq!((part.triggers_hazards.as_slice(), part.safety_realises.as_slice(), part.layer.as_deref()), (&["H-01".to_string(), "H-02".to_string()][..], &["SC-01".to_string(), "SC-02".to_string()][..], Some("Domain")));
        assert_eq!((req.triggers_hazards.as_slice(), req.safety_realises.as_slice(), req.layer.as_deref()), (&["H-01".to_string()][..], &["SYS-REQ-01".to_string()][..], Some("Safety")));
        Ok(())
    }
    #[test]
    fn test_slice_package_subsystem_within_budget_and_overflow() -> Result<(), String> {
        let sysml = "package FlightGuidance {\n    part def GuidanceComputer {\n        @TriggersHazard(\"H-01\")\n        @SafetyRealises(\"SC-01\")\n        @Layer(\"Domain\")\n        in port nav_in : NavBusPort;\n        out port cmd_out : ActuatorCmdPort;\n        satisfy requirement REQ-FG-01;\n    }\n    connection nav_to_cmd connect nav_in to cmd_out;\n    requirement def REQ-FG-01 { doc /* Compute guidance law. */ }\n    requirement def SC-01 { doc /* Bound pitch rate. */ }\n    hazard def H-01 { doc /* Uncommanded pitch. */ }\n}";
        let model = parse_sysml_model(sysml);
        let slice = model.slice_package_subsystem("FlightGuidance", 4096)?;
        assert_eq!((slice.target_package.as_str(), slice.packages.len(), slice.parts.len(), slice.ports.len(), slice.interfaces.len(), slice.requirements.len(), slice.hazards.len()), ("FlightGuidance", 1, 1, 2, 1, 2, 1));
        assert!(model.slice_package_subsystem("FlightGuidance", 1).is_err() && model.slice_package_subsystem("FlightGuidance", 0).is_err() && model.slice_package_subsystem("Missing", 4096).is_err());
        Ok(())
    }
    #[test]
    fn test_incremental_ast_cache_miss_and_hit() -> Result<(), String> {
        let tmp_root = std::env::temp_dir().join(format!("deap_core_ast_cache_test_{}", uuid::Uuid::new_v4()));
        let cache = IncrementalAstCache::new(&tmp_root);
        let sysml_source = "package CacheTestPkg { part def CachedSensor { @Layer(\"Domain\") attribute sample_rate_hz : Real = 100.0; } }";
        let (model_miss, was_hit_first) = cache.load_or_parse(sysml_source)?;
        let (model_hit, was_hit_second) = cache.load_or_parse(sysml_source)?;
        assert!(!was_hit_first && was_hit_second && model_miss == model_hit);
        std::fs::remove_dir_all(&tmp_root).map_err(|e| e.to_string())?;
        Ok(())
    }
    #[test]
    fn test_type_descriptor_from_part_def() -> Result<(), String> {
        let model = parse_sysml_model("package AvionicsPkg { part def AirDataSensor { @Layer(\"Domain\") attribute airspeed_mps : Real = 85.0 [m/s] [0.0..350.0]; satisfy requirement REQ-ADS-01; } }");
        let td = TypeDescriptor::from_part_def("AvionicsPkg", model.parts.first().ok_or("Expected AirDataSensor")?);
        let first_field = td.fields.first().ok_or("Expected airspeed_mps field")?;
        assert_eq!((td.type_name.as_str(), td.layer.as_deref(), first_field.si_unit.as_deref(), first_field.range_bounds), ("AirDataSensor", Some("Domain"), Some("m/s"), Some((0.0, 350.0))));
        Ok(())
    }
    #[test]
    fn test_ast_persistence_engine_and_cot_audit_log() -> Result<(), String> {
        let tmp_root = std::env::temp_dir().join(format!("deap_core_persistence_test_{}", uuid::Uuid::new_v4()));
        let engine = AstPersistenceEngine::new(tmp_root.clone());
        let model = parse_sysml_model("package PersistedPkg { part def PersistentNode { attribute node_id : Integer = 7; } }");
        let saved_path = engine.persist_model("model_alpha", &model)?;
        assert!(saved_path.is_file() && engine.load_model("model_alpha")? == model && engine.load_model("missing_model").is_err());
        let mut audit_log = CotAuditLog::new();
        audit_log.append_record(CotAuditRecord { step_id: "STEP-01".to_string(), model_id: "deepseek-r1-local".to_string(), prompt_sha256: IncrementalAstCache::compute_digest("prompt-payload"), reasoning_trace: "Verified PartDef PersistentNode invariants.".to_string(), verified_ast_refs: vec!["PersistedPkg::PersistentNode".to_string()] });
        assert!(audit_log.to_json_pretty()?.contains("PersistedPkg::PersistentNode"));
        std::fs::remove_dir_all(&tmp_root).map_err(|e| e.to_string())?;
        Ok(())
    }
    #[test]
    fn test_issue_549_discover_schema_files_excludes_digest_json_sidecars() -> Result<(), String> {
        let tmp_dir = std::env::temp_dir().join(format!("deap_core_issue_549_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        for fname in ["blueprint-01-alpha.sysml", "blueprint-01-alpha.digest.json", "schema-digest.json", "schema_digest.json", "digest.json", "valid_schema.json"] {
            std::fs::write(tmp_dir.join(fname), "{}").map_err(|e| e.to_string())?;
        }
        let discovered = UnifiedSchemaModelExtractor::discover_schema_files(std::slice::from_ref(&tmp_dir));
        std::fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        let names: Vec<String> = discovered.iter().filter_map(|p| p.file_name()?.to_str().map(String::from)).collect();
        assert!(names.contains(&"blueprint-01-alpha.sysml".to_string()) && names.contains(&"valid_schema.json".to_string()) && names.len() == 2);
        Ok(())
    }
    #[test]
    fn test_issue_551_extract_subsystem_parts_ignores_doc_prose_and_schema_digest() {
        let sysml_src = "package FlightPkg {\n    doc /* This module is part Boundaries of the counterpart TelemetryBus and part Hierarchy. */\n    part def FlightDirector {\n        part actuator_unit : ActuatorNode;\n    }\n}";
        let sysml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", sysml_src);
        assert!(sysml_parts.contains("FlightDirector") && sysml_parts.contains("actuator_unit") && !sysml_parts.contains("Boundaries") && !sysml_parts.contains("TelemetryBus") && !sysml_parts.contains("Hierarchy"));
        let digest_json = "{\n  \"format_version\": \"1.0\",\n  \"source_hash\": \"deadbeef\",\n  \"parts\": [{\"name\": \"DigestGhostPart\"}],\n  \"modules\": [{\"name\": \"DigestGhostModule\"}]\n}";
        assert!(UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", digest_json).is_empty());
    }
    #[test]
    fn test_issue_553_extract_subsystem_parts_ignores_unrelated_markdown_tables() {
        let md_src = "| ID | Subsystem | Owner |\n| --- | --- | --- |\n| 1 | NavigationUnit | Avionics |\n\n| Acronym | Meaning |\n| --- | --- |\n| GNSSReceiverTerm | Global |\n";
        let md_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", md_src);
        assert!(md_parts.contains("NavigationUnit") && !md_parts.contains("GNSSReceiverTerm"));
    }
}

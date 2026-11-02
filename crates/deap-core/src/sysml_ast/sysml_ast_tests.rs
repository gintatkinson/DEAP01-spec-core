//! Unit tests for SysML v2 AST parsing, extraction, slicing, and persistence. [DOC-TIER: A]
//! Safety & Traceability: Realises: [REQ-0101], [REQ-0801]
//! Contract: Deterministic SysML v2 AST unit-test suite covering grammar, queries, and digests.
//! Bounds & Faults: Bounded O(N) test execution; fails closed on assertion or parse mismatches.

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_parse_part_defs_and_ports() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
package SystemModel_SSOT {
    doc /* SSOT for Autonomous System Architecture and Safety Model */

    attribute actuatorCount : Integer = 4;
    attribute topologyConfiguration : String = "SymmetricTopology";
    attribute transientLoadLimit : Real = 12.0;
    attribute maxLoadFactor : Real = 12.0;

    part def StructuralFrame {
        attribute massKg : Real = 25.0;
    }

    part def PrimaryControlUnit {
        port ctrl_bus : SyncBus;
        port telemetry : TelemetryLink;
    }
}
"#;
        let model = parse_sysml(sysml_source);
        assert_eq!(model.package_name.as_deref(), Some("SystemModel_SSOT"));
        assert_eq!(model.parts.len(), 2);

        let frame = model.parts.first().ok_or("frame")?;
        assert_eq!(frame.name, "StructuralFrame");
        assert_eq!(frame.attributes.len(), 1);
        let attr0 = frame.attributes.first().ok_or("attr0")?;
        assert_eq!(attr0.name, "massKg");
        assert_eq!(attr0.type_name, "Real");
        assert_eq!(attr0.default_value.as_deref(), Some("25.0"));

        let ctrl = model.parts.get(1).ok_or("ctrl")?;
        assert_eq!(ctrl.name, "PrimaryControlUnit");
        assert_eq!(ctrl.ports.len(), 2);
        let port0 = ctrl.ports.first().ok_or("port0")?;
        assert_eq!(port0.name, "ctrl_bus");
        assert_eq!(port0.type_name, "SyncBus");
        let port1 = ctrl.ports.get(1).ok_or("port1")?;
        assert_eq!(port1.name, "telemetry");
        assert_eq!(port1.type_name, "TelemetryLink");
        Ok(())
    }

    #[test]
    fn test_parse_actions_and_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
action def ComputeStateTransition {
    in attribute currentState : Vector3;
    out attribute nextState : Vector3;
}

constraint def RateLimitConstraint(v : Real) {
    v <= 60.0;
}
"#;
        let actions = parse_action_defs(sysml_source);
        assert_eq!(actions.len(), 1);
        let act0 = actions.first().ok_or("act0")?;
        assert_eq!(act0.name, "ComputeStateTransition");
        assert_eq!(act0.in_params.len(), 1);
        assert_eq!(act0.in_params.first().ok_or("in0")?.name, "currentState");
        assert_eq!(act0.out_params.len(), 1);
        assert_eq!(act0.out_params.first().ok_or("out0")?.name, "nextState");

        let constraints = parse_constraint_defs(sysml_source);
        assert_eq!(constraints.len(), 1);
        let con0 = constraints.first().ok_or("con0")?;
        assert_eq!(con0.name, "RateLimitConstraint");
        assert_eq!(con0.parameters, vec!["v : Real"]);
        assert_eq!(con0.expression, "v <= 60.0");
        Ok(())
    }

    #[test]
    fn test_issue_428_parse_action_defs_nested_blocks_and_docblocks(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
/* Top-level license comment */
package SystemControl {
    attribute configStr : String = "action def FakeActionInStringLiteral { in a : Int; }";

    /* Unrelated model comment */
    doc /* Primary path planning action */
    action def PlanExecutionSequence {
        in attribute currentState : Vector3;
        if (check_bounds) {
            // Nested compound block with braces
            sub_routine();
        }
        out attribute targetState : Vector3;
        perform ControlProcessor;
        step computeSequence;
        step validateTransition;
    }

    doc /* Semicolon terminated action */
    action def EmergencyStop(in code : Integer);

    /* Comment before action usage */
    action executeTransition;
}
"#;
        let actions = parse_action_defs(sysml_source);
        // FakeActionInStringLiteral must NOT be parsed!
        assert!(!actions
            .iter()
            .any(|a| a.name == "FakeActionInStringLiteral"));

        let plan = actions
            .iter()
            .find(|a| a.name == "PlanExecutionSequence")
            .ok_or("PlanExecutionSequence should exist")?;
        assert_eq!(plan.doc.as_deref(), Some("Primary path planning action"));
        assert!(plan.is_def);
        assert_eq!(plan.in_params.len(), 1);
        assert_eq!(plan.in_params.first().ok_or("in0")?.name, "currentState");
        assert_eq!(plan.out_params.len(), 1);
        assert_eq!(plan.out_params.first().ok_or("out0")?.name, "targetState");
        assert_eq!(plan.performer.as_deref(), Some("ControlProcessor"));
        assert_eq!(plan.steps, vec!["computeSequence", "validateTransition"]);

        let estop = actions
            .iter()
            .find(|a| a.name == "EmergencyStop")
            .ok_or("EmergencyStop should exist")?;
        assert_eq!(estop.doc.as_deref(), Some("Semicolon terminated action"));
        assert!(estop.is_def);
        assert_eq!(estop.in_params.len(), 1);
        assert_eq!(estop.in_params.first().ok_or("in0")?.name, "code");

        let maneuver = actions
            .iter()
            .find(|a| a.name == "executeTransition")
            .ok_or("executeTransition should exist")?;
        assert_eq!(maneuver.doc, None);
        assert!(!maneuver.is_def);

        assert_eq!(actions.len(), 3);
        Ok(())
    }

    #[test]
    fn test_parse_connections_and_states() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
part def CoreSystem {
    state def OperationalState {
        entry InitializeState;
        do UpdateStateEstimator;
        exit ShutdownSensors;
    }

    connection def SensorToController {
        connect sensor.out_port to ctrl.in_port;
    }
}
"#;
        let parts = parse_part_defs(sysml_source);
        assert_eq!(parts.len(), 1);
        let core = parts.first().ok_or("core")?;
        assert_eq!(core.name, "CoreSystem");
        assert_eq!(core.states.len(), 1);
        let st0 = core.states.first().ok_or("st0")?;
        assert_eq!(st0.name, "OperationalState");
        assert_eq!(st0.entry_action.as_deref(), Some("InitializeState"));
        assert_eq!(core.connections.len(), 1);
        let conn0 = core.connections.first().ok_or("conn0")?;
        assert_eq!(conn0.name, "SensorToController");
        assert_eq!(conn0.source_port, "sensor.out_port");
        assert_eq!(conn0.target_port, "ctrl.in_port");
        Ok(())
    }

    #[test]
    fn test_parse_ground_truth_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = r#"
package SystemModel_SSOT {
    doc /* Ground truth reference model for structural parsing */

    attribute redundancyLevel : Integer = 2;

    part def PrimaryControlUnit {
        port ctrl_in : CommandPort;
        attribute cycleRateHz : Real = 50.0;
    }

    part def StateEstimator {
        port sync_pulse_out : SyncPort;
    }
}
"#;
        let model = parse_sysml(fixture);
        assert_eq!(model.package_name.as_deref(), Some("SystemModel_SSOT"));
        assert_eq!(model.parts.len(), 2);
        assert_eq!(
            model.parts.first().ok_or("part0")?.name,
            "PrimaryControlUnit"
        );
        assert_eq!(model.parts.get(1).ok_or("part1")?.name, "StateEstimator");
        Ok(())
    }

    #[test]
    fn test_parse_port_defs_syntax_variants() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            out port rules_out : CompilerRulePort {
            }
            port out token_out : TokenStreamPort;
            in port schema_in : RawSchemaStreamPort {
                attribute protocol_family : String = "ControlBus";
            }
            port in data_in : DataPortType;
            port cmd : Command;
        "#;
        let ports = parse_port_defs(text);
        assert_eq!(ports.len(), 5);
        let p0 = ports.first().ok_or("p0")?;
        assert_eq!(p0.name, "rules_out");
        assert_eq!(p0.direction, "out");
        assert_eq!(p0.type_name, "CompilerRulePort");

        let p1 = ports.get(1).ok_or("p1")?;
        assert_eq!(p1.name, "token_out");
        assert_eq!(p1.direction, "out");
        assert_eq!(p1.type_name, "TokenStreamPort");

        let p2 = ports.get(2).ok_or("p2")?;
        assert_eq!(p2.name, "schema_in");
        assert_eq!(p2.direction, "in");
        assert_eq!(p2.type_name, "RawSchemaStreamPort");

        let p3 = ports.get(3).ok_or("p3")?;
        assert_eq!(p3.name, "data_in");
        assert_eq!(p3.direction, "in");
        assert_eq!(p3.type_name, "DataPortType");

        let p4 = ports.get(4).ok_or("p4")?;
        assert_eq!(p4.name, "cmd");
        assert_eq!(p4.direction, "inout");
        assert_eq!(p4.type_name, "Command");
        Ok(())
    }

    #[test]
    fn test_parse_requirement_defs_with_attributes() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            requirement def REQ_0001_Test {
                id = "REQ-0001";
                text = "System shall do something.";
                attribute uuidv5 : String = "5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9";
                attribute complexity_class : String = "Class P";
            }
        "#;
        let reqs = parse_requirement_defs(text);
        assert_eq!(reqs.len(), 1);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!(r0.req_id, "REQ-0001");
        assert_eq!(r0.attributes.len(), 2);
        let a0 = r0.attributes.first().ok_or("a0")?;
        assert_eq!(a0.name, "uuidv5");
        assert_eq!(
            a0.default_value.as_deref(),
            Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"")
        );
        let a1 = r0.attributes.get(1).ok_or("a1")?;
        assert_eq!(a1.name, "complexity_class");
        assert_eq!(a1.default_value.as_deref(), Some("\"Class P\""));
        Ok(())
    }

    #[test]
    fn test_parse_requirement_defs_constraints_assumptions_derivations(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            requirement def REQ_0032_Test {
                id = "REQ-0032";
                text = "Requirements shall track assumptions, constraints, and derivations.";
                assume ValidSchemaInput;
                require Invariant_REQ_0032_StrictTotalOrdering;
                require Invariant_REQ_0032_Determinism;
                derived from REQ_0031_BaseEmission;
                derived from REQ_0019_IdentifierSanitization;
            }
        "#;
        let reqs = parse_requirement_defs(text);
        assert_eq!(reqs.len(), 1);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!(r0.assumes, vec!["ValidSchemaInput"]);
        assert_eq!(
            r0.requires,
            vec![
                "Invariant_REQ_0032_StrictTotalOrdering",
                "Invariant_REQ_0032_Determinism"
            ]
        );
        assert_eq!(
            r0.derived_from,
            vec!["REQ_0031_BaseEmission", "REQ_0019_IdentifierSanitization"]
        );
        Ok(())
    }

    #[test]
    fn test_parse_requirement_defs_with_encapsulated_constraints(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            requirement def REQ_0001_Universal_Grounding {
                id = "REQ-0001";
                text = "Compiler shall accept schema directory.";
                attribute uuidv5 : String = "d4865863-718a-53a8-bcf6-e070e176378e";
                doc /* \forall \sigma \in Symbols(A), \sigma \in Tokens(S) */
                constraint def Invariant_Universal_Symbol_Grounding;
                assert constraint Invariant_Zero_Hardcoded_Domain_Concepts;
                require Invariant_Universal_Symbol_Grounding;
                require Invariant_Zero_Hardcoded_Domain_Concepts;
            }
        "#;
        let reqs = parse_requirement_defs(text);
        assert_eq!(reqs.len(), 1);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!(r0.constraints.len(), 2);
        let c0 = r0.constraints.first().ok_or("c0")?;
        assert_eq!(c0.name, "Invariant_Universal_Symbol_Grounding");
        assert_eq!(
            c0.doc.as_deref(),
            Some("\\forall \\sigma \\in Symbols(A), \\sigma \\in Tokens(S)")
        );
        assert!(!c0.is_assertion);
        let c1 = r0.constraints.get(1).ok_or("c1")?;
        assert_eq!(c1.name, "Invariant_Zero_Hardcoded_Domain_Concepts");
        assert!(c1.is_assertion);
        Ok(())
    }

    #[test]
    fn test_package_def_recursive_ast_collectors() -> Result<(), Box<dyn std::error::Error>> {
        let mut root = PackageDef {
            name: "Root".to_string(),
            ..Default::default()
        };
        let mut sub = PackageDef {
            name: "Sub".to_string(),
            ..Default::default()
        };
        let mut nested = PackageDef {
            name: "Nested".to_string(),
            ..Default::default()
        };

        sub.capability_defs.push(CapabilityDef {
            name: "SubCap".to_string(),
            ..Default::default()
        });
        nested.part_defs.push(PartDef {
            name: "NestedPart".to_string(),
            ..Default::default()
        });
        nested.use_case_defs.push(UseCaseDef {
            name: "NestedUC".to_string(),
            ..Default::default()
        });
        nested.interaction_defs.push(InteractionDef {
            name: "NestedInter".to_string(),
            ..Default::default()
        });
        sub.action_defs.push(ActionDef {
            name: "SubAction".to_string(),
            ..Default::default()
        });
        nested.constraint_defs.push(ConstraintDef {
            name: "NestedConstraint".to_string(),
            ..Default::default()
        });

        sub.packages.push(nested);
        root.packages.push(sub);

        assert_eq!(root.get_all_packages().len(), 3);
        let caps = root.get_all_capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps.first().ok_or("cap0")?.name, "SubCap");
        let parts = root.get_all_parts();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts.first().ok_or("part0")?.name, "NestedPart");
        let ucs = root.get_all_use_cases();
        assert_eq!(ucs.len(), 1);
        assert_eq!(ucs.first().ok_or("uc0")?.name, "NestedUC");
        let inters = root.get_all_interactions();
        assert_eq!(inters.len(), 1);
        assert_eq!(inters.first().ok_or("inter0")?.name, "NestedInter");
        let acts = root.get_all_actions();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts.first().ok_or("act0")?.name, "SubAction");
        let cons = root.get_all_constraints();
        assert_eq!(cons.len(), 1);
        assert_eq!(cons.first().ok_or("con0")?.name, "NestedConstraint");
        Ok(())
    }

    #[test]
    fn test_parse_test_case_defs_with_assert_constraints() -> Result<(), Box<dyn std::error::Error>>
    {
        let content = r#"
            test case def TC_UCA_33_Verification {
                subject SafetyController;
                verify requirement Assert_UCA_33, Assert_UCA_34;
                objective "Verify safety interlock per UCA-33";
                step trigger_interlock;
            }
        "#;
        let test_cases = parse_test_case_defs(content);
        assert_eq!(test_cases.len(), 1);
        let tc0 = test_cases.first().ok_or("tc0")?;
        assert_eq!(tc0.name, "TC_UCA_33_Verification");
        assert_eq!(tc0.subject_part.as_deref(), Some("SafetyController"));
        assert_eq!(
            tc0.verified_requirements,
            vec!["Assert_UCA_33", "Assert_UCA_34"]
        );
        assert_eq!(
            tc0.objective.as_deref(),
            Some("Verify safety interlock per UCA-33")
        );
        assert_eq!(tc0.test_steps, vec!["trigger_interlock"]);
        Ok(())
    }

    #[test]
    fn test_component_matches_and_clean_names() {
        assert!(component_matches(
            "Primary Transducer",
            "Primary_Transducer"
        ));
        assert!(component_matches("PrimaryController", "PrimaryController"));
        assert!(component_matches("primary_controller", "PrimaryController"));
        assert!(component_matches("StateEstimator", "state_estimator"));
        assert!(component_matches(
            "[PrimaryController]",
            "PrimaryController"
        ));
        assert!(component_matches(
            "**PrimaryController**",
            "PrimaryController"
        ));
        assert!(component_matches(
            "`PrimaryController`",
            "PrimaryController"
        ));
        assert!(!component_matches("PhantomSubsystem", "PrimaryTransducer"));
        assert!(!component_matches("GhostUnit", "PrimaryController"));
    }

    #[test]
    fn test_extract_negated_components_detection() {
        let backup = vec!["backup".to_string()];
        assert_eq!(
            extract_negated_components(r#"attribute backupEnabled = 0;"#),
            backup
        );
        assert_eq!(
            extract_negated_components(r#"attribute backup_system = "none";"#),
            backup
        );
        assert_eq!(
            extract_negated_components(r#"TestPlatform.backupSystem = "No""#),
            backup
        );
        assert_eq!(extract_negated_components("Backup subsystem: No"), backup);
        assert_eq!(
            extract_negated_components("No backup unit installed"),
            backup
        );
        assert_eq!(extract_negated_components("!exists(Backup)"), backup);
        assert!(extract_negated_components(r#"attribute backupEnabled = 1;"#).is_empty());
        assert!(extract_negated_components(r#"attribute backupSystem = "Yes";"#).is_empty());
        assert!(extract_negated_components("Nominal operations").is_empty());
        assert!(extract_negated_components("Retry no more than 3 times").is_empty());
    }

    #[test]
    fn test_is_component_negated_detection() -> Result<(), Box<dyn std::error::Error>> {
        let model = "attribute backupEnabled = 0;\nattribute backupSystem = \"No\";";
        assert!(is_component_negated("AuxiliaryBackupUnit", model));
        assert!(is_component_negated("`AuxiliaryBackupUnit`", model));
        assert!(!is_component_negated("PrimaryController", model));
        assert!(!is_component_negated("StateEstimator", model));
        assert!(!is_component_negated("", model));

        let mut pkg = PackageDef {
            name: "SystemModel_SSOT".to_string(),
            ..Default::default()
        };
        pkg.attribute_defs.push(AttributeDef {
            name: "backupEnabled".to_string(),
            default_value: Some("0".to_string()),
            ..Default::default()
        });
        let mut sub = PackageDef {
            name: "Subsystems".to_string(),
            doc: Some("Redundant module: none".to_string()),
            ..Default::default()
        };
        sub.constraint_defs.push(ConstraintDef {
            name: "DiagnosticsAbsent".to_string(),
            expression: "!exists(DiagnosticsService)".to_string(),
            ..Default::default()
        });
        pkg.packages.push(sub);

        let stems = pkg.extract_negated_components();
        assert_eq!(stems, vec!["backup", "diagnosticsservice", "redundant"]);
        assert!(pkg.is_component_negated("AuxiliaryBackupUnit"));
        assert!(pkg.is_component_negated("RedundantModule"));
        assert!(pkg.is_component_negated("DiagnosticsService"));
        assert!(!pkg.is_component_negated("PrimaryController"));
        assert!(stems.first().ok_or("stem0")?.len() >= 3);
        Ok(())
    }

    #[test]
    fn test_numeric_state_attributes_do_not_negate_components() {
        let model = "attribute faultCount = 0;\nattribute retryLimit = 0;";
        assert!(extract_negated_components(model).is_empty());
        assert!(!is_component_negated("FaultMonitor", model));
        assert!(!is_component_negated("RetryHandler", model));

        let mut pkg = PackageDef {
            name: "SystemModel_SSOT".to_string(),
            ..Default::default()
        };
        for name in ["faultCount", "retryLimit"] {
            pkg.attribute_defs.push(AttributeDef {
                name: name.to_string(),
                default_value: Some("0".to_string()),
                ..Default::default()
            });
        }
        assert!(pkg.extract_negated_components().is_empty());
        assert!(!pkg.is_component_negated("FaultMonitor"));
        assert!(!pkg.is_component_negated("RetryHandler"));

        // Control: an explicit `has<Component>` presence flag still negates the component.
        let control = "attribute hasFaultMonitor = false;";
        assert_eq!(
            extract_negated_components(control),
            vec!["faultmonitor".to_string()]
        );
        assert!(is_component_negated("FaultMonitor", control));
        assert!(!is_component_negated("RetryHandler", control));

        // Morpheme boundaries: `has` inside `hash` is not a prefix, and scalar/metrology stems are not components.
        let boundary = "attribute hashSeed = 0;\nattribute timeUnit = 0;";
        assert!(extract_negated_components(boundary).is_empty());
        assert!(!is_component_negated("HashSeedGenerator", boundary));
        assert!(!is_component_negated("TimeSource", boundary));

        let mut boundary_pkg = PackageDef {
            name: "SystemModel_SSOT".to_string(),
            ..Default::default()
        };
        for name in ["hashSeed", "timeUnit"] {
            boundary_pkg.attribute_defs.push(AttributeDef {
                name: name.to_string(),
                default_value: Some("0".to_string()),
                ..Default::default()
            });
        }
        assert!(boundary_pkg.extract_negated_components().is_empty());
        assert!(!boundary_pkg.is_component_negated("HashSeedGenerator"));
        assert!(!boundary_pkg.is_component_negated("TimeSource"));

        // Control: a `has_<component>` snake-case presence flag still negates the component.
        let snake_control = "attribute has_backup_module = false;";
        assert_eq!(
            extract_negated_components(snake_control),
            vec!["backup".to_string()]
        );
    }

    #[test]
    fn test_multi_clause_prose_negation_extracts_every_clause() {
        let prose = "Negative safety model invariant: backupEnabled = 0, redundantSystem = \"No\"";
        let stems = extract_negated_components(prose);
        assert!(stems.contains(&"backup".to_string()), "stems: {stems:?}");
        assert!(stems.contains(&"redundant".to_string()), "stems: {stems:?}");
        assert!(is_component_negated("AuxiliaryBackupUnit", prose));
        assert!(!is_component_negated("PrimaryProcessor", prose));

        // Dotted owner paths in a labelled clause yield the leaf key token only.
        let dotted =
            extract_negated_components("Invariant: Owner.auxSystem = 0, redundantSystem = \"No\"");
        assert!(dotted.contains(&"aux".to_string()), "stems: {dotted:?}");
        assert!(
            !dotted.contains(&"owneraux".to_string()),
            "stems: {dotted:?}"
        );
    }

    #[test]
    fn test_unified_schema_model_extractor_all_formats() {
        // 1. SysML v2
        let sysml = r#"
            /* multi-line comment */
            package SampleSystem {
                // single line comment
                part def ControlProcessor;
                part StateEstimator :> SensorBase;
            }
        "#;
        let sysml_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", sysml);
        assert!(sysml_parts.contains("ControlProcessor"));
        assert!(sysml_parts.contains("StateEstimator"));

        // 2. YAML
        let yaml = r#"
            subsystems:
              - ControlProcessor
              - SequencingSystem
            components:
              - name: PowerDistributionUnit
                id: PDU-01
        "#;
        let yaml_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yaml", yaml);
        assert!(yaml_parts.contains("ControlProcessor"));
        assert!(yaml_parts.contains("SequencingSystem"));
        assert!(yaml_parts.contains("PowerDistributionUnit"));

        // 3. JSON
        let json = r#"
            {
                "schema_nodes": ["part:TelemetryTransceiver", "port:TX"],
                "components": [
                    {"name": "ThermalController", "id": "TCU"}
                ]
            }
        "#;
        let json_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", json);
        assert!(json_parts.contains("TelemetryTransceiver"));
        assert!(json_parts.contains("ThermalController"));

        // 4. Markdown architecture table
        let md = r#"
            # System Architecture

            ### 4.3 Subsystem Architecture
            | Subsystem | Function | Allocation |
            |---|---|---|
            | RedundantModule | Core processing | Segment A |
            | ActuatorControllerUnit | Output control | Segment B |
        "#;
        let md_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", md);
        assert!(md_parts.contains("RedundantModule"));
        assert!(md_parts.contains("ActuatorControllerUnit"));

        // 5. Protobuf
        let proto = r#"
            syntax = "proto3";
            package telemetry_pkg;
            service DiagnosticsService {
                rpc GetState (Empty) returns (State);
            }
            message SensorData {
                int64 timestamp = 1;
            }
        "#;
        let proto_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".proto", proto);
        assert!(proto_parts.contains("DiagnosticsService"));
        assert!(proto_parts.contains("SensorData"));

        // 6. OMG IDL
        let idl = r#"
            module SystemBus {
                interface BusController {
                    void transmit();
                };
            };
        "#;
        let idl_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".idl", idl);
        assert!(idl_parts.contains("BusController"));
        assert!(idl_parts.contains("SystemBus"));

        // 7. ARXML
        let arxml = r#"
            <AUTOSAR>
                <SW-COMPONENT-PROTOTYPE>
                    <SHORT-NAME>ProcessController</SHORT-NAME>
                </SW-COMPONENT-PROTOTYPE>
            </AUTOSAR>
        "#;
        let arxml_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".arxml", arxml);
        assert!(arxml_parts.contains("ProcessController"));
    }

    #[test]
    fn test_unified_schema_model_extractor_yang_container_grouping_list() {
        let yang = r#"
            module sample_module {
                yang-version 1.1;
                prefix sm;
                grouping EndpointSettings {
                    leaf label { type string; }
                }
                container NodeRegistry {
                    list NodeEntry {
                        key "label";
                        uses EndpointSettings;
                    }
                }
                typedef counter_kind { type uint32; }
                container access-table { leaf size { type uint32; } }
            }
        "#;
        let yang_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yang", yang);
        assert!(
            yang_parts.contains("EndpointSettings"),
            "parts: {yang_parts:?}"
        );
        assert!(yang_parts.contains("NodeRegistry"), "parts: {yang_parts:?}");
        assert!(yang_parts.contains("NodeEntry"), "parts: {yang_parts:?}");
        assert!(
            !yang_parts.contains("sample_module"),
            "parts: {yang_parts:?}"
        );
        assert!(
            !yang_parts.contains("counter_kind"),
            "parts: {yang_parts:?}"
        );
        assert!(!yang_parts.contains("label"), "parts: {yang_parts:?}");
        let upper = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".YANG", yang);
        assert_eq!(upper, yang_parts);
        assert!(UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS.contains(&".yang"));
    }

    #[test]
    fn test_parse_calc_defs_syntax_variants() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
            doc /* Calculates kinetic energy */
            calc def KineticEnergy(in massKg : Real, in speedMps : Real) : Real {
                0.5 * massKg * speedMps * speedMps
            }

            calc def VelocityCalc {
                in v0 : Real;
                in a : Real;
                in t : Real;
                return v : Real;
                v0 + a * t;
            }

            calc MarginCalculation;
        "#;

        let calcs = parse_calc_defs(content);
        assert_eq!(calcs.len(), 3);

        let ke = calcs.first().ok_or("ke")?;
        assert_eq!(ke.name, "KineticEnergy");
        assert_eq!(ke.doc.as_deref(), Some("Calculates kinetic energy"));
        assert_eq!(ke.in_params.len(), 2);
        let ke_p0 = ke.in_params.first().ok_or("ke_p0")?;
        assert_eq!(ke_p0.name, "massKg");
        assert_eq!(ke_p0.type_name, "Real");
        let ke_p1 = ke.in_params.get(1).ok_or("ke_p1")?;
        assert_eq!(ke_p1.name, "speedMps");
        assert_eq!(ke_p1.type_name, "Real");
        assert_eq!(ke.return_type.as_deref(), Some("Real"));
        assert_eq!(
            ke.expression.as_deref(),
            Some("0.5 * massKg * speedMps * speedMps")
        );

        let vc = calcs.get(1).ok_or("vc")?;
        assert_eq!(vc.name, "VelocityCalc");
        assert_eq!(vc.in_params.len(), 3);
        assert_eq!(vc.in_params.first().ok_or("vc_p0")?.name, "v0");
        assert_eq!(vc.in_params.get(1).ok_or("vc_p1")?.name, "a");
        assert_eq!(vc.in_params.get(2).ok_or("vc_p2")?.name, "t");
        assert_eq!(vc.return_type.as_deref(), Some("Real"));
        assert_eq!(vc.expression.as_deref(), Some("v0 + a * t"));

        let mc = calcs.get(2).ok_or("mc")?;
        assert_eq!(mc.name, "MarginCalculation");
        assert_eq!(mc.in_params.len(), 0);
        assert!(mc.return_type.is_none());
        assert!(mc.expression.is_none());
        Ok(())
    }

    #[test]
    fn test_package_def_queries_and_nested_ports() -> Result<(), Box<dyn std::error::Error>> {
        let mut root = PackageDef {
            name: "RootSystem".to_string(),
            calc_defs: vec![CalcDef {
                name: "RootCalc".to_string(),
                ..Default::default()
            }],
            port_defs: vec![PortDef {
                name: "root_port".to_string(),
                ..Default::default()
            }],
            hazard_defs: vec![HazardDef {
                name: "H-01".to_string(),
                severity: 3,
                ..Default::default()
            }],
            risk_defs: vec![RiskDef {
                name: "R-01".to_string(),
                severity: 3,
                ..Default::default()
            }],
            state_defs: vec![StateDef {
                name: "Nominal".to_string(),
                ..Default::default()
            }],
            connection_defs: vec![ConnectionDef {
                name: "C-01".to_string(),
                source_port: "p1".to_string(),
                target_port: "p2".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let sub_part = PartDef {
            name: "NestedSensor".to_string(),
            ports: vec![PortDef {
                name: "sync_pulse_out".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let top_part = PartDef {
            name: "ControlProcessor".to_string(),
            ports: vec![PortDef {
                name: "ctrl_bus".to_string(),
                ..Default::default()
            }],
            parts: vec![sub_part],
            calcs: vec![CalcDef {
                name: "InnerCalc".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        root.part_defs.push(top_part);

        // Test get_all_calcs
        let calcs = root.get_all_calcs();
        assert_eq!(calcs.len(), 2);
        let calc_names: Vec<&str> = calcs.iter().map(|c| c.name.as_str()).collect();
        assert!(calc_names.contains(&"RootCalc"));
        assert!(calc_names.contains(&"InnerCalc"));

        // Test get_all_ports
        let ports = root.get_all_ports();
        assert_eq!(ports.len(), 3);
        let port_names: Vec<&str> = ports.iter().map(|p| p.name.as_str()).collect();
        assert!(port_names.contains(&"root_port"));
        assert!(port_names.contains(&"ctrl_bus"));
        assert!(port_names.contains(&"sync_pulse_out"));

        // Test get_all_hazards
        let hazards = root.get_all_hazards();
        assert_eq!(hazards.len(), 1);
        assert_eq!(hazards.first().ok_or("h0")?.name, "H-01");

        // Test get_all_risks
        let risks = root.get_all_risks();
        assert_eq!(risks.len(), 1);
        assert_eq!(risks.first().ok_or("r0")?.name, "R-01");

        // Test get_all_states
        let states = root.get_all_states();
        assert_eq!(states.len(), 1);
        assert_eq!(states.first().ok_or("s0")?.name, "Nominal");

        // Test get_all_connections
        let conns = root.get_all_connections();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns.first().ok_or("c0")?.name, "C-01");

        // Test get_nested_ports: ONLY sync_pulse_out is nested; ctrl_bus and root_port are top-level!
        let nested_ports = root.get_nested_ports();
        assert_eq!(nested_ports.len(), 1);
        assert!(nested_ports.contains("sync_pulse_out"));
        assert!(!nested_ports.contains("ctrl_bus"));
        assert!(!nested_ports.contains("root_port"));
        Ok(())
    }

    #[test]
    fn test_sysml_ast_reexports_and_identifier_primitives_roundtrip() -> Result<(), String> {
        let masked = mask_string_literals("part def FlightCtrl { doc /* \"inner\" */ }");
        let open_idx = masked.find('{').ok_or("missing opening brace")?;
        let close_idx = find_matching_brace(&masked, open_idx).ok_or("missing closing brace")?;
        assert_eq!(masked.as_bytes().get(close_idx).copied(), Some(b'}'));
        assert!(is_valid_subsystem_identifier("FlightCtrl"));
        assert!(!is_valid_subsystem_identifier("package"));
        Ok(())
    }

    #[test]
    fn test_supported_schema_extension_enum_cardinality_order_and_catalog_parity() {
        let expected = vec![
            ".sysml",
            ".yaml",
            ".yml",
            ".json",
            ".proto",
            ".idl",
            ".arxml",
            ".md",
            ".markdown",
            ".yang",
        ];
        assert_eq!(SupportedSchemaExtension::ALL.len(), 10);
        let via_enum: Vec<&'static str> = SupportedSchemaExtension::ALL
            .iter()
            .map(|x| x.as_str())
            .collect();
        assert_eq!(via_enum, expected);
        let catalog = UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS;
        let via_catalog: Vec<&'static str> = catalog.iter().collect();
        assert_eq!(via_catalog, expected);
        for x in SupportedSchemaExtension::ALL {
            assert_eq!(x.as_str_ref(), x.as_str());
            assert!(catalog.contains(&x.as_str()));
        }
        let owned = String::from(".yaml");
        assert!(catalog.contains(&owned.as_str()));
        assert!(!catalog.contains(&".txt"));
        assert!(!catalog.contains(&"sysml"));
        assert!(!catalog.contains(&".SYSML"));
        assert!(!catalog.contains(&""));
    }

    #[test]
    fn test_supported_extensions_catalog_exact_size_and_double_ended_iter() {
        let mut iter = UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS.iter();
        assert_eq!(iter.len(), 10);
        assert_eq!(iter.next(), Some(".sysml"));
        assert_eq!(iter.next_back(), Some(".yang"));
        assert_eq!(iter.len(), 8);
    }

    #[test]
    fn test_parse_satisfied_requirements_and_part_defs_extraction() {
        let snippet = r#"
            part def SensorController {
                satisfy requirement REQ-CTRL-001;
                satisfy requirement "REQ-CTRL-002";
                satisfy REQ-PWR-03 by PowerBus;
                satisfy requirement REQ-CTRL-001;
                satisfy /* comment 1 */ requirement REQ-COM-04;
                satisfy/*comment 2*/requirement REQ-COM-05;
                satisfy requirement /*comment 3*/ REQ-COM-06;
                satisfy requirement REQ-COM-07 /*comment 4*/;
                satisfy requirement REQ-COM-08 /* c1 */ by /* c2 */ PowerBus /* c3 */;
            }
        "#;
        let satisfied = parse_satisfied_requirements(snippet);
        assert_eq!(
            satisfied,
            vec![
                "REQ-CTRL-001".to_string(),
                "REQ-CTRL-002".to_string(),
                "REQ-PWR-03".to_string(),
                "REQ-COM-04".to_string(),
                "REQ-COM-05".to_string(),
                "REQ-COM-06".to_string(),
                "REQ-COM-07".to_string(),
                "REQ-COM-08".to_string(),
            ]
        );

        let parts = parse_part_defs(snippet);
        assert_eq!(parts.len(), 1);
        assert_eq!(
            parts.first().map(|p| p.name.as_str()),
            Some("SensorController")
        );
        assert_eq!(
            parts.first().map(|p| p.satisfied_requirements.clone()),
            Some(vec![
                "REQ-CTRL-001".to_string(),
                "REQ-CTRL-002".to_string(),
                "REQ-PWR-03".to_string(),
                "REQ-COM-04".to_string(),
                "REQ-COM-05".to_string(),
                "REQ-COM-06".to_string(),
                "REQ-COM-07".to_string(),
                "REQ-COM-08".to_string(),
            ])
        );
    }

    #[test]
    fn test_sysml_ast_attribute_and_flow_serde_defaults_roundtrip() -> Result<(), String> {
        let attr: AttributeDef = serde_json::from_str(r#"{"name":"pressure","type_name":"Real"}"#)
            .map_err(|e| e.to_string())?;
        assert_eq!(attr.name, "pressure");
        assert!(attr.default_value.is_none());

        let flow: FlowDef =
            serde_json::from_str(r#"{"name":"telemetry_tx"}"#).map_err(|e| e.to_string())?;
        assert_eq!(flow.direction, "out");
        assert_eq!(flow.item_type, "Item");
        Ok(())
    }

    #[test]
    fn test_parse_triggers_hazard_safety_realises_and_layer_annotations() -> Result<(), String> {
        let sysml = r#"
            package SafetyPkg {
                part def FlightActuator {
                    @TriggersHazard { hazard = "H-01"; }
                    @TriggersHazard("H-02")
                    @SafetyRealises { requirement = "SC-01"; }
                    @SafetyRealises("SC-02")
                    @Layer { name = "Domain"; }
                    attribute position_deg : Real = 0.0;
                }

                requirement def SC-01 {
                    doc /* Prevent uncommanded actuator deflection. */
                    @TriggersHazard("H-01")
                    @SafetyRealises { requirement = "SYS-REQ-01"; }
                    @Layer("Safety")
                }
            }
        "#;

        let model = parse_sysml_model(sysml);
        assert_eq!(model.package_name.as_deref(), Some("SafetyPkg"));
        let part = model.parts.first().ok_or("Expected FlightActuator")?;
        assert_eq!(part.name, "FlightActuator");
        assert_eq!(
            part.triggers_hazards,
            vec!["H-01".to_string(), "H-02".to_string()]
        );
        assert_eq!(
            part.safety_realises,
            vec!["SC-01".to_string(), "SC-02".to_string()]
        );
        assert_eq!(part.layer, Some("Domain".to_string()));

        let req = model.requirements.first().ok_or("Expected SC-01")?;
        assert_eq!(req.name, "SC-01");
        assert_eq!(req.req_id, "SC-01");
        assert_eq!(req.triggers_hazards, vec!["H-01".to_string()]);
        assert_eq!(req.safety_realises, vec!["SYS-REQ-01".to_string()]);
        assert_eq!(req.layer, Some("Safety".to_string()));
        Ok(())
    }

    #[test]
    fn test_slice_package_subsystem_within_budget_and_overflow() -> Result<(), String> {
        let sysml = r#"
            package FlightGuidance {
                part def GuidanceComputer {
                    @TriggersHazard("H-01")
                    @SafetyRealises("SC-01")
                    @Layer("Domain")
                    in port nav_in : NavBusPort;
                    out port cmd_out : ActuatorCmdPort;
                    satisfy requirement REQ-FG-01;
                }
                connection nav_to_cmd connect nav_in to cmd_out;
                requirement def REQ-FG-01 {
                    doc /* Compute guidance law at 250 Hz. */
                }
                requirement def SC-01 {
                    doc /* Bound pitch rate command within envelope. */
                }
                hazard def H-01 {
                    doc /* Uncommanded pitch excursion. */
                }
            }
            package TelemetryLogging {
                part def TelemetryRecorder {
                    attribute buffer_kb : Integer = 512;
                }
            }
        "#;

        let model = parse_sysml_model(sysml);
        let slice = model.slice_package_subsystem("FlightGuidance", 4096)?;
        assert_eq!(slice.target_package, "FlightGuidance");
        assert_eq!(slice.packages.len(), 1);
        assert_eq!(slice.parts.len(), 1);
        assert_eq!(slice.ports.len(), 2);
        assert_eq!(slice.interfaces.len(), 1);
        assert_eq!(slice.requirements.len(), 2);
        assert_eq!(slice.hazards.len(), 1);
        assert!(slice.estimated_tokens > 0 && slice.estimated_tokens <= 4096);

        let overflow = model.slice_package_subsystem("FlightGuidance", 1);
        assert!(
            overflow.is_err(),
            "Expected budget overflow error when max_tokens == 1"
        );

        let zero_budget = model.slice_package_subsystem("FlightGuidance", 0);
        assert!(zero_budget.is_err(), "Expected error when max_tokens == 0");

        let missing = model.slice_package_subsystem("NonExistentSubsystem", 4096);
        assert!(
            missing.is_err(),
            "Expected error when neither package nor part matches"
        );
        Ok(())
    }

    #[test]
    fn test_incremental_ast_cache_miss_and_hit() -> Result<(), String> {
        let tmp_root =
            std::env::temp_dir().join(format!("deap_core_ast_cache_test_{}", uuid::Uuid::new_v4()));
        let cache = IncrementalAstCache::new(&tmp_root);
        assert_eq!(
            cache.cache_dir,
            tmp_root.join(".pipeline").join("cache").join("ast")
        );

        let digest_abc = IncrementalAstCache::compute_digest("abc");
        assert_eq!(
            digest_abc,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        let sysml_source = r#"
            package CacheTestPkg {
                part def CachedSensor {
                    @Layer("Domain")
                    attribute sample_rate_hz : Real = 100.0;
                }
            }
        "#;

        let (model_miss, was_hit_first) = cache.load_or_parse(sysml_source)?;
        assert!(!was_hit_first, "First call must be a cache miss");
        assert_eq!(model_miss.package_name.as_deref(), Some("CacheTestPkg"));
        assert_eq!(model_miss.parts.len(), 1);

        let (model_hit, was_hit_second) = cache.load_or_parse(sysml_source)?;
        assert!(was_hit_second, "Second call must be a cache hit");
        assert_eq!(model_miss, model_hit);

        std::fs::remove_dir_all(&tmp_root).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn test_type_descriptor_from_part_def() -> Result<(), String> {
        let sysml = r#"
            package AvionicsPkg {
                part def AirDataSensor {
                    @Layer("Domain")
                    attribute airspeed_mps : Real = 85.0 [m/s] [0.0..350.0];
                    attribute valid_flag : Boolean = true;
                    satisfy requirement REQ-ADS-01;
                }
            }
        "#;
        let model = parse_sysml_model(sysml);
        let pkg_name = model
            .package_name
            .as_deref()
            .ok_or("Expected AvionicsPkg")?;
        let part = model.parts.first().ok_or("Expected AirDataSensor")?;
        let td = TypeDescriptor::from_part_def(pkg_name, part);
        assert_eq!(td.type_name, "AirDataSensor");
        assert_eq!(td.package_name, "AvionicsPkg");
        assert_eq!(td.layer, Some("Domain".to_string()));
        assert_eq!(td.satisfied_requirements, vec!["REQ-ADS-01".to_string()]);
        assert_eq!(td.fields.len(), 2);
        let first_field = td.fields.first().ok_or("Expected airspeed_mps field")?;
        assert_eq!(first_field.name, "airspeed_mps");
        assert_eq!(first_field.type_name, "Real");
        assert_eq!(first_field.si_unit, Some("m/s".to_string()));
        assert_eq!(first_field.range_bounds, Some((0.0, 350.0)));
        Ok(())
    }

    #[test]
    fn test_ast_persistence_engine_and_cot_audit_log() -> Result<(), String> {
        let tmp_root = std::env::temp_dir().join(format!(
            "deap_core_persistence_test_{}",
            uuid::Uuid::new_v4()
        ));
        let engine = AstPersistenceEngine::new(tmp_root.clone());
        let model = parse_sysml_model(
            r#"
            package PersistedPkg {
                part def PersistentNode {
                    attribute node_id : Integer = 7;
                }
            }
        "#,
        );
        let saved_path = engine.persist_model("model_alpha", &model)?;
        assert!(saved_path.is_file());
        let loaded = engine.load_model("model_alpha")?;
        assert_eq!(loaded, model);
        assert!(engine.load_model("missing_model").is_err());

        let mut audit_log = CotAuditLog::new();
        audit_log.append_record(CotAuditRecord {
            step_id: "STEP-01".to_string(),
            model_id: "deepseek-r1-local".to_string(),
            prompt_sha256: IncrementalAstCache::compute_digest("prompt-payload"),
            reasoning_trace: "Verified PartDef PersistentNode invariants.".to_string(),
            verified_ast_refs: vec!["PersistedPkg::PersistentNode".to_string()],
        });
        assert_eq!(audit_log.records.len(), 1);
        let json_str = audit_log.to_json_pretty()?;
        assert!(json_str.contains("STEP-01"));
        assert!(json_str.contains("PersistedPkg::PersistentNode"));

        std::fs::remove_dir_all(&tmp_root).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Verifies `UnifiedSchemaModelExtractor::discover_schema_files` excludes `.digest.json` sidecars.
    #[test]
    fn test_issue_549_discover_schema_files_excludes_digest_json_sidecars() -> Result<(), String> {
        let tmp_dir =
            std::env::temp_dir().join(format!("deap_core_issue_549_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        let files = [
            "blueprint-01-alpha.sysml",
            "blueprint-01-alpha.digest.json",
            "schema-digest.json",
            "schema_digest.json",
            "digest.json",
            "valid_schema.json",
        ];
        for fname in &files {
            std::fs::write(tmp_dir.join(fname), "{}").map_err(|e| e.to_string())?;
        }

        let discovered =
            UnifiedSchemaModelExtractor::discover_schema_files(std::slice::from_ref(&tmp_dir));
        std::fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

        let discovered_names: Vec<String> = discovered
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
            .collect();

        assert!(
            discovered_names.contains(&"blueprint-01-alpha.sysml".to_string()),
            "Expected blueprint-01-alpha.sysml in discovered: {:?}",
            discovered_names
        );
        assert!(
            discovered_names.contains(&"valid_schema.json".to_string()),
            "Expected valid_schema.json in discovered: {:?}",
            discovered_names
        );
        assert!(
            !discovered_names.contains(&"blueprint-01-alpha.digest.json".to_string()),
            "Did not expect blueprint-01-alpha.digest.json in discovered: {:?}",
            discovered_names
        );
        assert!(
            !discovered_names.contains(&"schema-digest.json".to_string()),
            "Did not expect schema-digest.json in discovered: {:?}",
            discovered_names
        );
        assert!(
            !discovered_names.contains(&"schema_digest.json".to_string()),
            "Did not expect schema_digest.json in discovered: {:?}",
            discovered_names
        );
        assert!(
            !discovered_names.contains(&"digest.json".to_string()),
            "Did not expect digest.json in discovered: {:?}",
            discovered_names
        );
        Ok(())
    }

    /// Verifies `extract_subsystem_parts_from_text` ignores `doc /* ... */` prose and `SchemaDigest` JSON sidecars.
    #[test]
    fn test_issue_551_extract_subsystem_parts_ignores_doc_prose_and_schema_digest() {
        let sysml_src = r#"
package FlightPkg {
    doc /* This module is part Boundaries of the counterpart TelemetryBus and part Hierarchy. */
    part def FlightDirector {
        part actuator_unit : ActuatorNode;
    }
}
"#;
        let sysml_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", sysml_src);
        assert!(
            sysml_parts.contains("FlightDirector"),
            "Expected FlightDirector in extracted parts: {:?}",
            sysml_parts
        );
        assert!(
            sysml_parts.contains("actuator_unit"),
            "Expected actuator_unit in extracted parts: {:?}",
            sysml_parts
        );
        assert!(
            !sysml_parts.contains("Boundaries"),
            "Did not expect doc prose word Boundaries in extracted parts: {:?}",
            sysml_parts
        );
        assert!(
            !sysml_parts.contains("TelemetryBus"),
            "Did not expect counterpart substring TelemetryBus in extracted parts: {:?}",
            sysml_parts
        );
        assert!(
            !sysml_parts.contains("Hierarchy"),
            "Did not expect doc prose word Hierarchy in extracted parts: {:?}",
            sysml_parts
        );

        let digest_json = r#"{
  "format_version": "1.0",
  "source_hash": "deadbeef",
  "parts": [{"name": "DigestGhostPart"}],
  "modules": [{"name": "DigestGhostModule"}]
}"#;
        let json_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", digest_json);
        assert!(
            json_parts.is_empty(),
            "Expected SchemaDigest JSON to yield empty parts set, got: {:?}",
            json_parts
        );
    }

    /// Verifies `extract_subsystem_parts_from_text` ignores Markdown tables without a subsystem header column.
    #[test]
    fn test_issue_553_extract_subsystem_parts_ignores_unrelated_markdown_tables() {
        let md_src = r#"
# Architecture Overview

| ID | Subsystem | Owner |
| --- | --- | --- |
| 1 | NavigationUnit | Avionics |

## Glossary

| Acronym | Meaning |
| --- | --- |
| GNSSReceiverTerm | Global Navigation Satellite System |
| INSDriftTerm | Inertial Navigation System Drift |
"#;
        let md_parts =
            UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", md_src);
        assert!(
            md_parts.contains("NavigationUnit"),
            "Expected NavigationUnit in extracted parts: {:?}",
            md_parts
        );
        assert!(
            !md_parts.contains("GNSSReceiverTerm"),
            "Did not expect glossary term GNSSReceiverTerm in extracted parts: {:?}",
            md_parts
        );
        assert!(
            !md_parts.contains("INSDriftTerm"),
            "Did not expect glossary term INSDriftTerm in extracted parts: {:?}",
            md_parts
        );
    }

    /// Verifies `parse_state_defs` roundtrips nested `sub_states` and `transitions` with `(name, source, target, trigger)`.
    #[test]
    fn test_parse_state_defs_roundtrips_sub_states_and_transitions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
state def BP_01_LifecycleState {
    state Uninitialized;
    state Active;
    transition Initial_to_Uninitialized;
    transition Uninitialized_to_Active_on_parse_token;
    transition Active_to_Terminal;
}
"#;
        let states = parse_state_defs(src);
        assert_eq!(states.len(), 1);
        let st = states.first().ok_or("missing top-level StateDef")?;
        assert_eq!(st.name, "BP_01_LifecycleState");
        assert_eq!(st.sub_states.len(), 2);
        let sub0 = st.sub_states.first().ok_or("missing sub_state 0")?;
        let sub1 = st.sub_states.get(1).ok_or("missing sub_state 1")?;
        assert_eq!(sub0.name, "Uninitialized");
        assert_eq!(sub1.name, "Active");

        assert_eq!(st.transitions.len(), 3);
        let tr0 = st.transitions.first().ok_or("missing transition 0")?;
        assert_eq!(tr0.name.as_deref(), Some("Initial_to_Uninitialized"));
        assert_eq!(tr0.source.as_deref(), Some("Initial"));
        assert_eq!(tr0.target.as_deref(), Some("Uninitialized"));
        assert_eq!(tr0.trigger.as_deref(), None);

        let tr1 = st.transitions.get(1).ok_or("missing transition 1")?;
        assert_eq!(
            tr1.name.as_deref(),
            Some("Uninitialized_to_Active_on_parse_token")
        );
        assert_eq!(tr1.source.as_deref(), Some("Uninitialized"));
        assert_eq!(tr1.target.as_deref(), Some("Active"));
        assert_eq!(tr1.trigger.as_deref(), Some("parse_token"));

        let tr2 = st.transitions.get(2).ok_or("missing transition 2")?;
        assert_eq!(tr2.name.as_deref(), Some("Active_to_Terminal"));
        assert_eq!(tr2.source.as_deref(), Some("Active"));
        assert_eq!(tr2.target.as_deref(), Some("Terminal"));
        assert_eq!(tr2.trigger.as_deref(), None);

        Ok(())
    }

    /// Verifies `compute_sha256_hex` and `IncrementalAstCache::compute_digest` match `crate::digest::compute_sha256` across FIPS 180-4 vectors.
    #[test]
    fn test_issue_494_compute_sha256_hex_delegates_to_canonical_digest() {
        for sample in [
            "",
            "abc",
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
        ] {
            assert_eq!(
                compute_sha256_hex(sample.as_bytes()),
                crate::digest::compute_sha256(sample.as_bytes())
            );
            assert_eq!(
                IncrementalAstCache::compute_digest(sample),
                crate::digest::compute_sha256(sample.as_bytes())
            );
        }
    }

    /// Verifies `UnifiedSchemaModelExtractor::discover_schema_files` preserves depth-0 dot-prefixed root directories while filtering hidden entries at depth >= 1.
    #[test]
    fn test_issue_503_discover_schema_files_preserves_depth_zero_dot_root() -> Result<(), String> {
        let tmp_dir = std::env::temp_dir().join(format!(
            ".deap_core_issue_503_dot_root_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        let visible_file = tmp_dir.join("visible.sysml");
        let hidden_file = tmp_dir.join(".hidden.sysml");
        let hidden_subdir = tmp_dir.join(".hidden_dir");
        std::fs::create_dir_all(&hidden_subdir).map_err(|e| e.to_string())?;
        let hidden_nested = hidden_subdir.join("nested.sysml");

        std::fs::write(&visible_file, "package VisiblePkg { part def NodeA; }")
            .map_err(|e| e.to_string())?;
        std::fs::write(&hidden_file, "package HiddenPkg { part def NodeB; }")
            .map_err(|e| e.to_string())?;
        std::fs::write(
            &hidden_nested,
            "package HiddenNestedPkg { part def NodeC; }",
        )
        .map_err(|e| e.to_string())?;

        let discovered =
            UnifiedSchemaModelExtractor::discover_schema_files(std::slice::from_ref(&tmp_dir));
        std::fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

        let discovered_names: Vec<String> = discovered
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
            .collect();

        assert!(
            discovered_names.contains(&"visible.sysml".to_string()),
            "Expected visible.sysml inside depth-0 dot-prefixed root to be discovered, got: {:?}",
            discovered_names
        );
        assert!(
            !discovered_names.contains(&".hidden.sysml".to_string()),
            "Did not expect depth-1 .hidden.sysml in discovered: {:?}",
            discovered_names
        );
        assert!(
            !discovered_names.contains(&"nested.sysml".to_string()),
            "Did not expect depth-2 nested.sysml inside .hidden_dir in discovered: {:?}",
            discovered_names
        );
        Ok(())
    }

    /// Verifies `parse_state_defs` parses up to `MAX_AST_DEPTH` (64) and truncates `MAX_AST_DEPTH + 1` (65) cleanly.
    #[test]
    fn test_parse_state_defs_enforces_max_ast_depth_and_plus_one_bounds() -> Result<(), String> {
        const EXPECTED_MAX_DEPTH: usize = 64;
        let mut nested_sysml = format!("state Leaf_{};", EXPECTED_MAX_DEPTH.saturating_add(1));
        for level in (0..=EXPECTED_MAX_DEPTH).rev() {
            nested_sysml = format!("state def StateNode_{level} {{ {nested_sysml} }}");
        }
        let parsed = parse_state_defs(&nested_sysml);
        let mut cursor = parsed.first().ok_or("missing root StateNode_0")?;
        for expected_level in 0..=EXPECTED_MAX_DEPTH {
            assert_eq!(cursor.name, format!("StateNode_{expected_level}"));
            if expected_level < EXPECTED_MAX_DEPTH {
                cursor = cursor
                    .sub_states
                    .first()
                    .ok_or_else(|| format!("missing sub_state below StateNode_{expected_level}"))?;
            }
        }
        assert!(
            cursor.sub_states.is_empty(),
            "Sub-state at depth MAX_AST_DEPTH + 1 must be truncated cleanly without panicking"
        );
        Ok(())
    }
}

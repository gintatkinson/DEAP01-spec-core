//! Unit tests for SysML v2 AST schema extraction and state machine models. [DOC-TIER: A]
//! Safety & Traceability: Realises: [REQ-0101], [REQ-0801]
//! Contract: Deterministic SysML v2 schema extraction and state def unit tests.
//! Bounds & Faults: Bounded O(N) test execution; fails closed on assertion or parse mismatches.

#[cfg(test)]
mod tests {
    use super::super::*;

#[test]
fn test_package_def_queries_and_nested_ports() -> Result<(), Box<dyn std::error::Error>> {
    let sub_part = PartDef { name: "NestedSensor".to_string(), ports: vec![PortDef { name: "sync_pulse_out".to_string(), ..Default::default() }], ..Default::default() };
    let top_part = PartDef {
        name: "ControlProcessor".to_string(),
        ports: vec![PortDef { name: "ctrl_bus".to_string(), ..Default::default() }],
        parts: vec![sub_part],
        calcs: vec![CalcDef { name: "InnerCalc".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let root = PackageDef {
        name: "RootSystem".to_string(),
        calc_defs: vec![CalcDef { name: "RootCalc".to_string(), ..Default::default() }],
        port_defs: vec![PortDef { name: "root_port".to_string(), ..Default::default() }],
        hazard_defs: vec![HazardDef { name: "H-01".to_string(), severity: 3, ..Default::default() }],
        risk_defs: vec![RiskDef { name: "R-01".to_string(), severity: 3, ..Default::default() }],
        state_defs: vec![StateDef { name: "Nominal".to_string(), ..Default::default() }],
        connection_defs: vec![ConnectionDef { name: "C-01".to_string(), source_port: "p1".to_string(), target_port: "p2".to_string(), ..Default::default() }],
        part_defs: vec![top_part],
        ..Default::default()
    };
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
    assert_eq!(via_enum, expected);
    assert_eq!(via_catalog, expected);
    for x in SupportedSchemaExtension::ALL {
        assert_eq!(x.as_str_ref(), x.as_str());
        assert!(catalog.contains(&x.as_str()));
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
    let expected = vec!["REQ-CTRL-001", "REQ-CTRL-002", "REQ-PWR-03", "REQ-COM-04", "REQ-COM-05", "REQ-COM-06", "REQ-COM-07", "REQ-COM-08"].into_iter().map(String::from).collect::<Vec<_>>();
    assert_eq!(parse_satisfied_requirements(snippet), expected);
    let parts = parse_part_defs(snippet);
    assert_eq!(parts.first().map(|p| (p.name.as_str(), &p.satisfied_requirements)), Some(("SensorController", &expected)));
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
    let part = model.parts.first().ok_or("Expected FlightActuator")?;
    let exp_h = ["H-01".to_string(), "H-02".to_string()];
    let exp_s = ["SC-01".to_string(), "SC-02".to_string()];
    assert_eq!((part.triggers_hazards.as_slice(), part.safety_realises.as_slice(), part.layer.as_deref()), (exp_h.as_slice(), exp_s.as_slice(), Some("Domain")));
    let req = model.requirements.first().ok_or("Expected SC-01")?;
    let exp_rh = ["H-01".to_string()];
    let exp_rs = ["SYS-REQ-01".to_string()];
    assert_eq!((req.triggers_hazards.as_slice(), req.safety_realises.as_slice(), req.layer.as_deref()), (exp_rh.as_slice(), exp_rs.as_slice(), Some("Safety")));
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
    let part = model.parts.first().ok_or("Expected AirDataSensor")?;
    let td = TypeDescriptor::from_part_def("AvionicsPkg", part);
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
    audit_log.append_record(CotAuditRecord {
        step_id: "STEP-01".to_string(),
        model_id: "deepseek-r1-local".to_string(),
        prompt_sha256: IncrementalAstCache::compute_digest("prompt-payload"),
        reasoning_trace: "Verified PartDef PersistentNode invariants.".to_string(),
        verified_ast_refs: vec!["PersistedPkg::PersistentNode".to_string()],
    });
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
    assert!(sysml_parts.contains("FlightDirector") && sysml_parts.contains("actuator_unit") && !sysml_parts.contains("Boundaries") && !sysml_parts.contains("TelemetryBus"));
    let digest_json = r#"{"format_version":"1.0","source_hash":"deadbeef","parts":[{"name":"DigestGhostPart"}]}"#;
    assert!(UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", digest_json).is_empty());
}

#[test]
fn test_issue_553_extract_subsystem_parts_ignores_unrelated_markdown_tables() {
    let md_src = "| ID | Subsystem | Owner |\n| --- | --- | --- |\n| 1 | NavigationUnit | Avionics |\n\n| Acronym | Meaning |\n| --- | --- |\n| GNSSReceiverTerm | Global |\n";
    let md_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", md_src);
    assert!(md_parts.contains("NavigationUnit") && !md_parts.contains("GNSSReceiverTerm"));
}

#[test]
fn test_parse_state_defs_roundtrips_sub_states_and_transitions() -> Result<(), Box<dyn std::error::Error>> {
    let src = "state def BP_01_LifecycleState {\n    state Uninitialized;\n    state Active;\n    transition Initial_to_Uninitialized;\n    transition Uninitialized_to_Active_on_parse_token;\n    transition Active_to_Terminal;\n}";
    let states = parse_state_defs(src);
    let st = states.first().ok_or("missing top-level StateDef")?;
    assert_eq!((st.name.as_str(), st.sub_states.len(), st.transitions.len()), ("BP_01_LifecycleState", 2, 3));
    let tr1 = st.transitions.get(1).ok_or("missing transition 1")?;
    assert_eq!((tr1.source.as_deref(), tr1.target.as_deref(), tr1.trigger.as_deref()), (Some("Uninitialized"), Some("Active"), Some("parse_token")));
    Ok(())
}

#[test]
fn test_issue_494_compute_sha256_hex_delegates_to_canonical_digest() {
    for sample in ["", "abc", "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"] {
        assert_eq!(crate::sysml_ast::cache::compute_sha256_hex(sample.as_bytes()), crate::digest::compute_sha256(sample.as_bytes()));
        assert_eq!(IncrementalAstCache::compute_digest(sample), crate::digest::compute_sha256(sample.as_bytes()));
    }
}

#[test]
fn test_issue_503_discover_schema_files_preserves_depth_zero_dot_root() -> Result<(), String> {
    let tmp_dir = std::env::temp_dir().join(format!(".deap_core_issue_503_dot_root_{}", uuid::Uuid::new_v4()));
    let hidden_subdir = tmp_dir.join(".hidden_dir");
    std::fs::create_dir_all(&hidden_subdir).map_err(|e| e.to_string())?;
    std::fs::write(tmp_dir.join("visible.sysml"), "package VisiblePkg { part def NodeA; }").map_err(|e| e.to_string())?;
    std::fs::write(tmp_dir.join(".hidden.sysml"), "package HiddenPkg { part def NodeB; }").map_err(|e| e.to_string())?;
    std::fs::write(hidden_subdir.join("nested.sysml"), "package HiddenNestedPkg { part def NodeC; }").map_err(|e| e.to_string())?;
    let discovered = UnifiedSchemaModelExtractor::discover_schema_files(std::slice::from_ref(&tmp_dir));
    std::fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let names: Vec<String> = discovered.iter().filter_map(|p| p.file_name()?.to_str().map(String::from)).collect();
    assert_eq!(names, vec!["visible.sysml".to_string()]);
    Ok(())
}

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
            cursor = cursor.sub_states.first().ok_or("missing sub_state")?;
        }
    }
    assert!(cursor.sub_states.is_empty());
    Ok(())
}
}


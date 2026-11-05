//! Integration and regression tests for `MarkdownTranslator` and its CST/AST extraction helpers.
//! Realises: [REQ-0021] [REQ-0032] [REQ-SYSML-INGEST-MD]

use deap_core::sysml_ast::PackageDef;
use ingest_sysml::translators::markdown::{
    clean_bdd_text, clean_normative_text, extract_acceptance_criteria,
    extract_behavioral_ast_nodes, extract_formal_invariants, extract_normative_statement,
    extract_req_metadata, make_port, Frontmatter, MarkdownTranslator,
};
use std::fs;
use std::path::Path;

#[test]
fn test_parse_frontmatter_and_make_port() {
    let content = "---\nid: REQ-0001\ntitle: \"Abstract MBSE Compiler Mandate & Pure Schema-Driven Execution\"\nsubsystem: \"Subsystem 1: System Vision, Bootstrapping & Foundational Invariants\"\nuuidv5: 5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\n---\n\n# Heading 1\nProse\n";
    let (fm, rest) = MarkdownTranslator::parse_frontmatter(content);
    assert_eq!(fm.id.as_deref(), Some("REQ-0001"));
    assert_eq!(
        fm.title.as_deref(),
        Some("Abstract MBSE Compiler Mandate & Pure Schema-Driven Execution")
    );
    assert_eq!(
        fm.uuidv5.as_deref(),
        Some("5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9")
    );
    assert!(rest.starts_with("# Heading 1"));

    let p = make_port("rules_out", "CompilerRulePort", "out");
    assert_eq!((p.name.as_str(), p.type_name.as_str(), p.direction.as_str(), p.port_category.as_str()), ("rules_out", "CompilerRulePort", "out", ""));
}

#[test]
fn test_translate_req_0001_and_0032() -> Result<(), Box<dyn std::error::Error>> {
    let translator = MarkdownTranslator::new();
    let req1_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema/REQ-0001.md");
    if Path::new(req1_path).exists() {
        let content = fs::read_to_string(req1_path)?;
        let pkg = translator.translate(&content, "REQ-0001")?;
        assert_eq!(pkg.name, "Abstract_MBSE_Compiler_Mandate_Pure_Schema_Driven_Execution");
        let req = pkg.requirement_defs.first().ok_or("missing req")?;
        assert_eq!((req.name.as_str(), req.req_id.as_str()), ("REQ_0001_Abstract_MBSE_Compiler_Mandate", "REQ-0001"));
        assert!(req.text.contains("abstract Model-Based Systems Engineering"));
        let doc = req.doc.as_ref().ok_or("missing doc")?;
        assert!(doc.contains("UUIDv5: 5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9") && doc.contains("Diagnostics:"));
        let req_attr_names: Vec<&str> = req.attributes.iter().map(|a| a.name.as_str()).collect();
        for expected in ["uuidv5", "complexity_class", "governing_standard", "diagnostic_codes", "ac_01_pure_schema_driven_symbol_derivation"] {
            assert!(req_attr_names.contains(&expected), "missing {expected}");
        }
    }
    let req32_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema/REQ-0032.md");
    if Path::new(req32_path).exists() {
        let content = fs::read_to_string(req32_path)?;
        let pkg = translator.translate(&content, "REQ_0032")?;
        let req = pkg.requirement_defs.first().ok_or("missing req")?;
        assert!(req.requires.iter().any(|r| r.contains("Strict_Total_Ordering")));
        assert!(req.derived_from.iter().any(|d| d.contains("0031")));
        assert!(req.constraints.iter().any(|c| c.name.contains("Strict_Total_Ordering")));
    }
    Ok(())
}

#[test]
fn test_translate_bom_and_ports() -> Result<(), String> {
    let content = r#"
# Package_0

## Bill of Materials
| Component | Part Number | Mass (kg) | Power (W) | Description |
| :--- | :--- | :--- | :--- | :--- |
| Classifier_Alpha | PN_01 | 12.5 | 500.0 | Synthetic component alpha |
| Classifier_Beta | PN_02 | 2.1 | 15.0 | Synthetic component beta |

## System Interfaces
| Port | Direction | Type | Rate (Hz) | Description |
| :--- | :--- | :--- | :--- | :--- |
| Port_1 | out | Real | 100 | Port 1 signal |
| Port_2 | in | String | 10 | Port 2 command |

## System Parameters
| Parameter | Min | Max | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| param_x | 20.0 | 28.0 | V | Synthetic parameter x |
"#;
    let pkg = MarkdownTranslator::new().translate(content, "Package_0")?;
    assert_eq!(pkg.name, "Package_0");
    let alpha = pkg.part_defs.iter().find(|p| p.name == "Classifier_Alpha").ok_or("missing Classifier_Alpha")?;
    assert_eq!(alpha.doc.as_deref(), Some("Synthetic component alpha"));
    let alpha_attrs: Vec<&str> = alpha.attributes.iter().map(|a| a.name.as_str()).collect();
    assert!(alpha_attrs.contains(&"part_number") && alpha_attrs.contains(&"mass") && alpha_attrs.contains(&"power"));
    let port_names: Vec<&str> = pkg.port_defs.iter().map(|p| p.name.as_str()).collect();
    assert!(port_names.contains(&"Port_1") && port_names.contains(&"Port_2"));
    assert!(pkg.constraint_defs.iter().any(|c| c.name == "assert_param_x_range"));
    Ok(())
}

#[test]
fn test_issue_442_and_defect_02_translate_files() -> Result<(), Box<dyn std::error::Error>> {
    let translator = MarkdownTranslator::new();
    assert!(translator.translate_files(&[], "Test_System").is_err());
    let missing_path = std::env::temp_dir().join("nonexistent_defect_02_blueprint_491.md").to_string_lossy().to_string();
    assert!(translator.translate_files(std::slice::from_ref(&missing_path), "Test_System").is_err());

    let tmp_dir = std::env::temp_dir().join(format!("test_issue_442_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)?;
    let f1 = tmp_dir.join("REQ-1001.md");
    let f2 = tmp_dir.join("REQ-1002.md");
    fs::write(&f1, "---\nid: REQ-1001\ntitle: \"Attitude Determination Precision\"\nsubsystem: \"Subsystem 1: Attitude Determination & Control\"\nuuidv5: 00000000-0000-0000-0000-000000000001\n---\n# REQ-1001\nThe Attitude Determination system shall maintain pointing knowledge.\n")?;
    fs::write(&f2, "---\nid: REQ-1002\ntitle: \"Battery Bus Voltage Range\"\nsubsystem: \"Subsystem 2: Electrical Power Management\"\nuuidv5: 00000000-0000-0000-0000-000000000002\n---\n# REQ-1002\nThe Electrical Power subsystem shall maintain bus voltage.\n")?;

    let paths = vec![f1.to_string_lossy().to_string(), f2.to_string_lossy().to_string()];
    let res = translator.translate_files(&paths, "Satellite_System")?;
    let first_path = paths.first().ok_or("missing first path")?.clone();
    assert!(translator.translate_files(&[first_path, missing_path], "Test_System").is_err());
    let _ = fs::remove_dir_all(&tmp_dir);

    assert_eq!(res.name, "Satellite_System");
    assert_eq!(res.packages.len(), 2);
    assert_eq!(res.connection_defs.len(), 0);
    Ok(())
}

#[test]
fn test_extract_behavioral_ast_nodes_populates_sub_states_and_fallback_state_def() -> Result<(), String> {
    let md_with_state = "```mermaid\nsequenceDiagram\n  participant A as Alpha\n  participant B as Beta\n  A->>B: US_AC_01_DispatchCommand(payload)\n```\n```mermaid\nstateDiagram-v2\n  [*] --> Idle\n  Idle --> Active : DispatchCommand\n  Active --> [*]\n```\n";
    let mut pkg1 = PackageDef { name: "PkgOne".to_string(), ..Default::default() };
    extract_behavioral_ast_nodes(md_with_state, &mut pkg1);
    assert_eq!(pkg1.action_defs.first().ok_or("missing action")?.name, "DispatchCommand");
    let state1 = pkg1.state_defs.first().ok_or("missing state1")?;
    let sub_names: Vec<&str> = state1.sub_states.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(sub_names, vec!["Idle", "Active"]);
    let tr_names: Vec<&str> = state1.transitions.iter().filter_map(|t| t.name.as_deref()).collect();
    assert_eq!(tr_names, vec!["Initial_to_Idle", "Idle_to_Active_on_DispatchCommand", "Active_to_Terminal"]);

    let md_without_state = "```mermaid\nsequenceDiagram\n  participant C as Controller\n  participant D as Driver\n  C->>D: US_02_StartEngine(cmd)\n  D-->>C: US_AC_03_ConfirmReady(ack)\n```\n";
    let mut pkg2 = PackageDef { name: "PkgTwo".to_string(), ..Default::default() };
    extract_behavioral_ast_nodes(md_without_state, &mut pkg2);
    let state2 = pkg2.state_defs.first().ok_or("missing state2")?;
    assert_eq!(state2.sub_states.len(), 3);
    assert_eq!(state2.transitions.len(), 4);
    assert_eq!(state2.transitions.first().and_then(|t| t.source.as_deref()), Some("Initial"));
    assert_eq!(state2.transitions.get(1).and_then(|t| t.trigger.as_deref()), Some("StartEngine"));
    assert_eq!(state2.transitions.get(2).and_then(|t| t.trigger.as_deref()), Some("StartEngine"));
    assert_eq!(state2.transitions.get(3).and_then(|t| t.target.as_deref()), Some("Terminal"));
    Ok(())
}

#[test]
fn test_issue_475_blueprint_heading_row_attr_constraint_latency_and_uc_extraction() -> Result<(), String> {
    let blueprint_md = r#"# Dual-Use Drone Orchestration Blueprint

## 2.1 Operational Context
High-density tactical and civil airspace coordination.

## 3.1 Structural Components
| Component | Attribute | Type | Default / Bound | Description |
| :--- | :--- | :--- | :--- | :--- |
| `StrategicAirspaceCoordinatorNode` | `max_active_corridors` | `Integer` | `64` | Maximum concurrent corridors |
| `StrategicAirspaceCoordinatorNode` | `solver_horizon_s` | `Real` | `120.0` | Trajectory lookahead horizon |
| `TacticalEdgeCoordinatorNode` | `control_loop_hz` | `Real` | `200.0` | Edge control frequency |

## 4.1 Logical Interfaces
| Port | Direction | Type | Protocol | Source | Target | Latency Budget | Description |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `policy_out` | **out** | `CorridorPolicy` | `DDS-RTPS` | `StrategicAirspaceCoordinatorNode.policy_out` | `TacticalEdgeCoordinatorNode.policy_in` | `<= 500 us` | Policy broadcast |
| `telemetry_in` | `in` | `EdgeTelemetry` | `MAVLink2` | `TacticalEdgeCoordinatorNode.telemetry_out` | `StrategicAirspaceCoordinatorNode.telemetry_in` | `0.025 s` | Telemetry uplink |

## 5.2 Parametric Constraints
| Constraint Name | Parameter | Min | Max | Unit | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `SubsecondEdgeSolverLatencyBound` | `solver_latency_ms` | `0.1` | `10.0` | `ms` | Solver latency bound |
|  | `corridor_margin_m` | `5.0` | `50.0` | `m` | Corridor safety margin |

### 5.3 Acceptance Criteria
All safety invariants verified.

## 6. Operational Use Cases
| Use Case ID | Title | Primary Actor | Objective | Preconditions | Postconditions | Included Use Cases | Verification Test Cases |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `UC-01` | Dynamic Airspace Deconfliction | `AirspaceOperator` | Deconflict trajectories | Corridor active | Trajectories separated | `UC-02, UC-03` | `TC-01, TC-02` |

### 6.1 UC-M2M: Autonomous Fleet Re-routing
- **Primary Actor:** `StrategicAirspaceCoordinatorNode`
- **Secondary Actor:** `TacticalEdgeCoordinatorNode`
- **Objective:** Re-route fleet around dynamic hazard volume.
- **Preconditions:** Active mission plan loaded.
- **Main Success Scenario:**
  1. Detect hazard incursion in sector.
  2. Recompute conflict-free 4D corridors.
  3. Dispatch updated waypoints to edge nodes.
- **Postconditions:** Fleet maintains safe separation.
- **Included Use Cases:** `UC-01`
- **Verification Test Cases:** `TC-M2M-01`
"#;

    let pkg = MarkdownTranslator::new().translate(blueprint_md, "DroneBlueprint")?;
    let part_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
    for forbidden in ["Operational_Context", "Acceptance_Criteria", "Operational_Use_Cases", "UC_M2M"] {
        assert!(!part_names.iter().any(|n| n.contains(forbidden)));
    }
    let strat = pkg.part_defs.iter().find(|p| p.name == "StrategicAirspaceCoordinatorNode").ok_or("missing strat")?;
    let strat_attrs: Vec<&str> = strat.attributes.iter().map(|a| a.name.as_str()).collect();
    assert!(strat_attrs.contains(&"max_active_corridors") && strat_attrs.contains(&"solver_horizon_s"));
    assert!(!strat_attrs.contains(&"attribute") && !strat_attrs.contains(&"default_bound"));

    let con_names: Vec<&str> = pkg.constraint_defs.iter().map(|c| c.name.as_str()).collect();
    assert!(con_names.contains(&"SubsecondEdgeSolverLatencyBound") && con_names.contains(&"assert_corridor_margin_m_range"));

    let p_out = pkg.port_defs.iter().find(|p| p.name == "policy_out").ok_or("missing policy_out")?;
    assert_eq!((p_out.direction.as_str(), p_out.protocol_family.as_str()), ("out", "DDS-RTPS"));
    let conn_policy = pkg.connection_defs.iter().find(|c| c.name == "conn_policy_out").ok_or("missing conn_policy_out")?;
    assert_eq!(conn_policy.source_port, "StrategicAirspaceCoordinatorNode.policy_out");
    assert_eq!(conn_policy.target_port, "TacticalEdgeCoordinatorNode.policy_in");
    assert_eq!(conn_policy.latency_ms, Some(0.5));
    let conn_telem = pkg.connection_defs.iter().find(|c| c.name == "conn_telemetry_in").ok_or("missing conn_telemetry_in")?;
    assert_eq!(conn_telem.latency_ms, Some(25.0));

    let uc01 = pkg.use_case_defs.iter().find(|u| u.name.starts_with("UC_01")).ok_or("missing UC_01")?;
    assert_eq!(uc01.actor.as_deref(), Some("AirspaceOperator"));
    assert!(uc01.includes.iter().any(|inc| inc.contains("UC_02")));
    let uc_m2m = pkg.use_case_defs.iter().find(|u| u.name.starts_with("UC_M2M")).ok_or("missing UC_M2M")?;
    assert_eq!(uc_m2m.actor.as_deref(), Some("StrategicAirspaceCoordinatorNode"));
    assert_eq!(uc_m2m.steps.len(), 3);
    let tc_names: Vec<&str> = pkg.test_case_defs.iter().map(|t| t.name.as_str()).collect();
    assert!(tc_names.iter().any(|t| t.contains("TC_01")) && tc_names.iter().any(|t| t.contains("TC_M2M_01")));
    Ok(())
}

#[test]
fn test_issue_524_doc_sanitization_property_table_columns_and_fenced_code_scanners() -> Result<(), String> {
    let norm_cleaned = clean_normative_text("The system /* block */ shall sanitize */ tokens.");
    assert!(!norm_cleaned.contains("/*") && !norm_cleaned.contains("*/"));
    let bdd_cleaned = clean_bdd_text("Given /* state */ when trigger */ then active");
    assert!(!bdd_cleaned.contains("/*") && !bdd_cleaned.contains("*/"));

    let table_md = r#"# SyntheticSystem

## Component: ModuleAlpha
Section prose with /* comment delimiter */ inside doc.

| Property | Type | Default / Bound | Description |
| :--- | :--- | :--- | :--- |
| `prop_count` | `Integer` | `42` | Count /* doc */ |
| `prop_ratio` | `float32` | `3.5` | Ratio value |
| `prop_label` | `string` | `alpha_label` | Label text |
| `prop_mode` | `CustomModeType` | `MODE_A` | Custom typed property |
| `prop_unbounded` | `Integer` | `>= 10` | Non-literal bound |

## Bill of Materials
| Component | Attribute | Type | Default / Bound | Description |
| :--- | :--- | :--- | :--- | :--- |
| `ModuleBeta` | `max_items` | `Integer` | `<= 100` | Bounded /* item */ |
| `ModuleBeta` | `gain_factor` | `Real` | `TBD` | Unresolved real default |
| `ModuleBeta` | `flag_enabled` | `Boolean` | `N/A` | Unresolved boolean default |

## System Parameters
| Constraint Name | Parameter | Type | Default | Min | Max | Unit | Description |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `RateBound` | `cycle_rate` | `Integer` | `TBD` | `1` | `10` | `Hz` | Cycle /* constraint */ |
"#;
    let pkg = MarkdownTranslator::new().translate(table_md, "SyntheticSystem")?;
    let mod_alpha = pkg.part_defs.iter().find(|p| p.name == "ModuleAlpha").ok_or("missing ModuleAlpha")?;
    assert!(!mod_alpha.doc.as_deref().unwrap_or("").contains("/*"));

    fn find_attr<'a>(
        p: &'a deap_core::sysml_ast::PartDef,
        n: &str,
    ) -> Result<&'a deap_core::sysml_ast::AttributeDef, String> {
        p.attributes.iter().find(|a| a.name == n).ok_or_else(|| format!("missing {n}"))
    }
    let prop_count = find_attr(mod_alpha, "prop_count")?;
    assert_eq!((prop_count.type_name.as_str(), prop_count.default_value.as_deref()), ("Integer", Some("42")));
    let prop_ratio = find_attr(mod_alpha, "prop_ratio")?;
    assert_eq!((prop_ratio.type_name.as_str(), prop_ratio.default_value.as_deref()), ("Real", Some("3.5")));
    let prop_label = find_attr(mod_alpha, "prop_label")?;
    assert_eq!((prop_label.type_name.as_str(), prop_label.default_value.as_deref()), ("String", Some("\"alpha_label\"")));
    let prop_mode = find_attr(mod_alpha, "prop_mode")?;
    assert_eq!((prop_mode.type_name.as_str(), prop_mode.default_value.as_deref()), ("CustomModeType", Some("\"MODE_A\"")));
    let prop_unbounded = find_attr(mod_alpha, "prop_unbounded")?;
    assert_eq!((prop_unbounded.type_name.as_str(), prop_unbounded.default_value.as_deref()), ("Integer", None));

    let mod_beta = pkg.part_defs.iter().find(|p| p.name == "ModuleBeta").ok_or("missing ModuleBeta")?;
    assert_eq!(find_attr(mod_beta, "max_items")?.default_value, None);
    assert_eq!(find_attr(mod_beta, "gain_factor")?.default_value, None);
    assert_eq!(find_attr(mod_beta, "flag_enabled")?.default_value, None);

    let cycle_attr = pkg.attribute_defs.iter().find(|a| a.name == "cycle_rate").ok_or("missing cycle_rate")?;
    assert_eq!((cycle_attr.type_name.as_str(), cycle_attr.default_value.as_deref()), ("Integer", None));

    let fenced_req_md = "---\nid: REQ-0524\ntitle: \"Fence Aware Scanner Verification\"\nsubsystem: \"Subsystem Alpha\"\nuuidv5: 00000000-0000-0000-0000-000000000524\n---\n\n# REQ-0524\n\n```markdown\n| Complexity Class | Fake Complexity |\n| Governing Standard | Fake Standard |\n```\n\n| Field | Value |\n| :--- | :--- |\n| Complexity Class | Class NP-Hard |\n| Governing Standard | ISO-15288 |\n\n## 1. Normative Statement\n```text\n## Fake Heading Inside Fence\n| fake | table |\n```\nThe system shall ignore fenced blocks when scanning normative statements.\n\n## 2. Formal Invariant\n```rust\n### Fake Invariant Inside Code:\n## 3. Fake Section Boundary Inside Code\n```\n\n### Real Determinism Invariant:\n```math\nx \\ge 0\n```\n\n## 3. Acceptance Criteria\n### AC-01: Fenced Criteria Handling\n```yaml\n# comment inside fence\nGiven: fake fenced given\n```\n- Given: valid initial state\n- When: valid transition occurs\n- Then: valid output state is observed\n";
    let fm = Frontmatter {
        id: Some("REQ-0524".to_string()),
        title: Some("Fence Aware Scanner Verification".to_string()),
        subsystem: Some("Subsystem Alpha".to_string()),
        uuidv5: Some("00000000-0000-0000-0000-000000000524".to_string()),
    };
    let (_, complexity, standard, _) = extract_req_metadata(fenced_req_md, &fm);
    assert_eq!((complexity.as_str(), standard.as_str()), ("Class NP-Hard", "ISO-15288"));
    assert_eq!(extract_normative_statement(fenced_req_md), "The system shall ignore fenced blocks when scanning normative statements.");
    let acs = extract_acceptance_criteria(fenced_req_md);
    assert!(acs.first().ok_or("missing AC-01")?.2.contains("Given: valid initial state"));
    let invs = extract_formal_invariants(fenced_req_md, "0524");
    assert_eq!(invs.first().ok_or("missing inv")?.name, "Invariant_REQ_0524_Real_Determinism_Invariant");
    Ok(())
}

#[test]
fn test_issue_863_declared_diagnostic_namespace_fidelity() -> Result<(), String> {
    let content = r#"| Field | Value |
| :--- | :--- |
| Complexity Class | Class P |
| Governing Standard | ISO/IEC/IEEE 29148:2018 |
| Diagnostic Codes | DEAP-E0101, E0202 |

## 1. Normative Statement
The translator shall retain every declared diagnostic code in source order. Invalid tokens XDEAP-E9999 and DEAP-E99999 must not be harvested.

## 3. Acceptance Criteria
### AC-01: Diagnostic Code Retention
- Given: the blueprint declares DEAP-E12xx
- When: requirement metadata is extracted
- Then: the declared namespace is retained verbatim
- Diagnostic: DEAP-E12xx
"#;
    let fm = Frontmatter { id: Some("REQ-0863".to_string()), title: Some("Diagnostic Namespace Fidelity".to_string()), subsystem: Some("Subsystem 863".to_string()), uuidv5: Some("00000000-0000-0000-0000-000000000863".to_string()) };
    let (_, _, _, diagnostics) = extract_req_metadata(content, &fm);
    assert_eq!(diagnostics, "DEAP-E0101, E0202, DEAP-E12xx");
    assert!(!diagnostics.contains("E0100") && !diagnostics.contains("E9999"));
    let (_, _, _, empty_diagnostics) = extract_req_metadata("| Field | Value |\n| :--- | :--- |\n| Complexity Class | Class P |\n", &Frontmatter::default());
    assert_eq!(empty_diagnostics, "");
    Ok(())
}

#[test]
fn test_issue_863_full_normative_statement_not_truncated() -> Result<(), String> {
    let clause = format!("The system shall preserve {} terminal fidelity marker FULL_CLAUSE_END_863.", "complete requirement prose ".repeat(14));
    let content = format!("## 1. Normative Statement\n{clause}\n\n## 2. Formal Invariant\n");
    let text = extract_normative_statement(&content);
    assert!(text.len() > 300, "normative statement was truncated to {} bytes", text.len());
    assert!(text.ends_with("FULL_CLAUSE_END_863."), "missing terminal marker: {text}");
    Ok(())
}

#[test]
fn test_issue_863_literal_escape_sequences_preserved_in_acceptance_criteria() -> Result<(), String> {
    let content = r#"## 3. Acceptance Criteria
### AC-01: Escape Preservation
- Given: a clause contains literal \r and another \r escape
- When: the criteria are extracted
- Then: both \r sequences remain two-character escapes and no CR control character appears
- Diagnostic: DEAP-E12xx
"#;
    let bdd = extract_acceptance_criteria(content).first().ok_or("missing AC-01")?.2.clone();
    assert!(bdd.matches("\\r").count() >= 2, "expected two literal backslash-r sequences: {bdd}");
    assert!(!bdd.contains('\r'), "carriage-return control character leaked into AC prose");
    let cleaned = clean_normative_text("The clause contains \\r and \\n and \\t literals.");
    assert!(cleaned.contains("\\r") && cleaned.contains("\\n") && cleaned.contains("\\t"), "literal escapes lost: {cleaned}");
    assert!(!cleaned.contains('\r') && !cleaned.contains('\n') && !cleaned.contains('\t'));
    Ok(())
}

#[test]
fn test_issue_863_duplicate_sequence_messages_retained_in_source_order() -> Result<(), String> {
    let md = "```mermaid\nsequenceDiagram\n  participant Verifier\n  participant Compiler\n  Verifier->>Compiler: verify_lossless_cst(stage1_input)\n  Compiler-->>Verifier: lossless_cst_verified(stage1)\n  Verifier->>Compiler: verify_lossless_cst(stage2_input)\n  Compiler-->>Verifier: lossless_cst_verified(stage2)\n```\n";
    let mut pkg = PackageDef { name: "Pkg863".to_string(), ..Default::default() };
    extract_behavioral_ast_nodes(md, &mut pkg);
    let interaction = pkg.interaction_defs.first().ok_or("missing interaction")?;
    assert_eq!(interaction.messages.len(), 4, "duplicate messages were deduplicated: {:?}", interaction.messages);
    let expected = ["verify_lossless_cst from Verifier to Compiler", "reply_lossless_cst_verified from Compiler to Verifier", "verify_lossless_cst from Verifier to Compiler", "reply_lossless_cst_verified from Compiler to Verifier"];
    for (idx, exp) in expected.iter().enumerate() {
        assert_eq!(interaction.messages.get(idx).map(String::as_str), Some(*exp), "message {idx} out of order");
    }
    Ok(())
}

//! Unit test suite A for `compile-sysml` crate root (`lib.rs`) MBD synthesis and rustdoc contracts.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D rustdoc 7-section compliance,
//! custom stereotype preservation, and MBD `.slreqx`/`.sfx`/`.m`/`.json` synthesis and XML escaping.
//!
//! ## 2. Invariants & Mathematical Properties
//! - `lib.rs`, `cli.rs`, `driver.rs`, and `main.rs` preserve 7-section module and item documentation.
//! - `@TriggersHazard`, `@SafetyRealises`, and `@Layer` stereotypes round-trip into `PackageDef` and `MbdExportBundle`.
//! - All 5 reserved XML characters (`&`, `<`, `>`, `"`, `'`) are deterministically escaped in `.slreqx` and `.sfx`.
//!
//! ## 3. Preconditions
//! - Invoked via `cargo test` with writable OS temporary directory access.
//!
//! ## 4. Postconditions
//! - Temporary MBD export directories are removed prior to test completion.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over bounded in-memory SysML test schemas.
//! - Space Complexity: $O(N)$ for parsed ASTs and synthesized MBD strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates parse or I/O failures explicitly via `Result`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-LIB], [REQ-COMPILER-SYSML-TEST-DEFECT-06-MBD-XML-ESCAPING]

use super::*;

#[test]
fn test_compile_sysml_lib_7_section_rustdoc() {
    let src = include_str!("lib.rs");
    let module_doc: String = src
        .lines()
        .take_while(|line| line.starts_with("//!"))
        .collect::<Vec<&str>>()
        .join("\n");

    for heading in [
        "//! ## 1. Safety Intent & Regulatory Scope",
        "//! ## 2. Invariants & Mathematical Properties",
        "//! ## 3. Preconditions",
        "//! ## 4. Postconditions",
        "//! ## 5. Algorithmic Complexity Bounds",
        "//! ## 6. Failure Modes & Fault Tolerance",
        "//! ## 7. Realization & Traceability",
        "//! Realises: [REQ-COMPILER-SYSML-LIB], [DO-330-TQL-4-LIB]",
    ] {
        assert!(module_doc.contains(heading), "missing module doc heading: {heading}");
    }

    for item_marker in [
        "/// ## 1. Safety Intent & Regulatory Scope",
        "/// ## 2. Invariants & Mathematical Properties",
        "/// ## 3. Preconditions",
        "/// ## 4. Postconditions",
        "/// ## 5. Algorithmic Complexity Bounds",
        "/// ## 6. Failure Modes & Fault Tolerance",
        "/// ## 7. Realization & Traceability",
        "/// Realises: [REQ-COMPILER-SYSML-PARSE]",
        "/// Realises: [REQ-COMPILER-SYSML-ESCAPE-XML]",
    ] {
        assert!(src.contains(item_marker), "missing item marker: {item_marker}");
    }

    for (name, file_src) in [
        ("src/cli.rs", include_str!("cli.rs")),
        ("src/driver.rs", include_str!("driver.rs")),
        ("src/main.rs", include_str!("main.rs")),
    ] {
        assert!(file_src.contains("//! ## 1. Safety Intent & Regulatory Scope"), "{name} sec 1");
        assert!(file_src.contains("//! ## 7. Realization & Traceability"), "{name} sec 7");
        assert!(file_src.contains("/// ## 1. Safety Intent & Regulatory Scope"), "{name} item 1");
        assert!(file_src.contains("/// ## 7. Realization & Traceability"), "{name} item 7");
    }

    assert!(parse_sysml("package Demo {}").is_ok());
}

#[test]
fn test_compile_sysml_preserves_custom_safety_and_layer_stereotypes() -> Result<(), String> {
    let src = r#"
        package SafetyPkg {
            part def FlightController {
                @TriggersHazard("H-01")
                @TriggersHazard { hazard = "H-02"; }
                @SafetyRealises("SC-01")
                @Layer("Domain")
                attribute pitch_deg : Real;
            }
            requirement def PitchBoundReq {
                id = "REQ-SAFE-01";
                text = "Pitch angle shall remain within bounds.";
                @TriggersHazard("H-01")
                @SafetyRealises { requirement = "SC-01"; }
                @Layer("Safety")
            }
        }
    "#;
    let pkg = parse_sysml(src).map_err(|e| format!("{e:?}"))?;
    let part = pkg.part_defs.first().ok_or("FlightController should exist")?;
    assert_eq!(part.triggers_hazards, vec!["H-01".to_string(), "H-02".to_string()]);
    assert_eq!(part.safety_realises, vec!["SC-01".to_string()]);
    assert_eq!(part.layer.as_deref(), Some("Domain"));

    let req = pkg.requirement_defs.first().ok_or("PitchBoundReq should exist")?;
    assert_eq!(req.triggers_hazards, vec!["H-01".to_string()]);
    assert_eq!(req.safety_realises, vec!["SC-01".to_string()]);
    assert_eq!(req.layer.as_deref(), Some("Safety"));
    Ok(())
}

#[test]
fn test_synthesize_mbd_bundle_and_export_artifacts() -> Result<(), String> {
    let src = r#"
        package AvionicsPkg {
            port def TelemetryPort {
                attribute altitude_m : Real;
            }
            part def FlightController {
                @TriggersHazard("H-01")
                @SafetyRealises("SC-01")
                @Layer("Domain")
                attribute pitch_deg : Real;
                port out telemetry : TelemetryPort;
                satisfy REQ-FC-01;
                state def FlightMode {
                    state Init;
                    state Active;
                    transition first Init then Active;
                }
            }
            requirement def AttitudeReq {
                id = "REQ-FC-01";
                text = "Controller shall stabilize pitch.";
                @TriggersHazard("H-01")
                @SafetyRealises("SC-01")
                @Layer("Domain")
            }
        }
    "#;
    let pkg = parse_sysml(src).map_err(|e| format!("{e:?}"))?;
    let model = SysmlModel::from(pkg);
    let bundle = synthesize_mbd_bundle(&model);
    for needle in ["<slreq:Requirement", "REQ-FC-01", "SC-01", "H-01"] {
        assert!(bundle.slreq_xml.contains(needle));
    }
    for needle in ["FlightMode", "Init", "Active"] {
        assert!(bundle.stateflow_sfx.contains(needle));
    }
    for needle in ["Simulink.Bus", "Simulink.BusElement", "TelemetryPort"] {
        assert!(bundle.simulink_bus_m.contains(needle));
    }
    assert!(bundle.layer_contracts_json.contains("FlightController"));
    assert!(bundle.layer_contracts_json.contains("Domain"));

    let tmp_dir = std::env::temp_dir().join(format!("deap_mbd_export_test_{}", std::process::id()));
    std::fs::remove_dir_all(&tmp_dir).unwrap_or(());
    let written = export_mbd_artifacts(&model, &tmp_dir)?;
    assert_eq!(written.len(), 4);
    for name in ["requirements.slreqx", "stateflow_chart.sfx", "simulink_bus.m", "layer_contracts.json"] {
        assert!(tmp_dir.join(name).is_file());
    }
    std::fs::remove_dir_all(&tmp_dir).unwrap_or(());
    Ok(())
}

/// Asserts XML entity escaping across `.slreqx` and `.sfx` outputs for Defect 06.
/// Realises: [REQ-COMPILER-SYSML-TEST-DEFECT-06-MBD-XML-ESCAPING]
fn assert_defect_06_mbd_xml_escaping(bundle: &MbdExportBundle) {
    for needle in [
        "id=\"REQ-&quot;01&quot;-&apos;A&apos;\"",
        "name=\"Req&lt;Pitch&amp;Roll&gt;\"",
        "layer=\"Safety &amp; &quot;Control&quot; &lt;&apos;L1&apos;&gt;\"",
        "<slreq:Summary>Pitch &lt; 15.0 &amp; Roll &gt; -15.0 in &quot;Nominal&quot; &apos;Mode&apos;</slreq:Summary>",
        "<slreq:SatisfiedBy>FlightController&lt;Primary&amp;Backup&gt;</slreq:SatisfiedBy>",
        "<slreq:SafetyRealises>SC-01 &lt;&quot;Bound&quot; &amp; &apos;Safe&apos;&gt;</slreq:SafetyRealises>",
        "<slreq:TriggersHazards>H-01 &lt;&quot;Critical&quot; &amp; &apos;High&apos;&gt;</slreq:TriggersHazards>",
    ] {
        assert!(bundle.slreq_xml.contains(needle), "missing in slreq_xml: {needle}\n{}", bundle.slreq_xml);
    }
    for needle in [
        "source=\"Init\" target=\"SafeMode\"",
        "source=\"SafeMode\" target=\"Init\"",
        "trigger=\"FaultSignal\" guard=\"pitch_deg &gt; 15.0 &amp;&amp; roll_deg &lt; -10.0\" effect=\"EngageBackup\"",
        "entry=\"&quot;Init &lt; &apos;Safe&apos; &amp; Fast &gt;&quot;\"",
    ] {
        assert!(bundle.stateflow_sfx.contains(needle), "missing in stateflow_sfx: {needle}\n{}", bundle.stateflow_sfx);
    }
}

#[test]
fn test_defect_06_synthesize_mbd_bundle_transition_endpoints_and_xml_escaping(
) -> Result<(), Box<dyn std::error::Error>> {
    let src = r#"
        package AvionicsPkg {
            part def FlightController {
                state def FlightMode {
                    entry "Init < 'Safe' & Fast >";
                    state Init;
                    state SafeMode;
                    transition t1 first Init accept FaultSignal if pitch_deg > 15.0 && roll_deg < -10.0 do EngageBackup then SafeMode;
                    transition from SafeMode to Init;
                }
            }
        }
    "#;
    let pkg = parse_sysml(src)?;
    let mut model = SysmlModel::from(pkg);
    model.requirements.push(RequirementDef {
        name: "Req<Pitch&Roll>".to_string(),
        req_id: "REQ-\"01\"-'A'".to_string(),
        text: "Pitch < 15.0 & Roll > -15.0 in \"Nominal\" 'Mode'".to_string(),
        doc: None,
        attributes: Vec::new(),
        constraints: Vec::new(),
        assumes: Vec::new(),
        requires: Vec::new(),
        verified_by: Vec::new(),
        satisfied_by: vec!["FlightController<Primary&Backup>".to_string()],
        derived_from: Vec::new(),
        derives: Vec::new(),
        triggers_hazards: vec!["H-01 <\"Critical\" & 'High'>".to_string()],
        safety_realises: vec!["SC-01 <\"Bound\" & 'Safe'>".to_string()],
        layer: Some("Safety & \"Control\" <'L1'>".to_string()),
    });

    let bundle = synthesize_mbd_bundle(&model);
    assert_defect_06_mbd_xml_escaping(&bundle);
    Ok(())
}

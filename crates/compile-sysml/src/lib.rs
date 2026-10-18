//! SysML v2 / KerML compiler and specification synchronization engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the primary library facade for deterministic lexical analysis, syntactic parsing,
//! semantic validation, STPA/FMECA hazard synthesis, and bidirectional Markdown synchronization
//! under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic AST construction: identical UTF-8 input streams always yield structurally
//!   identical `PackageDef` syntax trees and cryptographic schema digests.
//! - Zero hardcoded domain concepts: all exported compiler primitives operate exclusively over
//!   abstract SysML v2 / KerML meta-model structures.
//!
//! ## 3. Preconditions
//! - Input SysML v2 source slices must be valid UTF-8 strings representing textual model
//!   declarations or workspace schema trees.
//!
//! ## 4. Postconditions
//! - Exported parser and synchronization entry points return strongly typed AST structures or
//!   explicit, span-annotated diagnostic errors without panicking or mutating global state.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Top-level parsing via `parse_sysml` operates in $O(N)$ time and $O(N)$ memory with respect
//!   to the byte length $N$ of the input source buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Lexical or grammatical syntax violations are captured as deterministic `ParseError` values
//!   carrying exact line, column, and byte span provenance.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-LIB], [DO-330-TQL-4-LIB]

pub mod docstring;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod stpa;
pub mod sync;

pub use deap_core::sysml_ast::*;
pub use docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions, DocstringInjectionReport,
};
pub use lexer::scanner::Scanner;
pub use lexer::token::{Span, Token, TokenKind};
pub use parser::expressions::{parse_expression, BinaryOp, Expr, ExpressionParser, UnaryOp};
pub use parser::grammar::{
    discover_sysml_files, merge_package_defs, parse_sysml_directory, parse_sysml_tree, ParseError,
    SysmlParser,
};
pub use semantic::digest::{
    compute_file_manifest, count_structural_elements, generate_digest, write_atomic,
    write_digest_atomic, FileDigestEntry, SchemaDigest,
};
pub use semantic::serializer::{to_sysml, SysmlSerializable};
pub use semantic::symbols::{Symbol, SymbolKind, SymbolTable};
pub use semantic::validator::{DiagnosticSeverity, SemanticDiagnostic, SemanticValidator};
pub use stpa::{
    compile_stpa_to_constraints, compile_stpa_to_constraints_filtered, emit_safety_suite, expand_cartesian_stpa, generate_fmeca_matrix,
    transpile_safety_suite, transpile_stpa, FmecaRow, FmecaScoringConfig, UnsafeControlAction,
};
pub use sync::{
    forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml, ForwardSyncOptions,
    ReverseSyncOptions,
};

/// Parse textual SysML v2 source into a strongly-typed PackageDef AST.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Serves as the canonical single-buffer entry point for parsing textual SysML v2 specifications
/// into an in-memory `PackageDef` abstract syntax tree under DO-330 TQL-4 determinism mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total function over `&str`: never panics or invokes undefined behavior on arbitrary input.
/// - Pure functional transformation: produces an isolated `PackageDef` root named `"SysML_Model"`.
///
/// ## 3. Preconditions
/// - `source` is an in-memory UTF-8 string slice containing SysML v2 declarations.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` containing all parsed child elements in declaration order when
///   `source` conforms to the grammar, or `Err(ParseError)` on the first syntax error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ where $N$ is `source.len()`.
/// - Space complexity: $O(N)$ proportional to token stream and AST node count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed tokens, unclosed delimiters, or unexpected keywords return `Err(ParseError)` with
///   source span coordinates.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-PARSE]
pub fn parse_sysml(source: &str) -> Result<PackageDef, ParseError> {
    SysmlParser::parse_source(source, "SysML_Model")
}

/// Deterministic Model-Based Design (MBD) artifact bundle synthesized from a `SysmlModel`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Encapsulates the four grounded MBD verification projections required for DO-178C Level A,
/// DO-331 model-based development, and ISO 26262 ASIL D bidirectional traceability between
/// SysML v2 architecture models and downstream simulation/verification toolchains.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every field is a deterministic function of the input `SysmlModel` AST with zero hardcoded
///   domain identifiers or synthetic placeholder blocks.
/// - `layer_contracts_json` serializes the exact sequence of `TypeDescriptor` contracts derived
///   via `TypeDescriptor::from_part_def`.
///
/// ## 3. Preconditions
/// - Constructed via `synthesize_mbd_bundle` from a validated `SysmlModel` instance.
///
/// ## 4. Postconditions
/// - Holds well-formed UTF-8 payloads for `.slreqx`, `.sfx`, `.m`, and `.json` MBD artifacts.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Space complexity: $O(R + S + P + A)$ proportional to the total requirements, state machines,
///   ports, parts, and attributes in the source `SysmlModel`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Value type with total equality and serialization semantics; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-MBD-BUNDLE]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MbdExportBundle {
    pub slreq_xml: String,
    pub stateflow_sfx: String,
    pub simulink_bus_m: String,
    pub layer_contracts_json: String,
}

/// Recursively collects all `PartDef` nodes in pre-order traversal from a slice of parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Guarantees complete hierarchical part discovery across nested SysML v2 composition trees
/// for DO-331 MBD bus, statechart, and layer-contract synthesis.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic pre-order declaration sequence across arbitrary composition depth.
///
/// ## 3. Preconditions
/// - `parts` is a valid slice of `PartDef` AST nodes; `out` is the destination accumulator vector.
///
/// ## 4. Postconditions
/// - Appends every root and nested `PartDef` in `parts` to `out` in pre-order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(V)$ where $V$ is the total number of `PartDef` nodes in the hierarchy.
/// - Space complexity: $O(V)$ for the accumulated `PartDef` clones plus $O(D)$ call-stack depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Finite tree traversal over owned AST vectors; cannot loop indefinitely or panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-COLLECT-PARTS]
fn collect_model_parts_recursive(parts: &[PartDef], out: &mut Vec<PartDef>) {
    for part in parts {
        out.push(part.clone());
        collect_model_parts_recursive(&part.parts, out);
    }
}

/// Synthesizes a grounded `MbdExportBundle` from an in-memory `SysmlModel` AST.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Generates traceable MATLAB Requirements Toolbox (`.slreqx`), Stateflow chart (`.sfx`),
/// `Simulink.Bus` (`.m`), and architectural layer contract (`.json`) projections under
/// DO-178C Level A, DO-330 TQL-4, and DO-331 model coverage mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Referentially transparent: identical `SysmlModel` inputs yield byte-for-byte identical
///   `MbdExportBundle` outputs.
/// - Preserves `@TriggersHazard`, `@SafetyRealises`, and `@Layer` stereotypes across both
///   requirement and part projections.
///
/// ## 3. Preconditions
/// - `model` is a populated `deap_core::sysml_ast::SysmlModel` reference.
///
/// ## 4. Postconditions
/// - Returns a populated `MbdExportBundle` containing all four grounded artifact strings.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(R \cdot P + S + B)$ where $R$ is requirement count, $P$ is part count,
///   $S$ is state/transition count, and $B$ is port/attribute bus element count.
/// - Space complexity: $O(R + S + P + B)$ for the synthesized output buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: handles empty models or missing optional fields gracefully without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-SYNTHESIZE-MBD]
pub fn synthesize_mbd_bundle(model: &deap_core::sysml_ast::SysmlModel) -> MbdExportBundle {
    let pkg_name = model.package_name.as_deref().unwrap_or("SysML_Model");
    let mut all_parts = Vec::new();
    collect_model_parts_recursive(&model.parts, &mut all_parts);

    let mut all_reqs: Vec<RequirementDef> = model.requirements.clone();
    for part in &all_parts {
        for req in &part.requirements {
            if !all_reqs.iter().any(|r| r.name == req.name && r.req_id == req.req_id) {
                all_reqs.push(req.clone());
            }
        }
    }

    let mut slreq_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    slreq_xml.push_str(&format!(
        "<slreq:RequirementSet xmlns:slreq=\"https://www.mathworks.com/slreq\" package=\"{pkg_name}\">\n"
    ));
    for req in &all_reqs {
        let id_str = if req.req_id.is_empty() {
            req.name.as_str()
        } else {
            req.req_id.as_str()
        };
        let layer_str = req.layer.as_deref().unwrap_or("Unspecified");
        let summary_str = if !req.text.is_empty() {
            req.text.as_str()
        } else {
            req.doc.as_deref().unwrap_or("")
        };
        let mut satisfied_by: Vec<String> = req.satisfied_by.clone();
        for part in &all_parts {
            if part
                .satisfied_requirements
                .iter()
                .any(|s| s == id_str || s == &req.name)
                && !satisfied_by.contains(&part.name)
            {
                satisfied_by.push(part.name.clone());
            }
        }
        slreq_xml.push_str(&format!(
            "  <slreq:Requirement id=\"{id_str}\" name=\"{}\" layer=\"{layer_str}\">\n",
            req.name
        ));
        slreq_xml.push_str(&format!("    <slreq:Summary>{summary_str}</slreq:Summary>\n"));
        slreq_xml.push_str(&format!(
            "    <slreq:SatisfiedBy>{}</slreq:SatisfiedBy>\n",
            satisfied_by.join(", ")
        ));
        slreq_xml.push_str(&format!(
            "    <slreq:SafetyRealises>{}</slreq:SafetyRealises>\n",
            req.safety_realises.join(", ")
        ));
        slreq_xml.push_str(&format!(
            "    <slreq:TriggersHazards>{}</slreq:TriggersHazards>\n",
            req.triggers_hazards.join(", ")
        ));
        slreq_xml.push_str("  </slreq:Requirement>\n");
    }
    slreq_xml.push_str("</slreq:RequirementSet>\n");

    let mut stateflow_sfx = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    stateflow_sfx.push_str(&format!("<StateflowChart package=\"{pkg_name}\">\n"));
    for part in &all_parts {
        for state_def in &part.states {
            stateflow_sfx.push_str(&format!(
                "  <Machine name=\"{}\" ownerPart=\"{}\">\n",
                state_def.name, part.name
            ));
            let root_entry = state_def.entry_action.as_deref().unwrap_or("");
            let root_do = state_def.do_action.as_deref().unwrap_or("");
            let root_exit = state_def.exit_action.as_deref().unwrap_or("");
            stateflow_sfx.push_str(&format!(
                "    <State name=\"{}\" entry=\"{root_entry}\" do=\"{root_do}\" exit=\"{root_exit}\" />\n",
                state_def.name
            ));
            for st in &state_def.sub_states {
                let entry_act = st.entry_action.as_deref().unwrap_or("");
                let do_act = st.do_action.as_deref().unwrap_or("");
                let exit_act = st.exit_action.as_deref().unwrap_or("");
                stateflow_sfx.push_str(&format!(
                    "    <State name=\"{}\" entry=\"{entry_act}\" do=\"{do_act}\" exit=\"{exit_act}\" />\n",
                    st.name
                ));
            }
            for tr in &state_def.transitions {
                let tr_name = tr.name.as_deref().unwrap_or("");
                let src = tr.source.as_deref().unwrap_or("");
                let tgt = tr.target.as_deref().unwrap_or("");
                let trig = tr.trigger.as_deref().unwrap_or("");
                let guard = tr.guard.as_deref().unwrap_or("");
                let effect = tr.effect.as_deref().unwrap_or("");
                stateflow_sfx.push_str(&format!(
                    "    <Transition name=\"{tr_name}\" source=\"{src}\" target=\"{tgt}\" trigger=\"{trig}\" guard=\"{guard}\" effect=\"{effect}\" />\n"
                ));
            }
            stateflow_sfx.push_str("  </Machine>\n");
        }
    }
    stateflow_sfx.push_str("</StateflowChart>\n");

    let mut simulink_bus_m = format!(
        "% Auto-generated Simulink.Bus definitions for package {pkg_name}\n"
    );
    for port_def in &model.ports {
        simulink_bus_m.push_str(&format!(
            "{}_Bus = Simulink.Bus;\n{}_Bus.Elements = [\n",
            port_def.name, port_def.name
        ));
        simulink_bus_m.push_str(&format!(
            "  Simulink.BusElement('Name', '{}', 'DataType', '{}', 'Direction', '{}');\n",
            port_def.name, port_def.type_name, port_def.direction
        ));
        for flow in &port_def.item_flows {
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{}', 'DataType', '{}', 'Direction', '{}');\n",
                flow.name, flow.item_type, flow.direction
            ));
        }
        for (elec_k, elec_v) in &port_def.electrical_attributes {
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{elec_k}', 'DataType', '{elec_v}');\n"
            ));
        }
        simulink_bus_m.push_str("];\n");
    }
    for part in &all_parts {
        simulink_bus_m.push_str(&format!(
            "{}_Bus = Simulink.Bus;\n{}_Bus.Elements = [\n",
            part.name, part.name
        ));
        for attr in &part.attributes {
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{}', 'DataType', '{}');\n",
                attr.name, attr.type_name
            ));
        }
        for port in &part.ports {
            let port_ty = if port.type_name.is_empty() {
                "Bus"
            } else {
                port.type_name.as_str()
            };
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{}', 'DataType', '{port_ty}', 'Direction', '{}');\n",
                port.name, port.direction
            ));
        }
        simulink_bus_m.push_str("];\n");
    }

    let descriptors: Vec<deap_core::sysml_ast::TypeDescriptor> = all_parts
        .iter()
        .map(|part| deap_core::sysml_ast::TypeDescriptor::from_part_def(pkg_name, part))
        .collect();
    let layer_contracts_json =
        serde_json::to_string_pretty(&descriptors).unwrap_or_else(|_| "[]".to_string());

    MbdExportBundle {
        slreq_xml,
        stateflow_sfx,
        simulink_bus_m,
        layer_contracts_json,
    }
}

/// Synthesizes and writes the four MBD verification artifacts to `out_dir`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides deterministic filesystem persistence for MBD synthesis artifacts (`.slreqx`, `.sfx`,
/// `.m`, and `layer_contracts.json`) under DO-330 TQL-4 tool qualification constraints.
///
/// ## 2. Invariants & Mathematical Properties
/// - Writes exactly four files in deterministic order: `requirements.slreqx`,
///   `stateflow_chart.sfx`, `simulink_bus.m`, and `layer_contracts.json`.
/// - Propagates all I/O failures explicitly via `Result<Vec<PathBuf>, String>` with zero silent drops.
///
/// ## 3. Preconditions
/// - `model` is a valid `SysmlModel` reference; `out_dir` is a writable target directory path.
///
/// ## 4. Postconditions
/// - Returns `Ok(Vec<PathBuf>)` containing the 4 written file paths upon success, or `Err(String)`
///   describing any directory creation or file write error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ where $N$ is the total byte size of the synthesized MBD bundle.
/// - Space complexity: $O(N)$ in-memory buffer allocation prior to flush.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Filesystem permission or I/O errors return descriptive `Err(String)` diagnostics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-EXPORT-MBD]
pub fn export_mbd_artifacts(
    model: &deap_core::sysml_ast::SysmlModel,
    out_dir: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "Failed to create MBD export directory '{}': {e}",
            out_dir.display()
        )
    })?;
    let bundle = synthesize_mbd_bundle(model);

    let slreq_path = out_dir.join("requirements.slreqx");
    std::fs::write(&slreq_path, &bundle.slreq_xml).map_err(|e| {
        format!(
            "Failed to write MBD requirements artifact '{}': {e}",
            slreq_path.display()
        )
    })?;

    let sfx_path = out_dir.join("stateflow_chart.sfx");
    std::fs::write(&sfx_path, &bundle.stateflow_sfx).map_err(|e| {
        format!(
            "Failed to write MBD Stateflow artifact '{}': {e}",
            sfx_path.display()
        )
    })?;

    let bus_path = out_dir.join("simulink_bus.m");
    std::fs::write(&bus_path, &bundle.simulink_bus_m).map_err(|e| {
        format!(
            "Failed to write MBD Simulink bus artifact '{}': {e}",
            bus_path.display()
        )
    })?;

    let contracts_path = out_dir.join("layer_contracts.json");
    std::fs::write(&contracts_path, &bundle.layer_contracts_json).map_err(|e| {
        format!(
            "Failed to write MBD layer contracts artifact '{}': {e}",
            contracts_path.display()
        )
    })?;

    Ok(vec![slreq_path, sfx_path, bus_path, contracts_path])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_sysml_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        let module_doc: String = src
            .lines()
            .take_while(|line| line.starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");

        assert!(module_doc.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(module_doc.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(module_doc.contains("//! ## 3. Preconditions"));
        assert!(module_doc.contains("//! ## 4. Postconditions"));
        assert!(module_doc.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(module_doc.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(module_doc.contains("//! ## 7. Realization & Traceability"));
        assert!(module_doc.contains("//! Realises: [REQ-COMPILER-SYSML-LIB], [DO-330-TQL-4-LIB]"));

        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("/// ## 3. Preconditions"));
        assert!(src.contains("/// ## 4. Postconditions"));
        assert!(src.contains("/// ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("/// ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("/// ## 7. Realization & Traceability"));
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-PARSE]"));

        let parsed = parse_sysml("package Demo {}");
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_compile_sysml_preserves_custom_safety_and_layer_stereotypes() {
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
        let pkg = parse_sysml(src).expect("should parse custom safety stereotypes");
        let part = pkg.part_defs.first().expect("FlightController should exist");
        assert_eq!(
            part.triggers_hazards,
            vec!["H-01".to_string(), "H-02".to_string()]
        );
        assert_eq!(part.safety_realises, vec!["SC-01".to_string()]);
        assert_eq!(part.layer.as_deref(), Some("Domain"));

        let req = pkg
            .requirement_defs
            .first()
            .expect("PitchBoundReq should exist");
        assert_eq!(req.triggers_hazards, vec!["H-01".to_string()]);
        assert_eq!(req.safety_realises, vec!["SC-01".to_string()]);
        assert_eq!(req.layer.as_deref(), Some("Safety"));
    }

    #[test]
    fn test_synthesize_mbd_bundle_and_export_artifacts() {
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
        let pkg = parse_sysml(src).expect("should parse AvionicsPkg");
        let model = deap_core::sysml_ast::SysmlModel::from(pkg);
        let bundle = synthesize_mbd_bundle(&model);
        assert!(bundle.slreq_xml.contains("<slreq:Requirement"));
        assert!(bundle.slreq_xml.contains("REQ-FC-01"));
        assert!(bundle.slreq_xml.contains("SC-01"));
        assert!(bundle.slreq_xml.contains("H-01"));
        assert!(bundle.stateflow_sfx.contains("FlightMode"));
        assert!(bundle.stateflow_sfx.contains("Init"));
        assert!(bundle.stateflow_sfx.contains("Active"));
        assert!(bundle.simulink_bus_m.contains("Simulink.Bus"));
        assert!(bundle.simulink_bus_m.contains("Simulink.BusElement"));
        assert!(bundle.simulink_bus_m.contains("TelemetryPort"));
        assert!(bundle.layer_contracts_json.contains("FlightController"));
        assert!(bundle.layer_contracts_json.contains("Domain"));

        let tmp_dir = std::env::temp_dir().join(format!(
            "deap_mbd_export_test_{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&tmp_dir).unwrap_or(());
        let written = export_mbd_artifacts(&model, &tmp_dir).expect("export should succeed");
        assert_eq!(written.len(), 4);
        assert!(tmp_dir.join("requirements.slreqx").is_file());
        assert!(tmp_dir.join("stateflow_chart.sfx").is_file());
        assert!(tmp_dir.join("simulink_bus.m").is_file());
        assert!(tmp_dir.join("layer_contracts.json").is_file());
        std::fs::remove_dir_all(&tmp_dir).unwrap_or(());
    }
}


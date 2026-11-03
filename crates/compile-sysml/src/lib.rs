//! SysML v2 / KerML compiler and specification synchronization engine.
//! ## 1. Safety Intent & Regulatory Scope
//! Primary library facade for deterministic lexical analysis, syntactic parsing, semantic validation,
//! STPA/FMECA synthesis, and Markdown sync under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D.
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic AST construction: identical UTF-8 inputs yield identical `PackageDef` trees and digests.
//! - Zero hardcoded domain concepts: primitives operate over abstract SysML v2 / KerML structures.
//! ## 3. Preconditions
//! - Input SysML v2 source slices must be valid UTF-8 strings representing model or workspace trees.
//! ## 4. Postconditions
//! - Entry points return strongly typed AST structures or span-annotated errors without panicking.
//! ## 5. Algorithmic Complexity Bounds
//! - Top-level parsing via `parse_sysml` operates in $O(N)$ time and $O(N)$ memory for byte length $N$.
//! ## 6. Failure Modes & Fault Tolerance
//! - Syntax violations are captured as deterministic `ParseError` values with line, column, and span.
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-LIB], [DO-330-TQL-4-LIB]
pub mod cli;
pub(crate) mod cli_command_builder;
pub mod cst_cli;
pub mod docstring;
pub(crate) mod docstring_compliance_check;
pub(crate) mod docstring_contract_synth;
pub mod docstring_cst;
pub(crate) mod docstring_dart_codemod;
pub(crate) mod docstring_dart_inspect;
pub(crate) mod docstring_rust_codemod;
pub(crate) mod docstring_rust_inspect;
pub(crate) mod docstring_rust_scan;
pub(crate) mod docstring_schema_bind;
pub(crate) mod docstring_schema_index;
pub mod driver;
pub mod driver_cst_tracks;
pub(crate) mod driver_mode_handlers;
pub(crate) mod driver_schema_loader;
#[cfg(test)]
#[path = "../tests/cli_integration.rs"]
mod cli_integration;
#[cfg(test)]
#[path = "../tests/docstring_tests.rs"]
mod docstring_tests;
pub mod flutter_codegen;
pub mod formal_harnesses;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod lib_tests_b;
pub mod modular_codegen;
pub mod salsa_db;
pub mod stpa;
pub mod syn_codegen;
pub mod sync;
pub use cli::*;
pub use deap_core::sysml_cst::*;
pub use docstring::{inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection, run_extract_source, DocstringExitCode, DocstringInjectionOptions, DocstringInjectionReport};
pub use docstring_cst::*;
pub use driver::*;
pub use flutter_codegen::{emit_flutter_contracts, FlutterCodegenError, FlutterContractOptions, FlutterContractReport};
pub use salsa_db::{
    compute_file_manifest, count_structural_elements, discover_sysml_files, generate_digest,
    merge_package_defs, parse_expression, parse_sysml_directory, parse_sysml_tree, to_sysml,
    write_atomic, write_digest_atomic, BinaryOp, DiagnosticSeverity, Expr, ExpressionParser,
    FileDigestEntry, ParseError, Scanner, SchemaDigest, SemanticDiagnostic, SemanticValidator,
    Span, Symbol, SymbolKind, SymbolTable, SysmlIncrementalDb, SysmlParser, SysmlSerializable,
    Token, TokenKind, UnaryOp,
};
pub use stpa::{compile_stpa_to_constraints, compile_stpa_to_constraints_filtered, emit_safety_suite, expand_cartesian_stpa, generate_fmeca_matrix, transpile_safety_suite, transpile_stpa, FmecaRow, FmecaScoringConfig, UnsafeControlAction};
pub use syn_codegen::{format_and_verify_rust_module_ast, synthesize_prettyplease_unit, synthesize_typed_domain_struct_tokens, verify_stage_fixed_point, StageTreeDigestManifest, SynCodegenFixedPointReceipt};
pub use sync::{forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml, ForwardSyncOptions, ReverseSyncOptions};

/// Parse textual SysML v2 source into a strongly-typed PackageDef AST.
/// ## 1. Safety Intent & Regulatory Scope -- Canonical single-buffer SysML v2 parser under DO-330 TQL-4.
/// ## 2. Invariants & Mathematical Properties -- Total pure function returning root `"SysML_Model"`.
/// ## 3. Preconditions -- `source` is an in-memory UTF-8 string slice of SysML v2 declarations.
/// ## 4. Postconditions -- Returns `Ok(PackageDef)` in declaration order or `Err(ParseError)`.
/// ## 5. Algorithmic Complexity Bounds -- Time: $O(N)$, Space: $O(N)$ for input byte length $N$.
/// ## 6. Failure Modes & Fault Tolerance -- Malformed syntax returns `Err(ParseError)` with span.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-PARSE]
pub fn parse_sysml(source: &str) -> Result<PackageDef, ParseError> {
    SysmlParser::parse_source(source, "SysML_Model")
}

/// Deterministic Model-Based Design (MBD) artifact bundle synthesized from a `SysmlModel`.
/// ## 1. Safety Intent & Regulatory Scope -- Encapsulates DO-178C Level A / DO-331 MBD verification projections.
/// ## 2. Invariants & Mathematical Properties -- Deterministic projection of `SysmlModel` with zero placeholders.
/// ## 3. Preconditions -- Constructed via `synthesize_mbd_bundle` from a validated `SysmlModel`.
/// ## 4. Postconditions -- Holds well-formed UTF-8 payloads for `.slreqx`, `.sfx`, `.m`, and `.json`.
/// ## 5. Algorithmic Complexity Bounds -- Space: $O(R + S + P + A)$ over AST elements.
/// ## 6. Failure Modes & Fault Tolerance -- Value type with total equality and serialization semantics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-MBD-BUNDLE]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MbdExportBundle {
    pub slreq_xml: String,
    pub stateflow_sfx: String,
    pub simulink_bus_m: String,
    pub layer_contracts_json: String,
}

/// Iteratively collects all `PartDef` references in pre-order traversal using an explicit `Vec` work-stack.
/// Realises: [REQ-COMPILER-SYSML-COLLECT-PARTS]
pub(crate) fn collect_model_parts_iterative<'a>(parts: &'a [PartDef], out: &mut Vec<&'a PartDef>) {
    let mut stack: Vec<&'a PartDef> = parts.iter().rev().collect();
    while let Some(part) = stack.pop() {
        out.push(part);
        stack.extend(part.parts.iter().rev());
    }
}

/// Escapes single-quoted MATLAB string literals by doubling apostrophes (`'` -> `''`).
/// ## 1. Safety Intent & Regulatory Scope -- Prevents MATLAB syntax injection in `Simulink.BusElement` codegen.
/// ## 2. Invariants & Mathematical Properties -- Maps every `'\''` to `"''"` while preserving all other chars.
/// ## 3. Preconditions -- `s` is any valid UTF-8 string slice.
/// ## 4. Postconditions -- Returns an owned `String` safe for single-quoted MATLAB string interpolation.
/// ## 5. Algorithmic Complexity Bounds -- Time: $O(N)$, Space: $O(N)$ linear in `s.len()`.
/// ## 6. Failure Modes & Fault Tolerance -- Total function; never panics or fails on any input.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-ESCAPE-MATLAB]
pub fn escape_matlab_single_quoted(s: &str) -> String {
    s.replace('\'', "''")
}

/// Iteratively flattens all nested subpackages in `root` into a single `PackageDef` in pre-order.
/// Realises: [REQ-COMPILER-SYSML-FLATTEN-ITERATIVE]
fn flatten_package_tree_iterative(mut root: PackageDef) -> PackageDef {
    let mut stack: Vec<PackageDef> = std::mem::take(&mut root.packages);
    stack.reverse();
    while let Some(mut sub) = stack.pop() {
        stack.extend(std::mem::take(&mut sub.packages).into_iter().rev());
        merge_package_defs(&mut root, sub);
    }
    for part in &mut root.part_defs {
        for item in &root.item_defs {
            if !part.item_defs.iter().any(|existing| existing.name == item.name) {
                part.item_defs.push(item.clone());
            }
        }
    }
    root
}

/// Flattens a hierarchical `PackageDef` tree into a consolidated `SysmlModel` using an iterative stack.
/// ## 1. Safety Intent & Regulatory Scope -- Guarantees deep package hierarchies retain all declarations in `SysmlModel`.
/// ## 2. Invariants & Mathematical Properties -- Zero-recursion $O(1)$ call-stack traversal with cross-package retention.
/// ## 3. Preconditions -- `pkg` is a parsed `PackageDef` root; `target_pkg` is an optional slice target.
/// ## 4. Postconditions -- Returns a consolidated `SysmlModel` containing all flattened AST definitions.
/// ## 5. Algorithmic Complexity Bounds -- Time: $O(P + E)$, Space: $O(P + E)$ over packages and elements.
/// ## 6. Failure Modes & Fault Tolerance -- Total function over `PackageDef` with zero recursion or panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-FLATTEN-MODEL]
pub fn flatten_package_to_model(pkg: PackageDef, target_pkg: Option<&str>) -> SysmlModel {
    if let Some(target) = target_pkg {
        let (mut matched_sub, mut search_stack): (Option<PackageDef>, Vec<&PackageDef>) = (None, pkg.packages.iter().rev().collect());
        while let Some(curr) = search_stack.pop() {
            if curr.name.eq_ignore_ascii_case(target) {
                matched_sub = Some(curr.clone());
                break;
            }
            search_stack.extend(curr.packages.iter().rev());
        }
        if let Some(sub_pkg) = matched_sub {
            let (mut target_flat, all_flat) = (flatten_package_tree_iterative(sub_pkg), flatten_package_tree_iterative(pkg));
            for req in all_flat.requirement_defs {
                if !target_flat.requirement_defs.iter().any(|r| r.name == req.name && r.req_id == req.req_id) {
                    target_flat.requirement_defs.push(req);
                }
            }
            for port in all_flat.port_defs {
                if !target_flat.port_defs.iter().any(|p| p.name == port.name) {
                    target_flat.port_defs.push(port);
                }
            }
            for item in all_flat.item_defs {
                if !target_flat.item_defs.iter().any(|i| i.name == item.name) {
                    target_flat.item_defs.push(item.clone());
                }
                for part in &mut target_flat.part_defs {
                    if !part.item_defs.iter().any(|i| i.name == item.name) {
                        part.item_defs.push(item.clone());
                    }
                }
            }
            return SysmlModel::from(target_flat);
        }
    }
    SysmlModel::from(flatten_package_tree_iterative(pkg))
}

/// Escapes reserved XML characters (`&`, `<`, `>`, `"`, `'`) into predefined XML entities.
/// ## 1. Safety Intent & Regulatory Scope -- Ensures well-formed `.slreqx` and `.sfx` XML generation under DO-178C Level A.
/// ## 2. Invariants & Mathematical Properties -- Replaces `&`, `<`, `>`, `"`, `'` with predefined XML entities.
/// ## 3. Preconditions -- `s` is any valid UTF-8 string slice.
/// ## 4. Postconditions -- Returns an owned `String` containing zero unescaped XML metacharacters.
/// ## 5. Algorithmic Complexity Bounds -- Time: $O(N)$, Space: $O(N)$ linear in `s.len()`.
/// ## 6. Failure Modes & Fault Tolerance -- Total function; never panics or fails on any input string.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-ESCAPE-XML]
pub fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Synthesizes `.slreqx` XML requirements traceability payload from `model` and `all_parts`.
/// Realises: [REQ-COMPILER-SYSML-SYNTHESIZE-MBD]
fn render_slreq_xml(pkg_name_xml: &str, model: &SysmlModel, all_parts: &[&PartDef]) -> String {
    let (mut all_reqs, mut seen_reqs) = (Vec::new(), std::collections::BTreeSet::new());
    for req in model.requirements.iter().chain(all_parts.iter().flat_map(|p| &p.requirements)) {
        if seen_reqs.insert((req.name.as_str(), req.req_id.as_str())) {
            all_reqs.push(req);
        }
    }
    let mut req_to_parts: std::collections::BTreeMap<&str, Vec<(usize, &str)>> = std::collections::BTreeMap::new();
    for (idx, part) in all_parts.iter().enumerate() {
        for sat in &part.satisfied_requirements {
            req_to_parts.entry(sat.as_str()).or_default().push((idx, part.name.as_str()));
        }
    }
    let mut xml = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<slreq:RequirementSet xmlns:slreq=\"https://www.mathworks.com/slreq\" package=\"{pkg_name_xml}\">\n");
    for req in all_reqs {
        let id_str = if req.req_id.is_empty() { req.name.as_str() } else { req.req_id.as_str() };
        let summary_str = if !req.text.is_empty() { req.text.as_str() } else { req.doc.as_deref().unwrap_or("") };
        let (mut satisfied_by, mut sat_seen) = (Vec::new(), std::collections::BTreeSet::new());
        for existing in &req.satisfied_by {
            if sat_seen.insert(existing.clone()) {
                satisfied_by.push(existing.clone());
            }
        }
        let mut matched: std::collections::BTreeMap<usize, &str> = std::collections::BTreeMap::new();
        for key in [id_str, req.name.as_str()] {
            if let Some(entries) = req_to_parts.get(key) {
                matched.extend(entries.iter().copied());
            }
        }
        for p_name in matched.into_values() {
            if sat_seen.insert(p_name.to_string()) {
                satisfied_by.push(p_name.to_string());
            }
        }
        xml.push_str(&format!("  <slreq:Requirement id=\"{}\" name=\"{}\" layer=\"{}\">\n    <slreq:Summary>{}</slreq:Summary>\n    <slreq:SatisfiedBy>{}</slreq:SatisfiedBy>\n    <slreq:SafetyRealises>{}</slreq:SafetyRealises>\n    <slreq:TriggersHazards>{}</slreq:TriggersHazards>\n  </slreq:Requirement>\n", escape_xml(id_str), escape_xml(&req.name), escape_xml(req.layer.as_deref().unwrap_or("Unspecified")), escape_xml(summary_str), escape_xml(&satisfied_by.join(", ")), escape_xml(&req.safety_realises.join(", ")), escape_xml(&req.triggers_hazards.join(", "))));
    }
    xml.push_str("</slreq:RequirementSet>\n");
    xml
}

/// Synthesizes `.sfx` Stateflow chart XML payload from `all_parts`.
/// Realises: [REQ-COMPILER-SYSML-SYNTHESIZE-MBD]
fn render_stateflow_sfx(pkg_name_xml: &str, all_parts: &[&PartDef]) -> String {
    let mut sfx = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<StateflowChart package=\"{pkg_name_xml}\">\n");
    for part in all_parts {
        let part_xml = escape_xml(&part.name);
        for sd in &part.states {
            sfx.push_str(&format!("  <Machine name=\"{}\" ownerPart=\"{part_xml}\">\n", escape_xml(&sd.name)));
            for st in std::iter::once(sd).chain(sd.sub_states.iter()) {
                sfx.push_str(&format!("    <State name=\"{}\" entry=\"{}\" do=\"{}\" exit=\"{}\" />\n", escape_xml(&st.name), escape_xml(st.entry_action.as_deref().unwrap_or("")), escape_xml(st.do_action.as_deref().unwrap_or("")), escape_xml(st.exit_action.as_deref().unwrap_or(""))));
            }
            for tr in &sd.transitions {
                sfx.push_str(&format!("    <Transition name=\"{}\" source=\"{}\" target=\"{}\" trigger=\"{}\" guard=\"{}\" effect=\"{}\" />\n", escape_xml(tr.name.as_deref().unwrap_or("")), escape_xml(tr.source.as_deref().unwrap_or("")), escape_xml(tr.target.as_deref().unwrap_or("")), escape_xml(tr.trigger.as_deref().unwrap_or("")), escape_xml(tr.guard.as_deref().unwrap_or("")), escape_xml(tr.effect.as_deref().unwrap_or(""))));
            }
            sfx.push_str("  </Machine>\n");
        }
    }
    sfx.push_str("</StateflowChart>\n");
    sfx
}

/// Synthesizes `.m` `Simulink.Bus` definitions from `model` and `all_parts`.
/// Realises: [REQ-COMPILER-SYSML-SYNTHESIZE-MBD]
fn render_simulink_bus_m(pkg_name: &str, model: &SysmlModel, all_parts: &[&PartDef]) -> String {
    let mut m = format!("% Auto-generated Simulink.Bus definitions for package {pkg_name}\n");
    for pd in &model.ports {
        m.push_str(&format!("{0}_Bus = Simulink.Bus;\n{0}_Bus.Elements = [\n  Simulink.BusElement('Name', '{1}', 'DataType', '{2}', 'Direction', '{3}');\n", pd.name, escape_matlab_single_quoted(&pd.name), escape_matlab_single_quoted(&pd.type_name), escape_matlab_single_quoted(&pd.direction)));
        for f in &pd.item_flows {
            m.push_str(&format!("  Simulink.BusElement('Name', '{}', 'DataType', '{}', 'Direction', '{}');\n", escape_matlab_single_quoted(&f.name), escape_matlab_single_quoted(&f.item_type), escape_matlab_single_quoted(&f.direction)));
        }
        for (ek, ev) in &pd.electrical_attributes {
            m.push_str(&format!("  Simulink.BusElement('Name', '{}', 'DataType', '{}');\n", escape_matlab_single_quoted(ek), escape_matlab_single_quoted(ev)));
        }
        m.push_str("];\n");
    }
    for part in all_parts {
        m.push_str(&format!("{0}_Bus = Simulink.Bus;\n{0}_Bus.Elements = [\n", part.name));
        for attr in &part.attributes {
            m.push_str(&format!("  Simulink.BusElement('Name', '{}', 'DataType', '{}');\n", escape_matlab_single_quoted(&attr.name), escape_matlab_single_quoted(&attr.type_name)));
        }
        for port in &part.ports {
            let port_ty = if port.type_name.is_empty() { "Bus" } else { port.type_name.as_str() };
            m.push_str(&format!("  Simulink.BusElement('Name', '{}', 'DataType', '{}', 'Direction', '{}');\n", escape_matlab_single_quoted(&port.name), escape_matlab_single_quoted(port_ty), escape_matlab_single_quoted(&port.direction)));
        }
        m.push_str("];\n");
    }
    m
}
/// Synthesizes a grounded `MbdExportBundle` from an in-memory `SysmlModel` AST.
/// ## 1. Safety Intent & Regulatory Scope -- Generates traceable `.slreqx`, `.sfx`, `.m`, and `.json` MBD projections.
/// ## 2. Invariants & Mathematical Properties -- Referentially transparent; preserves safety and layer stereotypes.
/// ## 3. Preconditions -- `model` is a populated `SysmlModel` reference.
/// ## 4. Postconditions -- Returns a populated `MbdExportBundle` containing all four grounded artifact strings.
/// ## 5. Algorithmic Complexity Bounds -- Time: $O((R + P) \log(R + P) + S + B)$, Space: $O(R + S + P + B)$.
/// ## 6. Failure Modes & Fault Tolerance -- Total function; handles empty models without panicking.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-SYNTHESIZE-MBD]
pub fn synthesize_mbd_bundle(model: &SysmlModel) -> MbdExportBundle {
    let pkg_name = model.package_name.as_deref().unwrap_or("SysML_Model");
    let pkg_name_xml = escape_xml(pkg_name);
    let mut all_parts: Vec<&PartDef> = Vec::new();
    collect_model_parts_iterative(&model.parts, &mut all_parts);
    let descriptors: Vec<TypeDescriptor> = all_parts.iter().map(|&p| TypeDescriptor::from_part_def(pkg_name, p)).collect();
    let layer_contracts_json = serde_json::to_string_pretty(&descriptors).unwrap_or_else(|_| "[]".to_string());
    MbdExportBundle { slreq_xml: render_slreq_xml(&pkg_name_xml, model, &all_parts), stateflow_sfx: render_stateflow_sfx(&pkg_name_xml, &all_parts), simulink_bus_m: render_simulink_bus_m(pkg_name, model, &all_parts), layer_contracts_json }
}
/// Synthesizes and writes the four MBD verification artifacts to `out_dir`.
/// ## 1. Safety Intent & Regulatory Scope -- Deterministic persistence for MBD artifacts under DO-330 TQL-4.
/// ## 2. Invariants & Mathematical Properties -- Writes `requirements.slreqx`, `stateflow_chart.sfx`, `simulink_bus.m`, and `layer_contracts.json` atomically.
/// ## 3. Preconditions -- `model` is a valid `SysmlModel` reference; `out_dir` is a writable directory path.
/// ## 4. Postconditions -- Returns `Ok(Vec<PathBuf>)` with the 4 written file paths or `Err(String)`.
/// ## 5. Algorithmic Complexity Bounds -- Time: $O(N)$, Space: $O(N)$ over synthesized MBD bundle bytes.
/// ## 6. Failure Modes & Fault Tolerance -- Propagates directory creation, serialization, and atomic write errors.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-EXPORT-MBD]
pub fn export_mbd_artifacts(model: &SysmlModel, out_dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| format!("Failed to create MBD export directory '{}': {e}", out_dir.display()))?;
    let pkg_name = model.package_name.as_deref().unwrap_or("SysML_Model");
    let mut all_parts: Vec<&PartDef> = Vec::new();
    collect_model_parts_iterative(&model.parts, &mut all_parts);
    let descriptors: Vec<TypeDescriptor> = all_parts.iter().map(|&p| TypeDescriptor::from_part_def(pkg_name, p)).collect();
    let mut bundle = synthesize_mbd_bundle(model);
    bundle.layer_contracts_json = serde_json::to_string_pretty(&descriptors).map_err(|e| format!("Failed to serialize MBD TypeDescriptor contracts to JSON: {e}"))?;
    let mut written = Vec::with_capacity(4);
    for (file_name, label, content) in [("requirements.slreqx", "requirements", &bundle.slreq_xml), ("stateflow_chart.sfx", "Stateflow", &bundle.stateflow_sfx), ("simulink_bus.m", "Simulink bus", &bundle.simulink_bus_m), ("layer_contracts.json", "layer contracts", &bundle.layer_contracts_json)] {
        let path = out_dir.join(file_name);
        crate::salsa_db::write_atomic(&path, content).map_err(|e| format!("Failed to write MBD {label} artifact '{}': {e}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

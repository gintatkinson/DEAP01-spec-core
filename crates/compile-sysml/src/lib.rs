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
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::field_reassign_with_default
    )
)]

pub mod cli;
pub mod docstring;
pub mod driver;
pub mod flutter_codegen;
pub mod lexer;
pub mod parser;
#[allow(clippy::needless_lifetimes)]
pub mod semantic;
pub mod stpa;
pub mod sync;

pub use cli::*;
pub use deap_core::sysml_ast::*;
pub use docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions, DocstringInjectionReport,
};
pub use driver::*;
pub use flutter_codegen::{
    emit_flutter_contracts, FlutterCodegenError, FlutterContractOptions, FlutterContractReport,
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

/// Escapes single-quoted MATLAB string literals by doubling apostrophes (`'` -> `''`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents MATLAB syntax injection and malformed `Simulink.BusElement` script generation when
/// SysML v2 identifiers, types, or electrical attributes contain single quotes under DO-178C Level A
/// and DO-330 TQL-4 code generation standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure total function over UTF-8 strings: every ASCII apostrophe `'\''` is mapped to `"''"` while
///   all other Unicode scalar values are preserved verbatim.
///
/// ## 3. Preconditions
/// - `s` is any valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns an owned `String` safe for interpolation inside a single-quoted MATLAB string literal.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ single linear pass where $N$ is `s.len()`.
/// - Space complexity: $O(N)$ bounded output buffer allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: never panics or fails on any input string.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-ESCAPE-MATLAB]
pub fn escape_matlab_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("''");
        } else {
            out.push(ch);
        }
    }
    out
}

/// Iteratively flattens all nested subpackages in `root` into a single `PackageDef` in pre-order.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Eliminates unbounded call-stack recursion when collapsing arbitrary-depth SysML v2 package
/// hierarchies (`d >= 2`) under DO-178C Level A and DO-330 TQL-4 bounded-resource mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero recursion: uses an explicit `Vec<PackageDef>` LIFO stack reversed per child list to
///   preserve deterministic pre-order declaration sequence.
/// - Propagates package-level `item_defs` onto `part.item_defs` so `SysmlModel` projections retain
///   all `ItemDef` nodes across nested packages.
///
/// ## 3. Preconditions
/// - `root` is a valid `PackageDef` AST node.
///
/// ## 4. Postconditions
/// - Returns `root` with `root.packages.is_empty()` and all descendant definitions merged in-order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(P + E)$ where $P$ is package count and $E$ is total AST definition count.
/// - Space complexity: $O(P + E)$ heap memory for the iterative work stack.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function over owned AST vectors; cannot overflow the call stack or panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-FLATTEN-ITERATIVE]
fn flatten_package_tree_iterative(mut root: deap_core::sysml_ast::PackageDef) -> deap_core::sysml_ast::PackageDef {
    let mut stack: Vec<deap_core::sysml_ast::PackageDef> = std::mem::take(&mut root.packages);
    stack.reverse();
    while let Some(mut sub) = stack.pop() {
        let children = std::mem::take(&mut sub.packages);
        for child in children.into_iter().rev() {
            stack.push(child);
        }
        merge_package_defs(&mut root, sub);
    }
    if !root.item_defs.is_empty() {
        for part in &mut root.part_defs {
            for item in &root.item_defs {
                if !part.item_defs.iter().any(|existing| existing.name == item.name) {
                    part.item_defs.push(item.clone());
                }
            }
        }
    }
    root
}

/// Flattens a hierarchical `PackageDef` tree into a consolidated `SysmlModel` using an iterative stack.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Guarantees that multi-level nested SysML v2 package hierarchies (`d >= 2`) retain 100% of their
/// `PartDef`, `PortDef`, `ItemDef`, and `RequirementDef` declarations when projected into `SysmlModel`
/// for MBD synthesis, Flutter contract codegen, and subsystem slicing under DO-178C Level A and DO-330 TQL-4.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero-recursion traversal: uses an explicit iterative `Vec<PackageDef>` work stack so stack space
///   is $O(1)$ on the call stack regardless of package nesting depth $d$.
/// - When `target_pkg` matches a subpackage at any depth, flattens that subpackage's entire subtree
///   while preserving cross-package `requirement_defs`, `port_defs`, and `item_defs`.
/// - When `target_pkg` is `None` (or matches a part rather than a subpackage), flattens all nested
///   subpackages across all depths into the root package in pre-order.
///
/// ## 3. Preconditions
/// - `pkg` is a parsed `PackageDef` root; `target_pkg` is an optional package or part slice target.
///
/// ## 4. Postconditions
/// - Returns a consolidated `deap_core::sysml_ast::SysmlModel` containing all flattened AST definitions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(P + E)$ where $P$ is total package count and $E$ is total AST element count.
/// - Space complexity: $O(P + E)$ heap memory for the iterative work stack and consolidated model.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function over `PackageDef` with zero recursion and zero panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-FLATTEN-MODEL]
pub fn flatten_package_to_model(
    pkg: deap_core::sysml_ast::PackageDef,
    target_pkg: Option<&str>,
) -> deap_core::sysml_ast::SysmlModel {
    if let Some(target) = target_pkg {
        let mut matched_sub: Option<deap_core::sysml_ast::PackageDef> = None;
        let mut search_stack: Vec<&deap_core::sysml_ast::PackageDef> =
            pkg.packages.iter().rev().collect();
        while let Some(curr) = search_stack.pop() {
            if curr.name == target || curr.name.eq_ignore_ascii_case(target) {
                matched_sub = Some(curr.clone());
                break;
            }
            for child in curr.packages.iter().rev() {
                search_stack.push(child);
            }
        }
        if let Some(sub_pkg) = matched_sub {
            let mut target_flat = flatten_package_tree_iterative(sub_pkg);
            let all_flat = flatten_package_tree_iterative(pkg);
            for req in all_flat.requirement_defs {
                if !target_flat
                    .requirement_defs
                    .iter()
                    .any(|r| r.name == req.name && r.req_id == req.req_id)
                {
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
            return deap_core::sysml_ast::SysmlModel::from(target_flat);
        }
    }
    let flat_root = flatten_package_tree_iterative(pkg);
    deap_core::sysml_ast::SysmlModel::from(flat_root)
}

/// Escapes reserved XML characters (`&`, `<`, `>`, `"`, `'`) into predefined XML entities.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures well-formed XML generation for MATLAB Requirements Toolbox (`.slreqx`) and
/// Stateflow (`.sfx`) artifacts under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure total function over UTF-8 strings: replaces `&` with `&amp;`, `<` with `&lt;`,
///   `>` with `&gt;`, `"` with `&quot;`, and `'` with `&apos;` while preserving all other
///   Unicode scalar values verbatim.
///
/// ## 3. Preconditions
/// - `s` is any valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns an owned `String` containing zero unescaped `<`, `>`, `"`, `'`, or raw `&` characters.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ single linear pass where $N$ is `s.len()`.
/// - Space complexity: $O(N)$ bounded output buffer allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: never panics or fails on any input string.
///
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
/// - Time complexity: $O((R + P) \log(R + P) + S + B)$ where $R$ is requirement count, $P$ is part count,
///   $S$ is state/transition count, and $B$ is port/attribute bus element count.
/// - Space complexity: $O(R + S + P + B)$ for the synthesized output buffers and `BTreeSet` indices.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: handles empty models or missing optional fields gracefully without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-SYNTHESIZE-MBD]
pub fn synthesize_mbd_bundle(model: &deap_core::sysml_ast::SysmlModel) -> MbdExportBundle {
    let pkg_name = model.package_name.as_deref().unwrap_or("SysML_Model");
    let pkg_name_xml = escape_xml(pkg_name);
    let mut all_parts = Vec::new();
    collect_model_parts_recursive(&model.parts, &mut all_parts);

    let mut all_reqs: Vec<RequirementDef> = Vec::with_capacity(model.requirements.len());
    let mut seen_reqs: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for req in &model.requirements {
        if seen_reqs.insert((req.name.clone(), req.req_id.clone())) {
            all_reqs.push(req.clone());
        }
    }
    for part in &all_parts {
        for req in &part.requirements {
            if seen_reqs.insert((req.name.clone(), req.req_id.clone())) {
                all_reqs.push(req.clone());
            }
        }
    }

    let mut req_to_parts: std::collections::BTreeMap<&str, Vec<(usize, &str)>> =
        std::collections::BTreeMap::new();
    for (part_idx, part) in all_parts.iter().enumerate() {
        for sat in &part.satisfied_requirements {
            req_to_parts
                .entry(sat.as_str())
                .or_default()
                .push((part_idx, part.name.as_str()));
        }
    }

    let mut slreq_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    slreq_xml.push_str(&format!(
        "<slreq:RequirementSet xmlns:slreq=\"https://www.mathworks.com/slreq\" package=\"{pkg_name_xml}\">\n"
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
        let mut satisfied_by: Vec<String> = Vec::with_capacity(req.satisfied_by.len());
        let mut satisfied_seen: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for existing in &req.satisfied_by {
            if satisfied_seen.insert(existing.clone()) {
                satisfied_by.push(existing.clone());
            }
        }
        let mut matched_parts: std::collections::BTreeMap<usize, &str> =
            std::collections::BTreeMap::new();
        if let Some(entries) = req_to_parts.get(id_str) {
            for &(idx, p_name) in entries {
                matched_parts.insert(idx, p_name);
            }
        }
        if let Some(entries) = req_to_parts.get(req.name.as_str()) {
            for &(idx, p_name) in entries {
                matched_parts.insert(idx, p_name);
            }
        }
        for p_name in matched_parts.into_values() {
            if satisfied_seen.insert(p_name.to_string()) {
                satisfied_by.push(p_name.to_string());
            }
        }
        let id_xml = escape_xml(id_str);
        let name_xml = escape_xml(&req.name);
        let layer_xml = escape_xml(layer_str);
        let summary_xml = escape_xml(summary_str);
        let satisfied_by_xml = escape_xml(&satisfied_by.join(", "));
        let safety_realises_xml = escape_xml(&req.safety_realises.join(", "));
        let triggers_hazards_xml = escape_xml(&req.triggers_hazards.join(", "));
        slreq_xml.push_str(&format!(
            "  <slreq:Requirement id=\"{id_xml}\" name=\"{name_xml}\" layer=\"{layer_xml}\">\n"
        ));
        slreq_xml.push_str(&format!("    <slreq:Summary>{summary_xml}</slreq:Summary>\n"));
        slreq_xml.push_str(&format!(
            "    <slreq:SatisfiedBy>{satisfied_by_xml}</slreq:SatisfiedBy>\n"
        ));
        slreq_xml.push_str(&format!(
            "    <slreq:SafetyRealises>{safety_realises_xml}</slreq:SafetyRealises>\n"
        ));
        slreq_xml.push_str(&format!(
            "    <slreq:TriggersHazards>{triggers_hazards_xml}</slreq:TriggersHazards>\n"
        ));
        slreq_xml.push_str("  </slreq:Requirement>\n");
    }
    slreq_xml.push_str("</slreq:RequirementSet>\n");

    let mut stateflow_sfx = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    stateflow_sfx.push_str(&format!("<StateflowChart package=\"{pkg_name_xml}\">\n"));
    for part in &all_parts {
        let part_name_xml = escape_xml(&part.name);
        for state_def in &part.states {
            let state_def_name_xml = escape_xml(&state_def.name);
            stateflow_sfx.push_str(&format!(
                "  <Machine name=\"{state_def_name_xml}\" ownerPart=\"{part_name_xml}\">\n"
            ));
            let root_entry = escape_xml(state_def.entry_action.as_deref().unwrap_or(""));
            let root_do = escape_xml(state_def.do_action.as_deref().unwrap_or(""));
            let root_exit = escape_xml(state_def.exit_action.as_deref().unwrap_or(""));
            stateflow_sfx.push_str(&format!(
                "    <State name=\"{state_def_name_xml}\" entry=\"{root_entry}\" do=\"{root_do}\" exit=\"{root_exit}\" />\n"
            ));
            for st in &state_def.sub_states {
                let st_name_xml = escape_xml(&st.name);
                let entry_act = escape_xml(st.entry_action.as_deref().unwrap_or(""));
                let do_act = escape_xml(st.do_action.as_deref().unwrap_or(""));
                let exit_act = escape_xml(st.exit_action.as_deref().unwrap_or(""));
                stateflow_sfx.push_str(&format!(
                    "    <State name=\"{st_name_xml}\" entry=\"{entry_act}\" do=\"{do_act}\" exit=\"{exit_act}\" />\n"
                ));
            }
            for tr in &state_def.transitions {
                let tr_name = escape_xml(tr.name.as_deref().unwrap_or(""));
                let src = escape_xml(tr.source.as_deref().unwrap_or(""));
                let tgt = escape_xml(tr.target.as_deref().unwrap_or(""));
                let trig = escape_xml(tr.trigger.as_deref().unwrap_or(""));
                let guard = escape_xml(tr.guard.as_deref().unwrap_or(""));
                let effect = escape_xml(tr.effect.as_deref().unwrap_or(""));
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
            escape_matlab_single_quoted(&port_def.name),
            escape_matlab_single_quoted(&port_def.type_name),
            escape_matlab_single_quoted(&port_def.direction)
        ));
        for flow in &port_def.item_flows {
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{}', 'DataType', '{}', 'Direction', '{}');\n",
                escape_matlab_single_quoted(&flow.name),
                escape_matlab_single_quoted(&flow.item_type),
                escape_matlab_single_quoted(&flow.direction)
            ));
        }
        for (elec_k, elec_v) in &port_def.electrical_attributes {
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{}', 'DataType', '{}');\n",
                escape_matlab_single_quoted(elec_k),
                escape_matlab_single_quoted(elec_v)
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
                escape_matlab_single_quoted(&attr.name),
                escape_matlab_single_quoted(&attr.type_name)
            ));
        }
        for port in &part.ports {
            let port_ty = if port.type_name.is_empty() {
                "Bus"
            } else {
                port.type_name.as_str()
            };
            simulink_bus_m.push_str(&format!(
                "  Simulink.BusElement('Name', '{}', 'DataType', '{}', 'Direction', '{}');\n",
                escape_matlab_single_quoted(&port.name),
                escape_matlab_single_quoted(port_ty),
                escape_matlab_single_quoted(&port.direction)
            ));
        }
        simulink_bus_m.push_str("];\n");
    }

    let descriptors: Vec<deap_core::sysml_ast::TypeDescriptor> = all_parts
        .iter()
        .map(|part| deap_core::sysml_ast::TypeDescriptor::from_part_def(pkg_name, part))
        .collect();
    let layer_contracts_json = match serde_json::to_string_pretty(&descriptors) {
        Ok(json) => json,
        Err(_) => String::from("[]"),
    };

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
/// - Writes exactly four files atomically in deterministic order: `requirements.slreqx`,
///   `stateflow_chart.sfx`, `simulink_bus.m`, and `layer_contracts.json`.
/// - Propagates all serialization and I/O failures explicitly via `Result<Vec<PathBuf>, String>` with zero silent drops.
///
/// ## 3. Preconditions
/// - `model` is a valid `SysmlModel` reference; `out_dir` is a writable target directory path.
///
/// ## 4. Postconditions
/// - Returns `Ok(Vec<PathBuf>)` containing the 4 written file paths upon success, or `Err(String)`
///   describing any directory creation, serialization, or atomic file write error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ where $N$ is the total byte size of the synthesized MBD bundle.
/// - Space complexity: $O(N)$ in-memory buffer allocation prior to flush.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Filesystem permission, serialization, or I/O errors return descriptive `Err(String)` diagnostics.
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
    let pkg_name = model.package_name.as_deref().unwrap_or("SysML_Model");
    let mut all_parts = Vec::new();
    collect_model_parts_recursive(&model.parts, &mut all_parts);
    let descriptors: Vec<deap_core::sysml_ast::TypeDescriptor> = all_parts
        .iter()
        .map(|part| deap_core::sysml_ast::TypeDescriptor::from_part_def(pkg_name, part))
        .collect();
    let layer_contracts_json = serde_json::to_string_pretty(&descriptors)
        .map_err(|e| format!("Failed to serialize MBD TypeDescriptor contracts to JSON: {e}"))?;

    let mut bundle = synthesize_mbd_bundle(model);
    bundle.layer_contracts_json = layer_contracts_json;

    let slreq_path = out_dir.join("requirements.slreqx");
    crate::semantic::digest::write_atomic(&slreq_path, &bundle.slreq_xml).map_err(|e| {
        format!(
            "Failed to write MBD requirements artifact '{}': {e}",
            slreq_path.display()
        )
    })?;

    let sfx_path = out_dir.join("stateflow_chart.sfx");
    crate::semantic::digest::write_atomic(&sfx_path, &bundle.stateflow_sfx).map_err(|e| {
        format!(
            "Failed to write MBD Stateflow artifact '{}': {e}",
            sfx_path.display()
        )
    })?;

    let bus_path = out_dir.join("simulink_bus.m");
    crate::semantic::digest::write_atomic(&bus_path, &bundle.simulink_bus_m).map_err(|e| {
        format!(
            "Failed to write MBD Simulink bus artifact '{}': {e}",
            bus_path.display()
        )
    })?;

    let contracts_path = out_dir.join("layer_contracts.json");
    crate::semantic::digest::write_atomic(&contracts_path, &bundle.layer_contracts_json).map_err(
        |e| {
            format!(
                "Failed to write MBD layer contracts artifact '{}': {e}",
                contracts_path.display()
            )
        },
    )?;

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
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-ESCAPE-XML]"));

        for (name, file_src) in [
            ("src/cli.rs", include_str!("cli.rs")),
            ("src/driver.rs", include_str!("driver.rs")),
            ("src/main.rs", include_str!("main.rs")),
        ] {
            assert!(
                file_src.contains("//! ## 1. Safety Intent & Regulatory Scope"),
                "{name} must contain module section 1"
            );
            assert!(
                file_src.contains("//! ## 7. Realization & Traceability"),
                "{name} must contain module section 7"
            );
            assert!(
                file_src.contains("/// ## 1. Safety Intent & Regulatory Scope"),
                "{name} must contain item section 1"
            );
            assert!(
                file_src.contains("/// ## 7. Realization & Traceability"),
                "{name} must contain item section 7"
            );
        }

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

    /// Verifies XML entity escaping and structured transition endpoints in `synthesize_mbd_bundle`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates
    /// for well-formed `.slreqx` and `.sfx` XML artifact synthesis in `crates/compile-sysml/src/lib.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - All five XML special characters (`&`, `<`, `>`, `"`, `'`) in requirement and statechart
    ///   fields are deterministically escaped to `&amp;`, `&lt;`, `&gt;`, `&quot;`, and `&apos;`.
    /// - Parsed state machine transitions populate `source` and `target` attributes in `.sfx` XML.
    ///
    /// ## 3. Preconditions
    /// - In-memory `SysmlModel` contains requirements and state transitions with XML reserved characters.
    ///
    /// ## 4. Postconditions
    /// - Asserts `.slreqx` and `.sfx` payloads contain escaped entities and populated transition endpoints.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over model size and synthesized XML string length.
    /// - Space Complexity: O(N) for synthesized `MbdExportBundle`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Deterministic assertions over in-memory strings.
    /// - State: Side-effect free unit test.
    /// - Action: Executes deterministically without filesystem mutation.
    /// - Resource: Bounded memory allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-TEST-DEFECT-06-MBD-XML-ESCAPING]
    #[test]
    fn test_defect_06_synthesize_mbd_bundle_transition_endpoints_and_xml_escaping() -> Result<(), Box<dyn std::error::Error>> {
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
        let mut model = deap_core::sysml_ast::SysmlModel::from(pkg);
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

        assert!(
            bundle.slreq_xml.contains("id=\"REQ-&quot;01&quot;-&apos;A&apos;\""),
            "slreq_xml must escape quotes and apostrophes in id attribute:\n{}",
            bundle.slreq_xml
        );
        assert!(
            bundle.slreq_xml.contains("name=\"Req&lt;Pitch&amp;Roll&gt;\""),
            "slreq_xml must escape <, &, > in name attribute:\n{}",
            bundle.slreq_xml
        );
        assert!(
            bundle.slreq_xml.contains("layer=\"Safety &amp; &quot;Control&quot; &lt;&apos;L1&apos;&gt;\""),
            "slreq_xml must escape all 5 XML entities in layer attribute:\n{}",
            bundle.slreq_xml
        );
        assert!(
            bundle.slreq_xml.contains("<slreq:Summary>Pitch &lt; 15.0 &amp; Roll &gt; -15.0 in &quot;Nominal&quot; &apos;Mode&apos;</slreq:Summary>"),
            "slreq_xml must escape all 5 XML entities in Summary element:\n{}",
            bundle.slreq_xml
        );
        assert!(
            bundle.slreq_xml.contains("<slreq:SatisfiedBy>FlightController&lt;Primary&amp;Backup&gt;</slreq:SatisfiedBy>"),
            "slreq_xml must escape SatisfiedBy element:\n{}",
            bundle.slreq_xml
        );
        assert!(
            bundle.slreq_xml.contains("<slreq:SafetyRealises>SC-01 &lt;&quot;Bound&quot; &amp; &apos;Safe&apos;&gt;</slreq:SafetyRealises>"),
            "slreq_xml must escape SafetyRealises element:\n{}",
            bundle.slreq_xml
        );
        assert!(
            bundle.slreq_xml.contains("<slreq:TriggersHazards>H-01 &lt;&quot;Critical&quot; &amp; &apos;High&apos;&gt;</slreq:TriggersHazards>"),
            "slreq_xml must escape TriggersHazards element:\n{}",
            bundle.slreq_xml
        );

        assert!(
            bundle.stateflow_sfx.contains("source=\"Init\" target=\"SafeMode\""),
            "stateflow_sfx must populate structured source and target for first..then transition:\n{}",
            bundle.stateflow_sfx
        );
        assert!(
            bundle.stateflow_sfx.contains("source=\"SafeMode\" target=\"Init\""),
            "stateflow_sfx must populate structured source and target for from..to transition:\n{}",
            bundle.stateflow_sfx
        );
        assert!(
            bundle.stateflow_sfx.contains("trigger=\"FaultSignal\" guard=\"pitch_deg &gt; 15.0 &amp;&amp; roll_deg &lt; -10.0\" effect=\"EngageBackup\""),
            "stateflow_sfx must escape relational and logical operators in guard attribute:\n{}",
            bundle.stateflow_sfx
        );
        assert!(
            bundle.stateflow_sfx.contains("entry=\"&quot;Init &lt; &apos;Safe&apos; &amp; Fast &gt;&quot;\""),
            "stateflow_sfx must escape all 5 XML entities in state entry attribute:\n{}",
            bundle.stateflow_sfx
        );

        Ok(())
    }

    /// Verifies that `flatten_package_to_model` flattens 3-level nested package hierarchies (`Root -> SubA -> SubB -> SubC`)
    /// at depth $d \ge 2$ using an iterative stack both when `target_pkg` is `None` and when `target_pkg` selects `SubA`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates
    /// for deep package hierarchy flattening (`ARCH-03`) in `crates/compile-sysml/src/lib.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves `part_defs`, `port_defs`, `item_defs`, and `requirement_defs` declared at depth $d \ge 2$ (`SubB`, `SubC`).
    /// - Preserves cross-package `requirement_defs`, `port_defs`, and `item_defs` when `target_pkg` selects `SubA`.
    ///
    /// ## 3. Preconditions
    /// - A 4-package nested tree (`Root -> SubA -> SubB -> SubC`) is constructed in memory.
    ///
    /// ## 4. Postconditions
    /// - Asserts all nested and cross-package definitions are present in the returned `SysmlModel`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(P + E) over packages and AST elements.
    /// - Space Complexity: O(P + E) for the flattened model.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Pure in-memory AST verification.
    /// - State: Stateless deterministic test.
    /// - Action: Executes with zero recursion.
    /// - Resource: Bounded memory allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-TEST-ARCH-03-FLATTEN-DEEP]
    #[test]
    fn test_arch_03_flatten_package_to_model_deep_hierarchy() -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            package Root {
                port def RootPort;
                item def RootItem {
                    attribute version : Integer;
                }
                requirement def RootReq {
                    id = "REQ-ROOT-01";
                    text = "Root level requirement.";
                }
                part def RootPart;

                package SubA {
                    part def PartA;
                    port def PortA;

                    package SubB {
                        part def PartB;
                        requirement def ReqB {
                            id = "REQ-B-01";
                            text = "Depth 2 requirement.";
                        }

                        package SubC {
                            part def PartC {
                                item def PartCInnerItem;
                            }
                            port def PortC;
                            item def ItemC {
                                attribute payload : Real;
                            }
                            requirement def ReqC {
                                id = "REQ-C-01";
                                text = "Depth 3 requirement.";
                            }
                        }
                    }
                }
            }
        "#;
        let pkg = parse_sysml(src)?;

        // Case 1: target_pkg is None -> flattens Root, SubA, SubB (d=2), and SubC (d=3)
        let model_all = flatten_package_to_model(pkg.clone(), None);
        assert_eq!(model_all.package_name.as_deref(), Some("Root"));
        assert!(model_all.parts.iter().any(|p| p.name == "RootPart"));
        assert!(model_all.parts.iter().any(|p| p.name == "PartA"));
        assert!(
            model_all.parts.iter().any(|p| p.name == "PartB"),
            "Depth-2 PartB must be flattened when target_pkg is None"
        );
        assert!(
            model_all.parts.iter().any(|p| p.name == "PartC"),
            "Depth-3 PartC must be flattened when target_pkg is None"
        );
        assert!(model_all.ports.iter().any(|p| p.name == "RootPort"));
        assert!(model_all.ports.iter().any(|p| p.name == "PortA"));
        assert!(
            model_all.ports.iter().any(|p| p.name == "PortC"),
            "Depth-3 PortC must be flattened when target_pkg is None"
        );
        assert!(model_all.requirements.iter().any(|r| r.name == "RootReq"));
        assert!(
            model_all.requirements.iter().any(|r| r.name == "ReqB"),
            "Depth-2 ReqB must be flattened when target_pkg is None"
        );
        assert!(
            model_all.requirements.iter().any(|r| r.name == "ReqC"),
            "Depth-3 ReqC must be flattened when target_pkg is None"
        );
        assert!(
            model_all
                .parts
                .iter()
                .flat_map(|p| &p.item_defs)
                .any(|i| i.name == "ItemC"),
            "Depth-3 ItemC must be preserved when target_pkg is None"
        );
        assert!(
            model_all
                .parts
                .iter()
                .flat_map(|p| &p.item_defs)
                .any(|i| i.name == "RootItem"),
            "RootItem must be preserved when target_pkg is None"
        );

        // Case 2: target_pkg selects SubA -> flattens SubA->SubB->SubC and preserves cross-package reqs, ports, items
        let model_suba = flatten_package_to_model(pkg, Some("SubA"));
        assert_eq!(model_suba.package_name.as_deref(), Some("SubA"));
        assert!(!model_suba.parts.iter().any(|p| p.name == "RootPart"));
        assert!(model_suba.parts.iter().any(|p| p.name == "PartA"));
        assert!(
            model_suba.parts.iter().any(|p| p.name == "PartB"),
            "Depth-2 PartB under SubA must be flattened when target_pkg selects SubA"
        );
        assert!(
            model_suba.parts.iter().any(|p| p.name == "PartC"),
            "Depth-3 PartC under SubA must be flattened when target_pkg selects SubA"
        );
        assert!(
            model_suba.ports.iter().any(|p| p.name == "PortC"),
            "Depth-3 PortC under SubA must be flattened when target_pkg selects SubA"
        );
        assert!(
            model_suba.ports.iter().any(|p| p.name == "RootPort"),
            "Cross-package RootPort must be preserved when target_pkg selects SubA"
        );
        assert!(
            model_suba.requirements.iter().any(|r| r.name == "ReqB"),
            "Depth-2 ReqB under SubA must be flattened when target_pkg selects SubA"
        );
        assert!(
            model_suba.requirements.iter().any(|r| r.name == "ReqC"),
            "Depth-3 ReqC under SubA must be flattened when target_pkg selects SubA"
        );
        assert!(
            model_suba.requirements.iter().any(|r| r.name == "RootReq"),
            "Cross-package RootReq must be preserved when target_pkg selects SubA"
        );
        assert!(
            model_suba
                .parts
                .iter()
                .flat_map(|p| &p.item_defs)
                .any(|i| i.name == "ItemC"),
            "Depth-3 ItemC under SubA must be preserved when target_pkg selects SubA"
        );
        assert!(
            model_suba
                .parts
                .iter()
                .flat_map(|p| &p.item_defs)
                .any(|i| i.name == "RootItem"),
            "Cross-package RootItem must be preserved when target_pkg selects SubA"
        );

        Ok(())
    }

    /// Verifies that `escape_matlab_single_quoted` doubles single quotes and `synthesize_mbd_bundle`
    /// properly escapes single quotes in MATLAB `.m` bus literals (`ARCH-06`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates
    /// for MATLAB codegen escaping (`ARCH-06`) in `crates/compile-sysml/src/lib.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `escape_matlab_single_quoted("Controller's_State")` equals `"Controller''s_State"`.
    /// - `synthesize_mbd_bundle` escapes single quotes across port, attribute, and flow literals in `simulink_bus_m`.
    ///
    /// ## 3. Preconditions
    /// - In-memory `SysmlModel` contains attributes and ports with single quotes.
    ///
    /// ## 4. Postconditions
    /// - Asserts `simulink_bus_m` contains doubled single quotes inside MATLAB string literals.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over input string length.
    /// - Space Complexity: O(N) for synthesized output.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Pure in-memory string assertions.
    /// - State: Side-effect free unit test.
    /// - Action: Deterministic execution.
    /// - Resource: Bounded memory allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-TEST-ARCH-06-MATLAB-ESCAPING]
    #[test]
    fn test_arch_06_escape_matlab_single_quoted_and_mbd_bus_literals() {
        assert_eq!(
            escape_matlab_single_quoted("Controller's_State"),
            "Controller''s_State"
        );

        let mut model = deap_core::sysml_ast::SysmlModel::default();
        model.package_name = Some("CtrlPkg".to_string());
        let mut part = PartDef::default();
        part.name = "PrimaryController".to_string();
        part.attributes.push(AttributeDef {
            name: "Controller's_State".to_string(),
            type_name: "Real's_Type".to_string(),
            default_value: None,
            doc: None,
        });
        model.parts.push(part);

        let bundle = synthesize_mbd_bundle(&model);
        assert!(
            bundle
                .simulink_bus_m
                .contains("Simulink.BusElement('Name', 'Controller''s_State', 'DataType', 'Real''s_Type');"),
            "simulink_bus_m must escape single quotes in MATLAB string literals:\n{}",
            bundle.simulink_bus_m
        );
    }
}


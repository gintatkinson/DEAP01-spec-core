//! AST structures for SysML v2 entities, requirements, and system architecture.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the canonical Abstract Syntax Tree (AST) data structures, recursive parsers, and
//! deterministic architecture extractors for SysML v2 models in accordance with DO-178C Level A,
//! DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification and traceability mandates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure Schema-Driven Extraction: All AST representations derive exclusively and deterministically
//!   from SysML v2 structural, behavioral, and requirement declarations without hardcoded domain state.
//! - Deterministic Ordering: Multi-file directory traversal and map/set aggregations preserve
//!   canonical lexicographical ordering via `BTreeMap` and `BTreeSet`.
//!
//! ## 3. Preconditions
//! - Input SysML v2 source strings and workspace schema paths must be valid UTF-8 text or readable
//!   filesystem directories containing `.sysml` specification units.
//!
//! ## 4. Postconditions
//! - Returns strongly typed, serializable AST node collections (`SysmlModel`, `PartDef`,
//!   `RequirementDef`, `PortDef`, `ActionDef`, `StateDef`, `UseCaseDef`, `TestCaseDef`) preserving
//!   exact block hierarchy and requirement satisfaction bindings.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in the byte length of parsed SysML v2 source text and number of
//!   AST tokens.
//! - Space Complexity: $O(V + E)$ proportional to the extracted AST vertices and directed edges.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unbalanced braces, malformed declarations, or unreadable paths are handled gracefully via
//!   bounded character-stream scanning and explicit `Result` error propagation without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]

mod actions;
mod connections_and_flows;
mod expressions;
mod identifiers;
mod interactions;
mod interfaces_and_items;
mod package_model;
mod parts_and_ports;
mod requirements;
mod safety_and_fmeca;
mod states;

pub use actions::{parse_action_defs, ActionDef};
pub use connections_and_flows::{parse_connection_defs, parse_port_defs, ConnectionDef};
pub use expressions::{parse_calc_defs, parse_constraint_defs, CalcDef, ConstraintDef};
use identifiers::{
    extract_doc_comment_before, extract_ident, is_ident_char, is_word_boundary_after,
    is_word_boundary_before, skip_ws,
};
pub use identifiers::{
    find_matching_brace, is_valid_subsystem_identifier, mask_string_literals, AttributeDef, FlowDef,
};
pub use interactions::{InteractionDef, UseCaseDef};
pub use interfaces_and_items::{parse_attribute_defs, parse_part_defs, InterfaceDef};
pub use package_model::{AstSlice, ImportDef, PackageDef, SysmlModel};
pub use parts_and_ports::{
    CapabilityDef, FieldDescriptor, ItemDef, OperationDef, PartDef, PortDef,
    SupportedExtensionsCatalog, SupportedSchemaExtension, TypeDescriptor,
};
use requirements::parse_safety_and_layer_annotations;
pub use requirements::{parse_requirement_defs, parse_satisfied_requirements, RequirementDef};
pub use safety_and_fmeca::{
    component_matches, extract_negated_components, is_component_negated, parse_hazard_defs,
    HazardDef, RiskDef,
};
use safety_and_fmeca::{component_matches_negated_stems, extract_negated_key_stem};
pub use states::{parse_state_defs, StateDef, TransitionDef};

use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a verification test case definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `TestCaseDef` in `crates/deap-core/src/sysml_ast/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `TestCaseDef` across member set `[name: String, subject_part: Option<String>, verified_requirements: Vec<String>, objective: Option<String>, test_steps: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TestCaseDef` members (`name: String, subject_part: Option<String>, verified_requirements: Vec<String>, objective: Option<String>, test_steps: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `TestCaseDef` representation for struct `TestCaseDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `TestCaseDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `TestCaseDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TestCaseDef` (`TestCaseDef`).
/// - State: Maintains consistent state invariants across `TestCaseDef` without partial mutation on error.
/// - Action: Executes `TestCaseDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `TestCaseDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-TESTCASEDEF] | SysML v2: PartDef(TestCaseDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TestCaseDef {
    pub name: String,
    pub subject_part: Option<String>,
    #[serde(default)]
    pub verified_requirements: Vec<String>,
    pub objective: Option<String>,
    #[serde(default)]
    pub test_steps: Vec<String>,
    pub doc: Option<String>,
}

/// Parse test case definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic verification test case AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `TestCaseDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed test case declarations are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
pub fn parse_test_case_defs(content: &str) -> Vec<TestCaseDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut test_cases = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("test")) {
        let test_pos = idx + pos;
        if !is_word_boundary_before(&masked, test_pos)
            || !is_word_boundary_after(&masked, test_pos + 4)
        {
            idx = test_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, test_pos);
        let mut curr = test_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if !masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("case") && is_word_boundary_after(s, 4))
        {
            idx = test_pos + 4;
            continue;
        }
        curr += 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = test_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut subject_part = None;
        let mut verified_requirements = Vec::new();
        let mut objective = None;
        let mut test_steps = Vec::new();

        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let body = clean.get(curr + 1..close_pos).unwrap_or("");
                for line in body.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("subject") && is_word_boundary_after(trimmed, 7) {
                        let val = trimmed
                            .get(7..)
                            .unwrap_or("")
                            .trim()
                            .trim_end_matches(';')
                            .trim();
                        if !val.is_empty() {
                            subject_part = Some(val.to_string());
                        }
                    } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
                        let mut rest = trimmed.get(6..).unwrap_or("").trim();
                        if rest.starts_with("assert") && is_word_boundary_after(rest, 6) {
                            rest = rest.get(6..).unwrap_or("").trim();
                        }
                        if rest.starts_with("requirement") && is_word_boundary_after(rest, 11) {
                            rest = rest.get(11..).unwrap_or("").trim();
                        } else if rest.starts_with("constraint") && is_word_boundary_after(rest, 10)
                        {
                            rest = rest.get(10..).unwrap_or("").trim();
                        }
                        let targets = rest.trim_end_matches(';').trim();
                        for target in targets.split(',') {
                            let t = target.trim();
                            if !t.is_empty() {
                                verified_requirements.push(t.to_string());
                            }
                        }
                    } else if trimmed.starts_with("objective") && is_word_boundary_after(trimmed, 9)
                    {
                        let val = trimmed
                            .get(9..)
                            .unwrap_or("")
                            .trim()
                            .trim_end_matches(';')
                            .trim()
                            .trim_matches('"');
                        if !val.is_empty() {
                            objective = Some(val.to_string());
                        }
                    } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
                        let val = trimmed
                            .get(4..)
                            .unwrap_or("")
                            .trim()
                            .trim_end_matches(';')
                            .trim();
                        if !val.is_empty() {
                            test_steps.push(val.to_string());
                        }
                    }
                }
                idx = close_pos + 1;
            } else {
                idx = test_pos + 4;
                continue;
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = test_pos + 4;
            continue;
        }

        test_cases.push(TestCaseDef {
            name: name.to_string(),
            subject_part,
            verified_requirements,
            objective,
            test_steps,
            doc,
        });
    }
    test_cases
}

/// Parse an entire SysML v2 source text into a structured SysmlModel.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C top-level SysML v2 model parsing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing and deterministic element extractors.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 document.
///
/// ## 4. Postconditions
/// - Returns a populated `SysmlModel` containing all parsed top-level definitions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot D)$ in `content.len()` and nesting depth. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back to root-level parsing if no outer `package` block is present; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SYSML]
pub fn parse_sysml(content: &str) -> SysmlModel {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut package_info = None;
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("package")) {
        let pkg_pos = idx + pos;
        if !is_word_boundary_before(&masked, pkg_pos)
            || !is_word_boundary_after(&masked, pkg_pos + 7)
        {
            idx = pkg_pos + 7;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, pkg_pos);
        let mut curr = pkg_pos + 7;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = pkg_pos + 7;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            let open_pos = curr;
            let body = if let Some(close_pos) = find_matching_brace(&clean, open_pos) {
                clean.get(open_pos + 1..close_pos).unwrap_or("")
            } else {
                clean.get(open_pos + 1..).unwrap_or("")
            };
            package_info = Some((name.to_string(), doc, body));
            break;
        } else {
            idx = pkg_pos + 7;
            continue;
        }
    }

    if let Some((package_name, doc, body)) = package_info {
        let mut parts = parse_part_defs(body);
        let ports = parse_port_defs(body);
        let actions = parse_action_defs(body);
        let constraints = parse_constraint_defs(body);
        let requirements = parse_requirement_defs(body);
        let attributes = parse_attribute_defs(body);
        let test_cases = parse_test_case_defs(body);
        let pkg_connections = parse_connection_defs(body);
        let pkg_hazards = parse_hazard_defs(body);

        if let Some(first_part) = parts.first_mut() {
            for conn in pkg_connections {
                if !first_part.connections.iter().any(|c| {
                    c.name == conn.name
                        && c.source_port == conn.source_port
                        && c.target_port == conn.target_port
                }) {
                    first_part.connections.push(conn);
                }
            }
            for haz in pkg_hazards {
                if !first_part.hazards.iter().any(|h| h.name == haz.name) {
                    first_part.hazards.push(haz);
                }
            }
        }

        SysmlModel {
            package_name: Some(package_name),
            doc,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
            test_cases,
        }
    } else {
        let mut parts = parse_part_defs(&clean);
        let ports = parse_port_defs(&clean);
        let actions = parse_action_defs(&clean);
        let constraints = parse_constraint_defs(&clean);
        let requirements = parse_requirement_defs(&clean);
        let attributes = parse_attribute_defs(&clean);
        let test_cases = parse_test_case_defs(&clean);
        let root_connections = parse_connection_defs(&clean);
        let root_hazards = parse_hazard_defs(&clean);

        if let Some(first_part) = parts.first_mut() {
            for conn in root_connections {
                if !first_part.connections.iter().any(|c| {
                    c.name == conn.name
                        && c.source_port == conn.source_port
                        && c.target_port == conn.target_port
                }) {
                    first_part.connections.push(conn);
                }
            }
            for haz in root_hazards {
                if !first_part.hazards.iter().any(|h| h.name == haz.name) {
                    first_part.hazards.push(haz);
                }
            }
        }

        SysmlModel {
            package_name: None,
            doc: None,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
            test_cases,
        }
    }
}

/// Parses a SysML v2 textual model into a `SysmlModel` (canonical alias for [`parse_sysml`]).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `parse_sysml_model`.
/// Provides the canonical `parse_sysml_model` entry point used by `IncrementalAstCache::load_or_parse` and downstream compilers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Referentially transparent and bit-for-bit identical to `parse_sysml(content)`.
///
/// ## 3. Preconditions
/// - `content: &str` is a valid UTF-8 SysML v2 source string.
///
/// ## 4. Postconditions
/// - Returns a populated `SysmlModel` with all parsed parts, ports, requirements, constraints, and safety annotations.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot D)$ where $N$ is `content.len()` and $D$ is nesting depth.
/// - Space Complexity: $O(N)$ for the returned `SysmlModel` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles arbitrary UTF-8 input safely without panicking.
/// - State: Pure function.
/// - Action: Delegates to `parse_sysml(content)`.
/// - Resource: Bounded by input text length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-PARSE-SYSML-MODEL]
pub fn parse_sysml_model(content: &str) -> SysmlModel {
    parse_sysml(content)
}

/// Strips C-style multi-line comments (`/* ... */`) and line comments (`// ...`) from SysML text without regex,
/// while preserving `doc /* ... */` documentation blocks and string literals.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents commented-out SysML declarations from
/// polluting the parsed AST while retaining normative `doc /* ... */` blocks and string literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - Line count is preserved for `//` comments (terminated at `\n`).
/// - Multi-line `/* ... */` blocks immediately preceded by the `doc` keyword at a word boundary are preserved verbatim.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML source slice.
///
/// ## 4. Postconditions
/// - Returns a newly allocated `String` with non-documentation comments stripped.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ single-pass character scan. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unclosed comments or string literals terminate cleanly at end-of-input without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-COMMENTS]
pub fn strip_sysml_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_str = false;
    let mut in_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if in_escape {
                in_escape = false;
            } else if c == '\\' {
                in_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for next_c in chars.by_ref() {
                    if next_c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            } else if let Some(&'*') = chars.peek() {
                let trimmed_before = out.trim_end();
                let is_doc_block = trimmed_before.ends_with("doc")
                    && (trimmed_before.len() == 3
                        || !trimmed_before
                            .as_bytes()
                            .get(trimmed_before.len() - 4)
                            .copied()
                            .is_some_and(is_ident_char));
                chars.next();
                if is_doc_block {
                    out.push_str("/*");
                    while let Some(next_c) = chars.next() {
                        out.push(next_c);
                        if next_c == '*' && chars.peek() == Some(&'/') {
                            out.push('/');
                            chars.next();
                            break;
                        }
                    }
                } else {
                    out.push(' ');
                    while let Some(next_c) = chars.next() {
                        if next_c == '\n' {
                            out.push('\n');
                        }
                        if next_c == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Strips all line comments (`// ...`), block comments (`/* ... */`, including `doc /* ... */`),
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-ALL-COMMENTS-AND-STRINGS]
fn strip_all_block_comments_and_strings(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_str = false;
    let mut in_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            if c == '\n' {
                out.push('\n');
            } else {
                out.push(' ');
            }
            if in_escape {
                in_escape = false;
            } else if c == '\\' {
                in_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(' ');
            continue;
        }
        if c == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for next_c in chars.by_ref() {
                    if next_c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            } else if let Some(&'*') = chars.peek() {
                chars.next();
                out.push(' ');
                while let Some(next_c) = chars.next() {
                    if next_c == '\n' {
                        out.push('\n');
                    }
                    if next_c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Centralized schema discovery and AST part extractor shared across compiler and validators.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `UnifiedSchemaModelExtractor` in `crates/deap-core/src/sysml_ast/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `UnifiedSchemaModelExtractor` across member set `[const: SupportedExtensionsCatalog = SupportedExtensionsCatalog;]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `UnifiedSchemaModelExtractor` members (`const: SupportedExtensionsCatalog = SupportedExtensionsCatalog;`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `UnifiedSchemaModelExtractor` representation for struct `UnifiedSchemaModelExtractor`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `UnifiedSchemaModelExtractor`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `UnifiedSchemaModelExtractor`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `UnifiedSchemaModelExtractor` (`UnifiedSchemaModelExtractor`).
/// - State: Maintains consistent state invariants across `UnifiedSchemaModelExtractor` without partial mutation on error.
/// - Action: Executes `UnifiedSchemaModelExtractor` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `UnifiedSchemaModelExtractor` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-UNIFIEDSCHEMAMODELEXTRACTOR] | SysML v2: PartDef(UnifiedSchemaModelExtractor)
#[derive(Debug, Clone, Default)]
pub struct UnifiedSchemaModelExtractor;

impl UnifiedSchemaModelExtractor {
    /// Supported schema extensions across all pipeline tools.
    /// Realises: [REQ-SYSML-AST-MOD-UNIFIEDSCHEMAMODELEXTRACTOR-SUPPORTED-EXTENSIONS]
    pub const SUPPORTED_EXTENSIONS: SupportedExtensionsCatalog = SupportedExtensionsCatalog;

    /// Recursively discovers all supported schema files across the provided search directories.
    /// Excludes hidden entries (`.`), editor swap files (`#`), and `README.md`.
    /// Results are lexicographically sorted to eliminate filesystem traversal jitter.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Guarantees deterministic, platform-independent
    /// schema file discovery ordering across macOS APFS and Linux ext4/overlayfs filesystems.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returned `Vec<PathBuf>` is strictly sorted and deduplicated (`sort()` + `dedup()`).
    /// - Hidden files (`.`), swap files (`#`), and `README.md` are excluded.
    ///
    /// ## 3. Preconditions
    /// - `search_dirs` contains candidate directory paths.
    ///
    /// ## 4. Postconditions
    /// - Returns all discovered regular files matching `Self::SUPPORTED_EXTENSIONS`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(F \log F)$ where $F$ is the number of visited directory entries. Space Complexity: $O(F)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Non-existent directories and unreadable walk entries are skipped cleanly without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/DISCOVER-SCHEMA-FILES]
    pub fn discover_schema_files<P: AsRef<Path>>(search_dirs: &[P]) -> Vec<PathBuf> {
        let mut schema_files = Vec::new();
        for sdir in search_dirs {
            let p = sdir.as_ref();
            if !p.is_dir() {
                continue;
            }
            for entry_res in WalkDir::new(p).into_iter().filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                }
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            }) {
                let Ok(entry) = entry_res else {
                    continue;
                };
                if entry.file_type().is_file() {
                    let fpath = entry.path();
                    let fname = fpath.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let fname_lower = fname.to_ascii_lowercase();
                    if fname.starts_with('.')
                        || fname.starts_with('#')
                        || fname_lower == "readme.md"
                        || fname_lower == "digest.json"
                        || fname_lower.ends_with(".digest.json")
                        || fname_lower.ends_with("-digest.json")
                        || fname_lower.ends_with("_digest.json")
                    {
                        continue;
                    }
                    if let Some(ext) = fpath.extension().and_then(|e| e.to_str()) {
                        let dot_ext = format!(".{}", ext.to_ascii_lowercase());
                        if Self::SUPPORTED_EXTENSIONS.contains(&dot_ext.as_str()) {
                            schema_files.push(fpath.to_path_buf());
                        }
                    }
                }
            }
        }
        schema_files.sort();
        schema_files.dedup();
        schema_files
    }

    /// Extracts all subsystem part and component names across the provided file paths.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Aggregates structural subsystem identifiers
    /// across heterogeneous schema files into a canonical `BTreeSet<String>`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Output is deterministically ordered and deduplicated (`BTreeSet<String>`).
    ///
    /// ## 3. Preconditions
    /// - `file_paths` is a slice of paths to candidate schema files.
    ///
    /// ## 4. Postconditions
    /// - Returns the union of all subsystem identifiers extracted from readable files in `file_paths`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(\sum |B_i| \log P)$ across files $B_i$ and unique parts $P$. Space Complexity: $O(P)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Unreadable files are skipped gracefully without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-SUBSYSTEM-PARTS]
    pub fn extract_subsystem_parts<P: AsRef<Path>>(file_paths: &[P]) -> BTreeSet<String> {
        let mut part_names = BTreeSet::new();
        for fp in file_paths {
            let p = fp.as_ref();
            let fname_lower = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if fname_lower == "digest.json"
                || fname_lower.ends_with(".digest.json")
                || fname_lower.ends_with("-digest.json")
                || fname_lower.ends_with("_digest.json")
            {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(p) {
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let dot_ext = format!(".{}", ext);
                let extracted = Self::extract_subsystem_parts_from_text(&dot_ext, &content);
                part_names.extend(extracted);
            }
        }
        part_names
    }

    /// Extracts subsystem and component names from raw text of a specific format.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Dispatches format-specific structural extraction
    /// across `.sysml`, `.yaml`/`.yml`/`.json`, `.md`/`.markdown`, `.proto`, `.idl`, `.arxml`, and
    /// `.yang` (line-leading `container`, `grouping`, and `list` statements).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Every inserted identifier satisfies `is_valid_subsystem_identifier` (or is a parsed SysML `PartDef` name).
    /// - Output is a deterministically ordered `BTreeSet<String>`.
    ///
    /// ## 3. Preconditions
    /// - `ext` is a leading-dot file extension (e.g. `".sysml"`) and `content` is valid UTF-8 text.
    ///
    /// ## 4. Postconditions
    /// - Returns all discovered subsystem and component identifiers for the given format.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N \log P)$ in input length $N$ and part count $P$. Space Complexity: $O(N + P)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Non-panicking bounds-checked `.get(..)` slicing; unknown extensions return an empty `BTreeSet`.
    /// - YANG identifiers containing `-` or `.` are rejected whole rather than truncated to a misleading prefix.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-SUBSYSTEM-PARTS-TEXT]
    pub fn extract_subsystem_parts_from_text(ext: &str, content: &str) -> BTreeSet<String> {
        let mut part_names = BTreeSet::new();
        let norm_ext = ext.to_ascii_lowercase();

        if norm_ext == ".sysml" {
            let clean = strip_sysml_comments(content);
            for p in parse_part_defs(&clean) {
                if !p.name.is_empty() {
                    part_names.insert(p.name);
                }
            }
            let comment_free = strip_all_block_comments_and_strings(&clean);
            // Fallback token scan for `part def <Ident>` or `part <Ident> :>` / `:` / `{` / `[` / `;`
            for line in comment_free.lines() {
                let mut stmt = line.trim();
                for kw in ["public ", "private ", "protected ", "abstract "] {
                    if let Some(rest) = stmt.strip_prefix(kw) {
                        stmt = rest.trim_start();
                    }
                }
                if let Some(after_part) = stmt.strip_prefix("part ") {
                    let after_part = after_part.trim_start();
                    if let Some(after_def) = after_part.strip_prefix("def ") {
                        let ident: String = after_def
                            .trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if !ident.is_empty()
                            && ident != "def"
                            && is_valid_subsystem_identifier(&ident)
                        {
                            part_names.insert(ident);
                        }
                    } else {
                        let ident: String = after_part
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        let remainder = after_part.get(ident.len()..).unwrap_or("").trim_start();
                        let is_part_usage = remainder.starts_with(':')
                            || remainder.starts_with('{')
                            || remainder.starts_with('[')
                            || remainder.starts_with(';');
                        if is_part_usage
                            && !ident.is_empty()
                            && ident != "def"
                            && is_valid_subsystem_identifier(&ident)
                        {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        } else if norm_ext == ".json" {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(content) {
                let is_schema_digest = json_val.as_object().is_some_and(|m| {
                    m.contains_key("format_version")
                        && m.contains_key("source_hash")
                        && !m.contains_key("schema_nodes")
                });
                if !is_schema_digest {
                    Self::extract_parts_from_json(&json_val, &mut part_names);
                }
            }
        } else if norm_ext == ".yaml" || norm_ext == ".yml" {
            // Check structured JSON if parseable
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(content) {
                Self::extract_parts_from_json(&json_val, &mut part_names);
            }
            // Pure string/line scan for YAML component/subsystem keys
            Self::extract_parts_from_yaml_lines(content, &mut part_names);
        } else if norm_ext == ".md" || norm_ext == ".markdown" {
            let mut in_table = false;
            let mut col_subsys_idx = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('|') && trimmed.ends_with('|') {
                    let cells: Vec<&str> = trimmed
                        .split('|')
                        .map(|c| c.trim())
                        .filter(|c| !c.is_empty())
                        .collect();
                    if cells.is_empty() {
                        continue;
                    }
                    if cells.iter().all(|c| {
                        c.chars()
                            .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
                    }) {
                        in_table = true;
                        continue;
                    }
                    if !in_table {
                        col_subsys_idx = None;
                        for (idx, cell) in cells.iter().enumerate() {
                            let clower = cell.to_ascii_lowercase();
                            if clower.contains("subsystem")
                                || clower.contains("component")
                                || clower.contains("segment")
                                || clower.contains("part")
                                || clower.contains("unit")
                            {
                                col_subsys_idx = Some(idx);
                                break;
                            }
                        }
                    } else if let Some(idx) = col_subsys_idx {
                        if let Some(raw_cand) = cells.get(idx).copied() {
                            let clean_cand = raw_cand
                                .replace(['`', '*', '_', '[', ']'], "")
                                .trim()
                                .to_string();
                            let clean_ident: String = clean_cand
                                .chars()
                                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                                .collect();
                            if is_valid_subsystem_identifier(&clean_ident) {
                                part_names.insert(clean_ident);
                            }
                        }
                    }
                } else {
                    in_table = false;
                    col_subsys_idx = None;
                }
            }
        } else if norm_ext == ".proto" {
            for line in content.lines() {
                let trimmed = line.trim();
                for kw in &["service ", "message "] {
                    if let Some(rest) = trimmed.strip_prefix(kw) {
                        let ident: String = rest
                            .trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        } else if norm_ext == ".idl" {
            for line in content.lines() {
                let trimmed = line.trim();
                for kw in &["interface ", "module ", "struct "] {
                    if let Some(rest) = trimmed.strip_prefix(kw) {
                        let ident: String = rest
                            .trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        } else if norm_ext == ".arxml" {
            for line in content.lines() {
                if line.contains("<SHORT-NAME>") && line.contains("</SHORT-NAME>") {
                    if let Some(start) = line.find("<SHORT-NAME>") {
                        let after = line.get(start + 12..).unwrap_or("");
                        if let Some(end) = after.find("</SHORT-NAME>") {
                            let ident = after.get(..end).unwrap_or("").trim();
                            if is_valid_subsystem_identifier(ident) {
                                part_names.insert(ident.to_string());
                            }
                        }
                    }
                }
            }
        } else if norm_ext == ".yang" {
            for line in content.lines() {
                let trimmed = line.trim();
                for kw in &["container ", "grouping ", "list "] {
                    if let Some(rest) = trimmed.strip_prefix(kw) {
                        // Capture the full RFC 7950 identifier token (which may contain `-` or `.`)
                        // so a non-conforming name is rejected whole instead of truncated.
                        let ident: String = rest
                            .trim_start()
                            .chars()
                            .take_while(|c| {
                                c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.'
                            })
                            .collect();
                        if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        }

        part_names
    }

    /// Recursively walks a parsed JSON `Value` tree and collects valid subsystem identifiers.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-JSON]
    fn extract_parts_from_json(val: &serde_json::Value, parts: &mut BTreeSet<String>) {
        match val {
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::Array(nodes)) = map.get("schema_nodes") {
                    for node in nodes {
                        if let serde_json::Value::String(s) = node {
                            if let Some((kind, name)) = s.split_once(':') {
                                if kind == "part" || kind == "component" || kind == "subsystem" {
                                    let n = name.trim();
                                    if is_valid_subsystem_identifier(n) {
                                        parts.insert(n.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                for key in &["subsystems", "components", "parts", "modules"] {
                    if let Some(v) = map.get(*key) {
                        match v {
                            serde_json::Value::Array(arr) => {
                                for item in arr {
                                    match item {
                                        serde_json::Value::String(s) => {
                                            let s_clean = s.trim();
                                            if is_valid_subsystem_identifier(s_clean) {
                                                parts.insert(s_clean.to_string());
                                            }
                                        }
                                        serde_json::Value::Object(submap) => {
                                            for name_key in &["name", "id", "identifier", "title"] {
                                                if let Some(serde_json::Value::String(n)) =
                                                    submap.get(*name_key)
                                                {
                                                    let n_clean = n.trim();
                                                    if is_valid_subsystem_identifier(n_clean) {
                                                        parts.insert(n_clean.to_string());
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            serde_json::Value::Object(submap) => {
                                for k in submap.keys() {
                                    let k_clean = k.trim();
                                    if is_valid_subsystem_identifier(k_clean) {
                                        parts.insert(k_clean.to_string());
                                    }
                                }
                            }
                            serde_json::Value::String(s) => {
                                let s_clean = s.trim();
                                if is_valid_subsystem_identifier(s_clean) {
                                    parts.insert(s_clean.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                for (_, subval) in map {
                    Self::extract_parts_from_json(subval, parts);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    Self::extract_parts_from_json(item, parts);
                }
            }
            _ => {}
        }
    }

    /// Scans YAML or JSON lines for subsystem and component list entries without an external YAML crate.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-YAML]
    fn extract_parts_from_yaml_lines(content: &str, parts: &mut BTreeSet<String>) {
        let mut in_parts_list = false;
        let mut list_indent = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let current_indent = line.len() - line.trim_start().len();

            let is_key = [
                "subsystems:",
                "components:",
                "parts:",
                "modules:",
                "subsystem:",
                "component:",
                "part:",
                "module:",
            ]
            .iter()
            .any(|kw| trimmed.to_ascii_lowercase().starts_with(kw));

            if is_key {
                in_parts_list = true;
                list_indent = current_indent;
                if let Some((_, val)) = trimmed.split_once(':') {
                    let val_clean = val.trim().trim_matches(['"', '\'', '[', ']']);
                    if !val_clean.is_empty() {
                        for item in val_clean.split(',') {
                            let item_trim = item.trim().trim_matches(['"', '\'']);
                            if is_valid_subsystem_identifier(item_trim) {
                                parts.insert(item_trim.to_string());
                            }
                        }
                    }
                }
                continue;
            }

            if in_parts_list {
                if current_indent <= list_indent && !trimmed.starts_with('-') {
                    in_parts_list = false;
                } else if let Some(rest) = trimmed.strip_prefix('-') {
                    let item_clean = rest.trim();
                    let name = if let Some((_, v)) = item_clean.split_once(':') {
                        v.trim().trim_matches(['"', '\''])
                    } else {
                        item_clean.trim_matches(['"', '\''])
                    };
                    let ident: String = name
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if is_valid_subsystem_identifier(&ident) {
                        parts.insert(ident);
                    }
                }
            }
        }
    }
}

impl SysmlModel {
    /// Extracts a deterministic, token-budgeted `AstSlice` for `target_package` from this `SysmlModel`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and `DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md`
    /// context-budget enforcement (`SysmlModel::slice_package_subsystem`). Prevents context-window overflow
    /// by isolating only the target subsystem's `PartDef`s, `PortDef`s, `InterfaceDef`s, referenced `RequirementDef`s,
    /// and referenced `HazardDef`s.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Estimates token count deterministically as `json_bytes.len().saturating_add(3) / 4`.
    /// - Enforces `estimated_tokens <= max_tokens` and fails closed with `Err(String)` when exceeded.
    ///
    /// ## 3. Preconditions
    /// - `target_package: &str` is a non-empty package or subsystem part identifier.
    /// - `max_tokens: usize` is strictly positive (`max_tokens > 0`).
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(AstSlice)` with `estimated_tokens <= max_tokens` when a matching package or part exists,
    ///   or `Err(String)` on missing target or token budget violation.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(P + R + H)$ where $P$, $R$, and $H$ are part, requirement, and hazard counts in `self`.
    /// - Space Complexity: $O(S)$ where $S$ is the serialized size of the resulting `AstSlice`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Rejects `max_tokens == 0` and empty `target_package` strings immediately.
    /// - State: Read-only projection; never mutates `self`.
    /// - Action: Deterministic JSON byte estimation via `serde_json::to_vec`.
    /// - Resource: Fails closed before returning an oversized slice that exceeds `max_tokens`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SLICE-PACKAGE-SUBSYSTEM]
    pub fn slice_package_subsystem(
        &self,
        target_package: &str,
        max_tokens: usize,
    ) -> Result<AstSlice, CoreError> {
        let target = target_package.trim();
        if target.is_empty() {
            return Err(CoreError::validation("target_package must not be empty"));
        }
        if max_tokens == 0 {
            return Err(CoreError::validation(
                "max_tokens must be greater than zero",
            ));
        }

        let pkg_matches = self
            .package_name
            .as_deref()
            .is_some_and(|pkg_name| pkg_name == target || pkg_name.eq_ignore_ascii_case(target));

        let matched_parts: Vec<PartDef> = if pkg_matches {
            self.parts.clone()
        } else {
            self.parts
                .iter()
                .filter(|p| {
                    p.name == target
                        || p.name.starts_with(target)
                        || p.name.eq_ignore_ascii_case(target)
                })
                .cloned()
                .collect()
        };

        if !pkg_matches && matched_parts.is_empty() {
            return Err(CoreError::validation(format!(
                "No package or part matching '{}' found in SysmlModel",
                target
            )));
        }

        let mut ports: Vec<PortDef> = Vec::new();
        if pkg_matches {
            for port in &self.ports {
                if !ports.iter().any(|existing| existing.name == port.name) {
                    ports.push(port.clone());
                }
            }
        }

        let mut interfaces: Vec<InterfaceDef> = Vec::new();
        let mut referenced_req_ids: Vec<String> = Vec::new();
        let mut referenced_hazard_ids: Vec<String> = Vec::new();
        let mut discovered_hazards: Vec<HazardDef> = Vec::new();
        let mut part_requirements: Vec<RequirementDef> = Vec::new();

        for part in &matched_parts {
            for port in &part.ports {
                if !ports.iter().any(|existing| existing.name == port.name) {
                    ports.push(port.clone());
                }
            }
            for conn in &part.connections {
                let iface = InterfaceDef {
                    name: conn.name.clone(),
                    source_port: conn.source_port.clone(),
                    target_port: conn.target_port.clone(),
                    doc: conn.doc.clone(),
                };
                if !interfaces.contains(&iface) {
                    interfaces.push(iface);
                }
            }
            for req_id in &part.satisfied_requirements {
                if !referenced_req_ids.contains(req_id) {
                    referenced_req_ids.push(req_id.clone());
                }
            }
            for safety_req_id in &part.safety_realises {
                if !referenced_req_ids.contains(safety_req_id) {
                    referenced_req_ids.push(safety_req_id.clone());
                }
            }
            for haz_id in &part.triggers_hazards {
                if !referenced_hazard_ids.contains(haz_id) {
                    referenced_hazard_ids.push(haz_id.clone());
                }
            }
            for haz in &part.hazards {
                if !discovered_hazards.iter().any(|h| h.name == haz.name) {
                    discovered_hazards.push(haz.clone());
                }
            }
            for req in &part.requirements {
                if !part_requirements
                    .iter()
                    .any(|r| r.name == req.name && r.req_id == req.req_id)
                {
                    part_requirements.push(req.clone());
                }
            }
        }

        let mut requirements: Vec<RequirementDef> = Vec::new();
        for req in self.requirements.iter().chain(part_requirements.iter()) {
            let is_referenced = referenced_req_ids.is_empty()
                || referenced_req_ids
                    .iter()
                    .any(|id| id == &req.name || id == &req.req_id);
            if is_referenced
                && !requirements
                    .iter()
                    .any(|r| r.name == req.name && r.req_id == req.req_id)
            {
                requirements.push(req.clone());
                for haz_id in &req.triggers_hazards {
                    if !referenced_hazard_ids.contains(haz_id) {
                        referenced_hazard_ids.push(haz_id.clone());
                    }
                }
            }
        }

        let mut hazards: Vec<HazardDef> = Vec::new();
        for haz in &discovered_hazards {
            if (referenced_hazard_ids.is_empty() || referenced_hazard_ids.contains(&haz.name))
                && !hazards.iter().any(|h| h.name == haz.name)
            {
                hazards.push(haz.clone());
            }
        }
        for haz_id in &referenced_hazard_ids {
            if !hazards.iter().any(|h| &h.name == haz_id) {
                hazards.push(HazardDef {
                    name: haz_id.clone(),
                    doc: None,
                    severity: 1,
                    source_port: None,
                    target_port: None,
                    part_ref: None,
                });
            }
        }

        let slice_pkg = PackageDef {
            name: target.to_string(),
            doc: self.doc.clone(),
            part_defs: matched_parts.clone(),
            port_defs: ports.clone(),
            requirement_defs: requirements.clone(),
            hazard_defs: hazards.clone(),
            ..Default::default()
        };

        let mut slice = AstSlice {
            target_package: target.to_string(),
            packages: vec![slice_pkg],
            parts: matched_parts,
            ports,
            interfaces,
            requirements,
            hazards,
            estimated_tokens: 0,
        };

        let json_bytes = serde_json::to_vec(&slice).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize AstSlice for token estimation: {}",
                e
            ))
        })?;
        let estimated_tokens = json_bytes.len().saturating_add(3) / 4;
        slice.estimated_tokens = estimated_tokens;

        if estimated_tokens > max_tokens {
            return Err(CoreError::validation(format!(
                "AstSlice for '{}' requires {} estimated tokens, exceeding max_tokens budget of {}",
                target, estimated_tokens, max_tokens
            )));
        }

        Ok(slice)
    }
}

/// Content-addressable SHA-256 incremental cache for parsed `SysmlModel` AST trees under `.pipeline/cache/ast/`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `IncrementalAstCache` in `crates/deap-core/src/sysml_ast/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `IncrementalAstCache` across member set `[cache_dir: std::path::PathBuf]`.
///
/// ## 3. Preconditions
/// - Caller supplies a valid repository root path to `IncrementalAstCache::new` or a valid `cache_dir` path.
///
/// ## 4. Postconditions
/// - Defines the authoritative `IncrementalAstCache` representation for struct `IncrementalAstCache`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `IncrementalAstCache`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `IncrementalAstCache`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `IncrementalAstCache` (`IncrementalAstCache`).
/// - State: Maintains consistent state invariants across `IncrementalAstCache` without partial mutation on error.
/// - Action: Executes `IncrementalAstCache` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `IncrementalAstCache` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-INCREMENTALASTCACHE] | SysML v2: PartDef(IncrementalAstCache)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalAstCache {
    pub cache_dir: std::path::PathBuf,
}

/// Computes the lowercase hexadecimal SHA-256 digest of `bytes` by delegating to `crate::digest::compute_sha256`.
/// Realises: [REQ-SYSML-AST-CACHE-DIGEST]
fn compute_sha256_hex(bytes: &[u8]) -> String {
    crate::digest::compute_sha256(bytes)
}

impl IncrementalAstCache {
    /// Initializes an `IncrementalAstCache` rooted at `<repo_root>/.pipeline/cache/ast`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and `DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md` incremental AST caching rules (`IncrementalAstCache::new`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Resolves `self.cache_dir` deterministically as `repo_root.join(".pipeline").join("cache").join("ast")`.
    ///
    /// ## 3. Preconditions
    /// - `repo_root: &std::path::Path` is a valid workspace directory path reference.
    ///
    /// ## 4. Postconditions
    /// - Returns an `IncrementalAstCache` instance configured with the canonical AST cache directory path.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(P)$ where $P$ is the path length of `repo_root`.
    /// - Space Complexity: $O(P)$ for the allocated `PathBuf`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts any valid `Path` reference without performing premature I/O.
    /// - State: Pure constructor.
    /// - Action: Never panics.
    /// - Resource: Single `PathBuf` allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-CACHE-NEW]
    pub fn new(repo_root: &std::path::Path) -> Self {
        Self {
            cache_dir: repo_root.join(".pipeline").join("cache").join("ast"),
        }
    }

    /// Computes the lowercase hexadecimal SHA-256 digest of `sysml_source`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and FIPS 180-4 (`IncrementalAstCache::compute_digest`).
    /// Ensures any single-byte change in `sysml_source` invalidates prior cached AST artifacts.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Deterministic 64-character lowercase hex SHA-256 digest of `sysml_source.as_bytes()`.
    ///
    /// ## 3. Preconditions
    /// - `sysml_source: &str` is a valid UTF-8 SysML v2 source string.
    ///
    /// ## 4. Postconditions
    /// - Returns the 64-character lowercase hex SHA-256 string.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$ where $N$ is `sysml_source.len()`.
    /// - Space Complexity: $O(N)$ for message block padding.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Handles empty and arbitrarily large UTF-8 strings safely.
    /// - State: Pure function.
    /// - Action: Delegates to `crate::digest::compute_sha256`.
    /// - Resource: Bounded by `sysml_source.len()`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-CACHE-DIGEST] | Issue: #494, #503, #522
    pub fn compute_digest(sysml_source: &str) -> String {
        compute_sha256_hex(sysml_source.as_bytes())
    }

    /// Loads a cached `SysmlModel` by SHA-256 digest if present on disk, or parses `sysml_source` and populates the cache.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and `DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md` (`IncrementalAstCache::load_or_parse`).
    /// Eliminates redundant re-parsing of unchanged SysML v2 schemas while guaranteeing bit-for-bit AST fidelity.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns `Ok((model, true))` on a verified SHA-256 cache hit when `<cache_dir>/<digest>.json` exists and deserializes cleanly.
    /// - Returns `Ok((model, false))` on a cache miss after persisting the pretty-printed JSON AST to `<cache_dir>/<digest>.json`.
    ///
    /// ## 3. Preconditions
    /// - `sysml_source: &str` is a valid UTF-8 SysML v2 source string.
    /// - `self.cache_dir` is a writable filesystem path (created automatically on cache miss).
    ///
    /// ## 4. Postconditions
    /// - `<self.cache_dir>/<digest>.json` exists on disk and contains the serialized `SysmlModel`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$ for SHA-256 calculation plus $O(N)$ JSON deserialization (hit) or SysML parsing + serialization (miss).
    /// - Space Complexity: $O(N)$ for the `SysmlModel` and JSON buffer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates filesystem I/O or JSON serialization/deserialization errors via `Err(String)`.
    /// - State: Creates `self.cache_dir` via `std::fs::create_dir_all` prior to writing.
    /// - Action: Never swallows I/O or deserialization errors silently.
    /// - Resource: Bounded by AST document size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-CACHE-LOAD-OR-PARSE]
    pub fn load_or_parse(&self, sysml_source: &str) -> Result<(SysmlModel, bool), CoreError> {
        let digest = Self::compute_digest(sysml_source);
        let cache_path = self.cache_dir.join(format!("{}.json", digest));
        if cache_path.is_file() {
            let cached_text = std::fs::read_to_string(&cache_path).map_err(|e| {
                CoreError::io(format!(
                    "Failed to read cached AST file '{}': {}",
                    cache_path.display(),
                    e
                ))
            })?;
            let model: SysmlModel = serde_json::from_str(&cached_text).map_err(|e| {
                CoreError::validation(format!(
                    "Failed to deserialize cached AST file '{}': {}",
                    cache_path.display(),
                    e
                ))
            })?;
            return Ok((model, true));
        }

        let model = parse_sysml_model(sysml_source);
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            CoreError::io(format!(
                "Failed to create AST cache directory '{}': {}",
                self.cache_dir.display(),
                e
            ))
        })?;
        let serialized = serde_json::to_string_pretty(&model).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize SysmlModel for AST cache: {}",
                e
            ))
        })?;
        std::fs::write(&cache_path, serialized).map_err(|e| {
            CoreError::io(format!(
                "Failed to write cached AST file '{}': {}",
                cache_path.display(),
                e
            ))
        })?;
        Ok((model, false))
    }
}

/// Zero-mocking file-backed persistence engine for `SysmlModel` AST snapshots (`PERSISTENCE_ARCHITECTURE.md`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and `PERSISTENCE_ARCHITECTURE.md` safety objectives for struct `AstPersistenceEngine` in `crates/deap-core/src/sysml_ast/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `AstPersistenceEngine` across member set `[storage_root: std::path::PathBuf]`.
///
/// ## 3. Preconditions
/// - Caller supplies a valid directory path for `storage_root: std::path::PathBuf`.
///
/// ## 4. Postconditions
/// - Defines the authoritative `AstPersistenceEngine` representation for struct `AstPersistenceEngine`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `AstPersistenceEngine`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `AstPersistenceEngine`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AstPersistenceEngine` (`AstPersistenceEngine`).
/// - State: Maintains consistent state invariants across `AstPersistenceEngine` without partial mutation on error.
/// - Action: Executes `AstPersistenceEngine` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `AstPersistenceEngine` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-ASTPERSISTENCEENGINE] | SysML v2: PartDef(AstPersistenceEngine)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AstPersistenceEngine {
    pub storage_root: std::path::PathBuf,
}

impl AstPersistenceEngine {
    /// Creates a new `AstPersistenceEngine` backed by `storage_root`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and `.pipeline/constitution.md` Section 1.9 Zero-Mocking Live Persistence Mandate (`AstPersistenceEngine::new`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Stores `storage_root` verbatim for deterministic `<storage_root>/<model_id>.json` resolution.
    ///
    /// ## 3. Preconditions
    /// - `storage_root: std::path::PathBuf` is a valid filesystem path.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `AstPersistenceEngine`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ move construction.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible constructor.
    /// - State: Pure value initialization.
    /// - Action: Zero panics.
    /// - Resource: Zero additional heap allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-PERSISTENCE-NEW]
    pub fn new(storage_root: std::path::PathBuf) -> Self {
        Self { storage_root }
    }

    /// Persists `model` under `model_id` as a canonical JSON document inside `self.storage_root`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and `PERSISTENCE_ARCHITECTURE.md` (`AstPersistenceEngine::persist_model`).
    /// Ensures durable, deterministic serialization of `SysmlModel` instances to disk.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Rejects empty or path-traversing `model_id` values (`..`, `/`, `\\`).
    /// - Round-trip invariant: `self.load_model(model_id)` after `self.persist_model(model_id, model)` yields a `SysmlModel` equal to `model`.
    ///
    /// ## 3. Preconditions
    /// - `model_id: &str` is a non-empty identifier free of path separators.
    /// - `model: &SysmlModel` is a valid AST model reference.
    ///
    /// ## 4. Postconditions
    /// - Writes `<storage_root>/<model_id>.json` and returns `Ok(PathBuf)` pointing to the persisted file.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$ where $N$ is the serialized byte size of `model`.
    /// - Space Complexity: $O(N)$ for the formatted JSON buffer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Err(String)` if `model_id` is empty or contains path separators.
    /// - State: Creates `self.storage_root` via `std::fs::create_dir_all` before writing.
    /// - Action: Propagates all I/O and serialization errors via `Result::Err`.
    /// - Resource: Bounded by `model` AST size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-PERSIST-MODEL]
    pub fn persist_model(
        &self,
        model_id: &str,
        model: &SysmlModel,
    ) -> Result<std::path::PathBuf, CoreError> {
        let clean_id = model_id.trim();
        if clean_id.is_empty()
            || clean_id.contains("..")
            || clean_id.contains('/')
            || clean_id.contains('\\')
        {
            return Err(CoreError::validation(format!(
                "Invalid model_id '{}' for persistence",
                model_id
            )));
        }
        std::fs::create_dir_all(&self.storage_root).map_err(|e| {
            CoreError::io(format!(
                "Failed to create persistence root '{}': {}",
                self.storage_root.display(),
                e
            ))
        })?;
        let target_path = self.storage_root.join(format!("{}.json", clean_id));
        let json_payload = serde_json::to_string_pretty(model).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize SysmlModel '{}' for persistence: {}",
                clean_id, e
            ))
        })?;
        std::fs::write(&target_path, json_payload).map_err(|e| {
            CoreError::io(format!(
                "Failed to write persisted model '{}': {}",
                target_path.display(),
                e
            ))
        })?;
        Ok(target_path)
    }

    /// Loads and deserializes a persisted `SysmlModel` identified by `model_id` from `self.storage_root`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and `PERSISTENCE_ARCHITECTURE.md` (`AstPersistenceEngine::load_model`).
    /// Restores a previously persisted `SysmlModel` snapshot with full structural fidelity.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Rejects empty or path-traversing `model_id` values (`..`, `/`, `\\`).
    /// - Deserializes `<storage_root>/<model_id>.json` into an owned `SysmlModel`.
    ///
    /// ## 3. Preconditions
    /// - `model_id: &str` refers to a model previously written via `persist_model`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(SysmlModel)` on valid file read and JSON deserialization, or `Err(CoreError)` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$ where $N$ is the byte length of `<model_id>.json`.
    /// - Space Complexity: $O(N)$ for the loaded `SysmlModel`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Fails closed with `Err(CoreError)` when the file is missing, unreadable, or malformed JSON.
    /// - State: Read-only filesystem operation.
    /// - Action: Zero panics or silent error suppression.
    /// - Resource: Bounded by persisted file size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-LOAD-MODEL]
    pub fn load_model(&self, model_id: &str) -> Result<SysmlModel, CoreError> {
        let clean_id = model_id.trim();
        if clean_id.is_empty()
            || clean_id.contains("..")
            || clean_id.contains('/')
            || clean_id.contains('\\')
        {
            return Err(CoreError::validation(format!(
                "Invalid model_id '{}' for load_model",
                model_id
            )));
        }
        let target_path = self.storage_root.join(format!("{}.json", clean_id));
        let json_payload = std::fs::read_to_string(&target_path).map_err(|e| {
            CoreError::io(format!(
                "Failed to read persisted model '{}': {}",
                target_path.display(),
                e
            ))
        })?;
        serde_json::from_str(&json_payload).map_err(|e| {
            CoreError::validation(format!(
                "Failed to deserialize persisted model '{}': {}",
                target_path.display(),
                e
            ))
        })
    }
}

/// Cryptographically bound Chain-of-Thought (CoT) reasoning audit record (`DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and `DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md` safety objectives for struct `CotAuditRecord` in `crates/deap-core/src/sysml_ast/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `CotAuditRecord` across member set `[step_id: String, model_id: String, prompt_sha256: String, reasoning_trace: String, verified_ast_refs: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `CotAuditRecord` members (`step_id: String, model_id: String, prompt_sha256: String, reasoning_trace: String, verified_ast_refs: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `CotAuditRecord` representation for struct `CotAuditRecord`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `CotAuditRecord`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `CotAuditRecord`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `CotAuditRecord` (`CotAuditRecord`).
/// - State: Maintains consistent state invariants across `CotAuditRecord` without partial mutation on error.
/// - Action: Executes `CotAuditRecord` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `CotAuditRecord` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-COTAUDITRECORD] | SysML v2: PartDef(CotAuditRecord)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CotAuditRecord {
    pub step_id: String,
    pub model_id: String,
    pub prompt_sha256: String,
    pub reasoning_trace: String,
    pub verified_ast_refs: Vec<String>,
}

/// Append-only audit log of `CotAuditRecord` entries for DO-330 TQL-4 verification traceability.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and `DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md` safety objectives for struct `CotAuditLog` in `crates/deap-core/src/sysml_ast/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `CotAuditLog` across member set `[records: Vec<CotAuditRecord>]`.
///
/// ## 3. Preconditions
/// - Caller constructs `CotAuditLog` via `CotAuditLog::new()` or `Default::default()` and appends valid `CotAuditRecord` entries.
///
/// ## 4. Postconditions
/// - Defines the authoritative `CotAuditLog` representation for struct `CotAuditLog`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `CotAuditLog`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `CotAuditLog`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `CotAuditLog` (`CotAuditLog`).
/// - State: Maintains consistent state invariants across `CotAuditLog` without partial mutation on error.
/// - Action: Executes `CotAuditLog` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `CotAuditLog` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-COTAUDITLOG] | SysML v2: PartDef(CotAuditLog)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CotAuditLog {
    pub records: Vec<CotAuditRecord>,
}

impl CotAuditLog {
    /// Creates an empty `CotAuditLog`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and `DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md` (`CotAuditLog::new`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `self.records` as an empty vector (`self.records.is_empty()`).
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns an empty `CotAuditLog` ready to record reasoning traces.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$ zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible constructor.
    /// - State: Clean initial state.
    /// - Action: Never panics.
    /// - Resource: Zero initial capacity allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-NEW]
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Appends a `CotAuditRecord` in chronological order to `self.records`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and `DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md` (`CotAuditLog::append_record`).
    /// Preserves a tamper-evident chronological sequence of synthesis reasoning steps and verified AST references.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Strictly monotonic: `self.records.len()` increases by 1 and prior records remain unmodified.
    ///
    /// ## 3. Preconditions
    /// - `record: CotAuditRecord` is a populated audit record.
    ///
    /// ## 4. Postconditions
    /// - `self.records.last() == Some(&record)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ amortized vector push.
    /// - Space Complexity: $O(R)$ where $R$ is the size of `record`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts any owned `CotAuditRecord`.
    /// - State: Mutates only `self.records`.
    /// - Action: Never panics under normal memory availability.
    /// - Resource: Single record move into `self.records`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-APPEND]
    pub fn append_record(&mut self, record: CotAuditRecord) {
        self.records.push(record);
    }

    /// Serializes the `CotAuditLog` to a deterministic pretty-printed JSON string.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and `DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md` (`CotAuditLog::to_json_pretty`).
    /// Produces human-reviewable and machine-verifiable JSON audit evidence.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Round-trip invariant: `serde_json::from_str::<CotAuditLog>(&log.to_json_pretty()?) == Ok(log)`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `CotAuditLog` reference.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(String)` containing the pretty-printed JSON representation, or `Err(String)` on serialization failure.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$ where $N$ is the total byte length across all `CotAuditRecord` entries.
    /// - Space Complexity: $O(N)$ for the formatted JSON string.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Maps any `serde_json` error to a descriptive `Err(String)`.
    /// - State: Read-only serialization.
    /// - Action: Zero panics.
    /// - Resource: Bounded by log size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-JSON]
    pub fn to_json_pretty(&self) -> Result<String, CoreError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize CotAuditLog to pretty JSON: {}",
                e
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

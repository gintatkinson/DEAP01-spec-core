//! Parity gates (Checks 21 through 31) mirroring DEAP specification compiler gates.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides comprehensive, deterministic verification gates enforcing
//! semantic diagram-to-AST topology parity, physical prose invariant grounding, Level 1C ICD
//! completeness, cross-document connection parity, and dual-schema SSOT synchronization.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Schema-AST Grounding Invariant: Zero undeclared physical hardware components or ungrounded
//!   ports in specification diagrams and interface matrices.
//! - Traceability Preservation: 1:1 bidirectional parity across ConOps SV-1, ICD_01, executive
//!   reports, and the canonical SSOT Architecture Views Library.
//! - Monotonic Determinism: Zero randomized hashing structures; all collections use BTreeMap
//!   and BTreeSet, and all directory scans sort paths lexicographically.
//! - Memory & Char Boundary Safety: All string slicing operations use character boundary validation
//!   and ASCII case transformations, eliminating panic hazards and internal compiler errors.
//!
//! ## 3. Preconditions
//! - Repository root must be an accessible local filesystem directory.
//! - Schemas and specifications must contain UTF-8 encoded text.
//!
//! ## 4. Postconditions
//! - Returns `Ok(ParityOutcome::Verified)` when all parity invariants hold.
//! - Returns `Ok(ParityOutcome::PendingOrClean)` when upstream clean landing zones or downstream
//!   pending states are detected.
//! - Returns `Err(Vec<String>)` containing explicit, actionable failure diagnostics upon violation.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N * L) where N is the total count of specification and schema files,
//!   and L is the average line length.
//! - Space Complexity: O(S) bounded by total token and symbol count across active schemas and specs.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Filesystem I/O, UTF-8 parsing, and Markdown AST parsing failures fail closed and propagate
//!   structured verification error diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-PARITY-GATES-21-THROUGH-31], [DO-330-TQL-4-VERIFICATION-TOOL].

pub use super::parity_check_conops_exec::{
    check_conops_and_mission_intent_completeness, check_executive_deliverable_traceability,
    check_research_inventory,
};
pub use super::parity_check_coverage::check_coverage_digest;
pub use super::parity_check_cross_doc::{
    check_cross_document_diagram_parity, extract_connection_ids,
};
pub use super::parity_check_discovery::{
    discover_architectural_sysml_model_text, discover_sysml_model_text,
};
pub use super::parity_check_dual_ssot::check_dual_schema_ssot_parity;
pub use super::parity_check_grounding::{
    audit_spec_citations_and_protocols, check_factual_grounding,
};
pub use super::parity_check_icd_ast::{
    extract_port_identifier, extract_single_conn_id, extract_single_signal_id,
    is_table_separator_line, parse_markdown_table_rows,
};
pub use super::parity_check_icd_gate::check_icd_completeness;
pub use super::parity_check_obligation::{
    check_architecture_viewpoint_diagrams, check_obligation_witness,
};
pub use super::parity_check_ops_std::{
    check_operational_allocation, check_standards_measurement,
};
pub use super::parity_check_semantic::{
    check_semantic_diagram_ast_parity, check_semantic_prose_invariants,
};
pub use super::parity_check_slicer::{extract_citations_from_line, CitationRef, SectionSlicer};

/// Outcome status for parity gates.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ParityOutcome` in `crates/verify-baseline/src/checks/parity_gates.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `ParityOutcome` across member set `[PendingOrClean, InterfacesDirMissing, IcdSpecsPending, ConopsDirMissing, Verified, DualSchemaSingleOrClean]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ParityOutcome` members (`PendingOrClean, InterfacesDirMissing, IcdSpecsPending, ConopsDirMissing, Verified, DualSchemaSingleOrClean`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ParityOutcome` representation for enum `ParityOutcome`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `ParityOutcome`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `ParityOutcome`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ParityOutcome` (`ParityOutcome`).
/// - State: Maintains consistent state invariants across `ParityOutcome` without partial mutation on error.
/// - Action: Executes `ParityOutcome` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ParityOutcome` (`CHECKS-PARITY-GATES`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECKS-PARITY-GATES-PARITYOUTCOME] | SysML v2: PartDef(ParityOutcome)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityOutcome {
    PendingOrClean,
    InterfacesDirMissing,
    IcdSpecsPending,
    ConopsDirMissing,
    Verified,
    DualSchemaSingleOrClean,
    DualSchemaIdentical,
}

/// Canonical Mermaid diagram types recognized across specification architecture viewpoints.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `DiagramType` in `crates/verify-baseline/src/checks/parity_gates.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `DiagramType` across member set `[Flowchart, Graph, ClassDiagram, StateDiagramV2, StateDiagram, SequenceDiagram]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `DiagramType` members (`Flowchart, Graph, ClassDiagram, StateDiagramV2, StateDiagram, SequenceDiagram`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `DiagramType` representation for enum `DiagramType`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `DiagramType`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `DiagramType`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `DiagramType` (`DiagramType`).
/// - State: Maintains consistent state invariants across `DiagramType` without partial mutation on error.
/// - Action: Executes `DiagramType` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `DiagramType` (`CHECKS-PARITY-GATES`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECKS-PARITY-GATES-DIAGRAMTYPE] | SysML v2: PartDef(DiagramType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagramType {
    Flowchart,
    Graph,
    ClassDiagram,
    StateDiagramV2,
    StateDiagram,
    SequenceDiagram,
    ErDiagram,
    Gantt,
    Pie,
    GitGraph,
    C4Context,
    Journey,
    QuadrantChart,
    XyChartBeta,
    Mindmap,
    Timeline,
    SankeyBeta,
    RequirementDiagram,
}

impl DiagramType {
    /// Parses a diagram header line into a strongly typed DiagramType variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Ensures diagram syntax conforms to standard Mermaid grammar without static string slices.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Token matching is ASCII case-insensitive and checks word boundaries.
    ///
    /// ## 3. Preconditions
    /// Input line must be valid UTF-8.
    ///
    /// ## 4. Postconditions
    /// Returns `Some(DiagramType)` if valid header found, `None` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(T) where T is token length.
    /// Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Returns `None` on unrecognized or empty lines without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MERMAID-HEADER-TYPED-AST].
    pub fn parse_header(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        let token = trimmed.split_whitespace().next()?;
        if token.eq_ignore_ascii_case("flowchart") {
            Some(Self::Flowchart)
        } else if token.eq_ignore_ascii_case("graph") {
            Some(Self::Graph)
        } else if token.eq_ignore_ascii_case("classdiagram") {
            Some(Self::ClassDiagram)
        } else if token.eq_ignore_ascii_case("statediagram-v2") {
            Some(Self::StateDiagramV2)
        } else if token.eq_ignore_ascii_case("statediagram") {
            Some(Self::StateDiagram)
        } else if token.eq_ignore_ascii_case("sequencediagram") {
            Some(Self::SequenceDiagram)
        } else if token.eq_ignore_ascii_case("erdiagram") {
            Some(Self::ErDiagram)
        } else if token.eq_ignore_ascii_case("gantt") {
            Some(Self::Gantt)
        } else if token.eq_ignore_ascii_case("pie") {
            Some(Self::Pie)
        } else if token.eq_ignore_ascii_case("gitgraph") {
            Some(Self::GitGraph)
        } else if token.eq_ignore_ascii_case("c4context") {
            Some(Self::C4Context)
        } else if token.eq_ignore_ascii_case("journey") {
            Some(Self::Journey)
        } else if token.eq_ignore_ascii_case("quadrantchart") {
            Some(Self::QuadrantChart)
        } else if token.eq_ignore_ascii_case("xychart-beta") {
            Some(Self::XyChartBeta)
        } else if token.eq_ignore_ascii_case("mindmap") {
            Some(Self::Mindmap)
        } else if token.eq_ignore_ascii_case("timeline") {
            Some(Self::Timeline)
        } else if token.eq_ignore_ascii_case("sankey-beta") {
            Some(Self::SankeyBeta)
        } else if token.eq_ignore_ascii_case("requirementdiagram") {
            Some(Self::RequirementDiagram)
        } else {
            None
        }
    }
}

/// Validate whether the given line matches a valid Mermaid diagram type header.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Gatekeeper verifying valid diagram type specification headers.
///
/// ## 2. Invariants & Mathematical Properties
/// Delegates to strongly-typed `DiagramType::parse_header`.
///
/// ## 3. Preconditions
/// Input string must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true if valid diagram type header detected.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(T) where T is token length.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Safe boolean return; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MERMAID-HEADER-VALIDATION].
pub fn is_valid_mermaid_header(s: &str) -> bool {
    DiagramType::parse_header(s).is_some()
}

/// Token scanner checking for forbidden Mermaid class method colon syntax:
/// e.g. `+method(): ReturnType` instead of `+ReturnType method()`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces syntax compliance for UML class diagrams in specifications.
///
/// ## 2. Invariants & Mathematical Properties
/// Rejects colon-after-parentheses method signatures in class member lines.
///
/// ## 3. Preconditions
/// Input string `s` must be valid UTF-8.
///
/// ## 4. Postconditions
/// Returns true if forbidden colon syntax is detected.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is string length.
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Uses safe slice bounds access; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MERMAID-CLASS-SYNTAX-GATE].
pub fn has_colon_method_syntax(s: &str) -> bool {
    let trimmed = s.trim();
    let mut chars = trimmed.chars();

    match chars.next() {
        Some('+' | '-' | '#' | '~') => {}
        _ => return false,
    }

    let rem = chars.as_str();
    let ident_end = rem
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rem.len());
    if ident_end == 0 {
        return false;
    }

    let after_ident = match rem.get(ident_end..) {
        Some(slice) => slice.trim_start(),
        None => return false,
    };
    if !after_ident.starts_with('(') {
        return false;
    }

    if let Some(close_idx) = after_ident.rfind(')') {
        let after_paren = match after_ident.get(close_idx.saturating_add(1)..) {
            Some(slice) => slice.trim_start(),
            None => return false,
        };
        if let Some(after_colon) = after_paren.strip_prefix(':') {
            let ret_type = after_colon.trim_start();
            if let Some(first_ch) = ret_type.chars().next() {
                if first_ch.is_alphanumeric() || first_ch == '_' {
                    return true;
                }
            }
        }
    }

    false
}

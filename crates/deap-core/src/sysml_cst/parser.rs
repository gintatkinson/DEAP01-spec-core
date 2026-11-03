//! # SysML v2 / KerML Rowan CST Parser Entrypoint
//! [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the primary lossless Rowan CST parser (`parse_sysml_cst`) and Clause 8.2.2 construct
//! classification for OMG SysML v2.0 and KerML 1.0 documents under DO-178C Level A (`BP-03`, `BP-10`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Lossless Identity: `parse_sysml_cst(source).syntax().text().to_string() == source` for all inputs.
//! - Hierarchical Tree: Constructs nested composite `SysmlSyntaxKind` child nodes via `SysmlCstBuilder`
//!   (`parser_clauses_1_3` and `parser_clauses_4_6`).
//!
//! ## 3. Preconditions
//! - `source` is any valid UTF-8 string slice.
//!
//! ## 4. Postconditions
//! - Returns a `ParsedSysmlCst` containing the immutable `rowan::GreenNode`, extracted
//!   `SysmlConstructSummary` inventory, and `DEAP-E1001`/`DEAP-E1002` diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) over input byte length `N`.
//! - Space Complexity: O(N) green-tree storage.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Never panics; malformed tokens and depth overflows emit `SysmlSyntaxKind::ErrorNode` and
//!   diagnostic strings.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

use std::ops::Range;

use rowan::GreenNode;

use super::lexer::tokenize_sysml;
use super::parser_clauses_1_3::SysmlCstBuilder;
use super::syntax_kind::{SysmlSyntaxKind, SysmlSyntaxNode};

/// OMG SysML v2 Clause 8.2.2 grammar construct taxonomy.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Enumerates the normative Clause 8.2.2 construct categories enforced by `check_sysml_clause8_coverage`.
/// ## 2. Invariants & Mathematical Properties
/// Bijective mapping onto the 21 required Clause 8.2.2 grammar productions.
/// ## 3. Preconditions
/// None.
/// ## 4. Postconditions
/// Provides canonical Clause 8.2.2 construct tags.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1); Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value enum with zero runtime faults.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SysmlClause8Kind {
    Package,
    Import,
    PartDef,
    PortDef,
    ActionDef,
    StateDef,
    RequirementDef,
    UseCaseDef,
    VerificationDef,
    ConstraintDef,
    ConnectionDef,
    InterfaceDef,
    FlowDef,
    AllocationDef,
    CalculationDef,
    OccurrenceDef,
    ViewDef,
    ViewpointDef,
    MetadataDef,
    Satisfy,
    Verify,
}

/// Extracted construct summary from a SysML CST parse pass.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Records the syntax kind, identifier, definition status, and byte span of each CST declaration.
/// ## 2. Invariants & Mathematical Properties
/// `byte_range.start <= byte_range.end`.
/// ## 3. Preconditions
/// Populated by `SysmlCstBuilder` during Rowan CST construction.
/// ## 4. Postconditions
/// Preserves declaration order across the source file.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) field access; Space Complexity: O(L) over identifier length.
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value struct with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysmlConstructSummary {
    pub kind: SysmlSyntaxKind,
    pub name: String,
    pub is_definition: bool,
    pub byte_range: Range<usize>,
}

/// Structured diagnostic emitted during SysML CST parsing.
/// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysmlCstDiagnostic {
    pub code: String,
    pub message: String,
    pub byte_range: Range<usize>,
}

/// Complete result of parsing a SysML v2 / KerML document into a Rowan CST.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Encapsulates the green CST root, extracted construct inventory, and syntax diagnostics.
/// ## 2. Invariants & Mathematical Properties
/// `self.syntax().text().to_string()` is byte-for-byte identical to the parsed source text.
/// ## 3. Preconditions
/// Produced by `parse_sysml_cst`.
/// ## 4. Postconditions
/// Provides O(1) `syntax()` root reconstruction and CST traversal helpers.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) root node projection; Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Total accessor methods with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSysmlCst {
    pub green: GreenNode,
    pub constructs: Vec<SysmlConstructSummary>,
    pub diagnostics: Vec<String>,
}

impl ParsedSysmlCst {
    /// Materializes a red `SysmlSyntaxNode` cursor over the green CST root.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Projects the immutable `GreenNode` into a navigable Rowan `SyntaxNode`.
    /// ## 2. Invariants & Mathematical Properties
    /// Root node kind is always `SysmlSyntaxKind::SourceFile`.
    /// ## 3. Preconditions
    /// `self.green` was built by `parse_sysml_cst`.
    /// ## 4. Postconditions
    /// Returns a `SysmlSyntaxNode` rooted at `SourceFile`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1); Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total function with zero panics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn syntax(&self) -> SysmlSyntaxNode {
        SysmlSyntaxNode::new_root(self.green.clone())
    }

    /// Collects all descendant composite `SysmlSyntaxNode`s matching `kind`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enables structural CST querying without regex or string scanning (`BP-03`).
    /// ## 2. Invariants & Mathematical Properties
    /// Returns nodes in deterministic pre-order traversal.
    /// ## 3. Preconditions
    /// `kind` is a `SysmlSyntaxKind` variant.
    /// ## 4. Postconditions
    /// Returns `Vec<SysmlSyntaxNode>` where every element has `node.kind() == kind`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(N) over CST node count; Space Complexity: O(M) matched nodes.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total traversal with zero panics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn nodes_of_kind(&self, kind: SysmlSyntaxKind) -> Vec<SysmlSyntaxNode> {
        self.syntax()
            .descendants()
            .filter(|n| n.kind() == kind)
            .collect()
    }
}

/// Classifies a SysML v2 / KerML keyword slice and `has_def` flag into its `SysmlSyntaxKind`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Maps all 6 SysML v2 grammar clauses to their corresponding composite `SysmlSyntaxKind`.
/// ## 2. Invariants & Mathematical Properties
/// Delegates to `SysmlCstBuilder::classify_clause_1_3_kind` and `classify_clause_4_6_kind`.
/// ## 3. Preconditions
/// `kw` is a keyword lexeme slice.
/// ## 4. Postconditions
/// Returns `Some(SysmlSyntaxKind)` for recognized keywords or `None` otherwise.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1); Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Total pure function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn classify_keyword_kind(kw: &str, has_def: bool) -> Option<SysmlSyntaxKind> {
    if kw == "use case" {
        return Some(if has_def {
            SysmlSyntaxKind::UseCaseDef
        } else {
            SysmlSyntaxKind::UseCaseUsage
        });
    }
    if kw == "test case" {
        return Some(if has_def {
            SysmlSyntaxKind::VerificationDef
        } else {
            SysmlSyntaxKind::VerificationUsage
        });
    }
    if let Some(kind) = SysmlCstBuilder::classify_clause_1_3_kind(kw, None, has_def, false) {
        return Some(kind);
    }
    let kind = SysmlCstBuilder::classify_clause_4_6_kind(kw, None, has_def);
    (kind != SysmlSyntaxKind::StatementNode).then_some(kind)
}

/// Parses a SysML v2 / KerML source string into a hierarchical Rowan CST (`ParsedSysmlCst`).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Primary 6-clause Rowan CST parser entrypoint (`BP-03`, `BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// - `parsed.syntax().text().to_string() == source` holds for all `source`.
/// - Constructs nested composite CST nodes (`PackageDecl`, `PartDef`, `ActionDef`, `StateDef`, etc.).
/// ## 3. Preconditions
/// `source` is a valid UTF-8 slice.
/// ## 4. Postconditions
/// Returns `ParsedSysmlCst` with populated `green`, `constructs`, and `diagnostics`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Never panics; syntax errors and depth overflows (`>= 64`) produce `ErrorNode`s and diagnostics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn parse_sysml_cst(source: &str) -> ParsedSysmlCst {
    let tokens = tokenize_sysml(source);
    let (green, diagnostics, constructs) = SysmlCstBuilder::new(&tokens).build_source_file();
    ParsedSysmlCst {
        green,
        constructs,
        diagnostics,
    }
}

/// Alias for [`parse_sysml_cst`] returning the green/red CST wrapper.
/// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn parse_sysml_green(source: &str) -> ParsedSysmlCst {
    parse_sysml_cst(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hierarchical_cst_and_lossless_roundtrip() {
        let src = "package FlightPkg {\n    doc /* Flight Control */\n    part def Autopilot :> BaseController {\n        port in cmd_in : CommandPort [1..1];\n        attribute gain : Real = 1.5;\n        state def ModeFsm {\n            state Idle;\n            transition first Idle then Idle;\n        }\n    }\n}\n";
        let parsed = parse_sysml_cst(src);
        assert_eq!(parsed.syntax().text().to_string(), src);
        assert!(parsed.diagnostics.is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::PackageDecl).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::PartDef).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::PortUsage).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::AttributeUsage).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::StateDef).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::TransitionDecl).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::SpecializationClause).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::MultiplicityBounds).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::ExpressionNode).is_empty());
    }

    #[test]
    fn test_depth_64_exceeded_emits_deap_e1002_and_preserves_text() {
        let open = "package P { ".repeat(65);
        let close = "}".repeat(65);
        let src = format!("{open}{close}");
        let parsed = parse_sysml_cst(&src);
        assert_eq!(parsed.syntax().text().to_string(), src);
        assert!(parsed.diagnostics.iter().any(|d| d.contains("DEAP-E1002")));
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::ErrorNode).is_empty());
    }
}

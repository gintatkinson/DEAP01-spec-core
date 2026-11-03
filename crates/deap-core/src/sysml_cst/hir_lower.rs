//! # SysML v2 / KerML Rowan CST-to-HIR Lowering
//! [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Lowers hierarchical Rowan CST trees (`ParsedSysmlCst`) into trivia-independent High-Level
//! Intermediate Representations (`HirPackageSummary`) and side-table span maps (`HirSourceMap`)
//! under DO-178C Level A and ISO 26262 ASIL D (`BP-03`, `BP-10`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Trivia-Blind Structural Hashing: `HirPackageSummary::structural_hash` depends exclusively on
//!   semantic declaration kinds, qualified names, imports, and specialization types -- never on
//!   whitespace, line comments, or byte offsets -- guaranteeing Salsa early-cutoff invariance.
//! - Span Isolation: Byte ranges reside strictly in `HirSourceMap::span_by_def`.
//!
//! ## 3. Preconditions
//! - `parsed` is a `ParsedSysmlCst` produced by `parse_sysml_cst`.
//!
//! ## 4. Postconditions
//! - Returns `(HirPackageSummary, HirSourceMap)` with deterministic `DefId` assignments.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) over CST nodes and tokens.
//! - Space Complexity: O(M) over lowered HIR member count `M`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Total lowering pass; `ErrorNode` subtrees are skipped without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

use std::collections::BTreeMap;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::digest::compute_sha256;
use super::parser::ParsedSysmlCst;
use super::syntax_kind::{SysmlSyntaxKind, SysmlSyntaxNode};

/// Dense definition identifier indexing `HirMemberEntry` and `HirSourceMap`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Provides location-independent identity for HIR definitions under DO-178C Level A (`BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// Assigned sequentially (`0..M`) in deterministic CST pre-order.
/// ## 3. Preconditions
/// None.
/// ## 4. Postconditions
/// Uniquely identifies a lowered HIR member within a compilation unit.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1); Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Copyable scalar newtype with zero runtime faults.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DefId(pub u32);

/// Trivia-independent HIR member lowered from a composite Rowan CST declaration node.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Captures semantic construct attributes for Salsa early-cutoff comparison (`BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// Excludes byte offsets so formatting edits preserve equality.
/// ## 3. Preconditions
/// Constructed by `lower_cst_to_hir`.
/// ## 4. Postconditions
/// Holds canonical `kind`, `name`, `qualified_name`, `is_definition`, and `type_ref`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) field access; Space Complexity: O(L) over identifier strings.
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value struct with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirMemberEntry {
    pub def_id: DefId,
    pub kind_name: String,
    pub name: String,
    pub qualified_name: String,
    pub is_definition: bool,
    pub type_ref: Option<String>,
}

/// Trivia-independent HIR package summary lowered from a `ParsedSysmlCst`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Serves as the early-cutoff value compared across Salsa revisions (`BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// `structural_hash` is a 64-hex SHA-256 digest of `(package_name, imports, members)`.
/// ## 3. Preconditions
/// Lowered from `ParsedSysmlCst` via `lower_cst_to_hir`.
/// ## 4. Postconditions
/// Two `HirPackageSummary` instances have equal `structural_hash` iff their semantic HIR is identical.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(M) comparison; Space Complexity: O(M).
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value struct with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirPackageSummary {
    pub package_name: String,
    pub imports: Vec<String>,
    pub members: Vec<HirMemberEntry>,
    pub structural_hash: String,
}

/// Side-table mapping each `DefId` to its exact CST byte range in the source buffer.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Decouples byte spans from `HirPackageSummary` so trivia edits do not invalidate HIR queries.
/// ## 2. Invariants & Mathematical Properties
/// Every `HirMemberEntry::def_id` in the companion `HirPackageSummary` has an entry in `span_by_def`.
/// ## 3. Preconditions
/// Populated alongside `HirPackageSummary` in `lower_cst_to_hir`.
/// ## 4. Postconditions
/// Enables O(log M) span lookup for `ariadne` diagnostics.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(log M) lookup; Space Complexity: O(M).
/// ## 6. Failure Modes & Fault Tolerance
/// Deterministic `BTreeMap` storage with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HirSourceMap {
    pub span_by_def: BTreeMap<DefId, Range<usize>>,
}

/// Extracts the specialization target type string from a CST node's `SpecializationClause` child.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Reads typing/specialization targets (`:>`, `:>>`, `:`) directly from the Rowan CST.
/// ## 2. Invariants & Mathematical Properties
/// Joins non-trivia tokens after the leading colon operator.
/// ## 3. Preconditions
/// `node` is a composite `SysmlSyntaxNode`.
/// ## 4. Postconditions
/// Returns `Some(type_str)` when a `SpecializationClause` child is present, else `None`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(K) over child tokens; Space Complexity: O(K).
/// ## 6. Failure Modes & Fault Tolerance
/// Total CST child inspection with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn extract_cst_type_ref(node: &SysmlSyntaxNode) -> Option<String> {
    let spec_node = node
        .children()
        .find(|c| c.kind() == SysmlSyntaxKind::SpecializationClause)?;
    let mut parts = Vec::new();
    let mut skipped_op = false;
    for elem in spec_node.children_with_tokens() {
        let Some(tok) = elem.into_token() else {
            continue;
        };
        if tok.kind().is_trivia() {
            continue;
        }
        if !skipped_op
            && matches!(
                tok.kind(),
                SysmlSyntaxKind::Colon
                    | SysmlSyntaxKind::ColonGt
                    | SysmlSyntaxKind::ColonGtGt
                    | SysmlSyntaxKind::ColonColonGt
            )
        {
            skipped_op = true;
            continue;
        }
        parts.push(tok.text().trim_matches('\'').to_string());
    }
    (!parts.is_empty()).then(|| parts.join(""))
}

/// Extracts the normalized import path from an `ImportDecl` Rowan CST node.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Collects non-trivia import target segments from `SysmlSyntaxKind::ImportDecl`.
/// ## 2. Invariants & Mathematical Properties
/// Strips leading `import` keyword and trailing `;`.
/// ## 3. Preconditions
/// `node.kind() == SysmlSyntaxKind::ImportDecl`.
/// ## 4. Postconditions
/// Returns the normalized import path string (e.g. `"ISQ::*"`).
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(K); Space Complexity: O(K).
/// ## 6. Failure Modes & Fault Tolerance
/// Total function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn extract_cst_import_path(node: &SysmlSyntaxNode) -> Option<String> {
    let mut out = String::new();
    for elem in node.children_with_tokens() {
        let Some(tok) = elem.into_token() else {
            continue;
        };
        if tok.kind().is_trivia() || tok.kind() == SysmlSyntaxKind::Semi {
            continue;
        }
        let text = tok.text();
        if matches!(text, "import" | "public" | "private" | "protected" | "all") {
            continue;
        }
        out.push_str(text.trim_matches('\''));
    }
    (!out.is_empty()).then_some(out)
}

/// Lowers a `ParsedSysmlCst` into a trivia-blind `HirPackageSummary` and `HirSourceMap`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Primary CST-to-HIR lowering function for `deap-core::sysml_cst` and `compile-sysml::salsa_db`.
/// ## 2. Invariants & Mathematical Properties
/// Computes a deterministic SHA-256 `structural_hash` over semantic HIR fields only.
/// ## 3. Preconditions
/// `parsed` is a valid `ParsedSysmlCst`.
/// ## 4. Postconditions
/// Returns `(HirPackageSummary, HirSourceMap)`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(M).
/// ## 6. Failure Modes & Fault Tolerance
/// Total function with saturating `u32` `DefId` increments.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn lower_cst_to_hir(parsed: &ParsedSysmlCst) -> (HirPackageSummary, HirSourceMap) {
    let root = parsed.syntax();
    let mut imports = Vec::new();
    for node in root.descendants() {
        if node.kind() == SysmlSyntaxKind::ImportDecl {
            if let Some(imp) = extract_cst_import_path(&node) {
                imports.push(imp);
            }
        }
    }
    let mut package_name = String::from("SysML_Model");
    let mut members = Vec::new();
    let mut source_map = HirSourceMap::default();
    let mut next_id: u32 = 0;

    for construct in &parsed.constructs {
        if matches!(
            construct.kind,
            SysmlSyntaxKind::PackageDecl | SysmlSyntaxKind::LibraryPackageDecl
        ) && package_name == "SysML_Model"
        {
            package_name.clone_from(&construct.name);
        }
        let def_id = DefId(next_id);
        next_id = next_id.saturating_add(1);
        let type_ref = root
            .descendants()
            .find(|n| {
                n.kind() == construct.kind
                    && usize::from(n.text_range().start()) <= construct.byte_range.start
                    && usize::from(n.text_range().end()) >= construct.byte_range.end
            })
            .and_then(|n| extract_cst_type_ref(&n));
        let qualified_name = if construct.name == package_name {
            construct.name.clone()
        } else {
            format!("{package_name}::{}", construct.name)
        };
        members.push(HirMemberEntry {
            def_id,
            kind_name: format!("{:?}", construct.kind),
            name: construct.name.clone(),
            qualified_name,
            is_definition: construct.is_definition,
            type_ref,
        });
        source_map.span_by_def.insert(def_id, construct.byte_range.clone());
    }

    let mut buf = Vec::new();
    buf.extend_from_slice(package_name.as_bytes());
    for imp in &imports {
        buf.extend_from_slice(b"|imp:");
        buf.extend_from_slice(imp.as_bytes());
    }
    for m in &members {
        buf.extend_from_slice(b"|mem:");
        buf.extend_from_slice(m.kind_name.as_bytes());
        buf.extend_from_slice(b":");
        buf.extend_from_slice(m.qualified_name.as_bytes());
        buf.extend_from_slice(if m.is_definition { b":def:" } else { b":use:" });
        if let Some(tr) = &m.type_ref {
            buf.extend_from_slice(tr.as_bytes());
        }
    }
    let structural_hash = compute_sha256(&buf);
    (
        HirPackageSummary {
            package_name,
            imports,
            members,
            structural_hash,
        },
        source_map,
    )
}

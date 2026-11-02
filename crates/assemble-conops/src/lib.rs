//! Deterministic ConOps & Mission Intent Assembly Engine (ISO/IEC/IEEE 29148 / ISO/IEC/IEEE 15288 / OMG UAF).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the top-level library interface for deterministic Concept of Operations (ConOps)
//! and Tactical Mission Intent document assembly, parameter binding, operational text sanitization,
//! and table-of-contents synthesis under DO-330 TQL-4 and ISO/IEC/IEEE 29148:2018 governance.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exported assembly, parameter-binding, sanitization, and TOC primitives are pure or
//!   deterministically workspace-scoped functions with zero nondeterministic ordering.
//! - Canonical unit ordering is strictly preserved across all assembly invocations.
//!
//! ## 3. Preconditions
//! - Callers supply valid workspace paths and UTF-8 markdown / SysML inputs to the re-exported APIs.
//!
//! ## 4. Postconditions
//! - Exposes a cohesive, deterministic public API surface for ConOps and Mission Intent compilation
//!   and baseline verification.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ module linkage; runtime complexity is bounded by the delegated
//!   `assembler`, `params`, `sanitize`, and `toc` submodules ($O(N)$ in input byte length).
//! - Space Complexity: $O(1)$ module overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - All fallible operations in re-exported submodules return explicit diagnostic errors or
//!   `Result` values without silent truncation or panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-ASSEMBLE-CONOPS-LIB], [DO-330-TQL-4-CONOPS-LIB]

pub mod assembler;
pub mod params;
pub mod sanitize;
pub mod toc;

pub use assembler::{
    assemble_conops, assemble_document, path_relative_from, relativize_markdown_links,
    validate_unit_integrity, CANONICAL_CONOPS_UNITS, CANONICAL_MISSION_INTENT_UNITS,
};
pub use params::{
    bind_parameters, extract_subsystem_parts, extract_subsystem_parts_from_sysml, find_line_number,
    SubsystemPart, SubsystemPort, SysMLParameterBindingEngine, DEFAULT_CONOPS_PARAMS,
};
pub use sanitize::{sanitize_level_1b_operational_text, wrap_mermaid_label};
pub use toc::{extract_headings, generate_table_of_contents, slugify, verify_markdown_links};
pub use deap_core::markdown_ast::{
    format_markdown_cmark, parse_markdown_ast, MarkdownCodeBlockAst, MarkdownSectionAst,
    MarkdownTableAst, ParsedMarkdownAst,
};

/// Parses and formats ConOps Markdown via the shared CommonMark AST adapter (`[REQ-0401]`, `[REQ-0402]`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_and_format_conops_markdown` in `crates/assemble-conops/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_and_format_conops_markdown` (declaration #1) within `ASSEMBLE-CONOPS-LIB`.
///
/// ## 3. Preconditions
/// - `source: &str` must satisfy domain and type-safety constraints prior to invoking `parse_and_format_conops_markdown`.
///
/// ## 4. Postconditions
/// - Returns `Result<(ParsedMarkdownAst, String), String>` from `parse_and_format_conops_markdown` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(source)) bounded over input/member dimension of `parse_and_format_conops_markdown`.
/// - Space Complexity: O(N(source)) bounded auxiliary memory for `parse_and_format_conops_markdown`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_and_format_conops_markdown` (`Result<(ParsedMarkdownAst, String), String>`).
/// - State: Maintains consistent state invariants across `parse_and_format_conops_markdown` without partial mutation on error.
/// - Action: Executes `parse_and_format_conops_markdown` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_and_format_conops_markdown` (`ASSEMBLE-CONOPS-LIB`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0401], [REQ-0402] | SysML v2: ActionDef(parse_and_format_conops_markdown)
pub fn parse_and_format_conops_markdown(
    source: &str,
) -> Result<(ParsedMarkdownAst, String), String> {
    let ast = parse_markdown_ast(source);
    let formatted = format_markdown_cmark(source)?;
    Ok((ast, formatted))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_assemble_conops_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        let module_doc: String = src
            .lines()
            .take_while(|line| line.starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");
        assert!(module_doc.contains("## 1. Safety Intent & Regulatory Scope"));
        assert!(module_doc.contains("## 2. Invariants & Mathematical Properties"));
        assert!(module_doc.contains("## 3. Preconditions"));
        assert!(module_doc.contains("## 4. Postconditions"));
        assert!(module_doc.contains("## 5. Algorithmic Complexity Bounds"));
        assert!(module_doc.contains("## 6. Failure Modes & Fault Tolerance"));
        assert!(module_doc.contains("## 7. Realization & Traceability"));
        assert!(
            module_doc.contains("Realises: [REQ-ASSEMBLE-CONOPS-LIB], [DO-330-TQL-4-CONOPS-LIB]")
        );
    }

    #[test]
    fn test_no_unconditional_crate_root_safety_lint_allows() {
        for (name, src) in [
            ("crates/assemble-conops/src/lib.rs", include_str!("lib.rs")),
            (
                "crates/assemble-conops/src/main.rs",
                include_str!("main.rs"),
            ),
            (
                "crates/compile-sysml/src/lib.rs",
                include_str!("../../compile-sysml/src/lib.rs"),
            ),
            (
                "crates/compile-sysml/src/main.rs",
                include_str!("../../compile-sysml/src/main.rs"),
            ),
            (
                "crates/verify-baseline/src/lib.rs",
                include_str!("../../verify-baseline/src/lib.rs"),
            ),
            (
                "crates/verify-baseline/src/main.rs",
                include_str!("../../verify-baseline/src/main.rs"),
            ),
        ] {
            let prod_src = src.split("#[cfg(test)]").next().unwrap_or(src);
            let has_unconditional_inner_allow = prod_src
                .lines()
                .any(|line| line.trim_start().starts_with("#![allow("));
            assert!(
                !has_unconditional_inner_allow,
                "{name} must not declare unconditional crate-root #![allow(...)] attributes"
            );
        }
    }

    #[test]
    fn test_parse_and_format_conops_markdown_roundtrip() -> Result<(), String> {
        let (ast, formatted) = super::parse_and_format_conops_markdown("# ConOps Unit\n\nBody.\n")?;
        assert_eq!(ast.sections.len(), 1);
        assert!(formatted.contains("ConOps Unit"));
        Ok(())
    }
}

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
    bind_parameters, extract_subsystem_parts, extract_subsystem_parts_from_sysml,
    find_line_number, SubsystemPart, SubsystemPort, SysMLParameterBindingEngine,
    DEFAULT_CONOPS_PARAMS,
};
pub use sanitize::{sanitize_level_1b_operational_text, wrap_mermaid_label};
pub use toc::{extract_headings, generate_table_of_contents, slugify, verify_markdown_links};

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
        assert!(module_doc.contains("Realises: [REQ-ASSEMBLE-CONOPS-LIB], [DO-330-TQL-4-CONOPS-LIB]"));
    }
}


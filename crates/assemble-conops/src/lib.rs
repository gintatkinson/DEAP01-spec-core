//! Deterministic ConOps & Mission Intent Assembly Engine (ISO/IEC/IEEE 29148 / ISO/IEC/IEEE 15288 / OMG UAF).

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

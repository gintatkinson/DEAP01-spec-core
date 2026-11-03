//! CommonMark AST-based ConOps & Mission Intent assembly, domain/metamodel conformance, and atomic emission (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

#[path = "cmark_aliases.rs"]
mod cmark_aliases;
#[path = "cmark_arch.rs"]
mod cmark_arch;
#[path = "cmark_document.rs"]
mod cmark_document;
#[path = "cmark_engine.rs"]
mod cmark_engine;
#[path = "cmark_ingest.rs"]
mod cmark_ingest;
#[path = "cmark_lifecycle.rs"]
mod cmark_lifecycle;
#[path = "cmark_links.rs"]
mod cmark_links;
#[path = "cmark_metamodel.rs"]
mod cmark_metamodel;
#[path = "cmark_physics.rs"]
mod cmark_physics;
#[path = "cmark_sanitize.rs"]
mod cmark_sanitize;
#[path = "cmark_schema.rs"]
mod cmark_schema;

#[cfg(test)]
#[path = "cmark_tests_a.rs"]
mod cmark_tests_a;
#[cfg(test)]
#[path = "cmark_tests_b.rs"]
mod cmark_tests_b;
#[cfg(test)]
#[path = "cmark_tests_c.rs"]
mod cmark_tests_c;

pub use cmark_document::assemble_document;
pub use cmark_engine::{
    bind_parameters, LifecycleType, SubsystemPart, SubsystemPort, SysMLParameterBindingEngine,
    DEFAULT_CONOPS_PARAMS,
};
pub use cmark_links::{
    find_raw_tokens, path_relative_from, relativize_markdown_links, validate_unit_integrity,
};
pub use cmark_metamodel::{
    allowed_envelopes, classify_domain_token, classify_performer_token, required_attributes,
    validate_node_envelopes, validate_part_envelopes, validate_process_layer, Allocation, Domain,
    Envelope, M2Type, MetamodelNode, PerformerKind, Performers, ProcessNode, ViewpointFamily,
};
pub use cmark_sanitize::{sanitize_level_1b_operational_text, wrap_mermaid_label};
pub use cmark_schema::{
    extract_subsystem_parts, extract_subsystem_parts_from_interface_schema,
    extract_subsystem_parts_from_json, extract_subsystem_parts_from_markdown,
    extract_subsystem_parts_from_sysml, extract_subsystem_parts_from_yaml, find_line_number,
    parse_subsystem_ports_from_body,
};

/// Canonical ConOps unit file whitelist (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub static CANONICAL_CONOPS_UNITS: &[&str] = &[
    "01_METADATA_AND_OVERVIEW.md",
    "02_DEFICIENCIES_AND_MOTIVATION.md",
    "03_PROPOSED_CAPABILITIES.md",
    "04_SYSTEM_ARCHITECTURE.md",
    "04_USER_CLASSES_AND_STAKEHOLDERS.md",
    "05_OPERATIONAL_STATE_SPACE_AND_RISK.md",
    "06_UAF_OPERATIONAL_ACTIVITIES.md",
    "07_OPTX_EXCHANGES.md",
    "08_SYSTEM_ARCHITECTURE.md",
    "08_ENVIRONMENTAL_OPERATING_LIMITS.md",
    "08_ENVIRONMENTAL_MIL_STD_810H.md",
    "09_SCENARIOS_AND_TIMELINES.md",
    "10_MAINTENANCE_AND_GSE_SUPPORT.md",
    "11_IMPACTS_AND_TRADE_STUDIES.md",
    "12_EMERGENCY_DECISION_MATRIX.md",
];

/// Canonical Mission Intent unit file whitelist (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub static CANONICAL_MISSION_INTENT_UNITS: &[&str] = &[
    "01_COMMANDERS_INTENT.md",
    "02_MISSION_ESSENTIAL_TASK_LIST.md",
    "03_INCOSE_MOE_MOP_MATH.md",
    "04_MULTI_DOMAIN_THREAT_MATRIX.md",
    "05_PACE_C2_PLAN.md",
    "06_SAFETY_INTERLOCKS.md",
    "06_ROE_SAFETY_INTERLOCKS.md",
    "06_RULES_OF_ENGAGEMENT.md",
    "07_AIRSPACE_GEOZONES.md",
    "08_GO_NO_GO_MATRIX.md",
    "09_ENERGY_AND_RESERVE_BOUNDS.md",
    "09_BINGO_ENERGY_MATH.md",
    "10_OPERATIONAL_ALLOCATION_TAGS.md",
];

/// [DOC-TIER: A] Error taxonomy for CommonMark AST ConOps assembly and atomic persistence.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0401] [FLS: fls_conops_cmark]
/// Contract: Captures empty unit Markdown, CommonMark serialization errors, and atomic filesystem I/O failures.
/// Bounds & Faults: O(1) error construction; implements `Display` and `std::error::Error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    EmptyUnitMarkdown { unit_id: String },
    MarkdownEmissionFailed(String),
    IoFailure(String),
}

impl fmt::Display for CompilerError {
    /// Formats `CompilerError` for diagnostic output (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUnitMarkdown { unit_id } => {
                write!(f, "ConOps unit '{unit_id}' contains empty Markdown body")
            }
            Self::MarkdownEmissionFailed(msg) => write!(f, "CommonMark emission failed: {msg}"),
            Self::IoFailure(msg) => write!(f, "I/O failure: {msg}"),
        }
    }
}

impl std::error::Error for CompilerError {}

/// [DOC-TIER: A] Unit document input record for `assemble_conops_master`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0401] [FLS: fls_conops_cmark]
/// Contract: Holds `unit_id` and raw `markdown_body` for AST-based ConOps master compilation.
/// Bounds & Faults: O(N) owned string storage; validated for non-emptiness prior to event stream assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConopsUnitDoc {
    pub unit_id: String,
    pub markdown_body: String,
}

/// [DOC-TIER: A] Normalizes CRLF and CR line endings to canonical LF (`\n`) with a single trailing newline.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0401] [FLS: fls_conops_cmark]
/// Contract: Replaces `\r\n` and `\r` with `\n` and ensures a single terminal `\n` when `input` is non-empty.
/// Bounds & Faults: O(N) time and space over `input.len()`; infallible string normalization.
pub fn normalize_line_endings(input: &str) -> String {
    let mut normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    while normalized.ends_with("\n\n") {
        normalized.pop();
    }
    if !normalized.ends_with('\n') && !normalized.is_empty() {
        normalized.push('\n');
    }
    normalized
}

/// [DOC-TIER: A] Emits normalized CommonMark text from a `pulldown_cmark::Event` stream via `pulldown_cmark_to_cmark`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0401] [FLS: fls_conops_cmark]
/// Contract: Serializes `events` into a deterministic CommonMark string with LF line endings.
/// Bounds & Faults: O(N) time and space over event stream; maps formatter errors to `CompilerError::MarkdownEmissionFailed`.
pub fn emit_lossless_markdown(events: Vec<Event<'_>>) -> Result<String, CompilerError> {
    let mut buf = String::with_capacity(4096);
    cmark(events.into_iter(), &mut buf, Options::all())
        .map_err(|e| CompilerError::MarkdownEmissionFailed(e.to_string()))?;
    Ok(normalize_line_endings(&buf))
}

/// [DOC-TIER: A] Assembles `ConopsUnitDoc` units into a master ConOps document via `pulldown-cmark` AST event streams.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0401] [FLS: fls_conops_cmark]
/// Contract: Parses each unit with `Options::all()`, shifts headings by +1 level, inserts `Event::Rule`, and emits CommonMark.
/// Bounds & Faults: O(N) time and space over aggregate unit byte length; rejects empty units with `CompilerError::EmptyUnitMarkdown`.
pub fn assemble_conops_master(units: &[ConopsUnitDoc]) -> Result<String, CompilerError> {
    let mut master_events: Vec<Event<'_>> = Vec::new();
    for (idx, unit) in units.iter().enumerate() {
        if unit.markdown_body.trim().is_empty() {
            return Err(CompilerError::EmptyUnitMarkdown {
                unit_id: unit.unit_id.clone(),
            });
        }
        if idx > 0 {
            master_events.push(Event::Rule);
        }
        let parser = Parser::new_ext(&unit.markdown_body, Options::all());
        for event in parser {
            let shifted = match event {
                Event::Start(Tag::Heading {
                    level,
                    id,
                    classes,
                    attrs,
                }) => Event::Start(Tag::Heading {
                    level: shift_heading_level(level),
                    id,
                    classes,
                    attrs,
                }),
                Event::End(TagEnd::Heading(level)) => {
                    Event::End(TagEnd::Heading(shift_heading_level(level)))
                }
                other => other,
            };
            master_events.push(shifted);
        }
    }
    emit_lossless_markdown(master_events)
}

/// Shifts a `HeadingLevel` down by one level (`H1` -> `H2`, clamping at `H6`) (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn shift_heading_level(level: HeadingLevel) -> HeadingLevel {
    match level {
        HeadingLevel::H1 => HeadingLevel::H2,
        HeadingLevel::H2 => HeadingLevel::H3,
        HeadingLevel::H3 => HeadingLevel::H4,
        HeadingLevel::H4 => HeadingLevel::H5,
        HeadingLevel::H5 | HeadingLevel::H6 => HeadingLevel::H6,
    }
}

/// [DOC-TIER: A] Writes a specification artifact atomically via a temporary sibling file and rename.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0401] [FLS: fls_conops_cmark]
/// Contract: Normalizes `content` to LF, writes and syncs `<target_path>.tmp`, and atomically renames to `target_path`.
/// Bounds & Faults: O(N) I/O cost over `content.len()`; returns `CompilerError::IoFailure` without partial target corruption.
pub fn write_spec_file_atomic(target_path: &Path, content: &str) -> Result<(), CompilerError> {
    let normalized = normalize_line_endings(content);
    let tmp_path = target_path.with_extension("tmp");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| CompilerError::IoFailure(e.to_string()))?;
    file.write_all(normalized.as_bytes())
        .map_err(|e| CompilerError::IoFailure(e.to_string()))?;
    file.sync_all()
        .map_err(|e| CompilerError::IoFailure(e.to_string()))?;
    fs::rename(&tmp_path, target_path).map_err(|e| CompilerError::IoFailure(e.to_string()))
}

/// [DOC-TIER: A] Orchestrates the assembly, parameter binding, and validation of `CONOPS.md` and `MISSION_INTENT.md`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-ASSEMBLE-CONOPS] [FLS: fls_conops_cmark]
/// Contract: Discovers ConOps and Mission Intent unit directories under `input_dir`, binds SysML parameters, and writes/verifies outputs in `output_dir`.
/// Bounds & Faults: O(U * N) time and space over unit files; returns `Ok(false)` when validation gates fail without partial output mutation.
pub fn assemble_conops(
    input_dir: &Path,
    output_dir: &Path,
    verify_only: bool,
    params: Option<&Path>,
    (domain, workspace_dir): (Option<&str>, Option<&Path>),
) -> Result<bool, Box<dyn std::error::Error>> {
    cmark_document::execute_conops_assembly(
        input_dir,
        output_dir,
        verify_only,
        params,
        (domain, workspace_dir),
    )
}

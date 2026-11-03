//! Specification Quality and Coverage Audit module conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Provides zero-copy streaming pull parsing of Markdown specifications, validation of
//! mandatory sections, metadata integrity verification, title uniqueness enforcement,
//! and filename convention auditing.

pub mod audit_gates;
pub mod audit_runner;
pub mod behavioral;
pub mod behavioral_matchers;
pub mod conops_architecture;
pub mod conops_audit;
pub mod conops_matrices;
pub mod conops_schema;
pub mod diagram_ast;
pub(crate) mod diagram_class_parser;
pub(crate) mod diagram_invariants;
pub(crate) mod diagram_parsers;
pub mod logical_ui;
pub mod logical_ui_platform;
pub mod markdown_ast;
pub mod markdown_placeholders;
pub mod metadata;
pub mod metadata_validators;
pub mod provenance;
pub mod provenance_grounding;
pub mod provenance_refs;
pub mod provenance_temporal;
pub mod sections;
pub mod sections_boilerplate;
pub(crate) mod uml_compliance_gates;
pub(crate) mod uml_coverage_gates;
pub(crate) mod uml_diagram_gates;
pub mod uml_validator;

pub use audit_runner::{collect_spec_files, load_sysml_symbol_index, run_spec_audit};
pub use behavioral::{
    collect_normalized_symbols, contains_token_with_word_boundary,
    extract_body_outside_code_blocks, load_behavioral_triggers, matches_target_type,
    normalize_symbol_name, parse_behavioral_triggers, spec_references_node,
    validate_behavioral_triggers, validate_behavioral_triggers_workspace, AuditedSpec,
    BehavioralRule, BehavioralTrigger,
};
pub use conops_audit::{
    check_conops_dir_exists, extract_markdown_tables, validate_conops,
    validate_conops_schema_contract, validate_conops_schema_file, validate_conops_workspace,
    validate_emergency_decision_matrix, validate_moe_mop_metrics,
    validate_operational_allocation_tags, validate_pace_plan, validate_subsystem_architecture,
    validate_super_system_architecture, validate_system_interface_topology, validate_threat_matrix,
    MarkdownTable, TableRow, ThreatDomainSpec, CANONICAL_EMERGENCY_TRIGGERS,
    MANDATORY_THREAT_DOMAINS, MOE_MOP_RECOGNIZED_UNITS, PACE_TIERS,
};
pub use diagram_ast::{
    detect_diagram_type, parse_mermaid_diagram, validate_mermaid_invariants, ClassAttributeAst,
    ClassDefAst, ClassDiagramAst, ClassOperationAst, ClassRelationAst, ClassRelationType,
    DiagramAst, DiagramType, ErDiagramAst, FlowEdgeAst, FlowNodeAst, FlowchartAst,
    GenericDiagramAst, SequenceActivationAst, SequenceDiagramAst, SequenceMessageAst,
    SequenceParticipantAst, StateDefAst, StateDiagramAst, StateNoteAst, StateTransitionAst,
    Visibility, XyChartAst,
};
pub use logical_ui::{
    detect_dom_leaks, detect_framework_leaks, detect_pixel_dimensions,
    find_pixel_dimensions_in_line, is_lumi_spec, validate_3layer_chain, validate_container_queries,
    validate_layout_guidelines, validate_logical_ui, validate_logical_ui_content,
    validate_platform_independence,
};
pub use markdown_ast::{
    find_unresolved_placeholders, is_metadata_header_table_row, parse_markdown, HeadingNode,
    MarkdownDoc, MermaidBlock, PlaceholderFinding, ProseSegment,
};
pub use metadata::{
    check_filename_anti_stutter, check_title_anti_stutter, has_concatenated_title_metadata,
    is_iso_date, is_semver, normalize_title, validate_filenames, validate_metadata,
    validate_title_uniqueness,
};
pub use provenance::{
    count_epic_features, count_feature_acceptance_criteria, extract_source_references_section,
    find_engineering_parameter, find_naked_numeric_quantities, is_assertion_grounded,
    is_recognized_unit, validate_cardinality_bounds, validate_factual_grounding,
    validate_factual_grounding_content, validate_provenance,
    validate_sequence_diagram_temporal_safety, validate_source_reference_clause_integrity,
    ENGINEERING_PARAMETER_TERMS, HITL_SENDER_TOKENS, NORMATIVE_STANDARDS,
    PHYSICAL_ARMING_TARGET_TOKENS, PROHIBITED_PLACEHOLDERS, RECOGNIZED_UNITS,
};
pub use sections::{
    validate_sections, validate_use_case_anti_boilerplate, SectionRule, SectionRules,
};
pub use uml_validator::{
    validate_class_diagram, validate_document_uml, validate_epic_statechart_heterogeneity,
    validate_feature_operation_coverage, validate_parent_epic_alignment, validate_sequence_diagram,
    validate_source_references, validate_state_diagram, validate_use_case_diagram_compliance,
    validate_user_story_test_cases, ClassifierSymbolInfo, SysmlSymbolIndex,
};

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Represents an individual finding or quality violation detected during specification audit.
/// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITFINDING]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecAuditFinding {
    /// Rule identifier (e.g. "behavioral-trigger-node-must-be-covered-by-a-specification").
    pub rule_id: String,
    /// Human-readable diagnostic description of the violation.
    pub message: String,
    /// Location identifier or relative file path of the audited entity.
    pub location: String,
    /// Optional 1-indexed line number in the source file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl SpecAuditFinding {
    /// Constructs a new specification audit finding.
    /// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITFINDING-NEW]
    pub fn new(
        rule_id: impl Into<String>,
        message: impl Into<String>,
        location: impl Into<String>,
        line: Option<usize>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            message: message.into(),
            location: location.into(),
            line,
        }
    }
}

impl fmt::Display for SpecAuditFinding {
    /// Formats a specification audit finding for human-readable output.
    /// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITFINDING-FMT]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(
                f,
                "{}:{}: [{}] {}",
                self.location, line, self.rule_id, self.message
            )
        } else if !self.location.is_empty() {
            write!(f, "{}: [{}] {}", self.location, self.rule_id, self.message)
        } else {
            write!(f, "[{}] {}", self.rule_id, self.message)
        }
    }
}

/// Formal error type for the specification audit pipeline.
/// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITERROR]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecAuditError {
    /// Markdown parsing or syntax failure.
    ParseError(String),
    /// File I/O failure.
    IoError(String),
    /// Configuration or rule ingestion failure.
    RuleError(String),
    /// Semantic or structure validation failure.
    ValidationError(String),
}

impl fmt::Display for SpecAuditError {
    /// Formats a specification audit error for display.
    /// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITERROR-FMT]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Markdown parse error: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::RuleError(msg) => write!(f, "Rule error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for SpecAuditError {}

impl From<std::io::Error> for SpecAuditError {
    /// Converts a standard I/O error into `SpecAuditError::IoError`.
    /// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITERROR-FROM]
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for SpecAuditError {
    /// Converts a JSON deserialization error into `SpecAuditError::RuleError`.
    /// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITERROR-FROM]
    fn from(err: serde_json::Error) -> Self {
        Self::RuleError(err.to_string())
    }
}

/// Configuration options governing execution of the specification audit suite.
/// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITOPTIONS]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecAuditOptions {
    /// Target destination directory or repository root.
    pub repo_root: PathBuf,
    /// Optional single specification markdown file to audit in isolation.
    pub only_file: Option<PathBuf>,
    /// Optional filter restricting audit to a single gate (e.g. "uml", "sections", "metadata").
    pub gate_filter: Option<String>,
    /// When true, allows empty specification landing zones without failing the audit.
    pub allow_missing_specs: bool,
    /// When true, strictly fails when specifications or gates report issues.
    pub strict: bool,
}

/// Consolidated audit report detailing executed gates, findings, and timing.
/// Realises: [REQ-SPEC-AUDIT-MOD-SPECAUDITSUMMARY]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecAuditSummary {
    /// Total count of specification Markdown files audited.
    pub total_files_audited: usize,
    /// Total count of quality violations detected across all active gates.
    pub total_findings: usize,
    /// Ordered list of all detected findings.
    pub findings: Vec<SpecAuditFinding>,
    /// Total audit elapsed duration in milliseconds.
    pub duration_ms: u128,
    /// Overall pass status (true if zero findings detected).
    pub passed: bool,
}

/// Active gate bitflags parsed from `SpecAuditOptions::gate_filter`.
/// Realises: [REQ-SPEC-AUDIT-MOD-ACTIVEGATES]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveGates {
    pub(crate) uml: bool,
    pub(crate) sections: bool,
    pub(crate) metadata: bool,
    pub(crate) behavioral: bool,
    pub(crate) logical_ui: bool,
    pub(crate) provenance: bool,
    pub(crate) conops: bool,
}

impl ActiveGates {
    /// Parses an optional gate filter string into `ActiveGates`.
    /// Realises: [REQ-SPEC-AUDIT-MOD-ACTIVEGATES-PARSE]
    pub(crate) fn parse(filter: Option<&str>) -> Result<Self, SpecAuditError> {
        let f = filter.map(|s| s.trim().to_lowercase()).unwrap_or_default();
        if f.is_empty() || f == "all" {
            return Ok(Self {
                uml: true,
                sections: true,
                metadata: true,
                behavioral: true,
                logical_ui: true,
                provenance: true,
                conops: true,
            });
        }

        let mut gates = Self {
            uml: false,
            sections: false,
            metadata: false,
            behavioral: false,
            logical_ui: false,
            provenance: false,
            conops: false,
        };
        match f.as_str() {
            "uml" => gates.uml = true,
            "sections" => gates.sections = true,
            "metadata" => gates.metadata = true,
            "behavioral" => gates.behavioral = true,
            "logical_ui" | "logical-ui" | "lui" => gates.logical_ui = true,
            "provenance" => gates.provenance = true,
            "conops" => gates.conops = true,
            unknown => {
                return Err(SpecAuditError::RuleError(format!(
                    "Unknown audit gate filter '{}'. Supported gates: all, uml, sections, metadata, behavioral, logical_ui, provenance, conops",
                    unknown
                )));
            }
        }
        Ok(gates)
    }
}

/// Infers specification category (epic, feature, user_story, use_case) from relative path.
/// Realises: [REQ-SPEC-AUDIT-MOD-INFER-SPEC-TYPE]
pub fn infer_spec_type(rel_path: &str) -> &'static str {
    let lower = rel_path.to_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or(&lower);

    if lower.contains("docs/epics") || file_name.starts_with("epic-") {
        "epic"
    } else if lower.contains("docs/features") || file_name.starts_with("feat-") {
        "feature"
    } else if lower.contains("docs/user-stories")
        || lower.contains("user_stories")
        || file_name.starts_with("us-")
    {
        "user_story"
    } else if lower.contains("docs/use-cases")
        || lower.contains("use_cases")
        || file_name.starts_with("uc-")
    {
        "use_case"
    } else {
        "generic"
    }
}

#[cfg(test)]
#[path = "spec_audit_tests.rs"]
mod tests;

//! Specification Quality and Coverage Audit module conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Provides zero-copy streaming pull parsing of Markdown specifications, validation of
//! mandatory sections, metadata integrity verification, title uniqueness enforcement,
//! and filename convention auditing.

pub mod behavioral;
pub mod conops_audit;
pub mod diagram_ast;
pub mod logical_ui;
pub mod markdown_ast;
pub mod metadata;
pub mod provenance;
pub mod sections;
pub mod uml_validator;

pub use behavioral::{
    collect_normalized_symbols, contains_token_with_word_boundary,
    extract_body_outside_code_blocks, load_behavioral_triggers, matches_target_type,
    normalize_symbol_name, parse_behavioral_triggers, spec_references_node,
    validate_behavioral_triggers, validate_behavioral_triggers_workspace, AuditedSpec,
    BehavioralRule, BehavioralTrigger,
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
    find_pixel_dimensions_in_line, is_lumi_spec, validate_3layer_chain,
    validate_container_queries, validate_layout_guidelines, validate_logical_ui,
    validate_logical_ui_content, validate_platform_independence,
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
pub use sections::{
    validate_sections, validate_use_case_anti_boilerplate, SectionRule, SectionRules,
};
pub use uml_validator::{
    validate_class_diagram, validate_document_uml, validate_epic_statechart_heterogeneity,
    validate_parent_epic_alignment, validate_sequence_diagram, validate_source_references,
    validate_state_diagram, validate_use_case_diagram_compliance, ClassifierSymbolInfo,
    SysmlSymbolIndex,
};
pub use conops_audit::{
    check_conops_dir_exists, extract_markdown_tables, validate_conops, validate_conops_workspace,
    validate_emergency_decision_matrix, validate_moe_mop_metrics,
    validate_operational_allocation_tags, validate_pace_plan, validate_threat_matrix,
    MarkdownTable, TableRow, ThreatDomainSpec, CANONICAL_EMERGENCY_TRIGGERS,
    MANDATORY_THREAT_DOMAINS, MOE_MOP_RECOGNIZED_UNITS, PACE_TIERS,
};
pub use provenance::{
    count_epic_features, count_feature_acceptance_criteria, extract_source_references_section,
    find_engineering_parameter, find_naked_numeric_quantities, is_assertion_grounded,
    is_recognized_unit, validate_cardinality_bounds, validate_factual_grounding,
    validate_factual_grounding_content, validate_provenance,
    validate_source_reference_clause_integrity, ENGINEERING_PARAMETER_TERMS, NORMATIVE_STANDARDS,
    PROHIBITED_PLACEHOLDERS, RECOGNIZED_UNITS,
};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an individual finding or quality violation detected during specification audit.
///
/// Preconditions: None.
/// Postconditions: Formats clean diagnostic messages with zero panics.
/// Algorithmic Complexity: O(1) construction and formatting.
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "{}:{}: [{}] {}", self.location, line, self.rule_id, self.message)
        } else if !self.location.is_empty() {
            write!(f, "{}: [{}] {}", self.location, self.rule_id, self.message)
        } else {
            write!(f, "[{}] {}", self.rule_id, self.message)
        }
    }
}


/// Formal error type for the specification audit pipeline.
///
/// Preconditions: None.
/// Postconditions: Formats clean error messages with zero panics.
/// Algorithmic Complexity: O(1) construction and formatting.
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
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for SpecAuditError {
    fn from(err: serde_json::Error) -> Self {
        Self::RuleError(err.to_string())
    }
}

use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Configuration options governing execution of the specification audit suite.
///
/// Preconditions: `repo_root` exists on disk.
/// Postconditions: Fully specifies all audit parameters and filter criteria.
/// Algorithmic Complexity: O(1) configuration initialization.
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
///
/// Preconditions: None.
/// Postconditions: Contains non-negative file count, findings list, duration, and pass status.
/// Algorithmic Complexity: O(1) construction.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveGates {
    uml: bool,
    sections: bool,
    metadata: bool,
    behavioral: bool,
    logical_ui: bool,
    provenance: bool,
    conops: bool,
}

impl ActiveGates {
    fn parse(filter: Option<&str>) -> Result<Self, SpecAuditError> {
        let f = match filter {
            Some(s) => s.trim().to_lowercase(),
            None => String::new(),
        };

        if f.is_empty() || f == "all" {
            Ok(Self {
                uml: true,
                sections: true,
                metadata: true,
                behavioral: true,
                logical_ui: true,
                provenance: true,
                conops: true,
            })
        } else {
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
}

/// Infers specification category (epic, feature, user_story, use_case) from relative path.
///
/// Preconditions: `rel_path` is a normalized relative path.
/// Postconditions: Returns canonical specification type string.
/// Algorithmic Complexity: O(L) where L is path string length.
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

/// Pre-indexes SysML v2 models from schema files into O(1) symbol lookup tables.
///
/// Preconditions: `repo_root` exists on disk.
/// Postconditions: Returns initialized SysmlSymbolIndex, or empty index if no schemas present.
/// Algorithmic Complexity: O(S + V + E) where S is schema byte length and V, E are AST entities.
pub fn load_sysml_symbol_index(repo_root: &Path) -> Result<SysmlSymbolIndex, SpecAuditError> {
    let schema_dir = repo_root.join("schema");
    let mut sysml_files = Vec::new();

    if schema_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&schema_dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let p = entry.path();
                if p.extension().map_or(false, |ext| ext == "sysml") {
                    sysml_files.push(p.to_path_buf());
                }
            }
        }
    }
    sysml_files.sort();

    let mut combined_content = String::new();
    for f in &sysml_files {
        let text = fs::read_to_string(f).map_err(|e| {
            SpecAuditError::IoError(format!(
                "Failed to read schema file at '{}': {}",
                f.display(),
                e
            ))
        })?;
        combined_content.push_str(&text);
        combined_content.push_str("\n\n");
    }

    if combined_content.trim().is_empty() {
        let pipeline_schema = repo_root.join(".pipeline").join("schema.sysml");
        if pipeline_schema.is_file() {
            let text = fs::read_to_string(&pipeline_schema).map_err(|e| {
                SpecAuditError::IoError(format!(
                    "Failed to read schema file at '{}': {}",
                    pipeline_schema.display(),
                    e
                ))
            })?;
            combined_content = text;
        }
    }

    if combined_content.trim().is_empty() {
        return Ok(SysmlSymbolIndex::new());
    }

    let parts = deap_core::sysml_ast::parse_part_defs(&combined_content);
    let ports = deap_core::sysml_ast::parse_port_defs(&combined_content);
    let actions = deap_core::sysml_ast::parse_action_defs(&combined_content);
    let constraints = deap_core::sysml_ast::parse_constraint_defs(&combined_content);
    let requirements = deap_core::sysml_ast::parse_requirement_defs(&combined_content);
    let attributes = deap_core::sysml_ast::parse_attribute_defs(&combined_content);
    let top_states = deap_core::sysml_ast::parse_state_defs(&combined_content);

    let model = deap_core::sysml_ast::SysmlModel {
        package_name: None,
        doc: None,
        attributes,
        parts,
        ports,
        actions,
        constraints,
        requirements,
    };

    let mut index = SysmlSymbolIndex::from_model(&model);
    for st in &top_states {
        index.index_state(st);
    }

    Ok(index)
}

/// Discovers all specification Markdown files targetable by the audit suite.
///
/// Preconditions: `repo_root` exists on disk.
/// Postconditions: Returns deterministic sorted list of file paths.
/// Algorithmic Complexity: O(D) where D is directory entry count.
pub fn collect_spec_files(
    repo_root: &Path,
    only_file: Option<&Path>,
) -> Result<Vec<PathBuf>, SpecAuditError> {
    if let Some(single_path) = only_file {
        let path = if single_path.is_absolute() {
            single_path.to_path_buf()
        } else {
            repo_root.join(single_path)
        };
        if !path.is_file() {
            return Err(SpecAuditError::IoError(format!(
                "Target specification file not found at: {}",
                path.display()
            )));
        }
        return Ok(vec![path]);
    }

    let target_dirs = [
        repo_root.join("docs").join("epics"),
        repo_root.join("docs").join("features"),
        repo_root.join("docs").join("user-stories"),
        repo_root.join("docs").join("use-cases"),
    ];

    let mut files = Vec::new();
    for dir in &target_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let p = entry.path();
                let fname = match p.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };
                if fname.ends_with(".md") && fname != "README.md" && fname != ".gitkeep" {
                    files.push(p.to_path_buf());
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Primary orchestrator for the specification quality and coverage audit suite.
///
/// Conforms to DO-178C Level A and ISO 26262 ASIL D verification principles.
/// Multi-core data parallelism via Rayon streams through all target specifications,
/// validating UML models, mandatory sections, metadata, behavioral triggers, LUMI,
/// provenance, and ConOps integrity with strictly zero regex and zero panic.
///
/// Preconditions: `options` specifies accessible workspace root.
/// Postconditions: Returns Ok(SpecAuditSummary) detailing pass status and findings.
/// Algorithmic Complexity: O((F * N) / P + CrossGates) where F is file count, N is size, and P is CPU cores.
pub fn run_spec_audit(options: &SpecAuditOptions) -> Result<SpecAuditSummary, SpecAuditError> {
    let start_time = Instant::now();
    let active_gates = ActiveGates::parse(options.gate_filter.as_deref())?;

    // 1. Load SysmlSymbolIndex once into memory
    let symbol_index = load_sysml_symbol_index(&options.repo_root)?;

    // 2. Load behavioral_triggers.json once into memory
    let triggers_path = options.repo_root.join("rules").join("behavioral_triggers.json");
    let triggers = if triggers_path.is_file() {
        load_behavioral_triggers(&triggers_path)?
    } else if options.allow_missing_specs || !active_gates.behavioral {
        Vec::new()
    } else {
        return Err(SpecAuditError::IoError(format!(
            "Behavioral triggers configuration not found at: {}",
            triggers_path.display()
        )));
    };

    // 3. Collect target specification files
    let files = collect_spec_files(&options.repo_root, options.only_file.as_deref())?;

    // 4. Handle empty specification set gracefully if allowed
    if files.is_empty() {
        let duration_ms = start_time.elapsed().as_millis();
        if options.allow_missing_specs {
            return Ok(SpecAuditSummary {
                total_files_audited: 0,
                total_findings: 0,
                findings: Vec::new(),
                duration_ms,
                passed: true,
            });
        } else {
            let finding = SpecAuditFinding::new(
                "specification-files-present",
                "No specification markdown files found under docs/{epics,features,user-stories,use-cases}",
                "docs",
                None,
            );
            return Ok(SpecAuditSummary {
                total_files_audited: 0,
                total_findings: 1,
                findings: vec![finding],
                duration_ms,
                passed: false,
            });
        }
    }

    // 5. Data-parallel traversal using Rayon
    struct AuditedFileOutcome {
        rel_path: String,
        spec_type: String,
        content: String,
        doc: MarkdownDoc,
        findings: Vec<SpecAuditFinding>,
    }

    let file_outcomes: Result<Vec<AuditedFileOutcome>, SpecAuditError> = files
        .par_iter()
        .map(|file_path| -> Result<AuditedFileOutcome, SpecAuditError> {
            let content = fs::read_to_string(file_path).map_err(|e| {
                SpecAuditError::IoError(format!(
                    "Failed to read specification file at '{}': {}",
                    file_path.display(),
                    e
                ))
            })?;

            let rel_path = file_path
                .strip_prefix(&options.repo_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file_path.to_string_lossy().to_string())
                .replace('\\', "/");

            let doc = parse_markdown(&content)?;
            let spec_type = infer_spec_type(&rel_path).to_string();

            let mut findings = Vec::new();

            // Sections gate
            if active_gates.sections {
                let section_errors = validate_sections(&spec_type, &doc, &rel_path);
                for err in section_errors {
                    findings.push(SpecAuditFinding::new(
                        "mandatory-sections-missing",
                        err,
                        &rel_path,
                        None,
                    ));
                }

                if spec_type == "use_case" {
                    let boilerplate_errors = validate_use_case_anti_boilerplate(&doc, &content, &rel_path);
                    for err in boilerplate_errors {
                        findings.push(SpecAuditFinding::new(
                            "anti-boilerplate-gate",
                            err,
                            &rel_path,
                            None,
                        ));
                    }
                }
            }

            // Metadata gate
            if active_gates.metadata {
                let meta_errors = validate_metadata(&spec_type, &doc, &content, &rel_path);
                for err in meta_errors {
                    let rule_id = if err.contains("stuttering prefix") || err.contains("repeating title prefix") {
                        "anti-stutter-gate"
                    } else {
                        "specification-metadata-integrity"
                    };
                    findings.push(SpecAuditFinding::new(
                        rule_id,
                        err,
                        &rel_path,
                        None,
                    ));
                }
                for ph in &doc.unresolved_placeholders {
                    findings.push(SpecAuditFinding::new(
                        "unresolved-placeholder",
                        format!("{}: {}", ph.label, ph.text),
                        &rel_path,
                        Some(ph.line),
                    ));
                }
            }

            // UML model integrity gate
            if active_gates.uml {
                if let Err(uml_errs) = validate_document_uml(&doc, &symbol_index) {
                    for err in uml_errs {
                        findings.push(SpecAuditFinding::new(
                            "uml-model-integrity",
                            err.to_string(),
                            &rel_path,
                            None,
                        ));
                    }
                }

                if spec_type == "use_case" {
                    let uc_diag_errors = validate_use_case_diagram_compliance(&doc, &rel_path);
                    for err in uc_diag_errors {
                        findings.push(SpecAuditFinding::new(
                            "use-case-diagram-compliance",
                            err,
                            &rel_path,
                            None,
                        ));
                    }
                }
            }

            // Logical UI and platform independence gate
            if active_gates.logical_ui {
                let lui_findings = validate_logical_ui(&doc, &content, &rel_path)?;
                findings.extend(lui_findings);
            }

            // Provenance and factual grounding gate
            if active_gates.provenance {
                let prov_findings =
                    validate_provenance(&doc, &content, &rel_path, Some(&symbol_index))?;
                findings.extend(prov_findings);
            }

            // ConOps gate
            if active_gates.conops {
                let conops_findings = validate_conops(&doc, &content, &rel_path)?;
                findings.extend(conops_findings);
            }

            Ok(AuditedFileOutcome {
                rel_path,
                spec_type,
                content,
                doc,
                findings,
            })
        })
        .collect();

    let file_outcomes = file_outcomes?;

    // 6. Aggregate per-file findings and execute cross-document workspace gates
    let mut all_findings = Vec::new();
    for outcome in &file_outcomes {
        all_findings.extend(outcome.findings.clone());
    }

    if options.only_file.is_none() {
        // Cross-document: Title uniqueness
        if active_gates.metadata {
            let title_tuples: Vec<(String, String, MarkdownDoc)> = file_outcomes
                .iter()
                .map(|o| (o.spec_type.clone(), o.rel_path.clone(), o.doc.clone()))
                .collect();
            let title_errors = validate_title_uniqueness(&title_tuples);
            for err in title_errors {
                all_findings.push(SpecAuditFinding::new(
                    "spec-title-uniqueness",
                    err,
                    "docs",
                    None,
                ));
            }

            // Cross-document: Filename conventions
            let mut grouped_by_dir: std::collections::HashMap<String, (String, Vec<String>)> =
                std::collections::HashMap::new();
            for o in &file_outcomes {
                let dir = match o.rel_path.rfind('/') {
                    Some(idx) => &o.rel_path[..idx],
                    None => "docs",
                };
                let fname = match o.rel_path.rfind('/') {
                    Some(idx) => &o.rel_path[idx + 1..],
                    None => &o.rel_path,
                };
                let entry = grouped_by_dir
                    .entry(dir.to_string())
                    .or_insert_with(|| (o.spec_type.clone(), Vec::new()));
                entry.1.push(fname.to_string());
            }

            for (dir_rel, (spec_type, fnames)) in &grouped_by_dir {
                let fname_slices: Vec<&str> = fnames.iter().map(|s| s.as_str()).collect();
                let fn_errors = validate_filenames(spec_type, &fname_slices, dir_rel);
                for err in fn_errors {
                    let rule_id = if err.contains("stuttering prefix") {
                        "anti-stutter-gate"
                    } else {
                        "spec-filename-convention"
                    };
                    all_findings.push(SpecAuditFinding::new(
                        rule_id,
                        err,
                        dir_rel,
                        None,
                    ));
                }
            }
        }

        let epics: Vec<(&str, &MarkdownDoc)> = file_outcomes
            .iter()
            .filter(|o| o.spec_type == "epic")
            .map(|o| (o.rel_path.as_str(), &o.doc))
            .collect();

        let features: Vec<(&str, &MarkdownDoc)> = file_outcomes
            .iter()
            .filter(|o| o.spec_type == "feature")
            .map(|o| (o.rel_path.as_str(), &o.doc))
            .collect();

        let stories: Vec<(&str, &MarkdownDoc)> = file_outcomes
            .iter()
            .filter(|o| o.spec_type == "user_story")
            .map(|o| (o.rel_path.as_str(), &o.doc))
            .collect();

        // Cross-document: Epic Statechart Heterogeneity Gate
        if active_gates.uml {
            let epic_het_errors = validate_epic_statechart_heterogeneity(&epics);
            for err in epic_het_errors {
                all_findings.push(SpecAuditFinding::new(
                    "epic-statechart-heterogeneity",
                    err,
                    "docs/epics",
                    None,
                ));
            }
        }

        // Cross-document: Parent Epic Alignment Gate
        if active_gates.metadata || active_gates.sections {
            let parent_align_errors = validate_parent_epic_alignment(&stories, &features, &epics);
            for err in parent_align_errors {
                all_findings.push(SpecAuditFinding::new(
                    "parent-epic-alignment",
                    err,
                    "docs/user-stories",
                    None,
                ));
            }
        }

        // Cross-document: Behavioral triggers
        if active_gates.behavioral {
            let audited_specs: Vec<AuditedSpec> = file_outcomes
                .iter()
                .map(|o| AuditedSpec {
                    rel_path: o.rel_path.clone(),
                    spec_type: o.spec_type.clone(),
                    doc: o.doc.clone(),
                    content: o.content.clone(),
                })
                .collect();
            let b_findings =
                validate_behavioral_triggers(&triggers, &symbol_index, &audited_specs)?;
            all_findings.extend(b_findings);
        }

        // Cross-document: ConOps workspace
        if active_gates.conops {
            let c_findings = validate_conops_workspace(&options.repo_root)?;
            all_findings.extend(c_findings);
        }
    }

    let duration_ms = start_time.elapsed().as_millis();
    let total_files_audited = file_outcomes.len();
    let total_findings = all_findings.len();
    let passed = total_findings == 0;

    if !all_findings.is_empty() {
        println!("Specification Audit Findings ({} total):", total_findings);
        for finding in &all_findings {
            if let Some(line) = finding.line {
                println!(
                    "  {}:{}: [{}] {}",
                    finding.location, line, finding.rule_id, finding.message
                );
            } else {
                println!(
                    "  {}: [{}] {}",
                    finding.location, finding.rule_id, finding.message
                );
            }
        }
    }

    println!(
        "Spec audit completed in {} ms ({} files audited, {} findings, passed: {})",
        duration_ms, total_files_audited, total_findings, passed
    );

    Ok(SpecAuditSummary {
        total_files_audited,
        total_findings,
        findings: all_findings,
        duration_ms,
        passed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_spec_type() {
        assert_eq!(infer_spec_type("docs/epics/epic-01-sysml.md"), "epic");
        assert_eq!(infer_spec_type("docs/features/feat-01-human.md"), "feature");
        assert_eq!(infer_spec_type("docs/user-stories/us-01-telemetry.md"), "user_story");
        assert_eq!(infer_spec_type("docs/use-cases/uc-01-mission.md"), "use_case");
        assert_eq!(infer_spec_type("feat-02-foo.md"), "feature");
        assert_eq!(infer_spec_type("other/notes.md"), "generic");
    }

    #[test]
    fn test_active_gates_filter() {
        let all = ActiveGates::parse(None).unwrap();
        assert!(all.uml && all.sections && all.metadata && all.behavioral);

        let uml = ActiveGates::parse(Some("uml")).unwrap();
        assert!(uml.uml && !uml.sections && !uml.metadata);

        let sections = ActiveGates::parse(Some("sections")).unwrap();
        assert!(sections.sections && !sections.uml);

        let invalid = ActiveGates::parse(Some("invalid-gate"));
        assert!(invalid.is_err());
    }

    #[test]
    fn test_empty_workspace_allow_missing() {
        let temp_dir = std::env::temp_dir().join(format!("spec_audit_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let options = SpecAuditOptions {
            repo_root: temp_dir.clone(),
            only_file: None,
            gate_filter: None,
            allow_missing_specs: true,
            strict: false,
        };

        let summary = run_spec_audit(&options).unwrap();
        assert_eq!(summary.total_files_audited, 0);
        assert_eq!(summary.total_findings, 0);
        assert!(summary.passed);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_empty_workspace_disallow_missing() {
        let temp_dir = std::env::temp_dir().join(format!("spec_audit_test_strict_{}", std::process::id()));
        let rules_dir = temp_dir.join("rules");
        let _ = fs::create_dir_all(&rules_dir);
        let _ = fs::write(rules_dir.join("behavioral_triggers.json"), "[]");

        let options = SpecAuditOptions {
            repo_root: temp_dir.clone(),
            only_file: None,
            gate_filter: None,
            allow_missing_specs: false,
            strict: true,
        };

        let summary = run_spec_audit(&options).unwrap();
        assert_eq!(summary.total_files_audited, 0);
        assert_eq!(summary.total_findings, 1);
        assert!(!summary.passed);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_gate_anti_stutter_detection() {
        assert!(check_title_anti_stutter("Use Case 01: UC 01 System Vision").is_some());
        assert!(check_title_anti_stutter("UC_XX_Workflow").is_some());
        assert!(check_title_anti_stutter("Use Case 01: System Vision and Foundational Invariants").is_none());

        assert!(check_filename_anti_stutter("UC-01-UC_01_System_Vision.md").is_some());
        assert!(check_filename_anti_stutter("UC-01-System_Vision_and_Foundational_Invariants.md").is_none());
    }

    #[test]
    fn test_gate_use_case_anti_boilerplate() {
        let bad_content = r#"| Metadata Field | Value |
|---|---|
| Title | Use Case 01: System Vision |
| Type | use_case |

## Main Success Scenario
1. Step one.
2. Step two.

## Alternate and Exception Flows
5a. Condition fails.
"#;
        let bad_doc = parse_markdown(bad_content).unwrap();
        let errors = validate_use_case_anti_boilerplate(&bad_doc, bad_content, "docs/use-cases/uc-01.md");
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("fewer than 5 steps")));
        assert!(errors.iter().any(|e| e.contains("fewer than 3 exception cases")));
    }

    #[test]
    fn test_gate_use_case_diagram_compliance() {
        let bad_diagram = r#"| Metadata Field | Value |
|---|---|
| Title | Use Case 01: System Vision |
| Type | use_case |

```mermaid
flowchart LR
    Actor --> Action
```
"#;
        let doc = parse_markdown(bad_diagram).unwrap();
        let errors = validate_use_case_diagram_compliance(&doc, "docs/use-cases/uc-01.md");
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("lacks a system boundary subgraph")));
    }

    #[test]
    fn test_gate_epic_statechart_heterogeneity() {
        let epic1_content = r#"```mermaid
stateDiagram-v2
    [*] --> Bootstrapping
    Bootstrapping --> Ingesting
    Ingesting --> Compiling
    Compiling --> Verifying
    Verifying --> [*]
```"#;
        let epic2_content = r#"```mermaid
stateDiagram-v2
    [*] --> Bootstrapping
    Bootstrapping --> Ingesting
    Ingesting --> Compiling
    Compiling --> Verifying
    Verifying --> [*]
```"#;
        let doc1 = parse_markdown(epic1_content).unwrap();
        let doc2 = parse_markdown(epic2_content).unwrap();
        let epics = vec![
            ("docs/epics/epic-01.md", &doc1),
            ("docs/epics/epic-02.md", &doc2),
        ];
        let errors = validate_epic_statechart_heterogeneity(&epics);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("share identical canned pipeline lifecycle statecharts")));
    }

    #[test]
    fn test_gate_parent_epic_alignment() {
        let story_content = r#"| Metadata Field | Value |
|---|---|
| Title | US 01: Invariant Checks |
| Type | user_story |
| Parent Epic | EPIC-02-ConOps |

Links to [FEAT-06-SystemVisionEngine.md](../features/feat-06.md).
"#;
        let feat_content = r#"| Metadata Field | Value |
|---|---|
| Title | Feature 06: [System Vision] SystemVisionEngine |
| Type | feature |
| Parent Epic | EPIC-03-SystemVision |
"#;
        let epic_content = r#"| Metadata Field | Value |
|---|---|
| Title | Epic 03: System Vision |
| Type | epic |
"#;
        let s_doc = parse_markdown(story_content).unwrap();
        let f_doc = parse_markdown(feat_content).unwrap();
        let e_doc = parse_markdown(epic_content).unwrap();

        let stories = vec![("docs/user-stories/us-01.md", &s_doc)];
        let features = vec![("docs/features/feat-06.md", &f_doc)];
        let epics = vec![("docs/epics/epic-03.md", &e_doc)];

        let errors = validate_parent_epic_alignment(&stories, &features, &epics);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Parent Epic Alignment Gate violation")));
    }
}

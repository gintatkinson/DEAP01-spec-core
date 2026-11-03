#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Metamodel component structs and source context for `verify-baseline audit-code` (`BP-06`).
//! Realises: [REQ-0606-AUDIT-CODE-MODULE]

use crate::audit_code::{
    run_audit_code, synthesize_defect_dossier_markdown, synthesize_section_4_uml, AuditCodeArgs,
    AuditFinding, AuditPillar,
};
use crate::audit_code_pillars_a::{audit_pillar_memory_safety, audit_pillar_resource_lifecycle};
use crate::audit_code_pillars_b::{audit_pillar_concurrency, audit_pillar_test_integrity};
use crate::audit_code_pillars_c::audit_pillar_semantic_traceability;
use crate::audit_code_scanner::{
    compute_rust_non_test_mask, mask_source_lines, resolve_canonical_test_target,
};
use crate::defect_dossier::{
    run_check_defect, validate_defect_dossier_scoped, DefectDossierArgs, DefectSchemaValidation,
};
use deap_core::rust_cst::source_bridge::{
    parse_dart_source_ast, parse_rust_source_ast, SourceAstItem, SourceLanguage,
};
use std::path::Path;

/// Per-file precomputed lexical and AST context passed to the 5 pillar evaluators.
/// Realises: [REQ-0606-EVALUATE-SOURCE-FILE-PILLARS]
pub struct SourceFileAuditCtx<'a> {
    pub rel_path: &'a str,
    pub raw_lines: Vec<&'a str>,
    pub masked_lines: Vec<String>,
    pub comment_stripped_lines: Vec<String>,
    pub non_test_mask: Vec<bool>,
    pub lang: SourceLanguage,
    pub ast_items: &'a [SourceAstItem],
    pub test_target: String,
}

impl<'a> SourceFileAuditCtx<'a> {
    /// Builds a `SourceFileAuditCtx` for `rel_path` and `source_text`.
    /// Realises: [REQ-0606-EVALUATE-SOURCE-FILE-PILLARS]
    pub fn new(
        workspace_root: &Path,
        rel_path: &'a str,
        source_text: &'a str,
        lang: SourceLanguage,
        ast_items: &'a [SourceAstItem],
    ) -> Self {
        let raw_lines: Vec<&str> = source_text.lines().collect();
        let masked_lines = mask_source_lines(source_text, true);
        let comment_stripped_lines = mask_source_lines(source_text, false);
        let non_test_mask = if lang == SourceLanguage::Rust {
            compute_rust_non_test_mask(&masked_lines)
        } else {
            vec![true; masked_lines.len()]
        };
        let test_target = resolve_canonical_test_target(workspace_root, rel_path);
        Self {
            rel_path,
            raw_lines,
            masked_lines,
            comment_stripped_lines,
            non_test_mask,
            lang,
            ast_items,
            test_target,
        }
    }
}

/// Evaluates all 5 correctness pillars (`E0601`..`E0605`) deterministically over a single source file.
/// Realises: [REQ-0606-EVALUATE-SOURCE-FILE-PILLARS]
pub fn evaluate_source_file_pillars(
    ctx: &SourceFileAuditCtx<'_>,
    filter: Option<AuditPillar>,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for &pillar in AuditPillar::ALL {
        if let Some(active) = filter {
            if active != pillar {
                continue;
            }
        }
        match pillar {
            AuditPillar::MemorySafety => audit_pillar_memory_safety(ctx, &mut findings),
            AuditPillar::ResourceLifecycle => audit_pillar_resource_lifecycle(ctx, &mut findings),
            AuditPillar::Concurrency => audit_pillar_concurrency(ctx, &mut findings),
            AuditPillar::TestIntegrity => audit_pillar_test_integrity(ctx, &mut findings),
            AuditPillar::SemanticTraceability => {
                if ctx.lang == SourceLanguage::Rust {
                    audit_pillar_semantic_traceability(ctx, &mut findings);
                }
            }
        }
    }
    findings
}

/// Top-level `BP-06` metamodel orchestrator (`DeterministicStaticDefectAuditor`).
/// Realises: [REQ-0606-METAMODEL-AUDITOR]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicStaticDefectAuditor {
    pub auditor_subagent_count: usize,
    pub dossier_schema_check_count: usize,
    pub pillar_coverage_count: usize,
    pub closed_loop_mismatch_bytes: usize,
}

impl DeterministicStaticDefectAuditor {
    /// Constructs a canonical `DeterministicStaticDefectAuditor` satisfying `C-AUD-01`..`C-AUD-04`.
    /// Realises: [REQ-0606-METAMODEL-AUDITOR-NEW]
    pub const fn new() -> Self {
        Self {
            auditor_subagent_count: 0,
            dossier_schema_check_count: 12,
            pillar_coverage_count: 5,
            closed_loop_mismatch_bytes: 0,
        }
    }

    /// Executes the end-to-end deterministic 5-pillar static audit and dossier persistence workflow.
    /// Realises: [REQ-0606-METAMODEL-AUDITOR-EXEC]
    pub fn execute(&self, workspace_root: &Path, args: &AuditCodeArgs) -> Result<(), i32> {
        if self.auditor_subagent_count != 0
            || self.dossier_schema_check_count != 12
            || self.pillar_coverage_count != 5
            || self.closed_loop_mismatch_bytes != 0
        {
            eprintln!("[AUDIT-CODE INVARIANT ERROR] Auditor metamodel constraints violated");
            return Err(1);
        }
        run_audit_code(workspace_root, args)
    }
}

impl Default for DeterministicStaticDefectAuditor {
    /// Returns the canonical default `DeterministicStaticDefectAuditor` (`BP-06` Section 2).
    /// Realises: [REQ-0606-METAMODEL-AUDITOR-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component evaluating all 5 static AST correctness pillars (`E0601`..`E0605`).
/// Realises: [REQ-0606-METAMODEL-ANALYZER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FivePillarStaticAstAnalyzer {
    pub active_pillar_count: usize,
    pub zero_regex_mode: bool,
}

impl FivePillarStaticAstAnalyzer {
    /// Constructs a canonical `FivePillarStaticAstAnalyzer` (`active_pillar_count = 5`).
    /// Realises: [REQ-0606-METAMODEL-ANALYZER-NEW]
    pub const fn new() -> Self {
        Self {
            active_pillar_count: 5,
            zero_regex_mode: true,
        }
    }

    /// Evaluates all enabled correctness pillars across a single Rust or Dart source buffer.
    /// Realises: [REQ-0606-METAMODEL-ANALYZER-EVAL]
    pub fn evaluate_source_file_pillars(
        &self,
        workspace_root: &Path,
        rel_path: &str,
        source_text: &str,
        filter: Option<AuditPillar>,
    ) -> Vec<AuditFinding> {
        if self.active_pillar_count == 0 || !self.zero_regex_mode {
            return Vec::new();
        }
        if rel_path.ends_with(".rs") {
            let Ok(items) = parse_rust_source_ast(source_text, rel_path) else {
                return Vec::new();
            };
            let ctx = SourceFileAuditCtx::new(
                workspace_root,
                rel_path,
                source_text,
                SourceLanguage::Rust,
                &items,
            );
            evaluate_source_file_pillars(&ctx, filter)
        } else if rel_path.ends_with(".dart") {
            let Ok(items) = parse_dart_source_ast(source_text, rel_path) else {
                return Vec::new();
            };
            let ctx = SourceFileAuditCtx::new(
                workspace_root,
                rel_path,
                source_text,
                SourceLanguage::Dart,
                &items,
            );
            evaluate_source_file_pillars(&ctx, filter)
        } else {
            Vec::new()
        }
    }
}

impl Default for FivePillarStaticAstAnalyzer {
    /// Returns the canonical default `FivePillarStaticAstAnalyzer`.
    /// Realises: [REQ-0606-METAMODEL-ANALYZER-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component synthesizing 7-section CommonMark defect dossiers.
/// Realises: [REQ-0606-METAMODEL-SYNTHESIZER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicDossierSynthesizer {
    pub mandatory_section_count: usize,
    pub whys_chain_depth: usize,
}

impl DeterministicDossierSynthesizer {
    /// Constructs a canonical `DeterministicDossierSynthesizer` (`7` sections, `5` Whys).
    /// Realises: [REQ-0606-METAMODEL-SYNTHESIZER-NEW]
    pub const fn new() -> Self {
        Self {
            mandatory_section_count: 7,
            whys_chain_depth: 5,
        }
    }

    /// Synthesizes a 12-check compliant 7-section defect dossier markdown string from `finding`.
    /// Realises: [REQ-0606-METAMODEL-SYNTHESIZER-EXEC]
    pub fn synthesize_defect_dossier_markdown(&self, finding: &AuditFinding) -> String {
        if self.mandatory_section_count == 0 || self.whys_chain_depth == 0 {
            return String::new();
        }
        synthesize_defect_dossier_markdown(finding)
    }
}

impl Default for DeterministicDossierSynthesizer {
    /// Returns the canonical default `DeterministicDossierSynthesizer`.
    /// Realises: [REQ-0606-METAMODEL-SYNTHESIZER-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component generating Check 13B-compliant Section 4 Mermaid diagrams.
/// Realises: [REQ-0606-METAMODEL-MERMAID-GEN]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MermaidUmlDiagramGenerator {
    pub forbid_semicolons: bool,
    pub check_13b_compliant: bool,
}

impl MermaidUmlDiagramGenerator {
    /// Constructs a canonical `MermaidUmlDiagramGenerator` (`forbid_semicolons = true`).
    /// Realises: [REQ-0606-METAMODEL-MERMAID-GEN-NEW]
    pub const fn new() -> Self {
        Self {
            forbid_semicolons: true,
            check_13b_compliant: true,
        }
    }

    /// Synthesizes Section 4 Mermaid UML markdown for `finding`.
    /// Realises: [REQ-0606-METAMODEL-MERMAID-GEN-EXEC]
    pub fn synthesize_section_4_uml(&self, finding: &AuditFinding) -> String {
        if !self.forbid_semicolons || !self.check_13b_compliant {
            return String::new();
        }
        synthesize_section_4_uml(finding)
    }
}

impl Default for MermaidUmlDiagramGenerator {
    /// Returns the canonical default `MermaidUmlDiagramGenerator`.
    /// Realises: [REQ-0606-METAMODEL-MERMAID-GEN-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component enforcing the 12-check dossier schema and closed-loop publish gate.
/// Realises: [REQ-0606-METAMODEL-PUBLISHER-GATE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedLoopDefectPublisherGate {
    pub schema_gate_count: usize,
    pub require_byte_parity: bool,
}

impl ClosedLoopDefectPublisherGate {
    /// Constructs a canonical `ClosedLoopDefectPublisherGate` (`schema_gate_count = 12`).
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-GATE-NEW]
    pub const fn new() -> Self {
        Self {
            schema_gate_count: 12,
            require_byte_parity: true,
        }
    }

    /// Validates a synthesized defect dossier against all 12 workspace-scoped schema gates.
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-VALIDATE]
    pub fn validate_scoped(
        &self,
        content: &str,
        target_repo: Option<&str>,
        workspace_root: Option<&Path>,
    ) -> Result<DefectSchemaValidation, Vec<String>> {
        if self.schema_gate_count != 12 || !self.require_byte_parity {
            return Err(vec![
                "ClosedLoopDefectPublisherGate invariant breach".to_string()
            ]);
        }
        validate_defect_dossier_scoped(content, target_repo, workspace_root)
    }

    /// Publishes or updates a remote tracker issue and verifies closed-loop payload byte parity.
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-RUN]
    pub fn publish_closed_loop(&self, args: &DefectDossierArgs) -> Result<(), i32> {
        if self.schema_gate_count != 12 || !self.require_byte_parity {
            return Err(1);
        }
        run_check_defect(args)
    }
}

impl Default for ClosedLoopDefectPublisherGate {
    /// Returns the canonical default `ClosedLoopDefectPublisherGate`.
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Deterministic 5-pillar static AST code auditor and 7-section defect dossier synthesizer (`BP-06`).
//! Realises: [REQ-0606-AUDIT-CODE-MODULE]

pub use crate::audit_code_models::{
    evaluate_source_file_pillars, ClosedLoopDefectPublisherGate, DeterministicDossierSynthesizer,
    DeterministicStaticDefectAuditor, FivePillarStaticAstAnalyzer, MermaidUmlDiagramGenerator,
    SourceFileAuditCtx,
};
use crate::audit_code_pillars_c::{
    is_already_recorded_locally, sanitize_mermaid_identifier, scan_workspace_path_deterministic,
};
use crate::defect_dossier::{
    derive_defect_title, DefectDossierArgs, DossierSection, DossierSeverity,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical 5-pillar correctness risk taxonomy (`E0601` through `E0605`) for static AST auditing.
/// Realises: [REQ-0606-AUDIT-PILLAR-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuditPillar {
    MemorySafety,
    ResourceLifecycle,
    Concurrency,
    TestIntegrity,
    SemanticTraceability,
}

impl AuditPillar {
    /// Exhaustive ordered slice of all 5 correctness risk pillars.
    /// Realises: [REQ-0606-AUDIT-PILLAR-ALL]
    pub const ALL: &'static [Self] = &[
        Self::MemorySafety,
        Self::ResourceLifecycle,
        Self::Concurrency,
        Self::TestIntegrity,
        Self::SemanticTraceability,
    ];

    /// Returns the canonical human-readable pillar title matching `Audit Source` governance.
    /// Realises: [REQ-0606-AUDIT-PILLAR-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MemorySafety => "Memory Safety",
            Self::ResourceLifecycle => "Resource Lifecycle",
            Self::Concurrency => "Concurrency",
            Self::TestIntegrity => "Test Integrity",
            Self::SemanticTraceability => "Semantic Traceability",
        }
    }

    /// Returns the deterministic `BP-06` diagnostic code (`E0601`..=`E0605`) for this pillar.
    /// Realises: [REQ-0606-AUDIT-PILLAR-DIAG-CODE]
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::MemorySafety => "E0601",
            Self::ResourceLifecycle => "E0602",
            Self::Concurrency => "E0603",
            Self::TestIntegrity => "E0604",
            Self::SemanticTraceability => "E0605",
        }
    }

    /// Returns the 1-based ordinal number (`1..=5`) for this correctness pillar.
    /// Realises: [REQ-0606-AUDIT-PILLAR-ORDINAL]
    pub const fn pillar_number(self) -> usize {
        match self {
            Self::MemorySafety => 1,
            Self::ResourceLifecycle => 2,
            Self::Concurrency => 3,
            Self::TestIntegrity => 4,
            Self::SemanticTraceability => 5,
        }
    }

    /// Formats the canonical Section 1 pillar descriptor (`Pillar <1..5> -- <Name> (<E0601..E0605>)`).
    /// Realises: [REQ-0606-AUDIT-PILLAR-HEADER]
    pub fn formatted_header(self) -> String {
        format!(
            "Pillar {} -- {} ({})",
            self.pillar_number(),
            self.as_str(),
            self.diagnostic_code()
        )
    }

    /// Parses a CLI `--pillar` token into an optional `AuditPillar` filter (`None` for `"all"`).
    /// Realises: [REQ-0606-AUDIT-PILLAR-FROM-CLI]
    pub fn from_cli_token(token: &str) -> Result<Option<Self>, String> {
        match token.trim() {
            "all" => Ok(None),
            "memory-safety" => Ok(Some(Self::MemorySafety)),
            "resource-lifecycle" => Ok(Some(Self::ResourceLifecycle)),
            "concurrency" => Ok(Some(Self::Concurrency)),
            "test-integrity" => Ok(Some(Self::TestIntegrity)),
            "semantic-traceability" => Ok(Some(Self::SemanticTraceability)),
            other => Err(format!("Unsupported audit pillar: {other}")),
        }
    }
}

/// Deterministic UML diagram archetype selected for Section 4 of a synthesized defect dossier.
/// Realises: [REQ-0606-UML-DIAGRAM-KIND]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UmlDiagramKind {
    SequenceDiagram,
    ClassDiagram,
}

/// Complete, line-exact static analysis finding ready for 7-section defect dossier synthesis.
/// Realises: [REQ-0606-AUDIT-FINDING-STRUCT]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    pub slug: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub pillar: AuditPillar,
    pub severity: DossierSeverity,
    pub symptom: String,
    pub test_target: String,
    pub whys: Vec<(String, String)>,
    pub correctness_trace: String,
    pub invariant_violated: String,
    pub uml_kind: UmlDiagramKind,
    pub caller_symbol: String,
    pub callee_symbol: String,
    pub affected_callers: Vec<(String, String)>,
    pub language_tag: String,
    pub proposed_correction_code: String,
}

/// Command-line configuration record for `verify-baseline audit-code`.
/// Realises: [REQ-0606-AUDIT-CODE-ARGS]
#[derive(Debug, Clone)]
pub struct AuditCodeArgs {
    pub path: PathBuf,
    pub pillar: Option<AuditPillar>,
    pub issue: Option<u64>,
    pub repo: Option<String>,
    pub publish: bool,
    pub dry_run: bool,
}

/// Synthesizes a Check 13B-compliant Section 4 Mermaid diagram block with zero semicolons.
/// Realises: [REQ-0606-SYNTHESIZE-SECTION-4-UML]
pub fn synthesize_section_4_uml(finding: &AuditFinding) -> String {
    if !finding.severity.requires_mermaid() {
        return format!("N/A -- {} severity.", finding.severity.as_str());
    }
    let clean_caller = sanitize_mermaid_identifier(&finding.caller_symbol, "CallerScope");
    let mut clean_callee = sanitize_mermaid_identifier(&finding.callee_symbol, "TargetSymbol");
    if clean_caller == clean_callee {
        clean_callee.push_str("Target");
    }
    let clean_inv = sanitize_mermaid_identifier(&finding.invariant_violated, "SafetyInvariant");
    match finding.uml_kind {
        UmlDiagramKind::SequenceDiagram => format!(
            "```mermaid\nsequenceDiagram\n    participant CallerScope as \"{clean_caller}\"\n    participant TargetSymbol as \"{clean_callee}\"\n    CallerScope->>TargetSymbol: \"Invoke operation at line {}\"\n    TargetSymbol-->>CallerScope: \"Violates {clean_inv}\"\n```",
            finding.start_line
        ),
        UmlDiagramKind::ClassDiagram => format!(
            "```mermaid\nclassDiagram\n    class {clean_caller} {{\n        +String fileLocation\n        +Boolean verifyInvariant()\n    }}\n    class {clean_callee} {{\n        +Integer lineSpan\n        +Boolean enforceGuard()\n    }}\n    {clean_caller} --> {clean_callee} : \"audits {clean_inv}\"\n```"
        ),
    }
}

/// Deterministically synthesizes a 12-check compliant 7-section defect dossier markdown string.
/// Realises: [REQ-0606-SYNTHESIZE-DOSSIER-MARKDOWN]
pub fn synthesize_defect_dossier_markdown(finding: &AuditFinding) -> String {
    let file_loc = format!(
        "{}:{}-{}",
        finding.file_path, finding.start_line, finding.end_line
    );
    let uml_block = synthesize_section_4_uml(finding);
    let callers_block = finding
        .affected_callers
        .iter()
        .map(|(caller, impact)| format!("- `{caller}` -- {impact}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut out = String::with_capacity(4096);
    out.push_str(DossierSection::ContextAndReferences.header());
    out.push_str(&format!(
        "\n\n<!-- test-target: {} -->\n\n- **File**: `{file_loc}`\n- **Pillar**: {}\n- **Symptom**: {}\n- **Test-Target**: `{}`\n\n",
        finding.test_target,
        finding.pillar.formatted_header(),
        finding.symptom,
        finding.test_target
    ));
    out.push_str(DossierSection::RootCauseAnalysis.header());
    out.push_str("\n\n");
    for (idx, (question, reason)) in finding.whys.iter().enumerate() {
        out.push_str(&format!(
            "{}. **Why {}?** Because {}.\n",
            idx.saturating_add(1),
            question.trim().trim_end_matches('?'),
            reason.trim().trim_end_matches('.')
        ));
    }
    out.push('\n');
    out.push_str(DossierSection::CorrectnessAnalysis.header());
    out.push_str(&format!(
        "\n\n{}\n\n- **Violated Invariant**: `{}` (`{}`)\n\n{}\n\n{uml_block}\n\n{}\n\n{callers_block}\n\n{}\n\n```{}\n{}\n```\n\n{}\n\nDiscovered in deterministic static 5-pillar AST audit -- new finding.\n\n## Audit Source\n\nAdversarial {} Audit\nSEVERITY: {}\nFILE_LOCATION: {file_loc}\n",
        finding.correctness_trace,
        finding.invariant_violated,
        finding.pillar.diagnostic_code(),
        DossierSection::UmlDiagrams.header(),
        DossierSection::AffectedCallers.header(),
        DossierSection::ProposedCorrection.header(),
        finding.language_tag,
        finding.proposed_correction_code.trim(),
        DossierSection::RelationshipToExistingIssues.header(),
        finding.pillar.as_str(),
        finding.severity.as_str()
    ));
    out
}

/// Executes the end-to-end deterministic 5-pillar static audit, 12-check validation, persistence, and SSOT publish gate.
/// Realises: [REQ-0606-RUN-AUDIT-CODE]
pub fn run_audit_code(workspace_root: &Path, args: &AuditCodeArgs) -> Result<(), i32> {
    let findings = scan_workspace_path_deterministic(workspace_root, &args.path, args.pillar)?;
    let defects_dir = workspace_root.join(".pipeline").join("defects");
    if let Err(err) = fs::create_dir_all(&defects_dir) {
        eprintln!(
            "[AUDIT-CODE IO ERROR] Failed to create {}: {err}",
            defects_dir.display()
        );
        return Err(1);
    }
    let is_upstream_ws = workspace_root.join(".pipeline").join("upstream").is_dir();
    let effective_repo = match &args.repo {
        Some(repo_slug) => Some(repo_slug.as_str()),
        None if is_upstream_ws => Some("upstream"),
        None => Some("local-offline/workspace"),
    };
    let synthesizer = DeterministicDossierSynthesizer::default();
    let gate = ClosedLoopDefectPublisherGate::default();

    for finding in &findings {
        let dossier_md = synthesizer.synthesize_defect_dossier_markdown(finding);
        let validation = gate
            .validate_scoped(&dossier_md, effective_repo, Some(workspace_root))
            .map_err(|errs| {
                for err_msg in errs {
                    eprintln!("[AUDIT-CODE SCHEMA ERROR] {err_msg}");
                }
                1_i32
            })?;
        if is_already_recorded_locally(&defects_dir, &validation.file_location, finding.pillar)?
            && args.issue.is_none()
        {
            continue;
        }
        let dossier_path = defects_dir.join(format!("DEFECT-{}.md", finding.slug));
        crate::write_atomic_file(&dossier_path, &dossier_md).map_err(|err| {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Failed to write {}: {err}",
                dossier_path.display()
            );
            1_i32
        })?;
        if !args.dry_run && (args.publish || args.issue.is_some()) {
            let title = derive_defect_title(&validation.file_location, &validation.symptom);
            let check_args = DefectDossierArgs {
                body_file: Some(dossier_path.clone()),
                title: Some(title),
                repo: args.repo.clone(),
                label: Some(validation.suggested_label),
                provider: "github".to_string(),
                dry_run: false,
                publish: args.publish,
                issue: args.issue,
                target: Some(dossier_path),
            };
            gate.publish_closed_loop(&check_args)?;
        }
    }
    Ok(())
}

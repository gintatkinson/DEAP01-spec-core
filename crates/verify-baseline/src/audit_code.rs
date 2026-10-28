//! Deterministic 5-pillar static AST code auditor and 7-section defect dossier synthesizer (`BP-06`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Implements the deterministic native Rust `verify-baseline audit-code` engine (`REQ-0606` / `BP-06`)
//! under DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C tool qualification rules,
//! replacing stochastic multi-agent code-auditing swarms with zero-subagent static AST analysis.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero-subagent execution (`auditor_subagent_count == 0`) using `deap_core::source_ast` slices
//!   and deterministic character-slice scanners with zero external pattern crates.
//! - Exhaustive 5-pillar risk evaluation (`E0601` through `E0605`) across Rust and Dart sources.
//! - Every synthesized `.pipeline/defects/DEFECT-<SLUG>.md` dossier passes all 12 checks in
//!   `validate_defect_dossier_scoped` prior to disk persistence or tracker synchronization.
//!
//! ## 3. Preconditions
//! - Target `--path` exists on disk and all `.rs` and `.dart` files satisfy balanced delimiter syntax.
//!
//! ## 4. Postconditions
//! - Deduplicated 7-section defect dossiers are deterministically persisted to `.pipeline/defects/`
//!   and optionally synchronized with the remote tracker under closed-loop byte-parity verification.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N + V + K)$ over source bytes $N$, AST items $V$, and findings $K$.
//! - Space Complexity: $O(N + V + K)$ bounded auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing paths, unbalanced source syntax, or 12-check dossier schema violations fail closed
//!   immediately with exit code `Err(1)` and structured `stderr` diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0606-AUDIT-CODE-MODULE]

use crate::defect_dossier::{
    derive_defect_title, run_check_defect, validate_defect_dossier_scoped, DefectDossierArgs,
    DefectSchemaValidation, DossierSection, DossierSeverity,
};
use deap_core::source_ast::{
    parse_dart_source_ast, parse_rust_source_ast, validate_source_syntax, SourceAstItem,
    SourceLanguage,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical 5-pillar correctness risk taxonomy (`E0601` through `E0605`) for static AST auditing.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1629A defect classification governance for `BP-06`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijectively maps each of the 5 correctness pillars to diagnostic codes `E0601`..=`E0605`.
///
/// ## 3. Preconditions
/// - Constructed via `AuditPillar::from_cli_token` or static pillar iteration over `AuditPillar::ALL`.
///
/// ## 4. Postconditions
/// - Preserves total ordering and deterministic formatting across dossier headers and audit footers.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ per variant operation.
/// - Space Complexity: $O(1)$ stack representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unknown CLI pillar strings return `None` in `from_cli_token` without panicking.
///
/// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees complete deterministic 5-pillar iteration order under DO-330 TQL-4.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `ALL.len() == 5` in canonical order `E0601` through `E0605`.
    ///
    /// ## 3. Preconditions
    /// - Static compile-time constant requiring no runtime initialization.
    ///
    /// ## 4. Postconditions
    /// - Yields all 5 `AuditPillar` variants deterministically.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Immutable static slice; zero runtime failure modes.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-AUDIT-PILLAR-ALL]
    pub const ALL: &'static [Self] = &[
        Self::MemorySafety,
        Self::ResourceLifecycle,
        Self::Concurrency,
        Self::TestIntegrity,
        Self::SemanticTraceability,
    ];

    /// Returns the canonical human-readable pillar title matching `Audit Source` governance.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces exact `Adversarial <Pillar> Audit` footer parity with `defect_dossier` Check 9.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns non-empty static ASCII title for every variant.
    ///
    /// ## 3. Preconditions
    /// - Valid `AuditPillar` variant.
    ///
    /// ## 4. Postconditions
    /// - Emits exact pillar title string.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Exhaustive match; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Anchors each static finding to its formal `REQ-0606` diagnostic identifier.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective mapping onto `{"E0601", "E0602", "E0603", "E0604", "E0605"}`.
    ///
    /// ## 3. Preconditions
    /// - Valid `AuditPillar` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns static 5-byte ASCII diagnostic code.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Exhaustive match; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Provides deterministic Section 1 `- **Pillar**:` numbering for synthesized defect dossiers.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Result lies strictly in `1..=5`.
    ///
    /// ## 3. Preconditions
    /// - Valid `AuditPillar` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns `1` through `5` matching `E0601` through `E0605`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Exhaustive match; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees machine-readable pillar and diagnostic traceability in Section 1 of every dossier.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Contains exact ASCII `--` separator, ordinal `Pillar N`, title, and `(E060N)` code.
    ///
    /// ## 3. Preconditions
    /// - Valid `AuditPillar` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns owned ASCII string free of Unicode dashes.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Bounded fixed-format string allocation; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Validates `--pillar` CLI arguments deterministically under DO-330 TQL-4 input gating.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `"all"` maps to `Ok(None)`; each valid kebab-case pillar maps to `Ok(Some(pillar))`.
    ///
    /// ## 3. Preconditions
    /// - `token` is a UTF-8 CLI option string.
    ///
    /// ## 4. Postconditions
    /// - Returns `Err(String)` for unrecognized pillar names.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(|token|)$. Space Complexity: $O(|token|)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Rejects unknown tokens via `Err(String)` without panicking.
    ///
    /// ## 7. Realization & Traceability
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces DO-178C Level A visual architectural traceability across behavioral and structural defects.
///
/// ## 2. Invariants & Mathematical Properties
/// - Maps control-flow defects to `sequenceDiagram` and structural/type defects to `classDiagram`.
///
/// ## 3. Preconditions
/// - Selected deterministically by each pillar rule evaluator.
///
/// ## 4. Postconditions
/// - Directs `MermaidUmlDiagramGenerator` to emit a Check 13B-compliant Mermaid block.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Closed two-variant enumeration; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-UML-DIAGRAM-KIND]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UmlDiagramKind {
    SequenceDiagram,
    ClassDiagram,
}

/// Complete, line-exact static analysis finding ready for 7-section defect dossier synthesis.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Captures full DO-178C Level A / Leveson STPA provenance for a single static code defect.
///
/// ## 2. Invariants & Mathematical Properties
/// - `start_line >= 1`, `end_line >= start_line`, and `whys.len() == 5`.
///
/// ## 3. Preconditions
/// - Populated from `SourceAstItem` and lexical slice coordinates within `workspace_root`.
///
/// ## 4. Postconditions
/// - Serializes deterministically into a 12-check compliant `.pipeline/defects/DEFECT-<slug>.md` dossier.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ field access. Space Complexity: $O(L)$ over diagnostic text length $L$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Validated downstream by `validate_defect_dossier_scoped` prior to disk persistence.
///
/// ## 7. Realization & Traceability
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Governs deterministic CLI invocation parameters under DO-330 TQL-4 tool qualification.
///
/// ## 2. Invariants & Mathematical Properties
/// - When `dry_run` is `true` or `(!publish && issue.is_none())`, zero remote tracker subprocesses run.
///
/// ## 3. Preconditions
/// - Constructed by `clap::Command` builder parsing in `crates/verify-baseline/src/main.rs`.
///
/// ## 4. Postconditions
/// - Controls target path, optional single-pillar filter, local persistence, and remote synchronization.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(|path| + |repo|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-existent `path` is rejected fail-closed at audit startup with exit code `1`.
///
/// ## 7. Realization & Traceability
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

/// Top-level `BP-06` metamodel orchestrator (`DeterministicStaticDefectAuditor`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Realizes the `BP-06` Section 2 `DeterministicStaticDefectAuditor` component with zero LLM subagents.
///
/// ## 2. Invariants & Mathematical Properties
/// - `auditor_subagent_count == 0`, `dossier_schema_check_count == 12`, `pillar_coverage_count == 5`, `closed_loop_mismatch_bytes == 0`.
///
/// ## 3. Preconditions
/// - Initialized via `DeterministicStaticDefectAuditor::default()` or `new()`.
///
/// ## 4. Postconditions
/// - Coordinates static AST analysis, 7-section dossier synthesis, and closed-loop SSOT gating.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ construction; $O(N + V + K)$ audit execution. Space Complexity: $O(1)$ struct size.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates non-zero exit codes on any syntax, I/O, or 12-check dossier gate failure.
///
/// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces `BP-06` Section 4.1 parametric safety bounds at compile time.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Sets `(0, 12, 5, 0)` across the four safety-critical metamodel attributes.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns valid `DeterministicStaticDefectAuditor`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure constant constructor; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Delegates to `run_audit_code` while asserting `BP-06` parametric invariants (`C-AUD-01`..`C-AUD-04`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Rejects execution if metamodel safety parameters deviate from `(0, 12, 5, 0)`.
    ///
    /// ## 3. Preconditions
    /// - `workspace_root` and `args.path` reference accessible filesystem locations.
    ///
    /// ## 4. Postconditions
    /// - Synthesizes and validates `.pipeline/defects/DEFECT-*.md` dossiers deterministically.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N + V + K)$. Space Complexity: $O(N + V + K)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `Err(1)` on metamodel invariant breach, I/O error, syntax error, or schema rejection.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees safe default initialization under DO-178C Level A.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Equivalent to `DeterministicStaticDefectAuditor::new()`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns `(0, 12, 5, 0)` safety parameter configuration.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-AUDITOR-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component evaluating all 5 static AST correctness pillars (`E0601`..`E0605`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Realizes `FivePillarStaticAstAnalyzer` under DO-178C Level A and DO-330 TQL-4 static analysis rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - `active_pillar_count == 5` and `zero_regex_mode == true`.
///
/// ## 3. Preconditions
/// - Initialized via `FivePillarStaticAstAnalyzer::default()` or `new()`.
///
/// ## 4. Postconditions
/// - Evaluates Rust and Dart `SourceAstItem` declarations using deterministic character-slice scanners.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N + V)$. Space Complexity: $O(K)$ emitted findings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking slice inspection via `.get(..)` and explicit bounds checks.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-METAMODEL-ANALYZER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FivePillarStaticAstAnalyzer {
    pub active_pillar_count: usize,
    pub zero_regex_mode: bool,
}

impl FivePillarStaticAstAnalyzer {
    /// Constructs a canonical `FivePillarStaticAstAnalyzer` (`active_pillar_count = 5`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces `BP-06` Section 2 static AST analyzer configuration invariants.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `(5, true)`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns valid `FivePillarStaticAstAnalyzer`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-ANALYZER-NEW]
    pub const fn new() -> Self {
        Self {
            active_pillar_count: 5,
            zero_regex_mode: true,
        }
    }

    /// Evaluates all enabled correctness pillars across a single Rust or Dart source buffer.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Dispatches `evaluate_source_file_pillars` over parsed `deap_core::source_ast` declarations.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Emits deterministic, sorted `AuditFinding` records.
    ///
    /// ## 3. Preconditions
    /// - `source_text` has passed balanced-delimiter syntax verification.
    ///
    /// ## 4. Postconditions
    /// - Returns all detected violations matching `filter`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N + V)$. Space Complexity: $O(N + V + K)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Unrecognized file extensions yield an empty finding vector without panicking.
    ///
    /// ## 7. Realization & Traceability
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
            let items = match parse_rust_source_ast(source_text, rel_path) {
                Ok(parsed) => parsed,
                Err(_) => return Vec::new(),
            };
            evaluate_source_file_pillars(
                workspace_root,
                rel_path,
                source_text,
                SourceLanguage::Rust,
                &items,
                filter,
            )
        } else if rel_path.ends_with(".dart") {
            let items = match parse_dart_source_ast(source_text, rel_path) {
                Ok(parsed) => parsed,
                Err(_) => return Vec::new(),
            };
            evaluate_source_file_pillars(
                workspace_root,
                rel_path,
                source_text,
                SourceLanguage::Dart,
                &items,
                filter,
            )
        } else {
            Vec::new()
        }
    }
}

impl Default for FivePillarStaticAstAnalyzer {
    /// Returns the canonical default `FivePillarStaticAstAnalyzer`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees safe default initialization under DO-178C Level A.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Equivalent to `FivePillarStaticAstAnalyzer::new()`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns `(5, true)` analyzer configuration.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-ANALYZER-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component synthesizing 7-section CommonMark defect dossiers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Realizes `DeterministicDossierSynthesizer` enforcing 7 canonical sections and 5-Whys chains.
///
/// ## 2. Invariants & Mathematical Properties
/// - `mandatory_section_count == 7` and `whys_chain_depth == 5`.
///
/// ## 3. Preconditions
/// - Initialized via `DeterministicDossierSynthesizer::default()` or `new()`.
///
/// ## 4. Postconditions
/// - Emits markdown payloads starting with `## 1. Context and References`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ over finding text size $L$. Space Complexity: $O(L)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Deterministic string formatting without panicking index operations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-METAMODEL-SYNTHESIZER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicDossierSynthesizer {
    pub mandatory_section_count: usize,
    pub whys_chain_depth: usize,
}

impl DeterministicDossierSynthesizer {
    /// Constructs a canonical `DeterministicDossierSynthesizer` (`7` sections, `5` Whys).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces `BP-06` Section 2 dossier synthesizer structural parameters.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `(7, 5)`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns valid `DeterministicDossierSynthesizer`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-SYNTHESIZER-NEW]
    pub const fn new() -> Self {
        Self {
            mandatory_section_count: 7,
            whys_chain_depth: 5,
        }
    }

    /// Synthesizes a 12-check compliant 7-section defect dossier markdown string from `finding`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Delegates to `synthesize_defect_dossier_markdown` under `BP-06` structural bounds.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Produces identical output bytes across repeated invocations for identical `finding`.
    ///
    /// ## 3. Preconditions
    /// - `finding` carries valid coordinates and a 5-element `whys` causal chain.
    ///
    /// ## 4. Postconditions
    /// - Returns complete 7-section CommonMark defect dossier payload.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(L)$. Space Complexity: $O(L)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure in-memory string assembly; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees safe default initialization under DO-178C Level A.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Equivalent to `DeterministicDossierSynthesizer::new()`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns `(7, 5)` synthesizer configuration.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-SYNTHESIZER-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component generating Check 13B-compliant Section 4 Mermaid diagrams.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Realizes `MermaidUmlDiagramGenerator` under DO-178C Level A visual architecture verification rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - `forbid_semicolons == true` and `check_13b_compliant == true`; emits zero `;` characters.
///
/// ## 3. Preconditions
/// - Initialized via `MermaidUmlDiagramGenerator::default()` or `new()`.
///
/// ## 4. Postconditions
/// - Emits clean `sequenceDiagram`, `classDiagram`, or `N/A --` declaration per finding severity.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S)$ over symbol length $S$. Space Complexity: $O(S)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Sanitizes non-alphanumeric symbol bytes deterministically without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-METAMODEL-MERMAID-GEN]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MermaidUmlDiagramGenerator {
    pub forbid_semicolons: bool,
    pub check_13b_compliant: bool,
}

impl MermaidUmlDiagramGenerator {
    /// Constructs a canonical `MermaidUmlDiagramGenerator` (`forbid_semicolons = true`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces `BP-06` Section 2 Mermaid generator flags at compile time.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `(true, true)`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns valid `MermaidUmlDiagramGenerator`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-MERMAID-GEN-NEW]
    pub const fn new() -> Self {
        Self {
            forbid_semicolons: true,
            check_13b_compliant: true,
        }
    }

    /// Synthesizes Section 4 Mermaid UML markdown for `finding`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees Check 13B compliance across all synthesized `Critical` and `Important` dossiers.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Emits zero semicolons and strictly alphanumeric participant/class identifiers.
    ///
    /// ## 3. Preconditions
    /// - `self.forbid_semicolons` and `self.check_13b_compliant` are `true`.
    ///
    /// ## 4. Postconditions
    /// - Returns Section 4 markdown block.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(S)$. Space Complexity: $O(S)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure in-memory formatting; infallible.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees safe default initialization under DO-178C Level A.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Equivalent to `MermaidUmlDiagramGenerator::new()`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns `(true, true)` Mermaid generator configuration.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-MERMAID-GEN-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// `BP-06` metamodel sub-component enforcing the 12-check dossier schema and closed-loop publish gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Realizes `ClosedLoopDefectPublisherGate` under DO-178C Level A / Design by Contract governance.
///
/// ## 2. Invariants & Mathematical Properties
/// - `schema_gate_count == 12` and `require_byte_parity == true`.
///
/// ## 3. Preconditions
/// - Initialized via `ClosedLoopDefectPublisherGate::default()` or `new()`.
///
/// ## 4. Postconditions
/// - Validates dossiers through all 12 schema checks and verifies remote tracker byte parity when publishing.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(D)$ over dossier byte length $D$. Space Complexity: $O(D)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails closed with structured diagnostics before persistence or tracker completion on any mismatch.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-METAMODEL-PUBLISHER-GATE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedLoopDefectPublisherGate {
    pub schema_gate_count: usize,
    pub require_byte_parity: bool,
}

impl ClosedLoopDefectPublisherGate {
    /// Constructs a canonical `ClosedLoopDefectPublisherGate` (`schema_gate_count = 12`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces `BP-06` Section 2 closed-loop gate parameters.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `(12, true)`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns valid `ClosedLoopDefectPublisherGate`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-GATE-NEW]
    pub const fn new() -> Self {
        Self {
            schema_gate_count: 12,
            require_byte_parity: true,
        }
    }

    /// Validates a synthesized defect dossier against all 12 workspace-scoped schema gates.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Wraps `validate_defect_dossier_scoped` with `schema_gate_count == 12` enforcement.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Rejects validation if `schema_gate_count != 12` or `!require_byte_parity`.
    ///
    /// ## 3. Preconditions
    /// - `content` is a synthesized 7-section CommonMark defect dossier string.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(DefectSchemaValidation)` iff all 12 checks succeed.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(|content|)$. Space Complexity: $O(|content|)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `Err(Vec<String>)` containing exact gate failure diagnostics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-VALIDATE]
    pub fn validate_scoped(
        &self,
        content: &str,
        target_repo: Option<&str>,
        workspace_root: Option<&Path>,
    ) -> Result<DefectSchemaValidation, Vec<String>> {
        if self.schema_gate_count != 12 || !self.require_byte_parity {
            return Err(vec![
                "ClosedLoopDefectPublisherGate invariant breach".to_string(),
            ]);
        }
        validate_defect_dossier_scoped(content, target_repo, workspace_root)
    }

    /// Publishes or updates a remote tracker issue and verifies closed-loop payload byte parity.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Delegates remote synchronization to `defect_dossier::run_check_defect`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Guarantees `closed_loop_mismatch_bytes == 0` on `Ok(())`.
    ///
    /// ## 3. Preconditions
    /// - Invoked exclusively when `!args.dry_run && (args.publish || args.issue.is_some())`.
    ///
    /// ## 4. Postconditions
    /// - Remote tracker issue body matches local persisted dossier byte-for-byte after LF normalization.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(D)$ local verification plus bounded tracker CLI roundtrip. Space Complexity: $O(D)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Propagates non-zero exit code `Err(code)` on CLI or parity failure.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Guarantees safe default initialization under DO-178C Level A.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Equivalent to `ClosedLoopDefectPublisherGate::new()`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns `(12, true)` gate configuration.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-METAMODEL-PUBLISHER-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

/// Walks `target_path` deterministically in sorted lexicographical order collecting `.rs` and `.dart` files.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Guarantees reproducible traversal ordering across host filesystems under DO-330 TQL-4.
///
/// ## 2. Invariants & Mathematical Properties
/// - Skips hidden child directories (`.` prefix) and build caches (`target`, `build`, `node_modules`).
///
/// ## 3. Preconditions
/// - `target_path` exists on disk.
///
/// ## 4. Postconditions
/// - Appends discovered `.rs` and `.dart` paths to `out` in deterministic order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(F \log F)$ over directory entries $F$. Space Complexity: $O(F)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates directory read failures explicitly as `Err(1)` without silent error masking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-COLLECT-TARGET-FILES]
fn collect_target_source_files(target_path: &Path, out: &mut Vec<PathBuf>) -> Result<(), i32> {
    if target_path.is_file() {
        let name = target_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.ends_with(".rs") || name.ends_with(".dart") {
            out.push(target_path.to_path_buf());
        }
        return Ok(());
    }
    if !target_path.is_dir() {
        return Ok(());
    }
    let entries = match fs::read_dir(target_path) {
        Ok(rd) => rd,
        Err(err) => {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Unable to read directory {}: {err}",
                target_path.display()
            );
            return Err(1);
        }
    };
    let mut children = Vec::new();
    for entry_res in entries {
        match entry_res {
            Ok(entry) => children.push(entry.path()),
            Err(err) => {
                eprintln!(
                    "[AUDIT-CODE IO ERROR] Directory entry error in {}: {err}",
                    target_path.display()
                );
                return Err(1);
            }
        }
    }
    children.sort();
    for child in children {
        let name = child
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.starts_with('.')
            || name == "target"
            || name == "build"
            || name == "node_modules"
        {
            continue;
        }
        if child.is_dir() {
            collect_target_source_files(&child, out)?;
        } else if name.ends_with(".rs") || name.ends_with(".dart") {
            out.push(child);
        }
    }
    Ok(())
}

/// Normalizes `file_path` relative to `workspace_root` using forward-slash separators.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces workspace-relative path invariance across all synthesized `FILE_LOCATION` records.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips `workspace_root` prefix and normalizes `\` separators to `/`.
///
/// ## 3. Preconditions
/// - `workspace_root` and `file_path` are valid paths.
///
/// ## 4. Postconditions
/// - Returns trimmed workspace-relative path without leading `./`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|file\_path|)$. Space Complexity: $O(|file\_path|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back cleanly to `file_path` display representation when outside `workspace_root`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-NORMALIZE-REL-PATH]
fn normalize_rel_path(workspace_root: &Path, file_path: &Path) -> String {
    let rel = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
    let normalized = rel.to_string_lossy().replace('\\', "/");
    let trimmed = normalized.trim();
    trimmed.strip_prefix("./").unwrap_or(trimmed).to_string()
}

/// Resolves the canonical `Test-Target` path for Section 1 of a synthesized defect dossier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Satisfies `defect_dossier` Check 12 (`Test-Target` workspace grounding) in upstream and downstream modes.
///
/// ## 2. Invariants & Mathematical Properties
/// - Upstream compiler mode maps non-crate paths to `crates/verify-baseline/tests/blueprint_gate_tests.rs`.
///
/// ## 3. Preconditions
/// - `rel_path` is a normalized workspace-relative file path.
///
/// ## 4. Postconditions
/// - Returns a non-empty, whitespace-free workspace-relative test target path.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|rel\_path|)$. Space Complexity: $O(|rel\_path|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure path prefix inspection; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-RESOLVE-TEST-TARGET]
fn resolve_canonical_test_target(workspace_root: &Path, rel_path: &str) -> String {
    let is_upstream_ws = workspace_root.join(".pipeline").join("upstream").is_dir();
    let trimmed = rel_path.trim();
    if is_upstream_ws {
        let in_upstream_crate = trimmed.starts_with("crates/deap-core/")
            || trimmed.starts_with("crates/verify-baseline/")
            || trimmed.starts_with("crates/compile-sysml/")
            || trimmed.starts_with("crates/ingest-sysml/")
            || trimmed.starts_with("crates/assemble-conops/")
            || trimmed.starts_with("crates/reconcile-backlog/");
        if !in_upstream_crate {
            return "crates/verify-baseline/tests/blueprint_gate_tests.rs".to_string();
        }
    }
    trimmed.to_string()
}

/// Converts an arbitrary file/symbol string into a deterministic kebab-case ASCII slug.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures filesystem-safe `.pipeline/defects/DEFECT-<slug>.md` naming across platforms.
///
/// ## 2. Invariants & Mathematical Properties
/// - Output contains exclusively lowercase ASCII alphanumeric characters and single `'-'` separators.
///
/// ## 3. Preconditions
/// - `raw` is any UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns non-empty kebab-case slug (falling back to `"finding"` when `raw` has no alphanumerics).
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|raw|)$. Space Complexity: $O(|raw|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure character iterator transformation; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-SANITIZE-SLUG]
fn sanitize_slug(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "finding".to_string()
    } else {
        out
    }
}

/// Masks comments (and optionally string/char literals) with spaces while preserving line boundaries.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Eliminates false-positive static findings inside docstrings, comments, and string literals without pattern crates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves 1-to-1 line count parity (`result.len() == source.lines().count()`).
/// - Distinguishes Rust lifetime tokens (`'static`) from character literals (`'x'`, `'\n'`).
///
/// ## 3. Preconditions
/// - `source` is valid UTF-8 source text.
///
/// ## 4. Postconditions
/// - Returns line-aligned masked source strings.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|source|)$. Space Complexity: $O(|source|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses peekable character iterator without slice indexing; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-MASK-SOURCE-LINES]
fn mask_source_lines(source: &str, mask_strings: bool) -> Vec<String> {
    let mut out = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let total = chars.len();
    let mut idx: usize = 0;
    let mut in_line_comment = false;
    let mut block_depth: usize = 0;
    let mut in_double_quote = false;
    let mut quote_escaped = false;

    while idx < total {
        let ch = chars.get(idx).copied().unwrap_or(' ');
        let next = chars.get(idx.saturating_add(1)).copied();

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push('\n');
            } else if ch == '\r' {
                out.push('\r');
            } else {
                out.push(' ');
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if block_depth > 0 {
            if ch == '/' && next == Some('*') {
                block_depth = block_depth.saturating_add(1);
                out.push_str("  ");
                idx = idx.saturating_add(2);
            } else if ch == '*' && next == Some('/') {
                block_depth = block_depth.saturating_sub(1);
                out.push_str("  ");
                idx = idx.saturating_add(2);
            } else if ch == '\n' || ch == '\r' {
                out.push(ch);
                idx = idx.saturating_add(1);
            } else {
                out.push(' ');
                idx = idx.saturating_add(1);
            }
            continue;
        }

        if in_double_quote {
            if quote_escaped {
                quote_escaped = false;
                out.push(if ch == '\n' || ch == '\r' {
                    ch
                } else if mask_strings {
                    ' '
                } else {
                    ch
                });
            } else if ch == '\\' {
                quote_escaped = true;
                out.push(if mask_strings { ' ' } else { ch });
            } else if ch == '"' {
                in_double_quote = false;
                out.push(if mask_strings { ' ' } else { '"' });
            } else if ch == '\n' || ch == '\r' {
                out.push(ch);
            } else {
                out.push(if mask_strings { ' ' } else { ch });
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if ch == '/' && next == Some('/') {
            in_line_comment = true;
            out.push_str("  ");
            idx = idx.saturating_add(2);
        } else if ch == '/' && next == Some('*') {
            block_depth = 1;
            out.push_str("  ");
            idx = idx.saturating_add(2);
        } else if ch == '"' {
            in_double_quote = true;
            quote_escaped = false;
            out.push(if mask_strings { ' ' } else { '"' });
            idx = idx.saturating_add(1);
        } else if mask_strings && ch == '\'' {
            let c1 = chars.get(idx.saturating_add(1)).copied();
            let c2 = chars.get(idx.saturating_add(2)).copied();
            let c3 = chars.get(idx.saturating_add(3)).copied();
            if c1 == Some('\\') && c2 != Some('\n') && c3 == Some('\'') {
                out.push_str("    ");
                idx = idx.saturating_add(4);
            } else if c1 != Some('\\') && c1 != Some('\n') && c2 == Some('\'') {
                out.push_str("   ");
                idx = idx.saturating_add(3);
            } else {
                out.push('\'');
                idx = idx.saturating_add(1);
            }
        } else {
            out.push(ch);
            idx = idx.saturating_add(1);
        }
    }

    out.lines().map(ToOwned::to_owned).collect()
}

/// Computes a line-aligned boolean mask marking non-test production lines (`true`) in Rust source.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Separates production safety-critical code from `#[cfg(test)]` and `#[test]` blocks deterministically.
///
/// ## 2. Invariants & Mathematical Properties
/// - `result.len() == masked_lines.len()`.
///
/// ## 3. Preconditions
/// - `masked_lines` has comments and string literals masked via `mask_source_lines`.
///
/// ## 4. Postconditions
/// - Marks every line within a `#[cfg(test)]` or `#[test]` brace scope as `false`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over source characters. Space Complexity: $O(L)$ over line count $L$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses saturating arithmetic on brace depth; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-RUST-NON-TEST-MASK]
fn compute_rust_non_test_mask(masked_lines: &[String]) -> Vec<bool> {
    let mut mask = Vec::with_capacity(masked_lines.len());
    let mut brace_depth: usize = 0;
    let mut pending_test_attr = false;
    let mut test_entry_depth: Option<usize> = None;

    for line in masked_lines {
        let trimmed = line.trim();
        if test_entry_depth.is_none()
            && (trimmed.starts_with("#[cfg(test)]")
                || trimmed.starts_with("#[test]")
                || trimmed.contains("::test]"))
        {
            pending_test_attr = true;
        }
        let in_test_now = test_entry_depth.is_some() || pending_test_attr;
        for ch in line.chars() {
            if ch == '{' {
                if pending_test_attr && test_entry_depth.is_none() {
                    test_entry_depth = Some(brace_depth);
                    pending_test_attr = false;
                }
                brace_depth = brace_depth.saturating_add(1);
            } else if ch == '}' {
                brace_depth = brace_depth.saturating_sub(1);
                if let Some(entry) = test_entry_depth {
                    if brace_depth <= entry {
                        test_entry_depth = None;
                    }
                }
            }
        }
        if pending_test_attr && trimmed.ends_with(';') {
            pending_test_attr = false;
        }
        mask.push(!in_test_now);
    }
    mask
}

/// Locates the 0-based closing line index of the brace-delimited block opening at or after `start_idx`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides deterministic lexical block span extraction for class, function, and test body analysis.
///
/// ## 2. Invariants & Mathematical Properties
/// - Result satisfies `result >= start_idx` and `result < masked_lines.len()` when non-empty.
///
/// ## 3. Preconditions
/// - `masked_lines` has comments and string literals masked.
///
/// ## 4. Postconditions
/// - Returns the 0-based line index where brace depth returns to `0` after the first `'{'`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(B)$ over block characters $B$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back safely to `start_idx` if no opening brace occurs.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-FIND-BRACE-BLOCK-END]
fn find_brace_block_end(masked_lines: &[String], start_idx: usize) -> usize {
    let mut depth: usize = 0;
    let mut opened = false;
    let mut idx = start_idx;
    while let Some(line) = masked_lines.get(idx) {
        for ch in line.chars() {
            if ch == '{' {
                depth = depth.saturating_add(1);
                opened = true;
            } else if ch == '}' && opened {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return idx;
                }
            }
        }
        idx = idx.saturating_add(1);
    }
    start_idx
}

/// Resolves the nearest enclosing `SourceAstItem` name at or preceding `line_1based`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Grounds Section 4 UML diagrams and Section 5 caller lists to concrete AST identifiers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Always returns a non-empty identifier string (`"ModuleScope"` fallback).
///
/// ## 3. Preconditions
/// - `line_1based >= 1`.
///
/// ## 4. Postconditions
/// - Returns closest AST declaration identifier.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|ast\_items|)$. Space Complexity: $O(1)$ auxiliary.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Gracefully handles empty `ast_items` slices.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-FIND-ENCLOSING-SYMBOL]
fn find_enclosing_symbol(ast_items: &[SourceAstItem], line_1based: usize) -> String {
    let mut best: Option<&SourceAstItem> = None;
    for item in ast_items {
        if item.line_no <= line_1based {
            match best {
                Some(prev) if prev.line_no > item.line_no => {}
                _ => best = Some(item),
            }
        }
    }
    match best {
        Some(item) if !item.name.trim().is_empty() => item.name.trim().to_string(),
        _ => "ModuleScope".to_string(),
    }
}

/// Detects unchecked `expr[idx]` or `expr[a..b]` bracket indexing on a masked Rust production line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces Pillar 1 (`E0601`) panic-free memory bounds discipline under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Distinguishes indexing (`ident[i]`, `call()[i]`) from slice types (`&[T]`), attributes (`#[...]`), and macros (`vec![...]`).
///
/// ## 3. Preconditions
/// - `masked_line` has comments and string literals replaced with spaces.
///
/// ## 4. Postconditions
/// - Returns `true` iff an identifier character or `')'` immediately precedes `'['`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|masked\_line|)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `.get(..)` byte inspection without direct indexing.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-DETECT-BRACKET-INDEX]
fn has_unchecked_bracket_index(masked_line: &str) -> bool {
    let trimmed = masked_line.trim_start();
    if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
        return false;
    }
    let bytes = masked_line.as_bytes();
    for (pos, byte) in bytes.iter().enumerate() {
        if *byte == b'[' && pos > 0 {
            if let Some(prev) = bytes.get(pos.saturating_sub(1)).copied() {
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' {
                    return true;
                }
            }
        }
    }
    false
}

/// Detects unchecked narrowing numeric `as <primitive>` casts on a masked Rust production line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents silent integer truncation hazards under Pillar 1 (`E0601` Memory Safety).
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches bounded primitive target types (`u8`, `u16`, `u32`, `usize`, `i8`, `i16`, `i32`, `isize`).
///
/// ## 3. Preconditions
/// - `masked_line` has comments and string literals masked.
///
/// ## 4. Postconditions
/// - Returns `Some(target_type)` when an unguarded narrowing `as` cast is present.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|masked\_line|)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `.get(..)` and `.strip_prefix(..)` without panicking slice bounds.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-DETECT-NARROWING-CAST]
fn detect_narrowing_as_cast(masked_line: &str) -> Option<&'static str> {
    let targets = ["u8", "u16", "u32", "usize", "i8", "i16", "i32", "isize"];
    for (idx, _) in masked_line.match_indices(" as ") {
        let after_as = match masked_line.get(idx.saturating_add(4)..) {
            Some(s) => s.trim_start(),
            None => continue,
        };
        for &ty in &targets {
            if let Some(rem) = after_as.strip_prefix(ty) {
                let boundary_ok = match rem.as_bytes().first().copied() {
                    Some(b) => !(b.is_ascii_alphanumeric() || b == b'_'),
                    None => true,
                };
                if boundary_ok {
                    return Some(ty);
                }
            }
        }
    }
    None
}

/// Evaluates Pillar 1 (`E0601` Memory Safety): unchecked indexing, narrowing `as` casts, and raw pointers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-06` Section 1.3 Pillar 1 (`E0601`) static memory and arithmetic safety checks.
///
/// ## 2. Invariants & Mathematical Properties
/// - Scans only non-test production spans and emits deterministic `AuditFinding` records.
///
/// ## 3. Preconditions
/// - `masked_lines` and `non_test_mask` share identical line lengths.
///
/// ## 4. Postconditions
/// - Appends `E0601` findings to `out`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$. Space Complexity: $O(K_1)$ emitted Pillar 1 findings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking slice traversal across all lines.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-AUDIT-PILLAR-1-MEMORY]
fn audit_pillar_memory_safety(
    rel_path: &str,
    masked_lines: &[String],
    non_test_mask: &[bool],
    lang: SourceLanguage,
    ast_items: &[SourceAstItem],
    test_target: &str,
    out: &mut Vec<AuditFinding>,
) {
    for (idx, line) in masked_lines.iter().enumerate() {
        if lang == SourceLanguage::Rust && non_test_mask.get(idx).copied() != Some(true) {
            continue;
        }
        let line_no = idx.saturating_add(1);
        let caller = find_enclosing_symbol(ast_items, line_no);

        let violation_opt: Option<(&str, String, &str, &str)> = if lang == SourceLanguage::Rust
            && has_unchecked_bracket_index(line)
        {
            Some((
                "idx",
                format!("Unchecked slice or bracket indexing in {caller} risks out-of-bounds runtime abort"),
                "SliceBoundsCheckedAccess",
                "SliceIndexer",
            ))
        } else if lang == SourceLanguage::Rust {
            if let Some(ty) = detect_narrowing_as_cast(line) {
                Some((
                    "cast",
                    format!("Unchecked numeric narrowing cast to {ty} in {caller} silently truncates upper bits"),
                    "LosslessCheckedNumericConversion",
                    "PrimitiveAsCast",
                ))
            } else if line.contains("*const ") || line.contains("*mut ") || line.contains("from_ptr(") {
                Some((
                    "ptr",
                    format!("Raw pointer dereference or foreign pointer alias in {caller} lacks lifetime bound"),
                    "BoundedPointerLifetimeSafety",
                    "RawPointerHandle",
                ))
            } else {
                None
            }
        } else if line.contains("Pointer<") || line.contains("fromAddress(") {
            Some((
                "ffi",
                format!("Foreign pointer handle in {caller} lacks deterministic finalizer attachment"),
                "FinalizerBackedForeignHandle",
                "ForeignPointer",
            ))
        } else {
            None
        };

        if let Some((tag, symptom, inv, callee)) = violation_opt {
            let lang_tag = if lang == SourceLanguage::Dart { "dart" } else { "rust" };
            let fix_code = if lang == SourceLanguage::Dart {
                "int readBoundedElement(List<int> buffer, int index) {\n  if (index < 0 || index >= buffer.length) {\n    return 0;\n  }\n  return buffer.elementAt(index);\n}"
            } else {
                "pub fn checked_slice_and_narrow_sample(buf: &[u8], wide: u64) -> Option<u8> {\n    let first = buf.first().copied()?;\n    let narrow = u8::try_from(wide).ok()?;\n    Some(first.wrapping_add(narrow))\n}"
            };
            out.push(AuditFinding {
                slug: format!("{}-e0601-{}-l{}", sanitize_slug(rel_path), tag, line_no),
                file_path: rel_path.to_string(),
                start_line: line_no,
                end_line: line_no,
                pillar: AuditPillar::MemorySafety,
                severity: DossierSeverity::Critical,
                symptom,
                test_target: test_target.to_string(),
                whys: vec![
                    ("does the operation risk memory or arithmetic corruption".to_string(), format!("function {caller} executes an unguarded memory index, cast, or pointer operation at line {line_no}")),
                    ("was the unguarded operation permitted to compile".to_string(), "primitive indexing and as-cast operators bypass fallible Result or Option return contracts".to_string()),
                    ("can caller-supplied inputs trigger the boundary fault".to_string(), "upstream callers can supply empty buffers or values exceeding the narrower integer range".to_string()),
                    ("was this missed prior to static AST verification".to_string(), "type-checking alone accepts unchecked indexing and lossy primitive casts without range proofs".to_string()),
                    ("is deterministic static enforcement required".to_string(), "safety-critical code must use .get(..) and TryFrom conversions to fail closed without panicking".to_string()),
                ],
                correctness_trace: format!(
                    "At `{rel_path}:{line_no}-{line_no}`, `{caller}` performs an unguarded memory bounds or numeric conversion step via `{callee}`. When runtime inputs exceed buffer length or target bit width, execution either aborts or propagates silently truncated state."
                ),
                invariant_violated: inv.to_string(),
                uml_kind: UmlDiagramKind::SequenceDiagram,
                caller_symbol: caller.clone(),
                callee_symbol: callee.to_string(),
                affected_callers: vec![
                    (caller, format!("Directly executes unguarded `{callee}` operation at line {line_no}")),
                ],
                language_tag: lang_tag.to_string(),
                proposed_correction_code: fix_code.to_string(),
            });
        }
    }
}

/// Checks whether `line` contains `word` bounded by non-identifier ASCII characters.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides zero-pattern-crate lexical keyword matching under Check 34 purity rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches `word` iff adjacent bytes are neither ASCII alphanumeric nor `'_'`.
///
/// ## 3. Preconditions
/// - `word` is non-empty ASCII.
///
/// ## 4. Postconditions
/// - Returns `true` iff at least one bounded token occurrence exists in `line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|line|)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `.get(..)` byte bounds exclusively; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-CONTAINS-BOUNDED-WORD]
fn contains_bounded_word(line: &str, word: &str) -> bool {
    let bytes = line.as_bytes();
    for (idx, _) in line.match_indices(word) {
        let before_ok = if idx == 0 {
            true
        } else {
            match bytes.get(idx.saturating_sub(1)).copied() {
                Some(b) => !(b.is_ascii_alphanumeric() || b == b'_'),
                None => true,
            }
        };
        let end_idx = idx.saturating_add(word.len());
        let after_ok = match bytes.get(end_idx).copied() {
            Some(b) => !(b.is_ascii_alphanumeric() || b == b'_'),
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Extracts the identifier immediately preceding `'('` on a function or method signature line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Grounds Dart and Rust control-flow findings to exact method names without pattern crates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns non-empty alphanumeric/underscore symbol name (`"AsyncMethod"` fallback).
///
/// ## 3. Preconditions
/// - `masked_line` is a single source line.
///
/// ## 4. Postconditions
/// - Extracts method identifier token prior to parameter list.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|masked\_line|)$. Space Complexity: $O(|masked\_line|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `split_once` without indexing; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-EXTRACT-CALLABLE-NAME]
fn extract_callable_name(masked_line: &str) -> String {
    let before_paren = match masked_line.split_once('(') {
        Some((prefix, _)) => prefix.trim(),
        None => masked_line.trim(),
    };
    let candidate = before_paren
        .split_whitespace()
        .last()
        .unwrap_or("AsyncMethod");
    let cleaned: String = candidate
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if cleaned.is_empty() {
        "AsyncMethod".to_string()
    } else {
        cleaned
    }
}

/// Evaluates Pillar 2 (`E0602` Resource Lifecycle): undisposed Dart controllers/subscriptions and `/tmp/` writes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-06` Section 1.3 Pillar 2 (`E0602`) deterministic resource lifecycle verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Flags `ChangeNotifier` / `State<T>` classes holding managed handles without `dispose()` cleanup.
/// - Flags hardcoded `/tmp/` writes bypassing RAII scoped temporary directories.
///
/// ## 3. Preconditions
/// - `masked_lines` and `comment_stripped_lines` correspond 1-to-1 with source lines.
///
/// ## 4. Postconditions
/// - Appends `E0602` findings to `out`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$. Space Complexity: $O(K_2)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking slice traversal across all lines.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-AUDIT-PILLAR-2-LIFECYCLE]
fn audit_pillar_resource_lifecycle(
    rel_path: &str,
    masked_lines: &[String],
    comment_stripped_lines: &[String],
    lang: SourceLanguage,
    ast_items: &[SourceAstItem],
    test_target: &str,
    out: &mut Vec<AuditFinding>,
) {
    if lang == SourceLanguage::Dart {
        let managed_types = [
            "StreamSubscription",
            "AnimationController",
            "TextEditingController",
            "FocusNode",
            "Timer",
        ];
        let mut idx: usize = 0;
        while let Some(line) = masked_lines.get(idx) {
            if contains_bounded_word(line, "class")
                && (line.contains("ChangeNotifier") || line.contains("State<"))
            {
                let class_name = line
                    .split_once("class ")
                    .and_then(|(_, rest)| rest.split_whitespace().next())
                    .map(|s| {
                        s.chars()
                            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect::<String>()
                    })
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "StatefulComponent".to_string());
                let end_idx = find_brace_block_end(masked_lines, idx);
                let mut found_res: Option<(&str, usize)> = None;
                let mut has_dispose_decl = false;
                let mut has_cleanup_call = false;

                let mut body_idx = idx;
                while body_idx <= end_idx {
                    if let Some(bline) = masked_lines.get(body_idx) {
                        if found_res.is_none() {
                            for &mty in &managed_types {
                                if contains_bounded_word(bline, mty) {
                                    found_res = Some((mty, body_idx.saturating_add(1)));
                                    break;
                                }
                            }
                        }
                        if bline.contains("void dispose(") {
                            has_dispose_decl = true;
                        }
                        if bline.contains(".cancel(")
                            || (bline.contains(".dispose(") && !bline.contains("super.dispose("))
                        {
                            has_cleanup_call = true;
                        }
                    }
                    body_idx = body_idx.saturating_add(1);
                }

                if let Some((res_ty, field_line)) = found_res {
                    if !(has_dispose_decl && has_cleanup_call) {
                        let end_line = end_idx.saturating_add(1);
                        out.push(AuditFinding {
                            slug: format!("{}-e0602-dispose-l{}", sanitize_slug(rel_path), field_line),
                            file_path: rel_path.to_string(),
                            start_line: field_line,
                            end_line,
                            pillar: AuditPillar::ResourceLifecycle,
                            severity: DossierSeverity::Critical,
                            symptom: format!(
                                "Class {class_name} holds {res_ty} handle without cancelling or disposing it in dispose()"
                            ),
                            test_target: test_target.to_string(),
                            whys: vec![
                                ("does the component leak active subscriptions or controllers".to_string(), format!("{class_name} retains a {res_ty} field declared at line {field_line} without overriding dispose()")),
                                ("does omitting dispose() cause runtime faults".to_string(), "active stream callbacks continue firing after widget or notifier teardown".to_string()),
                                ("was the missing lifecycle hook undetected at compile time".to_string(), "Dart ChangeNotifier and State base classes provide a no-op base dispose() implementation".to_string()),
                                ("does the leaked handle corrupt downstream application state".to_string(), "orphaned listeners retain object graphs in memory and invoke notifications on defunct owners".to_string()),
                                ("must deterministic lifecycle cleanup be enforced".to_string(), "every owned subscription, timer, or controller requires deterministic cancellation inside dispose()".to_string()),
                            ],
                            correctness_trace: format!(
                                "In `{rel_path}:{field_line}-{end_line}`, `{class_name}` allocates or stores `{res_ty}` without a matching `void dispose()` teardown routine. When `{class_name}` leaves the active lifecycle scope, the underlying handle remains registered and leaks callbacks and memory."
                            ),
                            invariant_violated: "DeterministicResourceDisposal".to_string(),
                            uml_kind: UmlDiagramKind::ClassDiagram,
                            caller_symbol: class_name.clone(),
                            callee_symbol: res_ty.to_string(),
                            affected_callers: vec![
                                (class_name, format!("Retains undisposed `{res_ty}` across lines {field_line}-{end_line}")),
                            ],
                            language_tag: "dart".to_string(),
                            proposed_correction_code: "@override\nvoid dispose() {\n  _subscription?.cancel();\n  _subscription = null;\n  super.dispose();\n}".to_string(),
                        });
                    }
                }
                idx = end_idx.saturating_add(1);
                continue;
            }
            idx = idx.saturating_add(1);
        }
    }

    for (idx, line) in comment_stripped_lines.iter().enumerate() {
        if line.contains("\"/tmp/")
            && (line.contains("write") || line.contains("File") || line.contains("create"))
        {
            let line_no = idx.saturating_add(1);
            let caller = find_enclosing_symbol(ast_items, line_no);
            let lang_tag = if lang == SourceLanguage::Dart { "dart" } else { "rust" };
            let fix_code = if lang == SourceLanguage::Dart {
                "Future<Directory> createScopedScratchDir() async {\n  return Directory.systemTemp.createTemp('scoped_audit_');\n}"
            } else {
                "pub fn create_unique_scratch_path(label: &str) -> std::path::PathBuf {\n    std::env::temp_dir().join(format!(\"scoped_{label}_{}\", std::process::id()))\n}"
            };
            out.push(AuditFinding {
                slug: format!("{}-e0602-tmp-l{}", sanitize_slug(rel_path), line_no),
                file_path: rel_path.to_string(),
                start_line: line_no,
                end_line: line_no,
                pillar: AuditPillar::ResourceLifecycle,
                severity: DossierSeverity::Important,
                symptom: format!("Hardcoded shared /tmp/ filesystem write in {caller} risks cross-process collision"),
                test_target: test_target.to_string(),
                whys: vec![
                    ("can concurrent test or tool processes collide on disk".to_string(), format!("{caller} writes directly to a fixed /tmp/ path at line {line_no}")),
                    ("is a fixed /tmp/ path non-hermetic".to_string(), "shared global filenames lack process isolation and deterministic RAII teardown".to_string()),
                    ("can residual files poison subsequent runs".to_string(), "early return or failure leaves stale artifacts in the shared directory".to_string()),
                    ("does host portability suffer".to_string(), "hardcoded /tmp/ literals ignore platform-configured temporary directory roots".to_string()),
                    ("must scoped RAII directories be mandated".to_string(), "process-unique scoped directories guarantee collision-free hermetic cleanup".to_string()),
                ],
                correctness_trace: format!(
                    "At `{rel_path}:{line_no}-{line_no}`, `{caller}` targets a hardcoded `/tmp/` path instead of an isolated RAII workspace directory, risking concurrent write races and leaked temporary files."
                ),
                invariant_violated: "HermeticRaiiTemporaryIsolation".to_string(),
                uml_kind: UmlDiagramKind::SequenceDiagram,
                caller_symbol: caller.clone(),
                callee_symbol: "SharedTmpFilesystem".to_string(),
                affected_callers: vec![
                    (caller, format!("Writes to un-isolated shared temporary path at line {line_no}")),
                ],
                language_tag: lang_tag.to_string(),
                proposed_correction_code: fix_code.to_string(),
            });
        }
    }
}

/// Evaluates Pillar 3 (`E0603` Concurrency): post-`await` unguarded Dart notifications and TOCTOU file races.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-06` Section 1.3 Pillar 3 (`E0603`) asynchronous lifecycle and filesystem atomicity checks.
///
/// ## 2. Invariants & Mathematical Properties
/// - Flags Dart `async` methods calling `notifyListeners(` or `setState(` after `await` without `mounted`/`_disposed`/`isDisposed`.
/// - Flags TOCTOU `.exists()` checks followed immediately by non-atomic file open/read/remove calls.
///
/// ## 3. Preconditions
/// - `masked_lines` has comments and string literals masked.
///
/// ## 4. Postconditions
/// - Appends `E0603` findings to `out`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$. Space Complexity: $O(K_3)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking slice traversal across all lines.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-AUDIT-PILLAR-3-CONCURRENCY]
fn audit_pillar_concurrency(
    rel_path: &str,
    masked_lines: &[String],
    non_test_mask: &[bool],
    lang: SourceLanguage,
    ast_items: &[SourceAstItem],
    test_target: &str,
    out: &mut Vec<AuditFinding>,
) {
    if lang == SourceLanguage::Dart {
        let mut idx: usize = 0;
        while let Some(line) = masked_lines.get(idx) {
            let next_has_brace = masked_lines
                .get(idx.saturating_add(1))
                .map(|n| n.contains('{'))
                .unwrap_or(false);
            if contains_bounded_word(line, "async") && (line.contains('{') || next_has_brace) {
                let method_name = extract_callable_name(line);
                let end_idx = find_brace_block_end(masked_lines, idx);
                let mut await_seen = false;
                let mut guarded_since_await = false;

                let mut body_idx = idx;
                while body_idx <= end_idx {
                    if let Some(bline) = masked_lines.get(body_idx) {
                        if contains_bounded_word(bline, "await") {
                            await_seen = true;
                            guarded_since_await = false;
                        } else if await_seen
                            && (contains_bounded_word(bline, "mounted")
                                || bline.contains("_disposed")
                                || bline.contains("isDisposed"))
                        {
                            guarded_since_await = true;
                        }

                        let has_notify = bline.contains("notifyListeners(");
                        let has_set_state = bline.contains("setState(");
                        if await_seen && !guarded_since_await && (has_notify || has_set_state) {
                            let callee = if has_set_state {
                                "setState"
                            } else {
                                "notifyListeners"
                            };
                            let start_line = idx.saturating_add(1);
                            let call_line = body_idx.saturating_add(1);
                            out.push(AuditFinding {
                                slug: format!("{}-e0603-async-l{}", sanitize_slug(rel_path), call_line),
                                file_path: rel_path.to_string(),
                                start_line,
                                end_line: call_line,
                                pillar: AuditPillar::Concurrency,
                                severity: DossierSeverity::Critical,
                                symptom: format!(
                                    "Async method {method_name} invokes {callee}() after await suspension without disposal or mounted guard"
                                ),
                                test_target: test_target.to_string(),
                                whys: vec![
                                    ("does the async callback throw after component teardown".to_string(), format!("{method_name} resumes after an await suspension and calls {callee}() at line {call_line} without checking disposal state")),
                                    ("can teardown occur while the Future is suspended".to_string(), "UI navigation or controller disposal can complete on the event loop before the awaited Future resolves".to_string()),
                                    ("did the compiler accept the unguarded post-await call".to_string(), "Dart static type checking does not track object lifecycle transitions across async suspension points".to_string()),
                                    ("does invoking notifications on a disposed owner cause unhandled faults".to_string(), "ChangeNotifier and State assert active lifecycle ownership and throw unhandled exceptions once disposed".to_string()),
                                    ("must post-await lifecycle guards be enforced".to_string(), "every state mutation or listener dispatch after await must fail closed behind a mounted or disposal check".to_string()),
                                ],
                                correctness_trace: format!(
                                    "Across `{rel_path}:{start_line}-{call_line}`, `{method_name}` suspends at an `await` boundary and subsequently invokes `{callee}()` without verifying `mounted` or `_disposed`. If the owning instance is disposed during suspension, resumption triggers a use-after-dispose exception."
                                ),
                                invariant_violated: "PostAwaitLifecycleGuard".to_string(),
                                uml_kind: UmlDiagramKind::SequenceDiagram,
                                caller_symbol: method_name.clone(),
                                callee_symbol: callee.to_string(),
                                affected_callers: vec![
                                    (method_name, format!("Invokes unguarded `{callee}()` at line {call_line} after `await`")),
                                ],
                                language_tag: "dart".to_string(),
                                proposed_correction_code: "Future<void> refreshAfterAwait(Future<int> pendingValue) async {\n  final resolved = await pendingValue;\n  if (_disposed) {\n    return;\n  }\n  latestValue = resolved;\n  notifyListeners();\n}".to_string(),
                            });
                            break;
                        }
                    }
                    body_idx = body_idx.saturating_add(1);
                }
            }
            idx = idx.saturating_add(1);
        }
    }

    for (idx, line) in masked_lines.iter().enumerate() {
        if lang == SourceLanguage::Rust && non_test_mask.get(idx).copied() != Some(true) {
            continue;
        }
        if line.contains(".exists()") {
            let mut look = idx.saturating_add(1);
            let limit = idx.saturating_add(4);
            while look <= limit {
                if let Some(next_line) = masked_lines.get(look) {
                    if next_line.contains("read_to_string(")
                        || next_line.contains("File::open(")
                        || next_line.contains("remove_file(")
                        || next_line.contains("readAsString(")
                    {
                        let start_line = idx.saturating_add(1);
                        let end_line = look.saturating_add(1);
                        let caller = find_enclosing_symbol(ast_items, start_line);
                        let lang_tag = if lang == SourceLanguage::Dart { "dart" } else { "rust" };
                        let fix_code = if lang == SourceLanguage::Dart {
                            "Future<String?> readAtomicIfPresent(File target) async {\n  try {\n    return await target.readAsString();\n  } on FileSystemException {\n    return null;\n  }\n}"
                        } else {
                            "pub fn read_atomic_if_present(path: &std::path::Path) -> Result<Option<String>, std::io::Error> {\n    match std::fs::read_to_string(path) {\n        Ok(content) => Ok(Some(content)),\n        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),\n        Err(err) => Err(err),\n    }\n}"
                        };
                        out.push(AuditFinding {
                            slug: format!("{}-e0603-toctou-l{}", sanitize_slug(rel_path), start_line),
                            file_path: rel_path.to_string(),
                            start_line,
                            end_line,
                            pillar: AuditPillar::Concurrency,
                            severity: DossierSeverity::Important,
                            symptom: format!(
                                "TOCTOU filesystem check-then-use sequence in {caller} risks concurrent race window"
                            ),
                            test_target: test_target.to_string(),
                            whys: vec![
                                ("can filesystem access fail despite a prior existence check".to_string(), format!("{caller} checks .exists() at line {start_line} before opening or removing the path at line {end_line}")),
                                ("is a separate existence probe non-atomic".to_string(), "another OS process or thread can unlink or replace the path between the two syscalls".to_string()),
                                ("do developers write check-then-open sequences".to_string(), "boolean existence helpers appear simpler than matching ErrorKind::NotFound directly".to_string()),
                                ("does the race window compromise reliability".to_string(), "concurrent directory mutations turn expected NotFound states into unhandled I/O failures".to_string()),
                                ("must atomic open-and-match semantics be used".to_string(), "performing the file operation directly and matching NotFound eliminates the race window".to_string()),
                            ],
                            correctness_trace: format!(
                                "Between `{rel_path}:{start_line}-{end_line}`, `{caller}` queries `.exists()` and then performs a separate filesystem syscall, introducing a Time-of-Check to Time-of-Use race window."
                            ),
                            invariant_violated: "AtomicFilesystemErrorHandling".to_string(),
                            uml_kind: UmlDiagramKind::SequenceDiagram,
                            caller_symbol: caller.clone(),
                            callee_symbol: "FilesystemToctouWindow".to_string(),
                            affected_callers: vec![
                                (caller, format!("Executes non-atomic check-then-use file sequence across lines {start_line}-{end_line}")),
                            ],
                            language_tag: lang_tag.to_string(),
                            proposed_correction_code: fix_code.to_string(),
                        });
                        break;
                    }
                }
                look = look.saturating_add(1);
            }
        }
    }
}

/// Evaluates Pillar 4 (`E0604` Test Integrity): zero-assertion `#[test]` functions and vacuous early returns.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-06` Section 1.3 Pillar 4 (`E0604`) DO-330 TQL-4 test oracle verification across all `#[test]` blocks.
///
/// ## 2. Invariants & Mathematical Properties
/// - Scans full masked Rust source directly so `#[cfg(test)]` modules skipped by `parse_rust_source_ast` are audited.
/// - Flags `#[test]` functions lacking assertion macros/predicates or containing vacuous `if !...exists() { return; }` skips.
///
/// ## 3. Preconditions
/// - `masked_lines` has comments and string literals masked.
///
/// ## 4. Postconditions
/// - Appends `E0604` findings to `out`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$. Space Complexity: $O(K_4)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking slice traversal across all lines.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-AUDIT-PILLAR-4-TESTS]
fn audit_pillar_test_integrity(
    rel_path: &str,
    masked_lines: &[String],
    lang: SourceLanguage,
    test_target: &str,
    out: &mut Vec<AuditFinding>,
) {
    if lang != SourceLanguage::Rust {
        return;
    }
    let mut idx: usize = 0;
    while let Some(line) = masked_lines.get(idx) {
        let trimmed = line.trim();
        if trimmed == "#[test]" || trimmed.starts_with("#[test]") || trimmed.contains("::test]") {
            let mut fn_idx = idx.saturating_add(1);
            let search_limit = idx.saturating_add(12);
            let mut found_fn: Option<usize> = None;
            while fn_idx <= search_limit {
                if let Some(fline) = masked_lines.get(fn_idx) {
                    if contains_bounded_word(fline, "fn") {
                        found_fn = Some(fn_idx);
                        break;
                    }
                } else {
                    break;
                }
                fn_idx = fn_idx.saturating_add(1);
            }

            if let Some(f_idx) = found_fn {
                let fn_line_str = masked_lines.get(f_idx).map(String::as_str).unwrap_or("");
                let fn_name = extract_callable_name(fn_line_str);
                let end_idx = find_brace_block_end(masked_lines, f_idx);
                let mut has_real_assertion = false;
                let mut has_vacuous_skip = false;

                let mut body_idx = f_idx;
                while body_idx <= end_idx {
                    if let Some(bline) = masked_lines.get(body_idx) {
                        let bt = bline.trim();
                        let line_has_assert = bt.contains("assert!(")
                            || bt.contains("assert_eq!(")
                            || bt.contains("assert_ne!(")
                            || bt.contains("debug_assert!(")
                            || bt.contains(".is_ok()")
                            || bt.contains(".is_err()");
                        let is_tautology =
                            bt.contains("assert!(true)") || bt.contains("size_of::<");
                        if line_has_assert && !is_tautology {
                            has_real_assertion = true;
                        }
                        let next_returns = masked_lines
                            .get(body_idx.saturating_add(1))
                            .map(|n| contains_bounded_word(n, "return"))
                            .unwrap_or(false);
                        if bt.starts_with("if !")
                            && bt.contains(".exists()")
                            && (contains_bounded_word(bt, "return") || next_returns)
                        {
                            has_vacuous_skip = true;
                        }
                    }
                    body_idx = body_idx.saturating_add(1);
                }

                if !has_real_assertion || has_vacuous_skip {
                    let start_line = f_idx.saturating_add(1);
                    let end_line = end_idx.saturating_add(1);
                    let symptom = if has_vacuous_skip {
                        format!(
                            "Test function {fn_name} silently returns early when fixture path is absent instead of failing closed"
                        )
                    } else {
                        format!(
                            "Test function {fn_name} executes without any behavioral state or output assertion macro"
                        )
                    };
                    out.push(AuditFinding {
                        slug: format!("{}-e0604-test-l{}", sanitize_slug(rel_path), start_line),
                        file_path: rel_path.to_string(),
                        start_line,
                        end_line,
                        pillar: AuditPillar::TestIntegrity,
                        severity: DossierSeverity::Important,
                        symptom,
                        test_target: test_target.to_string(),
                        whys: vec![
                            ("can the unit test pass even when target behavior regresses".to_string(), format!("test function {fn_name} at lines {start_line}-{end_line} lacks an active fail-closed assertion oracle")),
                            ("did the test runner report green status".to_string(), "Rust test harnesses mark any #[test] function returning () without panicking as passed".to_string()),
                            ("do zero-assertion or early-return tests enter suites".to_string(), "smoke probes or conditional fixture guards omit explicit postcondition assertions".to_string()),
                            ("does hollow test coverage create safety blind spots".to_string(), "coverage metrics count executed lines while behavioral regressions pass silently".to_string()),
                            ("must deterministic assertion checks be enforced".to_string(), "every #[test] function must verify concrete postconditions and fail closed on missing fixtures".to_string()),
                        ],
                        correctness_trace: format!(
                            "In `{rel_path}:{start_line}-{end_line}`, `#[test]` function `{fn_name}` completes execution without evaluating a non-tautological assertion predicate (or exits early via a vacuous `.exists()` skip), yielding a false-positive green test result."
                        ),
                        invariant_violated: "NonVacuousTestAssertionOracle".to_string(),
                        uml_kind: UmlDiagramKind::SequenceDiagram,
                        caller_symbol: fn_name.clone(),
                        callee_symbol: "AssertionOracle".to_string(),
                        affected_callers: vec![
                            (fn_name, format!("Reports false-positive test pass across lines {start_line}-{end_line}")),
                        ],
                        language_tag: "rust".to_string(),
                        proposed_correction_code: "fn verified_state_transition_probe() {\n    let computed = 2_u32.saturating_add(2);\n    assert_eq!(computed, 4);\n}".to_string(),
                    });
                }
                idx = end_idx.saturating_add(1);
                continue;
            }
        }
        idx = idx.saturating_add(1);
    }
}

/// Determines whether a masked Rust line declares a public `struct`, `enum`, `trait`, or `fn`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Scopes Pillar 5 (`E0605` Semantic Traceability) strictly to public API contracts under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Excludes crate-internal `pub(crate)` / `pub(super)` items while matching `pub` classifiers and functions.
///
/// ## 3. Preconditions
/// - `masked_line` has comments and string literals masked.
///
/// ## 4. Postconditions
/// - Returns `true` iff `masked_line` begins a public struct, enum, trait, or function declaration.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|masked\_line|)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Prefix matching only; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-IS-PUBLIC-RUST-DECL]
fn is_public_rust_declaration(masked_line: &str) -> bool {
    let trimmed = masked_line.trim_start();
    trimmed.starts_with("pub struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub const fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("pub unsafe fn ")
}

/// Checks contiguous preceding Rust doc-comment and attribute lines above `item_line_1based` for `Realises: [`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Verifies bijective requirement-to-code traceability tags under Pillar 5 (`E0605`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Traverses upward across `///`, `/**`, `*`, and `#[...]` lines until a blank or code boundary.
///
/// ## 3. Preconditions
/// - `raw_lines` contains the unmasked lines of the Rust source file.
///
/// ## 4. Postconditions
/// - Returns `true` iff any attached doc line contains `"Realises: ["` or `"Realises:["`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(D)$ over attached docstring line count $D$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `checked_sub` and `.get(..)` exclusively; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-HAS-PRECEDING-REALISES]
fn has_preceding_realises_doc(raw_lines: &[&str], item_line_1based: usize) -> bool {
    if item_line_1based <= 1 {
        return false;
    }
    let mut cur = item_line_1based.saturating_sub(2);
    while let Some(raw) = raw_lines.get(cur) {
        let trimmed = raw.trim();
        if trimmed.starts_with("///")
            || trimmed.starts_with("/**")
            || trimmed.starts_with('*')
            || trimmed.starts_with("#[doc")
        {
            if trimmed.contains("Realises: [") || trimmed.contains("Realises:[") {
                return true;
            }
        } else if trimmed.starts_with("#[") {
            // Continue upward past item attributes attached below doc comments.
        } else {
            break;
        }
        let Some(prev) = cur.checked_sub(1) else {
            break;
        };
        cur = prev;
    }
    false
}

/// Evaluates Pillar 5 (`E0605` Semantic Traceability): public Rust AST items lacking `Realises: [REQ-...]`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-06` Section 1.3 Pillar 5 (`E0605`) requirement traceability verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Inspects `SourceAstItem` nodes extracted by `parse_rust_source_ast` in non-test production spans.
///
/// ## 3. Preconditions
/// - `raw_lines`, `masked_lines`, and `non_test_mask` share identical line lengths.
///
/// ## 4. Postconditions
/// - Appends `E0605` findings to `out` for untraced public declarations.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V \cdot D)$ over AST items $V$ and docstring lines $D$. Space Complexity: $O(K_5)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking slice inspection across all AST items.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-AUDIT-PILLAR-5-TRACEABILITY]
fn audit_pillar_semantic_traceability(
    rel_path: &str,
    raw_lines: &[&str],
    masked_lines: &[String],
    non_test_mask: &[bool],
    ast_items: &[SourceAstItem],
    test_target: &str,
    out: &mut Vec<AuditFinding>,
) {
    for item in ast_items {
        let idx0 = item.line_no.saturating_sub(1);
        if non_test_mask.get(idx0).copied() != Some(true) {
            continue;
        }
        let decl_line = masked_lines.get(idx0).map(String::as_str).unwrap_or("");
        if !is_public_rust_declaration(decl_line) {
            continue;
        }
        if !item.satisfied_requirements.is_empty()
            || has_preceding_realises_doc(raw_lines, item.line_no)
        {
            continue;
        }
        let end_idx = find_brace_block_end(masked_lines, idx0);
        let start_line = item.line_no.max(1);
        let end_line = end_idx.saturating_add(1).max(start_line);
        let sym = if item.name.trim().is_empty() {
            "PublicDeclaration".to_string()
        } else {
            item.name.trim().to_string()
        };
        out.push(AuditFinding {
            slug: format!("{}-e0605-trace-l{}", sanitize_slug(rel_path), start_line),
            file_path: rel_path.to_string(),
            start_line,
            end_line,
            pillar: AuditPillar::SemanticTraceability,
            severity: DossierSeverity::Important,
            symptom: format!(
                "Public declaration {sym} omits mandatory Realises requirement traceability binding in docstring"
            ),
            test_target: test_target.to_string(),
            whys: vec![
                ("is the public API item unlinked from upstream system requirements".to_string(), format!("public declaration {sym} at line {start_line} carries zero Realises requirement tags")),
                ("did compilation succeed without traceability metadata".to_string(), "standard Rust compiler passes ignore missing requirement tags inside rustdoc comments".to_string()),
                ("do untraced public symbols accumulate during implementation".to_string(), "utility or helper functions are exposed publicly without binding them to an approved requirement ID".to_string()),
                ("does missing traceability break DO-178C bijective verification".to_string(), "untraced public code represents dead or unverified functionality outside the requirement matrix".to_string()),
                ("must static AST traceability checks be enforced".to_string(), "every public struct, enum, trait, and function must declare an explicit Realises requirement tag".to_string()),
            ],
            correctness_trace: format!(
                "At `{rel_path}:{start_line}-{end_line}`, public Rust AST item `{sym}` has an empty `satisfied_requirements` set and no `Realises: [REQ-...]` tag in its preceding documentation block, breaking bijective requirement-to-source traceability."
            ),
            invariant_violated: "BijectiveRequirementRealisesTag".to_string(),
            uml_kind: UmlDiagramKind::ClassDiagram,
            caller_symbol: sym.clone(),
            callee_symbol: "RequirementTraceabilityMatrix".to_string(),
            affected_callers: vec![
                (sym, format!("Exposes untraced public API surface across lines {start_line}-{end_line}")),
            ],
            language_tag: "rust".to_string(),
            proposed_correction_code: "/// Advances bounded counter by one saturating step.\n///\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-0606-TRACED-COUNTER-STEP]\npub fn traced_counter_step(current: u32) -> u32 {\n    current.saturating_add(1)\n}".to_string(),
        });
    }
}

/// Evaluates all 5 correctness pillars (`E0601`..`E0605`) deterministically over a single source file.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Primary per-file static AST evaluation entrypoint for `BP-06` (`REQ-0606`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Evaluates pillars in canonical `AuditPillar::ALL` order, respecting optional single-pillar `filter`.
///
/// ## 3. Preconditions
/// - `source_text` is valid UTF-8 and `ast_items` corresponds to `source_text`.
///
/// ## 4. Postconditions
/// - Returns deterministic `Vec<AuditFinding>` for `rel_path`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N + V)$. Space Complexity: $O(N + K)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure in-memory slice analysis; infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-EVALUATE-SOURCE-FILE-PILLARS]
pub fn evaluate_source_file_pillars(
    workspace_root: &Path,
    rel_path: &str,
    source_text: &str,
    lang: SourceLanguage,
    ast_items: &[SourceAstItem],
    filter: Option<AuditPillar>,
) -> Vec<AuditFinding> {
    let raw_lines: Vec<&str> = source_text.lines().collect();
    let masked_lines = mask_source_lines(source_text, true);
    let comment_stripped_lines = mask_source_lines(source_text, false);
    let non_test_mask = if lang == SourceLanguage::Rust {
        compute_rust_non_test_mask(&masked_lines)
    } else {
        vec![true; masked_lines.len()]
    };
    let test_target = resolve_canonical_test_target(workspace_root, rel_path);
    let mut findings = Vec::new();

    for &pillar in AuditPillar::ALL {
        if let Some(active) = filter {
            if active != pillar {
                continue;
            }
        }
        match pillar {
            AuditPillar::MemorySafety => {
                audit_pillar_memory_safety(
                    rel_path,
                    &masked_lines,
                    &non_test_mask,
                    lang,
                    ast_items,
                    &test_target,
                    &mut findings,
                );
            }
            AuditPillar::ResourceLifecycle => {
                audit_pillar_resource_lifecycle(
                    rel_path,
                    &masked_lines,
                    &comment_stripped_lines,
                    lang,
                    ast_items,
                    &test_target,
                    &mut findings,
                );
            }
            AuditPillar::Concurrency => {
                audit_pillar_concurrency(
                    rel_path,
                    &masked_lines,
                    &non_test_mask,
                    lang,
                    ast_items,
                    &test_target,
                    &mut findings,
                );
            }
            AuditPillar::TestIntegrity => {
                audit_pillar_test_integrity(
                    rel_path,
                    &masked_lines,
                    lang,
                    &test_target,
                    &mut findings,
                );
            }
            AuditPillar::SemanticTraceability => {
                if lang == SourceLanguage::Rust {
                    audit_pillar_semantic_traceability(
                        rel_path,
                        &raw_lines,
                        &masked_lines,
                        &non_test_mask,
                        ast_items,
                        &test_target,
                        &mut findings,
                    );
                }
            }
        }
    }

    findings
}

/// Sanitizes an arbitrary symbol into a pure ASCII PascalCase Mermaid identifier (`[A-Z][A-Za-z0-9]*`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Eliminates Mermaid parser syntax hazards (underscores, angle brackets, colons, semicolons) under Check 13B.
///
/// ## 2. Invariants & Mathematical Properties
/// - Output is non-empty, starts with an uppercase ASCII letter, and contains strictly `[A-Za-z0-9]`.
///
/// ## 3. Preconditions
/// - `fallback` starts with an uppercase ASCII letter and contains strictly `[A-Za-z0-9]`.
///
/// ## 4. Postconditions
/// - Returns pure alphanumeric PascalCase identifier.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|raw|)$. Space Complexity: $O(|raw|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back to `fallback` when `raw` contains no ASCII letters.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-SANITIZE-MERMAID-ID]
fn sanitize_mermaid_identifier(raw: &str, fallback: &str) -> String {
    let mut out = String::with_capacity(raw.len().saturating_add(4));
    let mut cap_next = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphabetic() {
            if cap_next {
                out.push(ch.to_ascii_uppercase());
                cap_next = false;
            } else {
                out.push(ch);
            }
        } else if ch.is_ascii_digit() {
            if !out.is_empty() {
                out.push(ch);
            }
        } else {
            cap_next = true;
        }
    }
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

/// Synthesizes a Check 13B-compliant Section 4 Mermaid diagram block with zero semicolons.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Guarantees visual architectural traceability for `Critical` and `Important` defect dossiers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Emits zero `;` characters, zero `{}` or `:` on class member lines, and quoted labels.
///
/// ## 3. Preconditions
/// - `finding` carries populated `caller_symbol`, `callee_symbol`, and `invariant_violated`.
///
/// ## 4. Postconditions
/// - Returns fenced Mermaid markdown or `N/A -- <Severity> severity.` string.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S)$ over symbol lengths $S$. Space Complexity: $O(S)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Deduplicates identical caller/callee class identifiers automatically.
///
/// ## 7. Realization & Traceability
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Produces standardized DO-178C Level A / Leveson STPA defect dossiers (`BP-06` Section 6.2).
///
/// ## 2. Invariants & Mathematical Properties
/// - Line 1 equals `## 1. Context and References` with matching HTML comment and bullet `Test-Target`.
/// - Emits all 7 canonical `DossierSection` headers plus `## Audit Source` footer.
///
/// ## 3. Preconditions
/// - `finding` has `whys.len() == 5` and non-empty `affected_callers`.
///
/// ## 4. Postconditions
/// - Returns complete CommonMark defect dossier string ready for `validate_defect_dossier_scoped`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ over finding text size $L$. Space Complexity: $O(L)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure deterministic string assembly; infallible.
///
/// ## 7. Realization & Traceability
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
    out.push_str("\n\n");
    out.push_str(&format!("<!-- test-target: {} -->\n\n", finding.test_target));
    out.push_str(&format!("- **File**: `{file_loc}`\n"));
    out.push_str(&format!(
        "- **Pillar**: {}\n",
        finding.pillar.formatted_header()
    ));
    out.push_str(&format!("- **Symptom**: {}\n", finding.symptom));
    out.push_str(&format!("- **Test-Target**: `{}`\n\n", finding.test_target));

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
    out.push_str("\n\n");
    out.push_str(&format!(
        "{}\n\n- **Violated Invariant**: `{}` (`{}`)\n\n",
        finding.correctness_trace,
        finding.invariant_violated,
        finding.pillar.diagnostic_code()
    ));

    out.push_str(DossierSection::UmlDiagrams.header());
    out.push_str("\n\n");
    out.push_str(&uml_block);
    out.push_str("\n\n");

    out.push_str(DossierSection::AffectedCallers.header());
    out.push_str("\n\n");
    out.push_str(&callers_block);
    out.push_str("\n\n");

    out.push_str(DossierSection::ProposedCorrection.header());
    out.push_str("\n\n");
    out.push_str(&format!(
        "```{}\n{}\n```\n\n",
        finding.language_tag,
        finding.proposed_correction_code.trim()
    ));

    out.push_str(DossierSection::RelationshipToExistingIssues.header());
    out.push_str("\n\n");
    out.push_str("Discovered in deterministic static 5-pillar AST audit -- new finding.\n\n");

    out.push_str("## Audit Source\n\n");
    out.push_str(&format!("Adversarial {} Audit\n", finding.pillar.as_str()));
    out.push_str(&format!("SEVERITY: {}\n", finding.severity.as_str()));
    out.push_str(&format!("FILE_LOCATION: {file_loc}\n"));
    out
}

/// Scans `target_path` deterministically, validating balanced syntax and evaluating all requested pillars.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces fail-closed missing-path and syntax gating (`AC-04`) prior to static AST evaluation.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects non-existent `target_path` and unbalanced `.rs`/`.dart` delimiter syntax with `Err(1)`.
///
/// ## 3. Preconditions
/// - `workspace_root` identifies the enclosing project root.
///
/// ## 4. Postconditions
/// - Returns deterministically ordered `Vec<AuditFinding>` across all discovered source files.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(F \log F + N + V)$. Space Complexity: $O(F + N + V + K)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Emits explicit `stderr` diagnostics and returns `Err(1)` on any path, I/O, or syntax fault.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-SCAN-WORKSPACE-DETERMINISTIC]
fn scan_workspace_path_deterministic(
    workspace_root: &Path,
    target_path: &Path,
    filter: Option<AuditPillar>,
) -> Result<Vec<AuditFinding>, i32> {
    let resolved_target = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        workspace_root.join(target_path)
    };
    if !resolved_target.exists() {
        eprintln!(
            "[AUDIT-CODE PATH ERROR] Target path does not exist: {}",
            resolved_target.display()
        );
        return Err(1);
    }
    let mut files = Vec::new();
    collect_target_source_files(&resolved_target, &mut files)?;

    let mut all_findings = Vec::new();
    for file_path in files {
        let rel_path = normalize_rel_path(workspace_root, &file_path);
        let source_text = match fs::read_to_string(&file_path) {
            Ok(txt) => txt,
            Err(err) => {
                eprintln!(
                    "[AUDIT-CODE IO ERROR] Unable to read {}: {err}",
                    file_path.display()
                );
                return Err(1);
            }
        };
        match validate_source_syntax(&source_text, &rel_path) {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[AUDIT-CODE SYNTAX ERROR] {msg}");
                return Err(1);
            }
        }
        let (lang, ast_items) = if rel_path.ends_with(".dart") {
            let parsed = match parse_dart_source_ast(&source_text, &rel_path) {
                Ok(items) => items,
                Err(msg) => {
                    eprintln!("[AUDIT-CODE SYNTAX ERROR] {msg}");
                    return Err(1);
                }
            };
            (SourceLanguage::Dart, parsed)
        } else {
            let parsed = match parse_rust_source_ast(&source_text, &rel_path) {
                Ok(items) => items,
                Err(msg) => {
                    eprintln!("[AUDIT-CODE SYNTAX ERROR] {msg}");
                    return Err(1);
                }
            };
            (SourceLanguage::Rust, parsed)
        };
        let file_findings = evaluate_source_file_pillars(
            workspace_root,
            &rel_path,
            &source_text,
            lang,
            &ast_items,
            filter,
        );
        all_findings.extend(file_findings);
    }
    Ok(all_findings)
}

/// Checks whether a `(file_location, pillar)` finding is already persisted under `defects_dir`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-06` `AC-03` idempotent local SSOT deduplication without external subprocesses.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches existing `.md` dossiers containing identical `FILE_LOCATION:` and `Adversarial <Pillar> Audit` markers.
///
/// ## 3. Preconditions
/// - `defects_dir` is a valid directory path.
///
/// ## 4. Postconditions
/// - Returns `Ok(true)` iff an existing local dossier records the same location and pillar.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(M \cdot B)$ over existing dossier count $M$ and byte size $B$. Space Complexity: $O(B)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Handles directory and file I/O errors explicitly via `Err(1)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-IS-ALREADY-RECORDED-LOCALLY]
fn is_already_recorded_locally(
    defects_dir: &Path,
    file_location: &str,
    pillar: AuditPillar,
) -> Result<bool, i32> {
    if !defects_dir.exists() {
        return Ok(false);
    }
    let entries = match fs::read_dir(defects_dir) {
        Ok(rd) => rd,
        Err(err) => {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Unable to read defects dir {}: {err}",
                defects_dir.display()
            );
            return Err(1);
        }
    };
    let loc_marker = format!("FILE_LOCATION: {file_location}");
    let pillar_marker = format!("Adversarial {} Audit", pillar.as_str());
    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                eprintln!("[AUDIT-CODE IO ERROR] Defects entry error: {err}");
                return Err(1);
            }
        };
        let path = entry.path();
        let is_md = path
            .file_name()
            .map(|n| n.to_string_lossy().ends_with(".md"))
            .unwrap_or(false);
        if !path.is_file() || !is_md {
            continue;
        }
        let body = match fs::read_to_string(&path) {
            Ok(b) => b,
            Err(err) => {
                eprintln!(
                    "[AUDIT-CODE IO ERROR] Unable to read dossier {}: {err}",
                    path.display()
                );
                return Err(1);
            }
        };
        if body.contains(&loc_marker) && body.contains(&pillar_marker) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Executes the end-to-end deterministic 5-pillar static audit, 12-check validation, persistence, and SSOT publish gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Primary CLI execution handler for `verify-baseline audit-code` (`BP-06` / `REQ-0606`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Enforces hermetic offline isolation (`AC-05`): zero `git` or `gh` subprocesses run when `args.dry_run` is `true`
///   or when `!args.publish && args.issue.is_none()`.
/// - Validates every synthesized dossier in-process via `validate_defect_dossier_scoped` prior to disk write.
///
/// ## 3. Preconditions
/// - `workspace_root` identifies the target workspace root and `args` holds parsed CLI flags.
///
/// ## 4. Postconditions
/// - Persists deduplicated `.pipeline/defects/DEFECT-<slug>.md` artifacts and optionally publishes closed-loop issues.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N + V + K)$. Space Complexity: $O(N + V + K)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err(1)` on missing path, syntax error, I/O failure, 12-check schema violation, or remote parity mismatch.
///
/// ## 7. Realization & Traceability
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
        None => {
            if is_upstream_ws {
                Some("upstream")
            } else {
                Some("local-offline/workspace")
            }
        }
    };

    let synthesizer = DeterministicDossierSynthesizer::default();
    let gate = ClosedLoopDefectPublisherGate::default();

    for finding in &findings {
        let dossier_md = synthesizer.synthesize_defect_dossier_markdown(finding);
        let validation =
            match gate.validate_scoped(&dossier_md, effective_repo, Some(workspace_root)) {
                Ok(v) => v,
                Err(errs) => {
                    for err_msg in errs {
                        eprintln!("[AUDIT-CODE SCHEMA ERROR] {err_msg}");
                    }
                    return Err(1);
                }
            };

        if is_already_recorded_locally(&defects_dir, &validation.file_location, finding.pillar)?
            && args.issue.is_none()
        {
            continue;
        }

        let dossier_path = defects_dir.join(format!("DEFECT-{}.md", finding.slug));
        if let Err(err) = fs::write(&dossier_path, &dossier_md) {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Failed to write {}: {err}",
                dossier_path.display()
            );
            return Err(1);
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies self-hosting 5-pillar static cleanliness of `audit_code.rs` and CLI pillar parsing.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-330 TQL-4 self-hosting static analyzer verification for `BP-06` (`REQ-0606`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `audit_code.rs` emits zero findings across Pillars 1..5 (`E0601`..`E0605`).
    /// - `AuditPillar::from_cli_token` accepts all 6 valid tokens and rejects invalid tokens.
    ///
    /// ## 3. Preconditions
    /// - `audit_code.rs` source is embedded at compile time via `include_str!`.
    ///
    /// ## 4. Postconditions
    /// - All assertions hold without panics or warnings.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$ over `audit_code.rs` byte length $N$. Space Complexity: $O(N)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Fails test immediately if any self-hosting defect or token mapping regression occurs.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0606-SELF-HOSTING-UNIT-TEST]
    #[test]
    fn test_audit_code_self_hosting_cleanliness_and_pillar_tokens() {
        let src = include_str!("audit_code.rs");
        let analyzer = FivePillarStaticAstAnalyzer::default();
        let findings = analyzer.evaluate_source_file_pillars(
            Path::new("."),
            "crates/verify-baseline/src/audit_code.rs",
            src,
            None,
        );
        assert!(
            findings.is_empty(),
            "Expected audit_code.rs to have 0 self-hosting findings, got: {findings:?}"
        );
        assert_eq!(AuditPillar::from_cli_token("all"), Ok(None));
        assert_eq!(
            AuditPillar::from_cli_token("memory-safety"),
            Ok(Some(AuditPillar::MemorySafety))
        );
        assert_eq!(
            AuditPillar::from_cli_token("resource-lifecycle"),
            Ok(Some(AuditPillar::ResourceLifecycle))
        );
        assert_eq!(
            AuditPillar::from_cli_token("concurrency"),
            Ok(Some(AuditPillar::Concurrency))
        );
        assert_eq!(
            AuditPillar::from_cli_token("test-integrity"),
            Ok(Some(AuditPillar::TestIntegrity))
        );
        assert_eq!(
            AuditPillar::from_cli_token("semantic-traceability"),
            Ok(Some(AuditPillar::SemanticTraceability))
        );
        assert!(AuditPillar::from_cli_token("unknown-pillar").is_err());
    }
}

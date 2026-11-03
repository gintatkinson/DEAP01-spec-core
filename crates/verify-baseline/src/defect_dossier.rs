//! Native Rust 12-check adversarial defect dossier validator, dual-location `.pipeline/defects/`
//! persistence engine, and closed-loop GitHub SSOT synchronization gate (refs #465, #466, #467, #468).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C problem reporting
//! and corrective action verification standards. Enforces all 12 Adversarial Code Auditor Step D
//! schema and Mermaid invariants natively in Rust without regular expressions, persists validated
//! dossiers into `.pipeline/defects/`, synchronizes issues to GitHub SSOT via `gh issue create` or
//! `gh issue edit`, and executes a mandatory Closed-Loop Payload Verification Gate via `gh issue view`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero-H1 Body Invariant: The first non-empty line of a defect dossier must be
//!   `## 1. Context and References`; leading `# ` H1 headers are unconditionally rejected.
//! - Exact 7-Section + Audit Source Topology: All seven canonical `DossierSection` headers and
//!   `## Audit Source` must appear in strict ascending line order outside fenced code blocks.
//! - Bidirectional Test-Target Parity: Section 1 must contain both `<!-- test-target: <path> -->`
//!   and `- **Test-Target**: <path>` with identical normalized target paths.
//! - Closed-Loop SSOT Equivalence: After `gh issue create` or `gh issue edit`, remote `title` and
//!   CRLF-normalized `body` fetched via `gh issue view --json title,body,url` must equal the local
//!   `.pipeline/defects/` payload byte-for-byte.
//!
//! ## 3. Preconditions
//! - Input markdown payloads are valid UTF-8 strings or accessible filesystem paths.
//! - Whenever `--dry-run` is not passed, the `gh` CLI is installed and
//!   authenticated against the target repository.
//!
//! ## 4. Postconditions
//! - Returns `Ok(DefectSchemaValidation)` from `validate_defect_dossier` if and only if all 12
//!   structural, lexical, and Mermaid invariants hold.
//! - Returns `Ok(())` from `run_check_defect` only after validation (and, whenever `--dry-run` is not
//!   passed, local `.pipeline/defects/` persistence and remote closed-loop verification) succeeds.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear scan over dossier byte length $N$ for all 12 validation gates,
//!   plus bounded subprocess invocation latency whenever `--dry-run` is not passed.
//! - Space Complexity: $O(N)$ auxiliary line slice and diagnostic buffer allocation.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - All 12 validation checks, filesystem persistence operations, and `gh` CLI subprocess gates
//!   fail closed, returning explicit diagnostic strings or non-zero exit code `Err(1)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-DEFECT-DOSSIER], [DO-330-TQL-4-DEFECT-GATE]

use std::path::{Path, PathBuf};
use std::process::Command;

pub use crate::defect_dossier_publish::publish_or_edit_github_issue;

/// Canonical 7-section headers required in every Adversarial Code Auditor defect dossier.
/// Realises: [REQ-DEFECT-DOSSIER-SECTION-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DossierSection {
    /// Section 1: `## 1. Context and References`.
    ContextAndReferences,
    /// Section 2: `## 2. Root Cause Analysis (5 Whys)`.
    RootCauseAnalysis,
    /// Section 3: `## 3. Correctness Analysis`.
    CorrectnessAnalysis,
    /// Section 4: `## 4. UML Diagrams`.
    UmlDiagrams,
    /// Section 5: `## 5. Affected Callers / Downstream Impact`.
    AffectedCallers,
    /// Section 6: `## 6. Proposed Correction`.
    ProposedCorrection,
    /// Section 7: `## 7. Relationship to Existing Issues`.
    RelationshipToExistingIssues,
}

impl DossierSection {
    /// Declaration-ordered catalog of all 7 mandatory defect dossier sections.
    pub const ALL: &'static [Self] = &[
        Self::ContextAndReferences,
        Self::RootCauseAnalysis,
        Self::CorrectnessAnalysis,
        Self::UmlDiagrams,
        Self::AffectedCallers,
        Self::ProposedCorrection,
        Self::RelationshipToExistingIssues,
    ];

    /// Projects a `DossierSection` variant onto its exact required Markdown H2 heading literal.
    /// Realises: [REQ-DEFECT-DOSSIER-SECTION-HEADER]
    pub const fn header(self) -> &'static str {
        match self {
            Self::ContextAndReferences => "## 1. Context and References",
            Self::RootCauseAnalysis => "## 2. Root Cause Analysis (5 Whys)",
            Self::CorrectnessAnalysis => "## 3. Correctness Analysis",
            Self::UmlDiagrams => "## 4. UML Diagrams",
            Self::AffectedCallers => "## 5. Affected Callers / Downstream Impact",
            Self::ProposedCorrection => "## 6. Proposed Correction",
            Self::RelationshipToExistingIssues => "## 7. Relationship to Existing Issues",
        }
    }
}

/// Strongly typed enumeration of permitted `SEVERITY:` classifications in an adversarial audit dossier.
/// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DossierSeverity {
    /// Crash, memory corruption, or resource leak reachable from current callers.
    Critical,
    /// Wrong behavior, degradation under load, or correctness risk in edge cases.
    Important,
    /// Forward-looking risk, missing guard for future code, test gap, or dead code.
    Suggestion,
    /// Style or naming finding with no runtime correctness impact.
    Nitpick,
}

impl DossierSeverity {
    /// Parses a trimmed `SEVERITY:` token into a strongly typed `DossierSeverity` variant.
    /// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-PARSE]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "Critical" => Some(Self::Critical),
            "Important" => Some(Self::Important),
            "Suggestion" => Some(Self::Suggestion),
            "Nitpick" => Some(Self::Nitpick),
            _ => None,
        }
    }

    /// Returns the canonical ASCII string literal for this `DossierSeverity` variant.
    /// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Important => "Important",
            Self::Suggestion => "Suggestion",
            Self::Nitpick => "Nitpick",
        }
    }

    /// Determines whether this severity level mandates a fenced Mermaid diagram in Section 4.
    /// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-MERMAID-GATE]
    pub const fn requires_mermaid(self) -> bool {
        matches!(self, Self::Critical | Self::Important)
    }

    /// Maps `DossierSeverity` to its mandatory GitHub issue tracker label (`bug` or `enhancement`).
    /// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-LABEL]
    pub const fn github_label(self) -> &'static str {
        match self {
            Self::Critical | Self::Important => "bug",
            Self::Suggestion | Self::Nitpick => "enhancement",
        }
    }
}

/// Command-line arguments for `verify-baseline check-defect`.
/// Realises: [REQ-DEFECT-DOSSIER-CLI-ARGS]
#[derive(Debug, Clone)]
pub struct DefectDossierArgs {
    pub body_file: Option<PathBuf>,
    pub title: Option<String>,
    pub repo: Option<String>,
    pub label: Option<String>,
    pub provider: String,
    pub dry_run: bool,
    pub publish: bool,
    pub issue: Option<u64>,
    pub target: Option<PathBuf>,
}

/// Validated metadata extracted from a compliant 7-section adversarial defect dossier.
/// Realises: [REQ-DEFECT-DOSSIER-VALIDATION-RECORD]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectSchemaValidation {
    pub severity: String,
    pub file_location: String,
    pub suggested_label: String,
    pub symptom: String,
    pub test_target: String,
}

/// Checks whether a trimmed metadata value is an unfilled template placeholder such as `[path]` or `[...]`.
/// Realises: [REQ-DEFECT-DOSSIER-PLACEHOLDER-GUARD]
pub(crate) fn is_unfilled_placeholder(val: &str) -> bool {
    let trimmed = val.trim();
    trimmed.is_empty() || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

/// Extracts the file basename (e.g. `bridge.cpp` or `SKILL.md`) from a `FILE_LOCATION` specification.
/// Realises: [REQ-DEFECT-DOSSIER-BASENAME-EXTRACTOR]
pub fn extract_file_basename(file_location: &str) -> String {
    let cleaned = file_location.trim().trim_matches('`').trim();
    if cleaned.is_empty() {
        return String::new();
    }
    let without_line = match cleaned.rsplit_once(':') {
        Some((before, after))
            if !before.is_empty()
                && after.chars().next().is_some_and(|c| c.is_ascii_digit()) =>
        {
            before
        }
        _ => cleaned,
    };
    without_line
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(without_line)
        .trim()
        .to_string()
}

/// Validates that a defect issue title conforms to Step D Check 11 (`[AUDIT] [<file.ext>]: <description>`).
/// Realises: [REQ-DEFECT-DOSSIER-TITLE-VALIDATOR]
pub fn validate_defect_title(title: &str, file_location: &str) -> Result<(), String> {
    crate::defect_dossier_repo::validate_defect_title_impl(title, file_location)
}

/// Derives a compliant `[AUDIT] [<file.ext>]: <description>` title from `FILE_LOCATION` and `Symptom`.
/// Realises: [REQ-DEFECT-DOSSIER-TITLE-DERIVATION]
pub fn derive_defect_title(file_location: &str, symptom: &str) -> String {
    let basename = extract_file_basename(file_location);
    let file_part = if basename.is_empty() {
        "dossier.md"
    } else {
        basename.as_str()
    };
    let first_line = symptom.lines().next().unwrap_or("").trim();
    let desc_part = if first_line.is_empty() || is_unfilled_placeholder(first_line) {
        "Adversarial audit finding"
    } else {
        first_line
    };
    format!("[AUDIT] [{}]: {}", file_part, desc_part)
}

/// Enforces the upstream domain cleanliness gate on Section 6 lines, rejecting downstream crate paths
/// when validating in an `UPSTREAM_SPEC_CORE_COMPILER` workspace.
/// Realises: [REQ-DEFECT-DOSSIER-DOMAIN-CLEANLINESS-GATE]
pub fn domain_cleanliness_gate(sec6_lines: &[String], is_upstream: bool, errors: &mut Vec<String>) {
    if !is_upstream {
        return;
    }
    for raw_line in sec6_lines {
        let mut rest = raw_line.as_str();
        while let Some((_, after_prefix)) = rest.split_once("crates/") {
            rest = after_prefix;
            let first_char = after_prefix.chars().next();
            if matches!(first_char, None | Some('{') | Some('<') | Some('[')) {
                continue;
            }
            let crate_name = after_prefix
                .split(['/', ' ', '\t', '`', '"', '\'', ')', ']', '>', ':', ';', ','])
                .next()
                .unwrap_or("");
            let is_crate_ident = crate_name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                && crate_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if is_crate_ident
                && !crate::defect_dossier_grounding::is_valid_upstream_crate_name(crate_name)
            {
                errors.push(format!(
                    "Domain cleanliness gate violation in Section 6: downstream crate path 'crates/{}' is forbidden in UPSTREAM_SPEC_CORE_COMPILER.",
                    crate_name
                ));
            }
        }
    }
}

/// Validates Section 6 proposed correction code blocks and verifies upstream `test-target` crate grounding.
/// Realises: [REQ-DEFECT-DOSSIER-VALIDATE-SEC6-AND-TARGET-GROUNDING]
pub fn validate_section_6_code_and_target_grounding(
    sec6_lines: &[String],
    test_target: &str,
    is_upstream: bool,
    errors: &mut Vec<String>,
) {
    crate::defect_dossier_grounding::validate_section_6_zero_regex(sec6_lines, errors);
    domain_cleanliness_gate(sec6_lines, is_upstream, errors);
    crate::defect_dossier_grounding::validate_upstream_test_target_grounding(
        test_target,
        is_upstream,
        errors,
    );
}

/// Validates a markdown defect dossier with explicit target repository and workspace root scoping.
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-SCOPED]
pub fn validate_defect_dossier_scoped(
    content: &str,
    target_repo: Option<&str>,
    workspace_root: Option<&Path>,
) -> Result<DefectSchemaValidation, Vec<String>> {
    crate::defect_dossier_repo::validate_defect_dossier_scoped_impl(
        content,
        target_repo,
        workspace_root,
    )
}

/// Validates a markdown defect dossier against all 12 Adversarial Code Auditor Step D checks.
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-12-CHECKS]
pub fn validate_defect_dossier(content: &str) -> Result<DefectSchemaValidation, Vec<String>> {
    validate_defect_dossier_scoped(content, None, None)
}

/// Validates a markdown defect dossier with explicit target repository scoping.
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-WITH-REPO]
pub fn validate_defect_dossier_with_repo(
    content: &str,
    target_repo: Option<&str>,
) -> Result<DefectSchemaValidation, Vec<String>> {
    validate_defect_dossier_scoped(content, target_repo, None)
}

/// Parses a `<owner>/<repo>` repository slug from a Git remote URL or explicit slug string.
/// Realises: [REQ-DEFECT-DOSSIER-PARSE-REPO-SLUG]
pub fn parse_github_repo_slug(remote_url: &str) -> Result<String, String> {
    crate::defect_dossier_repo::parse_github_repo_slug_impl(remote_url)
}

/// Resolves the target GitHub repository `<owner>/<repo>` slug from an explicit `--repo` argument
/// or by querying `git remote get-url origin` in the dossier's working directory.
/// Realises: [REQ-DEFECT-DOSSIER-RESOLVE-TARGET-REPO]
pub fn resolve_target_repo(explicit_repo: Option<&str>, work_dir: &Path) -> Result<String, String> {
    crate::defect_dossier_repo::resolve_target_repo_impl(explicit_repo, work_dir)
}

/// Checks whether a candidate file path points to an ephemeral `/tmp/` location outside `.pipeline/defects/`.
/// Realises: [REQ-DEFECT-DOSSIER-NO-TMP-PATH]
pub fn is_ephemeral_tmp_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let is_tmp_root = normalized.starts_with("/tmp/") || normalized.starts_with("/private/tmp/");
    is_tmp_root && !normalized.contains("/.pipeline/defects/")
}

/// Ensures the validated defect dossier is persisted inside `.pipeline/defects/<filename>.md`.
/// Realises: [REQ-DEFECT-DOSSIER-DUAL-PERSISTENCE]
pub fn ensure_persisted_in_pipeline_defects(
    body_file: &Path,
    content: &str,
) -> Result<PathBuf, String> {
    crate::defect_dossier_repo::ensure_persisted_in_pipeline_defects_impl(body_file, content)
}

/// Verifies the JSON output of `gh issue view --json title,body,url` against the local title and body.
/// Realises: [REQ-DEFECT-DOSSIER-CLOSED-LOOP-GATE]
pub fn verify_closed_loop_payload(
    gh_json_output: &str,
    expected_title: &str,
    expected_body: &str,
) -> Result<String, String> {
    crate::defect_dossier_publish::verify_closed_loop_payload(
        gh_json_output,
        expected_title,
        expected_body,
    )
}

/// Constructs a `gh` CLI subprocess `Command` configured with `/etc/ssl/cert.pem` TLS certificate fallback.
/// Realises: [REQ-DEFECT-DOSSIER-BUILD-GH-COMMAND]
pub fn build_gh_command() -> Command {
    crate::defect_dossier_publish::build_gh_command()
}

/// CLI execution handler for `verify-baseline check-defect`.
/// Realises: [REQ-RUN-CHECK-DEFECT-ENTRYPOINT]
pub fn run_check_defect(args: &DefectDossierArgs) -> Result<(), i32> {
    crate::defect_dossier_publish::execute_check_defect(args)
}

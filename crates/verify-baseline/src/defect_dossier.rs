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
//! - When `--publish` or `--issue` is active without `--dry-run`, the `gh` CLI is installed and
//!   authenticated against the target repository.
//!
//! ## 4. Postconditions
//! - Returns `Ok(DefectSchemaValidation)` from `validate_defect_dossier` if and only if all 12
//!   structural, lexical, and Mermaid invariants hold.
//! - Returns `Ok(())` from `run_check_defect` only after validation (and, when publishing/editing,
//!   local `.pipeline/defects/` persistence and remote closed-loop verification) succeeds.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear scan over dossier byte length $N$ for all 12 validation gates,
//!   plus bounded subprocess invocation latency when `--publish` or `--issue` is enabled.
//! - Space Complexity: $O(N)$ auxiliary line slice and diagnostic buffer allocation.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - All 12 validation checks, filesystem persistence operations, and `gh` CLI subprocess gates
//!   fail closed, returning explicit diagnostic strings or non-zero exit code `Err(1)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-DEFECT-DOSSIER], [DO-330-TQL-4-DEFECT-GATE]

use deap_core::markdown::check_mermaid_syntax;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Canonical 7-section headers required in every Adversarial Code Auditor defect dossier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C structural problem report taxonomy controls.
/// Replaces untyped string arrays with a closed, compile-time checked enumeration of the seven
/// mandatory defect dossier section headings.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijective mapping between the 7 enum variants in `DossierSection::ALL` and the 7 exact
///   Markdown H2 headings `## 1.` through `## 7.`.
///
/// ## 3. Preconditions
/// - Variants are constructed via `DossierSection::ALL` or explicit enum discriminants.
///
/// ## 4. Postconditions
/// - Every variant projects deterministically to its canonical Markdown H2 string literal via `header`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ discriminant lookup.
/// - Space Complexity: $O(1)$ zero-allocation static representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Exhaustive `match` arms guarantee compile-time rejection of any unhandled section variant.
///
/// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A. Enforces character-for-character section header identity
    /// for Step D Check 1 without using regular expressions or untyped string slice arrays.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Injective mapping from the 7 `DossierSection` variants to 7 distinct `'static` strings
    ///   prefixed with `## 1.` through `## 7.`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `DossierSection` enum value.
    ///
    /// ## 4. Postconditions
    /// - Returns the exact non-empty `'static` heading string required on its own line in a dossier.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ constant-time branch table lookup.
    /// - Space Complexity: $O(1)$ static read-only data reference.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible `const fn`; cannot panic or allocate heap memory.
    ///
    /// ## 7. Realization & Traceability
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D defect criticality classification rules.
/// Guarantees that every validated finding maps deterministically to its tracker label (`bug` vs
/// `enhancement`) so forward-looking suggestions never deadlock the `debug-protocol` loop.
///
/// ## 2. Invariants & Mathematical Properties
/// - Partition Invariant: `{Critical, Important}` map to `bug` and require Section 4 Mermaid UML;
///   `{Suggestion, Nitpick}` map to `enhancement` and permit `N/A -- ` in Section 4.
///
/// ## 3. Preconditions
/// - Parsed from the trimmed value following `SEVERITY:` in the `## Audit Source` footer.
///
/// ## 4. Postconditions
/// - Provides total projections to canonical severity string, Mermaid requirement flag, and tracker label.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ per classification query.
/// - Space Complexity: $O(1)$ stack-allocated discriminant.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unknown severity tokens evaluate to `None` in `DossierSeverity::parse` without panicking.
///
/// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A. Enforces Step D Check 3 (`SEVERITY: (Critical|Important|Suggestion|Nitpick)`)
    /// using an exhaustive lexical match with zero regular expressions.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Accepts only exact canonical tokens `"Critical"`, `"Important"`, `"Suggestion"`, or `"Nitpick"`.
    /// - Rejects placeholders such as `"[Critical | Important | Suggestion | Nitpick]"`.
    ///
    /// ## 3. Preconditions
    /// - `raw` is a UTF-8 string slice representing the text after `SEVERITY:`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Some(DossierSeverity)` on exact canonical match, or `None` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(L)$ where $L$ is the byte length of `raw.trim()`.
    /// - Space Complexity: $O(1)$ zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Total non-panicking parser; malformed or empty strings return `None`.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A. Ensures round-trip identity between parsed `DossierSeverity`
    /// variants and serialized `DefectSchemaValidation::severity` strings.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - For all `s: DossierSeverity`, `DossierSeverity::parse(s.as_str()) == Some(s)`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `DossierSeverity` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns `"Critical"`, `"Important"`, `"Suggestion"`, or `"Nitpick"`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible `const fn`; never panics.
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A. Mechanically separates Step D Check 7 (`Critical`/`Important`
    /// require a syntactically valid Mermaid diagram) from Check 8 (`Suggestion`/`Nitpick` allow `N/A -- `).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns `true` if and only if `self` is `Self::Critical` or `Self::Important`.
    ///
    /// ## 3. Preconditions
    /// - `self` is an initialized `DossierSeverity` value.
    ///
    /// ## 4. Postconditions
    /// - Returns a deterministic boolean predicate with zero side effects.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Total `const fn` over a 4-variant enum; cannot fail.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-MERMAID-GATE]
    pub const fn requires_mermaid(self) -> bool {
        matches!(self, Self::Critical | Self::Important)
    }

    /// Maps `DossierSeverity` to its mandatory GitHub issue tracker label (`bug` or `enhancement`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and issue #287 loop-termination invariants. Prevents non-bug
    /// `Suggestion` and `Nitpick` findings from entering the `debug-protocol` `--label bug` queue.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Maps `Critical` and `Important` to `"bug"`.
    /// - Maps `Suggestion` and `Nitpick` to `"enhancement"`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `DossierSeverity`.
    ///
    /// ## 4. Postconditions
    /// - Returns `"bug"` or `"enhancement"` as a `'static` ASCII slice.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible `const fn`; never panics or allocates.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-DEFECT-DOSSIER-SEVERITY-LABEL]
    pub const fn github_label(self) -> &'static str {
        match self {
            Self::Critical | Self::Important => "bug",
            Self::Suggestion | Self::Nitpick => "enhancement",
        }
    }
}

/// Command-line arguments for `verify-baseline check-defect`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 CLI qualification requirements. Defines the
/// command-line interface for validating defect dossiers, persisting `.pipeline/defects/` artifacts,
/// and publishing or editing GitHub SSOT issues with closed-loop verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Either `body_file` (`--body-file`) or `target` (`BODY_FILE`) identifies the input markdown path.
/// - When `publish` (`--publish`) or `issue` (`--issue <ID>`) is set, ephemeral `/tmp/` paths are
///   rejected and the validated dossier is persisted in `.pipeline/defects/` prior to GitHub sync.
///
/// ## 3. Preconditions
/// - Populated by `clap::Parser` or constructed directly by integration tests.
///
/// ## 4. Postconditions
/// - Carries all configuration parameters required by `run_check_defect`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ field access.
/// - Space Complexity: $O(P + T)$ where $P$ is path length and $T$ is title string length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing required file paths or invalid flags are rejected fail-closed by `run_check_defect`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-CLI-ARGS]
#[derive(clap::Args, Debug, Clone)]
pub struct DefectDossierArgs {
    /// Path to markdown defect dossier body file
    #[arg(long = "body-file")]
    pub body_file: Option<PathBuf>,

    /// Title of the defect issue (`[AUDIT] [file.ext]: <description>`)
    #[arg(long = "title")]
    pub title: Option<String>,

    /// Repository identifier (e.g. gintatkinson/DEAP01-spec-core)
    #[arg(long = "repo", default_value = "gintatkinson/DEAP01-spec-core")]
    pub repo: String,

    /// Override label to apply
    #[arg(long = "label")]
    pub label: Option<String>,

    /// Issue tracker provider (github or gitlab)
    #[arg(long = "provider", default_value = "github")]
    pub provider: String,

    /// Validate defect dossier schema without submitting to remote tracker
    #[arg(long = "dry-run", alias = "validate-only")]
    pub dry_run: bool,

    /// Publish a new issue to the remote GitHub SSOT tracker and run closed-loop verification
    #[arg(long = "publish")]
    pub publish: bool,

    /// Existing GitHub issue number to overwrite atomically (`gh issue edit`) and verify closed-loop
    #[arg(long = "issue")]
    pub issue: Option<u64>,

    /// Positional body file path
    #[arg(value_name = "BODY_FILE")]
    pub target: Option<PathBuf>,
}

/// Validated metadata extracted from a compliant 7-section adversarial defect dossier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A traceability controls. Encapsulates the verified severity,
/// file location, derived tracker label, symptom summary, and test-target path extracted during
/// `validate_defect_dossier`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `severity` is guaranteed to be one of `"Critical"`, `"Important"`, `"Suggestion"`, or `"Nitpick"`.
/// - `suggested_label` is `"bug"` for `Critical`/`Important` and `"enhancement"` for `Suggestion`/`Nitpick`.
/// - `test_target` matches both `<!-- test-target: ... -->` and `- **Test-Target**:` in Section 1.
///
/// ## 3. Preconditions
/// - Constructed exclusively upon successful completion of all 12 checks in `validate_defect_dossier`.
///
/// ## 4. Postconditions
/// - Exposes immutable, non-empty metadata fields for downstream CLI reporting and tracker publishing.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ struct construction and field access.
/// - Space Complexity: $O(M)$ where $M$ is the sum of extracted metadata string lengths.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Never constructed when any schema or Mermaid check fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-VALIDATION-RECORD]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefectSchemaValidation {
    /// Verified severity level (`Critical`, `Important`, `Suggestion`, or `Nitpick`).
    pub severity: String,
    /// Verified `FILE_LOCATION` value (`<path>:<line>`).
    pub file_location: String,
    /// Mandatory GitHub tracker label (`bug` or `enhancement`) mapped from `severity`.
    pub suggested_label: String,
    /// Verified `- **Symptom**:` description from Section 1.
    pub symptom: String,
    /// Verified test target path matched across HTML comment and `- **Test-Target**:` bullet.
    pub test_target: String,
}

/// Checks whether a trimmed metadata value is an unfilled template placeholder such as `[path]` or `[...]`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Prevents unpopulated Section 2 skeleton placeholders (`[description]`,
/// `[language]`, `[path]:[line-line]`) from passing automated defect dossier verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if `val` is empty or begins with `'['` and ends with `']'`.
///
/// ## 3. Preconditions
/// - `val` is a borrowed UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `true` when `val` represents an unfilled bracketed template token, `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ prefix and suffix byte inspection after trimming.
/// - Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function; never panics on empty or multi-byte UTF-8 inputs.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-PLACEHOLDER-GUARD]
fn is_unfilled_placeholder(val: &str) -> bool {
    let trimmed = val.trim();
    trimmed.is_empty() || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

/// Extracts the file basename (e.g. `bridge.cpp` or `SKILL.md`) from a `FILE_LOCATION` specification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Normalizes `FILE_LOCATION` strings of the form
/// `path/to/file.ext:56-61` into their terminal filename component `file.ext` for Step D Check 11
/// title verification (`[AUDIT] [<file.ext>]: <description>`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips surrounding backticks and whitespace.
/// - Strips trailing `:<line>` or `:<start>-<end>` suffixes when the segment after `:` begins with an ASCII digit.
/// - Extracts the substring after the final `'/'` or `'\\'` path separator.
///
/// ## 3. Preconditions
/// - `file_location` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns the extracted filename string (or empty `String` if `file_location` is blank).
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ where $L$ is the byte length of `file_location`.
/// - Space Complexity: $O(L)$ for the owned basename string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking total function using `rsplit_once` and `rsplit`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-BASENAME-EXTRACTOR]
pub fn extract_file_basename(file_location: &str) -> String {
    let cleaned = file_location.trim().trim_matches('`').trim();
    if cleaned.is_empty() {
        return String::new();
    }

    let without_line = match cleaned.rsplit_once(':') {
        Some((before, after))
            if !before.is_empty()
                && after
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false) =>
        {
            before
        }
        _ => cleaned,
    };

    let after_slash = without_line
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(without_line)
        .trim();
    after_slash.to_string()
}

/// Validates that a defect issue title conforms to Step D Check 11 (`[AUDIT] [<file.ext>]: <description>`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C tracker configuration management rules.
/// Enforces that every published or edited adversarial audit issue carries a standardized
/// `[AUDIT] [<file.ext>]: <description>` title whose `<file.ext>` matches the basename of `FILE_LOCATION`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Title must contain zero Unicode em dashes (`\u2014`).
/// - Title must begin with `"[AUDIT] ["` and contain `"]: "` separating `<file.ext>` and `<description>`.
/// - `<file.ext>` must equal `extract_file_basename(file_location)` whenever `file_location` is non-empty.
/// - `<description>` must be non-empty and not an unfilled `[...]` placeholder.
///
/// ## 3. Preconditions
/// - `title` and `file_location` are valid UTF-8 string slices.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if `title` satisfies all Check 11 formatting and basename parity rules.
/// - Returns `Err(String)` describing the exact title violation otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(T + F)$ where $T$ is `title.len()` and $F$ is `file_location.len()`.
/// - Space Complexity: $O(F)$ for basename normalization.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `strip_prefix` and `split_once` with zero panicking slice operations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-TITLE-VALIDATOR]
pub fn validate_defect_title(title: &str, file_location: &str) -> Result<(), String> {
    let trimmed = title.trim();
    if trimmed.contains('\u{2014}') {
        return Err(
            "Defect title contains forbidden Unicode em dash (\\u2014); use ASCII '--' instead."
                .to_string(),
        );
    }

    let Some(after_prefix) = trimmed.strip_prefix("[AUDIT] [") else {
        return Err(format!(
            "Invalid title '{}': must match '[AUDIT] [<file.ext>]: <description>' (Check 11).",
            trimmed
        ));
    };

    let Some((file_part, desc_part)) = after_prefix.split_once("]: ") else {
        return Err(format!(
            "Invalid title '{}': missing ']: ' separator in '[AUDIT] [<file.ext>]: <description>' (Check 11).",
            trimmed
        ));
    };

    let file_token = file_part.trim();
    let desc_token = desc_part.trim();
    if file_token.is_empty() || is_unfilled_placeholder(file_token) {
        return Err(format!(
            "Invalid title '{}': '<file.ext>' bracket token must be non-empty (Check 11).",
            trimmed
        ));
    }

    if desc_token.is_empty() || is_unfilled_placeholder(desc_token) {
        return Err(format!(
            "Invalid title '{}': '<description>' after ']: ' must be non-empty and not a placeholder (Check 11).",
            trimmed
        ));
    }

    let expected_basename = extract_file_basename(file_location);
    if !expected_basename.is_empty() && file_token != expected_basename {
        return Err(format!(
            "Title filename mismatch (Check 11): title specifies '[{}]' but FILE_LOCATION basename is '{}'.",
            file_token, expected_basename
        ));
    }

    Ok(())
}

/// Derives a compliant `[AUDIT] [<file.ext>]: <description>` title from `FILE_LOCATION` and `Symptom`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Guarantees that invocations of `verify-baseline check-defect`
/// omitting `--title` automatically synthesize a Check 11-compliant issue title grounded in
/// the dossier's `FILE_LOCATION` basename and Section 1 `Symptom`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Output always starts with `"[AUDIT] ["`, embeds `extract_file_basename(file_location)`,
///   and appends the first line of `symptom` after `"]: "`.
/// - Output satisfies `validate_defect_title(&output, file_location).is_ok()` for any non-placeholder inputs.
///
/// ## 3. Preconditions
/// - `file_location` and `symptom` are UTF-8 string slices extracted from a validated dossier.
///
/// ## 4. Postconditions
/// - Returns an owned `String` formatted as `[AUDIT] [<file.ext>]: <description>`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(F + S)$ where $F$ is `file_location.len()` and $S$ is `symptom.len()`.
/// - Space Complexity: $O(F + S)$ for the synthesized title string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back to `"dossier.md"` and `"Adversarial audit finding"` if inputs are empty; never panics.
///
/// ## 7. Realization & Traceability
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

/// Validates Section 1 (`## 1. Context and References`) for Check 5 and Check 12 compliance.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A traceability controls. Enforces that Section 1 contains the
/// machine-readable `<!-- test-target: <path> -->` HTML annotation followed by all four mandatory
/// bold-labeled bullets (`File`, `Pillar`, `Symptom`, `Test-Target`), and verifies that both
/// test-target paths are identical for automated execution by `reconcile-backlog`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `<!-- test-target: <path> -->` must contain a non-empty, whitespace-free path and precede
///   the Section 1 bullet list.
/// - All four bullets (`- **File**:`, `- **Pillar**:`, `- **Symptom**:`, `- **Test-Target**:`)
///   must be present on separate lines with non-empty, non-placeholder values.
/// - Normalized `comment_test_target` must equal normalized `bullet_test_target`.
///
/// ## 3. Preconditions
/// - `sec1_lines` contains the slice of lines between `## 1. Context and References` and `## 2.`.
///
/// ## 4. Postconditions
/// - Appends any Section 1 violations to `errors` and returns `(symptom, test_target)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L_1)$ where $L_1$ is the number of lines in Section 1.
/// - Space Complexity: $O(M)$ where $M$ is the byte length of extracted Section 1 metadata values.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking line scanner; records all missing or mismatched fields into `errors`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-SECTION1-VALIDATOR]
fn validate_section_1_context(
    sec1_lines: &[String],
    errors: &mut Vec<String>,
) -> (String, String) {
    let mut comment_target: Option<(usize, String)> = None;
    let mut first_bullet_idx: Option<usize> = None;
    let mut file_bullet: Option<String> = None;
    let mut pillar_bullet: Option<String> = None;
    let mut symptom_bullet: Option<String> = None;
    let mut test_target_bullet: Option<String> = None;

    for (idx, raw_line) in sec1_lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            let inner = trimmed
                .strip_prefix("<!--")
                .and_then(|s| s.strip_suffix("-->"))
                .unwrap_or("")
                .trim();
            if let Some(after_tag) = inner.strip_prefix("test-target:") {
                let target_val = after_tag.trim().trim_matches('`').trim();
                if !target_val.is_empty()
                    && !is_unfilled_placeholder(target_val)
                    && !target_val.chars().any(char::is_whitespace)
                {
                    comment_target = Some((idx, target_val.to_string()));
                }
            }
            continue;
        }

        let bullet_rest = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .map(str::trim_start);

        if let Some(rest) = bullet_rest {
            if first_bullet_idx.is_none() {
                first_bullet_idx = Some(idx);
            }
            if let Some(val) = rest.strip_prefix("**File**:") {
                let clean = val.trim().trim_matches('`').trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) {
                    file_bullet = Some(clean.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("**Pillar**:") {
                let clean = val.trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) {
                    pillar_bullet = Some(clean.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("**Symptom**:") {
                let clean = val.trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) {
                    symptom_bullet = Some(clean.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("**Test-Target**:") {
                let clean = val.trim().trim_matches('`').trim();
                if !clean.is_empty()
                    && !is_unfilled_placeholder(clean)
                    && !clean.chars().any(char::is_whitespace)
                {
                    test_target_bullet = Some(clean.to_string());
                }
            }
        }
    }

    if file_bullet.is_none() {
        errors.push("Section 1 missing mandatory '- **File**:' bullet (Check 5).".to_string());
    }
    if pillar_bullet.is_none() {
        errors.push("Section 1 missing mandatory '- **Pillar**:' bullet (Check 5).".to_string());
    }
    if symptom_bullet.is_none() {
        errors.push("Section 1 missing mandatory '- **Symptom**:' bullet (Check 5).".to_string());
    }
    if test_target_bullet.is_none() {
        errors.push(
            "Section 1 missing mandatory '- **Test-Target**:' bullet with valid path (Check 5 / Check 12)."
                .to_string(),
        );
    }

    match (&comment_target, &test_target_bullet) {
        (None, _) => {
            errors.push(
                "Section 1 missing mandatory '<!-- test-target: <path> -->' annotation (Check 12)."
                    .to_string(),
            );
        }
        (Some((c_idx, c_path)), Some(b_path)) => {
            if let Some(b_idx) = first_bullet_idx {
                if *c_idx > b_idx {
                    errors.push(
                        "Section 1 '<!-- test-target: <path> -->' must precede the four bullet lines (Check 12)."
                            .to_string(),
                    );
                }
            }
            if c_path != b_path {
                errors.push(format!(
                    "Section 1 test-target mismatch (Check 12): HTML comment has '{}' but '- **Test-Target**:' bullet has '{}'.",
                    c_path, b_path
                ));
            }
        }
        (Some(_), None) => {}
    }

    (
        symptom_bullet.unwrap_or_default(),
        test_target_bullet.unwrap_or_default(),
    )
}

/// Validates Section 2 (`## 2. Root Cause Analysis (5 Whys)`) for Step D Check 6 compliance.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A root-cause analysis governance. Verifies that Section 2 contains
/// all five numbered causal chain items `1.` through `5.` formatted as
/// `<k>. **Why <question>?** Because <reason>.` without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every integer $k \in \{1, 2, 3, 4, 5\}$ must match at least one line in Section 2 starting
///   with `"{k}. **Why "` and containing `"?** Because "` with non-empty question and reason clauses.
///
/// ## 3. Preconditions
/// - `sec2_lines` is the slice of lines between `## 2. Root Cause Analysis (5 Whys)` and `## 3.`.
///
/// ## 4. Postconditions
/// - Pushes a diagnostic into `errors` for any missing or malformed Why item $k \in 1..=5$.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L_2)$ where $L_2$ is the number of lines in Section 2.
/// - Space Complexity: $O(1)$ auxiliary state.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total lexical scan using `strip_prefix` and `split_once`; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-SECTION2-WHYS-VALIDATOR]
fn validate_section_2_whys(sec2_lines: &[String], errors: &mut Vec<String>) {
    for num in 1..=5 {
        let prefix = format!("{}. **Why ", num);
        let mut matched_valid = false;
        for raw_line in sec2_lines {
            let trimmed = raw_line.trim();
            if let Some(after_why) = trimmed.strip_prefix(&prefix) {
                if let Some((question, because_clause)) = after_why.split_once("?** Because ") {
                    let q_clean = question.trim();
                    let b_clean = because_clause.trim();
                    if !q_clean.is_empty()
                        && !is_unfilled_placeholder(q_clean)
                        && !b_clean.is_empty()
                        && !is_unfilled_placeholder(b_clean)
                    {
                        matched_valid = true;
                        break;
                    }
                }
            }
        }
        if !matched_valid {
            errors.push(format!(
                "Section 2 missing or malformed Why item {}: must match '{}. **Why <question>?** Because <reason>.' (Check 6).",
                num, num
            ));
        }
    }
}

/// Validates Section 4 (`## 4. UML Diagrams`) for Step D Check 7 and Check 8 compliance.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A architectural modeling rules and `rules/platform-independence.md`.
/// Enforces that `Critical` and `Important` findings include a fenced ```` ```mermaid ```` block that
/// passes `deap_core::markdown::check_mermaid_syntax` (plus class-diagram member brace checks),
/// and that `Suggestion` and `Nitpick` findings include either `N/A -- ` or a valid Mermaid diagram.
///
/// ## 2. Invariants & Mathematical Properties
/// - When `severity` is `Some(Critical | Important)`: Section 4 must contain ` ```mermaid ` and
///   `check_mermaid_syntax` must return zero errors.
/// - When `severity` is `Some(Suggestion | Nitpick)`: Section 4 must contain `"N/A -- "` or ` ```mermaid `.
/// - Any `classDiagram` block in Section 4 must not contain `{` or `}` inside class member lines.
///
/// ## 3. Preconditions
/// - `sec4_lines` is the slice of lines in Section 4; `full_content` is the complete dossier string.
///
/// ## 4. Postconditions
/// - Appends all detected Section 4 and Mermaid syntax violations to `errors`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over dossier byte length $N$.
/// - Space Complexity: $O(E)$ where $E$ is the number of detected Mermaid diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Offline deterministic grammar check; never contacts remote servers or panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-SECTION4-MERMAID-VALIDATOR]
fn validate_section_4_uml(
    sec4_lines: &[String],
    full_content: &str,
    severity: Option<DossierSeverity>,
    errors: &mut Vec<String>,
) {
    let has_mermaid_fence = sec4_lines.iter().any(|l| {
        let t = l.trim();
        t == "```mermaid" || t.starts_with("```mermaid ")
    });
    let has_na_marker = sec4_lines.iter().any(|l| l.contains("N/A -- "));

    match severity {
        Some(sev) if sev.requires_mermaid() => {
            if !has_mermaid_fence {
                errors.push(format!(
                    "Section 4 missing mandatory ```mermaid diagram block for {} severity finding (Check 7).",
                    sev.as_str()
                ));
            }
        }
        Some(sev) => {
            if !has_na_marker && !has_mermaid_fence {
                errors.push(format!(
                    "Section 4 for {} severity must contain 'N/A -- ' or a valid ```mermaid diagram (Check 8).",
                    sev.as_str()
                ));
            }
        }
        None => {}
    }

    if has_mermaid_fence || full_content.contains("```mermaid") {
        let mermaid_errs = check_mermaid_syntax(full_content, "defect_dossier.md");
        for err in mermaid_errs {
            errors.push(format!("Mermaid syntax violation (Check 7): {}", err));
        }

        // Enforce classDiagram curly-brace member rule from SKILL.md Section 1.5
        let mut in_mermaid = false;
        let mut is_class_diagram = false;
        for raw_line in sec4_lines {
            let trimmed = raw_line.trim();
            if !in_mermaid {
                if trimmed == "```mermaid" || trimmed.starts_with("```mermaid ") {
                    in_mermaid = true;
                    is_class_diagram = false;
                }
            } else if trimmed.starts_with("```") {
                in_mermaid = false;
                is_class_diagram = false;
            } else if !trimmed.is_empty() && !trimmed.starts_with("%%") {
                if trimmed.eq_ignore_ascii_case("classDiagram")
                    || trimmed.to_ascii_lowercase().starts_with("classdiagram ")
                {
                    is_class_diagram = true;
                } else if is_class_diagram
                    && (trimmed.contains('{') || trimmed.contains('}'))
                    && !trimmed.starts_with("class ")
                    && trimmed != "{"
                    && trimmed != "}"
                {
                    errors.push(format!(
                        "Mermaid syntax violation (Check 7): curly braces '{{}}' forbidden in class member line: '{}'.",
                        trimmed
                    ));
                }
            }
        }
    }
}

/// Validates Section 6 (`## 6. Proposed Correction`), global fence parity, ASCII art prohibition,
/// and Unicode em dash prohibition (Checks 9 and 10).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Ensures every defect dossier provides a concrete, language-tagged
/// proposed code correction in Section 6, maintains balanced triple-backtick fences across the entire
/// document, and contains zero unescaped ASCII art arrows (`->>` or `→`) outside Mermaid blocks
/// and zero Unicode em dashes (`\u2014`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Total count of lines whose trimmed prefix is `"```"` across `full_content` must be even.
/// - Section 6 must contain at least one opening fence `"```<lang>"` where `<lang>` is non-empty
///   and not an unfilled `[language]` placeholder.
/// - Every line outside ` ```mermaid ... ``` ` blocks must contain zero `"->>"` and zero `'\u{2192}'` tokens.
/// - `full_content` must contain zero `'\u{2014}'` characters.
///
/// ## 3. Preconditions
/// - `sec6_lines` is the slice of lines in Section 6; `full_content` is the complete dossier string.
///
/// ## 4. Postconditions
/// - Appends any detected fence, language tag, ASCII art, or em dash violations to `errors`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ single-pass line scan over `full_content` and `sec6_lines`.
/// - Space Complexity: $O(1)$ scalar state tracking.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total non-panicking lexical scan.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-FENCES-AND-ASCII-ART-VALIDATOR]
fn validate_fences_ascii_art_and_em_dash(
    sec6_lines: &[String],
    full_content: &str,
    errors: &mut Vec<String>,
) {
    // Check Section 6 language-tagged code fence
    let mut in_sec6_fence = false;
    let mut found_tagged_sec6_fence = false;
    for raw_line in sec6_lines {
        let trimmed = raw_line.trim();
        if let Some(after_ticks) = trimmed.strip_prefix("```") {
            if !in_sec6_fence {
                in_sec6_fence = true;
                let lang_tag = after_ticks.trim();
                if !lang_tag.is_empty() && !is_unfilled_placeholder(lang_tag) {
                    found_tagged_sec6_fence = true;
                }
            } else {
                in_sec6_fence = false;
            }
        }
    }
    if !found_tagged_sec6_fence {
        errors.push(
            "Section 6 ('## 6. Proposed Correction') must contain a triple-backtick code block with a non-empty language tag (e.g. ```rust)."
                .to_string(),
        );
    }

    // Check balanced code fences, no unescaped ->> or → outside mermaid blocks, and no leading H1 outside fences
    let mut fence_count = 0usize;
    let mut in_any_fence = false;
    let mut in_mermaid_fence = false;

    for (idx, raw_line) in full_content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim_start();
        if let Some(after_ticks) = trimmed.strip_prefix("```") {
            fence_count += 1;
            if !in_any_fence {
                in_any_fence = true;
                let tag = after_ticks.trim();
                in_mermaid_fence = tag.eq_ignore_ascii_case("mermaid");
            } else {
                in_any_fence = false;
                in_mermaid_fence = false;
            }
            continue;
        }

        if !in_mermaid_fence && (raw_line.contains("->>") || raw_line.contains('\u{2192}')) {
            errors.push(format!(
                "Line {}: forbidden ASCII art arrow ('->>' or '\\u2192') outside ```mermaid block (Check 10).",
                line_no
            ));
        }

        if !in_any_fence && trimmed.starts_with("# ") {
            errors.push(format!(
                "Line {}: forbidden leading '# ' H1 header in dossier body; start directly with '## 1. Context and References'.",
                line_no
            ));
        }
    }

    if !fence_count.is_multiple_of(2) {
        errors.push(format!(
            "Unbalanced code blocks (Check 9): found odd number ({}) of ``` fences.",
            fence_count
        ));
    }

    if full_content.contains('\u{2014}') {
        errors.push(
            "Dossier contains forbidden Unicode em dash (\\u2014); use ASCII '--' instead."
                .to_string(),
        );
    }
}

/// Validates a markdown defect dossier against all 12 Adversarial Code Auditor Step D checks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification tool qualification
/// standards. Mechanically enforces the complete 12-point Adversarial Code Auditor schema,
/// Mermaid diagram syntax, test-target parity, 5-Whys causal chain, and zero-H1/zero-em-dash
/// invariants in pure Rust with zero regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// 1. Rejects empty payloads or leading `# ` H1 headers (first non-empty line must be `## 1. Context and References`).
/// 2. Enforces exact presence and ascending order of all 7 `DossierSection` headers plus `## Audit Source`.
/// 3. Enforces valid `SEVERITY:` (`Critical`, `Important`, `Suggestion`, `Nitpick`) and `FILE_LOCATION: <path>:<line>`.
/// 4. Enforces Section 1 `<!-- test-target: <path> -->` and all 4 bullets (`File`, `Pillar`, `Symptom`, `Test-Target`) with matching test-target paths.
/// 5. Enforces Section 2 numbered lines `1.` through `5.` matching `<k>. **Why ...?** Because ...`.
/// 6. Enforces Section 4 Mermaid syntax via `deap_core::markdown::check_mermaid_syntax` (for `Critical`/`Important`) or `N/A -- ` (for `Suggestion`/`Nitpick`).
/// 7. Enforces Section 6 language-tagged code fence, balanced ``` fences, zero `->>`/`→` outside Mermaid blocks, and zero `\u2014`.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice representing the candidate defect dossier body.
///
/// ## 4. Postconditions
/// - Returns `Ok(DefectSchemaValidation)` when zero violations exist across all 12 gates.
/// - Returns `Err(Vec<String>)` listing every detected violation when one or more gates fail.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in the byte length $N$ of `content`.
/// - Space Complexity: $O(L + E)$ where $L$ is line count and $E$ is diagnostic error count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking total function using checked slice indexing (`.get(..)`) exclusively.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-12-CHECKS]
pub fn validate_defect_dossier(content: &str) -> Result<DefectSchemaValidation, Vec<String>> {
    let mut errors = Vec::new();
    let lines: Vec<String> = content.lines().map(str::to_string).collect();

    // 0. First non-empty line must be "## 1. Context and References" (no leading # H1 header)
    match lines.iter().map(|l| l.trim()).find(|l| !l.is_empty()) {
        None => {
            errors.push("Defect dossier body is empty.".to_string());
            return Err(errors);
        }
        Some(first_line) => {
            if first_line.starts_with("# ") {
                errors.push(format!(
                    "Forbidden leading H1 header '{}' on first line: dossier must begin directly with '## 1. Context and References'.",
                    first_line
                ));
            } else if first_line != DossierSection::ContextAndReferences.header() {
                errors.push(format!(
                    "First non-empty line must be '{}' (found '{}').",
                    DossierSection::ContextAndReferences.header(),
                    first_line
                ));
            }
        }
    }

    // 1 & 2. Locate exact 7 section headers and "## Audit Source" outside fenced code blocks
    let mut section_indices: Vec<(DossierSection, usize)> = Vec::new();
    let mut audit_source_idx: Option<usize> = None;
    let mut in_fence = false;

    for (idx, raw_line) in lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        for &sec in DossierSection::ALL {
            if trimmed == sec.header() && !section_indices.iter().any(|(s, _)| *s == sec) {
                section_indices.push((sec, idx));
            }
        }
        if trimmed == "## Audit Source" && audit_source_idx.is_none() {
            audit_source_idx = Some(idx);
        }
    }

    let mut prev_line_idx = 0usize;
    for &sec in DossierSection::ALL {
        match section_indices.iter().find(|(s, _)| *s == sec) {
            Some((_, idx)) => {
                if *idx < prev_line_idx {
                    errors.push(format!(
                        "Section header '{}' is out of order (Check 1).",
                        sec.header()
                    ));
                }
                prev_line_idx = *idx;
            }
            None => {
                errors.push(format!(
                    "Missing mandatory exact section header '{}' (Check 1).",
                    sec.header()
                ));
            }
        }
    }

    match audit_source_idx {
        Some(idx) => {
            if idx < prev_line_idx {
                errors.push(
                    "Section header '## Audit Source' must appear after Section 7 (Check 2)."
                        .to_string(),
                );
            }
        }
        None => {
            errors.push("Missing mandatory '## Audit Source' section header (Check 2).".to_string());
        }
    }

    // Helper closure to slice lines for a given DossierSection
    let get_section_slice = |target_sec: DossierSection| -> &[String] {
        let Some((_, start_idx)) = section_indices.iter().find(|(s, _)| *s == target_sec) else {
            return &[];
        };
        let mut end_idx = lines.len();
        for &(_, other_idx) in &section_indices {
            if other_idx > *start_idx && other_idx < end_idx {
                end_idx = other_idx;
            }
        }
        if let Some(a_idx) = audit_source_idx {
            if a_idx > *start_idx && a_idx < end_idx {
                end_idx = a_idx;
            }
        }
        lines.get(start_idx.saturating_add(1)..end_idx).unwrap_or(&[])
    };

    // 3. Check SEVERITY line
    let mut parsed_severity: Option<DossierSeverity> = None;
    for raw_line in &lines {
        let trimmed = raw_line.trim();
        if let Some(rest) = trimmed.strip_prefix("SEVERITY:") {
            if let Some(sev) = DossierSeverity::parse(rest) {
                parsed_severity = Some(sev);
                break;
            }
        }
    }
    let severity_str = match parsed_severity {
        Some(sev) => sev.as_str().to_string(),
        None => {
            errors.push(
                "Missing or invalid 'SEVERITY:' line (must be Critical, Important, Suggestion, or Nitpick) (Check 3)."
                    .to_string(),
            );
            String::new()
        }
    };

    // 4. Check FILE_LOCATION line (must match FILE_LOCATION: <path>:<line>)
    let mut file_location_opt: Option<String> = None;
    for raw_line in &lines {
        let trimmed = raw_line.trim();
        if let Some(rest) = trimmed.strip_prefix("FILE_LOCATION:") {
            let val = rest.trim().trim_matches('`').trim();
            if !val.is_empty() && !is_unfilled_placeholder(val) {
                if let Some((path_part, line_part)) = val.rsplit_once(':') {
                    let has_valid_line = line_part
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false);
                    if !path_part.trim().is_empty()
                        && !is_unfilled_placeholder(path_part)
                        && has_valid_line
                    {
                        file_location_opt = Some(val.to_string());
                        break;
                    }
                }
            }
        }
    }
    let file_location = match file_location_opt {
        Some(loc) => loc,
        None => {
            errors.push(
                "Missing or invalid mandatory 'FILE_LOCATION: <path>:<line>' line (Check 4)."
                    .to_string(),
            );
            String::new()
        }
    };

    // 5 & 12. Validate Section 1 Context, 4 mandatory bullets, and test-target parity
    let sec1_slice = get_section_slice(DossierSection::ContextAndReferences);
    let (symptom, test_target) = validate_section_1_context(sec1_slice, &mut errors);

    // 6. Validate Section 2 5-Whys
    let sec2_slice = get_section_slice(DossierSection::RootCauseAnalysis);
    validate_section_2_whys(sec2_slice, &mut errors);

    // 7 & 8. Validate Section 4 UML Diagrams
    let sec4_slice = get_section_slice(DossierSection::UmlDiagrams);
    validate_section_4_uml(sec4_slice, content, parsed_severity, &mut errors);

    // 9 & 10. Validate Section 6 code fence tag, balanced fences, no ASCII art UML, and no em dashes
    let sec6_slice = get_section_slice(DossierSection::ProposedCorrection);
    validate_fences_ascii_art_and_em_dash(sec6_slice, content, &mut errors);

    if !errors.is_empty() {
        Err(errors)
    } else {
        let suggested_label = parsed_severity
            .map(|s| s.github_label().to_string())
            .unwrap_or_else(|| "enhancement".to_string());

        Ok(DefectSchemaValidation {
            severity: severity_str,
            file_location,
            suggested_label,
            symptom,
            test_target,
        })
    }
}

/// Checks whether a candidate file path points to an ephemeral `/tmp/` location outside `.pipeline/defects/`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A configuration management controls. Prevents agents from filing
/// GitHub SSOT issues from ephemeral `/tmp/gh_body_*.md` files that bypass local `.pipeline/defects/`
/// persistence.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if the slash-normalized path starts with `"/tmp/"` or `"/private/tmp/"` and
///   does not reside within a `"/.pipeline/defects/"` directory hierarchy.
///
/// ## 3. Preconditions
/// - `path` is a valid filesystem `Path` reference.
///
/// ## 4. Postconditions
/// - Returns a deterministic boolean verdict without performing filesystem I/O.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P)$ where $P$ is the byte length of `path`.
/// - Space Complexity: $O(P)$ for slash normalization.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function; never panics on non-UTF-8 OS strings (via `to_string_lossy`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-NO-TMP-PATH]
pub fn is_ephemeral_tmp_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let is_tmp_root = normalized.starts_with("/tmp/") || normalized.starts_with("/private/tmp/");
    let in_pipeline_defects = normalized.contains("/.pipeline/defects/");
    is_tmp_root && !in_pipeline_defects
}

/// Ensures the validated defect dossier is persisted inside `.pipeline/defects/<filename>.md`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A dual-location SSOT archival rules. Guarantees that before any
/// GitHub issue is created or edited, the validated dossier body exists inside `.pipeline/defects/`
/// so local repository state and remote GitHub SSOT are backed by the identical `.pipeline/defects/` artifact.
///
/// ## 2. Invariants & Mathematical Properties
/// - If `body_file` already resides inside a `.pipeline/defects` directory, returns `body_file` unchanged.
/// - Otherwise creates `.pipeline/defects/` if absent and writes `content` atomically to
///   `.pipeline/defects/<filename>`.
///
/// ## 3. Preconditions
/// - `body_file` is the validated source path; `content` is the validated UTF-8 dossier payload.
///
/// ## 4. Postconditions
/// - Returns `Ok(PathBuf)` pointing to the `.pipeline/defects/<filename>` file containing `content`.
/// - Returns `Err(String)` if directory creation or file writing fails.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ filesystem write proportional to `content.len()`.
/// - Space Complexity: $O(P)$ for target `PathBuf` construction.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates any OS I/O error fail-closed as `Err(String)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-DUAL-PERSISTENCE]
pub fn ensure_persisted_in_pipeline_defects(
    body_file: &Path,
    content: &str,
) -> Result<PathBuf, String> {
    let normalized = body_file.to_string_lossy().replace('\\', "/");
    if normalized.starts_with(".pipeline/defects/") || normalized.contains("/.pipeline/defects/") {
        return Ok(body_file.to_path_buf());
    }

    let file_name = body_file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "defect_dossier.md".to_string());
    let defects_dir = PathBuf::from(".pipeline").join("defects");
    fs::create_dir_all(&defects_dir).map_err(|e| {
        format!(
            "Failed to create '{}': {}",
            defects_dir.display(),
            e
        )
    })?;
    let dest_path = defects_dir.join(file_name);
    fs::write(&dest_path, content).map_err(|e| {
        format!(
            "Failed to persist defect dossier to '{}': {}",
            dest_path.display(),
            e
        )
    })?;
    Ok(dest_path)
}

/// Normalizes CRLF (`\r\n`) line endings to LF (`\n`) and strips trailing newline characters for
/// closed-loop payload comparison.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A cross-platform transport verification rules. Accounts for GitHub's
/// REST/GraphQL API `\r\n` line-ending normalization while preserving strict byte-for-byte content
/// identity across all non-trailing characters.
///
/// ## 2. Invariants & Mathematical Properties
/// - Idempotent projection: `normalize_line_endings(&normalize_line_endings(s)) == normalize_line_endings(s)`.
/// - Preserves all internal lines, whitespace, code blocks, and punctuation verbatim.
///
/// ## 3. Preconditions
/// - `text` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns a `String` with `\r\n` replaced by `\n` and trailing `\r`/`\n` characters removed.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is `text.len()`.
/// - Space Complexity: $O(N)$ for the normalized string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-CRLF-NORMALIZATION]
fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
        .trim_end_matches(['\r', '\n'])
        .to_string()
}

/// Verifies the JSON output of `gh issue view --json title,body,url` against the local title and body.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 closed-loop tool verification requirements.
/// Prevents truncated, shell-corrupted, or mismatched issue payloads from being silently accepted
/// after `gh issue create` or `gh issue edit`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Remote JSON `title` field must equal `expected_title` character-for-character.
/// - Remote JSON `body` field (after `normalize_line_endings`) must equal `normalize_line_endings(expected_body)`
///   byte-for-byte.
///
/// ## 3. Preconditions
/// - `gh_json_output` is the raw stdout string emitted by `gh issue view <ID_OR_URL> --repo <REPO> --json title,body,url`.
/// - `expected_title` and `expected_body` are the local ground-truth strings.
///
/// ## 4. Postconditions
/// - Returns `Ok(String)` containing the verified remote issue URL if both `title` and `body` match.
/// - Returns `Err(String)` describing the parse error or payload mismatch otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is the byte length of `gh_json_output`.
/// - Space Complexity: $O(N)$ for JSON deserialization and CRLF-normalized body comparison.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails closed on malformed JSON, missing `title`/`body` keys, or any title/body divergence.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-CLOSED-LOOP-GATE]
pub fn verify_closed_loop_payload(
    gh_json_output: &str,
    expected_title: &str,
    expected_body: &str,
) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(gh_json_output).map_err(|e| {
        format!(
            "Closed-Loop Payload Verification Gate failed to parse 'gh issue view' JSON: {}",
            e
        )
    })?;

    let remote_title = parsed
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            "Closed-Loop Payload Verification Gate failed: missing 'title' field in 'gh issue view' JSON."
                .to_string()
        })?;

    let remote_body = parsed
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            "Closed-Loop Payload Verification Gate failed: missing 'body' field in 'gh issue view' JSON."
                .to_string()
        })?;

    let remote_url = parsed
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if remote_title != expected_title {
        return Err(format!(
            "Closed-Loop Payload Verification Gate FAILED: remote title '{}' does not match expected title '{}'.",
            remote_title, expected_title
        ));
    }

    let norm_remote = normalize_line_endings(remote_body);
    let norm_local = normalize_line_endings(expected_body);
    if norm_remote != norm_local {
        return Err(format!(
            "Closed-Loop Payload Verification Gate FAILED: remote issue body ({} bytes) does not match local .pipeline/defects/ file ({} bytes).",
            norm_remote.len(),
            norm_local.len()
        ));
    }

    Ok(remote_url)
}

/// Publishes a new GitHub issue or edits an existing GitHub issue from the persisted `.pipeline/defects/`
/// file and immediately executes the Closed-Loop Payload Verification Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 automated SSOT synchronization controls.
/// Executes `gh issue edit` (when `issue_id` is `Some`) or `gh issue create` (when `issue_id` is `None`)
/// using `--body-file` pointing to `.pipeline/defects/<filename>.md`, then queries `gh issue view`
/// and verifies byte-for-byte fidelity via `verify_closed_loop_payload`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Never invokes `gh issue comment` for defect bodies; always overwrites or creates the canonical
///   issue title and body atomically via `--title` and `--body-file`.
/// - Returns `Ok(String)` only if both the write command and the subsequent `gh issue view`
///   closed-loop check succeed.
///
/// ## 3. Preconditions
/// - `persisted_path` exists inside `.pipeline/defects/` and contains `local_body`.
/// - `title`, `repo`, and `label` are non-empty validated strings.
///
/// ## 4. Postconditions
/// - Remote GitHub issue is created or updated with `title`, `label`, and `local_body`, and verified
///   closed-loop against `local_body`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ local payload verification plus two sequential `gh` CLI process invocations.
/// - Space Complexity: $O(N)$ for subprocess stdout/stderr buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-zero exit codes from `gh issue create`, `gh issue edit`, or `gh issue view`, or any
///   payload mismatch in `verify_closed_loop_payload`, fail closed with a descriptive `Err(String)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-DOSSIER-GITHUB-SYNC]
pub fn publish_or_edit_github_issue(
    repo: &str,
    title: &str,
    label: &str,
    persisted_path: &Path,
    local_body: &str,
    issue_id: Option<u64>,
) -> Result<String, String> {
    let body_path_str = persisted_path.to_string_lossy().to_string();

    let issue_identifier = if let Some(id) = issue_id {
        let id_str = id.to_string();
        let output = Command::new("gh")
            .args([
                "issue",
                "edit",
                &id_str,
                "--repo",
                repo,
                "--title",
                title,
                "--body-file",
                &body_path_str,
                "--add-label",
                label,
            ])
            .output()
            .map_err(|e| format!("Failed to execute 'gh issue edit': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "'gh issue edit {}' failed: {}",
                id_str,
                stderr.trim()
            ));
        }
        id_str
    } else {
        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                repo,
                "--title",
                title,
                "--label",
                label,
                "--body-file",
                &body_path_str,
            ])
            .output()
            .map_err(|e| format!("Failed to execute 'gh issue create': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue create' failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let created_ref = stdout
            .lines()
            .rev()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .ok_or_else(|| "'gh issue create' succeeded but emitted empty stdout.".to_string())?;
        created_ref.to_string()
    };

    let view_output = Command::new("gh")
        .args([
            "issue",
            "view",
            &issue_identifier,
            "--repo",
            repo,
            "--json",
            "title,body,url",
        ])
        .output()
        .map_err(|e| format!("Failed to execute 'gh issue view': {}", e))?;

    if !view_output.status.success() {
        let stderr = String::from_utf8_lossy(&view_output.stderr);
        return Err(format!(
            "'gh issue view {}' failed: {}",
            issue_identifier,
            stderr.trim()
        ));
    }

    let view_stdout = String::from_utf8_lossy(&view_output.stdout);
    let verified_url = verify_closed_loop_payload(&view_stdout, title, local_body)?;
    if verified_url.is_empty() {
        Ok(issue_identifier)
    } else {
        Ok(verified_url)
    }
}

/// CLI execution handler for `verify-baseline check-defect`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 CLI tool qualification standards. Orchestrates
/// ephemeral `/tmp/` path rejection, 12-check dossier schema and Mermaid validation, Check 11 title
/// validation or derivation, local `.pipeline/defects/` persistence, and optional GitHub SSOT
/// publishing/editing with closed-loop verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects `/tmp/` paths outside `.pipeline/defects/` immediately when `--publish` or `--issue` is set.
/// - Validates or auto-derives the `[AUDIT] [<file.ext>]: <description>` title against `FILE_LOCATION`.
/// - Returns `Ok(())` if and only if all enabled validation, persistence, and closed-loop sync steps succeed.
///
/// ## 3. Preconditions
/// - `args` is a populated `DefectDossierArgs` reference.
///
/// ## 4. Postconditions
/// - Emits structured validation and closed-loop verification telemetry to stdout on `Ok(())`,
///   or diagnostic errors to stderr with `Err(1)` on any violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over dossier file byte length $N$, plus optional `gh` CLI subprocess round-trips.
/// - Space Complexity: $O(N)$ for file content and validation buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing files, unreadable files, `/tmp/` ephemeral paths, schema/Mermaid violations, title
///   mismatches, and remote GitHub discrepancies all fail closed with `Err(1)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RUN-CHECK-DEFECT-ENTRYPOINT]
pub fn run_check_defect(args: &DefectDossierArgs) -> Result<(), i32> {
    let body_path = args.body_file.as_ref().or(args.target.as_ref());

    let body_file = match body_path {
        Some(p) => p,
        None => {
            eprintln!("Error: --body-file (or positional BODY_FILE) is required and must exist.");
            return Err(1);
        }
    };

    if (args.publish || args.issue.is_some()) && is_ephemeral_tmp_path(body_file) {
        eprintln!(
            "Error: Ephemeral /tmp/ path '{}' is forbidden when --publish or --issue is set. Write defect dossiers directly to .pipeline/defects/<name>.md.",
            body_file.display()
        );
        return Err(1);
    }

    if !body_file.is_file() {
        eprintln!(
            "Error: Defect body file does not exist: {}",
            body_file.display()
        );
        return Err(1);
    }

    let content = match fs::read_to_string(body_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading body file {}: {}", body_file.display(), e);
            return Err(1);
        }
    };

    match validate_defect_dossier(&content) {
        Ok(validation) => {
            let label = args.label.clone().unwrap_or_else(|| {
                if args.provider == "gitlab" {
                    if validation.suggested_label == "bug" {
                        "type::bug".to_string()
                    } else {
                        "type::feature".to_string()
                    }
                } else {
                    validation.suggested_label.clone()
                }
            });

            let title = match &args.title {
                Some(explicit_title) => {
                    if let Err(title_err) =
                        validate_defect_title(explicit_title, &validation.file_location)
                    {
                        eprintln!(
                            "Defect dossier validation FAILED for {} (Check 11 title error):",
                            body_file.display()
                        );
                        eprintln!("  - {}", title_err);
                        return Err(1);
                    }
                    explicit_title.trim().to_string()
                }
                None => derive_defect_title(&validation.file_location, &validation.symptom),
            };

            let persisted_path = if args.publish || args.issue.is_some() {
                match ensure_persisted_in_pipeline_defects(body_file, &content) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error persisting dossier to .pipeline/defects/: {}", e);
                        return Err(1);
                    }
                }
            } else {
                body_file.to_path_buf()
            };

            println!(
                "Defect dossier validation PASSED for {}.",
                persisted_path.display()
            );
            println!("[VALIDATION] Target payload:");
            println!("  Provider: {}", args.provider);
            println!("  Repo: {}", args.repo);
            println!("  Title: {}", title);
            println!("  Severity: {}", validation.severity);
            println!("  File Location: {}", validation.file_location);
            println!("  Test Target: {}", validation.test_target);
            println!("  Label: {}", label);
            println!("  Body file: {}", persisted_path.display());

            if (args.publish || args.issue.is_some()) && !args.dry_run {
                match publish_or_edit_github_issue(
                    &args.repo,
                    &title,
                    &label,
                    &persisted_path,
                    &content,
                    args.issue,
                ) {
                    Ok(url) => {
                        println!(
                            "[CLOSED-LOOP VERIFIED] Remote GitHub SSOT issue synchronized and verified byte-for-byte: {}",
                            url
                        );
                    }
                    Err(err) => {
                        eprintln!("Error synchronizing defect dossier to GitHub SSOT: {}", err);
                        return Err(1);
                    }
                }
            }

            Ok(())
        }
        Err(errors) => {
            eprintln!(
                "Defect dossier validation FAILED for {} with {} violation(s):",
                body_file.display(),
                errors.len()
            );
            for err in errors {
                eprintln!("  - {}", err);
            }
            Err(1)
        }
    }
}

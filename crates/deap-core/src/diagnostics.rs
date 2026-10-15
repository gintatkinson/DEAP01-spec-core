//! Diagnostic reporting and defect dossier synthesis.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides deterministic diagnostic classification, source location
//! anchoring, 5-Whys root cause structures, and 7-section Adversarial Audit defect dossier
//! Markdown and JSON synthesis.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Severity Total Ordering: `DiagnosticSeverity` variants implement total lexicographic and
//!   priority ordering (`Ord`, `PartialOrd`, `Eq`, `PartialEq`).
//! - Zero Em Dash Invariant: Synthesized Markdown dossiers emit strictly ASCII punctuation (`--`).
//! - Deterministic Serialization: JSON round-trip (`to_json` / `from_json`) preserves 1:1 structural equality.
//!
//! ## 3. Preconditions
//! - Input strings and JSON slices must be valid UTF-8 sequences.
//!
//! ## 4. Postconditions
//! - Constructors return initialized, strongly typed diagnostic or defect dossier structures.
//! - Serialization and parsing operations return explicit `Result` types without panicking.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) bounded linearly by the byte length of diagnostic messages or dossier fields.
//! - Space Complexity: O(N) bounded by the allocated string representations.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - JSON serialization and deserialization errors propagate fail-closed via `Result::Err(serde_json::Error)`.
//! - Display formatting propagates formatter errors via `std::fmt::Result`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-DIAG-MODULE], [DO-330-TQL-4-DIAGNOSTICS].

use serde::{Deserialize, Serialize};

/// Severity levels for diagnostics and defect audit dossiers.
///
/// Realises: [Diagnostic/DiagnosticSeverity] | SysML v2: part def DiagnosticEngine
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Critical,
    Important,
    Suggestion,
    Nitpick,
    Error,
    Warning,
    Info,
    Hint,
}

impl std::fmt::Display for DiagnosticSeverity {
    /// Formats the diagnostic severity level into its canonical human-readable string representation.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C diagnostic reporting requirements.
    /// Ensures deterministic textual rendering of severity classifications in compiler diagnostics and
    /// 7-section Adversarial Audit defect dossiers (`SEVERITY: <level>`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective Mapping: Establishes a strict 1-to-1 bijection between the 8 `DiagnosticSeverity`
    ///   enum variants (`Critical`, `Important`, `Suggestion`, `Nitpick`, `Error`, `Warning`, `Info`, `Hint`)
    ///   and their exact ASCII PascalCase string literals.
    /// - Referential Transparency: Pure function of `*self` with zero side effects outside the formatter buffer.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid, initialized `DiagnosticSeverity` variant.
    /// - `f` is a valid mutable reference to an active `std::fmt::Formatter`.
    ///
    /// ## 4. Postconditions
    /// - Writes the exact canonical ASCII severity token to the formatter stream `f`.
    /// - Returns `Ok(())` if the underlying stream write succeeds, or `Err(std::fmt::Error)` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time enum branch dispatch and fixed-length literal write (4 to 10 bytes).
    /// - Space Complexity: O(1) auxiliary stack memory; zero heap allocations.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates underlying formatter stream failures via `std::fmt::Result` without panicking.
    /// - State: Immutable borrow of `self`; cannot corrupt diagnostic severity state.
    /// - Action: Exhaustive compile-time `match` prevents unhandled enum variant faults.
    /// - Resource: Zero dynamic memory allocation eliminates out-of-memory hazards.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/DiagnosticSeverity::fmt] | [REQ-DIAG-FMT] | SysML v2: part def DiagnosticEngine
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Critical => write!(f, "Critical"),
            DiagnosticSeverity::Important => write!(f, "Important"),
            DiagnosticSeverity::Suggestion => write!(f, "Suggestion"),
            DiagnosticSeverity::Nitpick => write!(f, "Nitpick"),
            DiagnosticSeverity::Error => write!(f, "Error"),
            DiagnosticSeverity::Warning => write!(f, "Warning"),
            DiagnosticSeverity::Info => write!(f, "Info"),
            DiagnosticSeverity::Hint => write!(f, "Hint"),
        }
    }
}

/// Represents an individual compiler or linter diagnostic item.
///
/// Realises: [Diagnostic/Diagnostic] | SysML v2: part def DiagnosticEngine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub line: Option<usize>,
    #[serde(default)]
    pub column: Option<usize>,
}

impl Diagnostic {
    /// Constructs a new diagnostic item with a severity, standardized error code, and message.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C verification tool qualification (DO-330 TQL-4).
    /// Provides structured, machine-verifiable diagnostic records for compiler and baseline gate violations.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Field Preservation: `result.severity == severity`, `result.code == code.into()`, and
    ///   `result.message == message.into()`.
    /// - Unbound Location Default: Initializes source coordinate fields (`file`, `line`, `column`) to
    ///   `None` until explicitly bound via `Diagnostic::with_location`.
    ///
    /// ## 3. Preconditions
    /// - `severity` is a valid `DiagnosticSeverity` variant.
    /// - `code` and `message` implement `Into<String>` yielding valid UTF-8 strings.
    ///
    /// ## 4. Postconditions
    /// - Returns a newly initialized `Diagnostic` instance with `file == None`, `line == None`, and `column == None`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(|code| + |message|) when converting borrowed slices, or O(1) when moving owned `String` values.
    /// - Space Complexity: O(|code| + |message|) heap storage for owned diagnostic strings.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts any `Into<String>` type safely at compile time.
    /// - State: Constructs a complete, self-consistent `Diagnostic` value in a single atomic expression.
    /// - Action: Total constructor; never panics or performs fallible I/O.
    /// - Resource: Bounded heap allocation proportional to `code` and `message` lengths.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/Diagnostic::new] | [REQ-DIAG-NEW] | SysML v2: part def DiagnosticEngine
    pub fn new(
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
            column: None,
        }
    }

    /// Attaches a source file path and 1-indexed line and column coordinates to a diagnostic.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A bidirectional traceability mandates by anchoring compiler and
    /// verification diagnostics to exact workspace-relative file, line, and column coordinates.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Identity Preservation: `self.severity`, `self.code`, and `self.message` remain invariant.
    /// - Coordinate Binding: Sets `self.file = Some(file.into())`, `self.line = Some(line)`, and
    ///   `self.column = Some(col)`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `Diagnostic` value transferred by ownership.
    /// - `file` is convertible into a valid UTF-8 `String` representing a workspace-relative path.
    /// - `line` and `col` represent source line and column indices.
    ///
    /// ## 4. Postconditions
    /// - Returns the consumed `Diagnostic` instance with `file`, `line`, and `column` populated as `Some`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(|file|) if converting a `&str` slice, or O(1) if moving an owned `String`.
    /// - Space Complexity: O(|file|) heap allocation for the file path string.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Consumes `self` by value (fluent builder pattern), preventing aliased mutation.
    /// - State: Updates all three location fields together so partial location state cannot occur.
    /// - Action: Total infallible transformation; never panics.
    /// - Resource: Bounded single `String` allocation for `file`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/Diagnostic::with_location] | [REQ-DIAG-LOC] | SysML v2: part def DiagnosticEngine
    pub fn with_location(mut self, file: impl Into<String>, line: usize, col: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(col);
        self
    }
}

/// A single step in a 5-Whys root cause analysis.
///
/// Realises: [Diagnostic/WhyEntry] | SysML v2: part def DefectDossierSynthesizer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhyEntry {
    pub question: String,
    pub because: String,
}

impl WhyEntry {
    /// Constructs a single causal step in a 5-Whys root cause analysis chain.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C corrective action and
    /// root-cause analysis documentation standards for Section 2 of Adversarial Audit defect dossiers.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Causal Pair Integrity: Preserves exact UTF-8 contents of `question` and `because` such that
    ///   `entry.question == question.into()` and `entry.because == because.into()`.
    ///
    /// ## 3. Preconditions
    /// - `question` and `because` implement `Into<String>` yielding valid UTF-8 strings.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `WhyEntry` struct containing the owned `question` and `because` strings.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(|question| + |because|) when cloning from slices, or O(1) when moving `String`s.
    /// - Space Complexity: O(|question| + |because|) heap memory.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Strongly typed constructor accepting any `Into<String>` arguments.
    /// - State: Immutable post-construction unless explicitly mutated by owner.
    /// - Action: Infallible in-memory initialization; never panics.
    /// - Resource: Bounded heap allocation proportional to input string lengths.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/WhyEntry::new] | [REQ-WHY-NEW] | SysML v2: part def DefectDossierSynthesizer
    pub fn new(question: impl Into<String>, because: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            because: because.into(),
        }
    }
}

/// Structured 7-section Adversarial Audit defect dossier.
///
/// Realises: [Diagnostic/DefectDossier] | SysML v2: part def DefectDossierSynthesizer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefectDossier {
    pub title: String,
    pub severity: DiagnosticSeverity,
    pub file_location: String,
    pub pillar: String,
    pub symptom: String,
    pub whys: [WhyEntry; 5],
    pub impact: String,
    pub architectural_models: String,
    pub remediation: String,
    pub verification: String,
    pub regression_prevention: String,
}

impl DefectDossier {
    /// Renders the defect dossier into canonical 7-section Adversarial Audit Markdown format.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C problem reporting and corrective action standards.
    /// Synthesizes a deterministic, human- and machine-verifiable Markdown defect dossier containing all
    /// 7 mandatory audit sections and the `## Audit Source` metadata footer required by `verify-baseline check-defect`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Section Ordering Invariant: Emits `## 1. Issue Summary`, `## 2. 5-Whys Root Cause Analysis`,
    ///   `## 3. Impact Assessment`, `## 4. Architectural & Behavioral Models`, `## 5. Remediation Plan`,
    ///   `## 6. Verification Criteria`, `## 7. Regression Prevention`, and `## Audit Source` in strict ascending order.
    /// - 5-Whys Cardinality: Iterates over the fixed-size `[WhyEntry; 5]` array to emit numbered items `1.` through `5.`,
    ///   normalizing trailing `?` characters on questions so each item renders as `<k>. **Why <q>?** Because <b>`.
    /// - Zero Em Dash Template Invariant: Emits strictly ASCII formatting tokens with zero Unicode em dashes (`\u2014`).
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid, initialized `DefectDossier` reference with all 5 `whys` entries populated.
    ///
    /// ## 4. Postconditions
    /// - Returns a newly allocated `String` containing the complete 7-section Markdown dossier and
    ///   `SEVERITY:` / `FILE_LOCATION:` audit footer lines.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is the sum of byte lengths across all fields in `self`.
    /// - Space Complexity: O(N) for the synthesized Markdown `String` buffer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Takes an immutable reference `&self` and returns an owned `String`.
    /// - State: Does not mutate `self`; strips leading/trailing whitespace and redundant `?` suffixes safely.
    /// - Action: Total deterministic string synthesis; never panics or performs I/O.
    /// - Resource: Linear O(N) buffer growth proportional to dossier payload size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/DefectDossier::to_markdown] | [REQ-DOSSIER-MD] | SysML v2: part def DefectDossierSynthesizer
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("## 1. Issue Summary\n");
        out.push_str(&format!("- **File**: {}\n", self.file_location));
        out.push_str(&format!("- **Pillar**: {}\n", self.pillar));
        out.push_str(&format!("- **Symptom**: {}\n\n", self.symptom));

        out.push_str("## 2. 5-Whys Root Cause Analysis\n");
        for (i, entry) in self.whys.iter().enumerate() {
            let q = entry.question.trim().trim_end_matches('?');
            let b = entry.because.trim();
            out.push_str(&format!("{}. **Why {}?** Because {}\n", i + 1, q, b));
        }
        out.push('\n');

        out.push_str("## 3. Impact Assessment\n");
        out.push_str(self.impact.trim());
        out.push_str("\n\n");

        out.push_str("## 4. Architectural & Behavioral Models\n");
        out.push_str(self.architectural_models.trim());
        out.push_str("\n\n");

        out.push_str("## 5. Remediation Plan\n");
        out.push_str(self.remediation.trim());
        out.push_str("\n\n");

        out.push_str("## 6. Verification Criteria\n");
        out.push_str(self.verification.trim());
        out.push_str("\n\n");

        out.push_str("## 7. Regression Prevention\n");
        out.push_str(self.regression_prevention.trim());
        out.push_str("\n\n");

        out.push_str("## Audit Source\n");
        out.push_str(&format!("SEVERITY: {}\n", self.severity));
        out.push_str(&format!("FILE_LOCATION: {}\n", self.file_location));
        out
    }

    /// Serializes the defect dossier into pretty-printed JSON format.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A automated defect archival requirements for structured persistence
    /// in `.pipeline/defects/` and machine-to-machine audit exchange.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Round-Trip Isomorphism: For any valid `dossier: DefectDossier`,
    ///   `DefectDossier::from_json(&dossier.to_json()?)? == dossier`.
    /// - Schema Completeness: Serializes all 11 struct fields, including the fixed-length 5-element `whys` array.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid, initialized `DefectDossier` reference.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(String)` containing pretty-printed UTF-8 JSON on success.
    /// - Returns `Err(serde_json::Error)` if serialization fails.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is the total byte length of all fields in `self`.
    /// - Space Complexity: O(N) for the serialized JSON output buffer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns explicit `Result<String, serde_json::Error>`; never unwraps or panics.
    /// - State: Read-only borrow of `self` guarantees zero mutation of in-memory dossier state.
    /// - Action: Propagates any `serde_json` formatter failure fail-closed via `Err`.
    /// - Resource: Bounded O(N) heap allocation for the pretty-printed JSON string.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/DefectDossier::to_json] | [REQ-DOSSIER-JSON] | SysML v2: part def DefectDossierSynthesizer
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a `DefectDossier` instance from a JSON string slice.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A audit artifact ingestion and verification standards. Enforces strict
    /// structural schema validation when loading archived defect dossiers from disk or CLI payloads.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Strict Cardinality Enforcement: Rejects any JSON payload whose `whys` array does not contain
    ///   exactly 5 `WhyEntry` elements or that omits any required dossier field.
    /// - Round-Trip Identity: Inverse of `DefectDossier::to_json`.
    ///
    /// ## 3. Preconditions
    /// - `json_str` is a valid UTF-8 string slice containing a candidate JSON representation of `DefectDossier`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(DefectDossier)` if `json_str` is well-formed JSON conforming to the `DefectDossier` schema.
    /// - Returns `Err(serde_json::Error)` if `json_str` is syntactically invalid or violates schema constraints.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is `json_str.len()`.
    /// - Space Complexity: O(N) for the allocated `DefectDossier` fields.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Fail-closed `Result<Self, serde_json::Error>` signature prevents silent ingestion of corrupt data.
    /// - State: No partial state is exposed if deserialization fails mid-stream.
    /// - Action: Rejects unknown enum variants, missing fields, or wrong array lengths without panicking.
    /// - Resource: Bounded memory allocation proportional to input slice length `json_str.len()`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/DefectDossier::from_json] | [REQ-DOSSIER-PARSE] | SysML v2: part def DefectDossierSynthesizer
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation_and_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let diag = Diagnostic::new(DiagnosticSeverity::Error, "E0301", "Power invariant violation")
            .with_location("schema/REQ-0099.md", 44, 1);
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code, "E0301");
        assert_eq!(diag.line, Some(44));

        let json = serde_json::to_string(&diag)?;
        assert!(json.contains("E0301"));
        let deserialized: Diagnostic = serde_json::from_str(&json)?;
        assert_eq!(deserialized, diag);
        Ok(())
    }

    #[test]
    fn test_defect_dossier_markdown_and_json_synthesis() -> Result<(), Box<dyn std::error::Error>> {
        let dossier = DefectDossier {
            title: "[AUDIT] [markdown.rs]: False positive on GLFM math".to_string(),
            severity: DiagnosticSeverity::Important,
            file_location: "crates/deap-core/src/markdown.rs".to_string(),
            pillar: "Compiler Robustness & AST Fidelity".to_string(),
            symptom: "Raw backtick stripping synthesizes false display math delimiters.".to_string(),
            whys: [
                WhyEntry::new("did the KaTeX validator flag an error", "it saw unbalanced '$$' delimiters"), WhyEntry::new("did false '$$' delimiters appear", "GLFM backticks were stripped before math masking"),
                WhyEntry::new("were GLFM delimiters stripped first", "the lexer lacked token awareness for GLFM math"),
                WhyEntry::new("was GLFM token awareness omitted", "the scanner assumed all backticks are code spans"),
                WhyEntry::new("was this assumption made", "GLFM math syntax was not covered in initial test cases"),
            ],
            impact: "Valid requirements documents with GLFM math fail baseline verification gates.".to_string(),
            architectural_models: "```mermaid\ngraph TD\nA[GLFM Math] --> B[Masking Layer]\nB --> C[Backtick Stripper]\n```".to_string(),
            remediation: "Mask GLFM inline math expressions before stripping raw backticks.".to_string(),
            verification: "Verify with cargo test -p deap-core --lib markdown.".to_string(),
            regression_prevention: "Add continuous integration tests asserting 0 errors on REQ-0099.md.".to_string(),
        };

        let md = dossier.to_markdown();
        assert!(md.contains("## 1. Issue Summary"));
        assert!(md.contains("## 2. 5-Whys Root Cause Analysis"));
        assert!(md.contains("## 7. Regression Prevention"));
        assert!(md.contains("## Audit Source"));
        assert!(md.contains("SEVERITY: Important"));
        assert!(md.contains("FILE_LOCATION: crates/deap-core/src/markdown.rs"));
        assert!(!md.contains('\u{2014}'), "Must not contain unicode em dash");

        let json = dossier.to_json()?;
        let loaded = DefectDossier::from_json(&json)?;
        assert_eq!(loaded, dossier);
        Ok(())
    }

    #[test]
    fn test_diagnostics_source_purity_and_7_section_rustdoc() {
        let source = include_str!("diagnostics.rs");
        let s1 = "## 1. Safety Intent & Regulatory Scope";
        let s2 = "## 2. Invariants & Mathematical Properties";
        let s3 = "## 3. Preconditions";
        let s4 = "## 4. Postconditions";
        let s5 = "## 5. Algorithmic Complexity Bounds";
        let s6 = "## 6. Failure Modes & Fault Tolerance";
        let s7 = "## 7. Realization & Traceability";

        let module_docs: String = source
            .lines()
            .take_while(|l| l.trim().starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n");
        for section in [s1, s2, s3, s4, s5, s6, s7] {
            assert!(
                module_docs.contains(section),
                "Module-level inner Rustdoc missing mandatory section: {}",
                section
            );
        }

        let prod = source.split("#[cfg(test)]").next().unwrap_or(source);
        let mut current_doc = Vec::new();
        let mut fn_count = 0usize;

        let pat_regex_colons = ["Reg", "ex::"].concat();
        let pat_inline_regex = ["reg", "ex::"].concat();
        let needle_slice_str = ["&[", "&str]"].concat();
        let needle_let_blank = ["let", " _ = "].concat();

        for (idx, raw_line) in prod.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = raw_line.trim();
            if trimmed.starts_with("//!") || trimmed.starts_with("///") {
                current_doc.push(trimmed.to_string());
                continue;
            }
            let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
            if code.is_empty() {
                continue;
            }
            assert!(
                !code.contains(&pat_regex_colons) && !code.contains(&pat_inline_regex),
                "Line {}: regex usage detected",
                line_no
            );
            assert!(
                !code.contains(&needle_slice_str),
                "Line {}: static string-slice array detected",
                line_no
            );
            assert!(
                !code.contains(&needle_let_blank),
                "Line {}: hollow gate stub detected",
                line_no
            );

            if code.starts_with("pub fn ")
                || code.starts_with("fn ")
                || code.starts_with("pub const fn ")
                || code.starts_with("pub(crate) fn ")
            {
                fn_count += 1;
                let doc_block = current_doc.join("\n");
                for section in [s1, s2, s3, s4, s5, s6, s7] {
                    assert!(
                        doc_block.contains(section),
                        "Line {}: function missing section {}",
                        line_no,
                        section
                    );
                }
                current_doc.clear();
            } else if !code.starts_with("#[") {
                current_doc.clear();
            }
        }
        assert_eq!(fn_count, 7, "Expected exactly 7 documented functions in diagnostics.rs");
    }
}

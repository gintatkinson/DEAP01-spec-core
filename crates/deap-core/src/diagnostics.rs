//! Diagnostic reporting and defect dossier synthesis.

use serde::{Deserialize, Serialize};

/// Severity levels for diagnostics and defect audit dossiers.
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

    pub fn with_location(mut self, file: impl Into<String>, line: usize, col: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(col);
        self
    }
}

/// A single step in a 5-Whys root cause analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhyEntry {
    pub question: String,
    pub because: String,
}

impl WhyEntry {
    pub fn new(question: impl Into<String>, because: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            because: because.into(),
        }
    }
}

/// Structured 7-section Adversarial Audit defect dossier.
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
    /// Render the defect dossier into Markdown conforming to scripts/file_defect.py.
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

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation_and_serialization() {
        let diag = Diagnostic::new(DiagnosticSeverity::Error, "E0301", "Power invariant violation")
            .with_location("schema/REQ-0099.md", 44, 1);

        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code, "E0301");
        assert_eq!(diag.line, Some(44));

        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("E0301"));
        let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, diag);
    }

    #[test]
    fn test_defect_dossier_markdown_and_json_synthesis() {
        let dossier = DefectDossier {
            title: "[AUDIT] [markdown.rs]: False positive on GLFM math".to_string(),
            severity: DiagnosticSeverity::Important,
            file_location: "crates/deap-core/src/markdown.rs".to_string(),
            pillar: "Compiler Robustness & AST Fidelity".to_string(),
            symptom: "Raw backtick stripping synthesizes false display math delimiters.".to_string(),
            whys: [
                WhyEntry::new("did the KaTeX validator flag an error", "it saw unbalanced '$$' delimiters"),
                WhyEntry::new("did false '$$' delimiters appear", "GLFM backticks were stripped before math masking"),
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

        let json = dossier.to_json().unwrap();
        let loaded = DefectDossier::from_json(&json).unwrap();
        assert_eq!(loaded, dossier);
    }
}

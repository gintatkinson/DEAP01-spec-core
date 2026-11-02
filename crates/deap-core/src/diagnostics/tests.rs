//! Unit tests for diagnostics, defect dossier synthesis, and source purity.

use super::*;

#[test]
fn test_diagnostic_creation_and_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let diag = Diagnostic::new(
        DiagnosticSeverity::Error,
        "E0301",
        "Power invariant violation",
    )
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

    let json = dossier.to_json()?;
    let loaded = DefectDossier::from_json(&json)?;
    assert_eq!(loaded, dossier);
    Ok(())
}

#[test]
fn test_diagnostics_source_purity_and_7_section_rustdoc() {
    let sources = [
        include_str!("mod.rs"),
        include_str!("severity.rs"),
        include_str!("item.rs"),
        include_str!("dossier.rs"),
    ];

    let s1 = "## 1. Safety Intent & Regulatory Scope";
    let s2 = "## 2. Invariants & Mathematical Properties";
    let s3 = "## 3. Preconditions";
    let s4 = "## 4. Postconditions";
    let s5 = "## 5. Algorithmic Complexity Bounds";
    let s6 = "## 6. Failure Modes & Fault Tolerance";
    let s7 = "## 7. Realization & Traceability";
    let mandatory_sections = [s1, s2, s3, s4, s5, s6, s7];

    let mod_doc: String = sources[0]
        .lines()
        .take_while(|l| l.trim().starts_with("//!"))
        .collect::<Vec<_>>()
        .join("\n");
    for section in mandatory_sections {
        assert!(
            mod_doc.contains(section),
            "Module-level inner Rustdoc missing mandatory section: {}",
            section
        );
    }

    let pat_regex_colons = ["Reg", "ex::"].concat();
    let pat_inline_regex = ["reg", "ex::"].concat();
    let needle_slice_str = ["&[", "&str]"].concat();
    let needle_let_blank = ["let", " _ = "].concat();

    let mut fn_count = 0usize;

    for (file_idx, source) in sources.iter().enumerate() {
        let prod = source.split("#[cfg(test)]").next().unwrap_or(source);
        let mut current_doc = Vec::new();

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
                "File {} Line {}: regex usage detected",
                file_idx,
                line_no
            );
            assert!(
                !code.contains(&needle_slice_str),
                "File {} Line {}: static string-slice array detected",
                file_idx,
                line_no
            );
            assert!(
                !code.contains(&needle_let_blank),
                "File {} Line {}: hollow gate stub detected",
                file_idx,
                line_no
            );

            if code.starts_with("pub fn ")
                || code.starts_with("fn ")
                || code.starts_with("pub const fn ")
                || code.starts_with("pub(crate) fn ")
            {
                fn_count += 1;
                let doc_block = current_doc.join("\n");
                for section in mandatory_sections {
                    assert!(
                        doc_block.contains(section),
                        "File {} Line {}: function missing section {}",
                        file_idx,
                        line_no,
                        section
                    );
                }
                current_doc.clear();
            } else if !code.starts_with("#[") {
                current_doc.clear();
            }
        }
    }

    assert_eq!(
        fn_count, 7,
        "Expected exactly 7 documented functions across diagnostics module files"
    );
}

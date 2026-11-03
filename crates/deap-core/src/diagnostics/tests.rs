//! Unit tests for diagnostics, defect dossier synthesis, and source purity.
use super::*;

/// Verifies `Diagnostic` creation, location attachment, and JSON round-trip. Realises: [REQ-0608]
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

/// Verifies `DefectDossier` Markdown rendering and JSON round-trip serialization. Realises: [REQ-0608]
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

/// Verifies module-level 7-section inner Rustdoc on `diagnostics/mod.rs`. Realises: [REQ-0608]
#[test]
fn test_diagnostics_module_docstrings() {
    let sources = [
        include_str!("mod.rs"),
        include_str!("severity.rs"),
        include_str!("item.rs"),
        include_str!("dossier.rs"),
    ];
    let mandatory_sections = [
        "## 1. Safety Intent & Regulatory Scope",
        "## 2. Invariants & Mathematical Properties",
        "## 3. Preconditions",
        "## 4. Postconditions",
        "## 5. Algorithmic Complexity Bounds",
        "## 6. Failure Modes & Fault Tolerance",
        "## 7. Realization & Traceability",
    ];
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
}

/// Verifies production lines in `source` contain no regex, static string-slice arrays, or hollow stubs. Realises: [REQ-0608]
fn check_file_source_purity(file_idx: usize, source: &str) {
    let pat_regex_colons = ["Reg", "ex::"].concat();
    let pat_inline_regex = ["reg", "ex::"].concat();
    let needle_slice_str = ["&[", "&str]"].concat();
    let needle_let_blank = ["let", " _ = "].concat();
    let prod = source.split("#[cfg(test)]").next().unwrap_or(source);
    for (idx, raw_line) in prod.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.starts_with("//!") || trimmed.starts_with("///") {
            continue;
        }
        let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
        if code.is_empty() {
            continue;
        }
        assert!(
            !code.contains(&pat_regex_colons) && !code.contains(&pat_inline_regex),
            "File {file_idx} Line {line_no}: regex usage detected"
        );
        assert!(
            !code.contains(&needle_slice_str),
            "File {file_idx} Line {line_no}: static string-slice array detected"
        );
        assert!(
            !code.contains(&needle_let_blank),
            "File {file_idx} Line {line_no}: hollow gate stub detected"
        );
    }
}

/// Verifies every function declaration in `source` carries all 7 mandatory Rustdoc sections. Realises: [REQ-0608]
fn check_file_7_section_rustdoc(
    file_idx: usize,
    source: &str,
    mandatory_sections: &[&str],
    fn_count: &mut usize,
) {
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
        let is_fn = code.starts_with("pub fn ")
            || code.starts_with("fn ")
            || code.starts_with("pub const fn ")
            || code.starts_with("pub(crate) fn ");
        if is_fn {
            *fn_count += 1;
            let doc_block = current_doc.join("\n");
            for section in mandatory_sections {
                assert!(
                    doc_block.contains(section),
                    "File {file_idx} Line {line_no}: function missing section {section}"
                );
            }
            current_doc.clear();
        } else if !code.starts_with("#[") {
            current_doc.clear();
        }
    }
}

/// Verifies source purity across all `diagnostics` module files. Realises: [REQ-0608]
#[test]
fn test_diagnostics_source_purity() {
    let sources = [
        include_str!("mod.rs"),
        include_str!("severity.rs"),
        include_str!("item.rs"),
        include_str!("dossier.rs"),
    ];
    for (file_idx, source) in sources.iter().enumerate() {
        check_file_source_purity(file_idx, source);
    }
}

/// Verifies 7-section Rustdoc coverage across all 9 functions in `diagnostics`. Realises: [REQ-0608]
#[test]
fn test_diagnostics_7_section_rustdoc() {
    let sources = [
        include_str!("mod.rs"),
        include_str!("severity.rs"),
        include_str!("item.rs"),
        include_str!("dossier.rs"),
    ];
    let mandatory_sections = [
        "## 1. Safety Intent & Regulatory Scope",
        "## 2. Invariants & Mathematical Properties",
        "## 3. Preconditions",
        "## 4. Postconditions",
        "## 5. Algorithmic Complexity Bounds",
        "## 6. Failure Modes & Fault Tolerance",
        "## 7. Realization & Traceability",
    ];
    let mut fn_count = 0usize;
    for (file_idx, source) in sources.iter().enumerate() {
        check_file_7_section_rustdoc(file_idx, source, &mandatory_sections, &mut fn_count);
    }
    assert_eq!(
        fn_count, 9,
        "Expected exactly 9 documented functions across diagnostics module files"
    );
}

/// Verifies single-span and multi-span `ariadne` diagnostic rendering. Realises: [REQ-0608]
#[test]
fn test_render_diagnostic_with_ariadne_multispan_color_and_note() {
    let diag = Diagnostic::new(DiagnosticSeverity::Error, "E0301", "Power invariant violation")
        .with_location("schema/REQ-0099.md", 44, 1);
    let src = "Interface_Gamma across e through f;\npower = e * f;\n";
    let rendered = render_diagnostic_with_ariadne(&diag, "schema/REQ-0099.md", src, 0..15);
    assert!(rendered.contains("E0301"));
    assert!(rendered.contains("Power invariant violation"));
    assert!(rendered.contains("severity: Error"));
    let spans = vec![
        (0..15, "primary violation".to_string(), ariadne::Color::Red),
        (36..49, "dependent power expr".to_string(), ariadne::Color::Yellow),
    ];
    let multispan = render_diagnostic_multispan_with_ariadne(
        &diag,
        "schema/REQ-0099.md",
        src,
        &spans,
        Some("DO-178C Level A invariant violated"),
    );
    assert!(multispan.contains("primary violation"));
    assert!(multispan.contains("dependent power expr"));
    assert!(multispan.contains("DO-178C Level A invariant violated"));
}

/// Verifies UTF-8 char-boundary snapping and inverted range clamping. Realises: [REQ-0608]
#[test]
fn rejects_inverted_or_mid_codepoint_utf8_spans() {
    let err = DiagnosticEmitError::InvalidUtf8Span;
    assert_eq!(err.diagnostic_code(), "DEAP-E0801");
    assert!(err.to_string().contains("DEAP-E0801"));
    let src = "\u{03b1}\u{03b2} = 42;\n";
    assert!(!src.is_char_boundary(1));
    let mid = resolve_utf8_span(src, 1..99, "src\\main.rs");
    assert_eq!(mid.file_name, "src/main.rs");
    assert_eq!(mid.byte_start, 0);
    assert_eq!(mid.byte_end, src.len());
    assert!(src.is_char_boundary(mid.byte_start));
    assert!(src.is_char_boundary(mid.byte_end));
    let inverted = resolve_utf8_span(src, std::ops::Range { start: 4, end: 2 }, "src/main.rs");
    assert_eq!(inverted.byte_start, 4);
    assert_eq!(inverted.byte_end, 4);
    assert_eq!(inverted.line_start, 1);
    assert_eq!(inverted.column_start, 3);
}

/// Verifies single-line zero-ANSI NDJSON formatting of `RustcCompilerMessageEnvelope`. Realises: [REQ-0608]
#[test]
fn formats_single_line_zero_ansi_rustc_ndjson() -> Result<(), serde_json::Error> {
    let mut span = resolve_utf8_span("let value = 10;\n", 4..9, "src/lib.rs");
    span.suggested_replacement = Some(String::from("_value"));
    span.suggestion_applicability = Some(Applicability::MachineApplicable);
    let env = RustcCompilerMessageEnvelope {
        message_type: String::new(),
        message: String::from("unused variable `value`"),
        code: Some(String::from("DEAP-E0801")),
        level: String::from("warning"),
        spans: vec![span],
        children: Vec::new(),
        rendered: Some(String::from("\x1b[33mwarning[DEAP-E0801]\x1b[0m\r\n")),
    };
    let line = format_ndjson_line(&env)?;
    assert!(line.contains("\"$message_type\":\"diagnostic\""));
    assert!(!line.contains('\n'));
    assert!(!line.contains('\r'));
    assert!(!line.contains('\x1b'));
    assert!(!line.contains("\\u001b"));
    let parsed: RustcCompilerMessageEnvelope = serde_json::from_str(&line)?;
    assert_eq!(parsed.message_type, "diagnostic");
    assert_eq!(parsed.code.as_deref(), Some("DEAP-E0801"));
    Ok(())
}

/// Verifies reverse byte-order span splicing and overlap rejection. Realises: [REQ-0608]
#[test]
fn applies_reverse_byte_order_machine_fixes_and_rejects_overlaps() {
    assert_eq!(
        DiagnosticEmitError::OverlappingMachineFixSpan.diagnostic_code(),
        "DEAP-E0802"
    );
    assert_eq!(
        DiagnosticEmitError::PostFixSyntaxError(String::from("syntax")).diagnostic_code(),
        "DEAP-E0803"
    );
    let src = "fn demo() { let a = 1; let b = 2; }\n";
    let edits = [
        crate::rust_cst::SpanSpliceEdit {
            span: crate::rust_cst::Utf8ByteSpan {
                byte_start: 16,
                byte_end: 17,
            },
            replacement: String::from("_a"),
        },
        crate::rust_cst::SpanSpliceEdit {
            span: crate::rust_cst::Utf8ByteSpan {
                byte_start: 27,
                byte_end: 28,
            },
            replacement: String::from("_b"),
        },
    ];
    let fixed = crate::rust_cst::splice_disjoint_reverse_spans(src, &edits);
    assert_eq!(
        fixed.as_deref(),
        Ok("fn demo() { let _a = 1; let _b = 2; }\n")
    );
    let overlapping = [
        crate::rust_cst::SpanSpliceEdit {
            span: crate::rust_cst::Utf8ByteSpan {
                byte_start: 12,
                byte_end: 20,
            },
            replacement: String::from("let x"),
        },
        crate::rust_cst::SpanSpliceEdit {
            span: crate::rust_cst::Utf8ByteSpan {
                byte_start: 16,
                byte_end: 22,
            },
            replacement: String::from("_a = 1"),
        },
    ];
    assert!(crate::rust_cst::splice_disjoint_reverse_spans(src, &overlapping).is_err());
}

/// Verifies NDJSON stream emission across all five CLI adapter envelopes. Realises: [REQ-0608]
#[test]
fn emits_locked_ndjson_across_all_five_cli_adapters() -> Result<(), serde_json::Error> {
    assert_eq!(
        DiagnosticEmitError::SerializationError(String::from("json")).diagnostic_code(),
        "DEAP-E0804"
    );
    assert_eq!(
        DiagnosticEmitError::IoError(String::from("stdout")).diagnostic_code(),
        "DEAP-E0804"
    );
    let adapters = [
        ("verify-baseline", "DEAP-E0801"),
        ("compile-sysml", "DEAP-E0802"),
        ("ingest-sysml", "DEAP-E0803"),
        ("reconcile-backlog", "DEAP-E0804"),
        ("assemble-conops", "DEAP-E0801"),
    ];
    let envelopes: Vec<RustcCompilerMessageEnvelope> = adapters
        .iter()
        .map(|(cli, code)| RustcCompilerMessageEnvelope {
            message_type: String::from("diagnostic"),
            message: format!("{cli} diagnostic check"),
            code: Some((*code).to_string()),
            level: String::from("error"),
            spans: vec![resolve_utf8_span("package Demo {}\n", 0..7, "schema/demo.sysml")],
            children: Vec::new(),
            rendered: Some(format!("\x1b[31m{cli}: {code}\x1b[0m\n")),
        })
        .collect();
    let stream = emit_cli_ndjson_stream(&envelopes);
    let lines: Vec<&str> = stream.lines().collect();
    assert_eq!(lines.len(), 5);
    for (idx, line) in lines.iter().enumerate() {
        assert!(!line.contains('\x1b'));
        let decoded: RustcCompilerMessageEnvelope = serde_json::from_str(line)?;
        assert_eq!(decoded.message_type, "diagnostic");
        assert!(adapters.get(idx).is_some_and(|(name, _)| decoded.message.contains(name)));
    }
    Ok(())
}

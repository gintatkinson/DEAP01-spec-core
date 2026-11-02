#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless Rust concrete syntax tree parsing and SLOC analysis. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D deterministic verification.
//! Exposes lossless CST parsing, syntax error diagnostics, and executable SLOC counting.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Lossless round-tripping: `tree().syntax().to_string() == source`.
//! - Line and column indexing is 1-based and deterministic across platforms.
//! - Executable SLOC strictly excludes comments, blank lines, and cfg(test) items.
//!
//! ## 3. Preconditions
//! - Source text is valid UTF-8.
//!
//! ## 4. Postconditions
//! - Parsing never panics even on ill-formed syntax, capturing syntax errors via `errors()`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Parsing: $O(N)$ linear time in source text length $N$.
//! - Line/col lookup: $O(\log L)$ where $L$ is total line count.
//! - Executable SLOC: $O(T)$ where $T$ is CST token count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Syntax errors are collected in `syntax_errors()` without panic.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_cst_parse]
//! SysML v2: part def ParsedRustFile

use ra_ap_line_index::LineIndex;
use ra_ap_syntax::ast::{self, AstNode, HasAttrs};
use ra_ap_syntax::{Edition, Parse, SourceFile, SyntaxKind, SyntaxNode, TextSize};
use std::collections::BTreeSet;

/// Lossless Rust syntax tree and line index representation.
/// Realises: [REQ-0608], [SC-01]
#[derive(Debug, Clone)]
pub struct ParsedRustFile {
    source: String,
    parsed: Parse<SourceFile>,
    line_index: LineIndex,
}

impl ParsedRustFile {
    /// Parses Rust source text into a lossless CST with line index.
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn parse_rust_source(source: &str) -> Self {
        let parsed = SourceFile::parse(source, Edition::CURRENT);
        let line_index = LineIndex::new(source);
        Self {
            source: source.to_string(),
            parsed,
            line_index,
        }
    }

    /// Returns the underlying parsed `SourceFile` root AST node.
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn tree(&self) -> SourceFile {
        self.parsed.tree()
    }

    /// Returns the root `SyntaxNode` of the concrete syntax tree.
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn syntax(&self) -> SyntaxNode {
        self.parsed.tree().syntax().clone()
    }

    /// Returns the original source string.
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns a reference to the line index.
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn line_index(&self) -> &LineIndex {
        &self.line_index
    }

    /// Collects all syntax error messages encountered during parsing.
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn syntax_errors(&self) -> Vec<String> {
        self.parsed.errors().iter().map(|e| e.to_string()).collect()
    }

    /// Verifies that serializing the CST exactly equals the original input.
    /// Realises: [REQ-0608], [SC-01]
    #[must_use]
    pub fn round_trip_lossless(&self) -> bool {
        self.parsed.tree().syntax().to_string() == self.source
    }

    /// Converts a text byte offset into 1-based (line, column).
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn line_col(&self, offset: TextSize) -> (usize, usize) {
        let lc = self.line_index.line_col(offset);
        (lc.line as usize + 1, lc.col as usize + 1)
    }

    /// Converts a usize byte offset into 1-based (line, column).
    /// Realises: [REQ-0608], [FLS: fls_cst_parse]
    #[must_use]
    pub fn line_col_at_byte(&self, byte_offset: usize) -> (usize, usize) {
        let offset = TextSize::try_from(byte_offset).unwrap_or_default();
        self.line_col(offset)
    }

    /// Computes executable SLOC strictly excluding blank lines, comments, and cfg(test).
    /// Realises: [REQ-0608], [SC-01]
    #[must_use]
    pub fn executable_sloc(&self) -> usize {
        let root = self.syntax();
        let mut executable_lines = BTreeSet::new();

        for element in root.descendants_with_tokens() {
            let Some(token) = element.into_token() else {
                continue;
            };
            if token.kind() == SyntaxKind::WHITESPACE || token.kind() == SyntaxKind::COMMENT {
                continue;
            }
            if is_inside_cfg_test(&token) {
                continue;
            }
            let start = self.line_col(token.text_range().start()).0;
            let end = self.line_col(token.text_range().end()).0;
            for line in start..=end {
                executable_lines.insert(line);
            }
        }

        executable_lines.len()
    }
}

/// Checks whether a syntax element resides within a cfg(test) item or module.
/// Realises: [REQ-0608], [FLS: fls_cst_parse]
fn is_inside_cfg_test(token: &ra_ap_syntax::SyntaxToken) -> bool {
    let mut current = token.parent();
    while let Some(node) = current {
        if let Some(item) = ast::Item::cast(node.clone()) {
            for attr in item.attrs() {
                if attr.syntax().text().to_string().contains("cfg(test)") {
                    return true;
                }
            }
        }
        current = node.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_round_trip_lossless() {
        let code = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let parsed = ParsedRustFile::parse_rust_source(code);
        assert!(parsed.syntax_errors().is_empty());
        assert!(parsed.round_trip_lossless());
        assert_eq!(parsed.source(), code);
    }

    #[test]
    fn test_syntax_errors_collected() {
        let invalid = "fn broken( {";
        let parsed = ParsedRustFile::parse_rust_source(invalid);
        assert!(!parsed.syntax_errors().is_empty());
        assert!(parsed.round_trip_lossless());
    }

    #[test]
    fn test_executable_sloc_excludes_comments_and_cfg_test() {
        let code = r#"
// Top-level comment
/* Block comment */

pub fn live_code() -> bool {
    let x = 42;
    x > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert!(live_code());
    }
}
"#;
        let parsed = ParsedRustFile::parse_rust_source(code);
        // Live lines: `pub fn live_code() -> bool {`, `    let x = 42;`, `    x > 0`, `}`
        assert_eq!(parsed.executable_sloc(), 4);
    }

    #[test]
    fn test_line_col_1_based() {
        let code = "abc\ndef";
        let parsed = ParsedRustFile::parse_rust_source(code);
        assert_eq!(parsed.line_col_at_byte(0), (1, 1));
        assert_eq!(parsed.line_col_at_byte(4), (2, 1));
    }
}

//! Recursive descent parser translating SysML v2 / KerML tokens into typed AST.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses lexed SysML v2 / KerML token streams and multi-file directory trees into deterministic `PackageDef` ASTs.
//! Bounds & Faults: Caps recursive-descent and package-merge nesting depth at `MAX_PARSE_DEPTH` (64), returning typed `ParseError` diagnostics without panicking.

mod behavioral_decls;
mod behavioral_transitions;
mod behavioral_use_cases;
mod safety_and_reqs;
mod safety_hazards_and_tests;
mod sigils_and_cursor;
mod structural_connections;
mod structural_decls;
mod structural_parts;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_composite_types;
#[cfg(test)]
mod tests_inner_blocks;
#[cfg(test)]
mod tests_satisfy;
#[cfg(test)]
mod tests_sigils_and_transitions;
#[cfg(test)]
mod tests_tree_and_decls;
#[cfg(test)]
mod tests_tree_loader;
pub mod tree_loader;

use crate::lexer::scanner::Scanner;
use crate::lexer::token::{Span, Token, TokenKind};
use deap_core::sysml_ast::*;
pub(crate) use sigils_and_cursor::*;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
pub use tree_loader::*;

/// Maximum recursive-descent and package-merge nesting depth permitted in `SysmlParser` and `merge_package_defs`.
/// Exceeding `MAX_PARSE_DEPTH` (64) terminates recursion cleanly without overflowing the call stack.
/// Realises: [REQ-PARSER-GRAMMAR-MAX-PARSE-DEPTH]
pub(crate) const MAX_PARSE_DEPTH: usize = 64;

/// Diagnostic syntax or filesystem loading error paired with a 1-based source location `Span`.
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-PARSEERROR]
/// Contract: Carries a deterministic human-readable diagnostic `message` and the offending lexical `span` (`line`, `col`, `start`, `end`).
/// Bounds & Faults: Value type implementing `Display` and `std::error::Error` with zero panics or side effects.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    /// Formats `ParseError` with 1-based line, column, and diagnostic message.
    /// Realises: [REQ-PARSER-GRAMMAR-PARSEERROR-FMT]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, col {}: {}",
            self.span.line, self.span.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Depth-bounded recursive-descent cursor translating a SysML v2 / KerML `Token` slice into typed AST definitions.
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER]
/// Contract: Maintains cursor index `pos` over `tokens` and active nesting `depth <= MAX_PARSE_DEPTH`, restoring `depth` across both `Ok` and `Err` paths.
/// Bounds & Faults: Rejects nesting beyond `MAX_PARSE_DEPTH` (64) with a typed `ParseError` and uses checked `.get()` slice indexing throughout.
pub struct SysmlParser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) pos: usize,
    pub(crate) depth: usize,
}

impl<'a> SysmlParser<'a> {
    /// Initializes a `SysmlParser` cursor positioned at token index `0` and recursion depth `0`.
    ///
    /// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-NEW]
    /// Contract: Binds `tokens` without copying and sets `pos == 0` and `depth == 0`.
    /// Bounds & Faults: Constant `O(1)` time and `O(1)` space infallible constructor.
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            depth: 0,
        }
    }

    /// Constructs a deterministic `ParseError` at the current cursor span when `MAX_PARSE_DEPTH` is reached.
    /// Realises: [REQ-PARSER-GRAMMAR-MAX-PARSE-DEPTH]
    pub(crate) fn max_depth_error(&self) -> ParseError {
        let tok = self.peek();
        ParseError {
            message: format!("Maximum SysML parse recursion depth ({MAX_PARSE_DEPTH}) exceeded"),
            span: tok.span,
        }
    }

    /// Tokenizes and parses complete SysML v2 / KerML source text into a root `PackageDef`.
    ///
    /// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-SOURCE]
    /// Contract: Normalizes and scans `source` via `SysmlParser::tokenize_source`, then parses top-level packages or flat container members under `default_pkg_name`.
    /// Bounds & Faults: Bounded by `MAX_PARSE_DEPTH` (64) and linear `O(N)` token stream size; returns a typed `ParseError` on any lexical, syntactic, or depth-limit violation.
    pub fn parse_source(source: &str, default_pkg_name: &str) -> Result<PackageDef, ParseError> {
        let tokens = Self::tokenize_source(source)?;
        let mut parser = SysmlParser::new(&tokens);
        parser.parse_package_or_file(default_pkg_name)
    }
}

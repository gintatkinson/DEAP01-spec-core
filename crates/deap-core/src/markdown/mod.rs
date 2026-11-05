//! Markdown token scanner, KaTeX, and math block parser.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides deterministic Markdown sanitization, KaTeX delimiter
//! balancing, Mermaid syntax verification, and CommonMark AST parsing and formatting.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exports lossless AST models, mathematical syntax checkers, and diagram validators.
//! - Code and math sanitization preserves line and column coordinate invariants.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 Markdown text.
//!
//! ## 4. Postconditions
//! - Exposes cohesive markdown processing surface failing closed on corrupt syntax.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) bounded linearly by document length.
//! - Space Complexity: O(N) auxiliary memory for parsed AST and diagnostic collections.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Syntax and formatting errors propagate fail-closed via explicit `Result` or diagnostic returns.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-MD-MODULE], [DO-330-TQL-4-MARKDOWN].

pub mod alignment;
pub mod ast;
pub mod katex;
pub mod mermaid;
pub mod mermaid_catalog;
pub mod sanitization;

pub use alignment::{AllowedAlignmentEnv, AllowedAlignmentEnvsCatalog, ALLOWED_ALIGNMENT_ENVS};
pub use ast::{
    format_markdown_cmark, parse_markdown_ast, MarkdownCodeBlockAst, MarkdownSectionAst,
    MarkdownTableAst, ParsedMarkdownAst,
};
pub use katex::check_latex_katex_syntax;
pub use mermaid::check_mermaid_syntax;
pub use mermaid_catalog::{
    ValidMermaidHeader, ValidMermaidHeadersCatalog, VALID_MERMAID_HEADERS,
};
pub use sanitization::{
    strip_code_and_glfm, strip_fenced_code_blocks, strip_glfm_inline_math, strip_inline_code,
};

#[cfg(test)]
mod tests;

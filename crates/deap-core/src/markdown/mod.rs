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
pub use crate::markdown_ast as ast;
pub mod katex;
pub mod mermaid;
pub mod mermaid_catalog;
pub mod sanitization;

pub use alignment::{AllowedAlignmentEnv, AllowedAlignmentEnvsCatalog, ALLOWED_ALIGNMENT_ENVS};
pub use ast::{
    cmark_options, collect_code_block_byte_ranges, emit_cmark_table, emit_lossless_markdown,
    format_markdown_cmark, is_br_html_tag, is_gfm_delimiter_cell, is_offset_in_byte_ranges,
    mask_html_comments_preserving_offsets, normalize_line_endings, parse_checklist_tasks,
    parse_frontmatter_kv_rows, parse_frontmatter_metadata, parse_markdown_ast,
    parse_markdown_tables, parse_single_gfm_row_cells, splice_markdown_byte_spans,
    write_spec_file_atomic, ChecklistDoc, CmarkTable, CompilerError, FrontmatterDoc,
    MarkdownCodeBlockAst, MarkdownKeyValueRow, MarkdownSectionAst, MarkdownTableAst,
    MarkdownTaskItemAst, ParsedMarkdownAst, TaskListItem,
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
#[path = "tests.rs"]
mod unit_tests;

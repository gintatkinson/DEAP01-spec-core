//! AST-guided frontmatter table field mutation and insertion helpers.
//!
//! Uses `deap_core::markdown_ast::parse_markdown_ast` and `pulldown_cmark` offset spans
//! to locate H2 section boundaries and fenced code blocks without ad-hoc fence state machines.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER]

use crate::ReconcileError;
use deap_core::markdown_ast::parse_markdown_ast;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::ops::Range;

/// Returns the byte offset of the first H2 (`## `) section in `content` using `deap_core::markdown_ast`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub fn first_h2_byte_offset(content: &str) -> usize {
    let ast = parse_markdown_ast(content);
    ast.sections
        .iter()
        .find(|s| s.level == 2)
        .map(|s| s.start_offset)
        .unwrap_or(content.len())
}

/// Collects byte offset ranges of all fenced/indented `Tag::CodeBlock` nodes in `content` via `pulldown-cmark`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub fn collect_code_block_ranges(content: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut active_start: Option<usize> = None;
    for (event, range) in Parser::new_ext(content, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                active_start = Some(range.start);
            }
            Event::End(TagEnd::CodeBlock) => {
                let start = active_start.take().unwrap_or(range.start);
                ranges.push(start..range.end);
            }
            _ => {}
        }
    }
    if let Some(start) = active_start {
        ranges.push(start..content.len());
    }
    ranges
}

/// Checks whether `byte_offset` falls inside any range in `ranges`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub fn is_offset_in_ranges(byte_offset: usize, ranges: &[Range<usize>]) -> bool {
    ranges
        .iter()
        .any(|r| byte_offset >= r.start && byte_offset < r.end)
}

/// [DOC-TIER: A] Updates or inserts a single frontmatter field in the Markdown table.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELD]
/// Contract: Delegates to `update_frontmatter_fields_batch` with a 1-element slice.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn update_frontmatter_field(
    content: &str,
    field_name: &str,
    new_value: &str,
) -> Result<String, ReconcileError> {
    update_frontmatter_fields_batch(content, &[(field_name, new_value)])
}

/// [DOC-TIER: A] Updates or inserts multiple frontmatter fields in a single pass while preserving line endings.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
/// Contract: Mutates only rows before the first AST H2 section outside `pulldown-cmark` code blocks.
/// Bounds & Faults: Time O(N * U), Space O(N) where N is `content.len()` and U is `updates.len()`.
pub fn update_frontmatter_fields_batch(
    content: &str,
    updates: &[(&str, &str)],
) -> Result<String, ReconcileError> {
    crate::cmark_reconciler::update_frontmatter_fields_via_cmark(content, updates)
}

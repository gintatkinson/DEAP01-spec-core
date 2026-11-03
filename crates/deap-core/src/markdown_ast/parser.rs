//! Deterministic `CommonMark` and GFM AST parser, frontmatter metadata extractor, and tasklist scanner. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides grammar-backed Markdown section, table, code block, tasklist, and frontmatter AST
//! extraction via `pulldown-cmark` and `toml_edit` under DO-178C Level A and DO-330 TQL-4.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Byte offsets (`start_offset`, `end_offset`) correspond to UTF-8 boundaries in `source`.
//! - Multi-line HTML comments (`<!-- ... -->`) outside code blocks are masked with spaces while
//!   preserving exact byte offsets so commented markup never spoofs live AST extraction.
//!
//! ## 3. Preconditions
//! - Input `source` is a valid UTF-8 Markdown slice.
//!
//! ## 4. Postconditions
//! - Returns structured AST nodes for headings/sections, GFM tables, code blocks, task items, and frontmatter.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ single-pass event stream traversal over `source.len()`.
//! - Space Complexity: $O(N)$ proportional to extracted AST text nodes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed Markdown degrades gracefully per `CommonMark` specification; invalid frontmatter
//!   returns `Err(CompilerError::FrontmatterSyntaxError)` fail-closed.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_fm_prs01, fls_err_def01]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::fmt;
use std::ops::Range;

use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, MetadataBlockKind, Options, Parser, Tag, TagEnd,
};

use super::table_ast::{is_br_html_tag, MarkdownTableAst};

/// Structured compilation and projection error type with diagnostic bindings (`BP-04` / `REQ-0604`).
/// Realises: [REQ-0604], [FLS: fls_err_def01]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    /// AST lowering or package validation failure (`E0100`). Realises: [REQ-0604], [FLS: fls_err_def01]
    AstLoweringFailure(String),
    /// Malformed YAML/TOML frontmatter block (`E0400`). Realises: [REQ-0604], [FLS: fls_err_def01]
    FrontmatterSyntaxError(String),
    /// Malformed CommonMark/GFM table structure (`E0200`). Realises: [REQ-0604], [FLS: fls_err_def01]
    TableParseError(String),
    /// Lossless CommonMark emission or atomic write failure (`E0300`). Realises: [REQ-0604], [FLS: fls_err_def01]
    LosslessEmitError(String),
}

impl fmt::Display for CompilerError {
    /// Formats a `CompilerError` with its `BP-04` error code. Realises: [REQ-0604], [FLS: fls_err_def01]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AstLoweringFailure(msg) => write!(f, "AST lowering failure (E0100): {msg}"),
            Self::FrontmatterSyntaxError(msg) => write!(f, "Frontmatter syntax error (E0400): {msg}"),
            Self::TableParseError(msg) => write!(f, "Table parse error (E0200): {msg}"),
            Self::LosslessEmitError(msg) => write!(f, "Lossless emit error (E0300): {msg}"),
        }
    }
}

impl std::error::Error for CompilerError {}

/// Typed frontmatter document parsed via `toml_edit`. Realises: [REQ-0604], [FLS: fls_fm_doc01]
#[derive(Debug, Clone)]
pub struct FrontmatterDoc {
    /// Format-preserving `toml_edit` document. Realises: [REQ-0604], [FLS: fls_fm_doc01]
    pub document: toml_edit::DocumentMut,
    /// Optional raw metadata block string. Realises: [REQ-0604], [FLS: fls_fm_doc01]
    pub raw_yaml: Option<String>,
}

/// Individual checklist task item binding task text to checked state. Realises: [REQ-0604], [FLS: fls_chk_item01]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListItem {
    /// Whether the checkbox is checked (`[x]`). Realises: [REQ-0604], [FLS: fls_chk_item01]
    pub is_checked: bool,
    /// Plain-text content of the task item. Realises: [REQ-0604], [FLS: fls_chk_item01]
    pub text: String,
}

/// Typed `CommonMark` task list checklist document. Realises: [REQ-0604], [FLS: fls_chk_doc01]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChecklistDoc {
    /// Ordered task list items. Realises: [REQ-0604], [FLS: fls_chk_doc01]
    pub tasks: Vec<TaskListItem>,
}

/// Parsed Markdown heading section node with byte span and body slice. Realises: [REQ-0401], [REQ-0604]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownSectionAst {
    /// Heading depth from 1 (`#`) to 6 (`######`). Realises: [REQ-0401]
    pub level: u8,
    /// Normalized plain-text heading title. Realises: [REQ-0401]
    pub heading: String,
    /// Starting byte offset of the heading in source. Realises: [REQ-0401]
    pub start_offset: usize,
    /// Ending byte offset of the section in source. Realises: [REQ-0401]
    pub end_offset: usize,
    /// Trimmed section body text following the heading line. Realises: [REQ-0401]
    pub body: String,
}

/// Parsed Markdown fenced or indented code block AST node. Realises: [REQ-0401], [REQ-0604]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownCodeBlockAst {
    /// Language info string on the code fence. Realises: [REQ-0401]
    pub info: String,
    /// Raw code content inside the block. Realises: [REQ-0401]
    pub code: String,
    /// Starting byte offset of the code block in source. Realises: [REQ-0401]
    pub start_offset: usize,
    /// Ending byte offset of the code block in source. Realises: [REQ-0401]
    pub end_offset: usize,
}

/// Parsed GFM task list item AST node with byte offsets. Realises: [REQ-0604], [FLS: fls_chk_item01]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownTaskItemAst {
    /// Whether the task item is checked. Realises: [REQ-0604]
    pub is_checked: bool,
    /// Plain-text content of the task item. Realises: [REQ-0604]
    pub text: String,
    /// Byte offset of the start of the `Tag::Item` in `source`. Realises: [REQ-0604]
    pub item_start_offset: usize,
    /// Byte offset of the end of the `Tag::Item` in `source`. Realises: [REQ-0604]
    pub item_end_offset: usize,
    /// Byte offset of the `[ ]` / `[x]` marker in `source`. Realises: [REQ-0604]
    pub marker_offset: usize,
}

/// Aggregated Markdown AST containing sections, tables, code blocks, task items, and metadata. Realises: [REQ-0401], [REQ-0604]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedMarkdownAst {
    /// Ordered heading sections extracted from the document. Realises: [REQ-0401]
    pub sections: Vec<MarkdownSectionAst>,
    /// Ordered GFM tables extracted from the document. Realises: [REQ-0401]
    pub tables: Vec<MarkdownTableAst>,
    /// Ordered code blocks extracted from the document. Realises: [REQ-0401]
    pub code_blocks: Vec<MarkdownCodeBlockAst>,
    /// Ordered GFM task list items extracted from the document. Realises: [REQ-0604]
    pub task_items: Vec<MarkdownTaskItemAst>,
    /// Ordered YAML/TOML metadata blocks extracted from the document. Realises: [REQ-0604]
    pub metadata_blocks: Vec<String>,
}

/// Returns the canonical `pulldown-cmark` parser options for DEAP specifications. Realises: [REQ-0604], [REQ-0401]
#[must_use]
pub fn cmark_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_GFM
}

/// Normalizes CRLF and CR line endings to LF (`\n`) across platforms (`BP-04` `SC-01`). Realises: [REQ-0604], [FLS: fls_norm_lf01]
#[must_use]
pub fn normalize_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

/// Collects byte offset ranges of all fenced/indented `Tag::CodeBlock` and inline `Event::Code` spans. Realises: [REQ-0604]
#[must_use]
pub fn collect_code_block_byte_ranges(source: &str) -> Vec<Range<usize>> {
    let (mut ranges, mut active_start) = (Vec::new(), None);
    for (event, range) in Parser::new_ext(source, cmark_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => active_start = Some(range.start),
            Event::End(TagEnd::CodeBlock) => ranges.push(active_start.take().unwrap_or(range.start)..range.end),
            Event::Code(_) => ranges.push(range),
            _ => {}
        }
    }
    if let Some(start) = active_start { ranges.push(start..source.len()); }
    ranges
}

/// Checks whether `byte_offset` falls inside any byte range in `ranges`. Realises: [REQ-0604]
#[must_use]
pub fn is_offset_in_byte_ranges(byte_offset: usize, ranges: &[Range<usize>]) -> bool {
    ranges.iter().any(|r| byte_offset >= r.start && byte_offset < r.end)
}

/// Masks multi-line HTML comments (`<!-- ... -->`) outside code blocks with spaces while preserving byte offsets. Realises: [REQ-0604]
#[must_use]
pub fn mask_html_comments_preserving_offsets(source: &str) -> String {
    if !source.contains("<!--") { return source.to_string(); }
    let code_ranges = collect_code_block_byte_ranges(source);
    let (mut bytes, mut idx, mut in_comment) = (source.as_bytes().to_vec(), 0usize, false);
    while idx < bytes.len() {
        if !in_comment {
            if !is_offset_in_byte_ranges(idx, &code_ranges) && source.get(idx..).is_some_and(|s| s.starts_with("<!--")) {
                in_comment = true;
                for offset in 0..4 { if let Some(b) = bytes.get_mut(idx.saturating_add(offset)) { *b = b' '; } }
                idx = idx.saturating_add(4);
            } else {
                idx = idx.saturating_add(1);
            }
        } else if bytes.get(idx..idx.saturating_add(3)) == Some(b"-->") {
            for offset in 0..3 { if let Some(b) = bytes.get_mut(idx.saturating_add(offset)) { *b = b' '; } }
            in_comment = false;
            idx = idx.saturating_add(3);
        } else {
            if let Some(b) = bytes.get_mut(idx).filter(|b| **b != b'\n' && **b != b'\r') { *b = b' '; }
            idx = idx.saturating_add(1);
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| source.to_string())
}

/// Converts a `pulldown_cmark::HeadingLevel` into its numeric `1..=6` level. Realises: [REQ-0401]
const fn heading_level_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1, HeadingLevel::H2 => 2, HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4, HeadingLevel::H5 => 5, HeadingLevel::H6 => 6,
    }
}

/// Internal state machine accumulating [`ParsedMarkdownAst`] nodes from `pulldown-cmark` events.
/// Realises: [REQ-0604], [REQ-0401], [FLS: fls_fm_prs01]
#[derive(Default)]
struct AstBuilder {
    ast: ParsedMarkdownAst,
    active_sec: Option<(u8, String, usize, usize)>,
    in_heading: Option<(u8, usize, String)>,
    in_table: Option<MarkdownTableAst>,
    cur_row: Vec<String>,
    cur_cell: Option<String>,
    in_code: Option<(String, String, usize)>,
    in_item_range: Option<Range<usize>>,
    in_task: Option<(bool, usize, String)>,
    in_meta: Option<String>,
}

impl AstBuilder {
    /// Processes a single `(Event, Range<usize>)` pair from `pulldown-cmark`. Realises: [REQ-0604]
    fn handle_event(&mut self, source: &str, event: Event<'_>, range: Range<usize>) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                self.flush_section(source, range.start);
                self.in_heading = Some((heading_level_u8(level), range.start, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((lvl, start, title)) = self.in_heading.take() {
                    self.active_sec = Some((lvl, title.trim().to_string(), start, range.end));
                }
            }
            Event::Start(Tag::Table(_)) => {
                self.in_table = Some(MarkdownTableAst { headers: Vec::new(), rows: Vec::new(), start_offset: range.start, end_offset: range.end });
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => self.cur_row.clear(),
            Event::End(TagEnd::TableHead) => if let Some(tbl) = &mut self.in_table { tbl.headers = std::mem::take(&mut self.cur_row); },
            Event::End(TagEnd::TableRow) => if let Some(tbl) = &mut self.in_table { tbl.rows.push(std::mem::take(&mut self.cur_row)); },
            Event::Start(Tag::TableCell) => self.cur_cell = Some(String::new()),
            Event::End(TagEnd::TableCell) => if let Some(cell) = self.cur_cell.take() { self.cur_row.push(cell.trim().to_string()); },
            Event::End(TagEnd::Table) => if let Some(mut tbl) = self.in_table.take() { tbl.end_offset = range.end; self.ast.tables.push(tbl); },
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind { CodeBlockKind::Fenced(lang) => lang.trim().to_string(), CodeBlockKind::Indented => String::new() };
                self.in_code = Some((info, String::new(), range.start));
            }
            Event::End(TagEnd::CodeBlock) => if let Some((info, code, start_offset)) = self.in_code.take() {
                self.ast.code_blocks.push(MarkdownCodeBlockAst { info, code, start_offset, end_offset: range.end });
            },
            Event::Start(Tag::Item) => self.in_item_range = Some(range),
            Event::TaskListMarker(checked) => self.in_task = Some((checked, range.start, String::new())),
            Event::End(TagEnd::Item) => self.flush_task_item(range.end),
            Event::Start(Tag::MetadataBlock(_)) => self.in_meta = Some(String::new()),
            Event::End(TagEnd::MetadataBlock(_)) => if let Some(m) = self.in_meta.take() { self.ast.metadata_blocks.push(m); },
            Event::Text(text) | Event::Code(text) => self.append_text(&text),
            Event::Html(html) | Event::InlineHtml(html) if is_br_html_tag(&html) => if let Some(cell) = &mut self.cur_cell { cell.push(' '); },
            _ => {}
        }
    }

    /// Appends text/code tokens to the active heading, cell, code block, task item, or metadata block. Realises: [REQ-0604]
    fn append_text(&mut self, text: &str) {
        if let Some((_, _, title)) = &mut self.in_heading { title.push_str(text); }
        else if let Some(cell) = &mut self.cur_cell { cell.push_str(text); }
        else if let Some((_, code, _)) = &mut self.in_code { code.push_str(text); }
        else if let Some((_, _, task_txt)) = &mut self.in_task { task_txt.push_str(text); }
        else if let Some(meta) = &mut self.in_meta { meta.push_str(text); }
    }

    /// Commits a completed task list item into `self.ast.task_items`. Realises: [REQ-0604]
    fn flush_task_item(&mut self, end_offset: usize) {
        let item_start = self.in_item_range.take().map_or(0, |r| r.start);
        if let Some((is_checked, marker_offset, text)) = self.in_task.take() {
            self.ast.task_items.push(MarkdownTaskItemAst {
                is_checked, text: text.trim().to_string(), item_start_offset: item_start, item_end_offset: end_offset, marker_offset,
            });
        }
    }

    /// Commits the active heading section into `self.ast.sections`. Realises: [REQ-0401]
    fn flush_section(&mut self, source: &str, next_offset: usize) {
        if let Some((lvl, title, start, body_start)) = self.active_sec.take() {
            let body = source.get(body_start..next_offset).unwrap_or_default().trim().to_string();
            self.ast.sections.push(MarkdownSectionAst { level: lvl, heading: title, start_offset: start, end_offset: next_offset, body });
        }
    }
}

/// Parses a Markdown document into sections, tables, code blocks, task items, and metadata via `pulldown-cmark`.
/// Realises: [REQ-0401], [REQ-0604], [FLS: fls_fm_prs01]
#[must_use]
pub fn parse_markdown_ast(source: &str) -> ParsedMarkdownAst {
    let masked = mask_html_comments_preserving_offsets(source);
    let mut builder = AstBuilder::default();
    for (event, range) in Parser::new_ext(&masked, cmark_options()).into_offset_iter() {
        builder.handle_event(source, event, range);
    }
    builder.flush_section(source, source.len());
    builder.ast
}

/// Parses metadata blocks into a `toml_edit` [`FrontmatterDoc`] without string line slicing.
/// Realises: [REQ-0604], [FLS: fls_fm_prs01]
pub fn parse_frontmatter_metadata(source: &str) -> Result<FrontmatterDoc, CompilerError> {
    let normalized = normalize_line_endings(&mask_html_comments_preserving_offsets(source));
    let (mut in_metadata, mut metadata_content) = (false, String::new());
    for event in Parser::new_ext(&normalized, cmark_options()) {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle | MetadataBlockKind::PlusesStyle)) => in_metadata = true,
            Event::End(TagEnd::MetadataBlock(_)) => break,
            Event::Text(txt) if in_metadata => metadata_content.push_str(&txt),
            _ => {}
        }
    }
    if metadata_content.is_empty() {
        return Ok(FrontmatterDoc { document: toml_edit::DocumentMut::new(), raw_yaml: None });
    }
    if let Ok(parsed_doc) = metadata_content.parse::<toml_edit::DocumentMut>() {
        return Ok(FrontmatterDoc { document: parsed_doc, raw_yaml: Some(metadata_content) });
    }
    let mut doc = toml_edit::DocumentMut::new();
    for entry in metadata_content.split('\n').map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let Some((k, v)) = entry.split_once(':') else {
            return Err(CompilerError::FrontmatterSyntaxError(format!("Invalid frontmatter entry: '{entry}'")));
        };
        let key = k.trim();
        if key.is_empty() {
            return Err(CompilerError::FrontmatterSyntaxError("Empty frontmatter key".to_string()));
        }
        doc.insert(key, toml_edit::value(v.trim().trim_matches(['"', '\''])));
    }
    Ok(FrontmatterDoc { document: doc, raw_yaml: Some(metadata_content) })
}

/// Extracts a typed [`ChecklistDoc`] from `source` via `pulldown-cmark` task list events.
/// Realises: [REQ-0604], [FLS: fls_chk_doc01]
#[must_use]
pub fn parse_checklist_tasks(source: &str) -> ChecklistDoc {
    let tasks = parse_markdown_ast(source)
        .task_items
        .into_iter()
        .map(|item| TaskListItem { is_checked: item.is_checked, text: item.text })
        .collect();
    ChecklistDoc { tasks }
}

//! Deterministic `CommonMark` and GFM AST parser and formatter.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides grammar-backed Markdown section, table, and fenced code block AST extraction
//! and canonical formatting via `pulldown-cmark` and `pulldown-cmark-to-cmark`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Byte offsets (`start_offset`, `end_offset`) correspond to UTF-8 boundaries in `source`.
//! - Parsing is total and deterministic over all valid UTF-8 inputs.
//!
//! ## 3. Preconditions
//! - Input `source` is a valid UTF-8 Markdown slice.
//!
//! ## 4. Postconditions
//! - Returns structured AST nodes for headings/sections, GFM tables, and code blocks.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) single-pass event stream traversal over `source.len()`.
//! - Space Complexity: O(N) proportional to extracted AST text nodes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed Markdown degrades gracefully per `CommonMark` specification without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0401]

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

/// Parsed Markdown heading section node with byte span and body slice.
/// Realises: [REQ-0401]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownSectionAst {
    /// Heading depth from 1 (`#`) to 6 (`######`).
    pub level: u8,
    /// Normalized plain-text heading title.
    pub heading: String,
    /// Starting byte offset of the heading in source.
    pub start_offset: usize,
    /// Ending byte offset of the section in source.
    pub end_offset: usize,
    /// Trimmed section body text following the heading line.
    pub body: String,
}

/// Parsed GitHub Flavored Markdown table AST node.
/// Realises: [REQ-0401]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownTableAst {
    /// Header cell strings in column order.
    pub headers: Vec<String>,
    /// Data rows of cell strings in column order.
    pub rows: Vec<Vec<String>>,
    /// Starting byte offset of the table in source.
    pub start_offset: usize,
    /// Ending byte offset of the table in source.
    pub end_offset: usize,
}

/// Parsed Markdown fenced or indented code block AST node.
/// Realises: [REQ-0401]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownCodeBlockAst {
    /// Language info string on the code fence.
    pub info: String,
    /// Raw code content inside the block.
    pub code: String,
    /// Starting byte offset of the code block in source.
    pub start_offset: usize,
    /// Ending byte offset of the code block in source.
    pub end_offset: usize,
}

/// Aggregated Markdown AST containing sections, tables, and code blocks.
/// Realises: [REQ-0401]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedMarkdownAst {
    /// Ordered heading sections extracted from the document.
    pub sections: Vec<MarkdownSectionAst>,
    /// Ordered GFM tables extracted from the document.
    pub tables: Vec<MarkdownTableAst>,
    /// Ordered code blocks extracted from the document.
    pub code_blocks: Vec<MarkdownCodeBlockAst>,
}

/// Maps a `pulldown_cmark::HeadingLevel` variant to its 1-based numeric heading depth (`1..=6`).
/// Realises: [REQ-0401]
const fn heading_level_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Parses a Markdown document into sections, tables, and code blocks via `pulldown-cmark`.
/// Realises: [REQ-0401]
#[must_use]
pub fn parse_markdown_ast(source: &str) -> ParsedMarkdownAst {
    let mut ast = ParsedMarkdownAst::default();
    let mut active_sec: Option<(u8, String, usize, usize)> = None;
    let mut in_heading: Option<(u8, usize, String)> = None;
    let mut in_table: Option<MarkdownTableAst> = None;
    let mut cur_row: Vec<String> = Vec::new();
    let mut cur_cell: Option<String> = None;
    let mut in_code: Option<(String, String, usize)> = None;

    for (event, range) in Parser::new_ext(source, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if let Some((lvl, title, start, body_start)) = active_sec.take() {
                    let body = source.get(body_start..range.start).unwrap_or_default().trim().to_string();
                    ast.sections.push(MarkdownSectionAst { level: lvl, heading: title, start_offset: start, end_offset: range.start, body });
                }
                in_heading = Some((heading_level_u8(level), range.start, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((lvl, start, title)) = in_heading.take() {
                    active_sec = Some((lvl, title.trim().to_string(), start, range.end));
                }
            }
            Event::Start(Tag::Table(_)) => {
                in_table = Some(MarkdownTableAst { headers: Vec::new(), rows: Vec::new(), start_offset: range.start, end_offset: range.end });
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => cur_row.clear(),
            Event::End(TagEnd::TableHead) => {
                if let Some(tbl) = &mut in_table { tbl.headers = std::mem::take(&mut cur_row); }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(tbl) = &mut in_table { tbl.rows.push(std::mem::take(&mut cur_row)); }
            }
            Event::Start(Tag::TableCell) => cur_cell = Some(String::new()),
            Event::End(TagEnd::TableCell) => {
                if let Some(cell) = cur_cell.take() { cur_row.push(cell.trim().to_string()); }
            }
            Event::End(TagEnd::Table) => {
                if let Some(mut tbl) = in_table.take() { tbl.end_offset = range.end; ast.tables.push(tbl); }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind { CodeBlockKind::Fenced(lang) => lang.trim().to_string(), CodeBlockKind::Indented => String::new() };
                in_code = Some((info, String::new(), range.start));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((info, code, start_offset)) = in_code.take() {
                    ast.code_blocks.push(MarkdownCodeBlockAst { info, code, start_offset, end_offset: range.end });
                }
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, _, title)) = &mut in_heading { title.push_str(&text); }
                else if let Some(cell) = &mut cur_cell { cell.push_str(&text); }
                else if let Some((_, code, _)) = &mut in_code { code.push_str(&text); }
            }
            _ => {}
        }
    }
    if let Some((lvl, title, start, body_start)) = active_sec {
        let body = source.get(body_start..source.len()).unwrap_or_default().trim().to_string();
        ast.sections.push(MarkdownSectionAst { level: lvl, heading: title, start_offset: start, end_offset: source.len(), body });
    }
    ast
}

/// Formats Markdown source into canonical `CommonMark` text using `pulldown-cmark-to-cmark`.
/// Realises: [REQ-0401]
pub fn format_markdown_cmark(source: &str) -> Result<String, String> {
    let options = Options::all();
    let parser = Parser::new_ext(source, options);
    let mut output = String::with_capacity(source.len());
    cmark(parser, &mut output, options).map_err(|err| err.to_string())?;
    Ok(output)
}

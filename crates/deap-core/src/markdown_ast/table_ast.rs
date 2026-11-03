//! Typed `CommonMark` and GFM table AST nodes and `pulldown-cmark` table extractors. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic GitHub Flavored Markdown (GFM) table and frontmatter key-value
//! table extraction via `pulldown-cmark` event streams under DO-178C Level A and DO-330 TQL-4.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every extracted table and cell preserves UTF-8 byte offset boundaries and column ordering.
//! - HTML comment blocks (`<!-- ... -->`) and fenced code blocks are excluded from table extraction.
//!
//! ## 3. Preconditions
//! - Input `source` is a valid UTF-8 Markdown slice.
//!
//! ## 4. Postconditions
//! - Returns strongly-typed [`CmarkTable`], [`MarkdownTableAst`], and [`MarkdownKeyValueRow`] collections.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear event stream traversal over input byte length $N$.
//! - Space Complexity: $O(N)$ proportional to extracted table cell strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed table blocks degrade gracefully per GFM specification without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604], [REQ-0401], [FLS: fls_tbl_node01]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use super::parser::{
    cmark_options, collect_code_block_byte_ranges, is_offset_in_byte_ranges,
    mask_html_comments_preserving_offsets, normalize_line_endings, parse_markdown_ast,
    CompilerError,
};

/// Strongly-typed `CommonMark` table AST node holding header and body cells. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Represents a normalized GFM table extracted via `pulldown-cmark` under `BP-04` (`REQ-0604`).
/// ## 2. Invariants & Mathematical Properties
/// - `headers` and each entry in `rows` preserve left-to-right column order.
/// ## 3. Preconditions
/// - Constructed from a `pulldown-cmark` `Tag::Table` event sequence.
/// ## 4. Postconditions
/// - Holds trimmed plain-text header and body cell strings.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ field access. Space Complexity: $O(R \cdot C)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total value type with zero panicking operations.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [REQ-0401], [FLS: fls_tbl_node01]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CmarkTable {
    /// Header cell strings in column order. Realises: [REQ-0604], [FLS: fls_tbl_node01]
    pub headers: Vec<String>,
    /// Data rows of cell strings in column order. Realises: [REQ-0604], [FLS: fls_tbl_node01]
    pub rows: Vec<Vec<String>>,
}

/// Parsed GitHub Flavored Markdown table AST node with source byte span. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Captures a GFM table together with its exact UTF-8 byte span in the source document.
/// ## 2. Invariants & Mathematical Properties
/// - `start_offset <= end_offset` and both lie on valid UTF-8 character boundaries in `source`.
/// ## 3. Preconditions
/// - Extracted by [`parse_markdown_ast`] from `pulldown-cmark` offset iterators.
/// ## 4. Postconditions
/// - Provides header strings, row cells, and byte span offsets.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ field access. Space Complexity: $O(R \cdot C)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total value type with zero panicking operations.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [REQ-0401], [FLS: fls_tbl_node01]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkdownTableAst {
    /// Header cell strings in column order. Realises: [REQ-0401], [FLS: fls_tbl_node01]
    pub headers: Vec<String>,
    /// Data rows of cell strings in column order. Realises: [REQ-0401], [FLS: fls_tbl_node01]
    pub rows: Vec<Vec<String>>,
    /// Starting byte offset of the table in source. Realises: [REQ-0401], [FLS: fls_tbl_node01]
    pub start_offset: usize,
    /// Ending byte offset of the table in source. Realises: [REQ-0401], [FLS: fls_tbl_node01]
    pub end_offset: usize,
}

/// Parsed 2-column frontmatter key-value row with raw and plain cell text and byte span. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enables AST-backed frontmatter metadata inspection and byte-span splicing without line-splitting.
/// ## 2. Invariants & Mathematical Properties
/// - `row_start_offset <= row_end_offset` marks the exact row slice in `source` (excluding `\r\n` / `\n`).
/// ## 3. Preconditions
/// - Extracted from the document region preceding the first H2 (`## `) section.
/// ## 4. Postconditions
/// - Holds both raw Markdown cell slices (`raw_key`, `raw_value`) and plain text (`plain_key`, `plain_value`).
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ field access. Space Complexity: $O(L)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total value type with zero panicking operations.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [FLS: fls_tbl_prs01]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownKeyValueRow {
    /// Raw Markdown slice of the first (key) cell. Realises: [REQ-0604], [FLS: fls_tbl_prs01]
    pub raw_key: String,
    /// Plain-text content of the first (key) cell. Realises: [REQ-0604], [FLS: fls_tbl_prs01]
    pub plain_key: String,
    /// Raw Markdown slice of the second (value) cell. Realises: [REQ-0604], [FLS: fls_tbl_prs01]
    pub raw_value: String,
    /// Plain-text content of the second (value) cell. Realises: [REQ-0604], [FLS: fls_tbl_prs01]
    pub plain_value: String,
    /// Byte offset of the start of the row in `source`. Realises: [REQ-0604], [FLS: fls_tbl_prs01]
    pub row_start_offset: usize,
    /// Byte offset of the end of the row in `source` (before trailing `\r` or `\n`). Realises: [REQ-0604], [FLS: fls_tbl_prs01]
    pub row_end_offset: usize,
}

/// Checks if an inline HTML event is a `<br>` or `<br/>` line break tag. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Normalizes HTML line breaks inside GFM table cells into single ASCII spaces during AST extraction.
/// ## 2. Invariants & Mathematical Properties
/// - Matches `<br>`, `<br/>`, and `<br />` case-insensitively.
/// ## 3. Preconditions
/// - `html` is a UTF-8 tag slice from `Event::Html` or `Event::InlineHtml`.
/// ## 4. Postconditions
/// - Returns `true` iff `html` represents a line-break tag.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|html|)$. Space Complexity: $O(1)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total non-panicking predicate.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [FLS: fls_tbl_prs01]
#[must_use]
pub fn is_br_html_tag(html: &str) -> bool {
    let t = html.trim();
    t.eq_ignore_ascii_case("<br>")
        || t.eq_ignore_ascii_case("<br/>")
        || t.eq_ignore_ascii_case("<br />")
}

/// Checks whether a cell string is a GFM table alignment delimiter (`---`, `:---`, `:---:`, `---:`). [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Identifies GFM table separator cells so delimiter rows are never mistaken for key-value data rows.
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` iff stripping whitespace and `:` leaves one or more `-` characters.
/// ## 3. Preconditions
/// - `cell` is a valid UTF-8 slice.
/// ## 4. Postconditions
/// - Returns boolean classification.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|cell|)$. Space Complexity: $O(1)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total non-panicking predicate.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [FLS: fls_tbl_prs01]
#[must_use]
pub fn is_gfm_delimiter_cell(cell: &str) -> bool {
    let stripped = cell.trim().trim_matches(':').trim();
    !stripped.is_empty() && stripped.chars().all(|c| c == '-')
}

/// Extracts typed [`CmarkTable`] AST nodes using `pulldown-cmark` table event iterators. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements `BP-04` (`REQ-0604` / `SC-01`) lossless CommonMark table extraction with HTML comment spoofing protection.
/// ## 2. Invariants & Mathematical Properties
/// - Ignores tables embedded inside multi-line HTML comments (`<!-- ... -->`) or fenced code blocks.
/// ## 3. Preconditions
/// - `source` is a valid UTF-8 Markdown document slice.
/// ## 4. Postconditions
/// - Returns `Ok(Vec<CmarkTable>)` in document order.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ single-pass scan over `source.len()`. Space Complexity: $O(N)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Never panics; returns `Result<Vec<CmarkTable>, CompilerError>`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [FLS: fls_tbl_prs01]
///
/// # Errors
/// Returns [`CompilerError`] only if table verification fails.
pub fn parse_markdown_tables(source: &str) -> Result<Vec<CmarkTable>, CompilerError> {
    let masked = mask_html_comments_preserving_offsets(source);
    let normalized = normalize_line_endings(&masked);
    let parser = Parser::new_ext(&normalized, cmark_options());
    let mut tables = Vec::new();
    let mut current_headers = Vec::new();
    let mut current_rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_cell = String::new();
    let mut in_head = false;

    for event in parser {
        match event {
            Event::Start(Tag::TableHead) => in_head = true,
            Event::End(TagEnd::TableHead) => in_head = false,
            Event::Start(Tag::TableRow) => current_row.clear(),
            Event::End(TagEnd::TableRow) => {
                if !in_head {
                    current_rows.push(std::mem::take(&mut current_row));
                }
            }
            Event::End(TagEnd::TableCell) => {
                let cell_text = std::mem::take(&mut current_cell);
                if in_head {
                    current_headers.push(cell_text.trim().to_string());
                } else {
                    current_row.push(cell_text.trim().to_string());
                }
            }
            Event::Text(txt) | Event::Code(txt) => current_cell.push_str(&txt),
            Event::Html(html) | Event::InlineHtml(html) if is_br_html_tag(&html) => {
                current_cell.push(' ');
            }
            Event::End(TagEnd::Table) => {
                tables.push(CmarkTable {
                    headers: std::mem::take(&mut current_headers),
                    rows: std::mem::take(&mut current_rows),
                });
            }
            _ => {}
        }
    }
    Ok(tables)
}

/// Parses a single pipe-delimited row via `pulldown-cmark` into `Vec<(raw_cell, plain_cell)>`. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Uses `pulldown-cmark` `Tag::TableCell` offset ranges to parse a single table row without `.split('|')`.
/// ## 2. Invariants & Mathematical Properties
/// - Escaped pipes (`\|`) and backtick code spans containing pipes are handled by the GFM grammar.
/// ## 3. Preconditions
/// - `row_slice` is a single trimmed table row starting and ending with `|`.
/// ## 4. Postconditions
/// - Returns `Vec<(raw_cell, plain_cell)>` for each cell in the row.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|row\_slice|)$. Space Complexity: $O(|row\_slice|)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns an empty vector if `row_slice` is not a valid GFM table row.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [FLS: fls_tbl_prs01]
#[must_use]
pub fn parse_single_gfm_row_cells(row_slice: &str) -> Vec<(String, String)> {
    let pipe_count = row_slice.bytes().filter(|&b| b == b'|').count();
    let col_guess = pipe_count.saturating_sub(1).max(1);
    let mut synthetic = String::with_capacity(
        row_slice
            .len()
            .saturating_add(col_guess.saturating_mul(6))
            .saturating_add(4),
    );
    synthetic.push_str(row_slice);
    synthetic.push('\n');
    synthetic.push('|');
    for _ in 0..col_guess {
        synthetic.push_str(" --- |");
    }
    synthetic.push('\n');

    let mut cells = Vec::new();
    let mut cur_plain = String::new();
    let mut cell_start: Option<usize> = None;

    for (event, range) in Parser::new_ext(&synthetic, cmark_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::TableCell) => {
                cur_plain.clear();
                cell_start = Some(range.start);
            }
            Event::Text(t) | Event::Code(t) if cell_start.is_some() => {
                cur_plain.push_str(&t);
            }
            Event::Html(h) | Event::InlineHtml(h)
                if cell_start.is_some() && is_br_html_tag(&h) =>
            {
                cur_plain.push(' ');
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(start) = cell_start.take() {
                    let end = range.end.min(row_slice.len());
                    let raw = row_slice.get(start..end).unwrap_or("").trim().to_string();
                    let plain = std::mem::take(&mut cur_plain).trim().to_string();
                    cells.push((raw, plain));
                }
            }
            _ => {}
        }
    }
    cells
}

/// Extracts all 2-column key-value table rows before the first H2 section using `pulldown-cmark`. [DOC-TIER: A]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Parses specification frontmatter tables (both standard GFM tables and headerless 2-column pipe tables)
/// while masking HTML comments and fenced code blocks.
/// ## 2. Invariants & Mathematical Properties
/// - Ignores rows inside `<!-- ... -->` HTML comments and fenced code blocks.
/// - Preserves exact `row_start_offset..row_end_offset` byte spans in `source`.
/// ## 3. Preconditions
/// - `source` is a valid UTF-8 Markdown slice.
/// ## 4. Postconditions
/// - Returns ordered [`MarkdownKeyValueRow`] items preceding the first H2 heading.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over the frontmatter prefix of `source`. Space Complexity: $O(F)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total non-panicking function.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604], [FLS: fls_tbl_prs01]
#[must_use]
pub fn parse_frontmatter_kv_rows(source: &str) -> Vec<MarkdownKeyValueRow> {
    let ast = parse_markdown_ast(source);
    let h2_offset = ast
        .sections
        .iter()
        .find(|s| s.level == 2)
        .map_or(source.len(), |s| s.start_offset);
    let prefix = source.get(..h2_offset).unwrap_or(source);
    let masked = mask_html_comments_preserving_offsets(prefix);
    let code_ranges = collect_code_block_byte_ranges(&masked);

    let mut rows = Vec::new();
    let mut cursor = 0usize;
    while cursor < masked.len() {
        let rest = masked.get(cursor..).unwrap_or("");
        let rel_nl = rest.find('\n').unwrap_or(rest.len());
        let line_end_raw = cursor.saturating_add(rel_nl);
        let row_end = if line_end_raw > cursor
            && source.as_bytes().get(line_end_raw.saturating_sub(1)) == Some(&b'\r')
        {
            line_end_raw.saturating_sub(1)
        } else {
            line_end_raw
        };
        let masked_line = masked.get(cursor..row_end).unwrap_or("").trim();
        if !is_offset_in_byte_ranges(cursor, &code_ranges)
            && masked_line.starts_with('|')
            && masked_line.ends_with('|')
        {
            let orig_line = source.get(cursor..row_end).unwrap_or("");
            let cells = parse_single_gfm_row_cells(orig_line);
            if let (Some((rk, pk)), Some((rv, pv))) = (cells.first(), cells.get(1)) {
                let is_delim = cells.iter().all(|(r, _)| is_gfm_delimiter_cell(r));
                if !is_delim {
                    rows.push(MarkdownKeyValueRow {
                        raw_key: rk.clone(),
                        plain_key: pk.clone(),
                        raw_value: rv.clone(),
                        plain_value: pv.clone(),
                        row_start_offset: cursor,
                        row_end_offset: row_end,
                    });
                }
            }
        }
        cursor = line_end_raw.saturating_add(1);
    }
    rows
}

//! Deterministic `CommonMark` and GFM AST emitter, byte-span splicer, and atomic writer. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides grammar-backed `CommonMark` and GFM AST serialization via `pulldown_cmark_to_cmark::cmark`,
//! reverse-order byte-span splicing, and atomic temp-file-and-rename persistence under
//! DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Serialization of valid `pulldown_cmark::Event` streams is deterministic and total.
//! - Byte-span splicing applies non-overlapping `Range<usize>` edits in descending offset order,
//!   preserving unmodified prefix, infix, and suffix slices identically.
//! - Atomic persistence writes to a sibling `.cmark-tmp` file and renames only after flush completes.
//!
//! ## 3. Preconditions
//! - Input event streams, table structures, and source strings are valid UTF-8.
//! - Byte-span edit ranges lie on valid UTF-8 character boundaries within `source`.
//!
//! ## 4. Postconditions
//! - Emits normalized `CommonMark` strings or persists them atomically to disk without partial writes.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over event stream length or source byte length.
//! - Space Complexity: $O(N)$ proportional to serialized output size.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Formatting, UTF-8 boundary, and filesystem errors return `Err(CompilerError::LosslessEmitError)` fail-closed.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e, fls_qcb1n9c0e5hz]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::fs;
use std::ops::Range;
use std::path::Path;

use pulldown_cmark::{Alignment, Event, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

use crate::markdown_ast::parser::{cmark_options, CompilerError};
use crate::markdown_ast::table_ast::CmarkTable;

/// Serializes a slice of `pulldown_cmark::Event` nodes back into canonical `CommonMark` text.
/// Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub fn emit_lossless_markdown(events: &[Event<'_>]) -> Result<String, CompilerError> {
    let mut buf = String::new();
    cmark(events.iter().cloned(), &mut buf, cmark_options())
        .map_err(|err| CompilerError::LosslessEmitError(err.to_string()))?;
    Ok(buf)
}

/// Formats Markdown source into canonical `CommonMark` text using `pulldown-cmark-to-cmark`.
/// Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub fn format_markdown_cmark(source: &str) -> Result<String, String> {
    let options = cmark_options();
    let parser = Parser::new_ext(source, options);
    let mut output = String::with_capacity(source.len());
    cmark(parser, &mut output, options).map_err(|err| err.to_string())?;
    Ok(output)
}

/// Serializes a structured [`CmarkTable`] into a GFM Markdown table via `pulldown_cmark` events.
/// Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub fn emit_cmark_table(table: &CmarkTable) -> Result<String, CompilerError> {
    let alignments: Vec<Alignment> = vec![Alignment::None; table.headers.len()];
    let mut events: Vec<Event<'_>> = Vec::new();
    events.push(Event::Start(Tag::Table(alignments)));
    events.push(Event::Start(Tag::TableHead));
    for header in &table.headers {
        events.push(Event::Start(Tag::TableCell));
        events.push(Event::Text(header.as_str().into()));
        events.push(Event::End(TagEnd::TableCell));
    }
    events.push(Event::End(TagEnd::TableHead));
    for row in &table.rows {
        events.push(Event::Start(Tag::TableRow));
        for cell in row {
            events.push(Event::Start(Tag::TableCell));
            events.push(Event::Text(cell.as_str().into()));
            events.push(Event::End(TagEnd::TableCell));
        }
        events.push(Event::End(TagEnd::TableRow));
    }
    events.push(Event::End(TagEnd::Table));
    let mut rendered = emit_lossless_markdown(&events)?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

/// Applies non-overlapping byte-span replacements to `source` in descending offset order.
/// Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_qcb1n9c0e5hz]
pub fn splice_markdown_byte_spans(
    source: &str,
    mut edits: Vec<(Range<usize>, String)>,
) -> Result<String, CompilerError> {
    edits.sort_by(|left, right| right.0.start.cmp(&left.0.start));
    let mut prev_start = source.len();
    for (range, _) in &edits {
        if range.start > range.end
            || range.end > prev_start
            || !source.is_char_boundary(range.start)
            || !source.is_char_boundary(range.end)
        {
            return Err(CompilerError::LosslessEmitError(format!(
                "Invalid span range {}..{} for len {}",
                range.start,
                range.end,
                source.len()
            )));
        }
        prev_start = range.start;
    }
    let mut result = source.to_owned();
    for (range, replacement) in edits {
        let prefix = result.get(..range.start).unwrap_or_default();
        let suffix = result.get(range.end..).unwrap_or_default();
        let mut next = String::with_capacity(prefix.len() + replacement.len() + suffix.len());
        next.push_str(prefix);
        next.push_str(&replacement);
        next.push_str(suffix);
        result = next;
    }
    Ok(result)
}

/// Atomically serializes `events` via `emit_lossless_markdown` and writes to `target_path`.
/// Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub fn write_spec_file_atomic(
    target_path: &Path,
    events: &[Event<'_>],
) -> Result<(), CompilerError> {
    let serialized = emit_lossless_markdown(events)?;
    let tmp_path = target_path.with_extension("cmark-tmp");
    fs::write(&tmp_path, serialized.as_bytes())
        .map_err(|err| CompilerError::LosslessEmitError(err.to_string()))?;
    fs::rename(&tmp_path, target_path)
        .map_err(|err| CompilerError::LosslessEmitError(err.to_string()))?;
    Ok(())
}

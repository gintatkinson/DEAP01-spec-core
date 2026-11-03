#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] CommonMark Event Stream, Table, Task-List, and Frontmatter Projection Primitives (`BP-04`).
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-0603], [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Emits and parses CommonMark tables, GFM task-lists, and YAML/TOML frontmatter using
//! `pulldown_cmark::Event`, `pulldown_cmark_to_cmark::cmark`, and `toml_edit::DocumentMut`.
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: Operates in O(N) time and space; fails closed via `CompilerError`.
//! ## 7. Realization & Traceability

use std::collections::BTreeMap;
use std::path::Path;

use pulldown_cmark::{Alignment, CowStr, Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;
use toml_edit::DocumentMut;

use super::{reverse, CompilerError};

/// Parsed CommonMark table extracted via `pulldown_cmark::Parser` (`BP-04` Section 6.2).
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Stores header cell strings, column alignments, and body row vectors.
/// Bounds & Faults: O(R * C) space in row count R and column count C.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CmarkTable {
    pub headers: Vec<String>,
    pub alignments: Vec<Alignment>,
    pub rows: Vec<Vec<String>>,
}

/// Parsed document frontmatter and body byte offset extracted via `pulldown_cmark` and `toml_edit`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Captures sorted key-value metadata and the UTF-8 byte offset where body content begins.
/// Bounds & Faults: O(K) space in metadata key count K.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrontmatterDoc {
    pub metadata: BTreeMap<String, String>,
    pub body_offset: usize,
}

/// Parsed GFM task-list item extracted via `pulldown_cmark::Event::TaskListMarker`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Pairs the boolean checkbox state with trimmed item text.
/// Bounds & Faults: O(L) space in item text byte length L.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListItem {
    pub checked: bool,
    pub text: String,
}

/// Normalizes CRLF (`\r\n`) and standalone CR (`\r`) line endings to canonical LF (`\n`).
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Preserves all non-CR Unicode scalar values identically.
/// Bounds & Faults: O(N) time and space in input byte length N; infallible.
#[must_use]
pub fn normalize_line_endings(raw: &str) -> String {
    raw.replace("\r\n", "\n").replace('\r', "\n")
}

/// Serializes a `pulldown_cmark::Event` iterator into LF-normalized CommonMark via `pulldown_cmark_to_cmark::cmark`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Ensures trailing newline and LF normalization across all serialized CommonMark output.
/// Bounds & Faults: O(E) time and space in event count E; maps formatting errors to `CompilerError::CmarkSerialization`.
pub fn emit_lossless_markdown<'a>(
    events: impl Iterator<Item = Event<'a>>,
) -> Result<String, CompilerError> {
    let mut buf = String::with_capacity(4096);
    let opts = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    cmark(events, &mut buf, opts).map_err(|e| CompilerError::CmarkSerialization(e.to_string()))?;
    let mut normalized = normalize_line_endings(&buf);
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    Ok(normalized)
}

/// Constructs a `pulldown_cmark::Event` stream for a GFM pipe table from headers, alignments, and row cells.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Emits balanced `Tag::Table`, `Tag::TableHead`, `Tag::TableRow`, and `Tag::TableCell` events.
/// Bounds & Faults: O(R * C) time and space over rows R and columns C.
#[must_use]
pub fn cmark_table_events<'a>(
    headers: &'a [&'a str],
    alignments: Vec<Alignment>,
    rows: &'a [Vec<String>],
) -> Vec<Event<'a>> {
    let mut events = Vec::new();
    events.push(Event::Start(Tag::Table(alignments)));
    events.push(Event::Start(Tag::TableHead));
    for &h in headers {
        events.push(Event::Start(Tag::TableCell));
        events.push(Event::Text(CowStr::Borrowed(h)));
        events.push(Event::End(TagEnd::TableCell));
    }
    events.push(Event::End(TagEnd::TableHead));
    for row in rows {
        events.push(Event::Start(Tag::TableRow));
        for cell in row {
            events.push(Event::Start(Tag::TableCell));
            events.push(Event::Text(CowStr::Borrowed(cell.as_str())));
            events.push(Event::End(TagEnd::TableCell));
        }
        events.push(Event::End(TagEnd::TableRow));
    }
    events.push(Event::End(TagEnd::Table));
    events
}

/// Renders a CommonMark pipe table via `cmark_table_events` and `emit_lossless_markdown`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Serializes structured table events through `pulldown_cmark_to_cmark::cmark`.
/// Bounds & Faults: O(R * C) time and space; propagates `CompilerError::CmarkSerialization`.
pub fn render_cmark_table(
    headers: &[&str],
    alignments: Vec<Alignment>,
    rows: &[Vec<String>],
) -> Result<String, CompilerError> {
    let events = cmark_table_events(headers, alignments, rows);
    emit_lossless_markdown(events.into_iter())
}

/// Parses all GFM tables from `markdown` using `pulldown_cmark::Parser::new_ext`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Extracts headers, alignments, and data rows from `Tag::Table` event streams.
/// Bounds & Faults: O(N) time and space in input byte length N; infallible.
#[must_use]
pub fn parse_markdown_tables(markdown: &str) -> Vec<CmarkTable> {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(markdown, opts);
    let mut tables = Vec::new();
    let mut cur_table: Option<CmarkTable> = None;
    let mut cur_row: Vec<String> = Vec::new();
    let mut cur_cell = String::new();
    let mut in_head = false;
    let mut in_cell = false;

    for event in parser {
        match event {
            Event::Start(Tag::Table(aligns)) => {
                cur_table = Some(CmarkTable { headers: Vec::new(), alignments: aligns, rows: Vec::new() });
            }
            Event::End(TagEnd::Table) => {
                if let Some(t) = cur_table.take() {
                    tables.push(t);
                }
            }
            Event::Start(Tag::TableHead) => {
                in_head = true;
                cur_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(ref mut t) = cur_table {
                    t.headers = std::mem::take(&mut cur_row);
                }
                in_head = false;
            }
            Event::Start(Tag::TableRow) => cur_row.clear(),
            Event::End(TagEnd::TableRow) => {
                if !in_head {
                    if let Some(ref mut t) = cur_table {
                        t.rows.push(std::mem::take(&mut cur_row));
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                in_cell = true;
                cur_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                in_cell = false;
                cur_row.push(cur_cell.trim().to_string());
            }
            Event::Text(t) | Event::Code(t) if in_cell => cur_cell.push_str(&t),
            _ => {}
        }
    }
    tables
}

/// Parses GFM task-list items (`- [ ]` / `- [x]`) from `markdown` via `pulldown_cmark::Parser`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Captures `Event::TaskListMarker(checked)` and subsequent inline text per list item.
/// Bounds & Faults: O(N) time and space in input byte length N.
#[must_use]
pub fn parse_task_list_items(markdown: &str) -> Vec<TaskListItem> {
    let parser = Parser::new_ext(markdown, Options::ENABLE_TASKLISTS);
    let mut items = Vec::new();
    let mut active_checked: Option<bool> = None;
    let mut buf = String::new();
    for event in parser {
        match event {
            Event::TaskListMarker(checked) => {
                active_checked = Some(checked);
                buf.clear();
            }
            Event::End(TagEnd::Item) => {
                if let Some(checked) = active_checked.take() {
                    items.push(TaskListItem { checked, text: buf.trim().to_string() });
                    buf.clear();
                }
            }
            Event::Text(t) | Event::Code(t) if active_checked.is_some() => buf.push_str(&t),
            _ => {}
        }
    }
    items
}

/// Parses YAML (`---`) or TOML (`+++`) frontmatter blocks via `pulldown_cmark` and `toml_edit`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Parses `MetadataBlockKind::PlusesStyle` via `toml_edit::DocumentMut` and `YamlStyle` via key-value extraction.
/// Bounds & Faults: O(N) time and space; returns `CompilerError::MalformedFrontmatter` on invalid TOML.
pub fn parse_frontmatter_metadata(markdown: &str) -> Result<FrontmatterDoc, CompilerError> {
    let opts = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    let parser = Parser::new_ext(markdown, opts).into_offset_iter();
    let mut metadata = BTreeMap::new();
    let mut in_meta: Option<MetadataBlockKind> = None;
    let mut raw_meta = String::new();
    let mut body_offset = 0usize;

    for (event, range) in parser {
        match event {
            Event::Start(Tag::MetadataBlock(kind)) if range.start == 0 => {
                in_meta = Some(kind);
                raw_meta.clear();
            }
            Event::End(TagEnd::MetadataBlock(_)) if in_meta.is_some() => {
                let kind = in_meta.take();
                body_offset = range.end;
                if matches!(kind, Some(MetadataBlockKind::PlusesStyle)) {
                    let doc = raw_meta.parse::<DocumentMut>().map_err(|e| CompilerError::MalformedFrontmatter {
                        detail: e.to_string(),
                    })?;
                    for (k, item) in doc.iter() {
                        if let Some(val) = item.as_value() {
                            let clean = val.as_str().map_or_else(|| val.to_string().trim().to_string(), str::to_string);
                            metadata.insert(k.to_string(), clean);
                        }
                    }
                } else {
                    let wrapped = format!("---\n{raw_meta}---\n");
                    let (fm, _) = reverse::parse_frontmatter(&wrapped);
                    metadata.extend(fm);
                }
                break;
            }
            Event::Text(t) if in_meta.is_some() => raw_meta.push_str(&t),
            _ if in_meta.is_none() => break,
            _ => {}
        }
    }
    Ok(FrontmatterDoc { metadata, body_offset })
}

/// Persists a specification artifact atomically with LF normalization when `dry_run` is `false`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Performs zero disk writes when `dry_run == true`; delegates to `crate::salsa_db::write_atomic` otherwise.
/// Bounds & Faults: O(N) I/O cost; maps filesystem errors to `CompilerError::AtomicWriteError`.
pub fn write_spec_file_atomic(
    target_path: &Path,
    content: &str,
    dry_run: bool,
) -> Result<(), CompilerError> {
    if dry_run {
        return Ok(());
    }
    let normalized = normalize_line_endings(content);
    crate::salsa_db::write_atomic(target_path, &normalized).map_err(|detail| CompilerError::AtomicWriteError {
        path: target_path.to_path_buf(),
        detail: detail.to_string(),
    })
}

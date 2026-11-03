//! Grammar-backed `CommonMark` and GFM AST backlog reconciler (`BP-04` / `REQ-0604`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic `pulldown-cmark`, `pulldown-cmark-to-cmark`, and `deap_core::markdown_ast`
//! frontmatter metadata extraction, byte-span table field mutation, and tasklist checkbox
//! reconciliation for `reconcile-backlog` under DO-178C Level A and DO-330 TQL-4.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Frontmatter key-value inspection and updates operate over `deap_core::markdown_ast`
//!   AST byte spans and `splice_markdown_byte_spans` without line-splitting loops.
//! - Checklist reconciliation operates over `MarkdownTaskItemAst` (`Event::TaskListMarker`)
//!   byte spans outside fenced code blocks and HTML comments.
//!
//! ## 3. Preconditions
//! - Input `content` is a valid UTF-8 Markdown specification slice.
//!
//! ## 4. Postconditions
//! - Produces reconciled Markdown strings and `ReconcileReceipt` records preserving document formatting.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \cdot U + K \log K)$ over document length $N$, updates $U$, and edits $K$.
//! - Space Complexity: $O(N)$ proportional to document byte length.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed frontmatter, arithmetic overflow, or span splicing faults fail closed via `Result`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604], [BP-04], [REQ-RECONCILE-BACKLOG-CORE] [FLS: fls_t4w1da759n2e, fls_qcb1n9c0e5hz]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::ops::Range;

use deap_core::markdown_ast::pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use deap_core::markdown_ast::pulldown_cmark_to_cmark::cmark;
use deap_core::markdown_ast::{
    cmark_options, emit_lossless_markdown, parse_checklist_tasks, parse_frontmatter_kv_rows,
    parse_frontmatter_metadata, parse_markdown_ast, splice_markdown_byte_spans, CompilerError,
};

use crate::checklist::{
    clean_unpopulated_placeholders_counted, extract_checklist_title_and_issue,
    replace_stale_issue_in_checklist_line, update_checkbox_state_on_line,
};
use crate::frontmatter::{
    apply_frontmatter_cell_pair, clean_markdown_key, extract_h1_title, Frontmatter,
};
use crate::tracker::IssueIndex;
use crate::ReconcileError;

/// Receipt summarizing AST-backed CommonMark backlog reconciliation (`BP-04` `01.04.05`).
/// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileReceipt {
    /// Document identifier extracted from frontmatter metadata (`"UNBOUND"` if absent). Realises: [REQ-0604]
    pub document_id: String,
    /// Count of checked (`[x]`) GFM task list items. Realises: [REQ-0604]
    pub completed_tasks: usize,
    /// Total count of GFM task list items in the specification. Realises: [REQ-0604]
    pub total_tasks: usize,
    /// Canonical `CommonMark` serialization emitted via `pulldown-cmark-to-cmark`. Realises: [REQ-0604]
    pub serialized_markdown: String,
}

/// Reconciles a Markdown backlog specification via `pulldown-cmark` and `pulldown-cmark-to-cmark`.
/// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub fn reconcile_markdown_backlog(source: &str) -> Result<ReconcileReceipt, CompilerError> {
    let fm_doc = parse_frontmatter_metadata(source)?;
    let document_id = fm_doc
        .document
        .get("id")
        .and_then(|v| v.as_str())
        .map_or_else(|| "UNBOUND".to_string(), str::to_string);
    let checklist = parse_checklist_tasks(source);
    let completed_tasks = checklist.tasks.iter().filter(|t| t.is_checked).count();
    let total_tasks = checklist.tasks.len();
    let events: Vec<Event<'_>> = Parser::new_ext(source, cmark_options()).collect();
    let serialized_markdown = emit_lossless_markdown(&events)?;
    Ok(ReconcileReceipt {
        document_id,
        completed_tasks,
        total_tasks,
        serialized_markdown,
    })
}

/// Computes total Epic cardinality including the mandatory `EPIC-00` architecture foundation (`BP-04` `AC-03`).
/// Realises: [REQ-0604], [BP-04] [FLS: fls_qcb1n9c0e5hz]
pub fn count_total_epics_inclusive(explicit_epics: usize) -> Result<usize, CompilerError> {
    explicit_epics.checked_add(1).ok_or_else(|| {
        CompilerError::AstLoweringFailure("Epic cardinality arithmetic overflow".to_string())
    })
}

/// Emits a `CommonMark` checklist section from typed `pulldown_cmark::Event` nodes via `cmark`.
/// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub fn emit_checklist_section_cmark(
    heading: &str,
    items: &[(bool, String)],
) -> Result<String, ReconcileError> {
    let mut events: Vec<Event<'_>> = Vec::new();
    events.push(Event::Start(Tag::Heading {
        level: HeadingLevel::H2,
        id: None,
        classes: Vec::new(),
        attrs: Vec::new(),
    }));
    events.push(Event::Text(heading.into()));
    events.push(Event::End(TagEnd::Heading(HeadingLevel::H2)));
    events.push(Event::Start(Tag::List(None)));
    for (checked, text) in items {
        events.push(Event::Start(Tag::Item));
        events.push(Event::TaskListMarker(*checked));
        events.push(Event::Text(text.as_str().into()));
        events.push(Event::End(TagEnd::Item));
    }
    events.push(Event::End(TagEnd::List(false)));
    let mut out = String::new();
    cmark(events.into_iter(), &mut out, cmark_options())
        .map_err(|e| ReconcileError::ParseError(e.to_string()))?;
    Ok(out)
}

/// Parses specification frontmatter metadata via `deap_core::markdown_ast::parse_frontmatter_kv_rows`.
/// Realises: [REQ-0604], [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER] [FLS: fls_t4w1da759n2e]
pub fn parse_frontmatter_via_cmark(content: &str) -> Result<Frontmatter, ReconcileError> {
    let mut fm = Frontmatter::default();
    for row in parse_frontmatter_kv_rows(content) {
        apply_frontmatter_cell_pair(&mut fm, &row.raw_key, &row.raw_value);
    }
    if fm.title.is_none() {
        if let Some(h1) = extract_h1_title(content) {
            fm.title = Some(h1);
        }
    }
    Ok(fm)
}

/// Checks whether two normalized frontmatter table keys refer to the same logical field (including issue aliases).
/// Realises: [REQ-0604], [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
#[must_use]
pub fn frontmatter_keys_match_cmark(raw_key: &str, target_key: &str) -> bool {
    let target = target_key.trim().to_lowercase();
    if raw_key == target {
        return true;
    }
    let is_issue_alias = |k: &str| {
        matches!(
            k,
            "github issue" | "issue" | "tracker issue" | "issue id" | "github"
        )
    };
    is_issue_alias(raw_key) && is_issue_alias(&target)
}

/// Updates or inserts multiple frontmatter fields via `deap_core::markdown_ast` byte-span splicing.
/// Realises: [REQ-0604], [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH] [FLS: fls_qcb1n9c0e5hz]
pub fn update_frontmatter_fields_via_cmark(
    content: &str,
    updates: &[(&str, &str)],
) -> Result<String, ReconcileError> {
    if updates.is_empty() {
        return Ok(content.to_string());
    }
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let kv_rows = parse_frontmatter_kv_rows(content);
    let mut updated_flags = vec![false; updates.len()];
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();

    for row in &kv_rows {
        let raw_key = clean_markdown_key(&row.raw_key);
        for ((field_name, new_value), flag) in updates.iter().zip(updated_flags.iter_mut()) {
            if !*flag && frontmatter_keys_match_cmark(&raw_key, field_name) {
                let key_cell = row.raw_key.trim();
                let display_key = if key_cell.starts_with("**") && key_cell.ends_with("**") {
                    key_cell.to_string()
                } else {
                    format!("**{}**", field_name.trim())
                };
                edits.push((
                    row.row_start_offset..row.row_end_offset,
                    format!("| {display_key} | {new_value} |"),
                ));
                *flag = true;
                break;
            }
        }
    }

    let remaining: Vec<(&str, &str)> = updates
        .iter()
        .zip(&updated_flags)
        .filter(|(_, flag)| !**flag)
        .map(|(pair, _)| *pair)
        .collect();

    if !remaining.is_empty() {
        append_missing_frontmatter_rows_cmark(
            content,
            &remaining,
            kv_rows.last().map(|r| r.row_end_offset),
            line_ending,
            &mut edits,
        );
    }

    splice_markdown_byte_spans(content, edits)
        .map_err(|e| ReconcileError::ParseError(e.to_string()))
}

/// Appends new frontmatter rows at the end of the frontmatter table or after H1 (`[REQ-0604]`, `[FLS: fls_qcb1n9c0e5hz]`).
fn append_missing_frontmatter_rows_cmark(
    content: &str,
    remaining: &[(&str, &str)],
    last_kv_row_end: Option<usize>,
    line_ending: &str,
    edits: &mut Vec<(Range<usize>, String)>,
) {
    let ast = parse_markdown_ast(content);
    let h2_offset = ast
        .sections
        .iter()
        .find(|s| s.level == 2)
        .map_or(content.len(), |s| s.start_offset);
    let tbl_end = ast
        .tables
        .iter()
        .filter(|t| t.start_offset < h2_offset)
        .map(|t| {
            content
                .get(..t.end_offset)
                .unwrap_or("")
                .trim_end_matches(['\r', '\n'])
                .len()
        })
        .max();
    let last_table_end = last_kv_row_end.into_iter().chain(tbl_end).max();
    let new_rows = remaining
        .iter()
        .map(|(f, v)| format!("| **{}** | {} |", f.trim(), v))
        .collect::<Vec<_>>()
        .join(line_ending);

    if let Some(end_pos) = last_table_end {
        edits.push((end_pos..end_pos, format!("{line_ending}{new_rows}")));
    } else if let Some(h1_sec) = ast
        .sections
        .iter()
        .find(|s| s.level == 1 && s.start_offset < h2_offset)
    {
        let h1_line_end = content
            .get(h1_sec.start_offset..)
            .and_then(|s| s.find('\n'))
            .map_or(content.len(), |nl| {
                let raw_end = h1_sec.start_offset.saturating_add(nl);
                if raw_end > h1_sec.start_offset
                    && content.as_bytes().get(raw_end.saturating_sub(1)) == Some(&b'\r')
                {
                    raw_end.saturating_sub(1)
                } else {
                    raw_end
                }
            });
        let block = format!(
            "{line_ending}{line_ending}| Field | Value |{line_ending}|---|---|{line_ending}{new_rows}"
        );
        edits.push((h1_line_end..h1_line_end, block));
    } else {
        let block = format!(
            "| Field | Value |{line_ending}|---|---|{line_ending}{new_rows}{line_ending}{line_ending}"
        );
        edits.push((0..0, block));
    }
}

/// Reconciles checklist items in `content` via `deap_core::markdown_ast` task item spans and `splice_markdown_byte_spans`.
/// Realises: [REQ-0604], [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-CHECKLISTS-IN-CONTENT] [FLS: fls_qcb1n9c0e5hz]
pub fn reconcile_checklists_via_cmark(
    content: &str,
    index: &IssueIndex,
) -> Result<(String, usize), ReconcileError> {
    let (cleaned, _purged_count) = clean_unpopulated_placeholders_counted(content);
    let ast = parse_markdown_ast(&cleaned);
    let mut updated_count = 0usize;
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();

    for task in &ast.task_items {
        let line_start = cleaned
            .get(..task.marker_offset)
            .and_then(|s| s.rfind('\n'))
            .map_or(0, |i| i.saturating_add(1));
        let line_end = cleaned
            .get(task.marker_offset..)
            .and_then(|s| s.find(['\r', '\n']))
            .map_or(cleaned.len(), |i| task.marker_offset.saturating_add(i));
        let line = cleaned.get(line_start..line_end).unwrap_or("");

        if let Some((title, issue_num)) = extract_checklist_title_and_issue(line) {
            let mut effective_line = line.to_string();
            let mut line_changed = false;
            let effective_issue = match issue_num {
                Some(num) if index.is_issue_valid_for_spec(num, &title, None) => Some(num),
                Some(stale_num) => {
                    let found = index.find_issue_for_spec(&title, None);
                    if let Some(new_num) = found {
                        if let Some(repatched) = replace_stale_issue_in_checklist_line(
                            &effective_line,
                            stale_num,
                            new_num,
                        ) {
                            effective_line = repatched;
                            line_changed = true;
                        }
                    }
                    found
                }
                None => index.find_issue_for_spec(&title, None),
            };

            if let Some(should_be_checked) =
                effective_issue.and_then(|num| index.is_issue_resolved(num))
            {
                if let Some(rewritten) =
                    update_checkbox_state_on_line(&effective_line, should_be_checked)
                {
                    effective_line = rewritten;
                    line_changed = true;
                }
            }
            if line_changed {
                edits.push((line_start..line_end, effective_line));
                updated_count = updated_count.saturating_add(1);
            }
        }
    }

    if edits.is_empty() {
        return Ok((cleaned, 0));
    }
    let spliced = splice_markdown_byte_spans(&cleaned, edits)
        .map_err(|e| ReconcileError::ParseError(e.to_string()))?;
    Ok((spliced, updated_count))
}

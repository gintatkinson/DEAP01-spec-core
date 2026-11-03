//! Grammar-backed `CommonMark` and GFM AST ingestion engine (`BP-04` / `REQ-0604`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic `pulldown-cmark` and `deap_core::markdown_ast` frontmatter, section,
//! GFM table, parametric constraint, and SysML v2 metamodel extraction for `ingest-sysml` under
//! DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - All heading, table, code block, and metadata spans are derived from `pulldown-cmark`
//!   offset iterators without ad-hoc line-by-line string splitting.
//! - Multi-line HTML comments (`<!-- ... -->`) and fenced code blocks are masked/excluded
//!   so commented or example tables never spoof live SysML v2 AST nodes.
//!
//! ## 3. Preconditions
//! - Input `content` is a valid UTF-8 Markdown specification slice.
//!
//! ## 4. Postconditions
//! - Produces deterministic [`Frontmatter`], [`ParsedSection`], [`MarkdownTable`], and [`PackageDef`] models.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ single-pass event stream traversal over `content.len()`.
//! - Space Complexity: $O(N)$ proportional to extracted section and table AST nodes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed Markdown or formatting faults propagate explicit `Result` diagnostics without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0021], [REQ-0032], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e, fls_tbl_prs01]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::ops::Range;

use deap_core::markdown_ast::pulldown_cmark::{
    Event, HeadingLevel, MetadataBlockKind, Parser, Tag, TagEnd,
};
use deap_core::markdown_ast::pulldown_cmark_to_cmark::cmark;
use deap_core::markdown_ast::{
    cmark_options, is_offset_in_byte_ranges, mask_html_comments_preserving_offsets,
    parse_frontmatter_kv_rows, parse_frontmatter_metadata, parse_markdown_ast,
};
use deap_core::{ConstraintDef, PackageDef};

use crate::cst::md_common::{
    clean_meta_val, find_req_ids, parse_comp_heading, strip_pkg_words, Frontmatter, ParsedSection,
};
use crate::table::{
    clean_cell_value, extract_unit_from_header_or_val, normalize_col, sanitize_identifier,
    MarkdownTable,
};

/// `CommonMark` GFM table and section AST extraction submodule (`BP-04` `01.04.04`).
/// Realises: [REQ-0604], [BP-04] [FLS: fls_tbl_prs01]
pub mod tables {
    use super::{
        clean_cell_value, cmark_options, extract_unit_from_header_or_val,
        is_offset_in_byte_ranges, mask_html_comments_preserving_offsets, normalize_col,
        parse_comp_heading, sanitize_identifier, strip_pkg_words, Event, Frontmatter,
        HeadingLevel, MarkdownTable, ParsedSection, Parser, Range, Tag, TagEnd,
    };

    /// Converts a `HeadingLevel` into its numeric `1..=6` depth. Realises: [REQ-0604]
    const fn level_u8(level: HeadingLevel) -> u8 {
        match level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }

    /// Initializes a [`MarkdownTable`] from raw header cell slices. Realises: [REQ-0604]
    #[must_use]
    pub fn build_markdown_table_header(raw_headers: Vec<String>) -> MarkdownTable {
        MarkdownTable {
            normalized_headers: raw_headers.iter().map(|c| normalize_col(c)).collect(),
            units: raw_headers.iter().map(|c| extract_unit_from_header_or_val(c).1).collect(),
            raw_headers,
            rows: Vec::new(),
        }
    }

    /// Extracts normalized section prose from `masked[span]` excluding `excluded` table/code byte spans.
    /// Realises: [REQ-0604] [FLS: fls_tbl_prs01]
    #[must_use]
    pub fn extract_cmark_section_prose(
        masked: &str,
        span: Range<usize>,
        excluded: &[Range<usize>],
    ) -> String {
        let (mut cursor, mut words) = (span.start, Vec::new());
        for ex in excluded {
            if ex.end <= cursor { continue; }
            if ex.start >= span.end { break; }
            let seg_end = ex.start.min(span.end);
            if cursor < seg_end {
                if let Some(seg) = masked.get(cursor..seg_end) {
                    words.extend(seg.split_whitespace());
                }
            }
            cursor = cursor.max(ex.end.min(span.end));
        }
        if cursor < span.end {
            if let Some(seg) = masked.get(cursor..span.end) {
                words.extend(seg.split_whitespace());
            }
        }
        words.join(" ").replace("*/", "* /").replace("/*", "/ *")
    }

    /// Internal event-driven builder for [`parse_cmark_markdown_structure`]. Realises: [REQ-0604]
    #[derive(Default)]
    struct StructurePass {
        headings: Vec<(u8, String, Range<usize>)>,
        tables: Vec<(Range<usize>, MarkdownTable)>,
        excluded_spans: Vec<Range<usize>>,
        code_spans: Vec<Range<usize>>,
        cur_table: Option<(usize, Vec<String>, Vec<Vec<String>>)>,
        cur_row: Vec<String>,
        cell_start: Option<usize>,
        in_head: bool,
        code_start: Option<usize>,
    }

    impl StructurePass {
        /// Processes a single `(Event, Range<usize>)` from `pulldown-cmark`. Realises: [REQ-0604]
        fn on_event(&mut self, content: &str, event: Event<'_>, range: Range<usize>) {
            match event {
                Event::Start(Tag::CodeBlock(_)) => self.code_start = Some(range.start),
                Event::End(TagEnd::CodeBlock) => {
                    let start = self.code_start.take().unwrap_or(range.start);
                    self.code_spans.push(start..range.end);
                    self.excluded_spans.push(start..range.end);
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    let lvl = level_u8(level);
                    if (1..=4).contains(&lvl) && !is_offset_in_byte_ranges(range.start, &self.code_spans) {
                        let raw_slice = content.get(range.clone()).unwrap_or("").trim();
                        let raw_hdr = raw_slice.trim_start_matches('#').trim().to_string();
                        self.headings.push((lvl, raw_hdr, range));
                    }
                }
                Event::Start(Tag::Table(_)) => self.cur_table = Some((range.start, Vec::new(), Vec::new())),
                Event::Start(Tag::TableHead) => { self.in_head = true; self.cur_row.clear(); }
                Event::End(TagEnd::TableHead) => {
                    self.in_head = false;
                    if let Some((_, ref mut hdrs, _)) = self.cur_table { *hdrs = std::mem::take(&mut self.cur_row); }
                }
                Event::Start(Tag::TableRow) => self.cur_row.clear(),
                Event::End(TagEnd::TableRow) => {
                    if let Some((_, _, ref mut rows)) = self.cur_table { rows.push(std::mem::take(&mut self.cur_row)); }
                }
                Event::Start(Tag::TableCell) => self.cell_start = Some(range.start),
                Event::End(TagEnd::TableCell) => if let Some(start) = self.cell_start.take() {
                    let end = range.end.min(content.len());
                    let raw = content.get(start..end).unwrap_or("").trim().trim_matches('|').trim().to_string();
                    self.cur_row.push(raw);
                },
                Event::End(TagEnd::Table) => self.finish_table(range.end),
                _ => {}
            }
        }

        /// Finalizes a parsed GFM table and records its byte span. Realises: [REQ-0604]
        fn finish_table(&mut self, end_offset: usize) {
            if let Some((start_offset, raw_hdrs, raw_rows)) = self.cur_table.take() {
                self.excluded_spans.push(start_offset..end_offset);
                let mut tbl = build_markdown_table_header(raw_hdrs);
                for cells in raw_rows {
                    let row = tbl
                        .normalized_headers
                        .iter()
                        .enumerate()
                        .map(|(idx, col)| (col.clone(), cells.get(idx).map_or_else(String::new, |c| clean_cell_value(c))))
                        .collect();
                    tbl.rows.push(row);
                }
                if !tbl.raw_headers.is_empty() && !tbl.rows.is_empty() {
                    self.tables.push((start_offset..end_offset, tbl));
                }
            }
        }
    }

    /// Decomposes Markdown headings, prose, and GFM tables via `pulldown-cmark` AST events.
    /// Realises: [REQ-0604], [BP-04] [FLS: fls_tbl_prs01]
    #[must_use]
    pub fn parse_cmark_markdown_structure(
        content: &str,
        default_name: &str,
        frontmatter: &Frontmatter,
        resolve_h2_to_h4: impl Fn(&str, usize, bool, Option<String>) -> Option<String>,
    ) -> (String, Vec<ParsedSection>) {
        let masked = mask_html_comments_preserving_offsets(content);
        let mut pass = StructurePass::default();
        for (event, range) in Parser::new_ext(&masked, cmark_options()).into_offset_iter() {
            pass.on_event(content, event, range);
        }
        pass.excluded_spans.sort_by_key(|r| r.start);

        let mut pkg_name = frontmatter.title.as_deref().map_or_else(
            || sanitize_identifier(default_name, "Markdown_Package"),
            |t| sanitize_identifier(t, default_name),
        );
        let explicit_comps = pass.headings.iter().any(|(lvl, hdr, _)| (2..=4).contains(lvl) && parse_comp_heading(hdr).0);
        let mut sections = Vec::with_capacity(pass.headings.len().saturating_add(1));
        let first_h_start = pass.headings.first().map_or(content.len(), |(_, _, r)| r.start);
        if first_h_start > 0 {
            let pre_doc = extract_cmark_section_prose(&masked, 0..first_h_start, &pass.excluded_spans);
            let pre_tables: Vec<MarkdownTable> = pass.tables.iter().filter(|(r, _)| r.start < first_h_start).map(|(_, t)| t.clone()).collect();
            if !pre_doc.is_empty() || !pre_tables.is_empty() {
                sections.push(ParsedSection { component_target: None, section_doc: pre_doc, tables: pre_tables });
            }
        }

        let mut comp_target: Option<String> = None;
        for (idx, (lvl, raw_hdr, h_range)) in pass.headings.iter().enumerate() {
            if *lvl == 1 {
                let clean = strip_pkg_words(raw_hdr);
                let cand = if clean.is_empty() { raw_hdr.as_str() } else { &clean };
                if !pkg_name.starts_with("BP_") {
                    pkg_name = sanitize_identifier(cand, &pkg_name);
                }
                comp_target = None;
            } else {
                comp_target = resolve_h2_to_h4(raw_hdr, usize::from(*lvl), explicit_comps, comp_target.take());
            }
            let body_end = pass.headings.get(idx.saturating_add(1)).map_or(content.len(), |(_, _, next_r)| next_r.start);
            let body_span = h_range.end.min(body_end)..body_end;
            let sec_doc = extract_cmark_section_prose(&masked, body_span.clone(), &pass.excluded_spans);
            let sec_tables: Vec<MarkdownTable> = pass
                .tables
                .iter()
                .filter(|(r, _)| r.start >= body_span.start && r.start < body_span.end)
                .map(|(_, t)| t.clone())
                .collect();
            if !raw_hdr.is_empty() || !sec_tables.is_empty() || !sec_doc.is_empty() {
                sections.push(ParsedSection { component_target: comp_target.clone(), section_doc: sec_doc, tables: sec_tables });
            }
        }
        (pkg_name, sections)
    }
}

/// `CommonMark` parametric constraint AST extraction submodule (`BP-04` `01.04.04`).
/// Realises: [REQ-0604], [BP-04] [FLS: fls_tbl_prs01]
pub mod constraints {
    use super::{ConstraintDef, PackageDef};
    use crate::cst::md_requirements::extract_formal_invariants;

    /// Extracts parametric [`ConstraintDef`] nodes from a Markdown specification via `cmark` AST.
    /// Realises: [REQ-0604], [BP-04] [FLS: fls_tbl_prs01]
    #[must_use]
    pub fn extract_cmark_constraints(content: &str, req_num: &str) -> Vec<ConstraintDef> {
        extract_formal_invariants(content, req_num)
    }

    /// Merges extracted [`ConstraintDef`] nodes into a target [`PackageDef`] without duplicates.
    /// Realises: [REQ-0604], [BP-04] [FLS: fls_tbl_prs01]
    pub fn attach_cmark_constraints(pkg: &mut PackageDef, constraints: Vec<ConstraintDef>) {
        for c in constraints {
            if !pkg.constraint_defs.iter().any(|existing| existing.name == c.name) {
                pkg.constraint_defs.push(c);
            }
        }
    }
}

/// `CommonMark` frontmatter and SysML v2 metamodel lowering submodule (`BP-04` `01.04.04`).
/// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
pub mod metamodel {
    use super::{
        clean_meta_val, cmark, cmark_options, find_req_ids, normalize_col,
        parse_frontmatter_kv_rows, parse_frontmatter_metadata, parse_markdown_ast, Event,
        Frontmatter, MetadataBlockKind, PackageDef, Parser, Tag, TagEnd,
    };

    /// Parses YAML frontmatter (`Tag::MetadataBlock`) or top-level 2-column metadata table via `deap_core::markdown_ast`.
    /// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
    #[must_use]
    pub fn parse_cmark_frontmatter(content: &str) -> (Frontmatter, &str) {
        let (mut in_meta, mut meta_end) = (false, None);
        for (event, range) in Parser::new_ext(content, cmark_options()).into_offset_iter() {
            match event {
                Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle | MetadataBlockKind::PlusesStyle)) => in_meta = true,
                Event::Text(_) if in_meta => {}
                Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle | MetadataBlockKind::PlusesStyle)) if in_meta => {
                    meta_end = Some(range.end);
                    break;
                }
                _ => break,
            }
        }
        if let Some(end_idx) = meta_end {
            if let Ok(fm_doc) = parse_frontmatter_metadata(content) {
                let get_str = |key: &str| fm_doc.document.get(key).and_then(|item| item.as_str()).map(|s| s.trim().to_string());
                let fm = Frontmatter {
                    id: get_str("id"), title: get_str("title"), subsystem: get_str("subsystem"), uuidv5: get_str("uuidv5"),
                };
                return (fm, content.get(end_idx..).unwrap_or("").trim_start_matches(['\r', '\n']));
            }
        }

        let ast = parse_markdown_ast(content);
        let h2_offset = ast.sections.iter().find(|s| s.level == 2).map_or(content.len(), |s| s.start_offset);
        let h1_title = ast.sections.iter().find(|s| s.level == 1 && s.start_offset < h2_offset).and_then(|s| {
            let raw_line = content.get(s.start_offset..s.end_offset).unwrap_or("").split('\n').next().unwrap_or("");
            let t = raw_line.trim().trim_start_matches('#').trim();
            (!t.is_empty()).then(|| t.to_string())
        });

        let (mut fm, mut blueprint_id) = (Frontmatter::default(), None);
        for row in parse_frontmatter_kv_rows(content) {
            let key = normalize_col(&row.raw_key);
            let val = clean_meta_val(&row.raw_value).trim().trim_matches(['"', '\'']).trim().to_string();
            if val.is_empty() { continue; }
            match key.as_str() {
                "requirement_id" | "id" if fm.id.is_none() => {
                    fm.id = Some(find_req_ids(&val).first().copied().unwrap_or(val.as_str()).to_string());
                }
                "blueprint_id" if blueprint_id.is_none() => blueprint_id = Some(val),
                "title" if fm.title.is_none() => fm.title = Some(val),
                "subsystem" if fm.subsystem.is_none() => fm.subsystem = Some(val),
                "uuidv5" | "deterministic_uuidv5" if fm.uuidv5.is_none() => fm.uuidv5 = Some(val),
                _ => {}
            }
        }
        if fm.id.is_none() { fm.id = blueprint_id.clone(); }
        if let Some(bp_id) = blueprint_id {
            let raw_title = fm.title.take().or(h1_title);
            fm.title = Some(match raw_title {
                Some(t) if !t.is_empty() && !t.starts_with(&bp_id) => format!("{bp_id} {t}"),
                Some(t) if !t.is_empty() => t,
                _ => bp_id,
            });
        } else if fm.id.is_some() && fm.title.is_none() {
            fm.title = h1_title;
        }
        (fm, content)
    }

    /// Normalizes a Markdown specification document via `pulldown_cmark_to_cmark::cmark`.
    /// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
    pub fn normalize_ingested_markdown_cmark(content: &str) -> Result<String, String> {
        let opts = cmark_options();
        let parser = Parser::new_ext(content, opts);
        let mut buf = String::with_capacity(content.len());
        cmark(parser, &mut buf, opts).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    /// Translates a Markdown specification into a SysML v2 [`PackageDef`] via `MarkdownTranslator`.
    /// Realises: [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e]
    pub fn ingest_markdown_via_cmark(content: &str, file_name: &str) -> Result<PackageDef, String> {
        crate::cst::markdown_cst::MarkdownTranslator::new().translate(content, file_name)
    }
}

pub use constraints::{attach_cmark_constraints, extract_cmark_constraints};
pub use metamodel::{
    ingest_markdown_via_cmark, normalize_ingested_markdown_cmark, parse_cmark_frontmatter,
};
pub use tables::{
    build_markdown_table_header, extract_cmark_section_prose, parse_cmark_markdown_structure,
};

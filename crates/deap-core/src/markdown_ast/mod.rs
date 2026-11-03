//! Grammar-backed `CommonMark` and GFM AST parsing, table extraction, and lossless emission. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the canonical `deap_core::markdown_ast` facade unifying `parser`, `table_ast`, and `emitter`
//! submodules under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - All AST spans (`start_offset`, `end_offset`) correspond to valid UTF-8 byte offsets in the source.
//! - Table, frontmatter, section, code block, and tasklist extraction operate via `pulldown-cmark`
//!   grammar events without ad-hoc line splitting.
//! - Serialization and formatting operate via `pulldown-cmark-to-cmark`.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 Markdown strings or `pulldown_cmark::Event` slices.
//!
//! ## 4. Postconditions
//! - Exposes a cohesive, deterministic `CommonMark` + GFM AST parsing and serialization surface.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear over document byte length $N$.
//! - Space Complexity: $O(N)$ proportional to extracted AST nodes and output buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed frontmatter, invalid span ranges, and I/O or serialization faults return `Err(CompilerError)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0401], [REQ-0604], [BP-04] [FLS: fls_t4w1da759n2e, fls_qcb1n9c0e5hz]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod emitter;
pub mod parser;
pub mod table_ast;

pub use emitter::{
    emit_cmark_table, emit_lossless_markdown, format_markdown_cmark, splice_markdown_byte_spans,
    write_spec_file_atomic,
};
pub use parser::{
    cmark_options, collect_code_block_byte_ranges, is_offset_in_byte_ranges,
    mask_html_comments_preserving_offsets, normalize_line_endings, parse_checklist_tasks,
    parse_frontmatter_metadata, parse_markdown_ast, ChecklistDoc, CompilerError, FrontmatterDoc,
    MarkdownCodeBlockAst, MarkdownSectionAst, MarkdownTaskItemAst, ParsedMarkdownAst, TaskListItem,
};
pub use pulldown_cmark;
pub use pulldown_cmark_to_cmark;
pub use table_ast::{
    is_br_html_tag, is_gfm_delimiter_cell, parse_frontmatter_kv_rows, parse_markdown_tables,
    parse_single_gfm_row_cells, CmarkTable, MarkdownKeyValueRow, MarkdownTableAst,
};
pub use toml_edit;

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    /// Verifies section, code block, tasklist, and frontmatter AST extraction.
    /// Realises: [REQ-0401], [REQ-0604]
    #[test]
    fn test_parse_sections_code_blocks_and_tasklists() {
        let md = "---\nid: FEAT-01\nstatus: Open\n---\n# Title\nIntro text.\n## Sub\n- [ ] Task A\n- [x] Task B\n```rust\nlet x = 1;\n```\n";
        let ast = parse_markdown_ast(md);
        assert_eq!(
            ast.metadata_blocks.first().map(|s| s.trim()),
            Some("id: FEAT-01\nstatus: Open")
        );
        assert_eq!(ast.sections.len(), 2);
        assert_eq!(
            ast.sections
                .first()
                .map(|s| (s.level, s.heading.as_str(), s.body.as_str())),
            Some((1, "Title", "Intro text."))
        );
        assert_eq!(ast.task_items.len(), 2);
        assert_eq!(
            ast.task_items
                .first()
                .map(|t| (t.is_checked, t.text.as_str())),
            Some((false, "Task A"))
        );
        assert_eq!(
            ast.task_items
                .get(1)
                .map(|t| (t.is_checked, t.text.as_str())),
            Some((true, "Task B"))
        );
        assert_eq!(
            ast.code_blocks.first().map(|c| c.info.as_str()),
            Some("rust")
        );
    }

    /// Verifies GFM table headers, rows, and inline HTML `<br>` handling.
    /// Realises: [REQ-0401], [REQ-0604]
    #[test]
    fn test_parse_markdown_table_ast_and_emit() -> Result<(), CompilerError> {
        let md = "| ID | State |\n| :--- | :---: |\n| `REQ-1` | Open<br/>Ready |\n";
        let ast = parse_markdown_ast(md);
        assert_eq!(
            ast.tables.first().map(|t| &t.headers),
            Some(&vec!["ID".to_string(), "State".to_string()])
        );
        assert_eq!(
            ast.tables.first().map(|t| &t.rows),
            Some(&vec![vec![
                "REQ-1".to_string(),
                "Open Ready".to_string()
            ]])
        );
        let ctables = parse_markdown_tables(md)?;
        assert_eq!(ctables.len(), 1);
        let emitted = emit_cmark_table(&CmarkTable {
            headers: vec!["Col1".to_string(), "Col2".to_string()],
            rows: vec![vec!["Val1".to_string(), "Val2".to_string()]],
        })?;
        assert!(emitted.contains("Col1"));
        assert!(emitted.contains("Val2"));
        Ok(())
    }

    /// Verifies frontmatter metadata parsing, checklist task extraction, and byte-span splicing.
    /// Realises: [REQ-0401], [REQ-0604]
    #[test]
    fn test_frontmatter_checklist_and_span_splicing() -> Result<(), CompilerError> {
        let md = "---\nid: EPIC-1\nstatus: Open\n---\n# Epic\n- [ ] (#10) Child item\n";
        let fm = parse_frontmatter_metadata(md)?;
        assert!(fm
            .raw_yaml
            .as_deref()
            .is_some_and(|y| y.contains("id: EPIC-1")));
        let checklist = parse_checklist_tasks(md);
        assert_eq!(checklist.tasks.len(), 1);
        let ast = parse_markdown_ast(md);
        let first_task = ast
            .task_items
            .first()
            .ok_or_else(|| CompilerError::LosslessEmitError("missing task".to_string()))?;
        let marker_end = first_task.marker_offset.saturating_add(3);
        let spliced = splice_markdown_byte_spans(
            md,
            vec![(first_task.marker_offset..marker_end, "[x]".to_string())],
        )?;
        assert!(spliced.contains("- [x] (#10) Child item"));
        Ok(())
    }

    /// Verifies canonical `CommonMark` roundtrip formatting and atomic file write.
    /// Realises: [REQ-0401], [REQ-0604]
    #[test]
    fn test_format_and_atomic_write() -> Result<(), CompilerError> {
        let formatted = format_markdown_cmark("# Heading\n\nParagraph\n")
            .map_err(CompilerError::LosslessEmitError)?;
        assert!(formatted.contains("# Heading"));
        let events: Vec<_> = Parser::new_ext("# Atomic\n\nBody\n", cmark_options()).collect();
        let tmp_dir = std::env::temp_dir().join(format!("deap_cmark_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir)
            .map_err(|e| CompilerError::LosslessEmitError(e.to_string()))?;
        let target = tmp_dir.join("spec.md");
        write_spec_file_atomic(&target, &events)?;
        let written = std::fs::read_to_string(&target)
            .map_err(|e| CompilerError::LosslessEmitError(e.to_string()))?;
        let _cleanup = std::fs::remove_dir_all(&tmp_dir);
        assert!(written.contains("# Atomic"));
        Ok(())
    }
}

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] CST-driven Dart 7-section safety docstring codemod (`FEAT-42` / `BP-05`).
//!
//! Safety & Traceability: Realises: [REQ-FEAT-42-INJECT-DART-DOCSTRINGS], [REQ-0605]
//! Contract: Synthesizes and injects 7-section safety docstrings into Dart source files via `deap_core::dart_cst` and `SourceLineEdits`.
//! Bounds & Faults: O(L + I) time and space where L is line count and I is callable docblock count; returns typed `DocstringError`.

use crate::docstring::{derive_module_req_tag, DocstringError};
use crate::docstring_compliance_check::{
    is_compliant_docblock_in_file_with_mode, DocCompliancePolicy,
};
use crate::docstring_contract_synth::{synthesize_7_section_docstring, ItemSynthesisContext};
use crate::docstring_cst::{
    classify_dart_member_ast, classify_dart_top_level_ast, leading_whitespace, parse_dart_cst,
    scan_line_with_unified_lex_state, strip_dart_annotations_tracking_state,
    strip_line_comment_preserving_strings, update_ast_delimiter_depths,
    validate_source_syntax_for_language, CstLexState, ScopedItemScannerState, SourceItemKind,
    SourceLanguage, SourceLineEdits,
};
use crate::docstring_dart_inspect::inspect_dart_item_details;
use crate::docstring_schema_bind::validate_unforced_schema_realises;
use deap_core::sysml_cst::PackageDef;
use std::collections::BTreeSet;

/// Target Dart item metadata passed to `DartInjectorState::validate_or_enrich_dart_item`.
/// Realises: [REQ-FEAT-42-DART-ITEM-TARGET]
struct DartItemTarget<'b> {
    idx: usize,
    kind: SourceItemKind,
    owner_name: Option<&'b str>,
    item_name: &'b str,
    check_uniqueness: bool,
}

/// Per-file state machine for Dart 7-section safety docstring synthesis.
/// Realises: [REQ-FEAT-42-DART-INJECTOR-STATE]
struct DartInjectorState<'a> {
    file_rel: &'a str,
    module_tag: String,
    schema_model: Option<&'a PackageDef>,
    force: bool,
    current_doc: Vec<String>,
    pending_doc_indices: Vec<usize>,
    first_annot_idx: Option<usize>,
    seen_docblocks: BTreeSet<String>,
    dart_lex_state: CstLexState,
    scoped_state: ScopedItemScannerState,
    brace_depth: usize,
    paren_depth: usize,
    active_class_name: Option<String>,
    edits: SourceLineEdits,
    item_ordinal: usize,
}

impl<'a> DartInjectorState<'a> {
    /// Initializes a `DartInjectorState` for `file_rel` across `total_lines`.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-STATE-NEW]
    fn new(
        total_lines: usize,
        file_rel: &'a str,
        schema_model: Option<&'a PackageDef>,
        force: bool,
    ) -> Self {
        Self {
            file_rel,
            module_tag: derive_module_req_tag(file_rel),
            schema_model,
            force,
            current_doc: Vec::new(),
            pending_doc_indices: Vec::new(),
            first_annot_idx: None,
            seen_docblocks: BTreeSet::new(),
            dart_lex_state: CstLexState {
                language: SourceLanguage::Dart,
                ..Default::default()
            },
            scoped_state: ScopedItemScannerState::default(),
            brace_depth: 0,
            paren_depth: 0,
            active_class_name: None,
            edits: SourceLineEdits::new(total_lines),
            item_ordinal: 0,
        }
    }

    /// Clears pending Dart docblock lines and leading `@annotation` index.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-RESET-PENDING]
    fn reset_pending_item_context(&mut self) {
        self.current_doc.clear();
        self.pending_doc_indices.clear();
        self.first_annot_idx = None;
    }

    /// Strips any placeholder lines from `current_doc` and resets pending item context.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-DISCARD-NON-ITEM]
    fn discard_non_item_doc(&mut self) {
        self.edits
            .strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
        self.reset_pending_item_context();
    }

    /// Scans and processes a single Dart line at `idx` in `raw_lines`.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-PROCESS-LINE]
    fn process_line(&mut self, raw_lines: &[&str], idx: usize) -> Result<(), DocstringError> {
        let raw_line = raw_lines.get(idx).copied().unwrap_or("");
        let trimmed = raw_line.trim();
        let started_in_comment_or_string = self.dart_lex_state.block_comment_depth > 0
            || self.dart_lex_state.active_string.is_some()
            || !self.dart_lex_state.interp_stack.is_empty();

        if !started_in_comment_or_string
            && self.scoped_state.dart_annotation_paren_depth == 0
            && trimmed.starts_with("///")
        {
            self.current_doc.push(trimmed.to_string());
            self.pending_doc_indices.push(idx);
            return Ok(());
        }

        let scanned = scan_line_with_unified_lex_state(trimmed, &mut self.dart_lex_state);
        if scanned.trim().is_empty() {
            return Ok(());
        }

        let code_raw = if !started_in_comment_or_string && !trimmed.contains("/*") {
            strip_line_comment_preserving_strings(trimmed)
        } else {
            strip_line_comment_preserving_strings(scanned.trim())
        };
        if code_raw.is_empty() {
            return Ok(());
        }

        if self.scoped_state.dart_annotation_paren_depth == 0
            && code_raw.starts_with('@')
            && self.first_annot_idx.is_none()
        {
            self.first_annot_idx = Some(idx);
        }
        let stripped_code = strip_dart_annotations_tracking_state(
            code_raw,
            &mut self.scoped_state.dart_annotation_paren_depth,
        );
        let code = stripped_code.trim();
        if code.is_empty() {
            return Ok(());
        }

        if self.paren_depth == 0 {
            self.dispatch_dart_declaration(raw_lines, idx, code)?;
        }

        let prev_brace_depth = self.brace_depth;
        update_ast_delimiter_depths(code, &mut self.brace_depth, &mut self.paren_depth);
        let closed_to_zero = prev_brace_depth > 0 && self.brace_depth == 0;
        let ended_at_zero = self.brace_depth == 0 && (code.ends_with(';') || code.ends_with('}'));
        if closed_to_zero || ended_at_zero {
            self.active_class_name = None;
        }
        Ok(())
    }

    /// Dispatches top-level (`brace_depth == 0`) or class-member (`brace_depth == 1`) Dart declarations.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-DISPATCH]
    fn dispatch_dart_declaration(
        &mut self,
        raw_lines: &[&str],
        idx: usize,
        code: &str,
    ) -> Result<(), DocstringError> {
        if self.brace_depth == 0 {
            if let Some((kind, item_name, _parent)) = classify_dart_top_level_ast(code) {
                let target = DartItemTarget {
                    idx,
                    kind,
                    owner_name: None,
                    item_name: &item_name,
                    check_uniqueness: kind == SourceItemKind::Function,
                };
                self.validate_or_enrich_dart_item(raw_lines, target)?;
                if matches!(
                    kind,
                    SourceItemKind::Class | SourceItemKind::Mixin | SourceItemKind::Extension
                ) {
                    self.active_class_name = Some(item_name);
                }
                self.reset_pending_item_context();
            } else {
                self.discard_non_item_doc();
            }
        } else if self.brace_depth == 1 {
            if let Some(cls_name) = self.active_class_name.clone() {
                if let Some((member_kind, member_name)) = classify_dart_member_ast(code, &cls_name)
                {
                    let target = DartItemTarget {
                        idx,
                        kind: member_kind,
                        owner_name: Some(cls_name.as_str()),
                        item_name: &member_name,
                        check_uniqueness: true,
                    };
                    self.validate_or_enrich_dart_item(raw_lines, target)?;
                    self.reset_pending_item_context();
                } else {
                    self.discard_non_item_doc();
                }
            } else {
                self.discard_non_item_doc();
            }
        } else {
            self.discard_non_item_doc();
        }
        Ok(())
    }

    /// Validates an existing Dart 7-section docblock or synthesizes a replacement 7-section docblock.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-VALIDATE-OR-ENRICH]
    fn validate_or_enrich_dart_item(
        &mut self,
        raw_lines: &[&str],
        target: DartItemTarget<'_>,
    ) -> Result<(), DocstringError> {
        self.item_ordinal = self.item_ordinal.saturating_add(1);
        let policy = DocCompliancePolicy {
            is_tier_a: true,
            check_uniqueness: target.check_uniqueness,
            allow_legacy_tier_bc_sections: false,
        };
        let is_valid = !self.force
            && is_compliant_docblock_in_file_with_mode(
                &self.current_doc,
                policy,
                &mut self.seen_docblocks,
            );

        if is_valid {
            validate_unforced_schema_realises(
                self.file_rel,
                target.item_name,
                &self.current_doc,
                self.schema_model,
                target.owner_name,
            )?;
        } else {
            self.edits.mark_removed(&self.pending_doc_indices);
            let raw_line = raw_lines.get(target.idx).copied().unwrap_or("");
            let indent = leading_whitespace(raw_line);
            let (params, return_type, fields_or_variants, invariants) =
                inspect_dart_item_details(raw_lines, target.idx, target.kind, target.item_name);
            let ctx = ItemSynthesisContext {
                file_rel: self.file_rel,
                module_tag: &self.module_tag,
                owner_name: target.owner_name,
                item_kind: target.kind,
                item_kw_override: None,
                item_name: target.item_name,
                ordinal_index: self.item_ordinal,
                indent,
                existing_doc: &self.current_doc,
                params,
                return_type,
                fields_or_variants,
                invariants,
                schema_model: self.schema_model,
            };
            let new_doc = synthesize_7_section_docstring(&ctx, &mut self.seen_docblocks);
            let target_insert_idx = self.first_annot_idx.unwrap_or(target.idx);
            self.edits.insert_before(target_insert_idx, new_doc);
            self.edits.enriched_count = self.edits.enriched_count.saturating_add(1);
        }
        Ok(())
    }
}

/// [DOC-TIER: A] Synthesizes and injects 7-section safety docstrings into a single Dart source buffer.
///
/// Safety & Traceability: Realises: [REQ-FEAT-42-INJECT-DART-DOCSTRINGS], [REQ-0605]
/// Contract: Parses `source` via `parse_dart_cst`, validates delimiter balance, and enriches Dart declarations with 7-section docstrings.
/// Bounds & Faults: O(L + I) time and space where L is line count and I is callable docblock count; returns `Err(DocstringError)` on syntax or unforced conflict error.
pub fn inject_dart_file_docstrings(
    source: &str,
    file_rel: &str,
    schema_model: Option<&PackageDef>,
    force: bool,
) -> Result<(String, usize), DocstringError> {
    let dart_cst = parse_dart_cst(source);
    let _cst_item_count = dart_cst.len();
    validate_source_syntax_for_language(source, file_rel, SourceLanguage::Dart)
        .map_err(|e| DocstringError::Parse(e.into_string()))?;

    let raw_lines: Vec<&str> = source.lines().collect();
    let mut state = DartInjectorState::new(raw_lines.len(), file_rel, schema_model, force);

    for idx in 0..raw_lines.len() {
        state.process_line(&raw_lines, idx)?;
    }

    if state.edits.enriched_count == 0 {
        return Ok((source.to_string(), 0));
    }

    let rendered = state.edits.render(source, &raw_lines);
    Ok((rendered, state.edits.enriched_count))
}

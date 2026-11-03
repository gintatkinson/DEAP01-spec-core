#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Per-file Rust safety docstring synthesizer and Tier B/C compactor state machine.
//! Safety & Traceability: Realises: [REQ-FEAT-42-INJECT-RUST-DOCSTRINGS], [REQ-0501], [REQ-0605]
//! Contract: Enriches Rust Tier A items with 7-section docstrings and compacts Tier B/C items to 2-line docblocks.
//! Bounds & Faults: O(L) time and space in file line count `L`; fails closed on syntax or unforced requirement conflicts.

use crate::docstring::{derive_module_req_tag, DocstringError};
use crate::docstring_compliance_check::{
    has_legacy_seven_section_headers, has_placeholder_doc_line,
    is_compliant_docblock_in_file_with_mode, DocCompliancePolicy,
};
use crate::docstring_cst::{
    is_modular_codegen_generated_rel, parse_rust_impl_header, scan_line_with_unified_lex_state,
    update_ast_delimiter_depths, validate_source_syntax_for_language, CstLexState, SourceItemKind,
    SourceLanguage, SourceLineEdits,
};
use crate::docstring_rust_inspect::{
    emit_synthesized_rust_item_doc, RustEmitDocRequest, RustLineScanContext,
};
pub(crate) use crate::docstring_rust_scan::{ClassifiedRustItem, PendingRustImpl, RustOwnerKind};
use crate::docstring_rust_scan::{
    advance_pending_rust_impl, classify_pending_rust_attr, classify_rust_item_for_injection,
    consume_leading_rust_attrs, extract_rust_impl_target_and_kind, is_lex_state_normal,
    is_mod_tests_entry, scan_rust_impl_segment, update_attr_bracket_depth, RustPendingAttr,
};
use crate::docstring_schema_bind::validate_unforced_schema_realises;
use deap_core::sysml_cst::PackageDef;
use std::collections::BTreeSet;

/// Per-file state machine for tiered Rust safety docstring synthesis and Tier B/C compaction.
/// Realises: [REQ-FEAT-42-RUST-INJECTOR-STATE]
struct RustInjectorState<'a> {
    file_rel: &'a str,
    module_tag: String,
    schema_model: Option<&'a PackageDef>,
    force: bool,
    skip_tier_bc_compaction: bool,
    current_doc: Vec<String>,
    pending_doc_indices: Vec<usize>,
    first_attr_idx: Option<usize>,
    seen_docblocks: BTreeSet<String>,
    lex_state: CstLexState,
    in_mod_tests: bool,
    pending_attr: RustPendingAttr,
    attr_bracket_depth: usize,
    brace_depth: usize,
    paren_depth: usize,
    active_fn_body_depth: Option<usize>,
    pending_fn_body_depth: Option<usize>,
    impl_owner_stack: Vec<(usize, String, RustOwnerKind)>,
    pending_impl: Option<PendingRustImpl>,
    edits: SourceLineEdits,
    item_ordinal: usize,
}

impl<'a> RustInjectorState<'a> {
    /// Initializes a `RustInjectorState` for `file_rel` across `total_lines`.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-STATE-NEW]
    fn new(total_lines: usize, file_rel: &'a str, schema_model: Option<&'a PackageDef>, force: bool) -> Self {
        Self {
            file_rel,
            module_tag: derive_module_req_tag(file_rel),
            schema_model,
            force,
            skip_tier_bc_compaction: is_modular_codegen_generated_rel(file_rel),
            current_doc: Vec::new(),
            pending_doc_indices: Vec::new(),
            first_attr_idx: None,
            seen_docblocks: BTreeSet::new(),
            lex_state: CstLexState { language: SourceLanguage::Rust, ..Default::default() },
            in_mod_tests: false,
            pending_attr: RustPendingAttr::None,
            attr_bracket_depth: 0,
            brace_depth: 0,
            paren_depth: 0,
            active_fn_body_depth: None,
            pending_fn_body_depth: None,
            impl_owner_stack: Vec::new(),
            pending_impl: None,
            edits: SourceLineEdits::new(total_lines),
            item_ordinal: 0,
        }
    }

    /// Clears pending docblock lines, attribute anchor index, and pending test attribute state.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-RESET-PENDING]
    fn reset_pending_item_context(&mut self) {
        self.pending_attr = RustPendingAttr::None;
        self.current_doc.clear();
        self.pending_doc_indices.clear();
        self.first_attr_idx = None;
    }

    /// Returns the non-empty nominal type identifier of the innermost enclosing `impl` or `trait` block.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-OWNER-NAME]
    fn current_owner_name(&self) -> Option<&str> {
        self.impl_owner_stack
            .last()
            .and_then(|(_, name, _)| (!name.is_empty()).then_some(name.as_str()))
    }

    /// Scans and processes a single line at `idx` in `raw_lines`.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-PROCESS-LINE]
    fn process_line(&mut self, raw_lines: &[&str], idx: usize) -> Result<(), DocstringError> {
        let raw_line = raw_lines.get(idx).copied().unwrap_or("");
        let trimmed = raw_line.trim();
        let started_in_normal = is_lex_state_normal(&self.lex_state);
        let started_in_string =
            self.lex_state.raw_hash_count.is_some() || self.lex_state.active_string.is_some();

        if started_in_normal {
            if trimmed.starts_with("//!") {
                if has_placeholder_doc_line(trimmed) {
                    self.edits.mark_removed(&[idx]);
                    self.edits.enriched_count = self.edits.enriched_count.saturating_add(1);
                }
                return Ok(());
            }
            if trimmed.starts_with("///") {
                if !self.in_mod_tests {
                    self.current_doc.push(trimmed.to_string());
                    self.pending_doc_indices.push(idx);
                }
                return Ok(());
            }
        }

        let scanned_code = scan_line_with_unified_lex_state(trimmed, &mut self.lex_state);
        let ended_in_normal = is_lex_state_normal(&self.lex_state);
        let raw_code = scanned_code.trim();
        let mut code = if started_in_string {
            raw_code.strip_prefix('"').unwrap_or(raw_code).trim()
        } else {
            raw_code
        };
        if code.is_empty() {
            return Ok(());
        }
        if !started_in_normal && (!ended_in_normal || started_in_string) {
            if self.attr_bracket_depth > 0 {
                update_attr_bracket_depth(code, &mut self.attr_bracket_depth);
            } else {
                self.update_depths_after_code(code);
            }
            return Ok(());
        }
        if is_mod_tests_entry(code, self.pending_attr) {
            self.in_mod_tests = true;
        }
        if self.in_mod_tests {
            return Ok(());
        }
        if self.attr_bracket_depth > 0 {
            update_attr_bracket_depth(code, &mut self.attr_bracket_depth);
            return Ok(());
        }
        if code.starts_with("#[") {
            if self.first_attr_idx.is_none() {
                self.first_attr_idx = Some(idx);
            }
            self.pending_attr = classify_pending_rust_attr(code, self.pending_attr);
            code = consume_leading_rust_attrs(code, &mut self.attr_bracket_depth);
            if code.is_empty() {
                return Ok(());
            }
        }
        if self.paren_depth == 0 {
            let scan = RustLineScanContext {
                raw_line,
                idx,
                code,
                inline_comment_closed: !started_in_normal && ended_in_normal,
            };
            self.handle_top_level_code(raw_lines, &scan)?;
        }
        self.update_depths_after_code(code);
        Ok(())
    }

    /// Dispatches `impl` headers and item declarations encountered at `paren_depth == 0`.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-HANDLE-TOP-LEVEL]
    fn handle_top_level_code(&mut self, raw_lines: &[&str], scan: &RustLineScanContext<'_>) -> Result<(), DocstringError> {
        let code = scan.code;
        let enclosing_owner_kind = self.impl_owner_stack.last().map(|&(_, _, kind)| kind);
        let in_fn_body = self.active_fn_body_depth.is_some_and(|open_d| self.brace_depth > open_d);

        if let Some((impl_target, impl_kind)) = extract_rust_impl_target_and_kind(code) {
            self.pending_impl = None;
            if code.contains('{') && !code.contains('}') {
                self.impl_owner_stack.push((self.brace_depth, impl_target, impl_kind));
            }
            self.edits.strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            self.reset_pending_item_context();
        } else if self.brace_depth == 0 && parse_rust_impl_header(&format!("{code} {{")).is_some() {
            if let Some(impl_target) = parse_rust_impl_header(&format!("{code} {{")) {
                if !code.ends_with(';') && !code.contains('}') {
                    let (trait_for_tail, saw_where) = scan_rust_impl_segment(code);
                    let kind = if trait_for_tail.is_some() {
                        RustOwnerKind::TraitImpl
                    } else {
                        RustOwnerKind::InherentImpl
                    };
                    self.pending_impl = Some(PendingRustImpl { target: impl_target, kind, in_where_clause: saw_where });
                }
            }
            self.edits.strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            self.reset_pending_item_context();
        } else if self.brace_depth == 0 && self.pending_impl.is_some() {
            advance_pending_rust_impl(code, self.brace_depth, &mut self.pending_impl, &mut self.impl_owner_stack);
        } else if let Some(item) = classify_rust_item_for_injection(code, enclosing_owner_kind, in_fn_body) {
            self.handle_classified_item(raw_lines, scan, item)?;
        } else {
            self.edits.strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            self.reset_pending_item_context();
        }
        Ok(())
    }

    /// Evaluates compliance, compaction, or synthesis for a classified Rust declaration.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-HANDLE-ITEM]
    fn handle_classified_item(
        &mut self,
        raw_lines: &[&str],
        scan: &RustLineScanContext<'_>,
        item: ClassifiedRustItem,
    ) -> Result<(), DocstringError> {
        if self.pending_attr != RustPendingAttr::None {
            if !self.skip_tier_bc_compaction && has_legacy_seven_section_headers(&self.current_doc) {
                let owner_exists = self.current_owner_name().map(str::to_string);
                let mut tier_bc_item = item.clone();
                tier_bc_item.is_tier_a = false;
                self.item_ordinal = self.item_ordinal.saturating_add(1);
                self.emit_synthesized_rust_doc(raw_lines, scan, &tier_bc_item, owner_exists.as_deref());
            } else {
                self.edits.strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            }
            self.register_item_scope(scan.code, &item);
            self.reset_pending_item_context();
            return Ok(());
        }

        self.item_ordinal = self.item_ordinal.saturating_add(1);
        let owner_exists = self.current_owner_name().map(str::to_string);
        let owner_ref = owner_exists.as_deref();
        let policy = DocCompliancePolicy {
            is_tier_a: item.is_tier_a,
            check_uniqueness: item.is_tier_a && item.kind == SourceItemKind::Function,
            allow_legacy_tier_bc_sections: self.skip_tier_bc_compaction,
        };
        let is_valid = !self.force
            && is_compliant_docblock_in_file_with_mode(&self.current_doc, policy, &mut self.seen_docblocks);
        if !self.force && (is_valid || has_legacy_seven_section_headers(&self.current_doc)) {
            validate_unforced_schema_realises(
                self.file_rel,
                &item.name,
                &self.current_doc,
                self.schema_model,
                owner_ref,
            )?;
        }
        if !is_valid {
            self.emit_synthesized_rust_doc(raw_lines, scan, &item, owner_ref);
        }
        self.register_item_scope(scan.code, &item);
        self.reset_pending_item_context();
        Ok(())
    }

    /// Synthesizes either a Tier A 7-section docblock or a concise Tier B/C 2-line docblock.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-EMIT-DOC]
    fn emit_synthesized_rust_doc(
        &mut self,
        raw_lines: &[&str],
        scan: &RustLineScanContext<'_>,
        item: &ClassifiedRustItem,
        owner_name: Option<&str>,
    ) {
        let req = RustEmitDocRequest {
            file_rel: self.file_rel,
            module_tag: &self.module_tag,
            owner_name,
            item,
            ordinal_index: self.item_ordinal,
            first_attr_idx: self.first_attr_idx,
            current_doc: &self.current_doc,
            pending_doc_indices: &self.pending_doc_indices,
            schema_model: self.schema_model,
        };
        emit_synthesized_rust_item_doc(raw_lines, scan, &req, &mut self.seen_docblocks, &mut self.edits);
    }

    /// Registers `TraitDef` owner scope or pending function body tracking before delimiter updates.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-REGISTER-SCOPE]
    fn register_item_scope(&mut self, code: &str, item: &ClassifiedRustItem) {
        if item.keyword_override.is_none()
            && item.kind == SourceItemKind::Trait
            && code.contains('{')
            && !code.contains('}')
        {
            self.impl_owner_stack.push((self.brace_depth, item.name.clone(), RustOwnerKind::TraitDef));
        }
        if item.keyword_override.is_none()
            && item.kind == SourceItemKind::Function
            && self.active_fn_body_depth.is_none()
        {
            self.pending_fn_body_depth = Some(self.brace_depth);
        }
    }

    /// Updates `(brace_depth, paren_depth)`, function-body scope tracking, and `impl_owner_stack` closures.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-UPDATE-DEPTHS]
    fn update_depths_after_code(&mut self, code: &str) {
        update_ast_delimiter_depths(code, &mut self.brace_depth, &mut self.paren_depth);
        if let Some(open_d) = self.pending_fn_body_depth {
            if self.brace_depth > open_d {
                self.active_fn_body_depth = Some(open_d);
                self.pending_fn_body_depth = None;
            } else if self.paren_depth == 0 && (code.ends_with(';') || code.ends_with('}')) {
                self.pending_fn_body_depth = None;
            }
        }
        if let Some(open_d) = self.active_fn_body_depth {
            if self.brace_depth <= open_d {
                self.active_fn_body_depth = None;
            }
        }
        while let Some(&(open_depth, _, _)) = self.impl_owner_stack.last() {
            if self.brace_depth <= open_depth {
                self.impl_owner_stack.pop();
            } else {
                break;
            }
        }
    }
}

/// [DOC-TIER: A] Synthesizes and compacts tiered v2.0 safety docstrings into a single Rust source buffer.
/// Safety & Traceability: Realises: [REQ-FEAT-42-INJECT-RUST-DOCSTRINGS], [REQ-0501]
/// Contract: Preserves or synthesizes 7-section / 4-Block Contract docstrings on Tier A items and 2-line docstrings on Tier B/C items.
/// Bounds & Faults: O(L) time and space in `source` lines `L`; returns `Err(DocstringError)` on syntax or unforced conflict errors.
pub fn inject_rust_file_docstrings(
    source: &str,
    file_rel: &str,
    schema_model: Option<&PackageDef>,
    force: bool,
) -> Result<(String, usize), DocstringError> {
    validate_source_syntax_for_language(source, file_rel, SourceLanguage::Rust)
        .map_err(|e| DocstringError::Parse(e.into_string()))?;

    let raw_lines: Vec<&str> = source.lines().collect();
    let mut state = RustInjectorState::new(raw_lines.len(), file_rel, schema_model, force);
    for idx in 0..raw_lines.len() {
        state.process_line(&raw_lines, idx)?;
    }
    if state.edits.enriched_count == 0 {
        return Ok((source.to_string(), 0));
    }
    let rendered = state.edits.render(source, &raw_lines);
    Ok((rendered, state.edits.enriched_count))
}

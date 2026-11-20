//! Recursive SysML v2 `part def` AST extraction with nested-part masking (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::collections::BTreeSet;

use deap_core::sysml_cst::{tokenize_sysml, LexedSysmlToken, SysmlToken};

use super::cmark_engine::SubsystemPart;
use super::cmark_schema::{
    extract_part_body_attributes, find_line_number, is_non_trivia, parse_subsystem_ports_from_body,
};

/// Maximum recursion depth for nested SysML `part def` extraction (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
const MAX_PART_SCAN_DEPTH: usize = 64;

/// Advances past `part def <name>` specializations and multiplicity clauses to `{` or `;` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn advance_part_header_to_body_or_semi(tokens: &[LexedSysmlToken<'_>], mut cur: usize) -> Option<usize> {
    while let Some(tok) = tokens.get(cur) {
        if !is_non_trivia(tok) {
            cur = cur.saturating_add(1);
            continue;
        }
        match tok.token {
            SysmlToken::LBrace | SysmlToken::Semi => return Some(cur),
            SysmlToken::Colon
            | SysmlToken::ColonGt
            | SysmlToken::ColonGtGt
            | SysmlToken::ColonColonGt
            | SysmlToken::ColonColon
            | SysmlToken::Comma
            | SysmlToken::Tilde
            | SysmlToken::DotDot
            | SysmlToken::LBracket
            | SysmlToken::RBracket
            | SysmlToken::NumberLit
            | SysmlToken::Ident
            | SysmlToken::UnrestrictedName => cur = cur.saturating_add(1),
            SysmlToken::KermlModifierKw
                if matches!(tok.slice, "specializes" | "redefines" | "subsets" | "references" | "ordered" | "nonunique") =>
            {
                cur = cur.saturating_add(1);
            }
            SysmlToken::Operator if matches!(tok.slice, "*" | ",") => cur = cur.saturating_add(1),
            _ => return None,
        }
    }
    None
}

/// Locates the matching `}` token index and inner byte span for a `LBrace` at `open_idx` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn find_matching_lbrace_span(tokens: &[LexedSysmlToken<'_>], open_idx: usize) -> Option<(usize, usize, usize)> {
    let open_end = tokens.get(open_idx)?.span.end;
    let mut depth = 0usize;
    for (k, t) in tokens.iter().enumerate().skip(open_idx) {
        if t.token == SysmlToken::LBrace {
            depth = depth.saturating_add(1);
        } else if t.token == SysmlToken::RBrace {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((k, open_end, t.span.start));
            }
        }
    }
    None
}

/// Masks brace-enclosed bodies of nested `part` declarations while preserving offsets and newlines (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn mask_nested_part_bodies(body: &str) -> String {
    let tokens = tokenize_sysml(body);
    let mut out = body.as_bytes().to_vec();
    let mut idx = 0usize;
    while let Some(tok) = tokens.get(idx) {
        if tok.token == SysmlToken::StructuralKw && tok.slice == "part" {
            let mut cur = idx.saturating_add(1);
            while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur = cur.saturating_add(1); }
            if tokens.get(cur).is_some_and(|t| t.token == SysmlToken::DefKw) {
                cur = cur.saturating_add(1);
                while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur = cur.saturating_add(1); }
            }
            if tokens.get(cur).is_some_and(|t| matches!(t.token, SysmlToken::Ident | SysmlToken::UnrestrictedName)) {
                if let Some(body_idx) = advance_part_header_to_body_or_semi(&tokens, cur.saturating_add(1)) {
                    if tokens.get(body_idx).is_some_and(|t| t.token == SysmlToken::LBrace) {
                        if let Some((close_idx, open_end, close_start)) = find_matching_lbrace_span(&tokens, body_idx) {
                            if let Some(slice) = out.get_mut(open_end..close_start) {
                                for byte in slice {
                                    if *byte != b'\n' { *byte = b' '; }
                                }
                            }
                            idx = close_idx.saturating_add(1);
                            continue;
                        }
                    }
                }
            }
        }
        idx = idx.saturating_add(1);
    }
    String::from_utf8(out).unwrap_or_else(|_| body.to_string())
}

/// Resolves the `{ ... }` body slice and next token index starting at `cur` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn resolve_part_body_span<'a>(
    slice: &'a str,
    tokens: &[LexedSysmlToken<'_>],
    cur: usize,
    part_pos: usize,
    idx: usize,
) -> Option<(&'a str, usize, usize)> {
    if tokens.get(cur).is_some_and(|t| t.token == SysmlToken::LBrace) {
        if let Some((r_idx, open_end, close_start)) = find_matching_lbrace_span(tokens, cur) {
            Some((slice.get(open_end..close_start).unwrap_or(""), open_end, r_idx.saturating_add(1)))
        } else {
            let open_end = tokens.get(cur)?.span.end;
            Some((slice.get(open_end..).unwrap_or(""), open_end, tokens.len()))
        }
    } else if tokens.get(cur).is_some_and(|t| t.token == SysmlToken::Semi) {
        Some(("", part_pos, cur.saturating_add(1)))
    } else {
        Some(("", part_pos, idx.saturating_add(1)))
    }
}

/// Parses a single `part def` declaration starting at token `idx` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn parse_part_from_tokens<'a>(
    full_content: &str,
    slice: &'a str,
    (base_offset, filepath): (usize, &str),
    tokens: &[LexedSysmlToken<'_>],
    idx: usize,
) -> Option<(SubsystemPart, &'a str, usize, usize)> {
    let part_pos = tokens.get(idx)?.span.start;
    let mut cur = idx.saturating_add(1);
    while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur = cur.saturating_add(1); }
    if !tokens.get(cur).is_some_and(|t| t.token == SysmlToken::DefKw) { return None; }
    cur = cur.saturating_add(1);
    while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur = cur.saturating_add(1); }
    let name_tok = tokens.get(cur)?;
    if !matches!(name_tok.token, SysmlToken::Ident | SysmlToken::UnrestrictedName) || name_tok.slice == "def" {
        return None;
    }
    let name = name_tok.slice.trim_matches('\'').to_string();
    let header_end = advance_part_header_to_body_or_semi(tokens, cur.saturating_add(1))?;
    let (body, body_offset, next_idx) = resolve_part_body_span(slice, tokens, header_end, part_pos, idx)?;
    let shallow_body = if body.is_empty() { String::new() } else { mask_nested_part_bodies(body) };
    let doc = tokenize_sysml(&shallow_body).into_iter().find(|t| t.token == SysmlToken::BlockComment).map(|t| {
        let inner = t.slice.trim().strip_prefix("/*").unwrap_or(t.slice).strip_suffix("*/").unwrap_or(t.slice).trim();
        inner.strip_prefix("doc").unwrap_or(inner).trim().to_string()
    }).unwrap_or_default();
    let abs_body_offset = base_offset.saturating_add(body_offset);
    let ports = if shallow_body.is_empty() {
        Vec::new()
    } else {
        parse_subsystem_ports_from_body(full_content, &shallow_body, abs_body_offset, filepath)
    };
    let (attributes, mass_kg, power_w, actions) = extract_part_body_attributes(&shallow_body);
    let part = SubsystemPart {
        name, doc, ports, mass_kg, power_w, actions, attributes, constraints: Vec::new(),
        source_file: filepath.to_string(),
        source_line: find_line_number(full_content, base_offset.saturating_add(part_pos)),
        epistemic_tier: "[TIER-1: OEM]".to_string(),
    };
    Some((part, body, abs_body_offset, next_idx.max(idx.saturating_add(1))))
}

/// Depth-bounded recursive accumulator for SysML `part def` extraction (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
struct PartScanCtx<'a> {
    full_content: &'a str,
    filepath: &'a str,
    seen: BTreeSet<String>,
    parts: Vec<SubsystemPart>,
}

impl<'a> PartScanCtx<'a> {
    /// Collects `part def` declarations recursively up to `MAX_PART_SCAN_DEPTH` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn collect_slice(&mut self, slice: &str, base_offset: usize, depth: usize) {
        if depth > MAX_PART_SCAN_DEPTH { return; }
        let tokens = tokenize_sysml(slice);
        let mut idx = 0usize;
        while let Some(tok) = tokens.get(idx) {
            if tok.token == SysmlToken::StructuralKw && tok.slice == "part" {
                if let Some((part, body, abs_body_offset, next_idx)) =
                    parse_part_from_tokens(self.full_content, slice, (base_offset, self.filepath), &tokens, idx)
                {
                    if self.seen.insert(part.name.clone()) { self.parts.push(part); }
                    if !body.is_empty() {
                        self.collect_slice(body, abs_body_offset, depth.saturating_add(1));
                    }
                    idx = next_idx;
                    continue;
                }
            }
            idx = idx.saturating_add(1);
        }
    }
}

/// [DOC-TIER: A] Extracts `SubsystemPart` definitions with ports, attributes, actions, and source provenance from SysML text.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-SYSML] [FLS: fls_conops_cmark]
/// Contract: Scans `content: &str` token stream recursively for `part def` declarations and returns deduplicated `Vec<SubsystemPart>`.
/// Bounds & Faults: O(N * D) time bounded by `MAX_PART_SCAN_DEPTH = 64`; records 1-indexed `source_line` and `[TIER-1: OEM]` provenance.
pub fn extract_subsystem_parts_from_sysml(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let mut ctx = PartScanCtx { full_content: content, filepath, seen: BTreeSet::new(), parts: Vec::new() };
    ctx.collect_slice(content, 0, 0);
    ctx.parts
}

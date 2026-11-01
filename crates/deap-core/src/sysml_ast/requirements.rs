//! Requirement definitions, satisfaction bindings, and safety/layer annotation AST parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-REQUIREMENTS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `RequirementDef` and provides deterministic, zero-regex `parse_requirement_defs` and `parse_satisfied_requirements` extractors together with `@TriggersHazard`, `@SafetyRealises`, and `@Layer` annotation parsing.
//! Bounds & Faults: Executes in linear O(N) time and O(N) space over comment-stripped buffers using bounds-checked `.get` slicing, saturating offset arithmetic, and zero panics on malformed or multi-byte UTF-8 inputs.

use super::expressions::{parse_constraint_defs, ConstraintDef};
use super::identifiers::{
    extract_doc_comment_before, extract_quoted_string, find_matching_brace, is_ident_char,
    is_word_boundary_after, is_word_boundary_before, mask_string_literals, skip_ws, AttributeDef,
};
use super::interfaces_and_items::parse_attribute_defs;
use super::strip_sysml_comments;
use serde::{Deserialize, Serialize};

/// Represents a normative requirement definition or requirement usage in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-REQUIREMENTDEF]
/// Contract: Preserves requirement `name`, canonical `req_id`, specification `text`, optional `doc` block, encapsulated `attributes` and `constraints`, `assumes`/`requires`/`verified_by`/`satisfied_by`/`derived_from`/`derives` traceability edges, and `@TriggersHazard`, `@SafetyRealises`, and `@Layer` annotations.
/// Bounds & Faults: O(1) default construction and linear O(M) clone and `serde` round-trip cost over declared member collections with zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RequirementDef {
    pub name: String,
    pub req_id: String,
    pub text: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub attributes: Vec<AttributeDef>,
    #[serde(default)]
    pub constraints: Vec<ConstraintDef>,
    #[serde(default)]
    pub assumes: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub verified_by: Vec<String>,
    #[serde(default)]
    pub satisfied_by: Vec<String>,
    #[serde(default)]
    pub derived_from: Vec<String>,
    #[serde(default)]
    pub derives: Vec<String>,
    #[serde(default)]
    pub triggers_hazards: Vec<String>,
    #[serde(default)]
    pub safety_realises: Vec<String>,
    #[serde(default)]
    pub layer: Option<String>,
}

/// Extracts all double-quoted string literals from the suffix following an `@Annotation` tag.
/// Realises: [REQ-SYSML-AST-ANNOTATION-VALUES]
fn extract_quoted_annotation_literals(after_tag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut cursor: usize = 0;
    while let Some(open_q) = after_tag.get(cursor..).and_then(|s| s.find('"')) {
        let start = cursor.saturating_add(open_q).saturating_add(1);
        let Some(close_q) = after_tag.get(start..).and_then(|s| s.find('"')) else {
            break;
        };
        let end = start.saturating_add(close_q);
        if let Some(val) = after_tag.get(start..end) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                values.push(trimmed.to_string());
            }
        }
        cursor = end.saturating_add(1);
    }
    values
}

/// Extracts quoted literals, parenthesized argument lists, or `= value` tokens from an `@Annotation` line.
/// Realises: [REQ-SYSML-AST-ANNOTATION-VALUES]
fn extract_annotation_values(after_tag: &str) -> Vec<String> {
    let mut values = extract_quoted_annotation_literals(after_tag);
    if !values.is_empty() {
        return values;
    }
    let trimmed = after_tag.trim();
    if let Some(inside_parens) = trimmed
        .strip_prefix('(')
        .and_then(|s| s.split_once(')').map(|(inner, _)| inner))
    {
        for part in inside_parens.split(',') {
            let clean = part.trim().trim_matches('\'').trim();
            if !clean.is_empty() {
                values.push(clean.to_string());
            }
        }
    } else if let Some((_, rhs)) = trimmed.split_once('=') {
        let clean = rhs
            .trim()
            .trim_end_matches('}')
            .trim()
            .trim_end_matches(';')
            .trim()
            .trim_matches('\'')
            .trim();
        if !clean.is_empty() {
            values.push(clean.to_string());
        }
    }
    values
}

/// Parses `@TriggersHazard`, `@SafetyRealises`, and `@Layer` annotations from a SysML v2 declaration body.
/// Realises: [REQ-SYSML-AST-SAFETY-LAYER-ANNOTATIONS]
pub(super) fn parse_safety_and_layer_annotations(
    body: &str,
) -> (Vec<String>, Vec<String>, Option<String>) {
    let mut triggers_hazards = Vec::new();
    let mut safety_realises = Vec::new();
    let mut layer = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(after) = trimmed.strip_prefix("@TriggersHazard") {
            for val in extract_annotation_values(after) {
                if !triggers_hazards.contains(&val) {
                    triggers_hazards.push(val);
                }
            }
        } else if let Some(after) = trimmed.strip_prefix("@SafetyRealises") {
            for val in extract_annotation_values(after) {
                if !safety_realises.contains(&val) {
                    safety_realises.push(val);
                }
            }
        } else if let Some(after) = trimmed.strip_prefix("@Layer") {
            if let Some(first_val) = extract_annotation_values(after).into_iter().next() {
                layer = Some(first_val);
            }
        }
    }

    (triggers_hazards, safety_realises, layer)
}

/// Extracts the first inline `doc /* ... */` documentation block inside a declaration body when non-empty.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
pub(super) fn extract_inline_doc_block(body: &str) -> Option<String> {
    let mut idx: usize = 0;
    while let Some(rel) = body.get(idx..).and_then(|s| s.find("doc")) {
        let doc_pos = idx.saturating_add(rel);
        let after_kw = doc_pos.saturating_add(3);
        if is_word_boundary_before(body, doc_pos) && is_word_boundary_after(body, after_kw) {
            let after_doc = body.get(after_kw..).unwrap_or("").trim_start();
            if let Some(after_open) = after_doc.strip_prefix("/*") {
                if let Some(close_doc) = after_open.find("*/") {
                    let clean_doc = after_open.get(..close_doc).unwrap_or("").trim();
                    if !clean_doc.is_empty() {
                        return Some(clean_doc.to_string());
                    }
                }
            }
        }
        idx = after_kw;
    }
    None
}

/// Strips a leading keyword `kw` at a SysML v2 word boundary and trims leading whitespace.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn strip_kw_at_boundary<'a>(s: &'a str, kw: &str) -> Option<&'a str> {
    if s.starts_with(kw) && is_word_boundary_after(s, kw.len()) {
        Some(s.get(kw.len()..).unwrap_or("").trim_start())
    } else {
        None
    }
}

/// Checks whether `requirement` at byte offset `req_pos` is preceded by the `satisfy` keyword.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn is_preceded_by_satisfy_keyword(masked: &str, req_pos: usize) -> bool {
    masked
        .get(..req_pos)
        .map(str::trim_end)
        .is_some_and(|before| {
            before.ends_with("satisfy")
                && is_word_boundary_before(before, before.len().saturating_sub(7))
        })
}

/// Extracts `<kw> = "<quoted>"` from a trimmed requirement body line.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn parse_assigned_quoted_clause(trimmed: &str, kw: &str) -> Option<String> {
    let after = strip_kw_at_boundary(trimmed, kw)?;
    let after_eq = after.strip_prefix('=')?;
    extract_quoted_string(after_eq.trim_start()).map(str::to_string)
}

/// Parses an optional `by` keyword followed by `<target>;` inside `verify` or `satisfy` clauses.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn parse_optional_by_target(after_kw: &str) -> Option<String> {
    let after_by = strip_kw_at_boundary(after_kw, "by").unwrap_or(after_kw);
    after_by.strip_suffix(';').map(|t| t.trim().to_string())
}

/// Parsed header and optional `{ ... }` body slice for a single `requirement` or `requirement def` block.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
struct ParsedRequirementSpan<'a> {
    name: &'a str,
    doc: Option<String>,
    body_str: Option<&'a str>,
    next_idx: usize,
}

/// Parses a `requirement [def] <name> ({ ... } | ;)` span starting at byte offset `req_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn parse_requirement_span<'a>(
    clean: &'a str,
    masked: &'a str,
    req_pos: usize,
) -> Option<ParsedRequirementSpan<'a>> {
    let doc = extract_doc_comment_before(clean, req_pos);
    let mut curr = req_pos.saturating_add(11);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    let mut has_def = false;
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        has_def = true;
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let rem_ident = masked.get(curr..).unwrap_or("");
    let name_len = if has_def {
        rem_ident
            .bytes()
            .take_while(|&b| is_ident_char(b) || b == b'-')
            .count()
    } else {
        rem_ident.bytes().take_while(|&b| is_ident_char(b)).count()
    };
    let name = rem_ident.get(..name_len).unwrap_or("");
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let rem_body = masked.get(curr..).unwrap_or("");
    let (body_str, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        (
            clean.get(curr.saturating_add(1)..close_pos),
            close_pos.saturating_add(1),
        )
    } else if rem_body.starts_with(';') {
        (None, curr.saturating_add(1))
    } else {
        return None;
    };

    Some(ParsedRequirementSpan {
        name,
        doc,
        body_str,
        next_idx,
    })
}

/// Applies a single trimmed requirement body statement onto `req`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn apply_requirement_body_line(trimmed: &str, req: &mut RequirementDef) {
    if let Some(val) = parse_assigned_quoted_clause(trimmed, "id")
        .or_else(|| parse_assigned_quoted_clause(trimmed, "req_id"))
    {
        req.req_id = val;
    } else if let Some(val) = parse_assigned_quoted_clause(trimmed, "doc")
        .or_else(|| parse_assigned_quoted_clause(trimmed, "text"))
    {
        req.text = val;
    } else if let Some(after) = strip_kw_at_boundary(trimmed, "assume") {
        if let Some(target) = after.strip_suffix(';') {
            req.assumes.push(target.trim().to_string());
        }
    } else if let Some(after) = strip_kw_at_boundary(trimmed, "require") {
        if let Some(target) = after.strip_suffix(';') {
            req.requires.push(target.trim().to_string());
        }
    } else if let Some(after) = strip_kw_at_boundary(trimmed, "derived") {
        if let Some(after_from) = strip_kw_at_boundary(after, "from") {
            if let Some(target) = after_from.strip_suffix(';') {
                req.derived_from.push(target.trim().to_string());
            }
        }
    } else if let Some(after) = strip_kw_at_boundary(trimmed, "verify") {
        if let Some(target) = parse_optional_by_target(after) {
            req.verified_by.push(target);
        }
    } else if let Some(after) = strip_kw_at_boundary(trimmed, "satisfy") {
        if let Some(target) = parse_optional_by_target(after) {
            req.satisfied_by.push(target);
        }
    }
}

/// Populates a `RequirementDef` from its parsed header span and optional inner block body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn build_requirement_from_span(span: ParsedRequirementSpan<'_>) -> RequirementDef {
    let mut req = RequirementDef {
        name: span.name.to_string(),
        doc: span.doc,
        ..Default::default()
    };
    if let Some(body) = span.body_str {
        if req.doc.is_none() {
            req.doc = extract_inline_doc_block(body);
        }
        for line in body.lines() {
            apply_requirement_body_line(line.trim(), &mut req);
        }
        req.attributes = parse_attribute_defs(body);
        req.constraints = parse_constraint_defs(body);
        let (th, sr, lyr) = parse_safety_and_layer_annotations(body);
        req.triggers_hazards = th;
        req.safety_realises = sr;
        req.layer = lyr;
    }
    if req.req_id.is_empty() {
        req.req_id = span.name.to_string();
    }
    req
}

/// Parses `RequirementDef` definitions and requirement usages from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
/// Contract: Extracts every `requirement def` and `requirement` declaration in appearance order while skipping `satisfy requirement <Id>;` clauses, comments, and string literals.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; malformed or unclosed blocks are skipped without panicking.
pub fn parse_requirement_defs(content: &str) -> Vec<RequirementDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut reqs = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("requirement")) {
        let req_pos = idx.saturating_add(pos);
        let after_kw = req_pos.saturating_add(11);
        if !is_word_boundary_before(&masked, req_pos)
            || !is_word_boundary_after(&masked, after_kw)
            || is_preceded_by_satisfy_keyword(&masked, req_pos)
        {
            idx = after_kw;
            continue;
        }
        if let Some(span) = parse_requirement_span(&clean, &masked, req_pos) {
            idx = span.next_idx;
            reqs.push(build_requirement_from_span(span));
        } else {
            idx = after_kw;
        }
    }
    reqs
}

/// Finds the byte offset of the standalone `by` keyword inside a `satisfy` statement slice.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SATISFIED-REQUIREMENTS]
fn find_by_keyword_offset(stmt_clean: &str) -> Option<usize> {
    let mut search_idx: usize = 0;
    while let Some(pos) = stmt_clean.get(search_idx..).and_then(|s| s.find("by")) {
        let actual_pos = search_idx.saturating_add(pos);
        let after_by = actual_pos.saturating_add(2);
        if is_word_boundary_before(stmt_clean, actual_pos)
            && is_word_boundary_after(stmt_clean, after_by)
        {
            return Some(actual_pos);
        }
        search_idx = after_by;
    }
    None
}

/// Extracts and unquotes the target requirement identifier from a single `satisfy` clause body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SATISFIED-REQUIREMENTS]
fn extract_satisfied_req_id_from_stmt(stmt_clean: &str) -> Option<String> {
    if stmt_clean.contains('{') || stmt_clean.contains('}') {
        return None;
    }
    let target_part = if let Some(by_pos) = find_by_keyword_offset(stmt_clean) {
        stmt_clean.get(..by_pos).unwrap_or("").trim()
    } else {
        stmt_clean
    };
    let req_id = if target_part.starts_with('"') {
        if let Some(unquoted) = extract_quoted_string(target_part) {
            unquoted.trim().to_string()
        } else {
            target_part.trim_matches('"').trim().to_string()
        }
    } else if target_part.starts_with('\'') {
        target_part.trim_matches('\'').trim().to_string()
    } else {
        target_part.trim().to_string()
    };
    if req_id.is_empty() {
        None
    } else {
        Some(req_id)
    }
}

/// Parses unique `satisfy [requirement] <ReqId> [by <Part>];` bindings from SysML v2 container bodies.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SATISFIED-REQUIREMENTS]
/// Contract: Preserves order of first appearance, deduplicates repeated bindings, and ignores inverse `satisfy by <Part>;` clauses inside requirement blocks.
/// Bounds & Faults: Executes in O(N * K) time over input byte length N and distinct requirement IDs K using checked `.get` slicing and saturating arithmetic; unclosed statements are skipped safely.
pub fn parse_satisfied_requirements(content: &str) -> Vec<String> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut satisfied = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("satisfy")) {
        let satisfy_pos = idx.saturating_add(pos);
        let after_kw = satisfy_pos.saturating_add(7);
        if !is_word_boundary_before(&masked, satisfy_pos)
            || !is_word_boundary_after(&masked, after_kw)
        {
            idx = after_kw;
            continue;
        }

        let mut curr = after_kw.saturating_add(skip_ws(masked.get(after_kw..).unwrap_or("")));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("by") && is_word_boundary_after(s, 2))
        {
            idx = after_kw;
            continue;
        }
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("requirement") && is_word_boundary_after(s, 11))
        {
            curr = curr.saturating_add(11);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }

        let rem_masked = masked.get(curr..).unwrap_or("");
        let rem_clean = clean.get(curr..).unwrap_or("");
        if let Some(semi_pos) = rem_masked.find(';') {
            let stmt_clean = rem_clean.get(..semi_pos).unwrap_or("").trim();
            if let Some(req_id) = extract_satisfied_req_id_from_stmt(stmt_clean) {
                if !satisfied.contains(&req_id) {
                    satisfied.push(req_id);
                }
            }
            idx = curr.saturating_add(semi_pos).saturating_add(1);
        } else {
            idx = after_kw;
        }
    }

    satisfied
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `parse_requirement_defs` extracts `verify [by]`, `satisfy [by]`, annotations, UTF-8 text, and skips malformed blocks.
    #[test]
    fn parse_requirement_defs_extracts_verify_satisfy_annotations_and_skips_malformed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            /* Ignore requirement def CommentedReq; and UTF-8 Δt <= 5 ms */
            attribute literal : String = "requirement def FakeInString { id = \"FAKE\"; }";
            satisfy requirement REQ-IGNORED-SATISFY;

            requirement def REQ-AV-101 {
                req_id = "REQ-AV-101";
                doc = "Actuator loop latency Δt <= 5 ms.";
                doc /* Normative inline safety requirement doc */
                assume SensorClockLocked;
                require EnvelopeInvariant;
                derived from SYS-REQ-001;
                verify by TestActuatorLatency;
                verify BenchMonitorCase;
                satisfy by PrimaryActuatorController;
                satisfy RedundantActuatorLane;
                @TriggersHazard('H-01', 'H-02')
                @SafetyRealises = 'SC-01';
                @Layer('Control')
            }

            requirement BareUsageReq;

            // Malformed declarations must be skipped without panicking
            requirement def ;
            requirement UnclosedBlock { id = "BROKEN";
            requirement MissingTerminator
        "#;

        let reqs = parse_requirement_defs(src);
        assert_eq!(reqs.len(), 2);

        let r0 = reqs.first().ok_or("missing REQ-AV-101")?;
        assert_eq!(r0.name, "REQ-AV-101");
        assert_eq!(r0.req_id, "REQ-AV-101");
        assert_eq!(r0.text, "Actuator loop latency Δt <= 5 ms.");
        assert_eq!(
            r0.doc.as_deref(),
            Some("Normative inline safety requirement doc")
        );
        assert_eq!(r0.assumes, vec!["SensorClockLocked"]);
        assert_eq!(r0.requires, vec!["EnvelopeInvariant"]);
        assert_eq!(r0.derived_from, vec!["SYS-REQ-001"]);
        assert_eq!(
            r0.verified_by,
            vec!["TestActuatorLatency", "BenchMonitorCase"]
        );
        assert_eq!(
            r0.satisfied_by,
            vec!["PrimaryActuatorController", "RedundantActuatorLane"]
        );
        assert_eq!(r0.triggers_hazards, vec!["H-01", "H-02"]);
        assert_eq!(r0.safety_realises, vec!["SC-01"]);
        assert_eq!(r0.layer.as_deref(), Some("Control"));

        let r1 = reqs.get(1).ok_or("missing BareUsageReq")?;
        assert_eq!(r1.name, "BareUsageReq");
        assert_eq!(r1.req_id, "BareUsageReq");
        Ok(())
    }

    /// Verifies `parse_satisfied_requirements` handles single-quoted, double-quoted, `by` clauses, inverse `satisfy by`, and unclosed statements.
    #[test]
    fn parse_satisfied_requirements_handles_quotes_by_clauses_and_malformed_inputs() {
        let src = r#"
            part def GuidanceController {
                satisfy requirement 'REQ-GC-01';
                satisfy "REQ-GC-02" by InnerLoop;
                satisfy by InverseClauseIgnored;
                satisfy requirement BracedIgnored { doc /* not a ref */ };
                satisfy requirement UnclosedWithoutSemicolon
            }
        "#;
        let ids = parse_satisfied_requirements(src);
        assert_eq!(ids, vec!["REQ-GC-01".to_string(), "REQ-GC-02".to_string()]);
    }
}

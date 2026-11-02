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
use crate::sysml_cst::lexer::SysmlToken;
use logos::Logos;
use serde::{Deserialize, Serialize};

/// Represents a normative requirement definition or requirement usage in SysML v2.
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

/// Extracts quoted literals, parenthesized argument lists, or `= value` tokens from an `@Annotation` line.
/// Realises: [REQ-SYSML-AST-ANNOTATION-VALUES]
fn extract_annotation_values(after_tag: &str) -> Vec<String> {
    let mut values: Vec<String> = SysmlToken::lexer(after_tag)
        .spanned()
        .filter_map(|(tok, span)| match tok {
            Ok(SysmlToken::StringLit) => after_tag
                .get(span)
                .and_then(extract_quoted_string)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            _ => None,
        })
        .collect();
    if !values.is_empty() {
        return values;
    }
    let trimmed = after_tag.trim();
    if let Some((inner, _)) = trimmed.strip_prefix('(').and_then(|s| s.split_once(')')) {
        values.extend(
            inner
                .split(',')
                .map(|p| p.trim().trim_matches('\'').trim())
                .filter(|s| !s.is_empty())
                .map(str::to_string),
        );
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

    for trimmed in body.lines().map(str::trim) {
        if let Some(after) = trimmed.strip_prefix("@TriggersHazard") {
            for v in extract_annotation_values(after) {
                if !triggers_hazards.contains(&v) {
                    triggers_hazards.push(v);
                }
            }
        } else if let Some(after) = trimmed.strip_prefix("@SafetyRealises") {
            for v in extract_annotation_values(after) {
                if !safety_realises.contains(&v) {
                    safety_realises.push(v);
                }
            }
        } else if let Some(after) = trimmed.strip_prefix("@Layer") {
            layer = extract_annotation_values(after).into_iter().next().or(layer);
        }
    }

    (triggers_hazards, safety_realises, layer)
}

/// Extracts the first inline `doc /* ... */` documentation block inside a declaration body when non-empty.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
pub(super) fn extract_inline_doc_block(body: &str) -> Option<String> {
    let mut prev_doc = false;
    for (tok, span) in SysmlToken::lexer(body)
        .spanned()
        .filter(|(t, _)| !matches!(t, Ok(SysmlToken::Whitespace)))
    {
        let slice = body.get(span).unwrap_or("");
        if prev_doc {
            if let Some(clean_doc) = slice
                .strip_prefix("/*")
                .and_then(|s| s.strip_suffix("*/"))
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                return Some(clean_doc.to_string());
            }
        }
        prev_doc = slice == "doc" && tok.is_ok();
    }
    None
}

/// Strips a leading keyword `kw` at a SysML v2 word boundary and trims leading whitespace.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn strip_kw_at_boundary<'a>(s: &'a str, kw: &str) -> Option<&'a str> {
    (s.starts_with(kw) && is_word_boundary_after(s, kw.len()))
        .then(|| s.get(kw.len()..).unwrap_or("").trim_start())
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
    let after_eq = strip_kw_at_boundary(trimmed, kw)?.strip_prefix('=')?;
    extract_quoted_string(after_eq.trim_start()).map(str::to_string)
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
    let name_len = rem_ident
        .bytes()
        .take_while(|&b| is_ident_char(b) || (has_def && b == b'-'))
        .count();
    let name = rem_ident.get(..name_len).unwrap_or("");
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let rem_body = masked.get(curr..).unwrap_or("");
    let (body_str, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        (clean.get(curr.saturating_add(1)..close_pos), close_pos.saturating_add(1))
    } else if rem_body.starts_with(';') {
        (None, curr.saturating_add(1))
    } else {
        return None;
    };

    Some(ParsedRequirementSpan { name, doc, body_str, next_idx })
}

/// Applies a single trimmed requirement body statement onto `req`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
fn apply_requirement_body_line(trimmed: &str, req: &mut RequirementDef) {
    let parse_by = |s: &str| {
        strip_kw_at_boundary(s, "by")
            .unwrap_or(s)
            .strip_suffix(';')
            .map(|t| t.trim().to_string())
    };
    if let Some(val) = parse_assigned_quoted_clause(trimmed, "id")
        .or_else(|| parse_assigned_quoted_clause(trimmed, "req_id"))
    {
        req.req_id = val;
    } else if let Some(val) = parse_assigned_quoted_clause(trimmed, "doc")
        .or_else(|| parse_assigned_quoted_clause(trimmed, "text"))
    {
        req.text = val;
    } else if let Some(target) =
        strip_kw_at_boundary(trimmed, "assume").and_then(|s| s.strip_suffix(';'))
    {
        req.assumes.push(target.trim().to_string());
    } else if let Some(target) =
        strip_kw_at_boundary(trimmed, "require").and_then(|s| s.strip_suffix(';'))
    {
        req.requires.push(target.trim().to_string());
    } else if let Some(target) = strip_kw_at_boundary(trimmed, "derived")
        .and_then(|s| strip_kw_at_boundary(s, "from"))
        .and_then(|s| s.strip_suffix(';'))
    {
        req.derived_from.push(target.trim().to_string());
    } else if let Some(target) = strip_kw_at_boundary(trimmed, "verify").and_then(parse_by) {
        req.verified_by.push(target);
    } else if let Some(target) = strip_kw_at_boundary(trimmed, "satisfy").and_then(parse_by) {
        req.satisfied_by.push(target);
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
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
/// Contract: Extracts every `requirement def` and `requirement` declaration in appearance order while skipping `satisfy requirement <Id>;` clauses, comments, and string literals.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; malformed or unclosed blocks are skipped without panicking.
pub fn parse_requirement_defs(content: &str) -> Vec<RequirementDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut reqs = Vec::new();
    let mut skip_until: usize = 0;

    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until
            || res.is_err()
            || masked.get(span.clone()) != Some("requirement")
            || is_preceded_by_satisfy_keyword(&masked, span.start)
        {
            continue;
        }
        if let Some(req_span) = parse_requirement_span(&clean, &masked, span.start) {
            skip_until = req_span.next_idx;
            reqs.push(build_requirement_from_span(req_span));
        }
    }
    reqs
}

/// Extracts and unquotes the target requirement identifier from a single `satisfy` clause body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SATISFIED-REQUIREMENTS]
fn extract_satisfied_req_id_from_stmt(stmt_clean: &str) -> Option<String> {
    if stmt_clean.contains('{') || stmt_clean.contains('}') {
        return None;
    }
    let by_pos = SysmlToken::lexer(stmt_clean)
        .spanned()
        .find_map(|(tok, sp)| (tok.is_ok() && stmt_clean.get(sp.clone()) == Some("by")).then_some(sp.start));
    let target_part = by_pos
        .and_then(|pos| stmt_clean.get(..pos))
        .unwrap_or(stmt_clean)
        .trim();
    let req_id = if target_part.starts_with('"') {
        extract_quoted_string(target_part)
            .unwrap_or_else(|| target_part.trim_matches('"'))
            .trim()
            .to_string()
    } else if target_part.starts_with('\'') {
        target_part.trim_matches('\'').trim().to_string()
    } else {
        target_part.trim().to_string()
    };
    (!req_id.is_empty()).then_some(req_id)
}

/// Parses unique `satisfy [requirement] <ReqId> [by <Part>];` bindings from SysML v2 container bodies.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SATISFIED-REQUIREMENTS]
/// Contract: Preserves order of first appearance, deduplicates repeated bindings, and ignores inverse `satisfy by <Part>;` clauses inside requirement blocks.
/// Bounds & Faults: Executes in O(N * K) time over input byte length N and distinct requirement IDs K using checked `.get` slicing and saturating arithmetic; unclosed statements are skipped safely.
pub fn parse_satisfied_requirements(content: &str) -> Vec<String> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut satisfied = Vec::new();
    let mut skip_until: usize = 0;

    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until || res.is_err() || masked.get(span.clone()) != Some("satisfy") {
            continue;
        }
        let after_kw = span.end;
        let mut curr = after_kw.saturating_add(skip_ws(masked.get(after_kw..).unwrap_or("")));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("by") && is_word_boundary_after(s, 2))
        {
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
            skip_until = curr.saturating_add(semi_pos).saturating_add(1);
        }
    }

    satisfied
}

#[cfg(test)]
mod tests {
    use super::*;

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
            requirement def ;
            requirement UnclosedBlock { id = "BROKEN";
            requirement MissingTerminator
        "#;

        let reqs = parse_requirement_defs(src);
        assert_eq!(reqs.len(), 2);

        let r0 = reqs.first().ok_or("missing REQ-AV-101")?;
        assert_eq!((r0.name.as_str(), r0.req_id.as_str(), r0.text.as_str()), ("REQ-AV-101", "REQ-AV-101", "Actuator loop latency Δt <= 5 ms."));
        assert_eq!(r0.doc.as_deref(), Some("Normative inline safety requirement doc"));
        assert_eq!((r0.assumes.as_slice(), r0.requires.as_slice(), r0.derived_from.as_slice()), (&["SensorClockLocked".to_string()][..], &["EnvelopeInvariant".to_string()][..], &["SYS-REQ-001".to_string()][..]));
        assert_eq!(r0.verified_by, vec!["TestActuatorLatency", "BenchMonitorCase"]);
        assert_eq!(r0.satisfied_by, vec!["PrimaryActuatorController", "RedundantActuatorLane"]);
        assert_eq!(r0.triggers_hazards, vec!["H-01", "H-02"]);
        assert_eq!((r0.safety_realises.as_slice(), r0.layer.as_deref()), (&["SC-01".to_string()][..], Some("Control")));

        let r1 = reqs.get(1).ok_or("missing BareUsageReq")?;
        assert_eq!((r1.name.as_str(), r1.req_id.as_str()), ("BareUsageReq", "BareUsageReq"));
        Ok(())
    }

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
        assert_eq!(parse_satisfied_requirements(src), vec!["REQ-GC-01".to_string(), "REQ-GC-02".to_string()]);
    }
}

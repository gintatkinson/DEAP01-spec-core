//! STPA hazard, safety risk, and FMECA component-matching and negation AST models and parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-SAFETY-AND-FMECA], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `HazardDef` and `RiskDef`, provides deterministic zero-regex `parse_hazard_defs` extraction, and implements symmetric FMECA table-to-AST `component_matches` alignment and explicit subsystem negation analysis (`extract_negated_components`, `is_component_negated`).
//! Bounds & Faults: Executes in linear or bounded polynomial time using `BTreeSet` ordering, bounds-checked `.get` slicing, saturating offset arithmetic, and zero panics on malformed or multi-byte UTF-8 inputs.

use super::identifiers::{
    default_one, extract_doc_comment_before, find_matching_brace, is_ident_char,
    is_word_boundary_after, mask_string_literals, skip_ws,
};
use super::interfaces_and_items::parse_attribute_defs;
use super::requirements::extract_inline_doc_block;
use super::strip_sysml_comments;
use crate::sysml_cst::lexer::SysmlToken;
use logos::Logos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Checks whether a lowercase token is an explicit negative presence literal (`0`, `false`, `no`, `none`).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-NEGATED-COMPONENTS]
fn is_negative_literal(val: &str) -> bool {
    matches!(val, "0" | "false" | "\"no\"" | "\"none\"" | "no" | "none")
}

/// Checks whether a lowercase token is a leading SysML v2 modifier keyword stripped from negated labels.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-LABEL-NORMALIZATION]
fn is_negated_label_keyword(word: &str) -> bool {
    matches!(
        word,
        "attribute" | "ref" | "readonly" | "derived" | "constant" | "in" | "out" | "inout" | "def"
    )
}

/// Checks whether a lowercase word is a structural component qualifier in `no <component> <qualifier>` prose.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-PROSE-PHRASES]
fn is_prose_component_qualifier(word: &str) -> bool {
    matches!(
        word,
        "system" | "subsystem" | "unit" | "module" | "device" | "installed" | "present"
    )
}

/// Strips one trailing component presence qualifier suffix (`subsystem` before `system`) when `>= 3` stem bytes remain.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-KEY-STEM]
fn strip_component_qualifier_suffix(stem: &str) -> Option<&str> {
    [
        "subsystem", "deployment", "controller", "installed", "actuator", "enabled", "present",
        "system", "module", "device", "sensor", "unit",
    ]
    .into_iter()
    .find_map(|sfx| stem.strip_suffix(sfx))
    .filter(|rest| rest.len() >= 3)
}

/// Normalizes a string to its ASCII-lowercased alphanumeric characters.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COMPONENT-MATCHES]
fn alnum_lower(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Checks whether a stripped stem names a scalar or metrology quantity (`time`, `rate`, `count`, etc.).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-KEY-STEM]
fn is_non_component_stem(stem: &str) -> bool {
    matches!(
        stem,
        "time" | "base" | "scale" | "measure" | "count" | "rate" | "limit" | "value" | "mode"
            | "state" | "status" | "index" | "size"
    )
}

/// Represents a system-level safety hazard definition (`hazard def`) in SysML v2 / STPA models.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-HAZARDDEF]
/// Contract: Captures hazard identifier `name`, optional `doc` rationale, MIL-STD-882E / ISO 26262 `severity` (defaulting to `1`), and optional `source_port`, `target_port`, and `part_ref` bindings.
/// Bounds & Faults: O(1) structural record construction with deterministic `Default` and `serde` wire defaults (`severity = 1`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HazardDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default = "default_one")]
    pub severity: i32,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
    pub part_ref: Option<String>,
}

/// Represents a safety risk assessment definition in SysML v2 / STPA / FMECA models.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-RISKDEF]
/// Contract: Captures risk `name`, optional `doc` metadata, `severity` rating (defaulting to `1`), linked `hazard_ref`, and optional `source_port` and `target_port` bindings.
/// Bounds & Faults: O(1) structural record construction with deterministic `Default` and `serde` wire defaults (`severity = 1`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RiskDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default = "default_one")]
    pub severity: i32,
    pub hazard_ref: Option<String>,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
}

/// Parses an integer `severity = <int>;` override inside a `hazard def` body, defaulting to `1`.
/// Realises: [REQ-SYSML-AST-PARSE-HAZARD-DEFS]
fn parse_hazard_severity(body: &str) -> i32 {
    body.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("severity")
                .and_then(|s| s.split_once('='))
                .and_then(|(_, rhs)| rhs.trim().trim_end_matches(';').trim().parse::<i32>().ok())
        })
        .next_back()
        .unwrap_or(1)
}

/// Parses a single `hazard def <Name> ({ ... } | ;)` declaration beginning at byte offset `haz_pos`.
/// Realises: [REQ-SYSML-AST-PARSE-HAZARD-DEFS]
fn parse_single_hazard(clean: &str, masked: &str, haz_pos: usize) -> Option<(HazardDef, usize)> {
    let doc_before = extract_doc_comment_before(clean, haz_pos);
    let mut curr = haz_pos.saturating_add(6);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if !masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        return None;
    }
    curr = curr.saturating_add(3);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let rem_ident = masked.get(curr..).unwrap_or("");
    let name_len = rem_ident
        .bytes()
        .take_while(|&b| is_ident_char(b) || b == b'-')
        .count();
    let name = rem_ident.get(..name_len).unwrap_or("");
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let rem_body = masked.get(curr..).unwrap_or("");
    let (body_opt, next_idx) = if rem_body.starts_with('{') {
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

    let doc = doc_before.or_else(|| body_opt.and_then(extract_inline_doc_block));
    let severity = body_opt.map_or(1, parse_hazard_severity);
    Some((
        HazardDef {
            name: name.to_string(),
            doc,
            severity,
            ..HazardDef::default()
        },
        next_idx,
    ))
}

/// Parses STPA `hazard def <Name>` declarations from a SysML v2 source text fragment.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-PARSE-HAZARD-DEFS]
/// Contract: Extracts every `hazard def` node in appearance order, supporting hyphenated IDs (`H-01`), preceding or inline `doc /* ... */` blocks, and `severity = <int>;` overrides (defaulting to `1`).
/// Bounds & Faults: Executes in O(N) time and O(H) output space over `content.len()` using bounds-checked `.get` slicing and saturating arithmetic; malformed or unclosed blocks are skipped without panicking.
pub fn parse_hazard_defs(content: &str) -> Vec<HazardDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut hazards = Vec::new();
    let mut skip_until: usize = 0;

    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until || res.is_err() || masked.get(span.clone()) != Some("hazard") {
            continue;
        }
        if let Some((hazard, next_idx)) = parse_single_hazard(&clean, &masked, span.start) {
            hazards.push(hazard);
            skip_until = next_idx;
        }
    }
    hazards
}

/// Checks whether `needle` appears as a distinct alphanumeric word inside `haystack` (case-insensitive).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/CONTAINS-WORD]
fn contains_word(haystack: &str, needle: &str) -> bool {
    let (h, n) = (haystack.to_lowercase(), needle.to_lowercase());
    !n.is_empty()
        && h.match_indices(&n).any(|(pos, _)| {
            let before_ok = h
                .get(..pos)
                .and_then(|s| s.chars().next_back())
                .is_none_or(|c| !c.is_alphanumeric());
            let after_ok = h
                .get(pos.saturating_add(n.len())..)
                .and_then(|s| s.chars().next())
                .is_none_or(|c| !c.is_alphanumeric());
            before_ok && after_ok
        })
}

/// Checks whether an FMECA table component label matches a SysML v2 AST `part def` identifier.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/COMPONENT-MATCHES]
/// Contract: Symmetric relation (`component_matches(a, b) == component_matches(b, a)`) matching case-insensitively on trimmed text, alphanumeric-normalized stems, or bidirectional word-boundary containment.
/// Bounds & Faults: Executes in O(|A| * |B|) time using UTF-8 character-boundary steps; empty or punctuation-only strings never spuriously match non-empty identifiers.
pub fn component_matches(table_comp: &str, ast_part_name: &str) -> bool {
    let tc = table_comp.trim();
    let pn = ast_part_name.trim();
    if tc.eq_ignore_ascii_case(pn) {
        return true;
    }

    let tc_clean = alnum_lower(tc);
    let pn_clean = alnum_lower(pn);
    (!tc_clean.is_empty() && tc_clean == pn_clean)
        || contains_word(tc, pn)
        || contains_word(pn, tc)
}

/// Checks whether the right-hand side of an assignment or prose label evaluates to a negative presence token.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-NEGATED-COMPONENTS]
fn is_negative_rhs_value(rhs: &str) -> bool {
    let val = rhs
        .trim()
        .trim_matches([';', '}', '{', ',', ')', '(', '.'])
        .trim();
    is_negative_literal(val)
}

/// Extracts the normalized lowercase alphanumeric stem from an explicit `!exists(<Part>)` predicate on `lower`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-NEGATED-COMPONENTS]
fn extract_exists_negation_stem(lower: &str) -> Option<String> {
    let (_, after) = lower.split_once("!exists(")?;
    let (inner, _) = after.split_once(')')?;
    let comp = alnum_lower(inner);
    (!comp.is_empty()).then_some(comp)
}

/// Extracts a negated component stem from a single `,`- or `;`-delimited assignment or labelled prose clause.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-NEGATED-COMPONENTS]
fn extract_clause_negation_stem(clause: &str) -> Option<String> {
    let clause = clause.trim();
    let (lhs, rhs) = [":=", "==", "=", ":"]
        .into_iter()
        .find_map(|sep| clause.split_once(sep))?;
    if !is_negative_rhs_value(&rhs.to_ascii_lowercase()) {
        return None;
    }
    lhs.rsplit(':')
        .map(|seg| extract_negated_key_stem(&normalize_negated_label(seg)))
        .find(|stem| !stem.is_empty())
}

/// Extracts sorted, deduplicated component stems explicitly negated in AST attributes, constraints, or specification prose.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-NEGATED-COMPONENTS]
/// Contract: Recognizes `<subsystem>Enabled = 0`, `has<Subsystem> = false`, `<subsystem>System = "No"`, `!exists(<Part>)`, and `no <component> <qualifier>` phrases while ignoring numeric state counters and metrology units.
/// Bounds & Faults: Executes in O(N log K) time over input byte length N and distinct stems K backed by `BTreeSet`; never panics on arbitrary UTF-8 input.
pub fn extract_negated_components(content: &str) -> Vec<String> {
    let mut negated = BTreeSet::new();

    for attr in parse_attribute_defs(content) {
        if attr
            .default_value
            .as_deref()
            .is_some_and(|v| is_negative_literal(&v.trim().to_ascii_lowercase()))
        {
            let stem = extract_negated_key_stem(&attr.name);
            if !stem.is_empty() {
                negated.insert(stem);
            }
        }
    }

    for line in content.lines() {
        let comment_pos = SysmlToken::lexer(line)
            .spanned()
            .find_map(|(tok, sp)| matches!(tok, Ok(SysmlToken::LineComment)).then_some(sp.start));
        let line_clean = comment_pos.and_then(|idx| line.get(..idx)).unwrap_or(line);
        let lower = line_clean.trim().to_lowercase();
        if let Some(comp) = extract_exists_negation_stem(&lower) {
            negated.insert(comp);
        }
        for clause in line_clean.split([',', ';']) {
            if let Some(stem) = extract_clause_negation_stem(clause) {
                negated.insert(stem);
            }
        }
        negated.extend(extract_no_phrase_stems(&lower));
    }

    negated.into_iter().collect()
}

/// Normalizes the left-hand side of a negating assignment or prose label into a single joined key token.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-LABEL-NORMALIZATION]
fn normalize_negated_label(lhs: &str) -> String {
    let head = lhs.split(':').next().unwrap_or(lhs);
    let head = head.rsplit(['{', '(', ';', ',']).next().unwrap_or(head);
    let head = head.rsplit('.').next().unwrap_or(head);
    let words: Vec<String> = head
        .split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        })
        .filter(|w| !w.is_empty() && !w.chars().all(|c| c.is_ascii_digit()))
        .collect();
    let start = words
        .iter()
        .take_while(|w| is_negated_label_keyword(&w.to_ascii_lowercase()))
        .count();
    let label_words = words.get(start..).unwrap_or(&[]);
    if label_words.is_empty() || label_words.len() > 4 {
        String::new()
    } else {
        label_words.join("")
    }
}

/// Extracts component stems from qualified prose negations of the form `no <component> <qualifier>`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-PROSE-PHRASES]
fn extract_no_phrase_stems(lower: &str) -> Vec<String> {
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
        .collect();
    let mut stems = Vec::new();
    for (i, _) in words.iter().enumerate().filter(|(_, w)| **w == "no") {
        let mut comp = String::new();
        let mut qualifier: Option<&str> = None;
        for next in words.iter().skip(i.saturating_add(1)).take(3) {
            if is_prose_component_qualifier(next) {
                qualifier = Some(*next);
                break;
            }
            if next.chars().all(|c| c.is_ascii_digit()) {
                break;
            }
            comp.push_str(next);
        }
        if let Some(qual) = qualifier.filter(|_| !comp.is_empty()) {
            let stem = extract_negated_key_stem(&format!("{comp}{qual}"));
            if !stem.is_empty() {
                stems.push(stem);
            }
        }
    }
    stems
}

/// Extracts the normalized lowercase alphanumeric component stem from a qualified negation key.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-KEY-STEM]
pub(super) fn extract_negated_key_stem(key: &str) -> String {
    let clean = alnum_lower(key);
    if clean.is_empty() {
        return String::new();
    }
    let mut stem: &str = clean.as_str();
    let mut qualified = false;
    let raw = key.trim_start_matches(|c: char| !c.is_alphanumeric());
    let has_boundary = raw.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("has"))
        && raw
            .get(3..)
            .and_then(|r| r.chars().next())
            .is_some_and(|c| c == '_' || c.is_ascii_uppercase());
    if has_boundary {
        if let Some(rest) = clean.strip_prefix("has").filter(|rest| rest.len() >= 3) {
            stem = rest;
            qualified = true;
        }
    }
    while let Some(rest) = strip_component_qualifier_suffix(stem) {
        stem = rest;
        qualified = true;
    }
    if qualified && stem.len() >= 3 && !is_non_component_stem(stem) {
        stem.to_string()
    } else {
        String::new()
    }
}

/// Checks whether a candidate component name is contradicted by any negative AST attribute or constraint in `content`.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/IS-COMPONENT-NEGATED]
/// Contract: Returns `true` iff `comp` matches at least one negated component stem extracted by `extract_negated_components(content)`.
/// Bounds & Faults: Executes in O(N + S * |C|) time; returns `false` cleanly when `comp` or `content` is empty.
pub fn is_component_negated(comp: &str, content: &str) -> bool {
    component_matches_negated_stems(comp, &extract_negated_components(content))
}

/// Checks whether a component name contains or structurally matches any of the supplied negated stems.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-STEM-MATCH]
pub(super) fn component_matches_negated_stems(comp: &str, stems: &[String]) -> bool {
    let comp_clean = alnum_lower(comp);
    !comp_clean.is_empty()
        && stems
            .iter()
            .any(|neg_stem| comp_clean.contains(neg_stem.as_str()) || component_matches(comp, neg_stem))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hazard_defs_extracts_severity_docs_and_skips_malformed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            /* Ignore hazard def H-COMMENT; */
            attribute msg : String = "hazard def H-STRING;";
            doc /* Preceding hazard doc for H-01 */
            hazard def H-01 { severity = 4; }
            hazard def H-02 {
                doc /* Inline hazard doc with UTF-8 Ω */
                severity = 2;
            }
            hazard def H-03;
            hazard MissingDefKeyword { severity = 3; }
            hazard def ;
            hazard def H-UNCLOSED { severity = 5;
            hazard def H-NO-TERM
        "#;

        let hazards = parse_hazard_defs(src);
        assert_eq!(hazards.len(), 3);

        let h0 = hazards.first().ok_or("missing H-01")?;
        assert_eq!((h0.name.as_str(), h0.doc.as_deref(), h0.severity), ("H-01", Some("Preceding hazard doc for H-01"), 4));

        let h1 = hazards.get(1).ok_or("missing H-02")?;
        assert_eq!((h1.name.as_str(), h1.doc.as_deref(), h1.severity), ("H-02", Some("Inline hazard doc with UTF-8 Ω"), 2));

        let h2 = hazards.get(2).ok_or("missing H-03")?;
        assert_eq!((h2.name.as_str(), h2.doc.as_deref(), h2.severity), ("H-03", None, 1));
        Ok(())
    }

    #[test]
    fn hazard_and_risk_serde_defaults_and_utf8_component_matching() -> Result<(), serde_json::Error>
    {
        let haz: HazardDef = serde_json::from_str(r#"{"name":"H-10"}"#)?;
        let risk: RiskDef = serde_json::from_str(r#"{"name":"R-10","hazard_ref":"H-10"}"#)?;
        assert_eq!((haz.severity, risk.severity, risk.hazard_ref.as_deref()), (1, 1, Some("H-10")));
        assert!(component_matches("Ω Sensor Module", "Sensor") && component_matches("Sensor", "Ω Sensor Module"));
        assert!(!component_matches("---", "Sensor"));
        Ok(())
    }
}

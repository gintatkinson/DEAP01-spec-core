//! Parametric constraint, assertion, and mathematical calculation AST definitions and parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-EXPRESSIONS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `ConstraintDef` (formal invariants, `assert constraint` declarations, parameters, expressions, and pre/post-conditions) and `CalcDef` (typed `calc` / `calc def` mathematical definitions) together with deterministic, zero-regex `parse_constraint_defs` and `parse_calc_defs` extractors.
//! Bounds & Faults: Executes in linear O(N) time and O(N) space over comment-stripped buffers using bounds-checked `.get` slicing, saturating offset arithmetic, and zero panics on malformed or multi-byte UTF-8 inputs.

use super::identifiers::{
    extract_doc_comment_before, extract_ident, extract_type_ident, find_matching_brace,
    is_word_boundary_after, is_word_boundary_before, mask_string_literals, skip_ws, AttributeDef,
};
use super::strip_sysml_comments;
use crate::sysml_cst::lexer::SysmlToken;
use logos::Logos;
use serde::{Deserialize, Serialize};

/// Represents a formal parametric constraint definition, invariant, or `assert constraint` in SysML v2.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-CONSTRAINTDEF]
/// Contract: Captures constraint `name`, boolean/relational `expression`, signature `parameters`, `is_assertion` modifier flag, optional `doc` block, and structured `pre_conditions` and `post_conditions`.
/// Bounds & Faults: O(1) default construction and O(P + E) clone/serialization cost over parameter count and expression byte length with deterministic `serde` defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConstraintDef {
    pub name: String,
    pub expression: String,
    #[serde(default)] pub parameters: Vec<String>,
    #[serde(default)] pub is_assertion: bool,
    pub doc: Option<String>,
    #[serde(default)] pub pre_conditions: Vec<String>,
    #[serde(default)] pub post_conditions: Vec<String>,
}

/// Represents a pure mathematical calculation or function definition in SysML v2 / KerML.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/CALC-DEF]
/// Contract: Encapsulates calculation `name`, optional `doc` metadata, typed `in_params` (`AttributeDef`), optional `return_type`, and optional body `expression` derived directly from SysML v2 AST source.
/// Bounds & Faults: O(1) default construction and O(P + E) clone/serialization cost over input parameters and expression byte length with zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CalcDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)] pub in_params: Vec<AttributeDef>,
    #[serde(default)] pub return_type: Option<String>,
    #[serde(default)] pub expression: Option<String>,
}

/// Parses `[def] <name> [(...)]` starting at `after_kw`, returning `(name, optional_paren_slice, post_ws_offset)`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-EXPRESSION-PARAMS]
fn parse_decl_header_with_parens<'a>(clean: &'a str, masked: &'a str, after_kw: usize) -> Option<(&'a str, Option<&'a str>, usize)> {
    let mut curr = after_kw.saturating_add(skip_ws(masked.get(after_kw..).unwrap_or("")));
    if masked.get(curr..).is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3)) {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if !masked.get(curr..).is_some_and(|s| s.starts_with('(')) {
        return Some((name, None, curr));
    }
    let inner_start = curr.saturating_add(1);
    let close_rel = masked.get(inner_start..)?.find(')')?;
    let inner_end = inner_start.saturating_add(close_rel);
    let inner = clean.get(inner_start..inner_end).unwrap_or("");
    let after_paren = inner_end.saturating_add(1);
    let next_curr = after_paren.saturating_add(skip_ws(masked.get(after_paren..).unwrap_or("")));
    Some((name, Some(inner), next_curr))
}

/// Detects an optional `assert` keyword immediately preceding `constraint` at `c_pos` and returns `(is_assertion, doc_end_offset)`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
fn detect_assert_prefix(masked: &str, c_pos: usize) -> (bool, usize) {
    let before_c = masked.get(..c_pos).unwrap_or("").trim_end();
    let assert_start = before_c.len().saturating_sub(6);
    if before_c.ends_with("assert") && is_word_boundary_before(before_c, assert_start) { (true, assert_start) } else { (false, c_pos) }
}

/// Parses a single `[assert] constraint [def] <name> [(...)] ({ ... } | ;)` declaration beginning at `c_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
fn parse_single_constraint(clean: &str, masked: &str, c_pos: usize) -> Option<(ConstraintDef, usize)> {
    let (is_assertion, doc_end) = detect_assert_prefix(masked, c_pos);
    let doc = extract_doc_comment_before(clean, doc_end);
    let (name, paren_opt, curr) = parse_decl_header_with_parens(clean, masked, c_pos.saturating_add(10))?;
    let parameters = paren_opt.into_iter().flat_map(|s| s.split(',')).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
    let rem_body = masked.get(curr..).unwrap_or("");
    let (expression, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        let expr_str = clean.get(curr.saturating_add(1)..close_pos).unwrap_or("");
        (expr_str.trim().trim_end_matches(';').trim().to_string(), close_pos.saturating_add(1))
    } else if rem_body.starts_with(';') {
        (String::new(), curr.saturating_add(1))
    } else {
        return None;
    };
    Some((ConstraintDef { name: name.to_string(), expression, parameters, is_assertion, doc, ..Default::default() }, next_idx))
}

/// Parses parametric `ConstraintDef` definitions and `assert constraint` clauses from SysML v2 source text.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
/// Contract: Extracts every `constraint def`, `constraint`, and `assert constraint` declaration in appearance order while ignoring tokens inside comments or string literals.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; malformed or unclosed blocks are skipped without panicking.
pub fn parse_constraint_defs(content: &str) -> Vec<ConstraintDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut constraints = Vec::new();
    let mut skip_until: usize = 0;

    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until || res.is_err() || masked.get(span.clone()) != Some("constraint") {
            continue;
        }
        if let Some((constraint, next_idx)) = parse_single_constraint(&clean, &masked, span.start) {
            skip_until = next_idx;
            constraints.push(constraint);
        }
    }
    constraints
}

/// Parses a single calculation parameter declaration into an `AttributeDef` (defaulting untyped parameters to `ScalarValues::Real`).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-PARAM]
fn parse_calc_param(s: &str) -> Option<AttributeDef> {
    let mut trimmed = s.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return None;
    }
    for kw in ["in", "attribute"] {
        if trimmed.starts_with(kw) && is_word_boundary_after(trimmed, kw.len()) {
            trimmed = trimmed.get(kw.len()..).unwrap_or("").trim_start();
        }
    }
    let (name, name_len) = extract_ident(trimmed);
    if name.is_empty() {
        return None;
    }
    let after_name = trimmed.get(name_len..).unwrap_or("").trim_start();
    let mut type_name = "ScalarValues::Real".to_string();
    let val_tail = if let Some(after_colon_raw) = after_name.strip_prefix(':') {
        let after_colon = after_colon_raw.trim_start();
        let (t_name, t_len) = extract_type_ident(after_colon);
        if !t_name.is_empty() {
            type_name = t_name.to_string();
        }
        after_colon.get(t_len..).unwrap_or("").trim_start()
    } else {
        after_name
    };
    let default_value = val_tail
        .strip_prefix('=')
        .map(|rhs| rhs.trim().trim_end_matches(';').trim())
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    Some(AttributeDef {
        name: name.to_string(),
        type_name,
        default_value,
        doc: None,
    })
}

/// Checks whether `s` contains any basic infix arithmetic operator (`+`, `-`, `*`, `/`).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn contains_arithmetic_operator(s: &str) -> bool {
    SysmlToken::lexer(s).spanned().any(|(t, span)| {
        matches!(t, Ok(SysmlToken::Operator))
            && s.get(span).is_some_and(|op| matches!(op, "+" | "-" | "*" | "/"))
    })
}

/// Extracts a pure return type identifier from a `return` clause when no arithmetic or call expression is present.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn try_extract_body_return_type(rest: &str) -> Option<String> {
    if contains_arithmetic_operator(rest) || rest.contains('(') {
        return None;
    }
    let has_type_shape =
        rest.contains(':') || rest.starts_with("type ") || rest.split_whitespace().count() <= 2;
    if !has_type_shape {
        return None;
    }
    let type_part = rest
        .split_once(':')
        .map_or_else(|| rest.trim_start_matches("type").trim(), |(_, rhs)| rhs.trim());
    let (t_ident, _) = extract_type_ident(type_part);
    (!t_ident.is_empty()).then(|| t_ident.to_string())
}

/// Extracts body-declared input parameters, inferred return type, and newline-joined expression lines from a `calc` body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn parse_calc_body(
    body: &str,
    in_params: &mut Vec<AttributeDef>,
    return_type: &mut Option<String>,
) -> Option<String> {
    let mut expr_lines = Vec::new();
    for trimmed in body.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let is_param = (trimmed.starts_with("in ")
            || trimmed.starts_with("in\t")
            || trimmed.starts_with("attribute "))
            && !trimmed.contains('=')
            && !contains_arithmetic_operator(trimmed);
        let is_ret = trimmed.starts_with("return ")
            || trimmed.starts_with("return\t")
            || trimmed.starts_with("return:");
        if is_param {
            if let Some(param) = parse_calc_param(trimmed) {
                in_params.push(param);
            }
        } else if is_ret {
            let rest = trimmed.trim_start_matches("return").trim();
            if return_type.is_none() {
                if let Some(parsed_ret) = try_extract_body_return_type(rest) {
                    *return_type = Some(parsed_ret);
                    continue;
                }
            }
            let expr_candidate = rest.trim_start_matches(':').trim().trim_end_matches(';').trim();
            if !expr_candidate.is_empty() {
                expr_lines.push(expr_candidate);
            }
        } else {
            expr_lines.push(trimmed.trim_end_matches(';').trim());
        }
    }
    (!expr_lines.is_empty()).then(|| expr_lines.join("\n"))
}

/// Parses an optional `: <ReturnType>` annotation following a calculation signature parameter list.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn parse_optional_signature_return_type(masked: &str, curr: usize) -> (Option<String>, usize) {
    if !masked.get(curr..).is_some_and(|s| s.starts_with(':')) {
        return (None, curr);
    }
    let after_colon = curr.saturating_add(1);
    let type_start = after_colon.saturating_add(skip_ws(masked.get(after_colon..).unwrap_or("")));
    let (ret_t, ret_len) = extract_type_ident(masked.get(type_start..).unwrap_or(""));
    if ret_t.is_empty() {
        return (None, type_start);
    }
    let after_type = type_start.saturating_add(ret_len);
    let next_curr = after_type.saturating_add(skip_ws(masked.get(after_type..).unwrap_or("")));
    (Some(ret_t.to_string()), next_curr)
}

/// Parses a single `calc [def] <name> [(...)] [: <ReturnType>] ({ ... } | ;)` declaration beginning at `calc_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn parse_single_calc(clean: &str, masked: &str, calc_pos: usize) -> Option<(CalcDef, usize)> {
    let doc = extract_doc_comment_before(clean, calc_pos);
    let (name, paren_opt, curr) =
        parse_decl_header_with_parens(clean, masked, calc_pos.saturating_add(4))?;
    let mut in_params: Vec<AttributeDef> = paren_opt
        .into_iter()
        .flat_map(|s| s.split(','))
        .filter_map(parse_calc_param)
        .collect();
    let (mut return_type, curr) = parse_optional_signature_return_type(masked, curr);
    let rem_body = masked.get(curr..).unwrap_or("");
    let (body, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr);
        let body_slice = close_pos.and_then(|cp| clean.get(curr.saturating_add(1)..cp)).unwrap_or("");
        (Some(body_slice), close_pos.unwrap_or(curr).saturating_add(1))
    } else if rem_body.starts_with(';') {
        (None, curr.saturating_add(1))
    } else {
        return None;
    };
    let expression = body.and_then(|b| parse_calc_body(b, &mut in_params, &mut return_type));
    Some((
        CalcDef {
            name: name.to_string(),
            doc,
            in_params,
            return_type,
            expression,
        },
        next_idx,
    ))
}

/// Parses `CalcDef` mathematical calculation definitions and usages from SysML v2 / KerML source text.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
/// Contract: Extracts every `calc def` and `calc` declaration in appearance order, preserving `doc /* ... */` blocks, signature and body `in`/`attribute` parameters, signature or body `return` types, and result expressions.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; syntax irregularities are skipped safely without panicking.
pub fn parse_calc_defs(content: &str) -> Vec<CalcDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut calcs = Vec::new();
    let mut skip_until: usize = 0;
    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until || res.is_err() || masked.get(span.clone()) != Some("calc") {
            continue;
        }
        if let Some((calc, next_idx)) = parse_single_calc(&clean, &masked, span.start) {
            skip_until = next_idx;
            calcs.push(calc);
        }
    }
    calcs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_and_calc_def_serde_roundtrip() -> Result<(), serde_json::Error> {
        let constraint = ConstraintDef { name: "EnvelopeLimit".to_string(), expression: "airspeed <= 350.0".to_string(), parameters: vec!["airspeed : Real".to_string()], is_assertion: true, doc: Some("Enforces Vne limit.".to_string()), pre_conditions: vec!["sensorValid == true".to_string()], post_conditions: vec!["latchArmed == false".to_string()] };
        assert_eq!(serde_json::from_str::<ConstraintDef>(&serde_json::to_string(&constraint)?)?, constraint);
        let calc_min: CalcDef = serde_json::from_str(r#"{"name":"ComputeLift","doc":null}"#)?;
        assert!(calc_min.name == "ComputeLift" && calc_min.in_params.is_empty() && calc_min.return_type.is_none() && calc_min.expression.is_none());
        Ok(())
    }

    #[test]
    fn parse_constraint_defs_extracts_assertions_docs_and_skips_malformed() -> Result<(), Box<dyn std::error::Error>> {
        let src = "/* Ignore constraint def CommentedOut { x > 0; } and UTF-8 Δv */\nattribute note : String = \"constraint def FakeInString { a == b; }\";\ndoc /* Formal rate bound */\nconstraint def RateBound(omega : Real, maxOmega : Real) { omega <= maxOmega; }\ndoc /* Asserted safety invariant */\nassert constraint ThermalMarginInvariant { tempK < 390.0; }\nassert constraint SemicolonAssertion;\nconstraint def ;\nconstraint BrokenParen(x : Real;\nconstraint NoTerminator\nconstraint UnclosedBrace { x > 0;";
        let parsed = parse_constraint_defs(src);
        let (c0, c1, c2) = (parsed.first().ok_or("c0")?, parsed.get(1).ok_or("c1")?, parsed.get(2).ok_or("c2")?);
        assert_eq!((parsed.len(), c0.name.as_str(), c0.is_assertion, c0.doc.as_deref(), c0.parameters.as_slice(), c0.expression.as_str()), (3, "RateBound", false, Some("Formal rate bound"), &["omega : Real".to_string(), "maxOmega : Real".to_string()][..], "omega <= maxOmega"));
        assert_eq!((c1.name.as_str(), c1.is_assertion, c1.doc.as_deref(), c1.expression.as_str()), ("ThermalMarginInvariant", true, Some("Asserted safety invariant"), "tempK < 390.0"));
        assert!(c2.name == "SemicolonAssertion" && c2.is_assertion && c2.expression.is_empty());
        Ok(())
    }

    #[test]
    fn parse_calc_defs_extracts_signature_and_body_variants_and_skips_malformed() -> Result<(), Box<dyn std::error::Error>> {
        let src = "/* Ignore calc def CommentedCalc; and UTF-8 ρ */\nattribute note : String = \"calc def FakeCalcInString;\";\ndoc /* Dynamic pressure calculation */\ncalc def DynamicPressure(in attribute rho : ScalarValues::Real = 1.225, v = 80.0) : ScalarValues::Real {\n    return 0.5 * rho * v * v;\n}\ncalc def BodyReturnTypeCalc {\n    in x : Real;\n    attribute gain : Real;\n    return type Real;\n    x * gain;\n}\ncalc StubCalc;\ncalc def ;\ncalc UnclosedParen(in x : Real;\ncalc MissingTerminator";
        let calcs = parse_calc_defs(src);
        let (dp, br, stub) = (calcs.first().ok_or("dp")?, calcs.get(1).ok_or("br")?, calcs.get(2).ok_or("stub")?);
        let (p0, p1) = (dp.in_params.first().ok_or("p0")?, dp.in_params.get(1).ok_or("p1")?);
        assert_eq!((calcs.len(), dp.name.as_str(), dp.doc.as_deref(), dp.return_type.as_deref(), dp.expression.as_deref()), (3, "DynamicPressure", Some("Dynamic pressure calculation"), Some("ScalarValues::Real"), Some("0.5 * rho * v * v")));
        assert_eq!((p0.name.as_str(), p0.type_name.as_str(), p0.default_value.as_deref(), p1.name.as_str(), p1.type_name.as_str(), p1.default_value.as_deref()), ("rho", "ScalarValues::Real", Some("1.225"), "v", "ScalarValues::Real", Some("80.0")));
        assert_eq!((br.name.as_str(), br.in_params.len(), br.return_type.as_deref(), br.expression.as_deref()), ("BodyReturnTypeCalc", 2, Some("Real"), Some("x * gain")));
        assert!(stub.name == "StubCalc" && stub.in_params.is_empty() && stub.return_type.is_none() && stub.expression.is_none());
        Ok(())
    }
}

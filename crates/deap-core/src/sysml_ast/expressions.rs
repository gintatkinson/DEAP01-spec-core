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
use serde::{Deserialize, Serialize};

/// Represents a formal parametric constraint definition, invariant, or `assert constraint` in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-CONSTRAINTDEF]
/// Contract: Captures constraint `name`, boolean/relational `expression`, signature `parameters`, `is_assertion` modifier flag, optional `doc` block, and structured `pre_conditions` and `post_conditions`.
/// Bounds & Faults: O(1) default construction and O(P + E) clone/serialization cost over parameter count and expression byte length with deterministic `serde` defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConstraintDef {
    pub name: String,
    pub expression: String,
    #[serde(default)]
    pub parameters: Vec<String>,
    #[serde(default)]
    pub is_assertion: bool,
    pub doc: Option<String>,
    #[serde(default)]
    pub pre_conditions: Vec<String>,
    #[serde(default)]
    pub post_conditions: Vec<String>,
}

/// Represents a pure mathematical calculation or function definition in SysML v2 / KerML.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/CALC-DEF]
/// Contract: Encapsulates calculation `name`, optional `doc` metadata, typed `in_params` (`AttributeDef`), optional `return_type`, and optional body `expression` derived directly from SysML v2 AST source.
/// Bounds & Faults: O(1) default construction and O(P + E) clone/serialization cost over input parameters and expression byte length with zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CalcDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub in_params: Vec<AttributeDef>,
    #[serde(default)]
    pub return_type: Option<String>,
    #[serde(default)]
    pub expression: Option<String>,
}

/// Outcome of scanning an optional parenthesized parameter list `(...)` at a declaration header.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-EXPRESSION-PARAMS]
enum ParenClause<'a> {
    Absent(usize),
    Present(&'a str, usize),
    Unclosed,
}

/// Scans an optional `(...)` parameter list at `curr`, returning the inner `clean` slice and post-whitespace offset.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-EXPRESSION-PARAMS]
fn scan_optional_paren_clause<'a>(clean: &'a str, masked: &'a str, curr: usize) -> ParenClause<'a> {
    if !masked.get(curr..).is_some_and(|s| s.starts_with('(')) {
        return ParenClause::Absent(curr);
    }
    let inner_start = curr.saturating_add(1);
    let Some(close_rel) = masked.get(inner_start..).and_then(|s| s.find(')')) else {
        return ParenClause::Unclosed;
    };
    let inner_end = inner_start.saturating_add(close_rel);
    let inner = clean.get(inner_start..inner_end).unwrap_or("");
    let after_paren = inner_end.saturating_add(1);
    let next_curr = after_paren.saturating_add(skip_ws(masked.get(after_paren..).unwrap_or("")));
    ParenClause::Present(inner, next_curr)
}

/// Detects an optional `assert` keyword immediately preceding `constraint` at `c_pos` and returns `(is_assertion, doc_end_offset)`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
fn detect_assert_prefix(masked: &str, c_pos: usize) -> (bool, usize) {
    let before_c = masked.get(..c_pos).unwrap_or("").trim_end();
    let assert_start = before_c.len().saturating_sub(6);
    if before_c.ends_with("assert") && is_word_boundary_before(before_c, assert_start) {
        (true, assert_start)
    } else {
        (false, c_pos)
    }
}

/// Parses a single `[assert] constraint [def] <name> [(...)] ({ ... } | ;)` declaration beginning at `c_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
fn parse_single_constraint(
    clean: &str,
    masked: &str,
    c_pos: usize,
) -> Option<(ConstraintDef, usize)> {
    let (is_assertion, doc_end) = detect_assert_prefix(masked, c_pos);
    let doc = extract_doc_comment_before(clean, doc_end);

    let mut curr = c_pos.saturating_add(10);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let parameters = match scan_optional_paren_clause(clean, masked, curr) {
        ParenClause::Absent(next) => {
            curr = next;
            Vec::new()
        }
        ParenClause::Present(p_str, next) => {
            curr = next;
            p_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
        ParenClause::Unclosed => return None,
    };

    let rem_body = masked.get(curr..).unwrap_or("");
    let (expression, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        let expr_str = clean.get(curr.saturating_add(1)..close_pos).unwrap_or("");
        let expr = expr_str.trim().trim_end_matches(';').trim().to_string();
        (expr, close_pos.saturating_add(1))
    } else if rem_body.starts_with(';') {
        (String::new(), curr.saturating_add(1))
    } else {
        return None;
    };

    Some((
        ConstraintDef {
            name: name.to_string(),
            expression,
            parameters,
            is_assertion,
            doc,
            ..Default::default()
        },
        next_idx,
    ))
}

/// Parses parametric `ConstraintDef` definitions and `assert constraint` clauses from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
/// Contract: Extracts every `constraint def`, `constraint`, and `assert constraint` declaration in appearance order while ignoring tokens inside comments or string literals.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; malformed or unclosed blocks are skipped without panicking.
pub fn parse_constraint_defs(content: &str) -> Vec<ConstraintDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut constraints = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("constraint")) {
        let c_pos = idx.saturating_add(pos);
        let after_kw = c_pos.saturating_add(10);
        if !is_word_boundary_before(&masked, c_pos) || !is_word_boundary_after(&masked, after_kw) {
            idx = after_kw;
            continue;
        }
        if let Some((constraint, next_idx)) = parse_single_constraint(&clean, &masked, c_pos) {
            idx = next_idx;
            constraints.push(constraint);
        } else {
            idx = after_kw;
        }
    }
    constraints
}

/// Extracts an optional `= <default>` literal suffix into a trimmed non-empty `String`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-PARAM]
fn parse_optional_default_value(after_token: &str) -> Option<String> {
    let rhs = after_token.strip_prefix('=')?;
    let val = rhs.trim().trim_end_matches(';').trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

/// Parses a single calculation parameter declaration into an `AttributeDef` (defaulting untyped parameters to `ScalarValues::Real`).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-PARAM]
fn parse_calc_param(s: &str) -> Option<AttributeDef> {
    let mut trimmed = s.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("in") && is_word_boundary_after(trimmed, 2) {
        trimmed = trimmed.get(2..).unwrap_or("").trim_start();
    }
    if trimmed.starts_with("attribute") && is_word_boundary_after(trimmed, 9) {
        trimmed = trimmed.get(9..).unwrap_or("").trim_start();
    }
    let (name, name_len) = extract_ident(trimmed);
    if name.is_empty() {
        return None;
    }
    let after_name = trimmed.get(name_len..).unwrap_or("").trim_start();
    let mut type_name = "ScalarValues::Real".to_string();
    let default_value = if let Some(after_colon_raw) = after_name.strip_prefix(':') {
        let after_colon = after_colon_raw.trim_start();
        let (t_name, t_len) = extract_type_ident(after_colon);
        if !t_name.is_empty() {
            type_name = t_name.to_string();
        }
        let after_type = after_colon.get(t_len..).unwrap_or("").trim_start();
        parse_optional_default_value(after_type)
    } else {
        parse_optional_default_value(after_name)
    };
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
    s.chars().any(|c| matches!(c, '+' | '-' | '*' | '/'))
}

/// Checks whether a trimmed calculation body line declares an `in` or `attribute` input parameter.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn is_calc_body_param_line(trimmed: &str) -> bool {
    let has_param_prefix = trimmed.starts_with("in ")
        || trimmed.starts_with("in\t")
        || trimmed.starts_with("attribute ");
    has_param_prefix && !trimmed.contains('=') && !contains_arithmetic_operator(trimmed)
}

/// Checks whether a trimmed calculation body line begins with a `return` clause.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn is_calc_return_line(trimmed: &str) -> bool {
    trimmed.starts_with("return ")
        || trimmed.starts_with("return\t")
        || trimmed.starts_with("return:")
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
    let type_part = if let Some((_, after_colon)) = rest.split_once(':') {
        after_colon.trim()
    } else {
        rest.trim_start_matches("type").trim()
    };
    let (t_ident, _) = extract_type_ident(type_part);
    if t_ident.is_empty() {
        None
    } else {
        Some(t_ident.to_string())
    }
}

/// Extracts body-declared input parameters, inferred return type, and newline-joined expression lines from a `calc` body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
fn parse_calc_body(
    body: &str,
    in_params: &mut Vec<AttributeDef>,
    return_type: &mut Option<String>,
) -> Option<String> {
    let mut expr_lines = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_calc_body_param_line(trimmed) {
            if let Some(param) = parse_calc_param(trimmed) {
                in_params.push(param);
            }
        } else if is_calc_return_line(trimmed) {
            let rest = trimmed.trim_start_matches("return").trim();
            if return_type.is_none() {
                if let Some(parsed_ret) = try_extract_body_return_type(rest) {
                    *return_type = Some(parsed_ret);
                    continue;
                }
            }
            let expr_candidate = rest
                .trim_start_matches(':')
                .trim()
                .trim_end_matches(';')
                .trim();
            if !expr_candidate.is_empty() {
                expr_lines.push(expr_candidate);
            }
        } else {
            expr_lines.push(trimmed.trim_end_matches(';').trim());
        }
    }
    if expr_lines.is_empty() {
        None
    } else {
        Some(expr_lines.join("\n"))
    }
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
    let mut curr = calc_pos.saturating_add(4);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let mut in_params = Vec::new();
    match scan_optional_paren_clause(clean, masked, curr) {
        ParenClause::Absent(next) => curr = next,
        ParenClause::Present(p_str, next) => {
            for part in p_str.split(',') {
                if let Some(param) = parse_calc_param(part) {
                    in_params.push(param);
                }
            }
            curr = next;
        }
        ParenClause::Unclosed => return None,
    }

    let (mut return_type, after_ret) = parse_optional_signature_return_type(masked, curr);
    curr = after_ret;

    let rem_body = masked.get(curr..).unwrap_or("");
    let (body, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr);
        let body_slice = match close_pos {
            Some(cp) => clean.get(curr.saturating_add(1)..cp).unwrap_or(""),
            None => "",
        };
        let next = match close_pos {
            Some(cp) => cp.saturating_add(1),
            None => curr.saturating_add(1),
        };
        (Some(body_slice), next)
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
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
/// Contract: Extracts every `calc def` and `calc` declaration in appearance order, preserving `doc /* ... */` blocks, signature and body `in`/`attribute` parameters, signature or body `return` types, and result expressions.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; syntax irregularities are skipped safely without panicking.
pub fn parse_calc_defs(content: &str) -> Vec<CalcDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut calcs = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("calc")) {
        let calc_pos = idx.saturating_add(pos);
        let after_kw = calc_pos.saturating_add(4);
        if !is_word_boundary_before(&masked, calc_pos) || !is_word_boundary_after(&masked, after_kw)
        {
            idx = after_kw;
            continue;
        }
        if let Some((calc, next_idx)) = parse_single_calc(&clean, &masked, calc_pos) {
            idx = next_idx;
            calcs.push(calc);
        } else {
            idx = after_kw;
        }
    }
    calcs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `ConstraintDef` and `CalcDef` default initialization and serde JSON round-trip behavior.
    #[test]
    fn constraint_and_calc_def_serde_roundtrip() -> Result<(), serde_json::Error> {
        let constraint = ConstraintDef {
            name: "EnvelopeLimit".to_string(),
            expression: "airspeed <= 350.0".to_string(),
            parameters: vec!["airspeed : Real".to_string()],
            is_assertion: true,
            doc: Some("Enforces Vne limit.".to_string()),
            pre_conditions: vec!["sensorValid == true".to_string()],
            post_conditions: vec!["latchArmed == false".to_string()],
        };
        let c_json = serde_json::to_string(&constraint)?;
        let c_back: ConstraintDef = serde_json::from_str(&c_json)?;
        assert_eq!(c_back, constraint);

        let calc_min: CalcDef = serde_json::from_str(r#"{"name":"ComputeLift","doc":null}"#)?;
        assert_eq!(calc_min.name, "ComputeLift");
        assert!(calc_min.in_params.is_empty());
        assert!(calc_min.return_type.is_none());
        assert!(calc_min.expression.is_none());
        Ok(())
    }

    /// Verifies `parse_constraint_defs` across `constraint def`, `assert constraint`, doc blocks, UTF-8, and malformed inputs.
    #[test]
    fn parse_constraint_defs_extracts_assertions_docs_and_skips_malformed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            /* Ignore constraint def CommentedOut { x > 0; } and UTF-8 Δv */
            attribute note : String = "constraint def FakeInString { a == b; }";

            doc /* Formal rate bound */
            constraint def RateBound(omega : Real, maxOmega : Real) {
                omega <= maxOmega;
            }

            doc /* Asserted safety invariant */
            assert constraint ThermalMarginInvariant {
                tempK < 390.0;
            }

            assert constraint SemicolonAssertion;

            // Malformed declarations must be skipped without panicking
            constraint def ;
            constraint BrokenParen(x : Real;
            constraint NoTerminator
            constraint UnclosedBrace { x > 0;
        "#;

        let parsed = parse_constraint_defs(src);
        assert_eq!(parsed.len(), 3);

        let c0 = parsed.first().ok_or("missing RateBound")?;
        assert_eq!(c0.name, "RateBound");
        assert!(!c0.is_assertion);
        assert_eq!(c0.doc.as_deref(), Some("Formal rate bound"));
        assert_eq!(c0.parameters, vec!["omega : Real", "maxOmega : Real"]);
        assert_eq!(c0.expression, "omega <= maxOmega");

        let c1 = parsed.get(1).ok_or("missing ThermalMarginInvariant")?;
        assert_eq!(c1.name, "ThermalMarginInvariant");
        assert!(c1.is_assertion);
        assert_eq!(c1.doc.as_deref(), Some("Asserted safety invariant"));
        assert_eq!(c1.expression, "tempK < 390.0");

        let c2 = parsed.get(2).ok_or("missing SemicolonAssertion")?;
        assert_eq!(c2.name, "SemicolonAssertion");
        assert!(c2.is_assertion);
        assert!(c2.expression.is_empty());
        Ok(())
    }

    /// Verifies `parse_calc_defs` across signature/body parameters, defaults, return clauses, and malformed inputs.
    #[test]
    fn parse_calc_defs_extracts_signature_and_body_variants_and_skips_malformed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            /* Ignore calc def CommentedCalc; and UTF-8 ρ */
            attribute note : String = "calc def FakeCalcInString;";

            doc /* Dynamic pressure calculation */
            calc def DynamicPressure(in attribute rho : ScalarValues::Real = 1.225, v = 80.0) : ScalarValues::Real {
                return 0.5 * rho * v * v;
            }

            calc def BodyReturnTypeCalc {
                in x : Real;
                attribute gain : Real;
                return type Real;
                x * gain;
            }

            calc StubCalc;

            // Malformed declarations must be skipped safely
            calc def ;
            calc UnclosedParen(in x : Real;
            calc MissingTerminator
        "#;

        let calcs = parse_calc_defs(src);
        assert_eq!(calcs.len(), 3);

        let dp = calcs.first().ok_or("missing DynamicPressure")?;
        assert_eq!(dp.name, "DynamicPressure");
        assert_eq!(dp.doc.as_deref(), Some("Dynamic pressure calculation"));
        assert_eq!(dp.return_type.as_deref(), Some("ScalarValues::Real"));
        assert_eq!(dp.expression.as_deref(), Some("0.5 * rho * v * v"));
        assert_eq!(dp.in_params.len(), 2);
        let p0 = dp.in_params.first().ok_or("missing rho")?;
        assert_eq!(p0.name, "rho");
        assert_eq!(p0.type_name, "ScalarValues::Real");
        assert_eq!(p0.default_value.as_deref(), Some("1.225"));
        let p1 = dp.in_params.get(1).ok_or("missing v")?;
        assert_eq!(p1.name, "v");
        assert_eq!(p1.type_name, "ScalarValues::Real");
        assert_eq!(p1.default_value.as_deref(), Some("80.0"));

        let br = calcs.get(1).ok_or("missing BodyReturnTypeCalc")?;
        assert_eq!(br.name, "BodyReturnTypeCalc");
        assert_eq!(br.in_params.len(), 2);
        assert_eq!(br.return_type.as_deref(), Some("Real"));
        assert_eq!(br.expression.as_deref(), Some("x * gain"));

        let stub = calcs.get(2).ok_or("missing StubCalc")?;
        assert_eq!(stub.name, "StubCalc");
        assert!(stub.in_params.is_empty());
        assert!(stub.return_type.is_none());
        assert!(stub.expression.is_none());
        Ok(())
    }
}

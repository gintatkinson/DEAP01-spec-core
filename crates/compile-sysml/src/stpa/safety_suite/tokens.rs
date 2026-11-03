#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Parameter token harvesting, constraint token extraction, and Markdown/Mermaid identifier sanitization.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety transpilation objectives for module `crates/compile-sysml/src/stpa/safety_suite/tokens.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Extracts deterministic `BTreeMap<String, String>` parameter bindings from SysML v2 package and part scopes.
//! - Resolves case-insensitive token lookups and sanitizes Mermaid IDs and single-line Markdown table cells.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `PackageDef` or UTF-8 string slice.
//!
//! ## 4. Postconditions
//! - Returns sorted `BTreeMap<String, String>` parameter maps and sanitized identifiers/cells.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N * log T) over AST nodes N and extracted token count T.
//! - Space Complexity: O(T) bounded auxiliary memory for extracted token bindings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Handles multi-byte UTF-8 prefixes and empty expressions without mid-codepoint slicing.
//! - State: Pure functional extraction and string sanitization.
//! - Action: Executes linear UTF-8-safe `.get(..)` passes with saturating arithmetic and zero panics.
//! - Resource: Bounded allocations proportional to AST attribute and constraint counts.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-TOKENS] | SysML v2: PackageDef(safety_suite_tokens)

use super::*;

/// Collects symbolic parameter tokens from package- and part-scoped AST attribute defaults and constraint expressions.
/// Realises: [REQ-STPA-SAFETY-SUITE-COLLECT-PARAMETER-TOKENS]
pub fn collect_parameter_tokens(pkg: &PackageDef) -> BTreeMap<String, String> {
    let mut tokens = BTreeMap::new();

    for p in pkg.get_all_packages() {
        for attr in &p.attribute_defs {
            if let Some(ref val) = attr.default_value {
                let s = val.trim().trim_matches('"').trim();
                if !s.is_empty() {
                    tokens.insert(attr.name.clone(), s.to_string());
                }
            }
        }
        for con in &p.constraint_defs {
            extract_constraint_tokens(&con.expression, &mut tokens);
        }
    }

    for part in pkg.get_all_parts() {
        for attr in &part.attributes {
            if let Some(ref val) = attr.default_value {
                let s = val.trim().trim_matches('"').trim();
                if !s.is_empty() {
                    tokens
                        .entry(attr.name.clone())
                        .or_insert_with(|| s.to_string());
                }
            }
        }
        for con in &part.constraints {
            extract_constraint_tokens(&con.expression, &mut tokens);
        }
    }

    tokens
}

/// Resolves a non-empty trimmed token value from `tokens` by exact key or lexicographic ASCII case-insensitive match.
/// Realises: [REQ-STPA-SAFETY-SUITE-FIND-TOKEN-CI]
pub(super) fn find_token_ci<'a>(
    tokens: &'a BTreeMap<String, String>,
    target: &str,
) -> Option<&'a str> {
    if let Some(exact) = tokens.get(target) {
        let trimmed = exact.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    for (k, v) in tokens {
        if k.eq_ignore_ascii_case(target) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

/// Resolves the lexicographically first non-empty trimmed token value in `tokens` matching `prefix` ASCII case-insensitively.
/// Realises: [REQ-STPA-SAFETY-SUITE-FIND-TOKEN-PREFIX-CI]
pub(super) fn find_token_prefix_ci<'a>(
    tokens: &'a BTreeMap<String, String>,
    prefix: &str,
) -> Option<&'a str> {
    let lower_prefix = prefix.to_ascii_lowercase();
    for (k, v) in tokens {
        if k.to_ascii_lowercase().starts_with(&lower_prefix) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

/// Scans a constraint expression for `<ident> <op> <value>` relational clauses and records first-seen parameter bindings in `tokens`.
/// Realises: [REQ-STPA-SAFETY-SUITE-EXTRACT-CONSTRAINT-TOKENS]
pub(super) fn extract_constraint_tokens(expr: &str, tokens: &mut BTreeMap<String, String>) {
    let operators = ["<=", ">=", "<", ">", "="];
    let mut idx = 0usize;
    while let Some(tail) = expr.get(idx..) {
        if tail.is_empty() {
            break;
        }
        let matched_op = operators.iter().find(|&&op| tail.starts_with(op));
        if let Some(&op) = matched_op {
            let before = expr.get(..idx).unwrap_or("");
            let before_trimmed = before.trim_end();
            let trailing_id_len = before_trimmed
                .as_bytes()
                .iter()
                .rev()
                .take_while(|&&b| b.is_ascii_alphanumeric() || b == b'_')
                .count();
            let id_start = before_trimmed.len().saturating_sub(trailing_id_len);
            let key = before_trimmed.get(id_start..).unwrap_or("");
            let key_valid = key.as_bytes().split_first().is_some_and(|(&first, rest)| {
                (first.is_ascii_alphabetic() || first == b'_')
                    && rest.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_')
            });

            let after = tail.strip_prefix(op).unwrap_or("");
            let after_trimmed = after.trim_start();
            let val_len = after_trimmed
                .find(|c: char| c.is_whitespace() || c == ';')
                .unwrap_or(after_trimmed.len());
            let val = after_trimmed.get(..val_len).unwrap_or("");

            if key_valid && !val.is_empty() {
                tokens
                    .entry(key.to_string())
                    .or_insert_with(|| val.to_string());
                let leading_ws = after.len().saturating_sub(after_trimmed.len());
                let consumed = op.len().saturating_add(leading_ws).saturating_add(val_len);
                idx = idx.saturating_add(consumed);
                continue;
            }
            idx = idx.saturating_add(op.len());
        } else {
            let step = tail.chars().next().map_or(1, char::len_utf8);
            idx = idx.saturating_add(step);
        }
    }
}

/// Collects all `ConstraintDef` nodes across package, subpackage, and part scopes in deterministic traversal order.
/// Realises: [REQ-STPA-SAFETY-SUITE-COLLECT-ALL-CONSTRAINTS]
pub fn collect_all_constraints(pkg: &PackageDef) -> Vec<ConstraintDef> {
    pkg.get_all_constraints()
}

/// Sanitizes an arbitrary identifier into a Mermaid-safe alphanumeric/underscore node ID prefixed with `Node_` when empty or leading-numeric.
/// Realises: [REQ-STPA-SAFETY-SUITE-SANITIZE-MERMAID-ID]
pub(super) fn sanitize_mermaid_id(id: &str) -> String {
    let clean: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if clean.is_empty() || clean.chars().next().is_some_and(|c| c.is_numeric()) {
        format!("Node_{}", clean)
    } else {
        clean
    }
}

/// Normalizes CR/LF line breaks into spaces and replaces `|` with `/` so arbitrary AST text embeds safely in single-line Markdown table cells.
/// Realises: [REQ-STPA-SAFETY-SUITE-SANITIZE-MARKDOWN-CELL]
pub(super) fn sanitize_markdown_cell(raw: &str) -> String {
    let mut buf = String::with_capacity(raw.len());
    for ch in raw.trim().chars() {
        match ch {
            '\r' | '\n' => buf.push(' '),
            '|' => buf.push('/'),
            _ => buf.push(ch),
        }
    }
    buf
}

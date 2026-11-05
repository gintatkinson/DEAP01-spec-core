//! AST ground-truth symbol harvester, protocol token classifier, and quantitative claim extractor.
//!
//! Safety & Traceability: Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION], [REQ-PROTOCOL-TOKEN-EXTRACTION]
//! Contract: Extracts SysML v2 AST ground-truth symbols/claims and classifies lexical protocol tokens.
//! Bounds & Faults: Pure non-panicking AST and lexical inspection with zero regex or hardcoded word lists.

use super::parity_check_icd_ast::is_valid_si_unit;
use deap_core::sysml_ast::{parse_constraint_defs, parse_part_defs, parse_port_defs};
use std::collections::BTreeSet;

/// Determines whether a lexical token has the structural morphology of a technical protocol identifier.
/// Realises: [REQ-PROTOCOL-TOKEN-EXTRACTION]
pub(crate) fn is_structural_protocol_token(token: &str) -> bool {
    if token.len() < 3
        || !token.starts_with(|c: char| c.is_ascii_uppercase())
        || !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        || token.contains("--")
    {
        return false;
    }
    if !token
        .split('-')
        .all(|seg| seg.starts_with(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit()))
    {
        return false;
    }
    if !token.contains('-')
        && !token.chars().any(|c| c.is_ascii_digit())
        && token
            .strip_suffix('s')
            .is_some_and(|stem| stem.len() >= 2 && stem.chars().all(|c| c.is_ascii_uppercase()))
    {
        return false;
    }
    let mbse_rest = token
        .strip_prefix("SC-")
        .or_else(|| token.strip_prefix("UCA-"))
        .or_else(|| token.strip_prefix("REQ-"))
        .or_else(|| token.strip_prefix("UC-"))
        .or_else(|| token.strip_prefix("US-"))
        .or_else(|| token.strip_prefix("FEAT-"))
        .or_else(|| token.strip_prefix("EPIC-"))
        .or_else(|| token.strip_prefix("LS-"))
        .or_else(|| token.strip_prefix("BP-"))
        .or_else(|| token.strip_prefix("H-"))
        .or_else(|| token.strip_prefix("C-"));
    if let Some(rest) = mbse_rest {
        if !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase() || c == '-')
        {
            return false;
        }
    }
    token.split('-').any(|seg| {
        (!seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()))
            || (seg.len() >= 2
                && seg.get(1..).is_some_and(|rest| {
                    rest.chars()
                        .any(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                }))
    })
}

/// Normalizes a protocol or AST identifier to lowercase ASCII alphanumeric characters.
/// Realises: [REQ-AST-SYMBOL-NORMALIZATION]
pub(crate) fn normalize_ast_protocol_symbol(token: &str) -> String {
    token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Recursively harvests AST symbols and quantitative claims from a slice of `PartDef` nodes.
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION]
pub(crate) fn harvest_ast_parts_ground_truth(
    parts: &[deap_core::sysml_ast::PartDef],
    symbols: &mut BTreeSet<String>,
    claims: &mut BTreeSet<String>,
) {
    for part in parts {
        if !part.name.is_empty() {
            symbols.insert(part.name.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&part.name));
        }
        for port in &part.ports {
            harvest_ast_port_ground_truth(port, symbols, claims);
        }
        for attr in &part.attributes {
            harvest_ast_attribute_ground_truth(attr, symbols, claims);
        }
        for c in &part.constraints {
            if !c.name.is_empty() {
                symbols.insert(c.name.to_ascii_lowercase());
                symbols.insert(normalize_ast_protocol_symbol(&c.name));
            }
            for q in extract_quantitative_unit_claims(&c.expression) {
                claims.insert(q);
            }
        }
        harvest_ast_parts_ground_truth(&part.parts, symbols, claims);
    }
}

/// Harvests AST symbols and quantitative claims from a single `PortDef` node.
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION]
pub(crate) fn harvest_ast_port_ground_truth(
    port: &deap_core::sysml_ast::PortDef,
    symbols: &mut BTreeSet<String>,
    claims: &mut BTreeSet<String>,
) {
    for field in [
        &port.name,
        &port.type_name,
        &port.protocol_family,
        &port.port_category,
    ] {
        if !field.is_empty() {
            symbols.insert(field.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(field));
        }
    }
    for flow in &port.item_flows {
        if !flow.name.is_empty() {
            symbols.insert(flow.name.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&flow.name));
        }
        if !flow.item_type.is_empty() {
            symbols.insert(flow.item_type.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&flow.item_type));
        }
    }
    for (k, v) in &port.electrical_attributes {
        symbols.insert(k.to_ascii_lowercase());
        symbols.insert(v.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(v));
        for q in extract_quantitative_unit_claims(v) {
            claims.insert(q);
        }
    }
}

/// Harvests AST symbols and quantitative claims from a single `AttributeDef` node.
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION]
pub(crate) fn harvest_ast_attribute_ground_truth(
    attr: &deap_core::sysml_ast::AttributeDef,
    symbols: &mut BTreeSet<String>,
    claims: &mut BTreeSet<String>,
) {
    if !attr.name.is_empty() {
        symbols.insert(attr.name.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&attr.name));
    }
    if !attr.type_name.is_empty() {
        symbols.insert(attr.type_name.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(&attr.type_name));
    }
    if let Some(ref dv) = attr.default_value {
        let clean = dv.trim_matches('"').trim();
        symbols.insert(clean.to_ascii_lowercase());
        symbols.insert(normalize_ast_protocol_symbol(clean));
        for q in extract_quantitative_unit_claims(clean) {
            claims.insert(q);
        }
    }
}

/// Parses SysML model text with the AST parser to construct ground-truth symbol and claim sets.
/// Realises: [REQ-AST-GROUND-TRUTH-EXTRACTION]
pub(crate) fn build_ast_model_ground_truth(model_str: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut symbols = BTreeSet::new();
    let mut claims = BTreeSet::new();
    if model_str.trim().is_empty() {
        return (symbols, claims);
    }

    let parts = parse_part_defs(model_str);
    harvest_ast_parts_ground_truth(&parts, &mut symbols, &mut claims);

    for port in parse_port_defs(model_str) {
        harvest_ast_port_ground_truth(&port, &mut symbols, &mut claims);
    }
    for attr in deap_core::sysml_ast::parse_attribute_defs(model_str) {
        harvest_ast_attribute_ground_truth(&attr, &mut symbols, &mut claims);
    }
    for c in parse_constraint_defs(model_str) {
        if !c.name.is_empty() {
            symbols.insert(c.name.to_ascii_lowercase());
            symbols.insert(normalize_ast_protocol_symbol(&c.name));
        }
        for q in extract_quantitative_unit_claims(&c.expression) {
            claims.insert(q);
        }
    }
    for q in extract_quantitative_unit_claims(model_str) {
        claims.insert(q);
    }

    (symbols, claims)
}

/// Replaces inline Markdown links `[label](target)` with `label` via pure character scanning.
/// Realises: [REQ-PROTOCOL-TOKEN-EXTRACTION]
pub(crate) fn strip_markdown_link_targets(line: &str) -> String {
    if !line.contains("](") {
        return line.to_string();
    }
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut idx = 0usize;
    while let Some(&ch) = chars.get(idx) {
        if ch == '[' {
            let mut bracket_depth = 1usize;
            let mut close_bracket = None;
            let mut scan = idx.saturating_add(1);
            while let Some(&c2) = chars.get(scan) {
                if c2 == '[' {
                    bracket_depth = bracket_depth.saturating_add(1);
                } else if c2 == ']' {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    if bracket_depth == 0 {
                        close_bracket = Some(scan);
                        break;
                    }
                }
                scan = scan.saturating_add(1);
            }
            if let Some(cb) = close_bracket {
                if chars.get(cb.saturating_add(1)) == Some(&'(') {
                    let mut paren_depth = 1usize;
                    let mut close_paren = None;
                    let mut pscan = cb.saturating_add(2);
                    while let Some(&c3) = chars.get(pscan) {
                        if c3 == '(' {
                            paren_depth = paren_depth.saturating_add(1);
                        } else if c3 == ')' {
                            paren_depth = paren_depth.saturating_sub(1);
                            if paren_depth == 0 {
                                close_paren = Some(pscan);
                                break;
                            }
                        }
                        pscan = pscan.saturating_add(1);
                    }
                    if let Some(cp) = close_paren {
                        if let Some(label_chars) = chars.get(idx.saturating_add(1)..cb) {
                            for &lc in label_chars {
                                out.push(lc);
                            }
                        }
                        idx = cp.saturating_add(1);
                        continue;
                    }
                }
            }
        }
        out.push(ch);
        idx = idx.saturating_add(1);
    }
    out
}

/// Extracts candidate communication protocol or hardware interface identifiers from a line.
/// Realises: [REQ-PROTOCOL-TOKEN-EXTRACTION]
pub(crate) fn extract_candidate_protocol_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower_full = line.to_ascii_lowercase();
    if lower_full.contains("[tier-3:") || lower_full.contains("[tier-4:") {
        return out;
    }

    let sanitized_line = strip_markdown_link_targets(line);
    for segment in sanitized_line.split('|') {
        let words: Vec<&str> = segment
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| {
                    matches!(
                        c,
                        '`' | '\'' | '"' | '(' | ')' | '[' | ']' | ',' | ';' | '.' | ':'
                    )
                })
            })
            .filter(|w| !w.is_empty())
            .collect();

        for (i, &w) in words.iter().enumerate() {
            if w == "over" || w == "via" {
                for j in i.saturating_add(1)..words.len() {
                    let Some(&cand_w) = words.get(j) else {
                        continue;
                    };
                    if is_valid_si_unit(cand_w) || !is_structural_protocol_token(cand_w) {
                        continue;
                    }
                    let combined = match words.get(j.saturating_add(1)) {
                        Some(&next_num)
                            if !next_num.is_empty()
                                && next_num.chars().all(|c| c.is_ascii_digit()) =>
                        {
                            format!("{} {}", cand_w, next_num)
                        }
                        _ => cand_w.to_string(),
                    };
                    if !out
                        .iter()
                        .any(|existing: &String| existing.eq_ignore_ascii_case(&combined))
                    {
                        out.push(combined);
                    }
                    break;
                }
            }
            let wl = w.to_ascii_lowercase();
            if (wl == "protocol" || wl == "interface" || wl == "bus") && i > 0 {
                if let Some(&prev_w) = words.get(i.saturating_sub(1)) {
                    if is_structural_protocol_token(prev_w)
                        && !is_valid_si_unit(prev_w)
                        && !out
                            .iter()
                            .any(|existing: &String| existing.eq_ignore_ascii_case(prev_w))
                    {
                        out.push(prev_w.to_string());
                    }
                }
            }
        }
    }
    out
}

/// Extracts numeric-unit pairs (such as `50 Hz` or `100 ms`) from a specification line.
/// Realises: [REQ-QUANTITATIVE-CLAIM-EXTRACTION]
pub(crate) fn extract_quantitative_unit_claims(line: &str) -> Vec<String> {
    let words: Vec<&str> = line
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| {
                !c.is_ascii_alphanumeric() && c != '.' && c != '/' && c != '^' && c != '%'
            })
        })
        .filter(|w| !w.is_empty())
        .collect();
    let mut claims = Vec::new();
    for i in 0..words.len() {
        if let (Some(&num_w), Some(&unit_w)) = (words.get(i), words.get(i.saturating_add(1))) {
            let is_num = !num_w.is_empty()
                && num_w.chars().all(|c| c.is_ascii_digit() || c == '.')
                && num_w.chars().any(|c| c.is_ascii_digit());
            if is_num && is_valid_si_unit(unit_w) {
                claims.push(format!(
                    "{} {}",
                    num_w.to_ascii_lowercase(),
                    unit_w.to_ascii_lowercase()
                ));
            }
        }
    }
    claims
}

//! Level 1B operational abstraction sanitization and Mermaid formatting routines.

mod mermaid_clean;
mod protocol_clean;

pub use mermaid_clean::wrap_mermaid_label;
use mermaid_clean::{
    clean_comma_period, clean_lead_comma_paren, clean_multi_commas, clean_multi_spaces,
    clean_space_commas, clean_trail_comma_paren, strip_empty_delimited,
};
use protocol_clean::{strip_baud_rates, strip_opcodes, strip_use_cases};

/// Matches a token after skipping horizontal ASCII whitespace at `idx`.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-MATCH-TOKEN] | SysML v2: ActionDef(match_token)
fn match_token(lower: &str, mut idx: usize, token: &str) -> Option<usize> {
    let bytes = lower.as_bytes();
    while bytes.get(idx).is_some_and(|&b| b == b' ' || b == b'\t') {
        idx += 1;
    }
    if lower[idx..].starts_with(token) {
        let after = idx + token.len();
        Some(after)
    } else {
        None
    }
}

/// Strips raw CRC-16 generator polynomial equations (`x^16 + x^12 + x^5 + 1`).
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-POLY-EQUATION] | SysML v2: ActionDef(strip_poly_equation)
fn strip_poly_equation(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("x^16") {
            let start = i + pos;
            let j = start + 4;
            let mut matched = false;
            if let Some(next_j) = match_token(&lower, j, "+") {
                if let Some(next_j) = match_token(&lower, next_j, "x^12") {
                    if let Some(next_j) = match_token(&lower, next_j, "+") {
                        if let Some(next_j) = match_token(&lower, next_j, "x^5") {
                            if let Some(next_j) = match_token(&lower, next_j, "+") {
                                if let Some(next_j) = match_token(&lower, next_j, "1") {
                                    result.push_str(&text[last_idx..start]);
                                    last_idx = next_j;
                                    i = next_j;
                                    matched = true;
                                }
                            }
                        }
                    }
                }
            }
            if !matched {
                i = start + 4;
            }
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Strips hexadecimal polynomial constants (`0x1021` or `_0x1021`).
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-POLY-HEX] | SysML v2: ActionDef(strip_poly_hex)
fn strip_poly_hex(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("0x1021") {
            let mut start = i + pos;
            let end = start + 6;
            if start > 0 && bytes.get(start - 1).copied() == Some(b'_') {
                start -= 1;
            }
            let right_ok = !bytes
                .get(end)
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
            if right_ok {
                result.push_str(&text[last_idx..start]);
                last_idx = end;
                i = end;
                continue;
            }
            i = end;
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Matches a `CRC-16` / `CRC16` token and optional `polynomial [equation|formula]` suffix at `start` (`[REQ-ASSEMBLE-CONOPS]`).
fn match_crc16_span(lower: &str, bytes: &[u8], start: usize) -> Option<usize> {
    let left_ok =
        start == 0 || !bytes.get(start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
    if !left_ok {
        return None;
    }
    let mut j = start + 3;
    if bytes.get(j).copied() == Some(b'-') {
        j += 1;
    }
    if !lower[j..].starts_with("16") {
        return None;
    }
    j += 2;
    if bytes.get(j).copied() == Some(b'-') {
        let mut k = j + 1;
        while bytes.get(k).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
            k += 1;
        }
        if k > j + 1 {
            j = k;
        }
    }
    let mut k = j;
    while bytes.get(k).is_some_and(|&b| b == b' ' || b == b'\t') {
        k += 1;
    }
    if k > j && lower[k..].starts_with("polynomial") {
        let poly_end = k + 10;
        if !bytes.get(poly_end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
            j = poly_end;
            let mut m = j;
            while bytes.get(m).is_some_and(|&b| b == b' ' || b == b'\t') {
                m += 1;
            }
            if m > j {
                for kw in ["equation", "formula"] {
                    if lower[m..].starts_with(kw) {
                        let end = m + kw.len();
                        if !bytes.get(end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                            j = end;
                        }
                        break;
                    }
                }
            }
        }
    }
    (!bytes.get(j).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')).then_some(j)
}

/// Replaces low-level `CRC-16` polynomial references with `Frame Integrity Check`.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-REPLACE-CRC16] | SysML v2: ActionDef(replace_crc16)
fn replace_crc16(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("crc") {
            let start = i + pos;
            if let Some(s_end) = match_crc16_span(&lower, bytes, start) {
                result.push_str(&text[last_idx..start]);
                result.push_str("Frame Integrity Check");
                last_idx = s_end;
                i = s_end;
                continue;
            }
            i = start + 3;
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Replaces `polynomial equation` phrases with `integrity validation`.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-REPLACE-POLYNOMIAL-EQUATION] | SysML v2: ActionDef(replace_polynomial_equation)
fn replace_polynomial_equation(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("polynomial") {
            let start = i + pos;
            let end_poly = start + 10;
            let left_ok = start == 0
                || !bytes
                    .get(start - 1)
                    .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
            if left_ok {
                let mut j = end_poly;
                while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                    j += 1;
                }
                if j > end_poly && lower[j..].starts_with("equation") {
                    let end = j + 8;
                    let right_ok = !bytes
                        .get(end)
                        .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
                    if right_ok {
                        result.push_str(&text[last_idx..start]);
                        result.push_str("integrity validation");
                        last_idx = end;
                        i = end;
                        continue;
                    }
                }
            }
            i = end_poly;
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

/// [DOC-TIER: A] Sanitizes text to enforce Level 1B operational abstraction.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-SANITIZE-LEVEL-1B-OPERATIONAL-TEXT] | SysML v2: ActionDef(sanitize_level_1b_operational_text)
/// Contract: Removes CRC polynomials, serial baud rates, wire opcodes, and Level 2 `UC-xx` tags from `text: &str` while normalizing punctuation.
/// Bounds & Faults: O(N) time and space over `text.len()`; preserves `UCL-xx` user class taxonomy tags.
pub fn sanitize_level_1b_operational_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut s = text.to_string();
    s = strip_poly_equation(&s);
    s = strip_poly_hex(&s);
    s = replace_crc16(&s);
    s = replace_polynomial_equation(&s);
    s = strip_baud_rates(&s);
    s = strip_opcodes(&s);
    s = strip_use_cases(&s);
    s = strip_empty_delimited(&s, '(', ')');
    s = strip_empty_delimited(&s, '[', ']');
    s = clean_lead_comma_paren(&s);
    s = clean_trail_comma_paren(&s);
    s = clean_multi_commas(&s);
    s = clean_space_commas(&s);
    s = clean_comma_period(&s);
    s = clean_multi_spaces(&s);

    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_level_1b_polynomial_and_baud() {
        let raw = "Data link at 115200 baud with x^16 + x^12 + x^5 + 1 CRC-16 polynomial formula.";
        let sanitized = sanitize_level_1b_operational_text(raw);
        assert!(!sanitized.contains("115200"));
        assert!(!sanitized.contains("x^16"));
        assert!(sanitized.contains("Frame Integrity Check"));
    }

    #[test]
    fn test_sanitize_level_1b_opcodes_and_use_cases() {
        let raw = "Triggered by opcode 0x12 and UC-01 (UC-02 and UC-03) command.";
        let sanitized = sanitize_level_1b_operational_text(raw);
        assert!(!sanitized.contains("0x12"));
        assert!(!sanitized.contains("UC-01"));
        assert!(!sanitized.contains("UC-02"));
        assert!(!sanitized.contains("opcode"));
    }

    #[test]
    fn test_wrap_mermaid_label() {
        let label =
            "This is a very long subsystem descriptive label that exceeds maximum width limit";
        let wrapped = wrap_mermaid_label(label, 25);
        assert!(wrapped.contains("<br/>"));
    }

    #[test]
    fn test_issue_291_user_class_taxonomy_preserved_and_collision_eliminated() {
        let legacy_conops =
            "Maintenance Technician (UC-04) performing pre-sortie inspection under UC-01.";
        let sanitized_legacy = sanitize_level_1b_operational_text(legacy_conops);
        assert!(!sanitized_legacy.contains("UC-04"));
        assert!(!sanitized_legacy.contains("UC-01"));

        let compliant_conops =
            "Maintenance Technician (UCL-04) performing pre-sortie inspection under UCL-01.";
        let sanitized_compliant = sanitize_level_1b_operational_text(compliant_conops);
        assert!(sanitized_compliant.contains("UCL-04"));
        assert!(sanitized_compliant.contains("UCL-01"));
    }
}

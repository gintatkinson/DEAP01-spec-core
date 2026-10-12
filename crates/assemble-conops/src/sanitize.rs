//! Level 1B operational abstraction sanitization and Mermaid formatting routines.

use regex::Regex;

/// Sanitizes text to enforce Level 1B operational abstraction.
/// Strips component-internal serial opcodes, baud rates, CRC-16 polynomial formulas,
/// and Level 2 System Use Cases (uc-xx / (UC-xx)).
pub fn sanitize_level_1b_operational_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut s = text.to_string();

    // 1. Remove polynomial equations
    let re_poly = Regex::new(r"(?i)x\^16\s*\+\s*x\^12\s*\+\s*x\^5\s*\+\s*1").unwrap();
    s = re_poly.replace_all(&s, "").to_string();

    let re_poly_hex = Regex::new(r"(?i)_?0x1021\b").unwrap();
    s = re_poly_hex.replace_all(&s, "").to_string();

    let re_crc16 = Regex::new(r"(?i)\bCRC-?16(?:-[A-Za-z0-9_]+)?(?:\s+polynomial(?:\s+(?:equation|formula))?)?\b").unwrap();
    s = re_crc16.replace_all(&s, "Frame Integrity Check").to_string();

    let re_poly_word = Regex::new(r"(?i)\bpolynomial\s+equation\b").unwrap();
    s = re_poly_word.replace_all(&s, "integrity validation").to_string();

    // 2. Remove baud rates
    let re_baud1 = Regex::new(r"(?i)\b(?:at\s+)?\d+(?:\.\d+)?\s*(?:k|M|G)?baud\b").unwrap();
    s = re_baud1.replace_all(&s, "").to_string();

    let re_baud2 = Regex::new(r"(?i)\b(?:at\s+)?(?:9600|19200|38400|57600|115200|230400|460800|921600)\s*(?:bps|baud)\b").unwrap();
    s = re_baud2.replace_all(&s, "").to_string();

    let re_baud3 = Regex::new(r"_(?:9600|19200|38400|57600|115200|230400|460800|921600)\b").unwrap();
    s = re_baud3.replace_all(&s, "").to_string();

    // 3. Remove serial opcodes
    let re_op1 = Regex::new(r"\b(?:[Oo]pcode|[Oo]pcodes)\s+0x[0-9a-fA-F]+\b").unwrap();
    s = re_op1.replace_all(&s, "").to_string();

    let re_op2 = Regex::new(r"\b(?:OPCODE|OPCODES)\s+0x[0-9a-fA-F]+\b").unwrap();
    s = re_op2.replace_all(&s, "").to_string();

    let re_op3 = Regex::new(r"\b(?:[Oo]pcode|[Oo]pcodes)\s+[0-9]+\b").unwrap();
    s = re_op3.replace_all(&s, "").to_string();

    let re_op4 = Regex::new(r"_?0x[0-9a-fA-F]+").unwrap();
    s = re_op4.replace_all(&s, "").to_string();

    let re_op5 = Regex::new(r"\b(?:Opcode|opcode|OPCODE)\b").unwrap();
    s = re_op5.replace_all(&s, "").to_string();

    // 4. Remove Level 2 System Use Cases (e.g. (UC-01 and UC-03), uc-01, UC-01)
    let re_uc_parens = Regex::new(r"\(\s*(?:UC|uc)-\d+(?:\s*(?:and|&|,)\s*(?:UC|uc)-\d+)*\s*\)").unwrap();
    s = re_uc_parens.replace_all(&s, "").to_string();

    let re_uc_bare = Regex::new(r"\b(?:UC|uc)-\d+\b").unwrap();
    s = re_uc_bare.replace_all(&s, "").to_string();

    // 5. Clean up punctuation artifacts, empty parens/brackets, duplicate commas, double spaces
    let re_empty_parens = Regex::new(r"\(\s*\)").unwrap();
    s = re_empty_parens.replace_all(&s, "").to_string();

    let re_empty_brackets = Regex::new(r"\[\s*\]").unwrap();
    s = re_empty_brackets.replace_all(&s, "").to_string();

    let re_lead_comma_paren = Regex::new(r"\(\s*,+\s*").unwrap();
    s = re_lead_comma_paren.replace_all(&s, "(").to_string();

    let re_trail_comma_paren = Regex::new(r",+\s*\)").unwrap();
    s = re_trail_comma_paren.replace_all(&s, ")").to_string();

    let re_multi_commas = Regex::new(r",\s*,+").unwrap();
    s = re_multi_commas.replace_all(&s, ",").to_string();

    let re_space_commas = Regex::new(r"[ \t]+,\s*").unwrap();
    s = re_space_commas.replace_all(&s, ", ").to_string();

    let re_comma_period = Regex::new(r",\s*\.").unwrap();
    s = re_comma_period.replace_all(&s, ".").to_string();

    let re_multi_spaces = Regex::new(r"[ \t]{2,}").unwrap();
    s = re_multi_spaces.replace_all(&s, " ").to_string();

    s.trim().to_string()
}

/// Wraps text into segments <= max_width characters using `<br/>`.
/// Enforces Rule E2 / Rule E3 visual ergonomics.
pub fn wrap_mermaid_label(text: &str, max_width: usize) -> String {
    if text.is_empty() {
        return String::new();
    }

    let seg_split_re = Regex::new(r"(?i)(<br\s*/?>|\r?\n)").unwrap();
    let html_tag_re = Regex::new(r"</?[a-zA-Z0-9_-]+\s*/?>").unwrap();

    let mut out_parts = Vec::new();
    let mut last_idx = 0;

    for mat in seg_split_re.find_iter(text) {
        if mat.start() > last_idx {
            let seg = &text[last_idx..mat.start()];
            process_segment(seg, max_width, &html_tag_re, &mut out_parts);
        }
        out_parts.push(mat.as_str().to_string());
        last_idx = mat.end();
    }

    if last_idx < text.len() {
        let seg = &text[last_idx..];
        process_segment(seg, max_width, &html_tag_re, &mut out_parts);
    }

    out_parts.join("")
}

fn process_segment(
    seg: &str,
    max_width: usize,
    html_tag_re: &Regex,
    out_parts: &mut Vec<String>,
) {
    let clean_seg = html_tag_re.replace_all(seg, "").trim().to_string();
    if clean_seg.len() <= max_width {
        out_parts.push(seg.to_string());
    } else {
        let words: Vec<&str> = seg.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut curr: Vec<&str> = Vec::new();
        let mut curr_len = 0;

        for w in words {
            let w_clean = html_tag_re.replace_all(w, "").to_string();
            let w_len = w_clean.len();

            if w_len > max_width {
                if !curr.is_empty() {
                    lines.push(curr.join(" "));
                    curr.clear();
                    curr_len = 0;
                }
                let mut c_idx = 0;
                while c_idx < w.len() {
                    let end_idx = (c_idx + max_width).min(w.len());
                    lines.push(w[c_idx..end_idx].to_string());
                    c_idx = end_idx;
                }
            } else if !curr.is_empty() && curr_len + 1 + w_len > max_width {
                lines.push(curr.join(" "));
                curr = vec![w];
                curr_len = w_len;
            } else {
                if !curr.is_empty() {
                    curr_len += 1;
                }
                curr.push(w);
                curr_len += w_len;
            }
        }
        if !curr.is_empty() {
            lines.push(curr.join(" "));
        }
        out_parts.push(lines.join("<br/>"));
    }
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
        let label = "This is a very long subsystem descriptive label that exceeds maximum width limit";
        let wrapped = wrap_mermaid_label(label, 25);
        assert!(wrapped.contains("<br/>"));
    }
}

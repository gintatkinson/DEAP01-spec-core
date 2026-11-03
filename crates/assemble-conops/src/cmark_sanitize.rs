//! Level 1B operational abstraction sanitization and Mermaid label wrapping (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

/// Matches a token after skipping horizontal ASCII whitespace at `idx` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn match_token(lower: &str, mut idx: usize, token: &str) -> Option<usize> {
    let bytes = lower.as_bytes();
    while bytes.get(idx).is_some_and(|&b| b == b' ' || b == b'\t') {
        idx += 1;
    }
    lower[idx..].starts_with(token).then(|| idx + token.len())
}

/// Strips raw CRC-16 generator polynomial equations (`x^16 + x^12 + x^5 + 1`) (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_poly_equation(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let (mut result, mut last_idx, mut i) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower[i..].find("x^16") {
        let start = i + pos;
        let end_opt = match_token(&lower, start + 4, "+")
            .and_then(|j| match_token(&lower, j, "x^12"))
            .and_then(|j| match_token(&lower, j, "+"))
            .and_then(|j| match_token(&lower, j, "x^5"))
            .and_then(|j| match_token(&lower, j, "+"))
            .and_then(|j| match_token(&lower, j, "1"));
        if let Some(next_j) = end_opt {
            result.push_str(&text[last_idx..start]);
            last_idx = next_j;
            i = next_j;
        } else {
            i = start + 4;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Strips hexadecimal polynomial constants (`0x1021` or `_0x1021`) (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_poly_hex(text: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut result, mut last_idx, mut i) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower[i..].find("0x1021") {
        let mut start = i + pos;
        let end = start + 6;
        if start > 0 && bytes.get(start - 1).copied() == Some(b'_') { start -= 1; }
        if !bytes.get(end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
            result.push_str(&text[last_idx..start]);
            last_idx = end;
        }
        i = end;
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Matches a `CRC-16` / `CRC16` token and optional `polynomial [equation|formula]` suffix (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn match_crc16_span(lower: &str, bytes: &[u8], start: usize) -> Option<usize> {
    if start > 0 && bytes.get(start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') { return None; }
    let mut j = start + 3 + usize::from(bytes.get(start + 3).copied() == Some(b'-'));
    if !lower[j..].starts_with("16") { return None; }
    j += 2;
    if bytes.get(j).copied() == Some(b'-') {
        let mut k = j + 1;
        while bytes.get(k).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') { k += 1; }
        if k > j + 1 { j = k; }
    }
    let mut k = j;
    while bytes.get(k).is_some_and(|&b| b == b' ' || b == b'\t') { k += 1; }
    if k > j && lower[k..].starts_with("polynomial") && !bytes.get(k + 10).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
        j = k + 10;
        let mut m = j;
        while bytes.get(m).is_some_and(|&b| b == b' ' || b == b'\t') { m += 1; }
        if m > j {
            for kw in ["equation", "formula"] {
                if lower[m..].starts_with(kw) && !bytes.get(m + kw.len()).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                    j = m + kw.len();
                    break;
                }
            }
        }
    }
    (!bytes.get(j).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')).then_some(j)
}

/// Replaces low-level `CRC-16` polynomial references and `polynomial equation` phrases (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn replace_crc16_and_poly_phrase(text: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut res, mut last, mut i) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower[i..].find("crc") {
        let start = i + pos;
        if let Some(s_end) = match_crc16_span(&lower, bytes, start) {
            res.push_str(&text[last..start]);
            res.push_str("Frame Integrity Check");
            last = s_end;
            i = s_end;
        } else {
            i = start + 3;
        }
    }
    res.push_str(&text[last..]);
    let (lower2, bytes2) = (res.to_ascii_lowercase(), res.as_bytes().to_vec());
    let (mut out, mut last2, mut idx) = (String::with_capacity(res.len()), 0, 0);
    while let Some(pos) = lower2[idx..].find("polynomial") {
        let (start, end_poly) = (idx + pos, idx + pos + 10);
        if start == 0 || !bytes2.get(start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
            let mut j = end_poly;
            while bytes2.get(j).is_some_and(|&b| b == b' ' || b == b'\t') { j += 1; }
            if j > end_poly && lower2[j..].starts_with("equation") && !bytes2.get(j + 8).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                out.push_str(&res[last2..start]);
                out.push_str("integrity validation");
                last2 = j + 8;
                idx = j + 8;
                continue;
            }
        }
        idx = end_poly;
    }
    out.push_str(&res[last2..]);
    out
}

/// Matches the start index of a numeric baud/bps literal preceding `i` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn match_preceding_baud_number(lower: &str, bytes: &[u8], i: usize, is_baud: bool) -> Option<usize> {
    let mut prev = i - usize::from(!is_baud && i > 0 && bytes.get(i - 1).is_some_and(|&b| matches!(b, b'k' | b'm' | b'g' | b'K' | b'M' | b'G')));
    while prev > 0 && bytes.get(prev - 1).is_some_and(|&b| b == b' ' || b == b'\t') { prev -= 1; }
    let (mut num_start, mut has_digits) = (prev, false);
    while num_start > 0 && bytes.get(num_start - 1).is_some_and(|&b| b.is_ascii_digit() || b == b'.') {
        if bytes.get(num_start - 1).is_some_and(u8::is_ascii_digit) { has_digits = true; }
        num_start -= 1;
    }
    if !has_digits { return None; }
    let (mut start, mut at_cur) = (num_start, num_start);
    while at_cur > 0 && bytes.get(at_cur - 1).is_some_and(|&b| b == b' ' || b == b'\t') { at_cur -= 1; }
    if at_cur >= 2 && lower[at_cur - 2..at_cur] == *"at" && (at_cur == 2 || !bytes.get(at_cur - 3).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')) {
        start = at_cur - 2;
    }
    (start == 0 || !bytes.get(start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')).then_some(start)
}

/// Strips low-level serial baud rates (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_baud_rates(text: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut result, mut last_idx, mut i) = (String::with_capacity(text.len()), 0, 0);
    let bauds = ["9600", "19200", "38400", "57600", "115200", "230400", "460800", "921600"];
    while i < lower.len() {
        if bytes.get(i).copied() == Some(b'_') {
            if let Some(b) = bauds.iter().find(|&&b| lower[i + 1..].starts_with(b) && !bytes.get(i + 1 + b.len()).is_some_and(|&c| c.is_ascii_alphanumeric() || c == b'_')) {
                result.push_str(&text[last_idx..i]);
                last_idx = i + 1 + b.len();
                i = last_idx;
                continue;
            }
        }
        let is_baud = lower[i..].starts_with("baud");
        let is_bps = lower[i..].starts_with("bps");
        if is_baud || is_bps {
            let end = i + if is_baud { 4 } else { 3 };
            if !bytes.get(end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                if let Some(start) = match_preceding_baud_number(&lower, bytes, i, is_baud) {
                    result.push_str(&text[last_idx..start]);
                    last_idx = end;
                    i = end;
                    continue;
                }
            }
        }
        let Some(ch) = text[i..].chars().next() else { break };
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Matches an `opcode` keyword and optional trailing hex/decimal literal (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn match_opcode_span(lower: &str, bytes: &[u8], i: usize) -> Option<usize> {
    if !lower[i..].starts_with("opcode") || (i > 0 && bytes.get(i - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')) {
        return None;
    }
    let j = i + 6 + usize::from(bytes.get(i + 6).is_some_and(|&b| b == b's' || b == b'S'));
    let mut k = j;
    while bytes.get(k).is_some_and(|&b| b == b' ' || b == b'\t') { k += 1; }
    if k > j && k < bytes.len() {
        if lower[k..].starts_with("0x") {
            let mut m = k + 2;
            while bytes.get(m).is_some_and(u8::is_ascii_hexdigit) { m += 1; }
            if m > k + 2 && !bytes.get(m).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') { return Some(m); }
        } else if bytes.get(k).is_some_and(u8::is_ascii_digit) {
            let mut m = k;
            while bytes.get(m).is_some_and(u8::is_ascii_digit) { m += 1; }
            if !bytes.get(m).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') { return Some(m); }
        }
    }
    (!bytes.get(j).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')).then_some(j)
}

/// Strips wire opcodes and Level 2 `UC-xx` tags while preserving `UCL-xx` tags (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_opcodes_and_use_cases(text: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut res, mut last, mut i) = (String::with_capacity(text.len()), 0, 0);
    while i < lower.len() {
        if let Some(end) = match_opcode_span(&lower, bytes, i) {
            res.push_str(&text[last..i]);
            last = end;
            i = end;
            continue;
        }
        let is_hex = (bytes.get(i).copied() == Some(b'_') && i + 2 < bytes.len() && lower[i + 1..].starts_with("0x")) || lower[i..].starts_with("0x");
        if is_hex {
            let mut j = if bytes.get(i).copied() == Some(b'_') { i + 3 } else { i + 2 };
            let start_hex = j;
            while bytes.get(j).is_some_and(u8::is_ascii_hexdigit) { j += 1; }
            if j > start_hex {
                res.push_str(&text[last..i]);
                last = j;
                i = j;
                continue;
            }
        }
        let Some(ch) = text[i..].chars().next() else { break };
        i += ch.len_utf8();
    }
    res.push_str(&text[last..]);
    strip_uc_tags(&res)
}

/// Strips `(UC-01, UC-02)` and standalone `UC-xx` tags (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_uc_tags(text: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut res, mut last, mut i) = (String::with_capacity(text.len()), 0, 0);
    while i < lower.len() {
        if bytes.get(i).copied() == Some(b'(') {
            let (mut j, mut valid_parens) = (i + 1, false);
            while j < bytes.len() {
                while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') { j += 1; }
                if !lower[j..].starts_with("uc-") { break; }
                j += 3;
                let d_start = j;
                while bytes.get(j).is_some_and(u8::is_ascii_digit) { j += 1; }
                if j == d_start { break; }
                while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') { j += 1; }
                if bytes.get(j).copied() == Some(b')') { j += 1; valid_parens = true; break; }
                else if bytes.get(j).is_some_and(|&b| b == b',' || b == b'&') { j += 1; }
                else if lower[j..].starts_with("and") { j += 3; }
                else { break; }
            }
            if valid_parens {
                res.push_str(&text[last..i]);
                last = j;
                i = j;
                continue;
            }
        }
        if lower[i..].starts_with("uc-") && (i == 0 || !bytes.get(i - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')) {
            let mut j = i + 3;
            while bytes.get(j).is_some_and(u8::is_ascii_digit) { j += 1; }
            if j > i + 3 && !bytes.get(j).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                res.push_str(&text[last..i]);
                last = j;
                i = j;
                continue;
            }
        }
        let Some(ch) = text[i..].chars().next() else { break };
        i += ch.len_utf8();
    }
    res.push_str(&text[last..]);
    res
}

/// Cleans empty delimiter pairs, redundant commas, and multi-space runs (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn clean_punctuation_artifacts(mut s: String) -> String {
    for (open, close) in [('(', ')'), ('[', ']')] {
        let (mut out, mut i) = (String::with_capacity(s.len()), 0);
        while let Some(c) = s[i..].chars().next() {
            if c == open {
                let (mut j, mut all_ws, mut close_end) = (i + c.len_utf8(), true, 0);
                while let Some(ch) = s[j..].chars().next() {
                    if ch == close { close_end = j + ch.len_utf8(); break; }
                    if !matches!(ch, ' ' | '\t' | '\r' | '\n') { all_ws = false; break; }
                    j += ch.len_utf8();
                }
                if close_end > 0 && all_ws { i = close_end; continue; }
            }
            out.push(c);
            i += c.len_utf8();
        }
        s = out;
    }
    for (pat, repl) in [("( ,", "("), ("(,", "("), (", )", ")"), (",)", ")"), (", ,", ","), (",,", ","), (" ,", ","), (", .", "."), (",.", ".")] {
        while s.contains(pat) { s = s.replace(pat, repl); }
    }
    let (mut out, mut i, bytes) = (String::with_capacity(s.len()), 0, s.as_bytes());
    while i < bytes.len() {
        if bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') {
            out.push(' ');
            while bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') { i += 1; }
        } else if let Some(ch) = s[i..].chars().next() {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    out
}

/// [DOC-TIER: A] Sanitizes text to enforce Level 1B operational abstraction.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-SANITIZE-LEVEL-1B-OPERATIONAL-TEXT] [FLS: fls_conops_cmark]
/// Contract: Removes CRC polynomials, serial baud rates, wire opcodes, and Level 2 `UC-xx` tags from `text: &str` while normalizing punctuation.
/// Bounds & Faults: O(N) time and space over `text.len()`; preserves `UCL-xx` user class taxonomy tags.
pub fn sanitize_level_1b_operational_text(text: &str) -> String {
    if text.is_empty() { return String::new(); }
    let s = strip_opcodes_and_use_cases(&strip_baud_rates(&replace_crc16_and_poly_phrase(&strip_poly_hex(&strip_poly_equation(text)))));
    clean_punctuation_artifacts(s).trim().to_string()
}

/// Strips inline HTML tags when computing visible Mermaid label length (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_html_tags(s: &str) -> String {
    let (mut result, mut i, bytes) = (String::with_capacity(s.len()), 0, s.as_bytes());
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b'<') {
            if let Some(rel_end) = s[i..].find('>').filter(|&e| e <= 12) {
                i += rel_end + 1;
                continue;
            }
        }
        let Some(ch) = s[i..].chars().next() else { break };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// [DOC-TIER: A] Wraps Mermaid node/edge label text into segments `<= max_width` characters using `<br/>`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-WRAP-MERMAID-LABEL] [FLS: fls_conops_cmark]
/// Contract: Splits `text: &str` at existing line breaks and wraps segments exceeding `max_width` visible characters with `<br/>`.
/// Bounds & Faults: O(N) time and space over `text.len()`; respects UTF-8 character boundaries when splitting long words.
pub fn wrap_mermaid_label(text: &str, max_width: usize) -> String {
    if text.is_empty() { return String::new(); }
    let mut out_segments = Vec::new();
    for seg in text.split('\n') {
        if strip_html_tags(seg).trim().len() <= max_width {
            out_segments.push(seg.to_string());
        } else {
            let (mut lines, mut curr, mut curr_len) = (Vec::new(), Vec::new(), 0usize);
            for w in seg.split_whitespace() {
                let w_len = strip_html_tags(w).len();
                if w_len > max_width {
                    if !curr.is_empty() { lines.push(curr.join(" ")); curr.clear(); curr_len = 0; }
                    let mut c_idx = 0;
                    while c_idx < w.len() {
                        let mut end_idx = (c_idx + max_width).min(w.len());
                        while end_idx < w.len() && !w.is_char_boundary(end_idx) { end_idx += 1; }
                        lines.push(w[c_idx..end_idx].to_string());
                        c_idx = end_idx;
                    }
                } else if !curr.is_empty() && curr_len + 1 + w_len > max_width {
                    lines.push(curr.join(" "));
                    curr = vec![w];
                    curr_len = w_len;
                } else {
                    if !curr.is_empty() { curr_len += 1; }
                    curr.push(w);
                    curr_len += w_len;
                }
            }
            if !curr.is_empty() { lines.push(curr.join(" ")); }
            out_segments.push(lines.join("<br/>"));
        }
    }
    out_segments.join("\n")
}

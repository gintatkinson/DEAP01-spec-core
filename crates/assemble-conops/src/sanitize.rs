//! Level 1B operational abstraction sanitization and Mermaid formatting routines.

fn match_token(lower: &str, mut idx: usize, token: &str) -> Option<usize> {
    let bytes = lower.as_bytes();
    while idx < bytes.len() && (bytes[idx] == b' ' || bytes[idx] == b'\t') {
        idx += 1;
    }
    if lower[idx..].starts_with(token) {
        let after = idx + token.len();
        Some(after)
    } else {
        None
    }
}

fn strip_poly_equation(text: &str) -> String {
    let lower = text.to_lowercase();
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

fn strip_poly_hex(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("0x1021") {
            let mut start = i + pos;
            let end = start + 6;
            if start > 0 && bytes[start - 1] == b'_' {
                start -= 1;
            }
            let right_ok = end == bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
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

fn replace_crc16(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        let mut found = None;
        if let Some(pos) = lower[i..].find("crc") {
            let start = i + pos;
            let left_ok = start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
            if left_ok {
                let mut j = start + 3;
                if j < bytes.len() && bytes[j] == b'-' {
                    j += 1;
                }
                if lower[j..].starts_with("16") {
                    j += 2;
                    if j < bytes.len() && bytes[j] == b'-' {
                        let mut k = j + 1;
                        while k < bytes.len() && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                            k += 1;
                        }
                        if k > j + 1 {
                            j = k;
                        }
                    }
                    let mut k = j;
                    while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
                        k += 1;
                    }
                    if k > j && lower[k..].starts_with("polynomial") {
                        let poly_end = k + 10;
                        let right_poly_ok = poly_end == bytes.len() || !(bytes[poly_end].is_ascii_alphanumeric() || bytes[poly_end] == b'_');
                        if right_poly_ok {
                            j = poly_end;
                            let mut m = j;
                            while m < bytes.len() && (bytes[m] == b' ' || bytes[m] == b'\t') {
                                m += 1;
                            }
                            if m > j {
                                if lower[m..].starts_with("equation") {
                                    let eq_end = m + 8;
                                    let right_eq = eq_end == bytes.len() || !(bytes[eq_end].is_ascii_alphanumeric() || bytes[eq_end] == b'_');
                                    if right_eq {
                                        j = eq_end;
                                    }
                                } else if lower[m..].starts_with("formula") {
                                    let form_end = m + 7;
                                    let right_form = form_end == bytes.len() || !(bytes[form_end].is_ascii_alphanumeric() || bytes[form_end] == b'_');
                                    if right_form {
                                        j = form_end;
                                    }
                                }
                            }
                        }
                    }

                    let right_ok = j == bytes.len() || !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_');
                    if right_ok {
                        found = Some((start, j));
                    }
                }
            }
            if let Some((s_start, s_end)) = found {
                result.push_str(&text[last_idx..s_start]);
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

fn replace_polynomial_equation(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("polynomial") {
            let start = i + pos;
            let end_poly = start + 10;
            let left_ok = start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
            if left_ok {
                let mut j = end_poly;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j > end_poly && lower[j..].starts_with("equation") {
                    let end = j + 8;
                    let right_ok = end == bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
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

fn strip_baud_rates(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    let standard_bauds = ["9600", "19200", "38400", "57600", "115200", "230400", "460800", "921600"];

    while i < lower.len() {
        if bytes[i] == b'_' {
            let mut matched_baud = false;
            for baud in &standard_bauds {
                if lower[i + 1..].starts_with(baud) {
                    let end = i + 1 + baud.len();
                    let right_ok = end == bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
                    if right_ok {
                        result.push_str(&text[last_idx..i]);
                        last_idx = end;
                        i = end;
                        matched_baud = true;
                        break;
                    }
                }
            }
            if matched_baud {
                continue;
            }
        }

        let is_baud = lower[i..].starts_with("baud");
        let is_bps = lower[i..].starts_with("bps");

        if is_baud || is_bps {
            let suffix_len = if is_baud { 4 } else { 3 };
            let end = i + suffix_len;
            let right_ok = end == bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');

            if right_ok {
                let mut prev = i;
                if is_baud && prev > 0 && (bytes[prev - 1] == b'k' || bytes[prev - 1] == b'm' || bytes[prev - 1] == b'g' || bytes[prev - 1] == b'K' || bytes[prev - 1] == b'M' || bytes[prev - 1] == b'G') {
                    prev -= 1;
                }
                while prev > 0 && (bytes[prev - 1] == b' ' || bytes[prev - 1] == b'\t') {
                    prev -= 1;
                }
                let num_end = prev;
                let mut num_start = num_end;
                let mut has_digits = false;
                while num_start > 0 && (bytes[num_start - 1].is_ascii_digit() || bytes[num_start - 1] == b'.') {
                    if bytes[num_start - 1].is_ascii_digit() {
                        has_digits = true;
                    }
                    num_start -= 1;
                }

                if has_digits {
                    let mut start = num_start;
                    let mut at_cursor = num_start;
                    while at_cursor > 0 && (bytes[at_cursor - 1] == b' ' || bytes[at_cursor - 1] == b'\t') {
                        at_cursor -= 1;
                    }
                    if at_cursor >= 2 && lower[at_cursor - 2..at_cursor] == *"at" {
                        let left_at_ok = at_cursor == 2 || !(bytes[at_cursor - 3].is_ascii_alphanumeric() || bytes[at_cursor - 3] == b'_');
                        if left_at_ok {
                            start = at_cursor - 2;
                        }
                    }
                    let left_ok = start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
                    if left_ok {
                        result.push_str(&text[last_idx..start]);
                        last_idx = end;
                        i = end;
                        continue;
                    }
                }
            }
        }

        let ch = text[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

fn strip_opcodes(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        let is_opcode = lower[i..].starts_with("opcode");
        if is_opcode {
            let left_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            if left_ok {
                let mut j = i + 6;
                if j < bytes.len() && (bytes[j] == b's' || bytes[j] == b'S') {
                    j += 1;
                }
                let mut k = j;
                while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
                    k += 1;
                }
                if k > j && k < bytes.len() {
                    if lower[k..].starts_with("0x") {
                        let mut m = k + 2;
                        while m < bytes.len() && bytes[m].is_ascii_hexdigit() {
                            m += 1;
                        }
                        if m > k + 2 {
                            let right_ok = m == bytes.len() || !(bytes[m].is_ascii_alphanumeric() || bytes[m] == b'_');
                            if right_ok {
                                result.push_str(&text[last_idx..i]);
                                last_idx = m;
                                i = m;
                                continue;
                            }
                        }
                    } else if bytes[k].is_ascii_digit() {
                        let mut m = k;
                        while m < bytes.len() && bytes[m].is_ascii_digit() {
                            m += 1;
                        }
                        let right_ok = m == bytes.len() || !(bytes[m].is_ascii_alphanumeric() || bytes[m] == b'_');
                        if right_ok {
                            result.push_str(&text[last_idx..i]);
                            last_idx = m;
                            i = m;
                            continue;
                        }
                    }
                }
                let right_word_ok = j == bytes.len() || !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_');
                if right_word_ok {
                    result.push_str(&text[last_idx..i]);
                    last_idx = j;
                    i = j;
                    continue;
                }
            }
        }

        let is_hex_lead = (bytes[i] == b'_' && i + 2 < bytes.len() && lower[i + 1..].starts_with("0x"))
            || lower[i..].starts_with("0x");
        if is_hex_lead {
            let start = i;
            let mut j = if bytes[i] == b'_' { i + 3 } else { i + 2 };
            let mut has_hex = false;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                has_hex = true;
                j += 1;
            }
            if has_hex {
                result.push_str(&text[last_idx..start]);
                last_idx = j;
                i = j;
                continue;
            }
        }

        let ch = text[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

fn strip_use_cases(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if bytes[i] == b'(' {
            let start = i;
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            let mut valid_uc_parens = false;
            if lower[j..].starts_with("uc-") {
                while j < bytes.len() {
                    while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                        j += 1;
                    }
                    if lower[j..].starts_with("uc-") {
                        j += 3;
                        let digit_start = j;
                        while j < bytes.len() && bytes[j].is_ascii_digit() {
                            j += 1;
                        }
                        if j == digit_start {
                            break;
                        }
                        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                            j += 1;
                        }
                        if j < bytes.len() && bytes[j] == b')' {
                            valid_uc_parens = true;
                            j += 1;
                            break;
                        } else if j < bytes.len() && (bytes[j] == b',' || bytes[j] == b'&') {
                            j += 1;
                            continue;
                        } else if lower[j..].starts_with("and") {
                            j += 3;
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            if valid_uc_parens {
                result.push_str(&text[last_idx..start]);
                last_idx = j;
                i = j;
                continue;
            }
        }

        let is_uc = lower[i..].starts_with("uc-");
        if is_uc {
            let left_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            if left_ok {
                let mut j = i + 3;
                let digit_start = j;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > digit_start {
                    let right_ok = j == bytes.len() || !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_');
                    if right_ok {
                        result.push_str(&text[last_idx..i]);
                        last_idx = j;
                        i = j;
                        continue;
                    }
                }
            }
        }

        let ch = text[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

fn strip_empty_delimited(text: &str, open: char, close: char) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == open {
            let j = i + c.len_utf8();
            let mut all_ws = true;
            let mut found_close = false;
            let mut close_end = j;
            for (idx, ch) in text[j..].char_indices() {
                if ch == close {
                    found_close = true;
                    close_end = j + idx + ch.len_utf8();
                    break;
                } else if ch != ' ' && ch != '\t' && ch != '\r' && ch != '\n' {
                    all_ws = false;
                    break;
                }
            }
            if found_close && all_ws {
                while let Some(&(k, _)) = chars.peek() {
                    if k < close_end {
                        chars.next();
                    } else {
                        break;
                    }
                }
                continue;
            }
        }
        result.push(c);
    }
    result
}

fn clean_lead_comma_paren(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            result.push('(');
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b',' {
                while j < bytes.len() && (bytes[j] == b',' || bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                i = j;
                continue;
            }
            i += 1;
            continue;
        }
        let ch = text[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn clean_trail_comma_paren(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            let mut j = i;
            while j < bytes.len() && (bytes[j] == b',' || bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b')' {
                result.push(')');
                i = j + 1;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn clean_multi_commas(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b',' {
                while j < bytes.len() && (bytes[j] == b',' || bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                result.push(',');
                i = j;
                continue;
            }
            result.push(',');
            i += 1;
            continue;
        }
        let ch = text[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn clean_space_commas(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            let mut j = i;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b',' {
                j += 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                result.push_str(", ");
                i = j;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn clean_comma_period(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                result.push('.');
                i = j + 1;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn clean_multi_spaces(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            result.push(' ');
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            i = j;
            continue;
        }
        let ch = text[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'/' {
                j += 1;
            }
            let tag_name_start = j;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-') {
                j += 1;
            }
            if j > tag_name_start {
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'/' {
                    j += 1;
                }
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'>' {
                    i = j + 1;
                    continue;
                }
            }
        }
        let ch = s[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Sanitizes text to enforce Level 1B operational abstraction.
/// Strips component-internal serial opcodes, baud rates, CRC-16 polynomial formulas,
/// and Level 2 System Use Cases (uc-xx / (UC-xx)).
pub fn sanitize_level_1b_operational_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut s = text.to_string();

    // 1. Remove polynomial equations
    s = strip_poly_equation(&s);
    s = strip_poly_hex(&s);
    s = replace_crc16(&s);
    s = replace_polynomial_equation(&s);

    // 2. Remove baud rates
    s = strip_baud_rates(&s);

    // 3. Remove serial opcodes
    s = strip_opcodes(&s);

    // 4. Remove Level 2 System Use Cases (e.g. (UC-01 and UC-03), uc-01, UC-01)
    s = strip_use_cases(&s);

    // 5. Clean up punctuation artifacts, empty parens/brackets, duplicate commas, double spaces
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

/// Wraps text into segments <= max_width characters using `<br/>`.
/// Enforces Rule E2 / Rule E3 visual ergonomics.
pub fn wrap_mermaid_label(text: &str, max_width: usize) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut out_parts = Vec::new();
    let mut last_idx = 0;
    let bytes = text.as_bytes();
    let lower = text.to_lowercase();
    let mut i = 0;

    while i < bytes.len() {
        let mut split_len = None;
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            split_len = Some(2);
        } else if bytes[i] == b'\n' {
            split_len = Some(1);
        } else if bytes[i] == b'<' && lower[i..].starts_with("<br") {
            let mut j = i + 3;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'/' {
                j += 1;
            }
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'>' {
                split_len = Some(j + 1 - i);
            }
        }

        if let Some(len) = split_len {
            if i > last_idx {
                let seg = &text[last_idx..i];
                process_segment(seg, max_width, &mut out_parts);
            }
            out_parts.push(text[i..i + len].to_string());
            last_idx = i + len;
            i = last_idx;
            continue;
        }

        let ch = text[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }

    if last_idx < text.len() {
        let seg = &text[last_idx..];
        process_segment(seg, max_width, &mut out_parts);
    }

    out_parts.join("")
}

fn process_segment(
    seg: &str,
    max_width: usize,
    out_parts: &mut Vec<String>,
) {
    let clean_seg = strip_html_tags(seg).trim().to_string();
    if clean_seg.len() <= max_width {
        out_parts.push(seg.to_string());
    } else {
        let words: Vec<&str> = seg.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut curr: Vec<&str> = Vec::new();
        let mut curr_len = 0;

        for w in words {
            let w_clean = strip_html_tags(w);
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

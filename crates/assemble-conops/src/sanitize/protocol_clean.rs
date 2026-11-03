//! Wire protocol opcode, serial baud rate, and Level 2 use-case tag stripping helpers for Level 1B ConOps text.

/// Matches an `opcode` keyword and optional trailing hex/decimal literal starting at `i` (`[REQ-ASSEMBLE-CONOPS]`).
fn match_opcode_span(lower: &str, bytes: &[u8], i: usize) -> Option<usize> {
    if !lower[i..].starts_with("opcode") {
        return None;
    }
    let left_ok =
        i == 0 || !bytes.get(i - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
    if !left_ok {
        return None;
    }
    let mut j = i + 6;
    if bytes.get(j).is_some_and(|&b| b == b's' || b == b'S') {
        j += 1;
    }
    let mut k = j;
    while bytes.get(k).is_some_and(|&b| b == b' ' || b == b'\t') {
        k += 1;
    }
    if k > j && k < bytes.len() {
        if lower[k..].starts_with("0x") {
            let mut m = k + 2;
            while bytes.get(m).is_some_and(u8::is_ascii_hexdigit) {
                m += 1;
            }
            if m > k + 2 && !bytes.get(m).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                return Some(m);
            }
        } else if bytes.get(k).is_some_and(u8::is_ascii_digit) {
            let mut m = k;
            while bytes.get(m).is_some_and(u8::is_ascii_digit) {
                m += 1;
            }
            if !bytes.get(m).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                return Some(m);
            }
        }
    }
    if !bytes.get(j).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
        Some(j)
    } else {
        None
    }
}

/// Strips low-level wire protocol opcodes and hexadecimal opcode constants.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-OPCODES] | SysML v2: ActionDef(strip_opcodes)
pub(super) fn strip_opcodes(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(end) = match_opcode_span(&lower, bytes, i) {
            result.push_str(&text[last_idx..i]);
            last_idx = end;
            i = end;
            continue;
        }

        let is_hex_lead = (bytes.get(i).copied() == Some(b'_')
            && i + 2 < bytes.len()
            && lower[i + 1..].starts_with("0x"))
            || lower[i..].starts_with("0x");
        if is_hex_lead {
            let start = i;
            let mut j = if bytes.get(i).copied() == Some(b'_') { i + 3 } else { i + 2 };
            let mut has_hex = false;
            while bytes.get(j).is_some_and(u8::is_ascii_hexdigit) {
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

        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Matches a parenthesized list of `UC-xx` tags `(UC-01, UC-02)` starting at `i` (`[REQ-ASSEMBLE-CONOPS]`).
fn match_uc_parens_span(lower: &str, bytes: &[u8], i: usize) -> Option<usize> {
    if bytes.get(i).copied() != Some(b'(') {
        return None;
    }
    let mut j = i + 1;
    while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
        j += 1;
    }
    if !lower[j..].starts_with("uc-") {
        return None;
    }
    while j < bytes.len() {
        while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
            j += 1;
        }
        if !lower[j..].starts_with("uc-") {
            return None;
        }
        j += 3;
        let digit_start = j;
        while bytes.get(j).is_some_and(u8::is_ascii_digit) {
            j += 1;
        }
        if j == digit_start {
            return None;
        }
        while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
            j += 1;
        }
        if bytes.get(j).copied() == Some(b')') {
            return Some(j + 1);
        } else if bytes.get(j).is_some_and(|&b| b == b',' || b == b'&') {
            j += 1;
        } else if lower[j..].starts_with("and") {
            j += 3;
        } else {
            return None;
        }
    }
    None
}

/// Strips Level 2 System Use Case tags (`UC-xx`) while preserving User Class (`UCL-xx`) tags.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-USE-CASES] | SysML v2: ActionDef(strip_use_cases)
pub(super) fn strip_use_cases(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(end) = match_uc_parens_span(&lower, bytes, i) {
            result.push_str(&text[last_idx..i]);
            last_idx = end;
            i = end;
            continue;
        }

        if lower[i..].starts_with("uc-") {
            let left_ok = i == 0
                || !bytes.get(i - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
            if left_ok {
                let mut j = i + 3;
                let digit_start = j;
                while bytes.get(j).is_some_and(u8::is_ascii_digit) {
                    j += 1;
                }
                if j > digit_start
                    && !bytes.get(j).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    result.push_str(&text[last_idx..i]);
                    last_idx = j;
                    i = j;
                    continue;
                }
            }
        }

        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

/// Matches the start index of a numeric baud/bps literal (and optional `at `) preceding `i` (`[REQ-ASSEMBLE-CONOPS]`).
fn match_preceding_baud_number(lower: &str, bytes: &[u8], i: usize, is_baud: bool) -> Option<usize> {
    let mut prev = i;
    if is_baud
        && prev > 0
        && bytes
            .get(prev - 1)
            .is_some_and(|&b| matches!(b, b'k' | b'm' | b'g' | b'K' | b'M' | b'G'))
    {
        prev -= 1;
    }
    while prev > 0 && bytes.get(prev - 1).is_some_and(|&b| b == b' ' || b == b'\t') {
        prev -= 1;
    }
    let mut num_start = prev;
    let mut has_digits = false;
    while num_start > 0
        && bytes
            .get(num_start - 1)
            .is_some_and(|&b| b.is_ascii_digit() || b == b'.')
    {
        if bytes.get(num_start - 1).is_some_and(u8::is_ascii_digit) {
            has_digits = true;
        }
        num_start -= 1;
    }
    if !has_digits {
        return None;
    }
    let mut start = num_start;
    let mut at_cursor = num_start;
    while at_cursor > 0 && bytes.get(at_cursor - 1).is_some_and(|&b| b == b' ' || b == b'\t') {
        at_cursor -= 1;
    }
    if at_cursor >= 2 && lower[at_cursor - 2..at_cursor] == *"at" {
        let left_at_ok = at_cursor == 2
            || !bytes
                .get(at_cursor - 3)
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
        if left_at_ok {
            start = at_cursor - 2;
        }
    }
    let left_ok = start == 0
        || !bytes
            .get(start - 1)
            .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
    left_ok.then_some(start)
}

/// Strips low-level serial baud rates and `bps`/`baud` link speed literals.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-BAUD-RATES] | SysML v2: ActionDef(strip_baud_rates)
pub(super) fn strip_baud_rates(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;
    let standard_bauds = [
        "9600", "19200", "38400", "57600", "115200", "230400", "460800", "921600",
    ];

    while i < lower.len() {
        if bytes.get(i).copied() == Some(b'_') {
            let mut matched_baud = false;
            for baud in &standard_bauds {
                if lower[i + 1..].starts_with(baud) {
                    let end = i + 1 + baud.len();
                    if !bytes.get(end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
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

        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        i += ch.len_utf8();
    }
    result.push_str(&text[last_idx..]);
    result
}

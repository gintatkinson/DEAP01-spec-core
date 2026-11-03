//! Low-level SysML v2 lexical and structural span helpers for ConOps parameter extraction.

/// Checks if a byte is a valid ASCII identifier character (`[A-Za-z0-9_]`).
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-IS-IDENT-CHAR] | SysML v2: ActionDef(is_ident_char)
pub(super) fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Finds the first `package <Identifier>` declaration in a SysML source string.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-FIND-FIRST-PACKAGE] | SysML v2: ActionDef(find_first_package)
pub(super) fn find_first_package(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(pos) = text.get(i..).and_then(|s| s.find("package")) {
            let start = i + pos;
            let end = start + 7;
            let left_ok = start == 0 || !bytes.get(start - 1).copied().is_some_and(is_ident_char);
            let right_ok = bytes.get(end).is_some_and(u8::is_ascii_whitespace);
            if left_ok && right_ok {
                let mut j = end;
                while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                    j += 1;
                }
                let ident_start = j;
                while bytes.get(j).copied().is_some_and(is_ident_char) {
                    j += 1;
                }
                if j > ident_start {
                    return text.get(ident_start..j);
                }
            }
            i = end;
        } else {
            break;
        }
    }
    None
}

/// Extracts all `attribute <name> [: <Type>] = <value>;` bindings from a SysML block.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-FIND-ALL-ATTRIBUTES] | SysML v2: ActionDef(find_all_attributes)
pub(super) fn find_all_attributes(text: &str) -> Vec<(&str, &str)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(pos) = text.get(i..).and_then(|s| s.find("attribute")) {
            let start = i + pos;
            let end = start + 9;
            let left_ok = start == 0 || !bytes.get(start - 1).copied().is_some_and(is_ident_char);
            let right_ok = bytes.get(end).is_some_and(u8::is_ascii_whitespace);
            if left_ok && right_ok {
                let mut j = end;
                while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                    j += 1;
                }
                let name_start = j;
                while bytes.get(j).copied().is_some_and(is_ident_char) {
                    j += 1;
                }
                if j > name_start {
                    if let Some(attr_name) = text.get(name_start..j) {
                        let mut k = j;
                        while bytes.get(k).is_some_and(u8::is_ascii_whitespace) {
                            k += 1;
                        }
                        if bytes.get(k) == Some(&b':') {
                            k += 1;
                            while bytes.get(k).is_some_and(u8::is_ascii_whitespace) {
                                k += 1;
                            }
                            while bytes.get(k).copied().is_some_and(is_ident_char) {
                                k += 1;
                            }
                            while bytes.get(k).is_some_and(u8::is_ascii_whitespace) {
                                k += 1;
                            }
                        }
                        if bytes.get(k) == Some(&b'=') {
                            k += 1;
                            let val_start = k;
                            while bytes.get(k).is_some_and(|&b| b != b';' && b != b'\n') {
                                k += 1;
                            }
                            if bytes.get(k) == Some(&b';') {
                                if let Some(val_slice) = text.get(val_start..k) {
                                    let raw_val = val_slice.trim().trim_matches(['"', '\'']);
                                    results.push((attr_name, raw_val));
                                    i = k + 1;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            i = end;
        } else {
            break;
        }
    }
    results
}

/// [DOC-TIER: A] Calculates the 1-indexed line number of a byte offset within `content`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-FIND-LINE-NUMBER] | SysML v2: ActionDef(find_line_number)
/// Contract: Counts newline bytes preceding `byte_offset` in `content: &str` and returns `Some(line)` or `None` if out of bounds.
/// Bounds & Faults: O(byte_offset) time and O(1) auxiliary space; safe slice bounds check via `.get(..byte_offset)`.
pub fn find_line_number(content: &str, byte_offset: usize) -> Option<usize> {
    let prefix = content.as_bytes().get(..byte_offset)?;
    let line = prefix.iter().filter(|&&b| b == b'\n').count() + 1;
    Some(line)
}

/// Extracts a preceding `/* ... */` or `doc /* ... */` comment immediately before `pos`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-DOC-COMMENT-BEFORE] | SysML v2: ActionDef(extract_doc_comment_before)
pub(super) fn extract_doc_comment_before(content: &str, pos: usize) -> String {
    let before = content.get(..pos).unwrap_or("").trim_end();
    if let Some(doc_end) = before.strip_suffix("*/") {
        if let Some(start_pos) = doc_end.rfind("/*") {
            let inner = doc_end.get(start_pos + 2..).unwrap_or("");
            let inner_trimmed = inner.trim();
            let stripped = inner_trimmed
                .strip_prefix("doc")
                .unwrap_or(inner_trimmed)
                .trim();
            return stripped.to_string();
        }
    }
    String::new()
}

/// Extracts the first inner `doc /* ... */` comment inside a block body.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-INNER-DOC] | SysML v2: ActionDef(extract_inner_doc)
pub(super) fn extract_inner_doc(body: &str) -> String {
    if let Some(start_pos) = body.find("/*") {
        if let Some(end_pos) = body.get(start_pos + 2..).and_then(|s| s.find("*/")) {
            let inner = body
                .get(start_pos + 2..start_pos + 2 + end_pos)
                .unwrap_or("");
            let inner_trimmed = inner.trim();
            let stripped = inner_trimmed
                .strip_prefix("doc")
                .unwrap_or(inner_trimmed)
                .trim();
            return stripped.to_string();
        }
    }
    String::new()
}

/// Finds the matching closing `}` for an opening `{` at `open_pos`, respecting comments and string literals.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-FIND-MATCHING-BRACE] | SysML v2: ActionDef(find_matching_brace)
pub(super) fn find_matching_brace(content: &str, open_pos: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    if bytes.get(open_pos) != Some(&b'{') {
        return None;
    }
    let mut depth = 0;
    let mut i = open_pos;
    let mut in_str = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(&b) = bytes.get(i) {
        if in_line_comment {
            if b == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if b == b'*' && bytes.get(i + 1) == Some(&b'/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_str {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if b == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        if b == b'/' {
            if bytes.get(i + 1) == Some(&b'/') {
                in_line_comment = true;
                i += 2;
                continue;
            } else if bytes.get(i + 1) == Some(&b'*') {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }

        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Detects a prefix port direction keyword (`inout`, `out`, `in`) immediately preceding `port`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-CHECK-DIRECTION-WORD] | SysML v2: ActionDef(check_direction_word)
pub(super) fn check_direction_word(before: &str) -> Option<(&'static str, usize)> {
    let trimmed = before.trim_end();
    if trimmed.ends_with("inout") {
        let start = trimmed.len() - 5;
        if start == 0
            || !trimmed
                .as_bytes()
                .get(start - 1)
                .copied()
                .is_some_and(is_ident_char)
        {
            return Some(("INOUT", start));
        }
    }
    if trimmed.ends_with("out") {
        let start = trimmed.len() - 3;
        if start == 0
            || !trimmed
                .as_bytes()
                .get(start - 1)
                .copied()
                .is_some_and(is_ident_char)
        {
            return Some(("OUT", start));
        }
    }
    if trimmed.ends_with("in") {
        let start = trimmed.len() - 2;
        if start == 0
            || !trimmed
                .as_bytes()
                .get(start - 1)
                .copied()
                .is_some_and(is_ident_char)
        {
            return Some(("IN", start));
        }
    }
    None
}

/// Extracts the first signed/decimal numeric substring from a raw attribute value string.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-NUMERIC-SUBSTRING] | SysML v2: ActionDef(extract_numeric_substring)
pub(super) fn extract_numeric_substring(val: &str) -> Option<&str> {
    let bytes = val.as_bytes();
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        let is_sign = (b == b'+' || b == b'-')
            && (bytes.get(i + 1).is_some_and(u8::is_ascii_digit)
                || (bytes.get(i + 1) == Some(&b'.')
                    && bytes.get(i + 2).is_some_and(u8::is_ascii_digit)));
        let is_digit = b.is_ascii_digit();
        let is_dot_digit = b == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit);

        if is_sign || is_digit || is_dot_digit {
            let start = i;
            let mut j = if is_sign { i + 1 } else { i };
            let mut has_dot = false;
            let mut has_digits = false;
            while let Some(&bj) = bytes.get(j) {
                if bj.is_ascii_digit() {
                    has_digits = true;
                    j += 1;
                } else if bj == b'.' && !has_dot {
                    if has_digits || bytes.get(j + 1).is_some_and(u8::is_ascii_digit) {
                        has_dot = true;
                        j += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if has_digits {
                return val.get(start..j);
            }
        }
        i += 1;
    }
    None
}

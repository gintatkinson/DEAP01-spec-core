//! Punctuation artifact cleanup and Mermaid diagram label wrapping helpers.

/// Strips empty delimiter pairs containing only whitespace (`()` or `[]`).
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-EMPTY-DELIMITED] | SysML v2: ActionDef(strip_empty_delimited)
pub(super) fn strip_empty_delimited(text: &str, open: char, close: char) -> String {
    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let Some(c) = text[i..].chars().next() else {
            break;
        };
        if c == open {
            let mut j = i + c.len_utf8();
            let mut all_ws = true;
            let mut found_close = false;
            let mut close_end = j;
            while j < text.len() {
                let Some(ch) = text[j..].chars().next() else {
                    break;
                };
                if ch == close {
                    found_close = true;
                    close_end = j + ch.len_utf8();
                    break;
                } else if ch != ' ' && ch != '\t' && ch != '\r' && ch != '\n' {
                    all_ws = false;
                    break;
                }
                j += ch.len_utf8();
            }
            if found_close && all_ws {
                i = close_end;
                continue;
            }
        }
        result.push(c);
        i += c.len_utf8();
    }
    result
}

/// Removes leading commas immediately following an opening parenthesis.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-CLEAN-LEAD-COMMA-PAREN] | SysML v2: ActionDef(clean_lead_comma_paren)
pub(super) fn clean_lead_comma_paren(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b'(') {
            result.push('(');
            let mut j = i + 1;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b',') {
                while bytes
                    .get(j)
                    .is_some_and(|&b| b == b',' || b == b' ' || b == b'\t')
                {
                    j += 1;
                }
                i = j;
                continue;
            }
            i += 1;
            continue;
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Removes trailing commas immediately preceding a closing parenthesis.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-CLEAN-TRAIL-COMMA-PAREN] | SysML v2: ActionDef(clean_trail_comma_paren)
pub(super) fn clean_trail_comma_paren(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b',') {
            let mut j = i;
            while bytes
                .get(j)
                .is_some_and(|&b| b == b',' || b == b' ' || b == b'\t')
            {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b')') {
                result.push(')');
                i = j + 1;
                continue;
            }
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Collapses consecutive comma sequences separated only by horizontal whitespace.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-CLEAN-MULTI-COMMAS] | SysML v2: ActionDef(clean_multi_commas)
pub(super) fn clean_multi_commas(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b',') {
            let mut j = i + 1;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b',') {
                while bytes
                    .get(j)
                    .is_some_and(|&b| b == b',' || b == b' ' || b == b'\t')
                {
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
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Normalizes spaces preceding commas into standard `, ` spacing.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-CLEAN-SPACE-COMMAS] | SysML v2: ActionDef(clean_space_commas)
pub(super) fn clean_space_commas(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') {
            let mut j = i;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b',') {
                j += 1;
                while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                    j += 1;
                }
                result.push_str(", ");
                i = j;
                continue;
            }
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Replaces `, .` punctuation sequences with a single period `.`.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-CLEAN-COMMA-PERIOD] | SysML v2: ActionDef(clean_comma_period)
pub(super) fn clean_comma_period(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b',') {
            let mut j = i + 1;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b'.') {
                result.push('.');
                i = j + 1;
                continue;
            }
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Collapses runs of horizontal whitespace into a single ASCII space.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-CLEAN-MULTI-SPACES] | SysML v2: ActionDef(clean_multi_spaces)
pub(super) fn clean_multi_spaces(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') {
            result.push(' ');
            let mut j = i + 1;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            i = j;
            continue;
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Strips inline HTML tags when computing visible Mermaid label length.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-STRIP-HTML-TAGS] | SysML v2: ActionDef(strip_html_tags)
fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b'<') {
            let mut j = i + 1;
            if bytes.get(j).copied() == Some(b'/') {
                j += 1;
            }
            let tag_name_start = j;
            while bytes
                .get(j)
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            {
                j += 1;
            }
            if j > tag_name_start {
                while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                    j += 1;
                }
                if bytes.get(j).copied() == Some(b'/') {
                    j += 1;
                }
                while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                    j += 1;
                }
                if bytes.get(j).copied() == Some(b'>') {
                    i = j + 1;
                    continue;
                }
            }
        }
        let Some(ch) = s[i..].chars().next() else {
            break;
        };
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// [DOC-TIER: A] Wraps Mermaid node/edge label text into segments `<= max_width` characters using `<br/>`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-WRAP-MERMAID-LABEL] | SysML v2: ActionDef(wrap_mermaid_label)
/// Contract: Splits `text: &str` at existing line breaks and wraps segments exceeding `max_width` visible characters with `<br/>`.
/// Bounds & Faults: O(N) time and space over `text.len()`; respects UTF-8 character boundaries when splitting long words.
pub fn wrap_mermaid_label(text: &str, max_width: usize) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out_parts = Vec::new();
    let mut last_idx = 0;
    let bytes = text.as_bytes();
    let lower = text.to_ascii_lowercase();
    let mut i = 0;

    while i < bytes.len() {
        let mut split_len = None;
        if bytes.get(i).copied() == Some(b'\r') && bytes.get(i + 1).copied() == Some(b'\n') {
            split_len = Some(2);
        } else if bytes.get(i).copied() == Some(b'\n') {
            split_len = Some(1);
        } else if bytes.get(i).copied() == Some(b'<') && lower[i..].starts_with("<br") {
            let mut j = i + 3;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b'/') {
                j += 1;
            }
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if bytes.get(j).copied() == Some(b'>') {
                split_len = Some(j + 1 - i);
            }
        }
        if let Some(len) = split_len {
            if i > last_idx {
                process_segment(&text[last_idx..i], max_width, &mut out_parts);
            }
            out_parts.push(text[i..i + len].to_string());
            last_idx = i + len;
            i = last_idx;
            continue;
        }
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        i += ch.len_utf8();
    }
    if last_idx < text.len() {
        process_segment(&text[last_idx..], max_width, &mut out_parts);
    }
    out_parts.join("")
}

/// Wraps a single line segment of a Mermaid label to `max_width` visible characters.
/// Realises: [REQ-ASSEMBLE-CONOPS-SANITIZE-PROCESS-SEGMENT] | SysML v2: ActionDef(process_segment)
fn process_segment(seg: &str, max_width: usize, out_parts: &mut Vec<String>) {
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
                    let mut end_idx = (c_idx + max_width).min(w.len());
                    while end_idx < w.len() && !w.is_char_boundary(end_idx) {
                        end_idx += 1;
                    }
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

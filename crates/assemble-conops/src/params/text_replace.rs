//! Deterministic template placeholder and case-insensitive word-boundary replacement helpers.

/// Replaces `{{KEY}}` and `{{KEY:DEFAULT}}` tokens using `resolver`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-REPLACE-PLACEHOLDERS] | SysML v2: ActionDef(replace_placeholders)
pub(super) fn replace_placeholders<F>(text: &str, mut resolver: F) -> (String, bool)
where
    F: FnMut(&str, Option<&str>) -> String,
{
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut idx = 0;
    let mut changed = false;

    while idx < bytes.len() {
        if idx + 4 <= bytes.len()
            && bytes.get(idx) == Some(&b'{')
            && bytes.get(idx + 1) == Some(&b'{')
        {
            let start = idx;
            let mut j = idx + 2;
            let mut found_end = None;
            while j + 1 < bytes.len() {
                if bytes.get(j) == Some(&b'}') && bytes.get(j + 1) == Some(&b'}') {
                    found_end = Some(j);
                    break;
                }
                if bytes.get(j) == Some(&b'\n') {
                    break;
                }
                j += 1;
            }

            if let Some(end_pos) = found_end {
                let inner = text.get(start + 2..end_pos).unwrap_or("");
                let (key, default_val) = match inner.split_once(':') {
                    Some((k, d)) => (k, Some(d)),
                    None => (inner, None),
                };
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    let replacement = resolver(key, default_val);
                    result.push_str(&replacement);
                    changed = true;
                    idx = end_pos + 2;
                    continue;
                }
            }
        }

        let Some(ch) = text.get(idx..).and_then(|s| s.chars().next()) else {
            break;
        };
        result.push(ch);
        idx += ch.len_utf8();
    }

    (result, changed)
}

/// Replaces whole-word occurrences of `target_word` case-insensitively with `replacement`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-REPLACE-WORD-CASE-INSENSITIVE] | SysML v2: ActionDef(replace_word_case_insensitive)
pub(super) fn replace_word_case_insensitive(
    text: &str,
    target_word: &str,
    replacement: &str,
) -> String {
    let lower_text = text.to_ascii_lowercase();
    let lower_target = target_word.to_ascii_lowercase();
    let target_len = lower_target.len();
    let bytes = text.as_bytes();

    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut search_idx = 0;

    while search_idx < lower_text.len() {
        if let Some(pos) = lower_text
            .get(search_idx..)
            .and_then(|s| s.find(&lower_target))
        {
            let start = search_idx + pos;
            let end = start + target_len;

            let left_ok = start == 0
                || !bytes
                    .get(start - 1)
                    .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
            let right_ok = end == bytes.len()
                || !bytes
                    .get(end)
                    .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');

            if left_ok && right_ok {
                if let Some(prefix) = text.get(last_idx..start) {
                    result.push_str(prefix);
                }
                result.push_str(replacement);
                last_idx = end;
                search_idx = end;
            } else {
                search_idx = start + 1;
            }
        } else {
            break;
        }
    }
    if let Some(tail) = text.get(last_idx..) {
        result.push_str(tail);
    }
    result
}

/// Replaces `"altitude AGL"` or `"m AGL"` occurrences with `replacement`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-REPLACE-ALTITUDE-AGL] | SysML v2: ActionDef(replace_altitude_agl)
pub(super) fn replace_altitude_agl(text: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower.get(i..).and_then(|s| s.find("agl")) {
            let agl_start = i + pos;
            let agl_end = agl_start + 3;
            let right_ok = agl_end == bytes.len()
                || !bytes
                    .get(agl_end)
                    .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');

            if right_ok
                && agl_start > 0
                && bytes
                    .get(agl_start - 1)
                    .is_some_and(u8::is_ascii_whitespace)
            {
                let mut prev = agl_start - 1;
                while prev > 0 && bytes.get(prev).is_some_and(u8::is_ascii_whitespace) {
                    prev -= 1;
                }
                let prev_end = prev + 1;
                let mut word_start = prev;
                while word_start > 0
                    && bytes
                        .get(word_start - 1)
                        .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    word_start -= 1;
                }
                let left_ok = word_start == 0
                    || !bytes
                        .get(word_start - 1)
                        .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
                let prev_word = lower.get(word_start..prev_end).unwrap_or("");

                if left_ok && (prev_word == "altitude" || prev_word == "m") {
                    if let Some(prefix) = text.get(last_idx..word_start) {
                        result.push_str(prefix);
                    }
                    result.push_str(replacement);
                    last_idx = agl_end;
                    i = agl_end;
                    continue;
                }
            }
            i = agl_end;
        } else {
            break;
        }
    }
    if let Some(tail) = text.get(last_idx..) {
        result.push_str(tail);
    }
    result
}

/// Replaces `"ASTM F3411"` or `"ASTM F3411-22a"` references with domain-appropriate standard `replacement`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-REPLACE-ASTM-F3411] | SysML v2: ActionDef(replace_astm_f3411)
pub(super) fn replace_astm_f3411(text: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower.get(i..).and_then(|s| s.find("astm")) {
            let start = i + pos;
            let mut j = start + 4;
            let left_ok = start == 0
                || !bytes
                    .get(start - 1)
                    .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
            if left_ok {
                while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                    j += 1;
                }
                if lower.get(j..).is_some_and(|s| s.starts_with("f3411")) {
                    j += 5;
                    if lower.get(j..).is_some_and(|s| s.starts_with("-22a")) {
                        j += 4;
                    }
                    if let Some(prefix) = text.get(last_idx..start) {
                        result.push_str(prefix);
                    }
                    result.push_str(replacement);
                    last_idx = j;
                    i = j;
                    continue;
                }
            }
            i = start + 4;
        } else {
            break;
        }
    }
    if let Some(tail) = text.get(last_idx..) {
        result.push_str(tail);
    }
    result
}

/// Replaces `"Rules of Engagement"` and `"Rules of Engagement (ROE)"` with `"operational safety interlocks"`.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-REPLACE-ROE-PHRASE] | SysML v2: ActionDef(replace_roe_phrase)
pub(super) fn replace_roe_phrase(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower.get(i..).and_then(|s| s.find("rules of engagement")) {
            let start = i + pos;
            let mut end = start + "rules of engagement".len();
            let mut j = end;
            while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') {
                j += 1;
            }
            if lower.get(j..).is_some_and(|s| s.starts_with("(roe)")) {
                end = j + 5;
            }
            if let Some(prefix) = text.get(last_idx..start) {
                result.push_str(prefix);
            }
            result.push_str("operational safety interlocks");
            last_idx = end;
            i = end;
        } else {
            break;
        }
    }
    if let Some(tail) = text.get(last_idx..) {
        result.push_str(tail);
    }
    result
}

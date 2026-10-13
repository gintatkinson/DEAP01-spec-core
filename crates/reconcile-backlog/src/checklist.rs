//! Markdown checklist parsing, cross-linking, and reconciliation.
//!
//! Realises: [Zero-Placeholder Invariant, Strict Bidirectional Checklist Sync]

/// Represents a parsed checklist item.
#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    pub line_number: usize,
    pub checked: bool,
    pub issue_number: Option<u64>,
    pub text: String,
    pub full_line: String,
}

/// Internal helper representing a parsed checklist line.
struct ParsedCheckline<'a> {
    checked: bool,
    issue_number: Option<u64>,
    text: &'a str,
    mark_pos: usize,
}

/// Parses a line into checklist components if it matches `- [ ]` or `- [x]`.
fn parse_checklist_line(line: &str) -> Option<ParsedCheckline<'_>> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();

    // Check for checkbox prefix with str::strip_prefix
    let (checked, after_box) = if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
        (false, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [ ]") {
        (false, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [x]").or_else(|| trimmed.strip_prefix("- [X]")) {
        (true, rest)
    } else {
        // Fallback for optional whitespace between '-' and '['
        let after_dash = trimmed.strip_prefix('-')?;
        let after_dash_trimmed = after_dash.trim_start();
        if let Some(rest) = after_dash_trimmed.strip_prefix("[ ] ") {
            (false, rest)
        } else if let Some(rest) = after_dash_trimmed.strip_prefix("[x] ").or_else(|| after_dash_trimmed.strip_prefix("[X] ")) {
            (true, rest)
        } else if let Some(rest) = after_dash_trimmed.strip_prefix("[ ]") {
            (false, rest)
        } else if let Some(rest) = after_dash_trimmed.strip_prefix("[x]").or_else(|| after_dash_trimmed.strip_prefix("[X]")) {
            (true, rest)
        } else {
            return None;
        }
    };

    // Calculate exact byte position of the checkbox mark character in `line`
    let bracket_offset = trimmed.find('[')?;
    let mark_pos = indent_len + bracket_offset + 1;

    // Parse optional issue number (#123) and title from `after_box`
    let after_ws = after_box.trim_start();
    let (issue_number, text) = if let Some(after_hash) = after_ws.strip_prefix('#') {
        let digits_len = after_hash.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits_len > 0 {
            let num = after_hash[..digits_len].parse::<u64>().ok();
            let after_digits = &after_hash[digits_len..];
            let after_digits_trimmed = after_digits.trim_start();
            let after_sep = if let Some(rest) = after_digits_trimmed.strip_prefix('-') {
                rest
            } else if let Some(rest) = after_digits_trimmed.strip_prefix(':') {
                rest
            } else {
                after_digits_trimmed
            };
            (num, after_sep.trim_start())
        } else {
            (None, after_ws)
        }
    } else {
        (None, after_ws)
    };

    Some(ParsedCheckline {
        checked,
        issue_number,
        text,
        mark_pos,
    })
}

/// Identifies if a string contains prohibited unpopulated fallback placeholders.
pub fn contains_unpopulated_placeholders(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("to be populated after phase 3")
        || lower.contains("to be populated after phase")
        || lower.contains("*to be populated*")
}

/// Replaces prohibited placeholder blocks with compliant allocated notes or removes them.
pub fn clean_unpopulated_placeholders(content: &str) -> String {
    let target = "to be populated after phase 3";
    let replacement = "- None allocated (structural subsystem component)";
    let mut result = String::with_capacity(content.len());
    let mut rest = content;

    while let Some(pos) = rest.to_ascii_lowercase().find(target) {
        // Look backwards from pos in rest for optional whitespace and optional '*'
        let before = &rest[..pos];
        let bytes_before = before.as_bytes();
        let mut idx = pos;
        while idx > 0 && bytes_before[idx - 1].is_ascii_whitespace() {
            idx -= 1;
        }
        if idx > 0 && bytes_before[idx - 1] == b'*' {
            idx -= 1;
        }
        let start_idx = idx;

        // Look forwards from pos + target.len() in rest for optional whitespace and optional '*'
        let after_pos = pos + target.len();
        let after = &rest[after_pos..];
        let bytes_after = after.as_bytes();
        let mut end_offset = 0;
        while end_offset < bytes_after.len() && bytes_after[end_offset].is_ascii_whitespace() {
            end_offset += 1;
        }
        if end_offset < bytes_after.len() && bytes_after[end_offset] == b'*' {
            end_offset += 1;
        }
        let end_idx = after_pos + end_offset;

        result.push_str(&rest[..start_idx]);
        result.push_str(replacement);
        rest = &rest[end_idx..];
    }
    result.push_str(rest);
    result
}

/// Parses all `- [ ]` or `- [x]` checklist items from markdown content.
pub fn parse_checklists(content: &str) -> Vec<ChecklistItem> {
    let mut items = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if let Some(parsed) = parse_checklist_line(line) {
            items.push(ChecklistItem {
                line_number: idx + 1,
                checked: parsed.checked,
                issue_number: parsed.issue_number,
                text: parsed.text.to_string(),
                full_line: line.to_string(),
            });
        }
    }

    items
}

/// Reconciles checklist items in markdown content using a mapping of issue numbers or normalized titles to resolved status.
pub fn reconcile_checklist_content<F>(
    content: &str,
    is_item_resolved: F,
) -> (String, usize)
where
    F: Fn(Option<u64>, &str) -> Option<bool>,
{
    let mut new_lines = Vec::new();
    let mut modified_count = 0;

    for line in content.lines() {
        if let Some(parsed) = parse_checklist_line(line) {
            if let Some(target_resolved) = is_item_resolved(parsed.issue_number, parsed.text.trim()) {
                if target_resolved != parsed.checked {
                    let mark = if target_resolved { "x" } else { " " };
                    let new_line = format!("{}{}{}", &line[..parsed.mark_pos], mark, &line[parsed.mark_pos + 1..]);
                    new_lines.push(new_line);
                    modified_count += 1;
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
    }

    (new_lines.join("\n"), modified_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checklists() {
        let md = r#"
## 2. Requirements & Checklist
- [ ] #10 - Feature 01: [ConOps] Human Engineer Interface
- [x] #11 - Feature 02: [ConOps] CI Runner
- [ ] Feature without issue
"#;
        let items = parse_checklists(md);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].checked, false);
        assert_eq!(items[0].issue_number, Some(10));
        assert_eq!(items[1].checked, true);
        assert_eq!(items[1].issue_number, Some(11));
        assert_eq!(items[2].checked, false);
        assert_eq!(items[2].issue_number, None);
    }

    #[test]
    fn test_contains_unpopulated_placeholders() {
        let placeholder = "*To be populated after Phase 3*";
        assert!(contains_unpopulated_placeholders(placeholder));
        let clean = "- None allocated (structural subsystem component)";
        assert!(!contains_unpopulated_placeholders(clean));
    }

    #[test]
    fn test_clean_unpopulated_placeholders() {
        let md = r#"
#### Associated Use Cases
*To be populated after Phase 3*
"#;
        let cleaned = clean_unpopulated_placeholders(md);
        assert!(!cleaned.contains("To be populated after Phase 3"));
        assert!(cleaned.contains("- None allocated (structural subsystem component)"));
    }

    #[test]
    fn test_reconcile_checklist_content() {
        let md = r#"
- [ ] #10 - Feature 01: Test
- [ ] #11 - Feature 02: Another
"#;
        let (reconciled, count) = reconcile_checklist_content(md, |issue_num, _| {
            if issue_num == Some(10) {
                Some(true)
            } else {
                None
            }
        });
        assert_eq!(count, 1);
        assert!(reconciled.contains("- [x] #10 - Feature 01: Test"));
        assert!(reconciled.contains("- [ ] #11 - Feature 02: Another"));
    }
}

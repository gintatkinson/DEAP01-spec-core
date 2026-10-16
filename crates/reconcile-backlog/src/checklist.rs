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
        } else {
            let rest = after_dash_trimmed
                .strip_prefix("[x]")
                .or_else(|| after_dash_trimmed.strip_prefix("[X]"))?;
            (true, rest)
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
    text.to_ascii_lowercase().contains("to be populated")
}

/// Replaces prohibited placeholder blocks with compliant allocated notes or removes them.
pub fn clean_unpopulated_placeholders(content: &str) -> String {
    clean_unpopulated_placeholders_counted(content).0
}

/// Replaces prohibited placeholder blocks with compliant allocated notes and returns the count of purged blocks.
pub fn clean_unpopulated_placeholders_counted(content: &str) -> (String, usize) {
    let target = "to be populated";
    let replacement = "- None allocated (structural subsystem component)";
    let mut result = String::with_capacity(content.len());
    let mut rest = content;
    let mut purged_count = 0;

    while let Some(pos) = rest.to_ascii_lowercase().find(target) {
        // Forward matching: check for optional trailing phase specifier e.g. " after phase [digits/alnum]"
        let mut cur_offset = target.len();
        let after_target = &rest[pos + cur_offset..];
        let after_lower = after_target.to_ascii_lowercase();

        if after_lower.starts_with(" after phase") {
            cur_offset += " after phase".len();
            // Skip optional spaces/tabs between "after phase" and phase identifier
            let rem = &rest[pos + cur_offset..];
            let rem_trimmed = rem.trim_start_matches(|c: char| c == ' ' || c == '\t');
            cur_offset += rem.len() - rem_trimmed.len();
            // Consume optional alphanumeric phase identifier (e.g., "1", "2", "3", "IV")
            let id_len = rem_trimmed.chars().take_while(|c| c.is_ascii_alphanumeric()).count();
            cur_offset += id_len;
        }

        // Forward scanning: consume trailing closing envelope on the line
        // Envelope characters: spaces, tabs, closing parenthesis ')', markdown emphasis '*', '_'
        let forward_slice = &rest[pos + cur_offset..];
        let forward_bytes = forward_slice.as_bytes();
        let mut i = 0;
        // Skip horizontal spaces/tabs
        while i < forward_bytes.len() && (forward_bytes[i] == b' ' || forward_bytes[i] == b'\t') {
            i += 1;
        }
        // Skip optional ')'
        if i < forward_bytes.len() && forward_bytes[i] == b')' {
            i += 1;
        }
        // Skip horizontal spaces/tabs
        while i < forward_bytes.len() && (forward_bytes[i] == b' ' || forward_bytes[i] == b'\t') {
            i += 1;
        }
        // Skip closing markdown emphasis: '*' or '_'
        while i < forward_bytes.len() && (forward_bytes[i] == b'*' || forward_bytes[i] == b'_') {
            i += 1;
        }
        // Skip optional ')' (in case formatted like (*...*))
        if i < forward_bytes.len() && forward_bytes[i] == b')' {
            i += 1;
        }
        // Skip trailing horizontal spaces/tabs up to newline
        while i < forward_bytes.len() && (forward_bytes[i] == b' ' || forward_bytes[i] == b'\t') {
            i += 1;
        }
        let end_idx = pos + cur_offset + i;

        // Backward scanning: check line preceding the placeholder in `rest`
        let before = &rest[..pos];
        let line_start = before.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
        let line_prefix = &rest[line_start..pos];

        // Measure leading indentation on this line
        let indent_len = line_prefix.len() - line_prefix.trim_start_matches(|c: char| c == ' ' || c == '\t').len();
        let after_indent = &line_prefix[indent_len..];

        // Check if after_indent contains only markdown markers/envelope:
        // bullets ('-', '*', '+'), emphasis ('*', '_'), parentheses ('('), whitespace (' ', '\t')
        let is_pure_envelope = !after_indent.chars().any(|c| {
            c != '-' && c != '*' && c != '+' && c != '_' && c != '(' && c != ' ' && c != '\t'
        });

        let start_idx = if is_pure_envelope {
            line_start + indent_len
        } else {
            // If there was preceding non-envelope text, peel only trailing envelope from `before`
            let mut j = pos;
            while j > line_start {
                let prev_char = rest[..j].chars().next_back().unwrap();
                if prev_char == ' ' || prev_char == '\t' || prev_char == '*' || prev_char == '_' || prev_char == '(' {
                    j -= prev_char.len_utf8();
                } else {
                    break;
                }
            }
            j
        };

        result.push_str(&rest[..start_idx]);
        result.push_str(replacement);
        rest = &rest[end_idx..];
        purged_count += 1;
    }

    result.push_str(rest);
    (result, purged_count)
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
    fn test_parse_checklists() -> Result<(), Box<dyn std::error::Error>> {
        let md = r#"
## 2. Requirements & Checklist
- [ ] #10 - Feature 01: [ConOps] Human Engineer Interface
- [x] #11 - Feature 02: [ConOps] CI Runner
- [ ] Feature without issue
"#;
        let items = parse_checklists(md);
        assert_eq!(items.len(), 3);
        let first = items.first().ok_or("missing checklist item 0")?;
        assert!(!first.checked);
        assert_eq!(first.issue_number, Some(10));
        let second = items.get(1).ok_or("missing checklist item 1")?;
        assert!(second.checked);
        assert_eq!(second.issue_number, Some(11));
        let third = items.get(2).ok_or("missing checklist item 2")?;
        assert!(!third.checked);
        assert_eq!(third.issue_number, None);
        Ok(())
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

    #[test]
    fn test_issue_439_placeholder_purging_regression() {
        let canonical_replacement = "- None allocated (structural subsystem component)";

        // 1. All placeholder phase variants and formatting variations
        let test_cases = vec![
            // Phase variants
            "*To be populated after Phase 3*",
            "*To be populated after Phase 2*",
            "*To be populated after Phase 1*",
            "*To be populated after Phase*",
            "*To be populated*",
            "*(To be populated)*",
            "*(To be populated after Phase 3)*",
            // Formatting variations
            "**To be populated after Phase 3**",
            "_To be populated after Phase 3_",
            "- *To be populated after Phase 3*",
            "TO BE POPULATED AFTER PHASE 2",
        ];

        for raw in test_cases {
            // Precondition: detector identifies raw placeholder as unpopulated
            assert!(
                contains_unpopulated_placeholders(raw),
                "Precondition failed: detector missed placeholder variant '{}'",
                raw
            );

            let (cleaned, count) = clean_unpopulated_placeholders_counted(raw);
            assert_eq!(
                count, 1,
                "Expected count == 1 for raw placeholder '{}', got count = {}",
                raw, count
            );
            assert!(
                cleaned.contains(canonical_replacement),
                "Cleaned text did not contain canonical replacement for '{}':\n{}",
                raw, cleaned
            );
            // Postcondition assertion
            assert!(
                !contains_unpopulated_placeholders(&cleaned),
                "Postcondition failed: cleaned text still contains unpopulated placeholder for '{}':\n{}",
                raw, cleaned
            );
            // Syntax purity assertions
            assert!(
                !cleaned.contains("*- None"),
                "Syntax corruption '*- None' found for '{}':\n{}",
                raw, cleaned
            );
            assert!(
                !cleaned.contains("_- None"),
                "Syntax corruption '_- None' found for '{}':\n{}",
                raw, cleaned
            );
            assert!(
                !cleaned.contains("- - None"),
                "Syntax corruption '- - None' found for '{}':\n{}",
                raw, cleaned
            );

            // Idempotency: running cleaner on already-cleaned text produces count == 0 and identical output
            let (second_cleaned, second_count) = clean_unpopulated_placeholders_counted(&cleaned);
            assert_eq!(second_count, 0, "Idempotency failed: second pass count = {} on '{}'", second_count, raw);
            assert_eq!(second_cleaned, cleaned, "Idempotency failed: output changed on second pass for '{}'", raw);
        }

        // 2. Multi-block counting: 3 blocks in one string -> count == 3
        let multi_block = r#"
#### Block 1
*To be populated after Phase 1*

#### Block 2
*To be populated after Phase 2*

#### Block 3
*To be populated after Phase 3*
"#;
        let (cleaned_multi, multi_count) = clean_unpopulated_placeholders_counted(multi_block);
        assert_eq!(multi_count, 3, "Multi-block test failed: expected count 3, got {}", multi_count);
        assert!(
            !contains_unpopulated_placeholders(&cleaned_multi),
            "Postcondition failed on multi-block:\n{}",
            cleaned_multi
        );
        let occurrences = cleaned_multi.matches(canonical_replacement).count();
        assert_eq!(occurrences, 3, "Expected 3 occurrences of canonical replacement, got {}", occurrences);
        assert!(!cleaned_multi.contains("*- None"));
        assert!(!cleaned_multi.contains("_- None"));
        assert!(!cleaned_multi.contains("- - None"));

        // Idempotency on multi-block
        let (idempotent_multi, idempotent_count) = clean_unpopulated_placeholders_counted(&cleaned_multi);
        assert_eq!(idempotent_count, 0);
        assert_eq!(idempotent_multi, cleaned_multi);
    }
}

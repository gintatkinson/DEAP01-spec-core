//! Markdown checklist parsing, cross-linking, and reconciliation.
//!
//! Realises: [Zero-Placeholder Invariant, Strict Bidirectional Checklist Sync]

use regex::Regex;

/// Represents a parsed checklist item.
#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    pub line_number: usize,
    pub checked: bool,
    pub issue_number: Option<u64>,
    pub text: String,
    pub full_line: String,
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
    let re = Regex::new(r"(?i)\*?\s*To be populated after Phase 3\s*\*?").unwrap();
    re.replace_all(content, "- None allocated (structural subsystem component)").to_string()
}

/// Parses all `- [ ]` or `- [x]` checklist items from markdown content.
pub fn parse_checklists(content: &str) -> Vec<ChecklistItem> {
    let mut items = Vec::new();
    let re = Regex::new(r"^(\s*-\s*\[([ xX])\])\s*(?:#(\d+)\s*[-:]?\s*)?(.*)$").unwrap();

    for (idx, line) in content.lines().enumerate() {
        if let Some(caps) = re.captures(line) {
            let checked = &caps[2] == "x" || &caps[2] == "X";
            let issue_number = caps.get(3).and_then(|m| m.as_str().parse::<u64>().ok());
            let text = caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();

            items.push(ChecklistItem {
                line_number: idx + 1,
                checked,
                issue_number,
                text,
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
    let re = Regex::new(r"^(\s*-\s*\[)([ xX])(\]\s*(?:#(\d+)\s*[-:]?\s*)?)(.*)$").unwrap();
    let mut new_lines = Vec::new();
    let mut modified_count = 0;

    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            let current_checked = &caps[2] == "x" || &caps[2] == "X";
            let issue_num = caps.get(4).and_then(|m| m.as_str().parse::<u64>().ok());
            let text = caps.get(5).map(|m| m.as_str().trim()).unwrap_or("");

            if let Some(target_resolved) = is_item_resolved(issue_num, text) {
                if target_resolved != current_checked {
                    let mark = if target_resolved { "x" } else { " " };
                    let prefix = &caps[1];
                    let suffix = &caps[3];
                    let rest = &caps[5];
                    new_lines.push(format!("{}{}{}{}", prefix, mark, suffix, rest));
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

//! Specification frontmatter extraction and updating.
//!
//! Realises: [Zero-Placeholder Invariant, Strict Schema-Driven Spec Frontmatter]

use std::collections::HashMap;

/// Parsed metadata from a specification file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpecFrontmatter {
    pub title: String,
    pub spec_type: String,
    pub issue_id: Option<u64>,
    pub epic: Option<String>,
    pub subsystem: Option<String>,
    pub part: Option<String>,
    pub version: Option<String>,
    pub date: Option<String>,
    pub raw_fields: HashMap<String, String>,
}

/// Extracts a normalized key from a table cell (e.g. `**Issue ID**` -> `"issue_id"`).
fn normalize_table_key(cell: &str) -> String {
    let mut key = String::new();
    for c in cell.chars() {
        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
            key.push(c);
        }
    }
    key.trim().to_lowercase().replace(' ', "_").replace('-', "_")
}

/// Parses a 2-column table row if valid.
fn parse_table_row(line: &str) -> Option<(String, String)> {
    let line_trimmed = line.trim();
    if !line_trimmed.starts_with('|') || !line_trimmed.ends_with('|') {
        return None;
    }
    let inner = line_trimmed.trim_matches('|');
    let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return None;
    }
    let key = normalize_table_key(parts[0]);
    if key.is_empty() || key == "attribute" || key == "key" {
        return None;
    }
    let val = parts[1]
        .trim()
        .trim_matches(|c| c == '`' || c == '"' || c == '\'');
    Some((key, val.to_string()))
}

/// Extracts metadata from native CommonMark 2-column table or YAML frontmatter.
pub fn parse_frontmatter(content: &str) -> Option<SpecFrontmatter> {
    let mut raw_fields = HashMap::new();
    let mut title = String::new();
    let mut spec_type = String::new();
    let mut issue_id = None;
    let mut epic = None;
    let mut subsystem = None;
    let mut part = None;
    let mut version = None;
    let mut date = None;

    let mut in_table = false;
    for line in content.lines().take(20) {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('|') && line_trimmed.ends_with('|') {
            in_table = true;
            if let Some((key, val)) = parse_table_row(line) {
                match key.as_str() {
                    "title" => {
                        if title.is_empty() {
                            title = val.clone();
                        }
                    }
                    "type" => {
                        if spec_type.is_empty() {
                            spec_type = val.clone();
                        }
                    }
                    "issue_id" => {
                        let cleaned = val
                            .trim_start_matches(|c| c == '#' || c == '[' || c == ']')
                            .trim();
                        if let Ok(id) = cleaned.parse::<u64>() {
                            issue_id = Some(id);
                        }
                    }
                    "epic" | "parent_epic" => {
                        if epic.is_none() {
                            epic = Some(val.clone());
                        }
                    }
                    "subsystem" => {
                        subsystem = Some(val.clone());
                    }
                    "package" => {
                        if subsystem.is_none() {
                            subsystem = Some(val.clone());
                        }
                    }
                    "part" => {
                        if part.is_none() {
                            part = Some(val.clone());
                        }
                    }
                    "schema_containers" => {
                        let p = val
                            .split("::")
                            .last()
                            .unwrap_or(&val)
                            .trim()
                            .trim_matches('`')
                            .to_string();
                        if part.is_none() {
                            part = Some(p.clone());
                        }
                        raw_fields.insert("part".to_string(), p);
                    }
                    "version" => {
                        if version.is_none() {
                            version = Some(val.clone());
                        }
                    }
                    "date" => {
                        if date.is_none() {
                            date = Some(val.clone());
                        }
                    }
                    "sysml_interaction" => {
                        raw_fields.insert("interaction".to_string(), val.clone());
                    }
                    "scope" => {
                        raw_fields.insert("subject".to_string(), val.clone());
                    }
                    "primary_actor" => {
                        raw_fields.insert("actors".to_string(), val.clone());
                    }
                    _ => {}
                }
                raw_fields.insert(key, val);
            }
        } else if in_table {
            break;
        }
    }

    // Fallback: If no CommonMark table metadata found, check legacy YAML frontmatter
    if raw_fields.is_empty() {
        let trimmed = content.trim_start();
        if trimmed.starts_with("---") {
            if let Some(rest) = trimmed.strip_prefix("---") {
                if let Some(end_idx) = rest.find("\n---") {
                    let fm_block = &rest[..end_idx];
                    for line in fm_block.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        if let Some((k, v)) = line.split_once(':') {
                            let key = k.trim().trim_matches(|c| c == '\'' || c == '"').to_lowercase();
                            let val = v.trim().trim_matches(|c| c == '\'' || c == '"').to_string();
                            if key.is_empty() {
                                continue;
                            }

                            match key.as_str() {
                                "title" => title = val.clone(),
                                "type" => spec_type = val.clone(),
                                "issue_id" => {
                                    let cleaned = val
                                        .trim_start_matches(|c| c == '#' || c == '[' || c == ']')
                                        .trim();
                                    if let Ok(id) = cleaned.parse::<u64>() {
                                        issue_id = Some(id);
                                    }
                                }
                                "epic" => epic = Some(val.clone()),
                                "subsystem" => subsystem = Some(val.clone()),
                                "part" => part = Some(val.clone()),
                                "version" => version = Some(val.clone()),
                                "date" => date = Some(val.clone()),
                                _ => {}
                            }
                            raw_fields.insert(key, val);
                        }
                    }
                }
            }
        }
    }

    if raw_fields.is_empty() {
        return None;
    }

    // Fallback: title from # H1 heading if not in frontmatter
    if title.is_empty() {
        for line in content.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("# ") {
                title = trimmed_line.trim_start_matches("# ").trim().to_string();
                break;
            }
        }
    }

    Some(SpecFrontmatter {
        title,
        spec_type,
        issue_id,
        epic,
        subsystem,
        part,
        version,
        date,
        raw_fields,
    })
}

/// Updates or injects `| **Issue ID** | #<id> |` in the CommonMark metadata table at lines 1-10.
///
/// If legacy YAML frontmatter fences are present, also updates `issue_id: <id>` within the fence,
/// but does NOT inject legacy YAML fences into documents that lack them.
pub fn update_frontmatter_issue_id(content: &str, issue_id: u64) -> String {
    let newline = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing_newline = content.ends_with('\n');

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Check if legacy YAML frontmatter is present at the start
    let has_yaml = content.trim_start().starts_with("---");
    let mut yaml_end_idx = None;

    if has_yaml {
        let mut in_yaml = false;
        let mut yaml_issue_id_idx = None;
        let mut yaml_closing_idx = None;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "---" {
                if !in_yaml {
                    in_yaml = true;
                } else {
                    yaml_closing_idx = Some(idx);
                    break;
                }
            } else if in_yaml {
                if let Some((k, _)) = trimmed.split_once(':') {
                    let key = k.trim().trim_matches(|c| c == '\'' || c == '"').to_lowercase();
                    if key == "issue_id" {
                        yaml_issue_id_idx = Some(idx);
                    }
                }
            }
        }

        if let Some(idx) = yaml_issue_id_idx {
            lines[idx] = format!("issue_id: {}", issue_id);
        } else if let Some(idx) = yaml_closing_idx {
            lines.insert(idx, format!("issue_id: {}", issue_id));
        }

        yaml_end_idx = yaml_closing_idx;
    }

    // Now update or inject into the CommonMark 2-column metadata table
    let search_start = yaml_end_idx.map(|idx| idx + 1).unwrap_or(0);
    let mut table_header_idx = None;
    let mut table_delim_idx = None;
    let mut table_issue_id_idx = None;
    let mut in_table = false;

    let scan_limit = (search_start + 25).min(lines.len());
    for idx in search_start..scan_limit {
        let line_trimmed = lines[idx].trim();
        if line_trimmed.starts_with('|') && line_trimmed.ends_with('|') {
            in_table = true;
            let inner = line_trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                let key = normalize_table_key(parts[0]);
                if key == "attribute" {
                    table_header_idx = Some(idx);
                } else if parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c == '-' || c == ':')) {
                    table_delim_idx = Some(idx);
                } else if key == "issue_id" {
                    table_issue_id_idx = Some(idx);
                    break;
                }
            }
        } else if in_table {
            break;
        }
    }

    let target_issue_row = format!("| **Issue ID** | #{} |", issue_id);

    if let Some(idx) = table_issue_id_idx {
        lines[idx] = target_issue_row;
    } else if let Some(delim_idx) = table_delim_idx {
        lines.insert(delim_idx + 1, target_issue_row);
    } else if let Some(header_idx) = table_header_idx {
        lines.insert(header_idx + 1, "| :--- | :--- |".to_string());
        lines.insert(header_idx + 2, target_issue_row);
    } else if !has_yaml {
        // No metadata table and no YAML: inject CommonMark metadata table at the start
        let mut new_lines = vec![
            "| Attribute | Specification Detail |".to_string(),
            "| :--- | :--- |".to_string(),
            target_issue_row,
            "".to_string(),
        ];
        new_lines.extend(lines);
        lines = new_lines;
    }

    let mut result = lines.join(newline);
    if trailing_newline && !result.ends_with(newline) {
        result.push_str(newline);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_commonmark_epic() {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #101 |
| **Title** | Epic 01: System Architecture and Compiler Primacy |
| **Type** | epic |
| **Package** | DEAP_Compiler_System |
| **Subsystem** | System Architecture |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Epic 01: System Architecture and Compiler Primacy
"#;
        let fm = parse_frontmatter(content).expect("Parsed CommonMark table");
        assert_eq!(fm.title, "Epic 01: System Architecture and Compiler Primacy");
        assert_eq!(fm.spec_type, "epic");
        assert_eq!(fm.issue_id, Some(101));
        assert_eq!(fm.subsystem.as_deref(), Some("System Architecture"));
    }

    #[test]
    fn test_parse_frontmatter_commonmark_feature() {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #10 |
| **Title** | Feature 01: [Con Ops] HumanEngineer |
| **Type** | feature |
| **Parent Epic** | [EPIC-02-ConOps](../epics/EPIC-02-ConOps.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `ConOps::HumanEngineer` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/conops/actors.sysml](../../schema/conops/actors.sysml) |

# Feature 01: [Con Ops] HumanEngineer
"#;
        let fm = parse_frontmatter(content).expect("Parsed CommonMark table");
        assert_eq!(fm.title, "Feature 01: [Con Ops] HumanEngineer");
        assert_eq!(fm.spec_type, "feature");
        assert_eq!(fm.issue_id, Some(10));
        assert_eq!(fm.part.as_deref(), Some("HumanEngineer"));
    }

    #[test]
    fn test_update_frontmatter_issue_id_commonmark() {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #101 |
| **Title** | Epic 01: System Architecture |
"#;
        let updated = update_frontmatter_issue_id(content, 42);
        assert!(updated.contains("| **Issue ID** | #42 |"));
        assert!(!updated.contains("#101"));
        assert!(!updated.lines().any(|l| l.trim() == "---"));
    }

    #[test]
    fn test_update_frontmatter_issue_id_commonmark_insert() {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Epic 01: System Architecture |
"#;
        let updated = update_frontmatter_issue_id(content, 42);
        assert!(updated.contains("| **Issue ID** | #42 |"));
        assert!(!updated.lines().any(|l| l.trim() == "---"));
    }

    #[test]
    fn test_parse_frontmatter_epic() {
        let content = r#"---
title: "Epic 01: System Architecture"
version: "1.0.0"
date: "2026-10-10"
type: epic
package: "DEAP_Compiler_System"
subsystem: "System Architecture"
issue_id: 101
generation_mode: subagent
---

# Epic 01: System Architecture
"#;
        let fm = parse_frontmatter(content).expect("Parsed frontmatter");
        assert_eq!(fm.title, "Epic 01: System Architecture");
        assert_eq!(fm.spec_type, "epic");
        assert_eq!(fm.issue_id, Some(101));
        assert_eq!(fm.subsystem.as_deref(), Some("System Architecture"));
    }

    #[test]
    fn test_update_frontmatter_issue_id_existing() {
        let content = r#"---
title: "Feature 01: Test"
type: feature
issue_id: 10
---

## Metadata
| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Feature 01: Test |
| **Issue ID** | #10 |
"#;
        let updated = update_frontmatter_issue_id(content, 42);
        assert!(updated.contains("issue_id: 42"));
        assert!(updated.contains("| **Issue ID** | #42 |"));
        assert!(!updated.contains("issue_id: 10"));
    }

    #[test]
    fn test_update_frontmatter_issue_id_insert() {
        let content = r#"---
title: "Feature 01: Test"
type: feature
---

## Metadata
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #[IssueID] |
"#;
        let updated = update_frontmatter_issue_id(content, 55);
        assert!(updated.contains("issue_id: 55"));
        assert!(updated.contains("| **Issue ID** | #55 |"));
    }
}

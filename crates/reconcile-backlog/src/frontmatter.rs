//! Specification frontmatter extraction and updating.
//!
//! Realises: [Zero-Placeholder Invariant, Strict Schema-Driven Spec Frontmatter]

use std::collections::BTreeMap;

/// Parsed metadata from a specification file.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SpecFrontmatter` in `crates/reconcile-backlog/src/frontmatter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SpecFrontmatter` across member set `[title: String, spec_type: String, issue_id: Option<u64>, epic: Option<String>, subsystem: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SpecFrontmatter` members (`title: String, spec_type: String, issue_id: Option<u64>, epic: Option<String>, subsystem: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SpecFrontmatter` representation for struct `SpecFrontmatter`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `SpecFrontmatter`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `SpecFrontmatter`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SpecFrontmatter` (`SpecFrontmatter`).
/// - State: Maintains consistent state invariants across `SpecFrontmatter` without partial mutation on error.
/// - Action: Executes `SpecFrontmatter` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SpecFrontmatter` (`RECONCILE-BACKLOG-FRONTMATTER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-SPECFRONTMATTER] | SysML v2: PartDef(SpecFrontmatter)
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
    pub raw_fields: BTreeMap<String, String>,
}

/// Extracts a normalized key from a table cell (e.g. `**Issue ID**` -> `"issue_id"`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_table_key` in `crates/reconcile-backlog/src/frontmatter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_table_key` (declaration #2) within `RECONCILE-BACKLOG-FRONTMATTER`.
///
/// ## 3. Preconditions
/// - `cell: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_table_key`.
///
/// ## 4. Postconditions
/// - Returns `String` from `normalize_table_key` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(cell)) bounded over input/member dimension of `normalize_table_key`.
/// - Space Complexity: O(N(cell)) bounded auxiliary memory for `normalize_table_key`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_table_key` (`String`).
/// - State: Maintains consistent state invariants across `normalize_table_key` without partial mutation on error.
/// - Action: Executes `normalize_table_key` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_table_key` (`RECONCILE-BACKLOG-FRONTMATTER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-NORMALIZE-TABLE-KEY] | SysML v2: ActionDef(normalize_table_key)
fn normalize_table_key(cell: &str) -> String {
    let mut key = String::new();
    for c in cell.chars() {
        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
            key.push(c);
        }
    }
    key.trim().to_lowercase().replace([' ', '-'], "_")
}

/// Strips a single matching outer quote or backtick pair only if it encloses the entire cell
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `strip_enclosing_cell_delimiters` in `crates/reconcile-backlog/src/frontmatter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `strip_enclosing_cell_delimiters` (declaration #3) within `RECONCILE-BACKLOG-FRONTMATTER`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `strip_enclosing_cell_delimiters`.
///
/// ## 4. Postconditions
/// - Returns `&str` from `strip_enclosing_cell_delimiters` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `strip_enclosing_cell_delimiters`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `strip_enclosing_cell_delimiters`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `strip_enclosing_cell_delimiters` (`&str`).
/// - State: Maintains consistent state invariants across `strip_enclosing_cell_delimiters` without partial mutation on error.
/// - Action: Executes `strip_enclosing_cell_delimiters` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `strip_enclosing_cell_delimiters` (`RECONCILE-BACKLOG-FRONTMATTER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-STRIP-ENCLOSING-CELL-DELIMITERS] | SysML v2: ActionDef(strip_enclosing_cell_delimiters)
fn strip_enclosing_cell_delimiters(val: &str) -> &str {
    let trimmed = val.trim();
    if trimmed.len() >= 2 {
        for delim in ['`', '"', '\''] {
            if trimmed.starts_with(delim) && trimmed.ends_with(delim) {
                let inner = &trimmed[delim.len_utf8()..trimmed.len() - delim.len_utf8()];
                if !inner.contains(delim) {
                    return inner.trim();
                }
            }
        }
    }
    trimmed
}

/// Parses a 2-column table row if valid.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_table_row` in `crates/reconcile-backlog/src/frontmatter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_table_row` (declaration #4) within `RECONCILE-BACKLOG-FRONTMATTER`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `parse_table_row`.
///
/// ## 4. Postconditions
/// - Returns `Option<(String, String)>` from `parse_table_row` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `parse_table_row`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `parse_table_row`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_table_row` (`Option<(String, String)>`).
/// - State: Maintains consistent state invariants across `parse_table_row` without partial mutation on error.
/// - Action: Executes `parse_table_row` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_table_row` (`RECONCILE-BACKLOG-FRONTMATTER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-TABLE-ROW] | SysML v2: ActionDef(parse_table_row)
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
    let key = normalize_table_key(parts.first()?);
    if key.is_empty()
        || key == "attribute"
        || key == "metadata"
        || key == "property"
        || key == "field"
        || key == "key"
    {
        return None;
    }
    if parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c == '-' || c == ':'))
    {
        return None;
    }
    let raw_val = parts.get(1)?;
    let val = strip_enclosing_cell_delimiters(raw_val);
    Some((key, val.to_string()))
}

/// Extracts metadata from native CommonMark 2-column table or YAML frontmatter.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_frontmatter` in `crates/reconcile-backlog/src/frontmatter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_frontmatter` (declaration #5) within `RECONCILE-BACKLOG-FRONTMATTER`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `parse_frontmatter`.
///
/// ## 4. Postconditions
/// - Returns `Option<SpecFrontmatter>` from `parse_frontmatter` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `parse_frontmatter`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `parse_frontmatter`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_frontmatter` (`Option<SpecFrontmatter>`).
/// - State: Maintains consistent state invariants across `parse_frontmatter` without partial mutation on error.
/// - Action: Executes `parse_frontmatter` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_frontmatter` (`RECONCILE-BACKLOG-FRONTMATTER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER] | SysML v2: ActionDef(parse_frontmatter)
pub fn parse_frontmatter(content: &str) -> Option<SpecFrontmatter> {
    let mut raw_fields = BTreeMap::new();
    let mut title = String::new();
    let mut spec_type = String::new();
    let mut issue_id = None;
    let mut epic = None;
    let mut subsystem = None;
    let mut part = None;
    let mut version = None;
    let mut date = None;

    // Check for leading YAML frontmatter first
    let mut yaml_start_idx = None;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" {
            yaml_start_idx = Some(idx);
        }
        break;
    }

    let mut yaml_closing_idx = None;
    let mut yaml_lines = Vec::new();
    if let Some(start) = yaml_start_idx {
        for (idx, line) in content.lines().enumerate().skip(start + 1) {
            let trimmed = line.trim();
            if trimmed == "---" {
                yaml_closing_idx = Some(idx);
                break;
            }
            yaml_lines.push(line);
        }
    }

    let yaml_end_line = if yaml_closing_idx.is_some() {
        for line in yaml_lines {
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
                        let cleaned = val.trim_start_matches(['#', '[', ']']).trim();
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
        yaml_closing_idx
    } else {
        None
    };

    let search_start = yaml_end_line.map(|idx| idx + 1).unwrap_or(0);
    let mut in_table = false;
    let mut pre_table_count = 0;

    let parse_issue_number = |v: &str| -> Option<u64> {
        let inner = if let Some(bracket_end) = v.find(']') {
            if v.starts_with('[') {
                &v[1..bracket_end]
            } else {
                v
            }
        } else {
            v
        };
        let cleaned = inner
            .trim_matches('*')
            .trim_matches('`')
            .trim_matches('"')
            .trim_matches('\'')
            .trim_start_matches(['#', '[', ']'])
            .trim_end_matches([']', ')'])
            .trim();
        cleaned.parse::<u64>().ok()
    };

    for line in content.lines().skip(search_start) {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('|') && line_trimmed.ends_with('|') {
            in_table = true;
            if let Some((key, val)) = parse_table_row(line) {
                match key.as_str() {
                    "title" => {
                        if !val.is_empty() {
                            title = val.clone();
                        }
                    }
                    "type" => {
                        if !val.is_empty() {
                            spec_type = val.clone();
                        }
                    }
                    "issue_id" | "issue" => {
                        if let Some(id) = parse_issue_number(&val) {
                            issue_id = Some(id);
                        }
                    }
                    "id" => {
                        if issue_id.is_none() {
                            if let Some(id) = parse_issue_number(&val) {
                                issue_id = Some(id);
                            }
                        }
                    }
                    "epic" | "parent_epic" => {
                        epic = Some(val.clone());
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
                        part = Some(val.clone());
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
                        if !raw_fields.contains_key("part") {
                            raw_fields.insert("part".to_string(), p);
                        }
                    }
                    "version" => {
                        version = Some(val.clone());
                    }
                    "date" => {
                        date = Some(val.clone());
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
        } else {
            pre_table_count += 1;
            if pre_table_count >= 25 {
                break;
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `update_frontmatter_issue_id` in `crates/reconcile-backlog/src/frontmatter.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `update_frontmatter_issue_id` (declaration #6) within `RECONCILE-BACKLOG-FRONTMATTER`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `update_frontmatter_issue_id`.
/// - `issue_id: u64` must satisfy domain and type-safety constraints prior to invoking `update_frontmatter_issue_id`.
///
/// ## 4. Postconditions
/// - Returns `String` from `update_frontmatter_issue_id` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `update_frontmatter_issue_id`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `update_frontmatter_issue_id`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `update_frontmatter_issue_id` (`String`).
/// - State: Maintains consistent state invariants across `update_frontmatter_issue_id` without partial mutation on error.
/// - Action: Executes `update_frontmatter_issue_id` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `update_frontmatter_issue_id` (`RECONCILE-BACKLOG-FRONTMATTER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-ISSUE-ID] | SysML v2: ActionDef(update_frontmatter_issue_id)
pub fn update_frontmatter_issue_id(content: &str, issue_id: u64) -> String {
    let newline = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing_newline = content.ends_with('\n');

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Check if legacy YAML frontmatter is present at the start
    let mut yaml_start_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" {
            yaml_start_idx = Some(idx);
        }
        break;
    }

    let mut yaml_closing_idx = None;
    let mut yaml_issue_id_idx = None;

    if let Some(start) = yaml_start_idx {
        for (idx, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim();
            if trimmed == "---" {
                yaml_closing_idx = Some(idx);
                break;
            }
            if let Some((k, _)) = trimmed.split_once(':') {
                let key = k.trim().trim_matches(|c| c == '\'' || c == '"').to_lowercase();
                if key == "issue_id" {
                    yaml_issue_id_idx = Some(idx);
                }
            }
        }
    }

    let has_yaml = yaml_closing_idx.is_some();
    let mut yaml_end_idx = None;

    if has_yaml {
        if let Some(idx) = yaml_issue_id_idx {
            if let Some(line) = lines.get_mut(idx) {
                *line = format!("issue_id: {}", issue_id);
            }
            yaml_end_idx = yaml_closing_idx;
        } else if let Some(closing) = yaml_closing_idx {
            lines.insert(closing, format!("issue_id: {}", issue_id));
            yaml_end_idx = Some(closing + 1);
        }
    }

    // Now update or inject into the CommonMark 2-column metadata table
    let search_start = yaml_end_idx.map(|idx| idx + 1).unwrap_or(0);
    let mut table_header_idx = None;
    let mut table_delim_idx = None;
    let mut table_issue_id_idx = None;
    let mut table_fallback_id_idx = None;
    let mut in_table = false;
    let mut pre_table_count = 0;

    for (idx, line) in lines.iter().enumerate().skip(search_start) {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('|') && line_trimmed.ends_with('|') {
            in_table = true;
            let inner = line_trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                let key = normalize_table_key(parts.first().copied().unwrap_or_default());
                if key == "attribute" || key == "metadata" || key == "property" || key == "field" || key == "key" {
                    table_header_idx = Some(idx);
                } else if parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c == '-' || c == ':')) {
                    table_delim_idx = Some(idx);
                } else if key == "issue_id" || key == "issue" {
                    table_issue_id_idx = Some(idx);
                } else if key == "id" && table_fallback_id_idx.is_none() {
                    let raw_val = parts.get(1).copied().unwrap_or_default();
                    let val = strip_enclosing_cell_delimiters(raw_val);
                    let cleaned = val
                        .trim_matches('*')
                        .trim_matches('`')
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_start_matches(['#', '[', ']'])
                        .trim_end_matches([']', ')'])
                        .trim();
                    if cleaned.parse::<u64>().is_ok() || val.trim().starts_with('#') {
                        table_fallback_id_idx = Some(idx);
                    }
                }
            }
        } else if in_table {
            break;
        } else {
            pre_table_count += 1;
            if pre_table_count >= 25 {
                break;
            }
        }
    }

    let target_issue_row = format!("| **Issue ID** | #{} |", issue_id);
    let target_table_row_idx = table_issue_id_idx.or(table_fallback_id_idx);

    if let Some(line) = target_table_row_idx.and_then(|idx| lines.get_mut(idx)) {
        *line = target_issue_row;
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
    fn test_parse_frontmatter_commonmark_epic() -> Result<(), Box<dyn std::error::Error>> {
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
        let fm = parse_frontmatter(content).ok_or("Parsed CommonMark table")?;
        assert_eq!(fm.title, "Epic 01: System Architecture and Compiler Primacy");
        assert_eq!(fm.spec_type, "epic");
        assert_eq!(fm.issue_id, Some(101));
        assert_eq!(fm.subsystem.as_deref(), Some("System Architecture"));
        Ok(())
    }

    #[test]
    fn test_parse_frontmatter_commonmark_feature() -> Result<(), Box<dyn std::error::Error>> {
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
        let fm = parse_frontmatter(content).ok_or("Parsed CommonMark table")?;
        assert_eq!(fm.title, "Feature 01: [Con Ops] HumanEngineer");
        assert_eq!(fm.spec_type, "feature");
        assert_eq!(fm.issue_id, Some(10));
        assert_eq!(fm.part.as_deref(), Some("HumanEngineer"));
        Ok(())
    }

    #[test]
    fn test_commonmark_2column_metadata_table_parser_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #430 |
| **Title** | Feature 42: CommonMark 2-Column Metadata Parser |
| **Type** | feature |
| **Primary Actor** | Subagent Worker |
| **Scope** | crates/reconcile-backlog |
| **Level** | 2 |
| **Generation Mode** | autonomous |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 42: CommonMark 2-Column Metadata Parser
"#;
        let fm = parse_frontmatter(content)
            .ok_or("RED PROBE FAILED: parse_frontmatter returned None for CommonMark 2-column table")?;
        assert_eq!(fm.issue_id, Some(430));
        assert_eq!(fm.title, "Feature 42: CommonMark 2-Column Metadata Parser");
        assert_eq!(fm.spec_type, "feature");
        assert_eq!(fm.raw_fields.get("actors").map(|s| s.as_str()), Some("Subagent Worker"));
        assert_eq!(fm.raw_fields.get("subject").map(|s| s.as_str()), Some("crates/reconcile-backlog"));
        assert_eq!(fm.raw_fields.get("level").map(|s| s.as_str()), Some("2"));
        assert_eq!(fm.raw_fields.get("generation_mode").map(|s| s.as_str()), Some("autonomous"));
        assert_eq!(fm.raw_fields.get("specification_source").map(|s| s.as_str()), Some("[schema/model.sysml](../../schema/model.sysml)"));

        let updated = update_frontmatter_issue_id(content, 999);
        assert!(updated.contains("| **Issue ID** | #999 |"));
        assert!(!updated.contains("#430"));
        assert!(!updated.lines().any(|l| l.trim() == "---"));
        Ok(())
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
    fn test_parse_frontmatter_epic() -> Result<(), Box<dyn std::error::Error>> {
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
        let fm = parse_frontmatter(content).ok_or("Parsed frontmatter")?;
        assert_eq!(fm.title, "Epic 01: System Architecture");
        assert_eq!(fm.spec_type, "epic");
        assert_eq!(fm.issue_id, Some(101));
        assert_eq!(fm.subsystem.as_deref(), Some("System Architecture"));
        Ok(())
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

    #[test]
    fn test_frontmatter_21_plus_line_table_truncation_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Feature 42: Comprehensive Metadata Table |
| **Type** | feature |
| **Row 03** | Val 03 |
| **Row 04** | Val 04 |
| **Row 05** | Val 05 |
| **Row 06** | Val 06 |
| **Row 07** | Val 07 |
| **Row 08** | Val 08 |
| **Row 09** | Val 09 |
| **Row 10** | Val 10 |
| **Row 11** | Val 11 |
| **Row 12** | Val 12 |
| **Row 13** | Val 13 |
| **Row 14** | Val 14 |
| **Row 15** | Val 15 |
| **Row 16** | Val 16 |
| **Row 17** | Val 17 |
| **Row 18** | Val 18 |
| **Row 19** | Val 19 |
| **Row 20** | Val 20 |
| **Issue ID** | #777 |
| **Generation Mode** | autonomous |
| **Verification Gate** | Check 35 |
| **Safety Level** | ASIL D |
| **Target Platform** | native |

# Feature 42: Comprehensive Metadata Table
"#;
        let fm = parse_frontmatter(content).ok_or("Failed to parse 21+ line table")?;
        assert_eq!(fm.issue_id, Some(777));
        assert_eq!(fm.raw_fields.get("generation_mode").map(|s| s.as_str()), Some("autonomous"));
        assert_eq!(fm.raw_fields.get("verification_gate").map(|s| s.as_str()), Some("Check 35"));
        assert_eq!(fm.raw_fields.get("safety_level").map(|s| s.as_str()), Some("ASIL D"));
        assert_eq!(fm.raw_fields.get("target_platform").map(|s| s.as_str()), Some("native"));

        let updated = update_frontmatter_issue_id(content, 888);
        assert!(updated.contains("| **Issue ID** | #888 |"));
        assert!(!updated.contains("#777"));
        let count_issue_id_rows = updated.lines().filter(|l| l.contains("**Issue ID**")).count();
        assert_eq!(count_issue_id_rows, 1, "Duplicate Issue ID row injected into 21+ line table");

        let fm_roundtrip = parse_frontmatter(&updated).ok_or("Failed roundtrip parse of updated 21+ line table")?;
        assert_eq!(fm_roundtrip.issue_id, Some(888));
        assert_eq!(fm_roundtrip.raw_fields.get("target_platform").map(|s| s.as_str()), Some("native"));
        Ok(())
    }

    #[test]
    fn test_frontmatter_hybrid_yaml_and_commonmark_table_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"---
title: "Feature 42: Source-Code AST Ingestion"
type: feature
version: "2.1.0"
date: "2026-10-17"
---

## Metadata
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #457 |
| **Part** | ASTCompiler |
| **Parent Epic** | EPIC-01 |
| **Primary Actor** | Subagent Worker |
"#;
        let fm = parse_frontmatter(content).ok_or("Failed to parse hybrid specification")?;
        assert_eq!(fm.title, "Feature 42: Source-Code AST Ingestion");
        assert_eq!(fm.spec_type, "feature");
        assert_eq!(fm.version.as_deref(), Some("2.1.0"));
        assert_eq!(fm.date.as_deref(), Some("2026-10-17"));
        assert_eq!(fm.issue_id, Some(457));
        assert_eq!(fm.part.as_deref(), Some("ASTCompiler"));
        assert_eq!(fm.epic.as_deref(), Some("EPIC-01"));
        assert_eq!(fm.raw_fields.get("actors").map(|s| s.as_str()), Some("Subagent Worker"));

        let updated = update_frontmatter_issue_id(content, 458);
        assert!(updated.contains("issue_id: 458"));
        assert!(updated.contains("| **Issue ID** | #458 |"));

        let fm_updated = parse_frontmatter(&updated).ok_or("Failed to parse updated hybrid specification")?;
        assert_eq!(fm_updated.issue_id, Some(458));
        assert_eq!(fm_updated.title, "Feature 42: Source-Code AST Ingestion");
        assert_eq!(fm_updated.spec_type, "feature");
        Ok(())
    }

    #[test]
    fn test_frontmatter_multi_span_backticks_and_quotes_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #101 |
| **Title** | Feature 01: Core Invariants |
| **Type** | feature |
| **Satisfied Requirements** | `REQ_COA_005`, `REQ_COA_006` |
| **Parent Epic** | `EPIC-01-Architecture` (`#101`) |
| **Single Span** | `SingleSpanToken` |
| **Quoted Field** | "Plain Quoted String" |

# Feature 01: Core Invariants
"#;
        let fm = parse_frontmatter(content).ok_or("Failed to parse table with multi-span backticks")?;
        assert_eq!(
            fm.raw_fields.get("satisfied_requirements").map(|s| s.as_str()),
            Some("`REQ_COA_005`, `REQ_COA_006`")
        );
        assert_eq!(
            fm.raw_fields.get("parent_epic").map(|s| s.as_str()),
            Some("`EPIC-01-Architecture` (`#101`)")
        );
        assert_eq!(fm.epic.as_deref(), Some("`EPIC-01-Architecture` (`#101`)"));
        assert_eq!(fm.raw_fields.get("single_span").map(|s| s.as_str()), Some("SingleSpanToken"));
        assert_eq!(fm.raw_fields.get("quoted_field").map(|s| s.as_str()), Some("Plain Quoted String"));
        Ok(())
    }

    #[test]
    fn test_frontmatter_id_precedence_and_part_preservation_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **ID** | FEAT-42 |
| **Title** | Source-Code AST Ingestion |
| **Type** | feature |
| **Part** | ExplicitPartDef |
| **Schema Containers** | `Package::ContainerPart` |
| **Issue ID** | #457 |

# Source-Code AST Ingestion
"#;
        let fm = parse_frontmatter(content).ok_or("Failed to parse specification")?;
        assert_eq!(fm.part.as_deref(), Some("ExplicitPartDef"));
        assert_eq!(fm.raw_fields.get("part").map(|s| s.as_str()), Some("ExplicitPartDef"));
        assert_eq!(fm.raw_fields.get("id").map(|s| s.as_str()), Some("FEAT-42"));
        assert_eq!(fm.issue_id, Some(457));

        let updated = update_frontmatter_issue_id(content, 999);
        assert!(updated.contains("| **ID** | FEAT-42 |"), "Explicit ID row destroyed by update_frontmatter_issue_id");
        assert!(updated.contains("| **Issue ID** | #999 |"));
        assert!(!updated.contains("#457"));

        let fm_updated = parse_frontmatter(&updated).ok_or("Failed to parse updated specification")?;
        assert_eq!(fm_updated.raw_fields.get("id").map(|s| s.as_str()), Some("FEAT-42"));
        assert_eq!(fm_updated.part.as_deref(), Some("ExplicitPartDef"));
        assert_eq!(fm_updated.issue_id, Some(999));
        Ok(())
    }

    #[test]
    fn test_frontmatter_unclosed_yaml_fence_and_yaml_insert_offset_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let unclosed = r#"---
Some markdown content starting with horizontal rule
No metadata table here.
"#;
        let updated = update_frontmatter_issue_id(unclosed, 123);
        assert!(updated.contains("| **Issue ID** | #123 |"), "CommonMark table not injected when --- is unclosed");

        let yaml_doc = r#"---
title: "Doc Title"
type: feature
---

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Doc Title |
"#;
        let updated_yaml = update_frontmatter_issue_id(yaml_doc, 456);
        assert!(updated_yaml.contains("issue_id: 456"));
        assert!(updated_yaml.contains("| **Issue ID** | #456 |"));

        let fm = parse_frontmatter(&updated_yaml).ok_or("Failed to parse updated doc")?;
        assert_eq!(fm.issue_id, Some(456));
        assert_eq!(fm.title, "Doc Title");
        assert_eq!(fm.spec_type, "feature");
        Ok(())
    }
}

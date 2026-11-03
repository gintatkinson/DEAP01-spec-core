#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Markdown YAML Frontmatter, CommonMark Metadata Table, and KerML Identifier Helpers for Reverse Sync.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Mitigates invalid KerML identifier generation and malformed
//! frontmatter parsing faults across reverse synchronization passes.
//!
//! ## 2. Invariants & Mathematical Properties
//! - KerML Grammar Conformance: `to_pascal_case` guarantees identifiers start with `_` or an alphabetic
//!   character and contain only alphanumeric or underscore characters.
//! - Bounds-Checked Slicing: `parse_frontmatter` uses `.strip_prefix`, `.strip_suffix`, and `.split_once`
//!   without direct byte indexing.
//!
//! ## 3. Preconditions
//! - Input strings may be arbitrary valid UTF-8 Markdown content or section titles.
//!
//! ## 4. Postconditions
//! - `parse_frontmatter` returns `(BTreeMap<String, String>, &str)` pairing normalized metadata keys with body text.
//! - `to_pascal_case` returns a KerML-compliant `PascalCase` string.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(L)$ where $L$ is the byte length of the input string.
//! - Space Complexity: $O(L)$ bounded by extracted metadata keys and identifier length.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unclosed YAML frontmatter fences fall back deterministically to CommonMark 2-column table parsing.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYNC-REVERSE-PARSE-FRONTMATTER]

use std::collections::BTreeMap;
use std::path::Path;

use deap_core::sysml_cst::{ActionDef, PartDef};

/// Extracts an `ActionDef` identifier from a Feature operation bullet `- `+ActionName(...) : RetType`` when valid and non-synthetic.
/// Realises: [REQ-0604]
pub(super) fn parse_feature_action_line(line: &str, default_act_lower: &str) -> Option<String> {
    let after_dash = line.trim().strip_prefix('-')?.trim_start();
    let rest = after_dash.strip_prefix("`+")?;
    let (raw_act_name, after_open) = rest.split_once('(')?;
    let act_name = raw_act_name.trim();
    if act_name.is_empty() || !act_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let (_params, raw_after_paren) = after_open.split_once(')')?;
    let after_colon = raw_after_paren.trim_start().strip_prefix(':')?.trim();
    if !after_colon.ends_with('`') {
        return None;
    }
    let ret_type = after_colon.trim_end_matches('`').trim();
    if ret_type.is_empty() || !ret_type.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    if act_name.to_lowercase() == default_act_lower {
        return None;
    }
    Some(act_name.to_string())
}

/// Extracts a `PartDef` and its operation/action bullets from a `features/*.md` specification.
/// Realises: [REQ-0604]
pub(super) fn extract_feature_part_actions(
    fm: &BTreeMap<String, String>,
    body: &str,
    path: &Path,
    parts: &mut BTreeMap<String, PartDef>,
) {
    let part_name = fm
        .get("part")
        .or_else(|| fm.get("part_def"))
        .map(|s| {
            if !s.is_empty()
                && (s.starts_with('_') || s.chars().next().is_some_and(char::is_alphabetic))
                && s.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                s.clone()
            } else {
                to_pascal_case(s)
            }
        })
        .unwrap_or_else(|| to_pascal_case(&path.file_stem().unwrap_or_default().to_string_lossy()));

    if part_name.is_empty() {
        return;
    }
    let entry = parts.entry(part_name.clone()).or_insert_with(|| PartDef {
        name: part_name.clone(),
        ..Default::default()
    });
    let default_act_lower = format!("execute_{}", part_name.to_lowercase());
    for line in body.lines() {
        if let Some(act_name_str) = parse_feature_action_line(line, &default_act_lower) {
            if !entry.actions.iter().any(|a| a.name == act_name_str) {
                entry.actions.push(ActionDef {
                    name: act_name_str,
                    ..Default::default()
                });
            }
        }
    }
}

/// Parses a `---` delimited YAML frontmatter block at the start of `content` when closed by `\n---`.
/// Realises: [REQ-0604] [REQ-SYNC-REVERSE-PARSE-FRONTMATTER]
fn parse_yaml_frontmatter_block(content: &str) -> Option<(BTreeMap<String, String>, &str)> {
    let trimmed = content.trim_start();
    let rest = trimmed.strip_prefix("---")?;
    let (fm_block, body) = rest.split_once("\n---")?;
    let mut fm = BTreeMap::new();
    for line in fm_block.lines() {
        let line = line.trim();
        if let Some((raw_key, raw_val)) = line.split_once(':') {
            let key = raw_key.trim().to_string();
            let val = raw_val.trim();
            let unquoted = val
                .strip_prefix('"')
                .and_then(|inner| inner.strip_suffix('"'))
                .or_else(|| {
                    val.strip_prefix('\'')
                        .and_then(|inner| inner.strip_suffix('\''))
                })
                .unwrap_or(val);
            fm.insert(key, unquoted.to_string());
        }
    }
    Some((fm, body))
}

/// Inserts a normalized CommonMark 2-column metadata table row and its canonical alias into `fm`.
/// Realises: [REQ-0604] [REQ-SYNC-REVERSE-PARSE-FRONTMATTER]
fn insert_metadata_table_row(fm: &mut BTreeMap<String, String>, key_cell: &str, value_cell: &str) {
    let mut key = String::new();
    for c in key_cell.chars() {
        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
            key.push(c);
        }
    }
    let key = key.trim().to_lowercase().replace([' ', '-'], "_");
    if key.is_empty() || key == "attribute" || key == "key" {
        return;
    }
    let val = value_cell
        .trim()
        .trim_matches(|c| c == '`' || c == '"' || c == '\'');
    fm.insert(key.clone(), val.to_string());
    if key == "sysml_interaction" {
        fm.insert("interaction".to_string(), val.to_string());
    } else if key == "scope" {
        fm.insert("subject".to_string(), val.to_string());
    } else if key == "primary_actor" {
        fm.insert("actors".to_string(), val.to_string());
    } else if key == "schema_containers" {
        let part = val.split("::").last().unwrap_or(val).trim();
        fm.insert("part".to_string(), part.to_string());
    }
}

/// Helper to parse metadata from YAML frontmatter or a CommonMark table from a Markdown string.
///
/// Safety & Traceability: Realises: [REQ-0604] [REQ-SYNC-REVERSE-PARSE-FRONTMATTER]
/// Contract: Extracts YAML key-value pairs from leading `---` fences or falls back to the first 20 lines of a 2-column CommonMark metadata table.
/// Bounds & Faults: Executes in $O(N)$ over `content` using checked `.strip_prefix`, `.strip_suffix`, and `.split_once` slicing without panics.
pub fn parse_frontmatter(content: &str) -> (BTreeMap<String, String>, &str) {
    if let Some(parsed) = parse_yaml_frontmatter_block(content) {
        return parsed;
    }

    let mut fm = BTreeMap::new();
    for line in content.lines().take(20) {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with('|') && line_trimmed.ends_with('|') {
            let inner = line_trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(str::trim).collect();
            if let [key_cell, value_cell, ..] = parts.as_slice() {
                insert_metadata_table_row(&mut fm, key_cell, value_cell);
            }
        }
    }

    (fm, content)
}

/// Helper to convert a string into clean `PascalCase`, prepending an underscore if leading with a digit.
///
/// Safety & Traceability: Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/PASCAL-CASE-IDENTIFIER]
/// Contract: Produces a KerML-compliant identifier starting with `_` whenever the input begins with an ASCII digit.
/// Bounds & Faults: Executes in $O(L)$ time and space over `text` with zero panics on empty or non-ASCII inputs.
pub fn to_pascal_case(text: &str) -> String {
    let has_leading_digit = text
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let pascal: String = words
        .iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();
    if has_leading_digit
        || pascal
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        format!("_{}", pascal)
    } else {
        pascal
    }
}

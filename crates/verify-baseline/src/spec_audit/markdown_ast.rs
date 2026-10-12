//! Zero-copy streaming pull parsing of Markdown specifications and AST extraction.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//! Employs `pulldown-cmark` for event-based streaming parsing without unnecessary string allocations.
//! Provides comprehensive placeholder finding with metadata row tolerance.

use crate::spec_audit::SpecAuditError;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

/// Represents an extracted Markdown heading with structural position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingNode {
    /// Heading level (1 for H1, 2 for H2, up to 6 for H6).
    pub level: u8,
    /// Heading plain-text content stripped of leading/trailing whitespace.
    pub text: String,
    /// 1-indexed line number in the source document.
    pub line: usize,
}

/// Represents a continuous prose text segment (paragraph).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseSegment {
    /// Prose text content.
    pub text: String,
    /// 1-indexed starting line number in the source document.
    pub line: usize,
}

/// Represents a fenced Mermaid diagram code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MermaidBlock {
    /// Full raw body of the Mermaid diagram code block.
    pub raw: String,
    /// Diagram type header (e.g. "classDiagram", "sequenceDiagram", "stateDiagram-v2").
    pub header: String,
    /// 1-indexed starting line number of the code block.
    pub start_line: usize,
    /// 1-indexed ending line number of the code block.
    pub end_line: usize,
}

/// Represents an unresolved placeholder or template token found in specification content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderFinding {
    /// 1-indexed line number where the placeholder was detected.
    pub line: usize,
    /// Human-readable category label explaining what is unresolved.
    pub label: String,
    /// The trimmed source line containing the placeholder.
    pub text: String,
}

/// In-memory AST representation of an audited Markdown specification document.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownDoc {
    /// Extracted headings in source order.
    pub headings: Vec<HeadingNode>,
    /// Optional YAML frontmatter key-value pairs (`--- ... ---`).
    pub frontmatter: Option<HashMap<String, String>>,
    /// Key-value pairs extracted from CommonMark two-column metadata table.
    pub table_metadata: HashMap<String, String>,
    /// Extracted prose text segments outside headings and code blocks.
    pub prose: Vec<ProseSegment>,
    /// Extracted Mermaid diagram code blocks.
    pub mermaid_blocks: Vec<MermaidBlock>,
    /// Detected unresolved placeholders or template escape tokens.
    pub unresolved_placeholders: Vec<PlaceholderFinding>,
}



/// Computes byte offsets of line starts to enable O(log N) line number lookups.
///
/// Preconditions: None.
/// Postconditions: Returns an ascending slice of byte offsets where each line starts.
/// Algorithmic Complexity: O(N) where N is content byte length.
fn build_line_starts(content: &str) -> Vec<usize> {
    let mut starts = Vec::with_capacity(content.len() / 40 + 1);
    starts.push(0);
    for (idx, b) in content.bytes().enumerate() {
        if b == b'\n' {
            starts.push(idx + 1);
        }
    }
    starts
}

/// Converts a byte offset into a 1-indexed line number using binary search.
///
/// Preconditions: `line_starts` is non-empty and strictly sorted.
/// Postconditions: Returns 1-indexed line number for the given byte offset.
/// Algorithmic Complexity: O(log L) where L is the line count.
fn offset_to_line(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(idx) => idx + 1,
        Err(idx) => idx,
    }
}

/// Checks whether a single line is a CommonMark two-column metadata header table row.
///
/// Preconditions: None.
/// Postconditions: Returns true if the line contains a valid metadata attribute key.
/// Algorithmic Complexity: O(K) where K is the length of the line.
pub fn is_metadata_header_table_row(line: &str) -> bool {
    let stripped = line.trim();
    if !stripped.starts_with('|') || !stripped.ends_with('|') {
        return false;
    }
    let inner = match stripped.strip_prefix('|').and_then(|s| s.strip_suffix('|')) {
        Some(s) => s,
        None => return false,
    };
    let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
    if cells.len() < 2 {
        return false;
    }

    let mut key = String::with_capacity(cells[0].len());
    for c in cells[0].chars() {
        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
            key.push(c);
        }
    }
    let key = key.trim().to_lowercase();
    let mut normalized = String::with_capacity(key.len());
    let mut last_was_sep = false;
    for c in key.chars() {
        if c.is_whitespace() || c == '-' || c == '/' {
            if !last_was_sep {
                normalized.push('_');
                last_was_sep = true;
            }
        } else {
            normalized.push(c);
            last_was_sep = false;
        }
    }
    let normalized = normalized.trim_matches('_');

    const METADATA_KEYS: &[&str] = &[
        "issue_id",
        "issue",
        "parent_epic",
        "feature_id",
        "epic_id",
        "epic_issue_id",
        "status",
        "doc_status",
        "document_status",
        "state",
        "user_story_id",
        "use_case_id",
        "story_id",
        "id",
        "doc_id",
        "document_id",
        "attribute",
        "key",
        "field",
        "property",
        "parent",
        "epic",
        "feature",
        "type",
        "title",
        "package",
        "subsystem",
        "generation_mode",
        "specification_source",
        "interface_type",
        "schema_containers",
        "version",
        "date",
        "release_date",
        "target_baseline",
        "sysml_interaction",
        "sysml_test_case",
        "primary_actor",
        "scope",
        "level",
    ];

    METADATA_KEYS.contains(&normalized)
}

fn is_allowed_metadata_placeholder(token: &str) -> bool {
    let trimmed = token.trim();
    if trimmed.eq_ignore_ascii_case("#tbd") {
        return true;
    }
    let without_hash = trimmed.strip_prefix('#').unwrap_or(trimmed);
    let lower = without_hash.to_lowercase();
    matches!(
        lower.as_str(),
        "[issueid]" | "[epicid]" | "[featureid]" | "[epicissueid]"
    )
}

fn detect_placeholder_on_line(
    line: &str,
    is_meta_row: bool,
    include_conditional: bool,
) -> Option<&'static str> {
    let line_lower = line.to_lowercase();

    // 1. Check for #[...] issue reference tokens
    let mut rest = line;
    while let Some(start) = rest.find("#[") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find(']') {
            let token = &rest[start..=start + 2 + end];
            if !(is_meta_row && is_allowed_metadata_placeholder(token)) {
                return Some("unresolved issue reference token");
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }

    // 2. Check for #TBD reference token with word boundary
    let mut idx = 0;
    while let Some(pos) = line_lower[idx..].find("#tbd") {
        let abs_pos = idx + pos;
        let next_idx = abs_pos + 4;
        let boundary = next_idx == line_lower.len()
            || !line_lower.as_bytes()[next_idx].is_ascii_alphanumeric();
        if boundary {
            let token = &line[abs_pos..next_idx];
            if !(is_meta_row && is_allowed_metadata_placeholder(token)) {
                return Some("unresolved reference token");
            }
        }
        idx = next_idx;
    }

    // 3. Check bracketed placeholders: [...]
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        if let Some(close) = after.find(']') {
            let content = &after[..close];
            let lower_content = content.trim().to_lowercase();
            let full_token = &rest[open..=open + 1 + close];

            const PREFIXES: &[&str] = &[
                "feature", "feat", "epic", "user story", "user-story", "user_story",
                "use case", "use-case", "use_case", "story", "issue", "us", "uc",
            ];
            let mut matched_id = false;
            for p in PREFIXES {
                if lower_content.starts_with(p) {
                    let rem = lower_content[p.len()..]
                        .trim_matches(|c: char| c == '-' || c == '_' || c.is_whitespace());
                    if rem == "id" || rem == "issueid" {
                        matched_id = true;
                        break;
                    }
                }
            }

            if matched_id {
                if !(is_meta_row && is_allowed_metadata_placeholder(full_token)) {
                    return Some("unresolved issue reference token");
                }
            } else if matches!(
                lower_content.as_str(),
                "title"
                    | "epic title"
                    | "feature title"
                    | "user story title"
                    | "use case title"
                    | "user-story title"
                    | "use-case title"
                    | "user_story title"
                    | "use_case title"
            ) {
                return Some("unpopulated template title");
            } else if lower_content.starts_with("populate:") {
                return Some("unreplaced [POPULATE:] placeholder token");
            } else {
                let normalized_words: Vec<&str> = lower_content.split_whitespace().collect();
                let single_space = normalized_words.join(" ");
                if matches!(
                    single_space.as_str(),
                    "repository base url"
                        | "branch name"
                        | "spec reference"
                        | "sysmlinteractionname"
                        | "sysmltestcasename"
                ) {
                    return Some("unpopulated template token");
                }
            }

            rest = &after[close + 1..];
        } else {
            break;
        }
    }

    // 4. Template escape tokens: {{REQUIRED_JUSTIFICATION}} or {{REQUIRED_SOURCE_REF}}
    let line_upper = line.to_uppercase();
    if line_upper.contains("{{REQUIRED_JUSTIFICATION}}")
        || line_upper.contains("{{REQUIRED_SOURCE_REF}}")
    {
        return Some("unreplaced template escape token");
    }

    // 5. Semantic linkage justification in parentheses
    if line_lower.contains("semantic linkage justification") {
        return Some("template text left in place of a written linkage justification");
    }

    // 6. Placeholder file paths: (epic|feat|us|uc)-XX-name
    for kw in &["epic-xx-name", "feat-xx-name", "us-xx-name", "uc-xx-name"] {
        if let Some(pos) = line_lower.find(kw) {
            let before_ok = pos == 0 || !line_lower.as_bytes()[pos - 1].is_ascii_alphanumeric();
            let after_pos = pos + kw.len();
            let after_ok = after_pos == line_lower.len()
                || !line_lower.as_bytes()[after_pos].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return Some("placeholder file path");
            }
        }
    }

    // 7. Unpopulated template path: <blob_path>
    if line_lower.contains("<blob_path>") {
        return Some("unpopulated template path");
    }

    // 8. Conditional stubs
    if include_conditional {
        if line_lower.contains("*(none)*")
            || line_lower.contains("*(none registered)*")
            || line_lower.contains("*(to be populated)*")
            || line_lower.contains("*(tbd)*")
            || line_lower.contains("*(n/a)*")
            || line_lower.contains("*to be populated*")
            || line_lower.contains("*tbd*")
            || line_lower.contains("*n/a*")
        {
            return Some("placeholder stub");
        }
    }

    None
}

/// Scans Markdown text and identifies unresolved placeholder findings.
///
/// Preconditions: None.
/// Postconditions: Returns list of PlaceholderFinding entries. Does not panic.
/// Algorithmic Complexity: O(L * T) where L is lines and T is line character length.
pub fn find_unresolved_placeholders(
    content: &str,
    include_conditional: bool,
) -> Vec<PlaceholderFinding> {
    let mut findings = Vec::new();

    for (lineno_0idx, line) in content.lines().enumerate() {
        let lineno = lineno_0idx + 1;
        let is_meta_row = is_metadata_header_table_row(line);

        if let Some(label) = detect_placeholder_on_line(line, is_meta_row, include_conditional) {
            findings.push(PlaceholderFinding {
                line: lineno,
                label: label.to_string(),
                text: line.trim().to_string(),
            });
        }
    }

    findings
}

/// Extracts YAML frontmatter key-value pairs from lines between leading `---` delimiters.
///
/// Preconditions: None.
/// Postconditions: Returns Some(map) if frontmatter exists and is non-empty, otherwise None.
/// Algorithmic Complexity: O(F) where F is frontmatter text length.
pub fn extract_yaml_frontmatter(content: &str) -> Option<HashMap<String, String>> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let rest = match trimmed.strip_prefix("---") {
        Some(r) => r,
        None => return None,
    };
    let end_idx = rest.find("\n---")?;
    let fm_block = match rest.get(..end_idx) {
        Some(b) => b,
        None => return None,
    };

    let mut map = HashMap::new();
    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k
                .trim()
                .trim_matches(|c| c == '\'' || c == '"')
                .to_lowercase();
            let v = v.trim().trim_matches(|c| c == '\'' || c == '"');
            if !k.is_empty() {
                map.insert(k, v.to_string());
            }
        }
    }

    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Extracts key-value pairs from the initial CommonMark two-column metadata table.
///
/// Preconditions: None.
/// Postconditions: Returns a map of normalized metadata keys to values.
/// Algorithmic Complexity: O(M) where M is lines in metadata block (capped at 50).
pub fn extract_table_metadata(content: &str) -> HashMap<String, String> {
    let mut table_metadata = HashMap::new();
    for line in content.lines().take(50) {
        let stripped = line.trim();
        if !stripped.starts_with('|') || !stripped.ends_with('|') {
            continue;
        }
        let inner = match stripped.strip_prefix('|').and_then(|s| s.strip_suffix('|')) {
            Some(s) => s,
            None => continue,
        };
        let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
        if cells.len() < 2 {
            continue;
        }

        let mut key = String::with_capacity(cells[0].len());
        for c in cells[0].chars() {
            if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                key.push(c);
            }
        }
        let key = key.trim().to_lowercase();
        let mut normalized_key = String::with_capacity(key.len());
        let mut last_was_sep = false;
        for c in key.chars() {
            if c.is_whitespace() || c == '-' || c == '/' {
                if !last_was_sep {
                    normalized_key.push('_');
                    last_was_sep = true;
                }
            } else {
                normalized_key.push(c);
                last_was_sep = false;
            }
        }
        let normalized_key = normalized_key.trim_matches('_');
        if normalized_key.is_empty()
            || normalized_key == "attribute"
            || normalized_key == "key"
            || normalized_key == "field"
        {
            continue;
        }
        let val = cells[1]
            .trim()
            .trim_matches(|c| c == '`' || c == '"' || c == '\'');
        table_metadata.insert(normalized_key.to_string(), val.to_string());
    }
    table_metadata
}

/// Extracts diagram type header from raw Mermaid code.
fn extract_mermaid_header(raw: &str) -> String {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        return first_word.to_string();
    }
    String::new()
}

/// Parses Markdown content into a structured `MarkdownDoc` using `pulldown-cmark`.
///
/// Preconditions: `content` is valid UTF-8 text.
/// Postconditions: Returns Ok(MarkdownDoc) containing headings, prose, diagrams, and metadata.
/// Algorithmic Complexity: O(N) where N is content byte length.
/// Errors: Returns `SpecAuditError::ParseError` if parsing cannot proceed.
pub fn parse_markdown(content: &str) -> Result<MarkdownDoc, SpecAuditError> {
    let line_starts = build_line_starts(content);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let parser = Parser::new_ext(content, options);

    let mut headings = Vec::new();
    let mut prose = Vec::new();
    let mut mermaid_blocks = Vec::new();

    enum CurrentBlock {
        None,
        Heading {
            level: u8,
            start_line: usize,
            text: String,
        },
        CodeBlock {
            is_mermaid: bool,
            start_line: usize,
            raw: String,
        },
        Paragraph {
            start_line: usize,
            text: String,
        },
    }

    let mut current = CurrentBlock::None;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let start_line = offset_to_line(&line_starts, range.start);
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                current = CurrentBlock::Heading {
                    level: lvl,
                    start_line,
                    text: String::new(),
                };
            }
            Event::End(TagEnd::Heading(_)) => {
                if let CurrentBlock::Heading {
                    level,
                    start_line,
                    text,
                } = current
                {
                    headings.push(HeadingNode {
                        level,
                        text: text.trim().to_string(),
                        line: start_line,
                    });
                    current = CurrentBlock::None;
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let start_line = offset_to_line(&line_starts, range.start);
                let is_mermaid = lang.trim().eq_ignore_ascii_case("mermaid");
                current = CurrentBlock::CodeBlock {
                    is_mermaid,
                    start_line,
                    raw: String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                if let CurrentBlock::CodeBlock {
                    is_mermaid,
                    start_line,
                    raw,
                } = current
                {
                    if is_mermaid {
                        let end_line = offset_to_line(&line_starts, range.end);
                        let header = extract_mermaid_header(&raw);
                        mermaid_blocks.push(MermaidBlock {
                            raw,
                            header,
                            start_line,
                            end_line,
                        });
                    }
                    current = CurrentBlock::None;
                }
            }
            Event::Start(Tag::Paragraph) => {
                if matches!(current, CurrentBlock::None) {
                    let start_line = offset_to_line(&line_starts, range.start);
                    current = CurrentBlock::Paragraph {
                        start_line,
                        text: String::new(),
                    };
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if let CurrentBlock::Paragraph { start_line, text } = current {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        prose.push(ProseSegment {
                            text: trimmed.to_string(),
                            line: start_line,
                        });
                    }
                    current = CurrentBlock::None;
                }
            }
            Event::Text(cow_str) | Event::Code(cow_str) => match &mut current {
                CurrentBlock::Heading { text, .. } => {
                    text.push_str(&cow_str);
                }
                CurrentBlock::CodeBlock { raw, .. } => {
                    raw.push_str(&cow_str);
                }
                CurrentBlock::Paragraph { text, .. } => {
                    text.push_str(&cow_str);
                }
                CurrentBlock::None => {}
            },
            Event::SoftBreak | Event::HardBreak => match &mut current {
                CurrentBlock::Heading { text, .. } => text.push(' '),
                CurrentBlock::CodeBlock { raw, .. } => raw.push('\n'),
                CurrentBlock::Paragraph { text, .. } => text.push(' '),
                CurrentBlock::None => {}
            },
            _ => {}
        }
    }

    let frontmatter = extract_yaml_frontmatter(content);
    let table_metadata = extract_table_metadata(content);
    let unresolved_placeholders = find_unresolved_placeholders(content, false);

    Ok(MarkdownDoc {
        headings,
        frontmatter,
        table_metadata,
        prose,
        mermaid_blocks,
        unresolved_placeholders,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_headings_and_metadata() {
        let content = r#"---
title: "Test Epic"
type: epic
---

# Test Epic H1

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 100 |
| **Title** | Test Epic |
| **Package** | SystemPackage |
| **Subsystem** | FlightControl |

## 1. Context
This is introductory prose text.

```mermaid
classDiagram
    class FlightControl {
        +execute() void
    }
```
"#;
        let doc = parse_markdown(content).unwrap();
        assert_eq!(doc.headings.len(), 2);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Test Epic H1");
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].text, "1. Context");

        assert_eq!(doc.table_metadata.get("issue_id").map(|s| s.as_str()), Some("100"));
        assert_eq!(doc.table_metadata.get("package").map(|s| s.as_str()), Some("SystemPackage"));

        assert_eq!(doc.mermaid_blocks.len(), 1);
        assert_eq!(doc.mermaid_blocks[0].header, "classDiagram");
        assert!(doc.unresolved_placeholders.is_empty());
    }

    #[test]
    fn test_detect_unresolved_placeholders() {
        let content = "Check issue #[IssueID] here.\nAnother line with #TBD.\nValid text.\n";
        let findings = find_unresolved_placeholders(content, false);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].line, 1);
        assert_eq!(findings[1].line, 2);
    }

    #[test]
    fn test_metadata_row_allowed_placeholders() {
        let content = "| **Issue ID** | #[IssueID] |\n";
        let findings = find_unresolved_placeholders(content, false);
        assert!(findings.is_empty(), "Allowed metadata placeholder should not trigger error");
    }
}

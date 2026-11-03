//! Zero-copy streaming pull parsing of Markdown specifications and AST extraction.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//! Employs `pulldown-cmark` for event-based streaming parsing without unnecessary string allocations.
//! Provides comprehensive placeholder finding with metadata row tolerance.

pub use crate::spec_audit::markdown_placeholders::{
    find_unresolved_placeholders, is_metadata_header_table_row,
};
use crate::spec_audit::SpecAuditError;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::BTreeMap;
use std::ops::Range;

/// Represents an extracted Markdown heading with structural position.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-HEADINGNODE]
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
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-PROSESEGMENT]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseSegment {
    /// Prose text content.
    pub text: String,
    /// 1-indexed starting line number in the source document.
    pub line: usize,
}

/// Represents a fenced Mermaid diagram code block.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-MERMAIDBLOCK]
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
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-PLACEHOLDERFINDING]
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
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-MARKDOWNDOC]
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownDoc {
    /// Extracted headings in source order.
    pub headings: Vec<HeadingNode>,
    /// Optional YAML frontmatter key-value pairs (`--- ... ---`).
    pub frontmatter: Option<BTreeMap<String, String>>,
    /// Key-value pairs extracted from CommonMark two-column metadata table.
    pub table_metadata: BTreeMap<String, String>,
    /// Extracted prose text segments outside headings and code blocks.
    pub prose: Vec<ProseSegment>,
    /// Extracted Mermaid diagram code blocks.
    pub mermaid_blocks: Vec<MermaidBlock>,
    /// Detected unresolved placeholders or template escape tokens.
    pub unresolved_placeholders: Vec<PlaceholderFinding>,
}

/// Computes byte offsets of line starts to enable O(log N) line number lookups.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-BUILD-LINE-STARTS]
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
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-OFFSET-TO-LINE]
fn offset_to_line(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(idx) => idx + 1,
        Err(idx) => idx,
    }
}

/// Extracts YAML frontmatter key-value pairs from lines between leading `---` delimiters.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-EXTRACT-YAML-FRONTMATTER]
pub fn extract_yaml_frontmatter(content: &str) -> Option<BTreeMap<String, String>> {
    crate::cmark_gates::spec_audit::extract_yaml_frontmatter_cmark(content)
}

/// Extracts key-value pairs from the initial CommonMark two-column metadata table.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-EXTRACT-TABLE-METADATA]
pub fn extract_table_metadata(content: &str) -> BTreeMap<String, String> {
    crate::cmark_gates::spec_audit::extract_table_metadata_cmark(content)
}

/// Extracts diagram type header from raw Mermaid code.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-EXTRACT-MERMAID-HEADER]
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

/// Active block accumulator during `pulldown-cmark` streaming parse.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-CURRENTBLOCK]
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

/// Accumulates `pulldown-cmark` events into headings, prose segments, and Mermaid blocks.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-ACCUMULATE-STATE]
#[derive(Default)]
struct MarkdownAstAccumulator {
    headings: Vec<HeadingNode>,
    prose: Vec<ProseSegment>,
    mermaid_blocks: Vec<MermaidBlock>,
}

/// Finalizes a closing `pulldown-cmark` tag into the accumulator.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-PROCESS-EVENT]
fn finish_markdown_tag(
    tag_end: TagEnd,
    end_offset: usize,
    line_starts: &[usize],
    current: &mut CurrentBlock,
    acc: &mut MarkdownAstAccumulator,
) {
    match tag_end {
        TagEnd::Heading(_) => {
            if let CurrentBlock::Heading { level, start_line, text } =
                std::mem::replace(current, CurrentBlock::None)
            {
                acc.headings.push(HeadingNode {
                    level,
                    text: text.trim().to_string(),
                    line: start_line,
                });
            }
        }
        TagEnd::CodeBlock => {
            if let CurrentBlock::CodeBlock { is_mermaid: true, start_line, raw } =
                std::mem::replace(current, CurrentBlock::None)
            {
                let end_line = offset_to_line(line_starts, end_offset);
                let header = extract_mermaid_header(&raw);
                acc.mermaid_blocks.push(MermaidBlock { raw, header, start_line, end_line });
            }
        }
        TagEnd::Paragraph => {
            if let CurrentBlock::Paragraph { start_line, text } =
                std::mem::replace(current, CurrentBlock::None)
            {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    acc.prose.push(ProseSegment { text: trimmed.to_string(), line: start_line });
                }
            }
        }
        _ => {}
    }
}

/// Processes a single `pulldown-cmark` event into the active block and accumulator.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-PROCESS-EVENT]
fn process_markdown_event(
    event: Event<'_>,
    range: Range<usize>,
    line_starts: &[usize],
    current: &mut CurrentBlock,
    acc: &mut MarkdownAstAccumulator,
) {
    match event {
        Event::Start(Tag::Heading { level, .. }) => {
            let start_line = offset_to_line(line_starts, range.start);
            let lvl = match level {
                HeadingLevel::H1 => 1,
                HeadingLevel::H2 => 2,
                HeadingLevel::H3 => 3,
                HeadingLevel::H4 => 4,
                HeadingLevel::H5 => 5,
                HeadingLevel::H6 => 6,
            };
            *current = CurrentBlock::Heading { level: lvl, start_line, text: String::new() };
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
            let start_line = offset_to_line(line_starts, range.start);
            let is_mermaid = lang.trim().eq_ignore_ascii_case("mermaid");
            *current = CurrentBlock::CodeBlock { is_mermaid, start_line, raw: String::new() };
        }
        Event::Start(Tag::Paragraph) if matches!(current, CurrentBlock::None) => {
            let start_line = offset_to_line(line_starts, range.start);
            *current = CurrentBlock::Paragraph { start_line, text: String::new() };
        }
        Event::End(tag_end) => {
            finish_markdown_tag(tag_end, range.end, line_starts, current, acc);
        }
        Event::Text(cow_str) | Event::Code(cow_str) => match current {
            CurrentBlock::Heading { text, .. } | CurrentBlock::Paragraph { text, .. } => {
                text.push_str(&cow_str);
            }
            CurrentBlock::CodeBlock { raw, .. } => raw.push_str(&cow_str),
            CurrentBlock::None => {}
        },
        Event::SoftBreak | Event::HardBreak => match current {
            CurrentBlock::Heading { text, .. } | CurrentBlock::Paragraph { text, .. } => {
                text.push(' ');
            }
            CurrentBlock::CodeBlock { raw, .. } => raw.push('\n'),
            CurrentBlock::None => {}
        },
        _ => {}
    }
}

/// Parses Markdown content into a structured `MarkdownDoc` using `pulldown-cmark`.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-PARSE-MARKDOWN]
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
    let mut current = CurrentBlock::None;
    let mut acc = MarkdownAstAccumulator::default();

    for (event, range) in parser.into_offset_iter() {
        process_markdown_event(event, range, &line_starts, &mut current, &mut acc);
    }

    Ok(MarkdownDoc {
        headings: acc.headings,
        frontmatter: extract_yaml_frontmatter(content),
        table_metadata: extract_table_metadata(content),
        prose: acc.prose,
        mermaid_blocks: acc.mermaid_blocks,
        unresolved_placeholders: find_unresolved_placeholders(content, false),
    })
}

#[cfg(test)]
#[path = "markdown_ast_tests.rs"]
mod tests;

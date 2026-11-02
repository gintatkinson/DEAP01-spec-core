//! Markdown code block, GLFM inline math, and inline code sanitizers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives.
//! Masks code spans and GLFM math before mathematical or prose syntax inspection.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Line and character offsets are preserved exactly across all sanitization phases.
//!
//! ## 3. Preconditions
//! - Caller provides valid UTF-8 Markdown text.
//!
//! ## 4. Postconditions
//! - Returns sanitized UTF-8 text with matched code spans replaced by ASCII whitespace.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) typical linear traversal.
//! - Space Complexity: O(N) auxiliary string buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Total non-panicking transformations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-MD-MODULE], [DO-330-TQL-4-MARKDOWN].

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Masks all non-newline characters within the sorted `ranges` of `content` with ASCII spaces.
/// Realises: [REQ-MD-MODULE]
fn mask_byte_ranges(content: &str, ranges: &[Range<usize>]) -> String {
    let mut out = String::with_capacity(content.len());
    let mut range_idx = 0;
    let mut byte_idx = 0;

    for ch in content.chars() {
        let ch_len = ch.len_utf8();
        while range_idx < ranges.len()
            && ranges.get(range_idx).is_some_and(|r| byte_idx >= r.end)
        {
            range_idx += 1;
        }

        let in_masked = ranges
            .get(range_idx)
            .is_some_and(|r| byte_idx >= r.start && byte_idx < r.end);

        if in_masked && ch != '\n' && ch != '\r' {
            out.push(' ');
        } else {
            out.push(ch);
        }
        byte_idx += ch_len;
    }

    out
}

/// [DOC-TIER: A] Removes fenced code blocks (` ```...``` ` and `~~~...~~~`), replacing non-newline characters
/// with ASCII spaces so downstream line and column offsets remain invariant.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents code literals inside fenced blocks
/// from triggering false-positive KaTeX math delimiter or prose linter diagnostics.
///
/// ## 2. Invariants & Mathematical Properties
/// - Line Preservation Invariant: `output.lines().count() == content.lines().count()`.
/// - Fence Matching Property: Matches opening fence of length $k \ge 3$ until closing fence line.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown string slice.
///
/// ## 4. Postconditions
/// Returns a newly allocated `String` where all fenced code regions are blanked with `' '`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) single pass over characters. Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function. Unclosed fences blank all remaining characters through EOF.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-FENCES]
pub fn strip_fenced_code_blocks(content: &str) -> String {
    let mut code_ranges = Vec::new();
    let mut current_block_start = None;

    for (event, range) in Parser::new_ext(content, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                current_block_start = Some(range.start);
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(start) = current_block_start.take() {
                    code_ranges.push(start..range.end);
                }
            }
            _ => {}
        }
    }
    if let Some(start) = current_block_start {
        code_ranges.push(start..content.len());
    }

    mask_byte_ranges(content, &code_ranges)
}

/// [DOC-TIER: A] Removes GitLab Flavored Markdown (GLFM) inline math spans (`$`...`$` or `$``...``$`),
/// replacing non-newline characters with spaces so adjacent `$` signs cannot coalesce into false `$$` tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Eliminates false `$$` display-math delimiter
/// synthesis when stripping backtick spans inside GLFM `$`+...`+$` math expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Character-Length Invariant: `output.chars().count() == content.chars().count()`.
/// - CommonMark Event-Offset Span Masking: Uses `pulldown_cmark::Parser` code span offsets.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// Returns a `String` with all matched GLFM inline math spans replaced by ASCII spaces.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) typical linear scan. Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function using `pulldown_cmark` event offsets.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-GLFM]
pub fn strip_glfm_inline_math(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut glfm_ranges = Vec::new();

    for (event, range) in Parser::new_ext(content, Options::empty()).into_offset_iter() {
        if let Event::Code(code_str) = event {
            if range.start > 0
                && bytes.get(range.start - 1) == Some(&b'$')
                && bytes.get(range.end) == Some(&b'$')
            {
                glfm_ranges.push((range.start - 1)..(range.end + 1));
            } else if code_str.starts_with("$`") && code_str.ends_with("`$") {
                glfm_ranges.push(range);
            }
        }
    }

    mask_byte_ranges(content, &glfm_ranges)
}

/// [DOC-TIER: A] Removes CommonMark inline code spans delimited by matching backtick runs of length $k \ge 1$,
/// replacing non-newline characters with ASCII spaces.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Masks inline code literals before syntax validation.
///
/// ## 2. Invariants & Mathematical Properties
/// - Character-Length Invariant: `output.chars().count() == content.chars().count()`.
/// - CommonMark Backtick Matching: Driven by `pulldown_cmark::Event::Code` offset spans.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// Returns a `String` where every matched inline code span is overwritten with `' '`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) typical. Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function using `pulldown_cmark` event offsets.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-INLINE]
pub fn strip_inline_code(content: &str) -> String {
    let mut code_ranges = Vec::new();

    for (event, range) in Parser::new_ext(content, Options::empty()).into_offset_iter() {
        if matches!(event, Event::Code(_)) {
            code_ranges.push(range);
        }
    }

    mask_byte_ranges(content, &code_ranges)
}

/// [DOC-TIER: A] Sequentially strips fenced code blocks, GLFM inline math (`$`...`$` ), and standard inline code
/// spans so only active markdown prose and display/inline LaTeX math remain.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces mandatory 3-stage sanitization ordering.
///
/// ## 2. Invariants & Mathematical Properties
/// Functional composition preserving total line count and newline offsets across all stages.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown document slice.
///
/// ## 4. Postconditions
/// Returns a sanitized `String` with all fenced blocks, GLFM inline math, and code spans masked.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) typical. Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-STRIP]
pub fn strip_code_and_glfm(content: &str) -> String {
    let no_fences = strip_fenced_code_blocks(content);
    let no_glfm = strip_glfm_inline_math(&no_fences);
    strip_inline_code(&no_glfm)
}

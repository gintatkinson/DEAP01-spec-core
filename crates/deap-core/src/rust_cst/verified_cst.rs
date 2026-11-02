#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless CST parsing, byte parity verification, and iterative depth control. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Implements bounded, lossless CST ingestion and GreenNode parity verification (TD-05 / US-56).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Leaf token byte concatenation equals source_text.as_bytes() bit-for-bit.
//! - CST nesting depth is strictly bounded by MAX_AST_DEPTH (64) via iterative worklist.
//!
//! ## 3. Preconditions
//! - Callers pass UTF-8 Rust source bytes and validated CstMetricsThresholds.
//!
//! ## 4. Postconditions
//! - Returns sealed VerifiedLosslessCst authorizing AnalyzingMetricsAndTiers.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N) linear time over N source bytes; O(N) heap space with zero recursion.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fails closed with typed CstMetricsError (DEAP-E0802..DEAP-E0804) on syntax or depth drift.
//!
//! ## 7. Realization & Traceability
//! Realises: [LosslessRustCstAnalyzer/parse_verified_lossless_cst], [REQ-0608]
//! SysML v2: part def LosslessRustCstAnalyzer

use super::envelope::MAX_AST_DEPTH;
use super::thresholds::{validate_metrics_thresholds, CstMetricsError, CstMetricsThresholds};

/// Sealed proof-carrying lossless ra_ap_syntax and rowan CST token.
/// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst], [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLosslessCst {
    source_path: String,
    source_text: String,
    green_node: rowan::GreenNode,
    source_file: ra_ap_syntax::SourceFile,
    line_starts: Vec<usize>,
    observed_max_depth: usize,
    verified_node_count: usize,
    lossless_red_green_tree_ready: bool,
}

impl VerifiedLosslessCst {
    pub fn source_path(&self) -> &str { &self.source_path }
    pub fn source_text(&self) -> &str { &self.source_text }
    pub fn source_bytes(&self) -> &[u8] { self.source_text.as_bytes() }
    pub const fn green_node(&self) -> &rowan::GreenNode { &self.green_node }
    pub const fn source_file(&self) -> &ra_ap_syntax::SourceFile { &self.source_file }
    pub fn syntax_node(&self) -> &ra_ap_syntax::SyntaxNode {
        ra_ap_syntax::AstNode::syntax(&self.source_file)
    }
    pub fn line_starts(&self) -> &[usize] { &self.line_starts }
    pub const fn observed_max_depth(&self) -> usize { self.observed_max_depth }
    pub const fn verified_node_count(&self) -> usize { self.verified_node_count }
    pub const fn is_lossless_red_green_tree_ready(&self) -> bool { self.lossless_red_green_tree_ready }
}

fn decode_strict_utf8_source(source_bytes: &[u8]) -> Result<&str, CstMetricsError> {
    std::str::from_utf8(source_bytes).map_err(|utf8_err| {
        let byte_start = utf8_err.valid_up_to();
        let byte_end = utf8_err.error_len().map_or(source_bytes.len(), |l| byte_start.saturating_add(l));
        CstMetricsError::MalformedUtf8OrRustSyntax {
            byte_start,
            byte_end,
            detail: utf8_err.to_string(),
        }
    })
}

pub(crate) fn verify_utf8_char_boundary_span(
    source_text: &str,
    byte_start: usize,
    byte_end: usize,
) -> Result<(), CstMetricsError> {
    if byte_start > byte_end || byte_end > source_text.len()
        || !source_text.is_char_boundary(byte_start)
        || !source_text.is_char_boundary(byte_end)
    {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start, byte_end });
    }
    Ok(())
}

pub(crate) const fn depth_exhaustion_err(
    observed_depth: usize,
    max_depth: usize,
    byte_offset: usize,
) -> CstMetricsError {
    CstMetricsError::AstDepthOrArithmeticExhaustion { observed_depth, max_depth, byte_offset }
}

pub(crate) fn checked_next_cst_depth(
    current_depth: usize,
    max_depth: usize,
    byte_offset: usize,
) -> Result<usize, CstMetricsError> {
    let next = current_depth.checked_add(1)
        .ok_or_else(|| depth_exhaustion_err(usize::MAX, max_depth, byte_offset))?;
    if next > max_depth {
        return Err(depth_exhaustion_err(next, max_depth, byte_offset));
    }
    Ok(next)
}

fn verify_cst_leaf_token_parity(
    token: &ra_ap_syntax::SyntaxToken,
    source_text: &str,
    cursor: usize,
    max_depth: usize,
) -> Result<usize, CstMetricsError> {
    let range = token.text_range();
    let (start, end) = (u32::from(range.start()) as usize, u32::from(range.end()) as usize);
    verify_utf8_char_boundary_span(source_text, start, end)?;
    let token_bytes = token.text().as_bytes();
    let next_cursor = cursor.checked_add(token_bytes.len())
        .ok_or_else(|| depth_exhaustion_err(usize::MAX, max_depth, cursor))?;
    if start != cursor || end != next_cursor || source_text.as_bytes().get(start..end) != Some(token_bytes) {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: start, byte_end: end });
    }
    Ok(next_cursor)
}

fn push_cst_children_reversed(
    node: &ra_ap_syntax::SyntaxNode,
    depth: usize,
    max_depth: usize,
    stack: &mut Vec<(ra_ap_syntax::SyntaxElement, usize)>,
) -> Result<(), CstMetricsError> {
    let children: Vec<ra_ap_syntax::SyntaxElement> = node.children_with_tokens().collect();
    for child in children.into_iter().rev() {
        let child_depth = match &child {
            ra_ap_syntax::SyntaxElement::Node(cn) => {
                let off = u32::from(cn.text_range().start()) as usize;
                checked_next_cst_depth(depth, max_depth, off)?
            }
            ra_ap_syntax::SyntaxElement::Token(_) => depth,
        };
        stack.push((child, child_depth));
    }
    Ok(())
}

pub(crate) fn verify_green_tree_roundtrip_and_depth(
    green_node: &rowan::GreenNode,
    source_text: &str,
    max_depth: usize,
) -> Result<(usize, usize), CstMetricsError> {
    let root = ra_ap_syntax::SyntaxNode::new_root(green_node.clone());
    let mut stack = vec![(ra_ap_syntax::SyntaxElement::Node(root), 0usize)];
    let (mut cursor, mut max_seen, mut node_count) = (0usize, 0usize, 0usize);
    while let Some((element, depth)) = stack.pop() {
        match element {
            ra_ap_syntax::SyntaxElement::Node(node) => {
                let range = node.text_range();
                let (start, end) = (u32::from(range.start()) as usize, u32::from(range.end()) as usize);
                if depth > max_depth {
                    return Err(depth_exhaustion_err(depth, max_depth, start));
                }
                verify_utf8_char_boundary_span(source_text, start, end)?;
                max_seen = max_seen.max(depth);
                node_count = node_count.checked_add(1)
                    .ok_or_else(|| depth_exhaustion_err(usize::MAX, max_depth, start))?;
                push_cst_children_reversed(&node, depth, max_depth, &mut stack)?;
            }
            ra_ap_syntax::SyntaxElement::Token(token) => {
                cursor = verify_cst_leaf_token_parity(&token, source_text, cursor, max_depth)?;
            }
        }
    }
    let green_len = u32::from(green_node.text_len()) as usize;
    if cursor != source_text.len() || green_len != source_text.len() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start: cursor.min(green_len),
            byte_end: source_text.len().max(green_len),
        });
    }
    Ok((max_seen, node_count))
}

fn compute_monotone_line_starts(source_text: &str) -> Result<Vec<usize>, CstMetricsError> {
    let mut starts = vec![0usize];
    for (idx, byte) in source_text.as_bytes().iter().copied().enumerate() {
        if byte == b'\n' {
            let next_start = idx.checked_add(1)
                .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, idx))?;
            starts.push(next_start);
        }
    }
    Ok(starts)
}

/// Parses raw Rust source bytes into a sealed VerifiedLosslessCst token.
/// Realises: [LosslessRustCstAnalyzer/parse_verified_lossless_cst], [REQ-0608]
pub fn parse_verified_lossless_cst(
    source_path: &str,
    source_bytes: &[u8],
    thresholds: &CstMetricsThresholds,
) -> Result<VerifiedLosslessCst, CstMetricsError> {
    validate_metrics_thresholds(thresholds)?;
    let source_text = decode_strict_utf8_source(source_bytes)?;
    let parse = ra_ap_syntax::SourceFile::parse(source_text, ra_ap_syntax::Edition::CURRENT);
    if let Some(first_err) = parse.errors().first() {
        let range = first_err.range();
        return Err(CstMetricsError::MalformedUtf8OrRustSyntax {
            byte_start: u32::from(range.start()) as usize,
            byte_end: u32::from(range.end()) as usize,
            detail: first_err.to_string(),
        });
    }
    let green_node = parse.syntax_node().green().to_owned();
    let (observed_max_depth, verified_node_count) =
        verify_green_tree_roundtrip_and_depth(&green_node, source_text, thresholds.max_ast_depth)?;
    let line_starts = compute_monotone_line_starts(source_text)?;
    Ok(VerifiedLosslessCst {
        source_path: source_path.to_string(),
        source_text: source_text.to_string(),
        green_node,
        source_file: parse.tree(),
        line_starts,
        observed_max_depth,
        verified_node_count,
        lossless_red_green_tree_ready: true,
    })
}

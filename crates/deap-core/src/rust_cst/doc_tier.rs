#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! DocTier taxonomy, test subtree pruning, and item classification. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Implements cfg(test) subtree pruning and DocTier item inventory synthesis (TD-06 / US-56).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Excludes 100% of #[cfg(test)] and #[test] subtrees from production items and SLOC.
//! - BTreeMap and BTreeSet backing ensures deterministic traversal across all platforms.
//!
//! ## 3. Preconditions
//! - Callers pass sealed VerifiedLosslessCst and validated CstMetricsThresholds.
//!
//! ## 4. Postconditions
//! - Returns DocTierInventory with partitioned RustTierA, RustTierBC, and TestExcluded counts.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N log L) time over N syntax elements and L lines; bounded heap space.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fails closed with CstMetricsError (DEAP-E0801..DEAP-E0804) on invalid state or depth breach.
//!
//! ## 7. Realization & Traceability
//! Realises: [LosslessRustCstAnalyzer/DocTier], [REQ-0608]
//! SysML v2: part def LosslessRustCstAnalyzer

use super::envelope::MAX_AST_DEPTH;
use super::thresholds::{validate_metrics_thresholds, CstMetricsError, CstMetricsThresholds};
use super::verified_cst::{
    checked_next_cst_depth, depth_exhaustion_err, verify_green_tree_roundtrip_and_depth,
    verify_utf8_char_boundary_span, VerifiedLosslessCst,
};
use ra_ap_syntax::ast::{HasName, HasVisibility};
use ra_ap_syntax::AstNode;
use std::collections::{BTreeMap, BTreeSet};

/// Documentation tier governing CST items and module headers.
/// Realises: [LosslessRustCstAnalyzer/DocTier], [REQ-0608]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum DocTier {
    RustTierA,
    RustTierBC,
    TestExcluded,
    TierA,
    TierB,
    TierC,
}

impl DocTier {
    pub const fn is_production(self) -> bool {
        matches!(self, Self::RustTierA | Self::RustTierBC | Self::TierA | Self::TierB | Self::TierC)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::RustTierA => "RustTierA",
            Self::RustTierBC => "RustTierBC",
            Self::TestExcluded => "TestExcluded",
            Self::TierA => "TierA",
            Self::TierB => "TierB",
            Self::TierC => "TierC",
        }
    }
}

/// Verified production DocTier descriptor for a single non-test Rust CST declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ClassifiedDocTierItem {
    pub symbol: String,
    pub tier: DocTier,
    pub byte_start: usize,
    pub byte_end: usize,
    pub doc_line_count: usize,
    pub is_callable: bool,
}

/// Deterministic BTreeMap/BTreeSet-ordered DocTier inventory.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DocTierInventory {
    pub source_path: String,
    pub production_items: BTreeMap<(u32, String), ClassifiedDocTierItem>,
    pub production_exec_lines: BTreeSet<usize>,
    pub production_exec_sloc: usize,
    pub rust_tier_a_count: usize,
    pub rust_tier_bc_count: usize,
    pub test_excluded_item_count: usize,
    pub pruned_test_subtree_ranges: Vec<(usize, usize)>,
}

fn new_empty_doc_tier_inventory(source_path: &str) -> DocTierInventory {
    DocTierInventory {
        source_path: source_path.to_string(),
        production_items: BTreeMap::new(),
        production_exec_lines: BTreeSet::new(),
        production_exec_sloc: 0,
        rust_tier_a_count: 0,
        rust_tier_bc_count: 0,
        test_excluded_item_count: 0,
        pruned_test_subtree_ranges: Vec::new(),
    }
}

pub(crate) fn is_trivia_token(tok: &ra_ap_syntax::SyntaxToken) -> bool {
    matches!(
        tok.kind(),
        ra_ap_syntax::SyntaxKind::WHITESPACE
            | ra_ap_syntax::SyntaxKind::COMMENT
            | ra_ap_syntax::SyntaxKind::OUTER_DOC_COMMENT
            | ra_ap_syntax::SyntaxKind::INNER_DOC_COMMENT
    )
}

fn collect_bounded_tokens(node: &ra_ap_syntax::SyntaxNode, limit: usize) -> Vec<ra_ap_syntax::SyntaxToken> {
    let mut out = Vec::new();
    for elem in node.descendants_with_tokens() {
        if let ra_ap_syntax::SyntaxElement::Token(tok) = elem {
            if !is_trivia_token(&tok) {
                out.push(tok);
                if out.len() >= limit { break; }
            }
        }
    }
    out
}

fn is_doc_meta_attr(attr: &ra_ap_syntax::ast::Attr) -> bool {
    let Some(meta) = attr.meta() else { return false; };
    let toks = collect_bounded_tokens(meta.syntax(), 3);
    toks.first().is_some_and(|t| t.text() == "doc")
        && toks.get(1).is_some_and(|t| t.text() == "=" || t.text() == "(")
}

pub(crate) fn is_inside_doc_attr(node: &ra_ap_syntax::SyntaxNode) -> bool {
    for ancestor in node.ancestors().take(MAX_AST_DEPTH.saturating_add(1)) {
        if let Some(attr) = ra_ap_syntax::ast::Attr::cast(ancestor) {
            if is_doc_meta_attr(&attr) { return true; }
        }
    }
    false
}

fn is_outer_cfg_test_attr(attr: &ra_ap_syntax::ast::Attr) -> bool {
    if attr.kind() != ra_ap_syntax::ast::AttrKind::Outer { return false; }
    let Some(meta) = attr.meta() else { return false; };
    let toks = collect_bounded_tokens(meta.syntax(), 5);
    toks.len() == 4
        && toks.first().is_some_and(|t| t.text() == "cfg")
        && toks.get(1).is_some_and(|t| t.text() == "(")
        && toks.get(2).is_some_and(|t| t.text() == "test")
        && toks.get(3).is_some_and(|t| t.text() == ")")
}

fn is_outer_test_fn_attr(attr: &ra_ap_syntax::ast::Attr) -> bool {
    if attr.kind() != ra_ap_syntax::ast::AttrKind::Outer { return false; }
    let Some(meta) = attr.meta() else { return false; };
    let toks = collect_bounded_tokens(meta.syntax(), 16);
    !toks.is_empty() && toks.len() < 16 && toks.last().is_some_and(|t| t.text() == "test")
}

pub(crate) fn node_enters_test_scope(node: &ra_ap_syntax::SyntaxNode) -> bool {
    if node.children().filter_map(ra_ap_syntax::ast::Attr::cast).any(|a| is_outer_cfg_test_attr(&a)) {
        return true;
    }
    if let Some(fn_item) = ra_ap_syntax::ast::Fn::cast(node.clone()) {
        return ra_ap_syntax::ast::HasAttrs::attrs(&fn_item).any(|a| is_outer_test_fn_attr(&a));
    }
    false
}

fn is_unrestricted_pub(vis: Option<ra_ap_syntax::ast::Visibility>) -> bool {
    let Some(v) = vis else { return false; };
    v.pub_token().is_some() && !v.syntax().descendants_with_tokens().any(|el| el.kind() == ra_ap_syntax::SyntaxKind::L_PAREN)
}

fn extract_named_target(node: &ra_ap_syntax::SyntaxNode, test_scope: bool) -> Option<(String, bool, bool)> {
    use ra_ap_syntax::ast::Item;
    let item = Item::cast(node.clone())?;
    match item {
        Item::Fn(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), true)),
        Item::Struct(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false)),
        Item::Enum(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false)),
        Item::Union(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false)),
        Item::Trait(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false)),
        Item::Const(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false)),
        Item::Static(it) => it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false)),
        Item::TypeAlias(it) if test_scope || it.syntax().parent().and_then(|p| p.parent()).and_then(ra_ap_syntax::ast::Impl::cast).and_then(|i| i.trait_()).is_none() => {
            it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false))
        }
        Item::Module(it) if test_scope || it.item_list().is_some() => {
            it.name().map(|n| (n.text().to_string(), is_unrestricted_pub(it.visibility()), false))
        }
        _ => None,
    }
}

fn outer_doc_lines(slice: &str) -> usize {
    if slice.starts_with("///") && !slice.starts_with("////") { 1 }
    else if slice.starts_with("/**") && !slice.starts_with("/***") && slice != "/**/" { slice.lines().count().max(1) }
    else { 0 }
}

fn extract_outer_doc_count(node: &ra_ap_syntax::SyntaxNode, src: &str) -> Result<usize, CstMetricsError> {
    let mut count = 0usize;
    for child in node.children_with_tokens() {
        let r = child.text_range();
        let (s, e) = (u32::from(r.start()) as usize, u32::from(r.end()) as usize);
        verify_utf8_char_boundary_span(src, s, e)?;
        match child {
            ra_ap_syntax::SyntaxElement::Token(tok) if tok.kind() == ra_ap_syntax::SyntaxKind::OUTER_DOC_COMMENT => {
                let sl = src.get(s..e).ok_or(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: s, byte_end: e })?;
                count = count.checked_add(outer_doc_lines(sl)).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, s))?;
            }
            ra_ap_syntax::SyntaxElement::Node(cn) if cn.kind() == ra_ap_syntax::SyntaxKind::DOC_COMMENT => {
                let sl = src.get(s..e).ok_or(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: s, byte_end: e })?;
                count = count.checked_add(outer_doc_lines(sl)).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, s))?;
            }
            ra_ap_syntax::SyntaxElement::Node(cn) => {
                let Some(attr) = ra_ap_syntax::ast::Attr::cast(cn) else { break; };
                if attr.kind() == ra_ap_syntax::ast::AttrKind::Outer && is_doc_meta_attr(&attr) {
                    count = count.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, s))?;
                }
            }
            _ => break,
        }
    }
    Ok(count)
}

pub(crate) fn resolve_byte_to_line(line_starts: &[usize], offset: usize) -> Result<usize, CstMetricsError> {
    let idx = match line_starts.binary_search(&offset) {
        Ok(i) => i,
        Err(pos) => pos.checked_sub(1).ok_or(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: offset, byte_end: offset })?,
    };
    line_starts.get(idx).map(|_| idx).ok_or(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: offset, byte_end: offset })
}

fn record_exec_lines(node: &ra_ap_syntax::SyntaxNode, src: &str, line_starts: &[usize], lines: &mut BTreeSet<usize>) -> Result<(), CstMetricsError> {
    if is_inside_doc_attr(node) { return Ok(()); }
    for child in node.children_with_tokens() {
        let ra_ap_syntax::SyntaxElement::Token(tok) = child else { continue; };
        if is_trivia_token(&tok) { continue; }
        let (s, e) = (u32::from(tok.text_range().start()) as usize, u32::from(tok.text_range().end()) as usize);
        verify_utf8_char_boundary_span(src, s, e)?;
        if let Some(last) = e.checked_sub(1) {
            if s <= last {
                let (f, l) = (resolve_byte_to_line(line_starts, s)?, resolve_byte_to_line(line_starts, last)?);
                let mut c = f;
                while c <= l {
                    lines.insert(c);
                    c = c.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, s))?;
                }
            }
        }
    }
    Ok(())
}

fn insert_prod_item(inv: &mut DocTierInventory, start_u32: u32, item: ClassifiedDocTierItem) -> Result<(), CstMetricsError> {
    let (bs, be) = (item.byte_start, item.byte_end);
    match item.tier {
        DocTier::RustTierA | DocTier::TierA => inv.rust_tier_a_count = inv.rust_tier_a_count.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, bs))?,
        DocTier::RustTierBC | DocTier::TierB | DocTier::TierC => inv.rust_tier_bc_count = inv.rust_tier_bc_count.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, bs))?,
        DocTier::TestExcluded => return Err(CstMetricsError::UnverifiedParsePrecondition { state: "AnalyzingMetricsAndTiers" }),
    }
    if inv.production_items.insert((start_u32, item.symbol.clone()), item).is_some() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: bs, byte_end: be });
    }
    Ok(())
}

pub(crate) fn push_tier_worklist_children(
    node: &ra_ap_syntax::SyntaxNode,
    depth: usize,
    max_depth: usize,
    test_scope: bool,
    stack: &mut Vec<(ra_ap_syntax::SyntaxNode, usize, bool)>,
) -> Result<(), CstMetricsError> {
    for child in node.children().collect::<Vec<_>>().into_iter().rev() {
        let off = u32::from(child.text_range().start()) as usize;
        let nd = checked_next_cst_depth(depth, max_depth, off)?;
        stack.push((child, nd, test_scope));
    }
    Ok(())
}

/// Prunes #[cfg(test)] and #[test] subtrees, classifying production items into DocTier.
/// Realises: [LosslessRustCstAnalyzer/classify_cfg_test_excluded_doc_tier], [REQ-0608]
pub fn classify_cfg_test_excluded_doc_tier(
    cst: &VerifiedLosslessCst,
    thresholds: &CstMetricsThresholds,
    is_generated_self_hosted: bool,
) -> Result<DocTierInventory, CstMetricsError> {
    if !cst.is_lossless_red_green_tree_ready() {
        return Err(CstMetricsError::UnverifiedParsePrecondition { state: "AnalyzingMetricsAndTiers" });
    }
    validate_metrics_thresholds(thresholds)?;
    verify_green_tree_roundtrip_and_depth(cst.green_node(), cst.source_text(), thresholds.max_ast_depth)?;
    let promote = is_generated_self_hosted || cst.source_path().ends_with(".g.rs");
    let mut inv = new_empty_doc_tier_inventory(cst.source_path());
    let mut stack = vec![(cst.syntax_node().clone(), 0usize, false)];
    while let Some((node, depth, in_test)) = stack.pop() {
        let (s, e) = (u32::from(node.text_range().start()) as usize, u32::from(node.text_range().end()) as usize);
        if depth > thresholds.max_ast_depth { return Err(depth_exhaustion_err(depth, thresholds.max_ast_depth, s)); }
        verify_utf8_char_boundary_span(cst.source_text(), s, e)?;
        let enters_test = !in_test && node_enters_test_scope(&node);
        if enters_test { inv.pruned_test_subtree_ranges.push((s, e)); }
        let eff_test = in_test || enters_test;
        if let Some((symbol, is_pub, is_callable)) = extract_named_target(&node, eff_test) {
            let doc_count = extract_outer_doc_count(&node, cst.source_text())?;
            if eff_test {
                inv.test_excluded_item_count = inv.test_excluded_item_count.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, s))?;
            } else {
                let tier = if is_pub || promote { DocTier::RustTierA } else { DocTier::RustTierBC };
                insert_prod_item(&mut inv, u32::from(node.text_range().start()), ClassifiedDocTierItem { symbol, tier, byte_start: s, byte_end: e, doc_line_count: doc_count, is_callable })?;
            }
        }
        if !eff_test { record_exec_lines(&node, cst.source_text(), cst.line_starts(), &mut inv.production_exec_lines)?; }
        push_tier_worklist_children(&node, depth, thresholds.max_ast_depth, eff_test, &mut stack)?;
    }
    inv.production_exec_sloc = inv.production_exec_lines.len();
    Ok(inv)
}

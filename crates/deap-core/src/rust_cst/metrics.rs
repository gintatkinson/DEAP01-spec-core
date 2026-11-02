#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Per-callable CST metrics extraction and v2_metrics report synthesis. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Implements receiver-excluded arity, mut bool, predicate clauses, and SLOC extraction (TD-07 / US-56).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Executable SLOC counts only non-trivia, non-doc tokens via monotone line table.
//! - BTreeMap backing ensures deterministic ordering keyed by (byte_start_u32, symbol).
//!
//! ## 3. Preconditions
//! - Callers provide sealed VerifiedLosslessCst and validated CstMetricsThresholds.
//!
//! ## 4. Postconditions
//! - Returns DeapV2AstMetricsReport with 100% of test subtrees pruned from production budgets.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N log L + F log F) over N syntax nodes, L lines, and F callables; bounded heap space.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fails closed with CstMetricsError (DEAP-E0801..DEAP-E0804) on invariant drift.
//!
//! ## 7. Realization & Traceability
//! Realises: [LosslessRustCstAnalyzer/analyze_cst_v2_metrics], [REQ-0608]
//! SysML v2: part def LosslessRustCstAnalyzer

use super::doc_tier::{
    classify_cfg_test_excluded_doc_tier, is_inside_doc_attr, is_trivia_token,
    node_enters_test_scope, push_tier_worklist_children, resolve_byte_to_line, DocTierInventory,
};
use super::envelope::MAX_AST_DEPTH;
use super::thresholds::{CstMetricsError, CstMetricsThresholds};
use super::verified_cst::{
    checked_next_cst_depth, depth_exhaustion_err, verify_utf8_char_boundary_span,
    VerifiedLosslessCst,
};
use ra_ap_syntax::AstNode;
use std::collections::{BTreeMap, BTreeSet};

/// Structural metrics extracted for a single callable declaration.
/// Realises: [LosslessRustCstAnalyzer/FnAstMetrics], [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct FnAstMetrics {
    pub symbol: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub exec_lines: BTreeSet<usize>,
    pub exec_sloc: usize,
    pub param_count: usize,
    pub has_self_receiver: bool,
    pub mut_bool_flag_count: usize,
    pub max_predicate_clauses: usize,
    pub is_rt_loop: bool,
    pub returns_result_string_err: bool,
}

/// Consolidated v2_metrics and DocTier report.
/// Realises: [LosslessRustCstAnalyzer/DeapV2AstMetricsReport], [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeapV2AstMetricsReport {
    pub source_path: String,
    pub callable_metrics: BTreeMap<(u32, String), FnAstMetrics>,
    pub doc_tier_inventory: DocTierInventory,
    pub production_exec_lines: BTreeSet<usize>,
    pub production_exec_sloc: usize,
    pub rust_tier_a_count: usize,
    pub rust_tier_bc_count: usize,
    pub test_excluded_item_count: usize,
    pub observed_max_depth: usize,
    pub lossless_roundtrip_verified: bool,
}

/// Accumulator for callable metrics during AST traversal. Realises: [REQ-0608]
struct CallableBodyAccumulator {
    exec_lines: BTreeSet<usize>,
    mut_bool_flag_count: usize,
    max_predicate_clauses: usize,
}

/// Computes root CST depth of a function. Realises: [REQ-0608]
fn compute_fn_root_cst_depth(fn_node: &ra_ap_syntax::ast::Fn) -> Result<usize, CstMetricsError> {
    let start = u32::from(fn_node.syntax().text_range().start()) as usize;
    let mut depth = 0usize;
    let mut cursor = fn_node.syntax().parent();
    while let Some(parent_node) = cursor {
        depth = checked_next_cst_depth(depth, MAX_AST_DEPTH, start)?;
        cursor = parent_node.parent();
    }
    Ok(depth)
}

/// Extracts parameter arity excluding receiver. Realises: [REQ-0608]
fn extract_receiver_excluded_arity(fn_node: &ra_ap_syntax::ast::Fn) -> Result<(usize, bool), CstMetricsError> {
    let Some(param_list) = fn_node.param_list() else { return Ok((0, false)); };
    let has_self = param_list.self_param().is_some();
    let start = u32::from(param_list.syntax().text_range().start()) as usize;
    let mut count = 0usize;
    for _ in param_list.params() {
        count = count.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
    }
    Ok((count, has_self))
}

/// Checks for outer rt_loop attribute. Realises: [REQ-0608]
fn has_outer_rt_loop_attr(fn_node: &ra_ap_syntax::ast::Fn) -> bool {
    ra_ap_syntax::ast::HasAttrs::attrs(fn_node).any(|attr| {
        attr.kind() == ra_ap_syntax::ast::AttrKind::Outer
            && attr.meta().and_then(|m| m.path()).and_then(|p| p.segment())
                .and_then(|s| s.name_ref()).is_some_and(|n| n.text() == "rt_loop")
    })
}

/// Unwraps paren AST type wrappers. Realises: [REQ-0608]
fn unwrap_paren_ast_type(mut ty: ra_ap_syntax::ast::Type) -> ra_ap_syntax::ast::Type {
    let mut steps = 0usize;
    while let ra_ap_syntax::ast::Type::ParenType(paren) = ty {
        let Some(inner) = paren.ty() else { return ra_ap_syntax::ast::Type::ParenType(paren); };
        ty = inner;
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH { break; }
    }
    ty
}

/// Checks whether AST type represents String. Realises: [REQ-0608]
fn is_rust_string_ast_type(ty: ra_ap_syntax::ast::Type) -> bool {
    use ra_ap_syntax::ast::HasGenericArgs;
    let ra_ap_syntax::ast::Type::PathType(pt) = unwrap_paren_ast_type(ty) else { return false; };
    let Some(p) = pt.path() else { return false; };
    let Some(seg) = p.segment() else { return false; };
    if seg.generic_arg_list().is_some() { return false; }
    let is_str = seg.name_ref().is_some_and(|n| n.text() == "String");
    let valid = p.segments().all(|s| s.generic_arg_list().is_none() && s.name_ref().is_some_and(|n| matches!(n.text(), "String" | "string" | "std" | "alloc")));
    is_str && valid
}

/// Checks whether function returns Result<T, String>. Realises: [REQ-0608]
fn fn_returns_result_string_err(fn_node: &ra_ap_syntax::ast::Fn) -> bool {
    use ra_ap_syntax::ast::HasGenericArgs;
    let Some(ret_ty) = fn_node.ret_type().and_then(|rt| rt.ty()) else { return false; };
    let ra_ap_syntax::ast::Type::PathType(pt) = unwrap_paren_ast_type(ret_ty) else { return false; };
    let Some(seg) = pt.path().and_then(|p| p.segment()) else { return false; };
    if !seg.name_ref().is_some_and(|n| n.text() == "Result") { return false; }
    let Some(args) = seg.generic_arg_list() else { return false; };
    let targs: Vec<_> = args.generic_args().filter_map(|a| match a { ra_ap_syntax::ast::GenericArg::TypeArg(ta) => ta.ty(), _ => None }).collect();
    targs.len() == 2 && targs.get(1).cloned().is_some_and(is_rust_string_ast_type)
}

/// Checks whether AST type represents bool. Realises: [REQ-0608]
fn is_bool_ast_type(ty: ra_ap_syntax::ast::Type) -> bool {
    use ra_ap_syntax::ast::HasGenericArgs;
    let ra_ap_syntax::ast::Type::PathType(pt) = unwrap_paren_ast_type(ty) else { return false; };
    let Some(p) = pt.path() else { return false; };
    let Some(seg) = p.segment() else { return false; };
    let is_b = seg.generic_arg_list().is_none() && seg.name_ref().is_some_and(|n| n.text() == "bool");
    let valid = p.segments().all(|s| s.generic_arg_list().is_none() && s.name_ref().is_some_and(|n| matches!(n.text(), "bool" | "primitive" | "core" | "std")));
    is_b && valid
}

/// Checks whether expression is boolean literal. Realises: [REQ-0608]
fn is_bool_literal_expr(mut expr: ra_ap_syntax::ast::Expr) -> bool {
    let mut steps = 0usize;
    loop {
        match expr {
            ra_ap_syntax::ast::Expr::ParenExpr(p) => { let Some(i) = p.expr() else { return false; }; expr = i; }
            ra_ap_syntax::ast::Expr::PrefixExpr(p) if p.op_kind() == Some(ra_ap_syntax::ast::UnaryOp::Not) => {
                let Some(i) = p.expr() else { return false; }; expr = i;
            }
            ra_ap_syntax::ast::Expr::Literal(lit) => return matches!(lit.token().kind(), ra_ap_syntax::SyntaxKind::TRUE_KW | ra_ap_syntax::SyntaxKind::FALSE_KW),
            _ => return false,
        }
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH { return false; }
    }
}

/// Counts mutable boolean flags in let statement. Realises: [REQ-0608]
fn count_let_stmt_mut_bool_flags(let_stmt: &ra_ap_syntax::ast::LetStmt) -> usize {
    let Some(pat) = let_stmt.pat() else { return 0; };
    match pat {
        ra_ap_syntax::ast::Pat::IdentPat(id) if id.mut_token().is_some() => {
            let ty_b = let_stmt.ty().is_some_and(is_bool_ast_type);
            let init_b = let_stmt.initializer().is_some_and(is_bool_literal_expr);
            usize::from(ty_b || init_b)
        }
        _ => 0,
    }
}

/// Checks whether binary expression is logical AND/OR. Realises: [REQ-0608]
fn is_logical_and_or_bin_expr(bin: &ra_ap_syntax::ast::BinExpr) -> bool {
    matches!(bin.op_kind(), Some(ra_ap_syntax::ast::BinaryOp::LogicOp(_)))
}

/// Checks whether binary expression is child of logical binary expr. Realises: [REQ-0608]
fn is_child_of_logical_bin_expr(bin: &ra_ap_syntax::ast::BinExpr) -> bool {
    let mut cursor = bin.syntax().parent();
    let mut steps = 0usize;
    while let Some(parent) = cursor {
        if ra_ap_syntax::ast::ParenExpr::can_cast(parent.kind()) {
            cursor = parent.parent();
        } else if let Some(pref) = ra_ap_syntax::ast::PrefixExpr::cast(parent.clone()) {
            if pref.op_kind() == Some(ra_ap_syntax::ast::UnaryOp::Not) { cursor = parent.parent(); } else { return false; }
        } else if let Some(p_bin) = ra_ap_syntax::ast::BinExpr::cast(parent) {
            return is_logical_and_or_bin_expr(&p_bin);
        } else { return false; }
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH { return false; }
    }
    false
}

/// Unwraps clause expression wrappers. Realises: [REQ-0608]
fn unwrap_clause_wrappers(mut expr: ra_ap_syntax::ast::Expr) -> ra_ap_syntax::ast::Expr {
    let mut steps = 0usize;
    loop {
        match expr {
            ra_ap_syntax::ast::Expr::ParenExpr(p) => { let Some(i) = p.expr() else { return ra_ap_syntax::ast::Expr::ParenExpr(p); }; expr = i; }
            ra_ap_syntax::ast::Expr::PrefixExpr(p) if p.op_kind() == Some(ra_ap_syntax::ast::UnaryOp::Not) => {
                let Some(i) = p.expr() else { return ra_ap_syntax::ast::Expr::PrefixExpr(p); }; expr = i;
            }
            other => return other,
        }
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH { return expr; }
    }
}

/// Counts predicate clauses in logical binary expr. Realises: [REQ-0608]
fn count_predicate_clauses(bin: &ra_ap_syntax::ast::BinExpr) -> Result<usize, CstMetricsError> {
    if !is_logical_and_or_bin_expr(bin) || is_child_of_logical_bin_expr(bin) { return Ok(0); }
    let start = u32::from(bin.syntax().text_range().start()) as usize;
    let mut stack = Vec::new();
    if let Some(r) = bin.rhs() { stack.push(r); }
    if let Some(l) = bin.lhs() { stack.push(l); }
    let mut clauses = 0usize;
    while let Some(raw) = stack.pop() {
        match unwrap_clause_wrappers(raw) {
            ra_ap_syntax::ast::Expr::BinExpr(ib) if is_logical_and_or_bin_expr(&ib) => {
                if let Some(r) = ib.rhs() { stack.push(r); }
                if let Some(l) = ib.lhs() { stack.push(l); }
            }
            _ => clauses = clauses.checked_add(1).ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?,
        }
    }
    Ok(clauses)
}

/// Records executable lines of callable node. Realises: [REQ-0608]
fn record_callable_lines(node: &ra_ap_syntax::SyntaxNode, line_starts: &[usize], lines: &mut BTreeSet<usize>) -> Result<(), CstMetricsError> {
    if is_inside_doc_attr(node) { return Ok(()); }
    for child in node.children_with_tokens() {
        let ra_ap_syntax::SyntaxElement::Token(tok) = child else { continue; };
        if is_trivia_token(&tok) { continue; }
        let (s, e) = (u32::from(tok.text_range().start()) as usize, u32::from(tok.text_range().end()) as usize);
        if s > e { return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: s, byte_end: e }); }
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

/// Accumulates callable metrics from AST node. Realises: [REQ-0608]
fn accumulate_callable_metrics(node: &ra_ap_syntax::SyntaxNode, is_root: bool, line_starts: &[usize], acc: &mut CallableBodyAccumulator) -> Result<bool, CstMetricsError> {
    if !is_root && (node_enters_test_scope(node) || ra_ap_syntax::ast::Fn::can_cast(node.kind())) { return Ok(false); }
    record_callable_lines(node, line_starts, &mut acc.exec_lines)?;
    let start = u32::from(node.text_range().start()) as usize;
    if let Some(let_st) = ra_ap_syntax::ast::LetStmt::cast(node.clone()) {
        acc.mut_bool_flag_count = acc.mut_bool_flag_count.checked_add(count_let_stmt_mut_bool_flags(&let_st))
            .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
    }
    if let Some(bin) = ra_ap_syntax::ast::BinExpr::cast(node.clone()) {
        let c = count_predicate_clauses(&bin)?;
        if c > acc.max_predicate_clauses { acc.max_predicate_clauses = c; }
    }
    Ok(true)
}

/// Extracts FnAstMetrics from a single ra_ap_syntax::ast::Fn CST node.
/// Realises: [LosslessRustCstAnalyzer/extract_callable_v2_metrics], [REQ-0608]
pub fn extract_callable_v2_metrics(
    fn_node: &ra_ap_syntax::ast::Fn,
    line_starts: &[usize],
) -> Result<FnAstMetrics, CstMetricsError> {
    let range = fn_node.syntax().text_range();
    let (byte_start, byte_end) = (u32::from(range.start()) as usize, u32::from(range.end()) as usize);
    let symbol = ra_ap_syntax::ast::HasName::name(fn_node).map(|n| n.text().to_string()).ok_or_else(|| {
        CstMetricsError::MalformedUtf8OrRustSyntax { byte_start, byte_end, detail: "callable ast::Fn missing identifier".to_string() }
    })?;
    let root_depth = compute_fn_root_cst_depth(fn_node)?;
    let (param_count, has_self_receiver) = extract_receiver_excluded_arity(fn_node)?;
    let is_rt_loop = has_outer_rt_loop_attr(fn_node);
    let returns_result_string_err = fn_returns_result_string_err(fn_node);
    let mut acc = CallableBodyAccumulator { exec_lines: BTreeSet::new(), mut_bool_flag_count: 0, max_predicate_clauses: 0 };
    let mut stack = vec![(fn_node.syntax().clone(), root_depth, true)];
    while let Some((node, depth, is_root)) = stack.pop() {
        let start = u32::from(node.text_range().start()) as usize;
        if depth > MAX_AST_DEPTH { return Err(depth_exhaustion_err(depth, MAX_AST_DEPTH, start)); }
        if accumulate_callable_metrics(&node, is_root, line_starts, &mut acc)? {
            push_tier_worklist_children(&node, depth, MAX_AST_DEPTH, false, &mut stack)?;
        }
    }
    let exec_sloc = acc.exec_lines.len();
    Ok(FnAstMetrics {
        symbol, byte_start, byte_end, exec_lines: acc.exec_lines, exec_sloc, param_count,
        has_self_receiver, mut_bool_flag_count: acc.mut_bool_flag_count,
        max_predicate_clauses: acc.max_predicate_clauses, is_rt_loop, returns_result_string_err,
    })
}

/// Prunes #[cfg(test)]/#[test] subtrees and extracts BTreeMap-ordered v2_metrics.
/// Realises: [LosslessRustCstAnalyzer/analyze_cst_v2_metrics], [REQ-0608]
pub fn analyze_cst_v2_metrics(
    cst: &VerifiedLosslessCst,
    thresholds: &CstMetricsThresholds,
) -> Result<DeapV2AstMetricsReport, CstMetricsError> {
    let doc_tier_inventory = classify_cfg_test_excluded_doc_tier(cst, thresholds, false)?;
    let mut callable_metrics: BTreeMap<(u32, String), FnAstMetrics> = BTreeMap::new();
    let mut stack = vec![(cst.syntax_node().clone(), 0usize, false)];
    let mut observed_max_depth = 0usize;
    while let Some((node, depth, in_test)) = stack.pop() {
        let range = node.text_range();
        let start_u32 = u32::from(range.start());
        let (start, end) = (start_u32 as usize, u32::from(range.end()) as usize);
        if depth > thresholds.max_ast_depth { return Err(depth_exhaustion_err(depth, thresholds.max_ast_depth, start)); }
        verify_utf8_char_boundary_span(cst.source_text(), start, end)?;
        observed_max_depth = observed_max_depth.max(depth);
        if !(in_test || node_enters_test_scope(&node)) {
            if let Some(fn_node) = ra_ap_syntax::ast::Fn::cast(node.clone()) {
                let metrics = extract_callable_v2_metrics(&fn_node, cst.line_starts())?;
                let key = (start_u32, metrics.symbol.clone());
                if callable_metrics.insert(key, metrics).is_some() {
                    return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch { byte_start: start, byte_end: end });
                }
            }
            push_tier_worklist_children(&node, depth, thresholds.max_ast_depth, false, &mut stack)?;
        }
    }
    let production_exec_lines = doc_tier_inventory.production_exec_lines.clone();
    let production_exec_sloc = doc_tier_inventory.production_exec_sloc;
    let rust_tier_a_count = doc_tier_inventory.rust_tier_a_count;
    let rust_tier_bc_count = doc_tier_inventory.rust_tier_bc_count;
    let test_excluded_item_count = doc_tier_inventory.test_excluded_item_count;
    Ok(DeapV2AstMetricsReport {
        source_path: cst.source_path().to_string(),
        callable_metrics,
        doc_tier_inventory,
        production_exec_lines,
        production_exec_sloc,
        rust_tier_a_count,
        rust_tier_bc_count,
        test_excluded_item_count,
        observed_max_depth: cst.observed_max_depth().max(observed_max_depth),
        lossless_roundtrip_verified: true,
    })
}

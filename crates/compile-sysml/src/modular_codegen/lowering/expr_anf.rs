//! A-Normal Form boolean constraint hoisting and Rust cyclomatic complexity estimation.
//!
//! Safety & Traceability: Realises: [REQ-0607]
//! Contract: Lowers compound `and`/`or`/`not` SysML constraint expressions into bounded `HoistedPredicate` chains obeying `v(G) <= max_vg` and estimates per-function McCabe `v(G)` without string or comment inflation.
//! Bounds & Faults: Operates strictly over `BTreeSet` and `Vec` structures with zero recursion, checked `.get()` slice indexing, saturating arithmetic, and fallback `"true"` lowering on blank clauses.

use super::outliner::{map_sysml_type_to_rust, to_snake_slug};
use crate::modular_codegen::domain::HoistedPredicate;
use std::collections::BTreeSet;

/// Masks string literal interiors with spaces and strips unquoted `//` trailing comments before `v(G)` scanning.
/// Realises: [REQ-0607]
fn mask_line_for_vg(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let (mut in_str, mut escaped, mut prev_slash) = (false, false, false);
    for ch in line.chars() {
        match (in_str, escaped, prev_slash, ch) {
            (true, true, _, _) => {
                escaped = false;
                out.push(' ');
            }
            (true, false, _, '\x22') => {
                in_str = false;
                out.push('\x22');
            }
            (true, false, _, _) => {
                escaped = ch == '\\';
                out.push(' ');
            }
            (false, _, true, '/') => {
                out.pop();
                break;
            }
            (false, _, _, _) => {
                prev_slash = ch == '/';
                in_str = ch == '\x22';
                out.push(ch);
            }
        }
    }
    out
}

/// Counts exact identifier-bounded occurrences of a Rust control-flow keyword inside masked source text.
/// Realises: [REQ-0607]
fn count_bounded_keyword(masked: &str, kw: &str) -> usize {
    masked
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|tok| *tok == kw)
        .count()
}

/// Estimates McCabe cyclomatic complexity `v(G)` for a single Rust function source slice.
///
/// Safety & Traceability: Realises: [REQ-0607]
/// Contract: Returns `1 + kw + logic + arms` over masked control keywords (`if`, `while`, `for`, `loop`), `&&`/`||`, and extra `match` arms.
/// Bounds & Faults: Linear `O(|fn_source|)` single-pass scan using saturating unsigned arithmetic with zero panics.
pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize {
    let masked = fn_source
        .lines()
        .map(mask_line_for_vg)
        .collect::<Vec<_>>()
        .join("\n");
    let kw = count_bounded_keyword(&masked, "if")
        .saturating_add(count_bounded_keyword(&masked, "while"))
        .saturating_add(count_bounded_keyword(&masked, "for"))
        .saturating_add(count_bounded_keyword(&masked, "loop"));
    let logic = masked
        .matches("&&")
        .count()
        .saturating_add(masked.matches("||").count());
    let arms = masked
        .matches("=>")
        .count()
        .saturating_sub(count_bounded_keyword(&masked, "match"));
    1usize
        .saturating_add(kw)
        .saturating_add(logic)
        .saturating_add(arms)
}

/// Splits a whitespace-delimited SysML boolean expression on top-level (`depth == 0`) occurrences of `keyword`.
/// Realises: [REQ-0607]
fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String> {
    let (mut parts, mut current, mut depth) = (Vec::new(), Vec::new(), 0usize);
    for tok in expr.split_whitespace() {
        depth = depth
            .saturating_add(tok.matches('(').count())
            .saturating_sub(tok.matches(')').count());
        if depth == 0 && tok == keyword {
            if !current.is_empty() {
                parts.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(tok);
        }
    }
    if !current.is_empty() {
        parts.push(current.join(" "));
    }
    parts
}

/// Translates an atomic SysML boolean clause into a `ctx.`-qualified Rust boolean expression.
/// Realises: [REQ-0607]
fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String {
    let mut out = String::with_capacity(clause.len().saturating_add(8));
    let mut word = String::new();
    for ch in clause.trim().chars().chain(std::iter::once(' ')) {
        if matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
            word.push(ch);
        } else {
            match (word.as_str(), field_names.contains(&word)) {
                ("", _) => {}
                ("not", _) => out.push('!'),
                (w, true) => {
                    out.push_str("ctx.");
                    out.push_str(w);
                }
                (w, false) => out.push_str(w),
            }
            word.clear();
            out.push(ch);
        }
    }
    let trimmed = out.trim();
    if !trimmed.is_empty() {
        trimmed.to_string()
    } else {
        String::from("true")
    }
}

/// Scoped identifier and parameter binding context shared across ANF batch emission passes.
/// Realises: [REQ-0607]
struct AnfScope<'a> {
    slug: &'a str,
    params: &'a [(String, String)],
}

/// Prefixes unreferenced parameter names in `params` with `_` so generated Rust helpers compile warning-free.
/// Realises: [REQ-0607]
fn bind_scope_params(rust_expr: &str, params: &[(String, String)]) -> Vec<(String, String)> {
    params
        .iter()
        .map(|(name, ty)| {
            let bound = if count_bounded_keyword(rust_expr, name) > 0 {
                name.clone()
            } else {
                format!("_{name}")
            };
            (bound, ty.clone())
        })
        .collect()
}

/// Folds adjacent `lhs >= lo` and `lhs <= hi` conjuncts into idiomatic `(lo..=hi).contains(&lhs)` range checks.
/// Realises: [REQ-0607]
fn fold_inclusive_ranges(clauses: Vec<String>) -> Vec<(String, usize)> {
    let (mut out, mut idx) = (Vec::with_capacity(clauses.len()), 0usize);
    while let Some(cur) = clauses.get(idx) {
        let folded = cur
            .split_once(">=")
            .zip(
                clauses
                    .get(idx.saturating_add(1))
                    .and_then(|n| n.split_once("<=")),
            )
            .and_then(|((la, lo), (lb, hi))| {
                let (la, lb, lo, hi) = (la.trim(), lb.trim(), lo.trim(), hi.trim());
                (!la.is_empty() && la == lb && lo.contains('.') == hi.contains('.'))
                    .then_some(format!("({lo}..={hi}).contains(&{la})"))
            });
        if let Some(range_expr) = folded {
            out.push((range_expr, 0));
            idx = idx.saturating_add(2);
        } else {
            out.push((cur.clone(), 0));
            idx = idx.saturating_add(1);
        }
    }
    out
}

/// Outlines a multi-term boolean batch into an indexed `HoistedPredicate` helper and returns its call expression.
/// Realises: [REQ-0607]
fn emit_hoisted_batch(
    scope: &AnfScope<'_>,
    batch: &[String],
    join_op: &str,
    batch_ops: usize,
    seq: &mut usize,
    out: &mut Vec<HoistedPredicate>,
) -> (String, usize) {
    if batch.len() <= 1 {
        return (
            batch.first().cloned().unwrap_or(String::from("true")),
            batch_ops,
        );
    }
    *seq = seq.saturating_add(1);
    let fn_name = format!("eval_{}_term_{:02}", scope.slug, *seq);
    let rust_expr = batch.join(join_op);
    let params = bind_scope_params(&rust_expr, scope.params);
    let call_args = scope
        .params
        .iter()
        .map(|(n, _)| n.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    out.push(HoistedPredicate {
        fn_name: fn_name.clone(),
        params,
        rust_expr,
        cyclomatic_complexity: 1usize.saturating_add(batch_ops),
    });
    (format!("{fn_name}({call_args})"), 0)
}

/// Iteratively packs weighted boolean terms into `HoistedPredicate` batches bounded by `max_ops` operators.
/// Realises: [REQ-0607]
fn reduce_weighted_terms(
    scope: &AnfScope<'_>,
    mut terms: Vec<(String, usize)>,
    join_op: &str,
    max_ops: usize,
    seq: &mut usize,
    out: &mut Vec<HoistedPredicate>,
) -> (String, usize) {
    while terms.len() > 1 {
        let prev_len = terms.len();
        let sum_ops: usize = terms.iter().map(|(_, ops)| *ops).sum();
        let total_ops = sum_ops.saturating_add(prev_len.saturating_sub(1));
        if total_ops <= max_ops {
            return (
                terms
                    .into_iter()
                    .map(|(e, _)| e)
                    .collect::<Vec<_>>()
                    .join(join_op),
                total_ops,
            );
        }
        let (mut batch, mut batch_ops, mut next_round) = (Vec::new(), 0usize, Vec::new());
        for (expr, ops) in terms {
            let link = usize::from(!batch.is_empty());
            if link.saturating_mul(batch_ops.saturating_add(link).saturating_add(ops)) > max_ops {
                next_round.push(emit_hoisted_batch(
                    scope, &batch, join_op, batch_ops, seq, out,
                ));
                batch.clear();
                batch_ops = 0;
            }
            batch.push(expr);
            batch_ops = batch_ops
                .saturating_add(usize::from(batch.len() > 1))
                .saturating_add(ops);
        }
        next_round.push(emit_hoisted_batch(
            scope, &batch, join_op, batch_ops, seq, out,
        ));
        if next_round.len() >= prev_len {
            let mut it = next_round.into_iter();
            let (lhs, lhs_ops) = it.next().unwrap_or_default();
            let (rhs, rhs_ops) = it.next().unwrap_or_default();
            let pair_ops = lhs_ops.saturating_add(rhs_ops).saturating_add(1);
            let paired = emit_hoisted_batch(scope, &[lhs, rhs], join_op, pair_ops, seq, out);
            terms = std::iter::once(paired).chain(it).collect();
        } else {
            terms = next_round;
        }
    }
    terms
        .into_iter()
        .next()
        .unwrap_or((String::from("true"), 0))
}

/// Lowers a parameterized SysML constraint expression into A-Normal Form `HoistedPredicate` helpers and a root validator.
///
/// Safety & Traceability: Realises: [REQ-0607]
/// Contract: Binds `PartDef` attributes in `field_names` to `ctx.<field>`, preserves external parameters in `raw_params` as typed Rust arguments, and emits `eval_<slug>_term_NN` helpers terminated by `validate_<slug>`.
/// Bounds & Faults: Deduplicates colliding parameter slugs via `BTreeSet`, lowers blank expressions to `"true"`, and converges in `O(N)` work across iterative reduction passes without panicking.
pub fn hoist_constraint_anf_with_params(
    constraint_slug: &str,
    ctx_type_name: &str,
    field_names: &BTreeSet<String>,
    raw_params: &[String],
    raw_expression: &str,
    max_vg: usize,
) -> Vec<HoistedPredicate> {
    let mut seen = BTreeSet::from([String::from("ctx")]);
    let extras = raw_params.iter().filter_map(|p| {
        let (lhs, rhs) = p.split_once(':').unwrap_or((p.as_str(), "Integer"));
        let slug = to_snake_slug(lhs.split_whitespace().next_back().unwrap_or(""));
        let keep = !slug.is_empty() && !field_names.contains(&slug) && seen.insert(slug.clone());
        let mapped = map_sysml_type_to_rust(rhs);
        let rty = if mapped == "String" { "&str" } else { mapped };
        keep.then_some((slug, rty.to_string()))
    });
    let all_params: Vec<(String, String)> =
        std::iter::once((String::from("ctx"), format!("&{ctx_type_name}")))
            .chain(extras)
            .collect();
    let scope = AnfScope {
        slug: constraint_slug,
        params: &all_params,
    };
    let max_ops = max_vg.saturating_sub(2).max(1);
    let cleaned = raw_expression.trim().trim_end_matches(';').trim();
    let (mut out, mut seq, mut disjunct_terms) = (Vec::new(), 0usize, Vec::new());
    for disj in split_top_level_keyword(cleaned, "or") {
        let conjuncts = fold_inclusive_ranges(
            split_top_level_keyword(&disj, "and")
                .into_iter()
                .map(|c| translate_atomic_clause(&c, field_names))
                .collect(),
        );
        disjunct_terms.push(reduce_weighted_terms(
            &scope, conjuncts, " && ", max_ops, &mut seq, &mut out,
        ));
    }
    let (root_expr, root_ops) =
        reduce_weighted_terms(&scope, disjunct_terms, " || ", max_ops, &mut seq, &mut out);
    let params = bind_scope_params(&root_expr, &all_params);
    out.push(HoistedPredicate {
        fn_name: format!("validate_{constraint_slug}"),
        params,
        rust_expr: root_expr,
        cyclomatic_complexity: 1usize.saturating_add(root_ops),
    });
    out
}

/// Lowers a raw parameterless SysML constraint expression into A-Normal Form `HoistedPredicate` helpers and a root validator.
///
/// Safety & Traceability: Realises: [REQ-0607]
/// Contract: Delegates to `hoist_constraint_anf_with_params` with an empty external parameter slice `&[]` for struct-field-only invariants.
/// Bounds & Faults: Total infallible delegation in linear-log time with zero panics.
pub fn hoist_constraint_anf(
    constraint_slug: &str,
    ctx_type_name: &str,
    field_names: &BTreeSet<String>,
    raw_expression: &str,
    max_vg: usize,
) -> Vec<HoistedPredicate> {
    hoist_constraint_anf_with_params(
        constraint_slug,
        ctx_type_name,
        field_names,
        &[],
        raw_expression,
        max_vg,
    )
}

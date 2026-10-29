//! A-Normal Form boolean constraint hoisting and Rust cyclomatic complexity estimation.
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic ANF predicate outlining (`expr_anf.rs`) so generated SysML v2 invariant validators obey DO-178C and ISO 26262 `v(G) <= 8` modularity ceilings.
//! ## 2. Invariants & Mathematical Properties
//! Callers supply normalized constraint slugs, target Rust context type names, ordered `BTreeSet<String>` field sets, and positive `max_vg` budgets.
//! ## 3. Preconditions
//! Lowers compound `and`/`or`/`not` SysML expressions into bounded `HoistedPredicate` chains and measures per-function McCabe `v(G)` without string or comment inflation.
//! ## 4. Postconditions
//! All lowering and complexity routines are total, infallible transformations that degrade empty or degenerate clauses safely to `"true"` with zero traps.
//! ## 5. Algorithmic Complexity Bounds
//! Operates strictly over `BTreeSet` and `Vec` structures with zero recursion, zero regular expressions, and linear-to-log-linear multi-pass batch reduction.
//! ## 6. Failure Modes & Fault Tolerance
//! Verified by unit and mutation tests covering comment/string masking, `v(G)` counting, nested parenthesis splitting, field qualification, and bounded ANF tree hoisting.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Supplies the expression-lowering and static complexity-estimation primitives for `compile-sysml` modular invariant facet generation.

use std::collections::BTreeSet;
use crate::modular_codegen::domain::HoistedPredicate;
use super::outliner::{map_sysml_type_to_rust, to_snake_slug};

/// Masks string literal interiors and strips trailing line comments prior to cyclomatic complexity scanning.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn mask_line_for_vg(line: &str) -> String` prevents false-positive `v(G)` inflation from keywords or operators embedded inside strings and `//` comments.
/// ## 2. Invariants & Mathematical Properties
/// `fn mask_line_for_vg(line: &str) -> String` accepts any single UTF-8 source line slice without newline delimiters.
/// ## 3. Preconditions
/// `fn mask_line_for_vg(line: &str) -> String` replaces string literal interior characters with ASCII spaces and truncates at the first unquoted `//` sequence.
/// ## 4. Postconditions
/// `fn mask_line_for_vg(line: &str) -> String` handles unclosed quotes, escaped backslashes, and division slashes totally without fallible indexing.
/// ## 5. Algorithmic Complexity Bounds
/// `fn mask_line_for_vg(line: &str) -> String` executes in `O(|line|)` single-pass time and `O(|line|)` output capacity at `v(G) = 6`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing `fn mask_line_for_vg(line: &str) -> String` on escaped quotes, division operators, quoted `"&&"`/`"//"` tokens, and trailing comments.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn mask_line_for_vg(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let (mut in_str, mut escaped, mut prev_slash) = (false, false, false);
    for ch in line.chars() {
        match (in_str, escaped, prev_slash, ch) {
            (true, true, _, _) => { escaped = false; out.push(' '); }
            (true, false, _, '\x22') => { in_str = false; out.push('\x22'); }
            (true, false, _, _) => { escaped = ch == '\\'; out.push(' '); }
            (false, _, true, '/') => { out.pop(); break; }
            (false, _, _, _) => { prev_slash = ch == '/'; in_str = ch == '\x22'; out.push(ch); }
        }
    }
    out
}

/// Counts exact identifier-bounded occurrences of a Rust control-flow keyword inside masked source text.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn count_bounded_keyword(masked: &str, kw: &str) -> usize` isolates true Rust branching keywords from composite identifiers such as `motif` or `loopback`.
/// ## 2. Invariants & Mathematical Properties
/// `fn count_bounded_keyword(masked: &str, kw: &str) -> usize` expects `masked` text already scrubbed by `mask_line_for_vg` and a non-empty ASCII keyword `kw`.
/// ## 3. Preconditions
/// `fn count_bounded_keyword(masked: &str, kw: &str) -> usize` splits on non-identifier boundaries and returns the exact count of tokens equal to `kw`.
/// ## 4. Postconditions
/// `fn count_bounded_keyword(masked: &str, kw: &str) -> usize` is a pure total counter returning `0` on empty inputs without intermediate heap collections.
/// ## 5. Algorithmic Complexity Bounds
/// `fn count_bounded_keyword(masked: &str, kw: &str) -> usize` runs in `O(|masked|)` streaming time and `O(1)` auxiliary space at `v(G) = 2`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing `fn count_bounded_keyword(masked: &str, kw: &str) -> usize` against standalone keywords alongside underscore-joined identifier substrings.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn count_bounded_keyword(masked: &str, kw: &str) -> usize {
    masked
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|tok| *tok == kw)
        .count()
}

/// Estimates McCabe cyclomatic complexity `v(G)` for a single Rust function source slice.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize` audits generated and hand-authored Rust functions against `E0704` complexity bounds.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize` borrows UTF-8 source text spanning one Rust function signature and body.
/// ## 3. Preconditions
/// `pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize` returns `1 + kw + logic + arms` over masked control keywords, `&&`/`||`, and extra `match` arms.
/// ## 4. Postconditions
/// `pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize` uses saturating unsigned arithmetic so degenerate inputs never underflow or overflow.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize` executes in `O(|fn_source|)` linear time with `v(G) = 1`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by scoring straight-line, branching, short-circuit boolean, `match`-arm, and comment/string decoy inputs through `pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
pub fn estimate_rust_fn_cyclomatic_complexity(fn_source: &str) -> usize {
    let masked = fn_source.lines().map(mask_line_for_vg).collect::<Vec<_>>().join("\n");
    let kw = count_bounded_keyword(&masked, "if")
        .saturating_add(count_bounded_keyword(&masked, "while"))
        .saturating_add(count_bounded_keyword(&masked, "for"))
        .saturating_add(count_bounded_keyword(&masked, "loop"));
    let logic = masked.matches("&&").count().saturating_add(masked.matches("||").count());
    let arms = masked.matches("=>").count().saturating_sub(count_bounded_keyword(&masked, "match"));
    1usize.saturating_add(kw).saturating_add(logic).saturating_add(arms)
}

/// Splits a whitespace-delimited SysML boolean expression on top-level occurrences of `keyword`.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String>` preserves parenthesized sub-expressions intact when decomposing `or` and `and` trees.
/// ## 2. Invariants & Mathematical Properties
/// `fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String>` accepts trimmed SysML expression text and a boolean separator token (`"or"` or `"and"`).
/// ## 3. Preconditions
/// `fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String>` splits only where parenthesis `depth == 0` and emits normalized space-joined non-empty clauses.
/// ## 4. Postconditions
/// `fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String>` uses `saturating_add`/`saturating_sub` on `depth` so unbalanced parentheses never underflow.
/// ## 5. Algorithmic Complexity Bounds
/// `fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String>` runs in single-pass `O(|expr|)` time without recursion at `v(G) = 6`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing `fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String>` across flat chains, nested parenthesized groups, and empty strings.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn split_top_level_keyword(expr: &str, keyword: &str) -> Vec<String> {
    let (mut parts, mut current, mut depth) = (Vec::new(), Vec::new(), 0usize);
    for tok in expr.split_whitespace() {
        depth = depth.saturating_add(tok.matches('(').count()).saturating_sub(tok.matches(')').count());
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
/// ## 1. Safety Intent & Regulatory Scope
/// `fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String` deterministically binds part attributes to `ctx.<field>` and lowers `not` to `!`.
/// ## 2. Invariants & Mathematical Properties
/// `fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String` receives one atomic clause slice and the lexicographic `BTreeSet<String>` of struct fields.
/// ## 3. Preconditions
/// `fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String` rewrites `"not"` to `"!"`, prefixes declared `field_names` with `"ctx."`, and defaults empty text to `"true"`.
/// ## 4. Postconditions
/// `fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String` preserves unknown literals and operators verbatim without fallible parsing.
/// ## 5. Algorithmic Complexity Bounds
/// `fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String` runs in `O(|clause| * log(|field_names| + 1))` deterministic time at `v(G) = 6`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing `fn translate_atomic_clause(clause: &str, field_names: &BTreeSet<String>) -> String` on `not` negation, member identifiers, numeric constants, and blank clauses.
/// ## 7. Realization & Traceability
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
                (w, true) => { out.push_str("ctx."); out.push_str(w); }
                (w, false) => out.push_str(w),
            }
            word.clear();
            out.push(ch);
        }
    }
    let trimmed = out.trim();
    if !trimmed.is_empty() { trimmed.to_string() } else { String::from("true") }
}

/// Scoped identifier and parameter binding context shared across ANF batch emission passes.
/// ## 1. Safety Intent & Regulatory Scope
/// `struct AnfScope` bounds helper parameter arity while propagating constraint parameter signatures across hoisted sub-predicates.
/// ## 2. Invariants & Mathematical Properties
/// `slug` holds a normalized `snake_case` constraint identifier and `params` lists `(name, rust_type)` pairs beginning with `ctx`.
/// ## 3. Preconditions
/// Supplies immutable naming and signature metadata to `emit_hoisted_batch` and `reduce_weighted_terms`.
/// ## 4. Postconditions
/// Pure borrowed view carrier with zero fallible operations or ownership traps.
/// ## 5. Algorithmic Complexity Bounds
/// Occupies two fat pointers (`O(1)` stack space) with zero heap allocations.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing ANF hoisting across both struct-field invariants and parameterized SysML constraints.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
struct AnfScope<'a> { slug: &'a str, params: &'a [(String, String)] }

/// Prefixes unreferenced parameter names in `params` with `_` so generated Rust helpers compile warning-free.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn bind_scope_params(rust_expr: &str, params: &[(String, String)]) -> Vec<(String, String)>` prevents `unused_variables` diagnostics across sharded ANF helpers.
/// ## 2. Invariants & Mathematical Properties
/// `rust_expr` holds a lowered Rust boolean expression and `params` lists canonical bare parameter names and types.
/// ## 3. Preconditions
/// Preserves `(name, ty)` verbatim when `count_bounded_keyword(rust_expr, name) > 0` and prefixes `_{name}` otherwise.
/// ## 4. Postconditions
/// Total pure transformation over UTF-8 slices with zero panics or unchecked indexing.
/// ## 5. Algorithmic Complexity Bounds
/// Executes in `O(|params| * |rust_expr|)` deterministic time at `v(G) = 2`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by hoisting multi-parameter constraints where individual ANF batches reference disjoint parameter subsets.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn bind_scope_params(rust_expr: &str, params: &[(String, String)]) -> Vec<(String, String)> {
    params.iter().map(|(name, ty)| {
        let bound = if count_bounded_keyword(rust_expr, name) > 0 { name.clone() } else { format!("_{name}") };
        (bound, ty.clone())
    }).collect()
}

/// Folds adjacent `lhs >= lo` and `lhs <= hi` conjuncts into idiomatic `(lo..=hi).contains(&lhs)` range checks.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn fold_inclusive_ranges(clauses: Vec<String>) -> Vec<(String, usize)>` satisfies `clippy::manual_range_contains` across generated Rust invariants while lowering `v(G)`.
/// ## 2. Invariants & Mathematical Properties
/// `clauses` holds ordered atomic Rust boolean expressions lowered from a single conjunctive `and` chain.
/// ## 3. Preconditions
/// Collapses matching `>=` / `<=` pairs sharing identical LHS and numeric kind into `(lo..=hi).contains(&lhs)` with `0` short-circuit ops.
/// ## 4. Postconditions
/// Uses checked `.get(...)` indexing exclusively with zero panics, preserving non-matching clauses verbatim.
/// ## 5. Algorithmic Complexity Bounds
/// Single linear `O(|clauses|)` pass over owned clause strings at `v(G) = 5`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by compiling generated `LayerHierarchySpec`, `ModularityBoundsSpec`, and `CompilationUnitSpec` invariants under `-D warnings`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn fold_inclusive_ranges(clauses: Vec<String>) -> Vec<(String, usize)> {
    let (mut out, mut idx) = (Vec::with_capacity(clauses.len()), 0usize);
    while let Some(cur) = clauses.get(idx) {
        let folded = cur.split_once(">=").zip(clauses.get(idx.saturating_add(1)).and_then(|n| n.split_once("<=")))
            .and_then(|((la, lo), (lb, hi))| {
                let (la, lb, lo, hi) = (la.trim(), lb.trim(), lo.trim(), hi.trim());
                (!la.is_empty() && la == lb && lo.contains('.') == hi.contains('.'))
                    .then_some(format!("({lo}..={hi}).contains(&{la})"))
            });
        if let Some(range_expr) = folded { out.push((range_expr, 0)); idx = idx.saturating_add(2); }
        else { out.push((cur.clone(), 0)); idx = idx.saturating_add(1); }
    }
    out
}

/// Outlines a multi-term boolean batch into an indexed `HoistedPredicate` helper and returns its call expression.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn emit_hoisted_batch` caps sub-expression `v(G)` via named intermediate helpers with deterministic parameter forwarding.
/// ## 2. Invariants & Mathematical Properties
/// Borrows `AnfScope`, lowered Rust sub-terms `batch`, boolean join token `join_op`, and monotonic sequence state `seq`.
/// ## 3. Preconditions
/// Returns singletons inline or appends `eval_<slug>_term_NN` forwarding all `scope.params` with `0` residual ops.
/// ## 4. Postconditions
/// Falls back to `"true"` on empty slices and prefixes unused parameter bindings via `bind_scope_params`.
/// ## 5. Algorithmic Complexity Bounds
/// Runs in `O(|batch| * |scope.params|)` deterministic time at `v(G) = 2`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified on empty, singleton, constant, `ctx`-referencing, and external-parameter ANF batches.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn emit_hoisted_batch(
    scope: &AnfScope<'_>, batch: &[String],
    join_op: &str, batch_ops: usize, seq: &mut usize, out: &mut Vec<HoistedPredicate>,
) -> (String, usize) {
    if batch.len() <= 1 {
        return (batch.first().cloned().unwrap_or(String::from("true")), batch_ops);
    }
    *seq = seq.saturating_add(1);
    let fn_name = format!("eval_{}_term_{:02}", scope.slug, *seq);
    let rust_expr = batch.join(join_op);
    let params = bind_scope_params(&rust_expr, scope.params);
    let call_args = scope.params.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", ");
    out.push(HoistedPredicate {
        fn_name: fn_name.clone(),
        params,
        rust_expr,
        cyclomatic_complexity: 1usize.saturating_add(batch_ops),
    });
    (format!("{fn_name}({call_args})"), 0)
}

/// Iteratively packs weighted boolean terms into `HoistedPredicate` batches bounded by `max_ops` operators.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn reduce_weighted_terms` enforces ANF tree height and width reduction without recursion under DO-178C Level A bounds.
/// ## 2. Invariants & Mathematical Properties
/// Accepts `AnfScope`, `(rust_expr, internal_ops)` pairs in `terms`, boolean `join_op`, and positive operator budget `max_ops`.
/// ## 3. Preconditions
/// Emits intermediate helpers until the root expression fits within `max_ops` short-circuit operators.
/// ## 4. Postconditions
/// Guarantees strict termination via force-pairing when `max_ops == 1` stalls greedy packing.
/// ## 5. Algorithmic Complexity Bounds
/// Converges in `O(N)` total work across logarithmic reduction rounds at `v(G) = 6`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified across multi-round fan-in boolean trees and `max_ops == 1` boundary inputs.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
fn reduce_weighted_terms(
    scope: &AnfScope<'_>, mut terms: Vec<(String, usize)>,
    join_op: &str, max_ops: usize, seq: &mut usize, out: &mut Vec<HoistedPredicate>,
) -> (String, usize) {
    while terms.len() > 1 {
        let prev_len = terms.len();
        let sum_ops: usize = terms.iter().map(|(_, ops)| *ops).sum();
        let total_ops = sum_ops.saturating_add(prev_len.saturating_sub(1));
        if total_ops <= max_ops {
            return (terms.into_iter().map(|(e, _)| e).collect::<Vec<_>>().join(join_op), total_ops);
        }
        let (mut batch, mut batch_ops, mut next_round) = (Vec::new(), 0usize, Vec::new());
        for (expr, ops) in terms {
            let link = usize::from(!batch.is_empty());
            if link.saturating_mul(batch_ops.saturating_add(link).saturating_add(ops)) > max_ops {
                next_round.push(emit_hoisted_batch(scope, &batch, join_op, batch_ops, seq, out));
                batch.clear();
                batch_ops = 0;
            }
            batch.push(expr);
            batch_ops = batch_ops.saturating_add(usize::from(batch.len() > 1)).saturating_add(ops);
        }
        next_round.push(emit_hoisted_batch(scope, &batch, join_op, batch_ops, seq, out));
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
    terms.into_iter().next().unwrap_or((String::from("true"), 0))
}

/// Lowers a parameterized SysML constraint expression into A-Normal Form `HoistedPredicate` helpers and a root validator.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn hoist_constraint_anf_with_params` binds `PartDef` attributes to `ctx.<field>` while preserving non-attribute constraint parameters as typed Rust arguments.
/// ## 2. Invariants & Mathematical Properties
/// Receives `constraint_slug`, `ctx_type_name`, `field_names` (`part.attributes` only), `raw_params` (`ConstraintDef.parameters`), `raw_expression`, and `max_vg`.
/// ## 3. Preconditions
/// Emits zero or more `eval_<slug>_term_NN` helpers terminated by `validate_<slug>` with `Copy`-safe parameter types (`i64`, `f64`, `bool`, `&str`).
/// ## 4. Postconditions
/// Deduplicates colliding parameter slugs via `BTreeSet`, defaults untyped parameters to `i64`, and lowers blank expressions to `"true"`.
/// ## 5. Algorithmic Complexity Bounds
/// Produces bit-identical predicate orderings across runs at `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by compiling Stage-1 `LayerHierarchySpec::NonUpwardLayerImport` and `CompilationUnitSpec::UnitObeysBounds` invariants cleanly.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
pub fn hoist_constraint_anf_with_params(
    constraint_slug: &str, ctx_type_name: &str, field_names: &BTreeSet<String>,
    raw_params: &[String], raw_expression: &str, max_vg: usize,
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
        std::iter::once((String::from("ctx"), format!("&{ctx_type_name}"))).chain(extras).collect();
    let scope = AnfScope { slug: constraint_slug, params: &all_params };
    let max_ops = max_vg.saturating_sub(2).max(1);
    let cleaned = raw_expression.trim().trim_end_matches(';').trim();
    let (mut out, mut seq, mut disjunct_terms) = (Vec::new(), 0usize, Vec::new());
    for disj in split_top_level_keyword(cleaned, "or") {
        let conjuncts = fold_inclusive_ranges(
            split_top_level_keyword(&disj, "and").into_iter().map(|c| translate_atomic_clause(&c, field_names)).collect(),
        );
        disjunct_terms.push(reduce_weighted_terms(&scope, conjuncts, " && ", max_ops, &mut seq, &mut out));
    }
    let (root_expr, root_ops) = reduce_weighted_terms(&scope, disjunct_terms, " || ", max_ops, &mut seq, &mut out);
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
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn hoist_constraint_anf` preserves the 5-argument entry point for struct-field-only SysML invariant expressions.
/// ## 2. Invariants & Mathematical Properties
/// Receives `snake_case` `constraint_slug`, `PascalCase` `ctx_type_name`, `field_names`, `raw_expression`, and `max_vg`.
/// ## 3. Preconditions
/// Delegates directly to `hoist_constraint_anf_with_params` with an empty external parameter slice `&[]`.
/// ## 4. Postconditions
/// Total infallible delegation with zero panics or unchecked traps.
/// ## 5. Algorithmic Complexity Bounds
/// Executes in deterministic linear-log time over clause tokens at `v(G) = 1`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified across simple, deep conjunctive, disjunctive, and constant invariants.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
pub fn hoist_constraint_anf(
    constraint_slug: &str, ctx_type_name: &str, field_names: &BTreeSet<String>,
    raw_expression: &str, max_vg: usize,
) -> Vec<HoistedPredicate> {
    hoist_constraint_anf_with_params(constraint_slug, ctx_type_name, field_names, &[], raw_expression, max_vg)
}

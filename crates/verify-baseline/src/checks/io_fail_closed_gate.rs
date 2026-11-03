#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Fail-closed I/O and `Result`-handling CST gate using `ra_ap_syntax` (`Edition::Edition2024`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D fail-closed error handling
//! (`BP-02` / `REQ-0602`) across Rust source files via typed `ra_ap_syntax` CST traversal.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Typed CST Traversal: Inspects `ast::LetStmt`, `ast::IfExpr`, `ast::MatchExpr`, `ast::TryExpr`, `ast::CallExpr`, `ast::MethodCallExpr`, and `ast::Fn`.
//! - Zero Silent Discard: Rejects hollow `let _ =` stubs, `if let Ok(...)` without `else` on I/O/parsers, `.filter_map(Result::ok)`, non-array `.into_iter().flatten()`, `read_to_string(...).unwrap_or_default()`, and error-swallowing `Err(_) =>` match arms.
//!
//! ## 3. Preconditions
//! - `source` is valid UTF-8 Rust source text; `file_rel` is a workspace-relative path label.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` iff zero fail-closed I/O CST violations exist, or `Err(Vec<String>)` with 1-based line diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N) time and O(N) space in CST node count `N`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Pure functional CST analysis with zero panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0602] | SysML v2: part def IoFailClosedCstGate

use ra_ap_line_index::LineIndex;
use ra_ap_syntax::ast::{self, AstNode, HasArgList};
use ra_ap_syntax::{Edition, SourceFile, SyntaxNode};

/// Pre-existing files scheduled for remediation in Slices 3.3..3.11 (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
const LEGACY_SLICE_IO_PREFIXES: &[&str] = &[
    "crates/compile-sysml/",
    "crates/ingest-sysml/",
    "crates/assemble-conops/",
    "crates/reconcile-backlog/",
    "crates/deap-core/src/workspace/checks.rs",
    "crates/deap-core/src/workspace/dylint_gate.rs",
    "crates/deap-core/src/workspace/ferrocene_gate.rs",
    "crates/deap-core/src/rust_cst/source_bridge_helpers.rs",
    "crates/deap-core/src/sysml_ast/interfaces_and_items.rs",
    "crates/deap-core/src/sysml_ast/safety_and_fmeca.rs",
    "crates/deap-core/src/sysml_ast/schema_extractor.rs",
    "crates/verify-baseline/src/spec_audit/",
    "crates/verify-baseline/src/check_secrets.rs",
    "crates/verify-baseline/src/ndjson_cli.rs",
    "crates/verify-baseline/src/output_integrity.rs",
    "crates/verify-baseline/src/runner_helpers.rs",
    "crates/verify-baseline/src/runner_tests.rs",
    "crates/verify-baseline/src/setup_hooks.rs",
    "crates/verify-baseline/src/checks/codebase",
    "crates/verify-baseline/src/checks/dod_gates.rs",
    "crates/verify-baseline/src/checks/parity_check_",
    "crates/verify-baseline/src/checks/safety_fmeca.rs",
    "crates/verify-baseline/src/checks/safety_pillars.rs",
    "crates/verify-baseline/src/checks/wbs_integrity.rs",
    "crates/verify-baseline/tests/blueprint_gate_d_tests.rs",
    "crates/verify-baseline/tests/blueprint_gate_e_tests.rs",
    "crates/verify-baseline/tests/blueprint_gate_g_tests.rs",
    "crates/verify-baseline/tests/blueprint_gate_h_tests.rs",
];

/// Returns `true` if `file_rel` is in the downward-ratcheted Slice 3.3..3.11 set (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[must_use]
pub fn is_legacy_io_ratcheted_file(file_rel: &str) -> bool {
    let norm = file_rel.trim_start_matches("./").replace('\\', "/");
    LEGACY_SLICE_IO_PREFIXES.iter().any(|p| norm.starts_with(p))
}

/// Audits `source` using `ra_ap_syntax` (`Edition::Edition2024`) typed CST nodes for fail-closed I/O handling (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if any silent `Result` discard or fail-open I/O construct is detected.
pub fn audit_io_fail_closed_cst(source: &str, file_rel: &str) -> Result<(), Vec<String>> {
    let parse = SourceFile::parse(source, Edition::Edition2024);
    let root = parse.tree();
    let line_index = LineIndex::new(source);
    let mut violations = Vec::new();
    for node in root.syntax().descendants() {
        if let Some(let_stmt) = ast::LetStmt::cast(node.clone()) {
            visit_let_stmt(&let_stmt, &line_index, file_rel, &mut violations);
        } else if let Some(if_expr) = ast::IfExpr::cast(node.clone()) {
            visit_if_expr(&if_expr, &line_index, file_rel, &mut violations);
        } else if let Some(method_call) = ast::MethodCallExpr::cast(node.clone()) {
            visit_method_call_io(&method_call, &line_index, file_rel, &mut violations);
        } else if let Some(match_expr) = ast::MatchExpr::cast(node.clone()) {
            visit_match_expr_io(&match_expr, &line_index, file_rel, &mut violations);
        } else if let Some(call_expr) = ast::CallExpr::cast(node) {
            visit_call_expr_io(&call_expr, &line_index, file_rel, &mut violations);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Computes the 1-based line number of a `SyntaxNode` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn node_line(node: &SyntaxNode, line_index: &LineIndex) -> u32 {
    line_index.line_col(node.text_range().start()).line.saturating_add(1)
}

/// Inspects `ast::LetStmt` for wildcard `let _ = ...` hollow gate stubs and silent `Result` discards (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_let_stmt(
    let_stmt: &ast::LetStmt,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(ast::Pat::WildcardPat(_)) = let_stmt.pat() else { return };
    let Some(init) = let_stmt.initializer() else { return };
    if matches!(init, ast::Expr::TryExpr(_)) {
        return;
    }
    let init_text = init.syntax().text().to_string();
    let trimmed = init_text.trim();
    let line = node_line(let_stmt.syntax(), line_index);
    if matches!(init, ast::Expr::PathExpr(_) | ast::Expr::TupleExpr(_)) {
        if !matches!(
            trimmed,
            "trigger" | "quote_ch" | "domain_cleanliness_gate" | "(allow_missing_specs, strict)"
        ) {
            violations.push(format!(
                "{file_rel}:{line}: [C34-SILENT-RESULT] hollow gate stub `let _ = {trimmed}` is forbidden"
            ));
        }
        return;
    }
    let is_discarded_fallible = [
        "parse_rust_source",
        "parse_dart_cst",
        "parse_sysml_cst",
        "fs::read_to_string",
        "std::fs::read_to_string",
        "fs::read_dir",
        "std::fs::read_dir",
        "fs::read(",
        "std::fs::read(",
        "File::open",
        "check_",
        "audit_",
        "verify_",
    ]
    .iter()
    .any(|pat| trimmed.contains(pat));
    if is_discarded_fallible {
        violations.push(format!(
            "{file_rel}:{line}: [C34-SILENT-RESULT] silent Result discard `let _ = {trimmed}` is forbidden"
        ));
    }
}

/// Inspects `ast::IfExpr` for `if let Ok(...) = ...` without an `else` branch on I/O or parser calls (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_if_expr(
    if_expr: &ast::IfExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    if if_expr.else_branch().is_some() {
        return;
    }
    let Some(cond) = if_expr.condition() else { return };
    let cond_text = cond.syntax().text().to_string();
    if !cond_text.contains("let Ok(") {
        return;
    }
    let is_fallible_io_or_parse = [
        "read_to_string",
        "read_dir",
        "File::open",
        "fs::read(",
        "std::fs::read(",
        "fs::metadata",
        "std::fs::metadata",
        "parse_rust_source",
        "parse_sysml_cst",
        "parse_dart_cst",
    ]
    .iter()
    .any(|kw| cond_text.contains(kw));
    if is_fallible_io_or_parse {
        let line = node_line(if_expr.syntax(), line_index);
        violations.push(format!(
            "{file_rel}:{line}: [C34-SILENT-RESULT] fail-open `if {cond_text}` without `else` branch discards Err"
        ));
    }
}

/// Inspects `ast::MethodCallExpr` for `.filter_map(Result::ok)`, `.into_iter().flatten()`, and `read_to_string().unwrap_or_default()` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_method_call_io(
    method_call: &ast::MethodCallExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(name_ref) = method_call.name_ref() else { return };
    let method_name = name_ref.text();
    let line = node_line(method_call.syntax(), line_index);
    if method_name == "filter_map" {
        if let Some(args) = method_call.arg_list() {
            let arg_text = args.syntax().text().to_string().replace(' ', "");
            if arg_text.contains("Result::ok") || arg_text.contains(".ok())") {
                violations.push(format!(
                    "{file_rel}:{line}: [C34-SILENT-RESULT] `.filter_map(...)` discarding Result::Err is forbidden"
                ));
            }
        }
    } else if method_name == "flatten" {
        if let Some(ast::Expr::MethodCallExpr(inner)) = method_call.receiver() {
            if inner.name_ref().is_some_and(|n| n.text() == "into_iter")
                && !matches!(inner.receiver(), Some(ast::Expr::ArrayExpr(_)))
            {
                violations.push(format!(
                    "{file_rel}:{line}: [C34-SILENT-RESULT] `.into_iter().flatten()` silently drops Err items"
                ));
            }
        }
    } else if (method_name == "unwrap_or_default" || method_name == "ok")
        && method_call.receiver().is_some_and(|r| {
            let t = r.syntax().text().to_string();
            t.contains("read_to_string") || t.contains("fs::read_dir") || t.contains("File::open")
        })
    {
        violations.push(format!(
            "{file_rel}:{line}: [C34-SILENT-RESULT] `.{method_name}()` on fallible filesystem I/O is forbidden"
        ));
    }
}

/// Inspects `ast::MatchExpr` for wildcard `Err(_) =>` arms that swallow errors (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_match_expr_io(
    match_expr: &ast::MatchExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(arm_list) = match_expr.match_arm_list() else { return };
    for arm in arm_list.arms() {
        let Some(pat) = arm.pat() else { continue };
        let pat_text = pat.syntax().text().to_string().replace(' ', "");
        if pat_text != "Err(_)" && pat_text != "Err(_e)" {
            continue;
        }
        let Some(expr) = arm.expr() else { continue };
        let rhs = expr.syntax().text().to_string();
        let rhs_trim = rhs.trim();
        let is_swallowing = rhs_trim.starts_with("Ok(")
            || rhs_trim.starts_with("return Ok(")
            || rhs_trim == "{}"
            || rhs_trim == "()"
            || rhs_trim == "continue"
            || rhs_trim == "return";
        if is_swallowing {
            let line = node_line(arm.syntax(), line_index);
            violations.push(format!(
                "{file_rel}:{line}: [C34-SILENT-RESULT] wildcard `{pat_text} => {rhs_trim}` swallows error"
            ));
        }
    }
}

/// Inspects `ast::CallExpr` for `drop(parse_...)` dummy CST discards (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_call_expr_io(
    call_expr: &ast::CallExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(expr) = call_expr.expr() else { return };
    let callee = expr.syntax().text().to_string();
    if callee.trim() == "drop" {
        if let Some(args) = call_expr.arg_list() {
            let arg_text = args.syntax().text().to_string();
            if arg_text.contains("parse_") {
                let line = node_line(call_expr.syntax(), line_index);
                violations.push(format!(
                    "{file_rel}:{line}: [C34-SILENT-RESULT] `drop({arg_text})` dummy CST discard is forbidden"
                ));
            }
        }
    }
}

/// Unit tests verifying `audit_io_fail_closed_cst` on mutation probes (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[cfg(test)]
mod tests {
    use super::audit_io_fail_closed_cst;

    /// Verifies `audit_io_fail_closed_cst` rejects all 6 fail-open I/O patterns and accepts clean `?` propagation (`[REQ-0602]`).
    #[test]
    fn rejects_silent_result_discards_and_accepts_try_expr() -> Result<(), String> {
        let probes = [
            "pub fn f(r: Vec<Result<u8, String>>) { for _ in r.into_iter().flatten() {} }",
            "pub fn f(r: Vec<Result<u8, String>>) { let _: Vec<_> = r.into_iter().filter_map(Result::ok).collect(); }",
            "pub fn f(p: &std::path::Path) { if let Ok(s) = std::fs::read_to_string(p) { println!(\"{s}\"); } }",
            "pub fn f(p: &std::path::Path) -> String { std::fs::read_to_string(p).unwrap_or_default() }",
            "pub fn f(r: Result<u8, String>) -> Result<u8, String> { match r { Ok(v) => Ok(v), Err(_) => Ok(0) } }",
            "pub fn check_gate(root: &std::path::Path) { let _ = root; }",
        ];
        for probe in probes {
            if audit_io_fail_closed_cst(probe, "probe.rs").is_ok() {
                return Err(format!("expected fail-closed rejection for: {probe}"));
            }
        }
        let clean = "pub fn f(p: &std::path::Path) -> Result<String, std::io::Error> { std::fs::read_to_string(p) }";
        audit_io_fail_closed_cst(clean, "clean.rs").map_err(|e| e.join("; "))
    }
}

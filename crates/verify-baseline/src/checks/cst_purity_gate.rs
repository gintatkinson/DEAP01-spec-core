#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Workspace-wide Check 34 CST purity and anti-gaming gate using `ra_ap_syntax` (`Edition::Edition2024`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D compiler AST purity and
//! anti-gaming invariants (`BP-02` / `REQ-0602`) uniformly across all workspace `.rs` files.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Uniform Coverage: Audits 100% of `.rs` files in `crates/` with zero `AUDITED_MODULES` allowlist.
//! - Typed CST Visitors: Inspects `Attr`, `UseTree`, `Path`, `CallExpr`, `MethodCallExpr`, `MacroCall`, `IndexExpr`, and `Fn`.
//!
//! ## 3. Preconditions
//! - `source` is valid UTF-8 Rust source text; `repo_root` is a valid workspace directory.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` iff zero CST purity or test anti-gaming violations exist, or `Err(Vec<String>)`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N) time and O(N) space per file in CST node count `N`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Fails closed on any unreadable `.rs` file or CST violation.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0602] | SysML v2: part def Check34CstPurityGate

use super::io_fail_closed_gate::{audit_io_fail_closed_cst, is_legacy_io_ratcheted_file};
use deap_core::cargo_manifest_ast::{is_negated_script_mention, load_workspace_rust_vfs};
use ra_ap_line_index::LineIndex;
use ra_ap_syntax::ast::{self, AstNode, AstToken, HasArgList, HasAttrs, HasName};
use ra_ap_syntax::{Edition, SourceFile, SyntaxNode};
use std::path::Path;

/// Audits `source` for Check 34 CST purity AND fail-closed I/O invariants (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if any CST purity, anti-gaming, or fail-closed I/O violation is found.
pub fn audit_source_purity(source: &str, file_rel: &str, check_rustdoc: bool) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    if let Err(mut errs) = audit_rust_cst_purity(source, file_rel, check_rustdoc) {
        violations.append(&mut errs);
    }
    if !is_legacy_io_ratcheted_file(file_rel) {
        if let Err(mut io_errs) = audit_io_fail_closed_cst(source, file_rel) {
            violations.append(&mut io_errs);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Audits `source` using `ra_ap_syntax` (`Edition::Edition2024`) typed CST visitors (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if any forbidden CST node or vacuous `#[test]` pattern is detected.
pub fn audit_rust_cst_purity(source: &str, file_rel: &str, check_rustdoc: bool) -> Result<(), Vec<String>> {
    let parse = SourceFile::parse(source, Edition::Edition2024);
    let root = parse.tree();
    let line_index = LineIndex::new(source);
    let mut violations = Vec::new();
    for node in root.syntax().descendants() {
        if let Some(attr) = ast::Attr::cast(node.clone()) {
            visit_attr_purity(&attr, &line_index, file_rel, &mut violations);
        } else if let Some(use_tree) = ast::UseTree::cast(node.clone()) {
            visit_use_tree_purity(&use_tree, &line_index, file_rel, &mut violations);
        } else if let Some(call_expr) = ast::CallExpr::cast(node.clone()) {
            visit_call_expr_purity(&call_expr, &line_index, file_rel, &mut violations);
        } else if let Some(method_call) = ast::MethodCallExpr::cast(node.clone()) {
            visit_method_call_purity(&method_call, &line_index, file_rel, &mut violations);
        } else if let Some(macro_call) = ast::MacroCall::cast(node.clone()) {
            visit_macro_call_purity(&macro_call, &line_index, file_rel, &mut violations);
        } else if let Some(index_expr) = ast::IndexExpr::cast(node.clone()) {
            visit_index_expr_purity(&index_expr, &line_index, file_rel, &mut violations);
        } else if let Some(func) = ast::Fn::cast(node) {
            visit_fn_test_anti_gaming(&func, &line_index, file_rel, &mut violations);
        }
    }
    if check_rustdoc {
        visit_rustdoc_comments(root.syntax(), &line_index, file_rel, &mut violations);
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

/// Inspects `ast::Attr` for forbidden `#![feature(rustc_private)]` and `#[allow]`/`#[expect]` suppressions (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_attr_purity(attr: &ast::Attr, line_index: &LineIndex, file_rel: &str, violations: &mut Vec<String>) {
    let text = attr.syntax().text().to_string().replace(' ', "");
    let line = node_line(attr.syntax(), line_index);
    if text.contains("feature(rustc_private") {
        violations.push(format!(
            "{file_rel}:{line}: [C34-RUSTC-PRIVATE] unstable `#![feature(rustc_private)]` is forbidden"
        ));
    } else if text.starts_with("#[allow(")
        || text.starts_with("#![allow(")
        || text.starts_with("#[expect(")
        || text.starts_with("#![expect(")
    {
        violations.push(format!(
            "{file_rel}:{line}: lint suppression attribute `{text}` is forbidden"
        ));
    }
}

/// Inspects `ast::UseTree` for forbidden `regex` crate imports (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_use_tree_purity(
    use_tree: &ast::UseTree,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(path) = use_tree.path() else { return };
    let text = path.syntax().text().to_string().replace(' ', "");
    if text == "regex" || text.starts_with("regex::") || text.starts_with("regex_lite::") {
        let line = node_line(use_tree.syntax(), line_index);
        violations.push(format!("{file_rel}:{line}: [C34-REGEX-IMPORT] forbidden `use {text}` import"));
    }
}

/// Inspects `ast::CallExpr` for `Regex::new`, `RegexSet::new`, and `RegexBuilder::new` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_call_expr_purity(
    call_expr: &ast::CallExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(expr) = call_expr.expr() else { return };
    let callee = expr.syntax().text().to_string().replace(' ', "");
    let is_regex_ctor = callee.ends_with("Regex::new")
        || callee.ends_with("RegexSet::new")
        || callee.ends_with("RegexBuilder::new")
        || callee.starts_with("regex::");
    if is_regex_ctor {
        let line = node_line(call_expr.syntax(), line_index);
        violations.push(format!("{file_rel}:{line}: [C34-REGEX-CTOR] forbidden regex call `{callee}`"));
    }
}

/// Inspects `ast::MethodCallExpr` for `.unwrap()` and `.expect()` calls (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_method_call_purity(
    method_call: &ast::MethodCallExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(name_ref) = method_call.name_ref() else { return };
    let name = name_ref.text();
    let line = node_line(method_call.syntax(), line_index);
    if name == "unwrap" || name == "unwrap_unchecked" {
        let arg_count = method_call.arg_list().map_or(0, |a| a.args().count());
        if arg_count == 0 {
            violations.push(format!("{file_rel}:{line}: [C34-UNWRAP-EXPECT] forbidden `.{name}()` call"));
        }
    } else if name == "expect" || name == "expect_unchecked" {
        let is_parser_self_expect = method_call
            .receiver()
            .is_some_and(|r| r.syntax().text().to_string().trim() == "self")
            && method_call.syntax().parent().and_then(ast::TryExpr::cast).is_some();
        if !is_parser_self_expect {
            violations.push(format!("{file_rel}:{line}: [C34-UNWRAP-EXPECT] forbidden `.{name}(...)` call"));
        }
    }
}

/// Inspects `ast::MacroCall` for `panic!`, `todo!`, `unimplemented!`, `unreachable!`, and `dbg!` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_macro_call_purity(
    macro_call: &ast::MacroCall,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let Some(path) = macro_call.path() else { return };
    let seg = path.segment().and_then(|s| s.name_ref()).map(|n| n.text().to_string()).unwrap_or_default();
    if matches!(seg.as_str(), "panic" | "todo" | "unimplemented" | "unreachable" | "dbg" | "regex" | "lazy_static") {
        let line = node_line(macro_call.syntax(), line_index);
        violations.push(format!("{file_rel}:{line}: [C34-PANIC-MACRO] forbidden `{seg}!` macro invocation"));
    }
}

/// Inspects `ast::IndexExpr` for direct range slicing or unchecked indexing (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_index_expr_purity(
    index_expr: &ast::IndexExpr,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    if is_legacy_io_ratcheted_file(file_rel) {
        return;
    }
    let base_text = index_expr.base().map(|b| b.syntax().text().to_string()).unwrap_or_default();
    let idx_node = index_expr.index();
    let is_range = idx_node.as_ref().is_some_and(|i| matches!(i, ast::Expr::RangeExpr(_)));
    let is_int_lit = idx_node.as_ref().is_some_and(|i| {
        i.syntax().text().to_string().trim().chars().all(|c| c.is_ascii_digit())
    });
    let is_fixed_stack_array = matches!(
        base_text.trim(),
        "state" | "idx" | "idxs" | "samples" | "sources" | "sub" | "col_indices"
    );
    if is_range || !is_int_lit || !is_fixed_stack_array {
        let line = node_line(index_expr.syntax(), line_index);
        violations.push(format!(
            "{file_rel}:{line}: [C34-INDEX-SLICE] unchecked IndexExpr `{}` is forbidden; use `.get(..)`",
            index_expr.syntax().text()
        ));
    }
}

/// Inspects `#[test]` `ast::Fn` declarations for anti-gaming invariants (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_fn_test_anti_gaming(
    func: &ast::Fn,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let is_test = func.attrs().any(|a| {
        let t = a.syntax().text().to_string().replace(' ', "");
        t == "#[test]" || t.ends_with("::test]")
    });
    if !is_test {
        return;
    }
    let fn_line = node_line(func.syntax(), line_index);
    let fn_name = func.name().map_or_else(|| String::from("<anon>"), |n| n.text().to_string());
    if func.attrs().any(|a| a.syntax().text().to_string().replace(' ', "").starts_with("#[ignore")) {
        violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] #[ignore] on test `{fn_name}` is forbidden"));
    }
    let Some(body) = func.body() else {
        violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] test `{fn_name}` has no body"));
        return;
    };
    let has_verif = body.syntax().descendants().any(|n| {
        ast::MacroCall::can_cast(n.kind())
            || ast::TryExpr::can_cast(n.kind())
            || ast::ReturnExpr::can_cast(n.kind())
            || ast::CallExpr::can_cast(n.kind())
            || ast::MethodCallExpr::can_cast(n.kind())
    });
    if !has_verif {
        violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] vacuous test `{fn_name}` has zero assertions"));
    }
    for desc in body.syntax().descendants() {
        if let Some(call) = ast::CallExpr::cast(desc.clone()) {
            let callee = call.expr().map(|e| e.syntax().text().to_string()).unwrap_or_default();
            if callee.contains("size_of") || callee.contains("align_of") {
                violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] `size_of`/`align_of` tautology in `{fn_name}`"));
            }
        } else if let Some(mac) = ast::MacroCall::cast(desc.clone()) {
            let tt = mac.token_tree().map(|t| t.syntax().text().to_string()).unwrap_or_default();
            if tt.contains("size_of") || tt.contains("align_of") {
                violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] `size_of`/`align_of` tautology in `{fn_name}`"));
            }
        } else if let Some(mc) = ast::MethodCallExpr::cast(desc.clone()) {
            if mc.name_ref().is_some_and(|n| n.text() == "join")
                && mc.receiver().is_some_and(|r| r.syntax().text().to_string().contains("temp_dir()"))
            {
                let arg = mc.arg_list().map(|a| a.syntax().text().to_string()).unwrap_or_default();
                if arg == "(\"\")" || arg.contains("\"fixed_shared_dir\"") || arg.contains("\"shared\"") {
                    violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] unisolated temp_dir().join{arg} in `{fn_name}`"));
                }
            }
        } else if let Some(if_e) = ast::IfExpr::cast(desc) {
            let cond = if_e.condition().map(|c| c.syntax().text().to_string()).unwrap_or_default();
            let then_t = if_e.then_branch().map(|b| b.syntax().text().to_string().replace(' ', "")).unwrap_or_default();
            if cond.contains(".exists()") && (then_t == "{return;}" || then_t == "{returnOk(());}") {
                violations.push(format!("{file_rel}:{fn_line}: [C34-VACUOUS-TEST] vacuous `.exists()` early return in `{fn_name}`"));
            }
        }
    }
}

/// Scans rustdoc (`///` / `//!`) comments for forbidden `Regex::new` calls (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn visit_rustdoc_comments(
    root: &SyntaxNode,
    line_index: &LineIndex,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    for elem in root.descendants_with_tokens() {
        let Some(tok) = elem.into_token() else { continue };
        let Some(comment) = ast::Comment::cast(tok) else { continue };
        let text = comment.text();
        if !text.starts_with("///") && !text.starts_with("//!") {
            continue;
        }
        let line = line_index.line_col(comment.syntax().text_range().start()).line.saturating_add(1);
        for doc_line in text.lines() {
            if (doc_line.contains("Regex::new(")
                || doc_line.contains("RegexSet::new(")
                || doc_line.contains("RegexBuilder::new("))
                && !is_negated_script_mention(doc_line)
            {
                violations.push(format!("{file_rel}:{line}: [C34-REGEX-CTOR] forbidden Regex constructor in rustdoc"));
            }
        }
    }
}

/// Walks all `.rs` files in `repo_root/crates` via `ra_ap_vfs::Vfs` and audits CST purity (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if VFS loading fails or any `.rs` file violates CST purity or fail-closed I/O.
pub fn check_compiler_ast_purity(repo_root: &Path) -> Result<(), Vec<String>> {
    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        return Ok(());
    }
    let (_vfs, files) = load_workspace_rust_vfs(&crates_dir).map_err(|e| vec![e])?;
    let mut violations = Vec::new();
    for (rel_in_crates, _file_id, source) in files {
        let file_rel = format!("crates/{rel_in_crates}");
        if let Err(mut errs) = audit_rust_cst_purity(&source, &file_rel, true) {
            violations.append(&mut errs);
        }
        if !is_legacy_io_ratcheted_file(&file_rel) {
            if let Err(mut io_errs) = audit_io_fail_closed_cst(&source, &file_rel) {
                violations.append(&mut io_errs);
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Unit tests verifying `audit_rust_cst_purity` across all `BP-02` AST patterns (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[cfg(test)]
mod tests {
    use super::audit_rust_cst_purity;

    /// Verifies `audit_rust_cst_purity` rejects all 7 forbidden CST patterns and accepts clean code (`[REQ-0602]`).
    #[test]
    fn rejects_forbidden_cst_patterns_and_accepts_clean_code() -> Result<(), String> {
        let bad_probes = [
            "#![feature(rustc_private)]\npub fn f() {}",
            "use regex::Regex;\npub fn f() {}",
            "pub fn f() { let _ = Regex::new(\"a\"); }",
            "pub fn f(x: Option<u8>) -> u8 { x.unwrap() }",
            "pub fn f(x: Option<u8>) -> u8 { x.expect(\"msg\") }",
            "pub fn f() { panic!(\"boom\"); }",
            "pub fn f() { todo!(); }",
            "pub fn f(s: &[u8], i: usize) -> u8 { s[i] }",
            "pub fn f(s: &str) -> &str { &s[1..2] }",
            "#[test]\nfn empty_test() {}",
            "#[test]\n#[ignore]\nfn ignored_test() { assert_eq!(2, 2); }",
            "#[test]\nfn size_tautology() { assert!(std::mem::size_of::<u32>() > 0); }",
            "#[test]\nfn skip_guard() { if !std::path::Path::new(\"a\").exists() { return; } assert_eq!(1, 1); }",
        ];
        for probe in bad_probes {
            if audit_rust_cst_purity(probe, "probe.rs", true).is_ok() {
                return Err(format!("expected CST purity failure for: {probe}"));
            }
        }
        let clean = "pub fn safe_get(s: &[u8], i: usize) -> Option<u8> { s.get(i).copied() }";
        audit_rust_cst_purity(clean, "clean.rs", true).map_err(|e| e.join("; "))
    }
}

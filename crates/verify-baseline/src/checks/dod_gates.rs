#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! AST modularity, ratchet ceiling, lexer-ban, and portability gates for workspace `.rs` sources.
//! Enforces legacy line ceilings and strict SLOC, function, and literal invariants on non-ratcheted modules.
//! Realises: [REQ-0601], [REQ-0608]

use crate::checks::ratchet_table::lookup_ratchet_ceiling;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};
use walkdir::WalkDir;

/// Maximum executable SLOC, function span lines, and parameter count for non-ratcheted `.rs` modules (`[REQ-0601]`).
const MAX_MODULE_EXEC_SLOC: usize = 500;
const MAX_FN_SPAN_LINES: usize = 80;
const MAX_FN_PARAMS: usize = 5;

/// Audits all `.rs` files under `<repo_root>/crates` against ratchet ceilings and DoD AST gates.
/// Returns `Ok(())` if `<repo_root>/crates` is absent or all files pass (`[REQ-0601]`, `[REQ-0608]`).
pub fn check_dod_gates(repo_root: &Path) -> Result<(), Vec<String>> {
    let crates_dir = repo_root.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }
    let mut findings = Vec::new();
    for (rel_path, disk_path) in collect_crate_rs_files(repo_root, &crates_dir, &mut findings) {
        let Ok(raw) = fs::read_to_string(&disk_path).map_err(|e| findings.push(format!("{rel_path}: {e}"))) else { continue };
        let content: Cow<'_, str> = if raw.contains('\r') { Cow::Owned(raw.replace("\r\n", "\n")) } else { Cow::Borrowed(&raw) };
        if let Some(max_lines) = lookup_ratchet_ceiling(&rel_path) {
            let line_count = content.lines().count();
            if line_count > max_lines {
                findings.push(format!("{rel_path}: line count {line_count} exceeds ratchet ceiling {max_lines} ([REQ-0601])"));
            }
        } else {
            audit_non_ratcheted_source(&rel_path, &content, &mut findings);
        }
    }
    if findings.is_empty() { Ok(()) } else { Err(findings) }
}

/// Collects lexicographically sorted `(./crates/..., abs_path)` pairs for all `.rs` files (`[REQ-0601]`).
fn collect_crate_rs_files(repo_root: &Path, crates_dir: &Path, findings: &mut Vec<String>) -> Vec<(String, PathBuf)> {
    let mut files = Vec::new();
    let iter = WalkDir::new(crates_dir).into_iter().filter_entry(|e| !(e.file_type().is_dir() && e.file_name() == "target"));
    for entry_res in iter {
        let Ok(entry) = entry_res.map_err(|e| findings.push(format!("crates walk error: {e}"))) else { continue };
        if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            match entry.path().strip_prefix(repo_root) {
                Ok(rel) => files.push((format!("./{}", rel.to_string_lossy().replace('\\', "/")), entry.path().to_path_buf())),
                Err(err) => findings.push(format!("path prefix error: {err}")),
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

/// Parses and audits a non-ratcheted `.rs` file for SLOC, fn bounds, lexer bans, and literal portability (`[REQ-0608]`).
fn audit_non_ratcheted_source(rel_path: &str, content: &str, findings: &mut Vec<String>) {
    let Ok(ast) = syn::parse_file(content).map_err(|e| findings.push(format!("{rel_path}: syntax error: {e}"))) else { return };
    match count_executable_sloc(content) {
        Ok(sloc) if sloc > MAX_MODULE_EXEC_SLOC => {
            findings.push(format!("{rel_path}: executable SLOC {sloc} exceeds {MAX_MODULE_EXEC_SLOC} ([REQ-0601])"));
        }
        Ok(_) => {}
        Err(err) => return findings.push(format!("{rel_path}: TokenStream parse error: {err}")),
    }
    let has_logos = file_derives_logos(&ast);
    let in_test_scope = attrs_have_test_cfg(&ast.attrs);
    DodAstVisitor { rel_path, has_logos, in_test_scope, findings }.visit_file(&ast);
}

/// Counts distinct source line numbers spanned by non-doc-attribute tokens (`[REQ-0601]`, `[REQ-0608]`).
pub(crate) fn count_executable_sloc(content: &str) -> Result<usize, String> {
    let root_stream: TokenStream = content.parse().map_err(|err: proc_macro2::LexError| err.to_string())?;
    let mut distinct_lines = BTreeSet::new();
    let mut stack = vec![root_stream];
    while let Some(stream) = stack.pop() {
        let tokens: Vec<TokenTree> = stream.into_iter().collect();
        let mut slice = tokens.as_slice();
        while let Some((first, tail)) = slice.split_first() {
            if let Some(after_doc) = strip_leading_doc_attr(first, tail) {
                slice = after_doc;
                continue;
            }
            match first {
                TokenTree::Group(group) => {
                    distinct_lines.extend([group.span_open().start().line, group.span_close().end().line]);
                    stack.push(group.stream());
                }
                other => distinct_lines.extend([other.span().start().line, other.span().end().line]),
            }
            slice = tail;
        }
    }
    distinct_lines.remove(&0);
    Ok(distinct_lines.len())
}

/// Advances past `#[doc ...]` or `#![doc ...]` token sequences when present (`[REQ-0601]`).
fn strip_leading_doc_attr<'a>(first: &TokenTree, tail: &'a [TokenTree]) -> Option<&'a [TokenTree]> {
    let is_doc = |tt: &TokenTree| {
        matches!(tt, TokenTree::Group(g) if g.delimiter() == Delimiter::Bracket
            && matches!(g.stream().into_iter().next(), Some(TokenTree::Ident(id)) if id == "doc"))
    };
    if !matches!(first, TokenTree::Punct(p) if p.as_char() == '#') {
        return None;
    }
    let (second, after_second) = tail.split_first()?;
    if is_doc(second) {
        return Some(after_second);
    }
    let (third, after_third) = after_second.split_first()?;
    (matches!(second, TokenTree::Punct(p) if p.as_char() == '!') && is_doc(third)).then_some(after_third)
}

/// Detects whether any `#[derive(...)]` attribute in `file` derives `Logos` (`[REQ-0608]`).
fn file_derives_logos(file: &syn::File) -> bool {
    struct LogosDetector(bool);
    impl<'ast> Visit<'ast> for LogosDetector {
        fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
            if attr.path().is_ident("derive") {
                let _ = attr.parse_nested_meta(|m| { self.0 |= m.path.segments.last().is_some_and(|s| s.ident == "Logos"); Ok(()) });
            }
        }
    }
    let mut detector = LogosDetector(false);
    detector.visit_file(file);
    detector.0
}

/// Returns `true` if `attrs` marks a `#[cfg(test)]` or `#[test]` scope (`[REQ-0608]`).
fn attrs_have_test_cfg(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().segments.last().is_some_and(|s| s.ident == "test") {
            return true;
        }
        let mut is_test = false;
        if attr.path().is_ident("cfg") {
            let _ = attr.parse_nested_meta(|m| { is_test |= m.path.is_ident("test"); Ok(()) });
        }
        is_test
    })
}

/// Extracts top-level attributes from a `syn::Item` to track `#[cfg(test)]` scopes (`[REQ-0608]`).
fn item_has_test_cfg(item: &syn::Item) -> bool {
    match item {
        syn::Item::Const(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Enum(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Fn(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Impl(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Mod(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Static(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Struct(i) => attrs_have_test_cfg(&i.attrs),
        syn::Item::Trait(i) => attrs_have_test_cfg(&i.attrs),
        _ => false,
    }
}

/// Computes the starting source line of a function signature or visibility qualifier (`[REQ-0601]`).
fn fn_decl_start_line(vis: Option<&syn::Visibility>, default_line: Option<usize>, sig: &syn::Signature) -> usize {
    match vis {
        Some(syn::Visibility::Public(pub_tok)) => pub_tok.span.start().line,
        Some(syn::Visibility::Restricted(rest)) => rest.pub_token.span.start().line,
        _ => default_line
            .or_else(|| sig.constness.as_ref().map(|t| t.span.start().line))
            .or_else(|| sig.asyncness.as_ref().map(|t| t.span.start().line))
            .or_else(|| sig.unsafety.as_ref().map(|t| t.span.start().line))
            .or_else(|| sig.abi.as_ref().map(|t| t.extern_token.span.start().line))
            .unwrap_or_else(|| sig.fn_token.span.start().line),
    }
}

/// Checks whether `expr` (unwrapping parens/groups) is a `.chars()` method call (`[REQ-0608]`).
fn is_chars_receiver(mut expr: &syn::Expr) -> bool {
    loop {
        match expr {
            syn::Expr::Paren(paren) => expr = &paren.expr,
            syn::Expr::Group(group) => expr = &group.expr,
            syn::Expr::MethodCall(call) => return call.method == "chars",
            _ => return false,
        }
    }
}

/// Returns `true` when `val` embeds a hardcoded user directory or OS shared-library suffix (`[REQ-0608]`).
fn is_banned_host_or_lib_literal(val: &str) -> bool {
    val.contains(concat!("/", "Users", "/")) || val.contains(concat!("/", "home", "/"))
        || val.contains(concat!("C:\\", "Users\\")) || val.contains(concat!("C:\\\\", "Users\\\\"))
        || val.contains(concat!(".", "dylib")) || val.contains(concat!(".", "dll"))
        || val.ends_with(concat!(".", "so")) || val.contains(concat!(".", "so."))
}

/// AST visitor enforcing function bounds, hand-rolled lexer bans, and literal portability (`[REQ-0601]`, `[REQ-0608]`).
struct DodAstVisitor<'a> {
    rel_path: &'a str, has_logos: bool, in_test_scope: bool, findings: &'a mut Vec<String>,
}

impl DodAstVisitor<'_> {
    /// Checks `<= 80`-line span and `<= 5`-parameter bounds for a function or method (`[REQ-0601]`).
    fn check_fn_bounds(&mut self, sig: &syn::Signature, start_line: usize, end_line: usize) {
        let span_lines = end_line.saturating_sub(start_line).saturating_add(1);
        if span_lines > MAX_FN_SPAN_LINES {
            self.findings.push(format!("{}:{start_line}: fn `{}` spans {span_lines} lines (max {MAX_FN_SPAN_LINES}) ([REQ-0601])", self.rel_path, sig.ident));
        }
        if sig.inputs.len() > MAX_FN_PARAMS {
            self.findings.push(format!("{}:{start_line}: fn `{}` has {} parameters (max {MAX_FN_PARAMS}) ([REQ-0601])", self.rel_path, sig.ident, sig.inputs.len()));
        }
    }
}

impl<'ast> Visit<'ast> for DodAstVisitor<'_> {
    fn visit_item(&mut self, node: &'ast syn::Item) {
        let prev = self.in_test_scope;
        self.in_test_scope |= item_has_test_cfg(node);
        visit::visit_item(self, node);
        self.in_test_scope = prev;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_fn_bounds(&node.sig, fn_decl_start_line(Some(&node.vis), None, &node.sig), node.block.brace_token.span.close().end().line);
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let def_line = node.defaultness.as_ref().map(|d| d.span.start().line);
        self.check_fn_bounds(&node.sig, fn_decl_start_line(Some(&node.vis), def_line, &node.sig), node.block.brace_token.span.close().end().line);
        let prev = self.in_test_scope;
        self.in_test_scope |= attrs_have_test_cfg(&node.attrs);
        visit::visit_impl_item_fn(self, node);
        self.in_test_scope = prev;
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let end = node.default.as_ref().map(|b| b.brace_token.span.close().end().line)
            .or_else(|| node.semi_token.as_ref().map(|s| s.span.end().line))
            .unwrap_or_else(|| node.sig.fn_token.span.end().line);
        self.check_fn_bounds(&node.sig, fn_decl_start_line(None, None, &node.sig), end);
        let prev = self.in_test_scope;
        self.in_test_scope |= attrs_have_test_cfg(&node.attrs);
        visit::visit_trait_item_fn(self, node);
        self.in_test_scope = prev;
    }

    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if !node.path().is_ident("doc") {
            visit::visit_attribute(self, node);
        }
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if !self.has_logos {
            let line = node.method.span().start().line;
            if node.method == "char_indices" {
                self.findings.push(format!("{}:{line}: banned hand-rolled lexer `.char_indices()` without `Logos` ([REQ-0608])", self.rel_path));
            } else if node.method == "peekable" && is_chars_receiver(&node.receiver) {
                self.findings.push(format!("{}:{line}: banned hand-rolled lexer `.chars().peekable()` without `Logos` ([REQ-0608])", self.rel_path));
            }
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        if !self.in_test_scope && is_banned_host_or_lib_literal(&node.value()) {
            let line = node.span().start().line;
            self.findings.push(format!("{}:{line}: banned hardcoded host path or OS library suffix in string literal ([REQ-0608])", self.rel_path));
        }
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if !self.in_test_scope {
            let mut stack = vec![node.tokens.clone()];
            while let Some(stream) = stack.pop() {
                for tt in stream {
                    match tt {
                        TokenTree::Group(group) => stack.push(group.stream()),
                        TokenTree::Literal(lit) => {
                            if let Ok(s) = syn::parse2::<syn::LitStr>(TokenTree::Literal(lit).into()) { self.visit_lit_str(&s); }
                        }
                        _ => {}
                    }
                }
            }
        }
        visit::visit_macro(self, node);
    }
}

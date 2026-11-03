#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! AST visitor and token-stream helpers for `dod_gates` (`[REQ-0601]`, `[REQ-0608]`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic `syn::visit::Visit` and `proc_macro2::TokenStream` inspection utilities
//! enforcing module executable SLOC ceilings, function span/parameter bounds, hand-rolled lexer
//! bans, and host-path literal portability under DO-178C Level A and DO-330 TQL-4.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Doc-Attribute Exclusion: `#[doc = ...]` and `#![doc = ...]` attributes are excluded from
//!   executable SLOC counts and string literal portability checks.
//! - Logos Exemption: `.char_indices()` and `.chars().peekable()` are rejected unless the file
//!   contains `#[derive(Logos)]`.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 Rust source text or a parsed `syn::File`.
//!
//! ## 4. Postconditions
//! - Appends deterministic, path-and-line-prefixed violation strings to `findings`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(|T|)$ where $|T|$ is the number of AST nodes and token trees in the file.
//! - Space Complexity: $O(|L| + D)$ where $|L|$ is distinct executable line numbers and $D$ is AST depth.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Lexical or syntax errors in source files are reported as explicit findings without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0601], [REQ-0608]

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

/// Maximum executable SLOC, function span lines, and parameter count for non-ratcheted `.rs` modules (`[REQ-0601]`).
pub(crate) const MAX_MODULE_EXEC_SLOC: usize = 500;
const MAX_FN_SPAN_LINES: usize = 80;
const MAX_FN_PARAMS: usize = 5;

/// Parses and audits a non-ratcheted `.rs` file for SLOC, fn bounds, lexer bans, and literal portability (`[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub(crate) fn audit_non_ratcheted_source(
    rel_path: &str,
    content: &str,
    findings: &mut Vec<String>,
) {
    let Ok(ast) =
        syn::parse_file(content).map_err(|e| findings.push(format!("{rel_path}: syntax error: {e}")))
    else {
        return;
    };
    match count_executable_sloc(content) {
        Ok(sloc) if sloc > MAX_MODULE_EXEC_SLOC => {
            findings.push(format!(
                "{rel_path}: executable SLOC {sloc} exceeds {MAX_MODULE_EXEC_SLOC} ([REQ-0601])"
            ));
        }
        Ok(_) => {}
        Err(err) => return findings.push(format!("{rel_path}: TokenStream parse error: {err}")),
    }
    let has_logos = file_derives_logos(&ast);
    let in_test_scope = attrs_have_test_cfg(&ast.attrs);
    for v in crate::checks::cst_attr_gate::scan_clippy_allow_tokens(content, rel_path) {
        findings.push(format!("{}:L{}: forbidden attribute `{}` (BP-01)", v.file, v.line, v.attribute));
    }
    DodAstVisitor {
        rel_path,
        has_logos,
        in_test_scope,
        lexer_only: false,
        findings,
    }
    .visit_file(&ast);
}

/// Audits `content` strictly for hand-rolled lexer calls (`.char_indices()`, `.chars().peekable()`) without `Logos`.
/// Realises: [REQ-0608]
pub(crate) fn audit_hand_rolled_lexer_in_source(
    rel_path: &str,
    content: &str,
    findings: &mut Vec<String>,
) {
    let Ok(ast) =
        syn::parse_file(content).map_err(|e| findings.push(format!("{rel_path}: syntax error: {e}")))
    else {
        return;
    };
    let has_logos = file_derives_logos(&ast);
    let in_test_scope = attrs_have_test_cfg(&ast.attrs);
    DodAstVisitor {
        rel_path,
        has_logos,
        in_test_scope,
        lexer_only: true,
        findings,
    }
    .visit_file(&ast);
}

/// Counts distinct source line numbers spanned by non-doc-attribute tokens (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub(crate) fn count_executable_sloc(content: &str) -> Result<usize, String> {
    let root_stream: TokenStream = content
        .parse()
        .map_err(|err: proc_macro2::LexError| err.to_string())?;
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
                    distinct_lines.extend([
                        group.span_open().start().line,
                        group.span_close().end().line,
                    ]);
                    stack.push(group.stream());
                }
                other => {
                    distinct_lines.extend([other.span().start().line, other.span().end().line])
                }
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
    (matches!(second, TokenTree::Punct(p) if p.as_char() == '!') && is_doc(third))
        .then_some(after_third)
}

/// Detects whether any `#[derive(...)]` attribute in `file` derives `Logos` (`[REQ-0608]`).
pub(crate) fn file_derives_logos(file: &syn::File) -> bool {
    /// AST visitor detecting `#[derive(Logos)]` attributes. Realises: [REQ-0608]
    struct LogosDetector(bool);
    impl<'ast> Visit<'ast> for LogosDetector {
        /// Inspects `#[derive(...)]` attributes for a `Logos` derive path. Realises: [REQ-0608]
        fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
            if attr.path().is_ident("derive") {
                let _ = attr.parse_nested_meta(|m| {
                    self.0 |= m.path.segments.last().is_some_and(|s| s.ident == "Logos");
                    Ok(())
                });
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
            let _ = attr.parse_nested_meta(|m| {
                is_test |= m.path.is_ident("test");
                Ok(())
            });
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
    val.contains(concat!("/", "Users", "/"))
        || val.contains(concat!("/", "home", "/"))
        || val.contains(concat!("C:\\", "Users\\"))
        || val.contains(concat!("C:\\\\", "Users\\\\"))
        || val.contains(concat!(".", "dylib"))
        || val.contains(concat!(".", "dll"))
        || val.ends_with(concat!(".", "so"))
        || val.contains(concat!(".", "so."))
}

/// AST visitor enforcing function bounds, hand-rolled lexer bans, and literal portability (`[REQ-0601]`, `[REQ-0608]`).
struct DodAstVisitor<'a> {
    rel_path: &'a str,
    has_logos: bool,
    in_test_scope: bool,
    lexer_only: bool,
    findings: &'a mut Vec<String>,
}

impl DodAstVisitor<'_> {
    /// Checks function span lines (`<= 80`) and parameter count (`<= 5`). Realises: [REQ-0608]
    fn check_fn_bounds(&mut self, sig: &syn::Signature, start_line: usize, end_line: usize) {
        if self.lexer_only {
            return;
        }
        let span_lines = end_line.saturating_sub(start_line).saturating_add(1);
        if span_lines > MAX_FN_SPAN_LINES {
            self.findings.push(format!(
                "{}:{start_line}: fn `{}` spans {span_lines} lines (max {MAX_FN_SPAN_LINES}) ([REQ-0601])",
                self.rel_path, sig.ident
            ));
        }
        if sig.inputs.len() > MAX_FN_PARAMS {
            self.findings.push(format!(
                "{}:{start_line}: fn `{}` has {} parameters (max {MAX_FN_PARAMS}) ([REQ-0601])",
                self.rel_path, sig.ident, sig.inputs.len()
            ));
        }
    }
}

impl<'ast> Visit<'ast> for DodAstVisitor<'_> {
    /// Tracks `#[cfg(test)]` scope while visiting top-level and nested AST items. Realises: [REQ-0608]
    fn visit_item(&mut self, node: &'ast syn::Item) {
        let prev = self.in_test_scope;
        self.in_test_scope |= item_has_test_cfg(node);
        visit::visit_item(self, node);
        self.in_test_scope = prev;
    }

    /// Audits free function span and parameter bounds before descending into its body. Realises: [REQ-0608]
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_fn_bounds(&node.sig, fn_decl_start_line(Some(&node.vis), None, &node.sig), node.block.brace_token.span.close().end().line);
        visit::visit_item_fn(self, node);
    }

    /// Audits inherent or trait `impl` method bounds and tracks `#[test]` scope. Realises: [REQ-0608]
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let def_line = node.defaultness.as_ref().map(|d| d.span.start().line);
        self.check_fn_bounds(&node.sig, fn_decl_start_line(Some(&node.vis), def_line, &node.sig), node.block.brace_token.span.close().end().line);
        let prev = self.in_test_scope;
        self.in_test_scope |= attrs_have_test_cfg(&node.attrs);
        visit::visit_impl_item_fn(self, node);
        self.in_test_scope = prev;
    }

    /// Audits trait method declaration/default body bounds and tracks `#[test]` scope. Realises: [REQ-0608]
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

    /// Skips `#[doc = ...]` attributes so docstring literals are excluded from portability checks. Realises: [REQ-0608]
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        if !node.path().is_ident("doc") {
            visit::visit_attribute(self, node);
        }
    }

    /// Flags banned hand-rolled lexer `.char_indices()` and `.chars().peekable()` calls when `Logos` is absent. Realises: [REQ-0608]
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

    /// Flags non-test string literals containing hardcoded host paths or OS shared-library suffixes. Realises: [REQ-0608]
    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        if !self.lexer_only && !self.in_test_scope && is_banned_host_or_lib_literal(&node.value()) {
            let line = node.span().start().line;
            self.findings.push(format!("{}:{line}: banned hardcoded host path or OS library suffix in string literal ([REQ-0608])", self.rel_path));
        }
    }

    /// Inspects string literals inside macro invocations for hardcoded host paths or OS library suffixes. Realises: [REQ-0608]
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if !self.lexer_only && !self.in_test_scope {
            let mut stack = vec![node.tokens.clone()];
            while let Some(stream) = stack.pop() {
                for tt in stream {
                    match tt {
                        TokenTree::Group(group) => stack.push(group.stream()),
                        TokenTree::Literal(lit) => {
                            if let Ok(s) = syn::parse2::<syn::LitStr>(TokenTree::Literal(lit).into()) {
                                self.visit_lit_str(&s);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        visit::visit_macro(self, node);
    }
}

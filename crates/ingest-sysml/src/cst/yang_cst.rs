//! # RFC 6020 / RFC 7950 YANG Lossless Concrete Syntax Tree (`logos` + `rowan`)
//!
//! **[DOC-TIER: A]**
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides a compiler-grade, lossless Concrete Syntax Tree (CST) frontend for RFC 6020 and
//! RFC 7950 YANG data models using `logos` lexical analysis and `rowan` red-green syntax trees.
//!
//! ## 2. Invariants & Mathematical Properties
//! - **Lossless Round-Trip Conservation**: Concatenating all CST leaf tokens reproduces the
//!   exact input byte stream (`lossless_roundtrip_eq == true`).
//! - **Zero `unsafe` Code**: `rowan::Language` conversion uses total `match` mapping without `transmute`.
//!
//! ## 3. Preconditions
//! - Input `source` is a valid UTF-8 slice representing a YANG module or submodule text.
//!
//! ## 4. Postconditions
//! - Returns `ParsedYangCst` containing the `rowan::GreenNode`, extracted `YangNodeSummary` records,
//!   and the verified `lossless_roundtrip_eq` conservation flag.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in input byte length `source.len()`.
//! - Space Complexity: $O(N)$ auxiliary allocation for green tree nodes and statement summaries.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unrecognized bytes are captured as `YangSyntaxKind::Error` tokens without panicking.
//!
//! ## 7. Realization & Traceability
//! - Realises: `BP-10`, `REQ-1002`, `REQ-0015`, `REQ-0201` (Official YANG `logos` + `rowan` CST Parser).

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, Language, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken};

pub use super::yang_lower::{to_camel_case, to_pascal_case, to_snake_case, YangTranslator};

/// Lexical token kinds for RFC 6020 / RFC 7950 YANG specifications. Realises: [REQ-0201]
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YangToken {
    #[regex(r"[ \t\r\n]+")]
    Whitespace,
    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment,
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", allow_greedy = true)]
    BlockComment,
    #[regex(r"module|submodule|container|list|leaf-list|leaf|grouping|uses|typedef|rpc|notification|augment|type|description|key|mandatory|config", priority = 3)]
    Keyword,
    #[regex(r"[A-Za-z_][A-Za-z0-9_\-:\.]*", priority = 2)]
    Ident,
    #[regex(r#""([^"\\]|\\(.|\n))*"|'[^']*'"#, allow_greedy = true)]
    StringLit,
    #[regex(r"-?[0-9]+(\.[0-9]+)?")]
    NumberLit,
    #[regex(r"[{};+]")]
    Punct,
    Error,
}

/// Lossless syntax node and token discriminator for `rowan`. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YangSyntaxKind {
    Root,
    Statement,
    Whitespace,
    LineComment,
    BlockComment,
    Keyword,
    Ident,
    StringLit,
    NumberLit,
    Punct,
    Error,
}

/// Safe `rowan::Language` tag for YANG CST trees. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YangLanguage {}

impl Language for YangLanguage {
    type Kind = YangSyntaxKind;

    /// Maps raw syntax kind to `YangSyntaxKind`. Realises: [REQ-0201]
    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => YangSyntaxKind::Root,
            1 => YangSyntaxKind::Statement,
            2 => YangSyntaxKind::Whitespace,
            3 => YangSyntaxKind::LineComment,
            4 => YangSyntaxKind::BlockComment,
            5 => YangSyntaxKind::Keyword,
            6 => YangSyntaxKind::Ident,
            7 => YangSyntaxKind::StringLit,
            8 => YangSyntaxKind::NumberLit,
            9 => YangSyntaxKind::Punct,
            _ => YangSyntaxKind::Error,
        }
    }

    /// Maps `YangSyntaxKind` to raw syntax kind. Realises: [REQ-0201]
    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
        SyntaxKind(match kind {
            YangSyntaxKind::Root => 0,
            YangSyntaxKind::Statement => 1,
            YangSyntaxKind::Whitespace => 2,
            YangSyntaxKind::LineComment => 3,
            YangSyntaxKind::BlockComment => 4,
            YangSyntaxKind::Keyword => 5,
            YangSyntaxKind::Ident => 6,
            YangSyntaxKind::StringLit => 7,
            YangSyntaxKind::NumberLit => 8,
            YangSyntaxKind::Punct => 9,
            YangSyntaxKind::Error => 10,
        })
    }
}

/// Extracted summary of a single RFC 7950 YANG statement node. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YangNodeSummary {
    pub keyword: String,
    pub argument: String,
    pub span_start: usize,
    pub span_end: usize,
}

/// Result of parsing a YANG specification into a lossless `rowan` CST. Realises: [REQ-0201]
#[derive(Debug, Clone)]
pub struct ParsedYangCst {
    pub green: GreenNode,
    pub statements: Vec<YangNodeSummary>,
    pub lossless_roundtrip_eq: bool,
}

/// Maps a lexical `YangToken` into its corresponding `YangSyntaxKind`. Realises: [REQ-0201]
const fn token_syntax_kind(tok: YangToken) -> YangSyntaxKind {
    match tok {
        YangToken::Whitespace => YangSyntaxKind::Whitespace,
        YangToken::LineComment => YangSyntaxKind::LineComment,
        YangToken::BlockComment => YangSyntaxKind::BlockComment,
        YangToken::Keyword => YangSyntaxKind::Keyword,
        YangToken::Ident => YangSyntaxKind::Ident,
        YangToken::StringLit => YangSyntaxKind::StringLit,
        YangToken::NumberLit => YangSyntaxKind::NumberLit,
        YangToken::Punct => YangSyntaxKind::Punct,
        YangToken::Error => YangSyntaxKind::Error,
    }
}

/// Emits a statement delimiter (`;`, `{`, `}`) into the `rowan` CST builder while maintaining statement nesting. Realises: [REQ-0201]
fn emit_cst_punct(
    builder: &mut GreenNodeBuilder<'static>,
    slice: &str,
    raw_kind: SyntaxKind,
    in_stmt_header: &mut bool,
    open_blocks: &mut usize,
) {
    match slice {
        ";" => {
            if !*in_stmt_header {
                builder.start_node(YangLanguage::kind_to_raw(YangSyntaxKind::Statement));
            }
            builder.token(raw_kind, slice);
            builder.finish_node();
            *in_stmt_header = false;
        }
        "{" => {
            if !*in_stmt_header {
                builder.start_node(YangLanguage::kind_to_raw(YangSyntaxKind::Statement));
            }
            builder.token(raw_kind, slice);
            *open_blocks += 1;
            *in_stmt_header = false;
        }
        _ => {
            if *in_stmt_header {
                builder.finish_node();
                *in_stmt_header = false;
            }
            builder.token(raw_kind, slice);
            if *open_blocks > 0 {
                *open_blocks -= 1;
                builder.finish_node();
            }
        }
    }
}

/// Parses a YANG source string into a lossless `rowan` CST and statement summary list. Realises: [REQ-0201]
#[must_use]
pub fn parse_yang_cst(source: &str) -> ParsedYangCst {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(YangLanguage::kind_to_raw(YangSyntaxKind::Root));
    let mut statements = Vec::new();
    let mut pending: Option<(String, String, usize)> = None;
    let mut at_stmt_start = true;
    let mut in_stmt_header = false;
    let mut open_blocks = 0usize;

    for (res, span) in YangToken::lexer(source).spanned() {
        let Some(slice) = source.get(span.clone()) else { continue };
        let tok = res.unwrap_or(YangToken::Error);
        let raw_kind = YangLanguage::kind_to_raw(token_syntax_kind(tok));
        if matches!(tok, YangToken::Whitespace | YangToken::LineComment | YangToken::BlockComment) {
            builder.token(raw_kind, slice);
            continue;
        }
        if tok == YangToken::Punct && matches!(slice, "{" | "}" | ";") {
            if let Some((keyword, argument, span_start)) = pending.take() {
                statements.push(YangNodeSummary { keyword, argument, span_start, span_end: span.end });
            }
            at_stmt_start = true;
            emit_cst_punct(&mut builder, slice, raw_kind, &mut in_stmt_header, &mut open_blocks);
            continue;
        }
        if !in_stmt_header {
            builder.start_node(YangLanguage::kind_to_raw(YangSyntaxKind::Statement));
            in_stmt_header = true;
        }
        builder.token(raw_kind, slice);
        if at_stmt_start {
            at_stmt_start = false;
            if tok == YangToken::Keyword {
                pending = Some((slice.to_string(), String::new(), span.start));
            }
        } else if let Some((_, ref mut arg, _)) = pending {
            if arg.is_empty() && tok != YangToken::Punct {
                *arg = slice.trim_matches('"').trim_matches('\'').to_string();
            }
        }
    }
    if let Some((keyword, argument, span_start)) = pending {
        statements.push(YangNodeSummary { keyword, argument, span_start, span_end: source.len() });
    }
    if in_stmt_header {
        builder.finish_node();
    }
    while open_blocks > 0 {
        open_blocks -= 1;
        builder.finish_node();
    }
    builder.finish_node();
    let green = builder.finish();
    let root: SyntaxNode<YangLanguage> = SyntaxNode::new_root(green.clone());
    let lossless_roundtrip_eq = root.text() == source;
    ParsedYangCst { green, statements, lossless_roundtrip_eq }
}

/// Intermediate structural AST representation of a parsed YANG statement. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT]
#[derive(Debug, Clone)]
pub struct YangStmt {
    pub keyword: String,
    pub argument: Option<String>,
    pub substatements: Vec<Self>,
}

impl YangStmt {
    /// Returns the argument of the first substatement matching `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-GET-ARG]
    #[must_use]
    pub fn get_arg(&self, kw: &str) -> Option<&str> {
        self.substatements.iter().find(|s| s.keyword == kw).and_then(|s| s.argument.as_deref())
    }
    /// Returns the arguments of all substatements matching `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-GET-ALL-ARGS]
    #[must_use]
    pub fn get_all_args(&self, kw: &str) -> Vec<&str> {
        self.substatements.iter().filter(|s| s.keyword == kw).filter_map(|s| s.argument.as_deref()).collect()
    }
    /// Finds the first substatement with keyword `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-FIND-FIRST]
    #[must_use]
    pub fn find_first(&self, kw: &str) -> Option<&Self> {
        self.substatements.iter().find(|s| s.keyword == kw)
    }
    /// Finds all substatements with keyword `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-FIND-ALL]
    #[must_use]
    pub fn find_all(&self, kw: &str) -> Vec<&Self> {
        self.substatements.iter().filter(|s| s.keyword == kw).collect()
    }
}

/// Unescapes a single-quoted or double-quoted YANG string literal token from the CST. Realises: [REQ-0015], [REQ-0201]
fn unescape_yang_string(raw: &str) -> String {
    if let Some(inner) = raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return inner.to_string();
    }
    let inner = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(raw);
    let (mut out, mut chars) = (String::with_capacity(inner.len()), inner.chars());
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(c) => out.push(c),
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Coalesces non-trivia CST header tokens (before `;` or `{`) into `[keyword, optional_arg]`. Realises: [REQ-0015], [REQ-0201]
fn coalesce_header_words(tokens: &[SyntaxToken<YangLanguage>]) -> Result<Vec<String>, String> {
    let (mut words, mut prev_end, mut idx) = (Vec::<String>::new(), None, 0usize);
    while let Some(tok) = tokens.get(idx) {
        idx += 1;
        if tok.kind() == YangSyntaxKind::Error && (tok.text().starts_with('"') || tok.text().starts_with('\'')) {
            return Err("Unterminated quoted string in YANG statement".to_string());
        }
        if tok.kind() == YangSyntaxKind::StringLit {
            let mut s = unescape_yang_string(tok.text());
            while tokens.get(idx).is_some_and(|t| t.kind() == YangSyntaxKind::Punct && t.text() == "+") {
                idx += 1;
                let Some(next_str) = tokens.get(idx).filter(|t| t.kind() == YangSyntaxKind::StringLit) else {
                    return Err("Expected string after '+' concatenation operator in YANG".to_string());
                };
                s.push_str(&unescape_yang_string(next_str.text()));
                idx += 1;
            }
            words.push(s);
            prev_end = None;
        } else {
            let (start, end) = (usize::from(tok.text_range().start()), usize::from(tok.text_range().end()));
            if prev_end == Some(start) {
                if let Some(last) = words.last_mut() {
                    last.push_str(tok.text());
                }
            } else {
                words.push(tok.text().to_string());
            }
            prev_end = Some(end);
        }
    }
    Ok(words)
}

/// Lowers a single `rowan` CST `Statement` node into a recursive `YangStmt` AST node. Realises: [REQ-0015], [REQ-0201]
fn lower_cst_statement(node: &SyntaxNode<YangLanguage>) -> Result<YangStmt, String> {
    let direct_tokens: Vec<SyntaxToken<YangLanguage>> = node
        .children_with_tokens()
        .filter_map(NodeOrToken::into_token)
        .filter(|t| !matches!(t.kind(), YangSyntaxKind::Whitespace | YangSyntaxKind::LineComment | YangSyntaxKind::BlockComment))
        .collect();
    let header_slice = match direct_tokens.as_slice() {
        [head @ .., last] if last.kind() == YangSyntaxKind::Punct && last.text() == ";" => head,
        [head @ .., open, close] if open.kind() == YangSyntaxKind::Punct && open.text() == "{" && close.kind() == YangSyntaxKind::Punct && close.text() == "}" => head,
        _ if direct_tokens.iter().any(|t| t.kind() == YangSyntaxKind::Punct && t.text() == "{") => return Err("Unclosed YANG block: missing '}'".to_string()),
        _ => return Err("Unexpected end of tokens in YANG statement".to_string()),
    };
    let mut words = coalesce_header_words(header_slice)?;
    let (keyword, argument) = match words.len() {
        1 => (words.remove(0), None),
        2 => {
            let arg = words.pop();
            (words.remove(0), arg)
        }
        0 => return Err("Expected statement keyword in YANG statement".to_string()),
        _ => return Err(format!("Expected ';' or '{{' after argument for '{}'", words.first().map_or("", String::as_str))),
    };
    let substatements = node.children().filter(|c| c.kind() == YangSyntaxKind::Statement).map(|c| lower_cst_statement(&c)).collect::<Result<Vec<_>, _>>()?;
    Ok(YangStmt { keyword, argument, substatements })
}

/// Parses a YANG source string into top-level `YangStmt` trees via `parse_yang_cst`. Realises: [REQ-TRANSLATORS-YANG-PARSE-YANG-STATEMENTS]
///
/// # Errors
/// Returns an `Err` if the YANG source contains unterminated strings, unclosed blocks, or malformed statements.
pub fn parse_yang_statements(source: &str) -> Result<Vec<YangStmt>, String> {
    let cst = parse_yang_cst(source);
    let root: SyntaxNode<YangLanguage> = SyntaxNode::new_root(cst.green);
    if root.children_with_tokens().filter_map(NodeOrToken::into_token).any(|t| !matches!(t.kind(), YangSyntaxKind::Whitespace | YangSyntaxKind::LineComment | YangSyntaxKind::BlockComment)) {
        return Err("Unexpected top-level token outside YANG statement".to_string());
    }
    root.children().filter(|n| n.kind() == YangSyntaxKind::Statement).map(|n| lower_cst_statement(&n)).collect()
}

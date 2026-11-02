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
//! - Realises: `BP-10`, `REQ-1002` (Official YANG `logos` + `rowan` CST Parser).

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, Language, SyntaxKind, SyntaxNode};

/// Lexical token kinds for RFC 6020 / RFC 7950 YANG specifications. Realises: [REQ-0201]
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YangToken {
    #[regex(r"[ \t\r\n]+")]
    Whitespace,
    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment,
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", allow_greedy = true)]
    BlockComment,
    #[regex(
        r"module|submodule|container|list|leaf-list|leaf|grouping|uses|typedef|rpc|notification|augment|type|description|key|mandatory|config",
        priority = 3
    )]
    Keyword,
    #[regex(r"[A-Za-z_][A-Za-z0-9_\-:\.]*", priority = 2)]
    Ident,
    #[regex(r#""([^"\\]|\\.)*"|'[^']*'"#, allow_greedy = true)]
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

/// Parses a YANG source string into a lossless `rowan` CST and statement summary list. Realises: [REQ-0201]
#[must_use]
pub fn parse_yang_cst(source: &str) -> ParsedYangCst {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(YangLanguage::kind_to_raw(YangSyntaxKind::Root));
    let mut statements = Vec::new();
    let mut pending: Option<(String, String, usize)> = None;
    let mut at_stmt_start = true;

    for (res, span) in YangToken::lexer(source).spanned() {
        let slice = &source[span.clone()];
        let tok = res.unwrap_or(YangToken::Error);
        let kind = match tok {
            YangToken::Whitespace => YangSyntaxKind::Whitespace,
            YangToken::LineComment => YangSyntaxKind::LineComment,
            YangToken::BlockComment => YangSyntaxKind::BlockComment,
            YangToken::Keyword => YangSyntaxKind::Keyword,
            YangToken::Ident => YangSyntaxKind::Ident,
            YangToken::StringLit => YangSyntaxKind::StringLit,
            YangToken::NumberLit => YangSyntaxKind::NumberLit,
            YangToken::Punct => YangSyntaxKind::Punct,
            YangToken::Error => YangSyntaxKind::Error,
        };
        builder.token(YangLanguage::kind_to_raw(kind), slice);
        if matches!(tok, YangToken::Whitespace | YangToken::LineComment | YangToken::BlockComment) {
            continue;
        }
        if tok == YangToken::Punct && matches!(slice, "{" | "}" | ";") {
            if let Some((keyword, argument, span_start)) = pending.take() {
                statements.push(YangNodeSummary { keyword, argument, span_start, span_end: span.end });
            }
            at_stmt_start = true;
        } else if at_stmt_start {
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
    builder.finish_node();
    let green = builder.finish();
    let root: SyntaxNode<YangLanguage> = SyntaxNode::new_root(green.clone());
    let lossless_roundtrip_eq = root.text() == source;
    ParsedYangCst { green, statements, lossless_roundtrip_eq }
}

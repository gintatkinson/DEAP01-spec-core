//! [DOC-TIER: A]
//! 1. Purpose: IDL concrete syntax tree (CST) parser with lossless roundtripping.
//! 2. Scope: Concrete syntax tokenization and CST construction for OMG IDL v4.
//! 3. Invariants: Lossless source reconstruction across all valid or invalid token sequences.
//! 4. Concurrency: Stateless pure parser; all data structures are `Send` + `Sync`.
//! 5. Error Handling: Error resilient parsing collecting unknown tokens without panics.
//! 6. Security & Safety: Pure safe Rust, zero unsafe blocks, bounded recursion.
//! 7. Cross-Platform: Standard CRLF/LF normalized lexing, cross-platform portable.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

/// IDL lexer tokens. Realises: [REQ-0201]
#[derive(logos::Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdlToken {
    #[regex(r"[ \t\r\n]+")]
    Whitespace,
    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment,
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", allow_greedy = true)]
    BlockComment,
    #[regex(r"module|interface|struct|enum|typedef|const|exception|sequence|attribute|readonly|in|out|inout")]
    Keyword,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", priority = 1)]
    Ident,
    #[regex(r"[0-9]+(\.[0-9]+)?")]
    NumberLit,
    #[regex(r#""([^"\\]|\\.)*""#, allow_greedy = true)]
    StringLit,
    #[regex(r"::|[{}();,<>:=\[\]+*/%-]")]
    Punct,
    Error,
}

/// IDL syntax kind for Rowan CST. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum IdlSyntaxKind {
    Whitespace = 0,
    LineComment,
    BlockComment,
    Keyword,
    Ident,
    NumberLit,
    StringLit,
    Punct,
    ErrorToken,
    SourceFile,
}

/// IDL Rowan language definition. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdlLanguage {}

impl rowan::Language for IdlLanguage {
    type Kind = IdlSyntaxKind;

    /// Maps raw syntax kind to `IdlSyntaxKind`. Realises: [REQ-0201]
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => IdlSyntaxKind::Whitespace,
            1 => IdlSyntaxKind::LineComment,
            2 => IdlSyntaxKind::BlockComment,
            3 => IdlSyntaxKind::Keyword,
            4 => IdlSyntaxKind::Ident,
            5 => IdlSyntaxKind::NumberLit,
            6 => IdlSyntaxKind::StringLit,
            7 => IdlSyntaxKind::Punct,
            9 => IdlSyntaxKind::SourceFile,
            _ => IdlSyntaxKind::ErrorToken,
        }
    }

    /// Maps `IdlSyntaxKind` to raw syntax kind. Realises: [REQ-0201]
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        match kind {
            IdlSyntaxKind::Whitespace => rowan::SyntaxKind(0),
            IdlSyntaxKind::LineComment => rowan::SyntaxKind(1),
            IdlSyntaxKind::BlockComment => rowan::SyntaxKind(2),
            IdlSyntaxKind::Keyword => rowan::SyntaxKind(3),
            IdlSyntaxKind::Ident => rowan::SyntaxKind(4),
            IdlSyntaxKind::NumberLit => rowan::SyntaxKind(5),
            IdlSyntaxKind::StringLit => rowan::SyntaxKind(6),
            IdlSyntaxKind::Punct => rowan::SyntaxKind(7),
            IdlSyntaxKind::ErrorToken => rowan::SyntaxKind(8),
            IdlSyntaxKind::SourceFile => rowan::SyntaxKind(9),
        }
    }
}

/// Summary of an IDL construct. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdlNodeSummary {
    pub kind: String,
    pub name: String,
    pub span_start: usize,
    pub span_end: usize,
}

/// Parsed IDL CST output. Realises: [REQ-0201]
#[derive(Debug, Clone)]
pub struct ParsedIdlCst {
    pub green: rowan::GreenNode,
    pub nodes: Vec<IdlNodeSummary>,
    pub lossless_roundtrip_eq: bool,
}

const TARGET_KEYWORDS: &[&str] = &[
    "module", "interface", "struct", "enum", "typedef", "const", "exception", "attribute",
];

/// Maps token to syntax kind. Realises: [REQ-0201]
const fn token_to_syntax_kind(token: IdlToken) -> IdlSyntaxKind {
    match token {
        IdlToken::Whitespace => IdlSyntaxKind::Whitespace,
        IdlToken::LineComment => IdlSyntaxKind::LineComment,
        IdlToken::BlockComment => IdlSyntaxKind::BlockComment,
        IdlToken::Keyword => IdlSyntaxKind::Keyword,
        IdlToken::Ident => IdlSyntaxKind::Ident,
        IdlToken::NumberLit => IdlSyntaxKind::NumberLit,
        IdlToken::StringLit => IdlSyntaxKind::StringLit,
        IdlToken::Punct => IdlSyntaxKind::Punct,
        IdlToken::Error => IdlSyntaxKind::ErrorToken,
    }
}

/// Parses IDL source into lossless CST. Realises: [REQ-0201]
#[must_use]
pub fn parse_idl_cst(source: &str) -> ParsedIdlCst {
    let mut builder = rowan::GreenNodeBuilder::new();
    builder.start_node(<IdlLanguage as rowan::Language>::kind_to_raw(IdlSyntaxKind::SourceFile));

    let mut nodes = Vec::new();
    let mut pending_kw: Option<(String, usize)> = None;

    for (token_res, span) in logos::Logos::lexer(source).spanned() {
        let token = token_res.unwrap_or(IdlToken::Error);
        let syntax_kind = token_to_syntax_kind(token);
        let slice = source.get(span.clone()).unwrap_or("");
        builder.token(<IdlLanguage as rowan::Language>::kind_to_raw(syntax_kind), slice);

        match token {
            IdlToken::Keyword if TARGET_KEYWORDS.contains(&slice) => {
                pending_kw = Some((slice.to_owned(), span.start));
            }
            IdlToken::Ident => {
                if let Some((kw_kind, kw_start)) = pending_kw.take() {
                    nodes.push(IdlNodeSummary {
                        kind: kw_kind,
                        name: slice.to_owned(),
                        span_start: kw_start,
                        span_end: span.end,
                    });
                }
            }
            IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment => {}
            _ => {
                pending_kw = None;
            }
        }
    }

    builder.finish_node();
    let green = builder.finish();
    let root = rowan::SyntaxNode::<IdlLanguage>::new_root(green.clone());
    let lossless_roundtrip_eq = root.text() == source;

    ParsedIdlCst {
        green,
        nodes,
        lossless_roundtrip_eq,
    }
}

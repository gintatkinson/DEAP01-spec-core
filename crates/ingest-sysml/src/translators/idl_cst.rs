//! [DOC-TIER: A]
//! 1. Purpose: IDL concrete syntax tree (CST) parser with lossless roundtripping.
//! 2. Scope: Concrete syntax tokenization and CST construction for OMG IDL v4.
//! 3. Invariants: Lossless source reconstruction across all valid or invalid token sequences.
//! 4. Concurrency: Stateless pure parser; all data structures are `Send` + `Sync`.
//! 5. Error Handling: Error resilient parsing collecting unknown tokens without panics.
//! 6. Security & Safety: Pure safe Rust, zero unsafe blocks, bounded recursion.
//! 7. Cross-Platform: Standard CRLF/LF normalized lexing, cross-platform portable.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum IdlSyntaxKind {
    WHITESPACE = 0,
    LINE_COMMENT,
    BLOCK_COMMENT,
    KEYWORD,
    IDENT,
    NUMBER_LIT,
    STRING_LIT,
    PUNCT,
    ERROR_TOKEN,
    SOURCE_FILE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdlLanguage {}

impl rowan::Language for IdlLanguage {
    type Kind = IdlSyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => IdlSyntaxKind::WHITESPACE,
            1 => IdlSyntaxKind::LINE_COMMENT,
            2 => IdlSyntaxKind::BLOCK_COMMENT,
            3 => IdlSyntaxKind::KEYWORD,
            4 => IdlSyntaxKind::IDENT,
            5 => IdlSyntaxKind::NUMBER_LIT,
            6 => IdlSyntaxKind::STRING_LIT,
            7 => IdlSyntaxKind::PUNCT,
            9 => IdlSyntaxKind::SOURCE_FILE,
            _ => IdlSyntaxKind::ERROR_TOKEN,
        }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        match kind {
            IdlSyntaxKind::WHITESPACE => rowan::SyntaxKind(0),
            IdlSyntaxKind::LINE_COMMENT => rowan::SyntaxKind(1),
            IdlSyntaxKind::BLOCK_COMMENT => rowan::SyntaxKind(2),
            IdlSyntaxKind::KEYWORD => rowan::SyntaxKind(3),
            IdlSyntaxKind::IDENT => rowan::SyntaxKind(4),
            IdlSyntaxKind::NUMBER_LIT => rowan::SyntaxKind(5),
            IdlSyntaxKind::STRING_LIT => rowan::SyntaxKind(6),
            IdlSyntaxKind::PUNCT => rowan::SyntaxKind(7),
            IdlSyntaxKind::ERROR_TOKEN => rowan::SyntaxKind(8),
            IdlSyntaxKind::SOURCE_FILE => rowan::SyntaxKind(9),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdlNodeSummary {
    pub kind: String,
    pub name: String,
    pub span_start: usize,
    pub span_end: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedIdlCst {
    pub green: rowan::GreenNode,
    pub nodes: Vec<IdlNodeSummary>,
    pub lossless_roundtrip_eq: bool,
}

const TARGET_KEYWORDS: &[&str] = &[
    "module", "interface", "struct", "enum", "typedef", "const", "exception", "attribute",
];

const fn token_to_syntax_kind(token: IdlToken) -> IdlSyntaxKind {
    match token {
        IdlToken::Whitespace => IdlSyntaxKind::WHITESPACE,
        IdlToken::LineComment => IdlSyntaxKind::LINE_COMMENT,
        IdlToken::BlockComment => IdlSyntaxKind::BLOCK_COMMENT,
        IdlToken::Keyword => IdlSyntaxKind::KEYWORD,
        IdlToken::Ident => IdlSyntaxKind::IDENT,
        IdlToken::NumberLit => IdlSyntaxKind::NUMBER_LIT,
        IdlToken::StringLit => IdlSyntaxKind::STRING_LIT,
        IdlToken::Punct => IdlSyntaxKind::PUNCT,
        IdlToken::Error => IdlSyntaxKind::ERROR_TOKEN,
    }
}

#[must_use]
pub fn parse_idl_cst(source: &str) -> ParsedIdlCst {
    let mut builder = rowan::GreenNodeBuilder::new();
    builder.start_node(<IdlLanguage as rowan::Language>::kind_to_raw(IdlSyntaxKind::SOURCE_FILE));

    let mut nodes = Vec::new();
    let mut pending_kw: Option<(String, usize)> = None;

    for (token_res, span) in logos::Logos::lexer(source).spanned() {
        let token = token_res.unwrap_or(IdlToken::Error);
        let syntax_kind = token_to_syntax_kind(token);
        let slice = &source[span.clone()];
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

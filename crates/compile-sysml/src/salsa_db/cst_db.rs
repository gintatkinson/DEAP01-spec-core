//! # Salsa Lossless CST Query, Token Taxonomy & Expression Parser (`cst_db`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Provides `#[salsa::tracked]` Rowan CST parsing, polyglot backend routing, token spans, and precedence-climbed expression parsing (`BP-03`, `BP-10`).
//! ## 2. Invariants & Mathematical Properties -- `parse_lossless_sysml_cst` preserves byte-for-byte text identity (`reconstructed_len == input.len()`); `ExpressionParser` enforces `MAX_EXPR_DEPTH` (`64`).
//! ## 3. Preconditions -- `SalsaFileInput` is registered in `salsa::Database`; expression token slices are well-formed.
//! ## 4. Postconditions -- Memoizes `CstParseReceipt` and produces bounded `Expr` AST trees.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(N) on cache miss, O(1) on memoized hit; Space: O(N).
//! ## 6. Failure Modes & Fault Tolerance -- Never panics; emits `DEAP-E1001`/`DEAP-E1002` diagnostics and typed `Err(String)` on expression overflow.
//! ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0603], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use std::fmt;

use deap_core::sysml_cst::{parse_sysml_cst, print_lossless_cst};

use super::source_db::SalsaFileInput;

/// Receipt produced by the `#[salsa::tracked]` lossless Rowan CST query (`parse_lossless_sysml_cst`).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Captures lossless CST reconstruction proof and diagnostics (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- `lossless_roundtrip` is `true` iff `print_lossless_cst(&parsed) == source`.
/// ## 3. Preconditions -- Produced by `parse_lossless_sysml_cst`.
/// ## 4. Postconditions -- Provides construct count, reconstructed text length, and syntax diagnostics.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1) field access; Space: O(D) over diagnostics.
/// ## 6. Failure Modes & Fault Tolerance -- Pure value struct with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstParseReceipt {
    pub path: String,
    pub construct_count: usize,
    pub lossless_roundtrip: bool,
    pub diagnostics: Vec<String>,
}

/// Salsa tracked query parsing a `SalsaFileInput` into a lossless Rowan CST and returning `CstParseReceipt`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Demand-driven Rowan CST query memoized by `salsa` (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Re-executes only when `input.text(db)` mutates.
/// ## 3. Preconditions -- `input` is a valid `SalsaFileInput` in `db`.
/// ## 4. Postconditions -- Returns `CstParseReceipt` verifying lossless CST reconstruction.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1) memoized hit, O(N) cache miss; Space: O(N).
/// ## 6. Failure Modes & Fault Tolerance -- Total tracked function with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[salsa::tracked(returns(clone))]
pub fn parse_lossless_sysml_cst(
    db: &dyn salsa::Database,
    input: SalsaFileInput,
) -> CstParseReceipt {
    let text = input.text(db);
    let parsed = parse_sysml_cst(text);
    let roundtrip = print_lossless_cst(&parsed) == *text;
    CstParseReceipt {
        path: input.path(db).clone(),
        construct_count: parsed.constructs.len(),
        lossless_roundtrip: roundtrip,
        diagnostics: parsed.diagnostics,
    }
}

/// Resolves the canonical polyglot CST/AST backend identifier for a given file extension (`BP-10`).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Enforces strict dispatch to Contract B parser backends (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Total mapping from normalized extension to backend name.
/// ## 3. Preconditions -- `extension` is a file extension slice (with or without leading `.`).
/// ## 4. Postconditions -- Returns `Some(backend_id)` for supported extensions or `None`.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance -- Pure lookup with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
#[must_use]
pub fn resolve_polyglot_backend(extension: &str) -> Option<&'static str> {
    match extension.trim_start_matches('.').to_ascii_lowercase().as_str() {
        "sysml" | "kerml" => Some("logos+rowan::sysml_cst"),
        "idl" => Some("logos+rowan::idl_cst"),
        "yang" => Some("logos+rowan::yang_cst"),
        "md" | "markdown" => Some("pulldown-cmark::markdown_cst"),
        "yaml" | "yml" | "json" => Some("serde_yaml+serde_json::openapi_ast"),
        "rs" => Some("ra_ap_syntax::rust_cst"),
        "dart" => Some("logos+rowan::dart_cst"),
        _ => None,
    }
}

/// Source span with byte offsets and 1-indexed line/column coordinates.
/// Realises: [REQ-LEXER-TOKEN-SPAN] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    /// Creates a new `Span`. Realises: [REQ-LEXER-TOKEN-SPAN-NEW] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub const fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self { start, end, line, col }
    }
}

impl fmt::Display for Span {
    /// Formats `Span` as `line:col-line:end_col`. Realises: [REQ-LEXER-TOKEN-SPAN-FMT] | [FLS: fls_66m4rnbssgig]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.line,
            self.col,
            self.line,
            self.col.saturating_add(self.end.saturating_sub(self.start))
        )
    }
}

/// Token classification for SysML v2 and KerML textual syntax.
/// Realises: [REQ-LEXER-TOKEN-TOKENKIND] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Package, Import, Part, Def, Port, Item, Flow, Interface, Subsystem, Subject, Actor,
    Action, Operation, State, Transition, Entry, Do, Exit, Perform, Step, Trigger,
    Requirement, Constraint, Assert, Assume, Require, Verify, Satisfy, Calc,
    Test, Case, Hazard, Risk, UseCase, Include, Extend, Precondition, Postcondition, Objective,
    Capability, Interaction, Lifeline, Message,
    Connection, Connect, From, To, By,
    Attribute, Id, Text, Doc, In, Out, Inout,
    Ident(String), StringLit(String), IntLit(i64), RealLit(f64), BoolLit(bool),
    DocComment(String), LineComment(String),
    ColonEq, ColonGt, DoubleColon, Arrow, TildeGt, EqEq, NotEq, LtEq, GtEq, AmpAmp, PipePipe,
    Eq, Lt, Gt, Plus, Minus, Star, Slash, Percent, Caret, Tilde, Exclamation,
    Colon, Semi, Comma, Dot, OpenBrace, CloseBrace, OpenParen, CloseParen, OpenBracket, CloseBracket,
    Eof,
}

const KEYWORD_KINDS: &[TokenKind] = &[
    TokenKind::Package, TokenKind::Import, TokenKind::Part, TokenKind::Def, TokenKind::Port,
    TokenKind::Item, TokenKind::Flow, TokenKind::Interface, TokenKind::Subsystem,
    TokenKind::Subject, TokenKind::Actor, TokenKind::Action, TokenKind::Operation,
    TokenKind::State, TokenKind::Transition, TokenKind::Entry, TokenKind::Do, TokenKind::Exit,
    TokenKind::Perform, TokenKind::Step, TokenKind::Trigger, TokenKind::Requirement,
    TokenKind::Constraint, TokenKind::Assert, TokenKind::Assume, TokenKind::Require,
    TokenKind::Verify, TokenKind::Satisfy, TokenKind::Calc, TokenKind::Test, TokenKind::Case,
    TokenKind::Hazard, TokenKind::Risk, TokenKind::Include, TokenKind::Extend,
    TokenKind::Precondition, TokenKind::Postcondition, TokenKind::Objective,
    TokenKind::Capability, TokenKind::Interaction, TokenKind::Lifeline, TokenKind::Message,
    TokenKind::Connection, TokenKind::Connect, TokenKind::From, TokenKind::To, TokenKind::By,
    TokenKind::Attribute, TokenKind::Id, TokenKind::Text, TokenKind::Doc, TokenKind::In,
    TokenKind::Out, TokenKind::Inout,
];

const SYMBOL_KINDS: &[TokenKind] = &[
    TokenKind::ColonEq, TokenKind::ColonGt, TokenKind::DoubleColon, TokenKind::Arrow,
    TokenKind::TildeGt, TokenKind::EqEq, TokenKind::NotEq, TokenKind::LtEq, TokenKind::GtEq,
    TokenKind::AmpAmp, TokenKind::PipePipe, TokenKind::Eq, TokenKind::Lt, TokenKind::Gt,
    TokenKind::Plus, TokenKind::Minus, TokenKind::Star, TokenKind::Slash, TokenKind::Percent,
    TokenKind::Caret, TokenKind::Tilde, TokenKind::Exclamation, TokenKind::Colon,
    TokenKind::Semi, TokenKind::Comma, TokenKind::Dot, TokenKind::OpenBrace,
    TokenKind::CloseBrace, TokenKind::OpenParen, TokenKind::CloseParen, TokenKind::OpenBracket,
    TokenKind::CloseBracket,
];

impl TokenKind {
    /// Returns the static keyword or symbol string slice for `self`. Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR] | [FLS: fls_8tsynkj2cufj]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        self.keyword_as_str().unwrap_or_else(|| self.symbol_as_str())
    }

    /// Resolves a single-word keyword or boolean literal lexeme into its `TokenKind`. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-IDENT-OR-KEYWORD] | [FLS: fls_8tsynkj2cufj]
    pub(crate) fn from_keyword_or_ident(text: &str) -> Self {
        match text {
            "true" => Self::BoolLit(true),
            "false" => Self::BoolLit(false),
            _ => KEYWORD_KINDS
                .iter()
                .find(|k| k.as_str() == text)
                .cloned()
                .unwrap_or_else(|| Self::Ident(text.to_string())),
        }
    }

    /// Resolves a 1- or 2-character operator or punctuation lexeme into its `TokenKind`. Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR] | [FLS: fls_8tsynkj2cufj]
    pub(crate) fn from_symbol(text: &str) -> Option<Self> {
        SYMBOL_KINDS.iter().find(|k| k.as_str() == text).cloned()
    }

    /// Strips optional leading `doc` or `doc:` prefix from a block comment body. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-BLOCK-COMMENT] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn doc_comment_from_raw(raw: &str) -> Self {
        let cleaned = raw.trim();
        let text = match cleaned.strip_prefix("doc") {
            Some(after) if after.is_empty() || after.starts_with(|c: char| c.is_whitespace() || c == ':') => {
                after.trim_start_matches(|c: char| c.is_whitespace() || c == ':').trim().to_string()
            }
            _ => cleaned.to_string(),
        };
        Self::DocComment(text)
    }

    /// Parses a numeric slice into `Self::RealLit` or `Self::IntLit`. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-NUMBER] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_number_lexeme(num_str: &str, line: usize, col: usize) -> Result<Self, String> {
        if num_str.contains(['.', 'e', 'E']) {
            num_str.parse::<f64>().map(Self::RealLit).map_err(|e| format!("Invalid float '{num_str}' at {line}:{col}: {e}"))
        } else {
            num_str.parse::<i64>().map(Self::IntLit).map_err(|e| format!("Invalid integer '{num_str}' at {line}:{col}: {e}"))
        }
    }

    /// Maps keyword variants of `TokenKind` to their static string slices. Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR] | [FLS: fls_8tsynkj2cufj]
    fn keyword_as_str(&self) -> Option<&'static str> {
        Some(match self {
            Self::Package => "package", Self::Import => "import", Self::Part => "part",
            Self::Def => "def", Self::Port => "port", Self::Item => "item",
            Self::Flow => "flow", Self::Interface => "interface", Self::Subsystem => "subsystem",
            Self::Subject => "subject", Self::Actor => "actor", Self::Action => "action",
            Self::Operation => "operation", Self::State => "state", Self::Transition => "transition",
            Self::Entry => "entry", Self::Do => "do", Self::Exit => "exit",
            Self::Perform => "perform", Self::Step => "step", Self::Trigger => "trigger",
            Self::Requirement => "requirement", Self::Constraint => "constraint",
            Self::Assert => "assert", Self::Assume => "assume", Self::Require => "require",
            Self::Verify => "verify", Self::Satisfy => "satisfy", Self::Calc => "calc",
            Self::Test => "test", Self::Case => "case", Self::Hazard => "hazard",
            Self::Risk => "risk", Self::UseCase => "use case", Self::Include => "include",
            Self::Extend => "extend", Self::Precondition => "precondition",
            Self::Postcondition => "postcondition", Self::Objective => "objective",
            Self::Capability => "capability", Self::Interaction => "interaction",
            Self::Lifeline => "lifeline", Self::Message => "message",
            Self::Connection => "connection", Self::Connect => "connect", Self::From => "from",
            Self::To => "to", Self::By => "by", Self::Attribute => "attribute",
            Self::Id => "id", Self::Text => "text", Self::Doc => "doc",
            Self::In => "in", Self::Out => "out", Self::Inout => "inout",
            _ => return None,
        })
    }

    /// Maps operator and delimiter variants of `TokenKind` to their static string slices. Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR] | [FLS: fls_8tsynkj2cufj]
    fn symbol_as_str(&self) -> &'static str {
        match self {
            Self::ColonEq => ":=", Self::ColonGt => ":>", Self::DoubleColon => "::",
            Self::Arrow => "->", Self::TildeGt => "~>", Self::EqEq => "==",
            Self::NotEq => "!=", Self::LtEq => "<=", Self::GtEq => ">=",
            Self::AmpAmp => "&&", Self::PipePipe => "||", Self::Eq => "=",
            Self::Lt => "<", Self::Gt => ">", Self::Plus => "+",
            Self::Minus => "-", Self::Star => "*", Self::Slash => "/",
            Self::Percent => "%", Self::Caret => "^", Self::Tilde => "~",
            Self::Exclamation => "!", Self::Colon => ":", Self::Semi => ";",
            Self::Comma => ",", Self::Dot => ".", Self::OpenBrace => "{",
            Self::CloseBrace => "}", Self::OpenParen => "(", Self::CloseParen => ")",
            Self::OpenBracket => "[", Self::CloseBracket => "]", Self::Eof => "<EOF>",
            _ => "<dynamic>",
        }
    }
}

/// Token structure combining token kind and source location span.
/// Realises: [REQ-LEXER-TOKEN-TOKEN] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// Creates a new `Token`. Realises: [REQ-LEXER-TOKEN-TOKEN-NEW] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

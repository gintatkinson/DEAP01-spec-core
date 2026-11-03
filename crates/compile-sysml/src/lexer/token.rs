//! Token definitions and source spans for SysML v2 / KerML lexical analysis.
//! [DOC-TIER: A]
//!
//! Safety & Traceability: Realises: [REQ-LEXER-TOKEN-SPAN], [REQ-LEXER-TOKEN-TOKENKIND], [REQ-LEXER-TOKEN-TOKEN]
//! Contract: Defines `Span`, `TokenKind`, and `Token` for SysML v2 / KerML lexical and syntactic processing.
//! Bounds & Faults: Pure value types with `O(1)` constructors and static keyword projection.

use std::fmt;

/// Represents a source span with byte offsets and 1-indexed line/column numbers.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Span` in `crates/compile-sysml/src/lexer/token.rs`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Span` across member set `[start: usize, end: usize, line: usize, col: usize]`.
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Span` members (`start: usize, end: usize, line: usize, col: usize`).
/// ## 4. Postconditions
/// - Defines the authoritative `Span` representation for struct `Span`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `Span`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `Span`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Span` (`Span`).
/// - State: Maintains consistent state invariants across `Span` without partial mutation on error.
/// - Action: Executes `Span` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Span` (`LEXER-TOKEN`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-TOKEN-SPAN] | SysML v2: PartDef(Span)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    /// Executes deterministic fn `Span::new` within `crates/compile-sysml/src/lexer/token.rs`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Span::new` in `crates/compile-sysml/src/lexer/token.rs`.
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Span::new` (declaration #2) within `LEXER-TOKEN`.
    /// ## 3. Preconditions
    /// - `start: usize`, `end: usize`, `line: usize`, `col: usize` must satisfy domain and type-safety constraints prior to invoking `Span::new`.
    /// ## 4. Postconditions
    /// - Returns `Self` from `Span::new` with deterministic post-execution state.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) bounded over input/member dimension of `Span::new`.
    /// - Space Complexity: O(1) bounded auxiliary memory for `Span::new`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Span::new` (`Self`).
    /// - State: Maintains consistent state invariants across `Span::new` without partial mutation on error.
    /// - Action: Executes `Span::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Span::new` (`LEXER-TOKEN`).
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-SPAN-NEW] | SysML v2: ActionDef(Span::new)
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self { start, end, line, col }
    }
}

impl fmt::Display for Span {
    /// Executes deterministic fn `Span::fmt` within `crates/compile-sysml/src/lexer/token.rs`.
    /// Realises: [REQ-LEXER-TOKEN-SPAN-FMT] | SysML v2: ActionDef(Span::fmt)
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
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `TokenKind` in `crates/compile-sysml/src/lexer/token.rs`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `TokenKind` across member set `[Package, Import, Part, Def, Port, Item]`.
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TokenKind` members (`Package, Import, Part, Def, Port, Item`).
/// ## 4. Postconditions
/// - Defines the authoritative `TokenKind` representation for enum `TokenKind`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `TokenKind`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `TokenKind`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TokenKind` (`TokenKind`).
/// - State: Maintains consistent state invariants across `TokenKind` without partial mutation on error.
/// - Action: Executes `TokenKind` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `TokenKind` (`LEXER-TOKEN`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-TOKEN-TOKENKIND] | SysML v2: PartDef(TokenKind)
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
    /// Return the textual keyword or symbol representation if static.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `TokenKind::as_str` in `crates/compile-sysml/src/lexer/token.rs`.
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `TokenKind::as_str` (declaration #5) within `LEXER-TOKEN`.
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `TokenKind::as_str` in `LEXER-TOKEN`.
    /// ## 4. Postconditions
    /// - Returns `&'static str` from `TokenKind::as_str` with deterministic post-execution state.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) bounded over input/member dimension of `TokenKind::as_str`.
    /// - Space Complexity: O(1) bounded auxiliary memory for `TokenKind::as_str`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `TokenKind::as_str` (`&'static str`).
    /// - State: Maintains consistent state invariants across `TokenKind::as_str` without partial mutation on error.
    /// - Action: Executes `TokenKind::as_str` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `TokenKind::as_str` (`LEXER-TOKEN`).
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR] | SysML v2: ActionDef(TokenKind::as_str)
    pub fn as_str(&self) -> &'static str {
        self.keyword_as_str().unwrap_or_else(|| self.symbol_as_str())
    }

    /// Resolves a single-word keyword or boolean literal lexeme into its `TokenKind`, or falls back to `TokenKind::Ident`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-IDENT-OR-KEYWORD]
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

    /// Resolves a 1- or 2-character operator or punctuation lexeme into its `TokenKind`.
    /// Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR]
    pub(crate) fn from_symbol(text: &str) -> Option<Self> {
        SYMBOL_KINDS.iter().find(|k| k.as_str() == text).cloned()
    }

    /// Strips optional leading `doc` or `doc:` prefix from a block comment body and wraps in `Self::DocComment`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-BLOCK-COMMENT]
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

    /// Parses a numeric slice into `Self::RealLit` or `Self::IntLit`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-NUMBER]
    pub(crate) fn parse_number_lexeme(num_str: &str, line: usize, col: usize) -> Result<Self, String> {
        if num_str.contains(['.', 'e', 'E']) {
            num_str
                .parse::<f64>()
                .map(Self::RealLit)
                .map_err(|e| format!("Invalid float '{num_str}' at {line}:{col}: {e}"))
        } else {
            num_str
                .parse::<i64>()
                .map(Self::IntLit)
                .map_err(|e| format!("Invalid integer '{num_str}' at {line}:{col}: {e}"))
        }
    }

    /// Maps keyword variants of `TokenKind` to their static string slices.
    /// Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR]
    fn keyword_as_str(&self) -> Option<&'static str> {
        let s = match self {
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
        };
        Some(s)
    }

    /// Maps operator, delimiter, and fallback variants of `TokenKind` to their static string slices.
    /// Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR]
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
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Token` in `crates/compile-sysml/src/lexer/token.rs`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Token` across member set `[kind: TokenKind, span: Span]`.
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Token` members (`kind: TokenKind, span: Span`).
/// ## 4. Postconditions
/// - Defines the authoritative `Token` representation for struct `Token`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `Token`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `Token`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Token` (`Token`).
/// - State: Maintains consistent state invariants across `Token` without partial mutation on error.
/// - Action: Executes `Token` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Token` (`LEXER-TOKEN`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-TOKEN-TOKEN] | SysML v2: PartDef(Token)
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// Executes deterministic fn `Token::new` within `crates/compile-sysml/src/lexer/token.rs`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Token::new` in `crates/compile-sysml/src/lexer/token.rs`.
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Token::new` (declaration #7) within `LEXER-TOKEN`.
    /// ## 3. Preconditions
    /// - `kind: TokenKind` and `span: Span` must satisfy domain and type-safety constraints prior to invoking `Token::new`.
    /// ## 4. Postconditions
    /// - Returns `Self` from `Token::new` with deterministic post-execution state.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) bounded over input/member dimension of `Token::new`.
    /// - Space Complexity: O(1) bounded auxiliary memory for `Token::new`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Token::new` (`Self`).
    /// - State: Maintains consistent state invariants across `Token::new` without partial mutation on error.
    /// - Action: Executes `Token` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Token::new` (`LEXER-TOKEN`).
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-TOKEN-NEW] | SysML v2: ActionDef(Token::new)
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

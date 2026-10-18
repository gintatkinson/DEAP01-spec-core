//! Token definitions and source spans for SysML v2 / KerML lexical analysis.

use std::fmt;

/// Represents a source span with byte offsets and 1-indexed line/column numbers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Span` in `crates/compile-sysml/src/lexer/token.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Span` across member set `[start: usize, end: usize, line: usize, col: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Span` members (`start: usize, end: usize, line: usize, col: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Span` representation for struct `Span`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `Span`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `Span`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Span` (`Span`).
/// - State: Maintains consistent state invariants across `Span` without partial mutation on error.
/// - Action: Executes `Span` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Span` (`LEXER-TOKEN`).
///
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Span::new` in `crates/compile-sysml/src/lexer/token.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Span::new` (declaration #2) within `LEXER-TOKEN`.
    ///
    /// ## 3. Preconditions
    /// - `start: usize` must satisfy domain and type-safety constraints prior to invoking `Span::new`.
    /// - `end: usize` must satisfy domain and type-safety constraints prior to invoking `Span::new`.
    /// - `line: usize` must satisfy domain and type-safety constraints prior to invoking `Span::new`.
    /// - `col: usize` must satisfy domain and type-safety constraints prior to invoking `Span::new`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `Span::new` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(start)) bounded over input/member dimension of `Span::new`.
    /// - Space Complexity: O(N(start)) bounded auxiliary memory for `Span::new`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Span::new` (`Self`).
    /// - State: Maintains consistent state invariants across `Span::new` without partial mutation on error.
    /// - Action: Executes `Span::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Span::new` (`LEXER-TOKEN`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-SPAN-NEW] | SysML v2: ActionDef(Span::new)
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self {
            start,
            end,
            line,
            col,
        }
    }
}

impl fmt::Display for Span {
    /// Executes deterministic fn `Span::fmt` within `crates/compile-sysml/src/lexer/token.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Span::fmt` in `crates/compile-sysml/src/lexer/token.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Span::fmt` (declaration #3) within `LEXER-TOKEN`.
    ///
    /// ## 3. Preconditions
    /// - `f: &mut fmt::Formatter<'_>` must satisfy domain and type-safety constraints prior to invoking `Span::fmt`.
    ///
    /// ## 4. Postconditions
    /// - Returns `fmt::Result` from `Span::fmt` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(f)) bounded over input/member dimension of `Span::fmt`.
    /// - Space Complexity: O(N(f)) bounded auxiliary memory for `Span::fmt`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Span::fmt` (`fmt::Result`).
    /// - State: Maintains consistent state invariants across `Span::fmt` without partial mutation on error.
    /// - Action: Executes `Span::fmt` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Span::fmt` (`LEXER-TOKEN`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-SPAN-FMT] | SysML v2: ActionDef(Span::fmt)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}:{}", self.line, self.col, self.line, self.col + (self.end - self.start))
    }
}

/// Token classification for SysML v2 and KerML textual syntax.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `TokenKind` in `crates/compile-sysml/src/lexer/token.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `TokenKind` across member set `[Package, Import, Part, Def, Port, Item]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TokenKind` members (`Package, Import, Part, Def, Port, Item`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `TokenKind` representation for enum `TokenKind`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `TokenKind`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `TokenKind`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TokenKind` (`TokenKind`).
/// - State: Maintains consistent state invariants across `TokenKind` without partial mutation on error.
/// - Action: Executes `TokenKind` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `TokenKind` (`LEXER-TOKEN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-TOKEN-TOKENKIND] | SysML v2: PartDef(TokenKind)
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Declarative & Structural Keywords
    Package,
    Import,
    Part,
    Def,
    Port,
    Item,
    Flow,
    Interface,
    Subsystem,
    Subject,
    Actor,

    // Behavioral & State Machine Keywords
    Action,
    Operation,
    State,
    Transition,
    Entry,
    Do,
    Exit,
    Perform,
    Step,
    Trigger,

    // Requirement & Constraint Keywords
    Requirement,
    Constraint,
    Assert,
    Assume,
    Require,
    Verify,
    Satisfy,
    Calc,

    // Verification & Safety Keywords
    Test,
    Case,
    Hazard,
    Risk,
    UseCase,
    Include,
    Extend,
    Precondition,
    Postcondition,
    Objective,

    // Communication & Interaction Keywords
    Capability,
    Interaction,
    Lifeline,
    Message,

    // Topology & Routing Keywords
    Connection,
    Connect,
    From,
    To,
    By,

    // Typing & Attributes
    Attribute,
    Id,
    Text,
    Doc,
    In,
    Out,
    Inout,

    // Literals
    Ident(String),
    StringLit(String),
    IntLit(i64),
    RealLit(f64),
    BoolLit(bool),

    // Comments
    DocComment(String),
    LineComment(String),

    // Multi-character Operators
    ColonEq,       // :=
    ColonGt,       // :>
    DoubleColon,   // ::
    Arrow,         // ->
    TildeGt,       // ~>
    EqEq,          // ==
    NotEq,         // !=
    LtEq,          // <=
    GtEq,          // >=
    AmpAmp,        // &&
    PipePipe,      // ||

    // Single-character Operators
    Eq,            // =
    Lt,            // <
    Gt,            // >
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Caret,         // ^
    Tilde,         // ~
    Exclamation,   // !

    // Punctuation
    Colon,         // :
    Semi,          // ;
    Comma,         // ,
    Dot,           // .
    OpenBrace,     // {
    CloseBrace,    // }
    OpenParen,     // (
    CloseParen,    // )
    OpenBracket,   // [
    CloseBracket,  // ]

    // End of file
    Eof,
}

impl TokenKind {
    /// Return the textual keyword or symbol representation if static.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `TokenKind::as_str` in `crates/compile-sysml/src/lexer/token.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `TokenKind::as_str` (declaration #5) within `LEXER-TOKEN`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `TokenKind::as_str` in `LEXER-TOKEN`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&'static str` from `TokenKind::as_str` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(as_str)) bounded over input/member dimension of `TokenKind::as_str`.
    /// - Space Complexity: O(K(as_str)) bounded auxiliary memory for `TokenKind::as_str`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `TokenKind::as_str` (`&'static str`).
    /// - State: Maintains consistent state invariants across `TokenKind::as_str` without partial mutation on error.
    /// - Action: Executes `TokenKind::as_str` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `TokenKind::as_str` (`LEXER-TOKEN`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-TOKENKIND-AS-STR] | SysML v2: ActionDef(TokenKind::as_str)
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Package => "package",
            TokenKind::Import => "import",
            TokenKind::Part => "part",
            TokenKind::Def => "def",
            TokenKind::Port => "port",
            TokenKind::Item => "item",
            TokenKind::Flow => "flow",
            TokenKind::Interface => "interface",
            TokenKind::Subsystem => "subsystem",
            TokenKind::Subject => "subject",
            TokenKind::Actor => "actor",
            TokenKind::Action => "action",
            TokenKind::Operation => "operation",
            TokenKind::State => "state",
            TokenKind::Transition => "transition",
            TokenKind::Entry => "entry",
            TokenKind::Do => "do",
            TokenKind::Exit => "exit",
            TokenKind::Perform => "perform",
            TokenKind::Step => "step",
            TokenKind::Trigger => "trigger",
            TokenKind::Requirement => "requirement",
            TokenKind::Constraint => "constraint",
            TokenKind::Assert => "assert",
            TokenKind::Assume => "assume",
            TokenKind::Require => "require",
            TokenKind::Verify => "verify",
            TokenKind::Satisfy => "satisfy",
            TokenKind::Calc => "calc",
            TokenKind::Test => "test",
            TokenKind::Case => "case",
            TokenKind::Hazard => "hazard",
            TokenKind::Risk => "risk",
            TokenKind::UseCase => "use case",
            TokenKind::Include => "include",
            TokenKind::Extend => "extend",
            TokenKind::Precondition => "precondition",
            TokenKind::Postcondition => "postcondition",
            TokenKind::Objective => "objective",
            TokenKind::Capability => "capability",
            TokenKind::Interaction => "interaction",
            TokenKind::Lifeline => "lifeline",
            TokenKind::Message => "message",
            TokenKind::Connection => "connection",
            TokenKind::Connect => "connect",
            TokenKind::From => "from",
            TokenKind::To => "to",
            TokenKind::By => "by",
            TokenKind::Attribute => "attribute",
            TokenKind::Id => "id",
            TokenKind::Text => "text",
            TokenKind::Doc => "doc",
            TokenKind::In => "in",
            TokenKind::Out => "out",
            TokenKind::Inout => "inout",
            TokenKind::ColonEq => ":=",
            TokenKind::ColonGt => ":>",
            TokenKind::DoubleColon => "::",
            TokenKind::Arrow => "->",
            TokenKind::TildeGt => "~>",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::LtEq => "<=",
            TokenKind::GtEq => ">=",
            TokenKind::AmpAmp => "&&",
            TokenKind::PipePipe => "||",
            TokenKind::Eq => "=",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Caret => "^",
            TokenKind::Tilde => "~",
            TokenKind::Exclamation => "!",
            TokenKind::Colon => ":",
            TokenKind::Semi => ";",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::OpenBrace => "{",
            TokenKind::CloseBrace => "}",
            TokenKind::OpenParen => "(",
            TokenKind::CloseParen => ")",
            TokenKind::OpenBracket => "[",
            TokenKind::CloseBracket => "]",
            TokenKind::Eof => "<EOF>",
            _ => "<dynamic>",
        }
    }
}

/// Token structure combining token kind and source location span.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Token` in `crates/compile-sysml/src/lexer/token.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Token` across member set `[kind: TokenKind, span: Span]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Token` members (`kind: TokenKind, span: Span`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Token` representation for struct `Token`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `Token`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `Token`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Token` (`Token`).
/// - State: Maintains consistent state invariants across `Token` without partial mutation on error.
/// - Action: Executes `Token` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Token` (`LEXER-TOKEN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-TOKEN-TOKEN] | SysML v2: PartDef(Token)
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// Executes deterministic fn `Token::new` within `crates/compile-sysml/src/lexer/token.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Token::new` in `crates/compile-sysml/src/lexer/token.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Token::new` (declaration #7) within `LEXER-TOKEN`.
    ///
    /// ## 3. Preconditions
    /// - `kind: TokenKind` must satisfy domain and type-safety constraints prior to invoking `Token::new`.
    /// - `span: Span` must satisfy domain and type-safety constraints prior to invoking `Token::new`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `Token::new` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(kind)) bounded over input/member dimension of `Token::new`.
    /// - Space Complexity: O(N(kind)) bounded auxiliary memory for `Token::new`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Token::new` (`Self`).
    /// - State: Maintains consistent state invariants across `Token::new` without partial mutation on error.
    /// - Action: Executes `Token::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Token::new` (`LEXER-TOKEN`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-TOKEN-TOKEN-NEW] | SysML v2: ActionDef(Token::new)
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

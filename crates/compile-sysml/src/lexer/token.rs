//! Token definitions and source spans for SysML v2 / KerML lexical analysis.

use std::fmt;

/// Represents a source span with byte offsets and 1-indexed line/column numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}:{}", self.line, self.col, self.line, self.col + (self.end - self.start))
    }
}

/// Token classification for SysML v2 and KerML textual syntax.
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
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

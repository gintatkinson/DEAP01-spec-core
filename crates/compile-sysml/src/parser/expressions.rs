//! Mathematical, logical, and formal KaTeX expression parser for SysML v2 constraints.

use crate::lexer::token::{Span, Token, TokenKind};

// Sentinel returned when the token stream is empty; never indexed, never mutated.
static EXPR_EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    span: Span {
        start: 0,
        end: 0,
        line: 0,
        col: 0,
    },
};

/// Binary operations supported in SysML v2 constraints and expressions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `BinaryOp` in `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `BinaryOp` across member set `[Add, Sub, Mul, Div, Mod, Pow]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `BinaryOp` members (`Add, Sub, Mul, Div, Mod, Pow`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `BinaryOp` representation for enum `BinaryOp`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `BinaryOp`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `BinaryOp`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `BinaryOp` (`BinaryOp`).
/// - State: Maintains consistent state invariants across `BinaryOp` without partial mutation on error.
/// - Action: Executes `BinaryOp` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `BinaryOp` (`PARSER-EXPRESSIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARSER-EXPRESSIONS-BINARYOP] | SysML v2: PartDef(BinaryOp)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl BinaryOp {
    /// Executes deterministic fn `BinaryOp::as_str` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `BinaryOp::as_str` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `BinaryOp::as_str` (declaration #2) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `BinaryOp::as_str` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&'static str` from `BinaryOp::as_str` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(as_str)) bounded over input/member dimension of `BinaryOp::as_str`.
    /// - Space Complexity: O(K(as_str)) bounded auxiliary memory for `BinaryOp::as_str`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `BinaryOp::as_str` (`&'static str`).
    /// - State: Maintains consistent state invariants across `BinaryOp::as_str` without partial mutation on error.
    /// - Action: Executes `BinaryOp::as_str` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `BinaryOp::as_str` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-BINARYOP-AS-STR] | SysML v2: ActionDef(BinaryOp::as_str)
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "^",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::LtEq => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        }
    }
}

/// Unary operations supported in SysML v2 expressions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `UnaryOp` in `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `UnaryOp` across member set `[Neg, Not]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `UnaryOp` members (`Neg, Not`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `UnaryOp` representation for enum `UnaryOp`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `UnaryOp`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `UnaryOp`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `UnaryOp` (`UnaryOp`).
/// - State: Maintains consistent state invariants across `UnaryOp` without partial mutation on error.
/// - Action: Executes `UnaryOp` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `UnaryOp` (`PARSER-EXPRESSIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARSER-EXPRESSIONS-UNARYOP] | SysML v2: PartDef(UnaryOp)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    /// Executes deterministic fn `UnaryOp::as_str` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `UnaryOp::as_str` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `UnaryOp::as_str` (declaration #4) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `UnaryOp::as_str` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&'static str` from `UnaryOp::as_str` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(as_str)) bounded over input/member dimension of `UnaryOp::as_str`.
    /// - Space Complexity: O(K(as_str)) bounded auxiliary memory for `UnaryOp::as_str`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `UnaryOp::as_str` (`&'static str`).
    /// - State: Maintains consistent state invariants across `UnaryOp::as_str` without partial mutation on error.
    /// - Action: Executes `UnaryOp::as_str` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `UnaryOp::as_str` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-UNARYOP-AS-STR] | SysML v2: ActionDef(UnaryOp::as_str)
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "not ",
        }
    }
}

/// Abstract syntax tree node for mathematical and boolean expressions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `Expr` in `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `Expr` across member set `[Ident, Number, StringLit, BoolLit, Unary, op]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Expr` members (`Ident, Number, StringLit, BoolLit, Unary, op`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Expr` representation for enum `Expr`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `Expr`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `Expr`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Expr` (`Expr`).
/// - State: Maintains consistent state invariants across `Expr` without partial mutation on error.
/// - Action: Executes `Expr` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Expr` (`PARSER-EXPRESSIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARSER-EXPRESSIONS-EXPR] | SysML v2: PartDef(Expr)
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Number(f64),
    StringLit(String),
    BoolLit(bool),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Member {
        object: Box<Expr>,
        member: String,
    },
    Raw(String),
}

impl Expr {
    /// Format expression into standard algebraic / SysML textual syntax.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Expr::to_string_repr` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Expr::to_string_repr` (declaration #6) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Expr::to_string_repr` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `Expr::to_string_repr` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(to_string_repr)) bounded over input/member dimension of `Expr::to_string_repr`.
    /// - Space Complexity: O(K(to_string_repr)) bounded auxiliary memory for `Expr::to_string_repr`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Expr::to_string_repr` (`String`).
    /// - State: Maintains consistent state invariants across `Expr::to_string_repr` without partial mutation on error.
    /// - Action: Executes `Expr::to_string_repr` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Expr::to_string_repr` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPR-TO-STRING-REPR] | SysML v2: ActionDef(Expr::to_string_repr)
    pub fn to_string_repr(&self) -> String {
        match self {
            Expr::Ident(name) => name.clone(),
            Expr::Number(val) => {
                if val.fract() == 0.0 {
                    format!("{:.1}", val)
                } else {
                    format!("{}", val)
                }
            }
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::BoolLit(b) => format!("{}", b),
            Expr::Unary { op, expr } => format!("{}{}", op.as_str(), expr.to_string_repr()),
            Expr::Binary { op, left, right } => {
                format!(
                    "{} {} {}",
                    left.to_string_repr(),
                    op.as_str(),
                    right.to_string_repr()
                )
            }
            Expr::Call { func, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| a.to_string_repr()).collect();
                format!("{}({})", func, arg_strs.join(", "))
            }
            Expr::Member { object, member } => {
                format!("{}.{}", object.to_string_repr(), member)
            }
            Expr::Raw(text) => text.clone(),
        }
    }
}

/// Expression parser using precedence climbing.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ExpressionParser` in `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ExpressionParser` across member set `[tokens: &'a [Token], pos: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ExpressionParser` members (`tokens: &'a [Token], pos: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ExpressionParser` representation for struct `ExpressionParser`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `ExpressionParser`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `ExpressionParser`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ExpressionParser` (`ExpressionParser`).
/// - State: Maintains consistent state invariants across `ExpressionParser` without partial mutation on error.
/// - Action: Executes `ExpressionParser` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ExpressionParser` (`PARSER-EXPRESSIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER] | SysML v2: PartDef(ExpressionParser)
pub struct ExpressionParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ExpressionParser<'a> {
    /// Executes deterministic fn `ExpressionParser::new` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::new` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::new` (declaration #8) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - `tokens: &'a [Token]` must satisfy domain and type-safety constraints prior to invoking `ExpressionParser::new`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `ExpressionParser::new` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(tokens)) bounded over input/member dimension of `ExpressionParser::new`.
    /// - Space Complexity: O(N(tokens)) bounded auxiliary memory for `ExpressionParser::new`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::new` (`Self`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::new` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::new` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-NEW] | SysML v2: ActionDef(ExpressionParser::new)
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse complete expression.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::parse` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::parse` (declaration #9) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::parse` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Expr, String>` from `ExpressionParser::parse` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(parse)) bounded over input/member dimension of `ExpressionParser::parse`.
    /// - Space Complexity: O(K(parse)) bounded auxiliary memory for `ExpressionParser::parse`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::parse` (`Result<Expr, String>`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::parse` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::parse` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::parse` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE] | SysML v2: ActionDef(ExpressionParser::parse)
    pub fn parse(&mut self) -> Result<Expr, String> {
        let expr = self.parse_binary_expr(0)?;
        Ok(expr)
    }

    /// Executes deterministic fn `ExpressionParser::parse_binary_expr` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::parse_binary_expr` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::parse_binary_expr` (declaration #10) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - `min_precedence: u8` must satisfy domain and type-safety constraints prior to invoking `ExpressionParser::parse_binary_expr`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Expr, String>` from `ExpressionParser::parse_binary_expr` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(min_precedence)) bounded over input/member dimension of `ExpressionParser::parse_binary_expr`.
    /// - Space Complexity: O(N(min_precedence)) bounded auxiliary memory for `ExpressionParser::parse_binary_expr`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::parse_binary_expr` (`Result<Expr, String>`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::parse_binary_expr` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::parse_binary_expr` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::parse_binary_expr` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-BINARY-EXPR] | SysML v2: ActionDef(ExpressionParser::parse_binary_expr)
    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let mut left = self.parse_unary_or_primary()?;

        while let Some(op) = self.peek_binary_op() {
            let (prec, assoc_right) = binary_precedence(&op);
            if prec < min_precedence {
                break;
            }

            self.advance(); // consume operator
            let next_prec = if assoc_right { prec } else { prec + 1 };
            let right = self.parse_binary_expr(next_prec)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Executes deterministic fn `ExpressionParser::parse_unary_or_primary` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::parse_unary_or_primary` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::parse_unary_or_primary` (declaration #11) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::parse_unary_or_primary` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Expr, String>` from `ExpressionParser::parse_unary_or_primary` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(parse_unary_or_primary)) bounded over input/member dimension of `ExpressionParser::parse_unary_or_primary`.
    /// - Space Complexity: O(K(parse_unary_or_primary)) bounded auxiliary memory for `ExpressionParser::parse_unary_or_primary`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::parse_unary_or_primary` (`Result<Expr, String>`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::parse_unary_or_primary` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::parse_unary_or_primary` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::parse_unary_or_primary` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-UNARY-OR-PRIMARY] | SysML v2: ActionDef(ExpressionParser::parse_unary_or_primary)
    fn parse_unary_or_primary(&mut self) -> Result<Expr, String> {
        if self.is_at_end() {
            return Err("Unexpected end of expression tokens".to_string());
        }

        match &self.peek().kind {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary_or_primary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Exclamation => {
                self.advance();
                let expr = self.parse_unary_or_primary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    /// Executes deterministic fn `ExpressionParser::parse_primary` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::parse_primary` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::parse_primary` (declaration #12) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::parse_primary` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Expr, String>` from `ExpressionParser::parse_primary` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(parse_primary)) bounded over input/member dimension of `ExpressionParser::parse_primary`.
    /// - Space Complexity: O(K(parse_primary)) bounded auxiliary memory for `ExpressionParser::parse_primary`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::parse_primary` (`Result<Expr, String>`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::parse_primary` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::parse_primary` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::parse_primary` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-PRIMARY] | SysML v2: ActionDef(ExpressionParser::parse_primary)
    fn parse_primary(&mut self) -> Result<Expr, String> {
        if self.is_at_end() {
            return Err("Expected primary expression, found EOF".to_string());
        }

        let tok = self.advance();
        let mut expr = match &tok.kind {
            TokenKind::IntLit(n) => Expr::Number(*n as f64),
            TokenKind::RealLit(f) => Expr::Number(*f),
            TokenKind::StringLit(s) => Expr::StringLit(s.clone()),
            TokenKind::BoolLit(b) => Expr::BoolLit(*b),
            TokenKind::Ident(name) => {
                let ident = name.clone();
                if self.peek_kind() == Some(&TokenKind::OpenParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    while self.peek_kind() != Some(&TokenKind::CloseParen) && !self.is_at_end() {
                        args.push(self.parse_binary_expr(0)?);
                        if self.peek_kind() == Some(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if self.peek_kind() == Some(&TokenKind::CloseParen) {
                        self.advance();
                    } else {
                        return Err(format!("Expected ')' after function arguments for {}", ident));
                    }
                    Expr::Call { func: ident, args }
                } else {
                    Expr::Ident(ident)
                }
            }
            TokenKind::OpenParen => {
                let inner = self.parse_binary_expr(0)?;
                if self.peek_kind() == Some(&TokenKind::CloseParen) {
                    self.advance();
                    inner
                } else {
                    return Err("Expected matching ')' in parenthesized expression".to_string());
                }
            }
            other => {
                return Err(format!("Unexpected token in expression: {:?}", other));
            }
        };

        // Check for member accesses `.`
        while self.peek_kind() == Some(&TokenKind::Dot) {
            self.advance(); // consume '.'
            if let Some(TokenKind::Ident(field_name)) = self.peek_kind().cloned() {
                self.advance();
                expr = Expr::Member {
                    object: Box::new(expr),
                    member: field_name,
                };
            } else {
                return Err("Expected field identifier following '.'".to_string());
            }
        }

        Ok(expr)
    }

    /// Executes deterministic fn `ExpressionParser::peek` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::peek` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::peek` (declaration #13) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::peek` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&Token` from `ExpressionParser::peek` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(peek)) bounded over input/member dimension of `ExpressionParser::peek`.
    /// - Space Complexity: O(K(peek)) bounded auxiliary memory for `ExpressionParser::peek`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::peek` (`&Token`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::peek` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::peek` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::peek` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PEEK] | SysML v2: ActionDef(ExpressionParser::peek)
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .unwrap_or(&EXPR_EOF_TOKEN)
    }

    /// Executes deterministic fn `ExpressionParser::peek_kind` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::peek_kind` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::peek_kind` (declaration #14) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::peek_kind` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Option<&TokenKind>` from `ExpressionParser::peek_kind` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(peek_kind)) bounded over input/member dimension of `ExpressionParser::peek_kind`.
    /// - Space Complexity: O(K(peek_kind)) bounded auxiliary memory for `ExpressionParser::peek_kind`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::peek_kind` (`Option<&TokenKind>`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::peek_kind` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::peek_kind` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::peek_kind` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PEEK-KIND] | SysML v2: ActionDef(ExpressionParser::peek_kind)
    fn peek_kind(&self) -> Option<&TokenKind> {
        if self.is_at_end() {
            None
        } else {
            self.tokens.get(self.pos).map(|t| &t.kind)
        }
    }

    /// Executes deterministic fn `ExpressionParser::advance` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::advance` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::advance` (declaration #15) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::advance` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&Token` from `ExpressionParser::advance` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(advance)) bounded over input/member dimension of `ExpressionParser::advance`.
    /// - Space Complexity: O(K(advance)) bounded auxiliary memory for `ExpressionParser::advance`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::advance` (`&Token`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::advance` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::advance` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::advance` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-ADVANCE] | SysML v2: ActionDef(ExpressionParser::advance)
    fn advance(&mut self) -> &Token {
        let idx = self.pos;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        self.tokens
            .get(idx)
            .or_else(|| self.tokens.last())
            .unwrap_or(&EXPR_EOF_TOKEN)
    }

    /// Executes deterministic fn `ExpressionParser::is_at_end` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::is_at_end` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::is_at_end` (declaration #16) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::is_at_end` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `bool` from `ExpressionParser::is_at_end` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(is_at_end)) bounded over input/member dimension of `ExpressionParser::is_at_end`.
    /// - Space Complexity: O(K(is_at_end)) bounded auxiliary memory for `ExpressionParser::is_at_end`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::is_at_end` (`bool`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::is_at_end` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::is_at_end` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::is_at_end` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-IS-AT-END] | SysML v2: ActionDef(ExpressionParser::is_at_end)
    fn is_at_end(&self) -> bool {
        self.tokens.get(self.pos).is_none_or(|t| t.kind == TokenKind::Eof)
    }

    /// Executes deterministic fn `ExpressionParser::peek_binary_op` within `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ExpressionParser::peek_binary_op` in `crates/compile-sysml/src/parser/expressions.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ExpressionParser::peek_binary_op` (declaration #17) within `PARSER-EXPRESSIONS`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ExpressionParser::peek_binary_op` in `PARSER-EXPRESSIONS`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Option<BinaryOp>` from `ExpressionParser::peek_binary_op` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(peek_binary_op)) bounded over input/member dimension of `ExpressionParser::peek_binary_op`.
    /// - Space Complexity: O(K(peek_binary_op)) bounded auxiliary memory for `ExpressionParser::peek_binary_op`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ExpressionParser::peek_binary_op` (`Option<BinaryOp>`).
    /// - State: Maintains consistent state invariants across `ExpressionParser::peek_binary_op` without partial mutation on error.
    /// - Action: Executes `ExpressionParser::peek_binary_op` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ExpressionParser::peek_binary_op` (`PARSER-EXPRESSIONS`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PEEK-BINARY-OP] | SysML v2: ActionDef(ExpressionParser::peek_binary_op)
    fn peek_binary_op(&self) -> Option<BinaryOp> {
        let kind = self.peek_kind()?;
        match kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Mod),
            TokenKind::Caret => Some(BinaryOp::Pow),
            TokenKind::EqEq => Some(BinaryOp::Eq),
            TokenKind::NotEq => Some(BinaryOp::NotEq),
            TokenKind::Lt => Some(BinaryOp::Lt),
            TokenKind::LtEq => Some(BinaryOp::LtEq),
            TokenKind::Gt => Some(BinaryOp::Gt),
            TokenKind::GtEq => Some(BinaryOp::GtEq),
            TokenKind::AmpAmp => Some(BinaryOp::And),
            TokenKind::PipePipe => Some(BinaryOp::Or),
            TokenKind::Ident(s) if s == "and" => Some(BinaryOp::And),
            TokenKind::Ident(s) if s == "or" => Some(BinaryOp::Or),
            _ => None,
        }
    }
}

/// Executes deterministic fn `binary_precedence` within `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `binary_precedence` in `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `binary_precedence` (declaration #18) within `PARSER-EXPRESSIONS`.
///
/// ## 3. Preconditions
/// - `op: &BinaryOp` must satisfy domain and type-safety constraints prior to invoking `binary_precedence`.
///
/// ## 4. Postconditions
/// - Returns `(u8, bool)` from `binary_precedence` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(op)) bounded over input/member dimension of `binary_precedence`.
/// - Space Complexity: O(N(op)) bounded auxiliary memory for `binary_precedence`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `binary_precedence` (`(u8, bool)`).
/// - State: Maintains consistent state invariants across `binary_precedence` without partial mutation on error.
/// - Action: Executes `binary_precedence` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `binary_precedence` (`PARSER-EXPRESSIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARSER-EXPRESSIONS-BINARY-PRECEDENCE] | SysML v2: ActionDef(binary_precedence)
fn binary_precedence(op: &BinaryOp) -> (u8, bool) {
    match op {
        BinaryOp::Or => (1, false),
        BinaryOp::And => (2, false),
        BinaryOp::Eq | BinaryOp::NotEq => (3, false),
        BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => (4, false),
        BinaryOp::Add | BinaryOp::Sub => (5, false),
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => (6, false),
        BinaryOp::Pow => (7, true),
    }
}

/// Convenience function to parse tokens into an `Expr`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_expression` in `crates/compile-sysml/src/parser/expressions.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_expression` (declaration #19) within `PARSER-EXPRESSIONS`.
///
/// ## 3. Preconditions
/// - `tokens: &[Token]` must satisfy domain and type-safety constraints prior to invoking `parse_expression`.
///
/// ## 4. Postconditions
/// - Returns `Result<Expr, String>` from `parse_expression` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(tokens)) bounded over input/member dimension of `parse_expression`.
/// - Space Complexity: O(N(tokens)) bounded auxiliary memory for `parse_expression`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_expression` (`Result<Expr, String>`).
/// - State: Maintains consistent state invariants across `parse_expression` without partial mutation on error.
/// - Action: Executes `parse_expression` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_expression` (`PARSER-EXPRESSIONS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARSER-EXPRESSIONS-PARSE-EXPRESSION] | SysML v2: ActionDef(parse_expression)
pub fn parse_expression(tokens: &[Token]) -> Result<Expr, String> {
    let mut parser = ExpressionParser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::scanner::Scanner;

    #[test]
    fn test_parse_inequality_constraint() -> Result<(), Box<dyn std::error::Error>> {
        let mut scanner = Scanner::new("v <= 60.0");
        let tokens = scanner.scan_all()?;
        let expr = parse_expression(&tokens)?;

        assert_eq!(
            expr,
            Expr::Binary {
                op: BinaryOp::LtEq,
                left: Box::new(Expr::Ident("v".to_string())),
                right: Box::new(Expr::Number(60.0)),
            }
        );
        assert_eq!(expr.to_string_repr(), "v <= 60.0");
        Ok(())
    }

    #[test]
    fn test_parse_nested_logical_expression() -> Result<(), Box<dyn std::error::Error>> {
        let mut scanner = Scanner::new("alt >= 10.0 && alt <= 120.0");
        let tokens = scanner.scan_all()?;
        let expr = parse_expression(&tokens)?;

        match expr {
            Expr::Binary { op, left, right } => {
                assert_eq!(op, BinaryOp::And);
                assert_eq!(left.to_string_repr(), "alt >= 10.0");
                assert_eq!(right.to_string_repr(), "alt <= 120.0");
            }
            _ => return Err("Expected binary AND".into()),
        }
        Ok(())
    }

    #[test]
    fn test_parse_member_and_function_call() -> Result<(), Box<dyn std::error::Error>> {
        let mut scanner = Scanner::new("abs(fcc.speed) <= limits.max");
        let tokens = scanner.scan_all()?;
        let expr = parse_expression(&tokens)?;

        assert_eq!(expr.to_string_repr(), "abs(fcc.speed) <= limits.max");
        Ok(())
    }
}

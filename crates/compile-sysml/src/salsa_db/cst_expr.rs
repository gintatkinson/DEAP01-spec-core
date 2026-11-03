//! # Precedence-Climbed SysML v2 Expression Parser (`cst_expr`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Parses mathematical and logical SysML v2 expressions into `Expr` trees bounded by `MAX_EXPR_DEPTH` (`64`).
//! ## 2. Invariants & Mathematical Properties -- Enforces operator precedence and `depth < 64` recursion guard.
//! ## 3. Preconditions -- `tokens` is a slice of `Token` values produced by `Scanner`.
//! ## 4. Postconditions -- Returns `Ok(Expr)` or `Err(String)` on syntax/depth violation.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(N) over token count; Space: O(min(D, 64)).
//! ## 6. Failure Modes & Fault Tolerance -- Rejects deep expression nesting and malformed tokens deterministically.
//! ## 7. Realization & Traceability -- Realises: [REQ-PARSER-EXPRESSIONS-EXPR], [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use super::cst_db::{Span, Token, TokenKind};

static EXPR_EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    span: Span { start: 0, end: 0, line: 0, col: 0 },
};

pub(crate) const MAX_EXPR_DEPTH: usize = 64;

/// Validates that `depth < MAX_EXPR_DEPTH`. Realises: [REQ-PARSER-EXPRESSIONS-MAX-EXPR-DEPTH] | [FLS: fls_66m4rnbssgig]
fn ensure_expr_depth(depth: usize) -> Result<(), String> {
    if depth >= MAX_EXPR_DEPTH {
        Err(format!("Expression recursion depth exceeds maximum limit of {MAX_EXPR_DEPTH}"))
    } else {
        Ok(())
    }
}

/// Binary operations supported in SysML v2 constraints and expressions. Realises: [REQ-PARSER-EXPRESSIONS-BINARYOP] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow, Eq, NotEq, Lt, LtEq, Gt, GtEq, And, Or,
}

impl BinaryOp {
    /// Returns the canonical operator token string for `BinaryOp`. Realises: [REQ-PARSER-EXPRESSIONS-BINARYOP-AS-STR] | [FLS: fls_8tsynkj2cufj]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+", Self::Sub => "-", Self::Mul => "*", Self::Div => "/",
            Self::Mod => "%", Self::Pow => "^", Self::Eq => "==", Self::NotEq => "!=",
            Self::Lt => "<", Self::LtEq => "<=", Self::Gt => ">", Self::GtEq => ">=",
            Self::And => "and", Self::Or => "or",
        }
    }
}

/// Unary operations supported in SysML v2 expressions. Realises: [REQ-PARSER-EXPRESSIONS-UNARYOP] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    /// Returns the canonical operator prefix string for `UnaryOp`. Realises: [REQ-PARSER-EXPRESSIONS-UNARYOP-AS-STR] | [FLS: fls_8tsynkj2cufj]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Neg => "-",
            Self::Not => "not ",
        }
    }
}

/// Abstract syntax tree node for mathematical and boolean expressions. Realises: [REQ-PARSER-EXPRESSIONS-EXPR] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Number(f64),
    StringLit(String),
    BoolLit(bool),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
    Member { object: Box<Expr>, member: String },
    Raw(String),
}

impl Expr {
    /// Formats expression into standard algebraic / SysML textual syntax. Realises: [REQ-PARSER-EXPRESSIONS-EXPR-TO-STRING-REPR] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn to_string_repr(&self) -> String {
        self.to_string_repr_bounded(0)
    }

    /// Formats an expression tree into algebraic syntax up to `MAX_EXPR_DEPTH`. Realises: [REQ-PARSER-EXPRESSIONS-EXPR-TO-STRING-REPR] | [FLS: fls_66m4rnbssgig]
    fn to_string_repr_bounded(&self, depth: usize) -> String {
        if depth >= MAX_EXPR_DEPTH {
            return String::from("...");
        }
        let next_depth = depth.saturating_add(1);
        match self {
            Self::Ident(name) => name.clone(),
            Self::Number(val) if val.fract() == 0.0 => format!("{val:.1}"),
            Self::Number(val) => format!("{val}"),
            Self::StringLit(s) => format!("\"{s}\""),
            Self::BoolLit(b) => format!("{b}"),
            Self::Unary { op, expr } => format!("{}{}", op.as_str(), expr.to_string_repr_bounded(next_depth)),
            Self::Binary { op, left, right } => format!(
                "{} {} {}",
                left.to_string_repr_bounded(next_depth),
                op.as_str(),
                right.to_string_repr_bounded(next_depth)
            ),
            Self::Call { func, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| a.to_string_repr_bounded(next_depth)).collect();
                format!("{func}({})", arg_strs.join(", "))
            }
            Self::Member { object, member } => format!("{}.{member}", object.to_string_repr_bounded(next_depth)),
            Self::Raw(text) => text.clone(),
        }
    }
}

/// Expression parser using precedence climbing. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER] | [FLS: fls_66m4rnbssgig]
pub struct ExpressionParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ExpressionParser<'a> {
    /// Creates a new `ExpressionParser` over `tokens`. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-NEW] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub const fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parses a complete expression from the token stream. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE] | [FLS: fls_66m4rnbssgig]
    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_binary_expr(0, 0)
    }

    /// Parses binary operator expressions via precedence climbing. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-BINARY-EXPR] | [FLS: fls_66m4rnbssgig]
    fn parse_binary_expr(&mut self, min_precedence: u8, depth: usize) -> Result<Expr, String> {
        ensure_expr_depth(depth)?;
        let mut left = self.parse_unary_or_primary(depth)?;
        let mut chain_depth = depth;
        while let Some(op) = self.peek_binary_op() {
            let (prec, assoc_right) = binary_precedence(&op);
            if prec < min_precedence {
                break;
            }
            chain_depth = chain_depth.saturating_add(1);
            ensure_expr_depth(chain_depth)?;
            self.advance();
            let next_prec = if assoc_right { prec } else { prec.saturating_add(1) };
            let right = self.parse_binary_expr(next_prec, chain_depth)?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    /// Parses prefix unary operators or primary expressions. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-UNARY-OR-PRIMARY] | [FLS: fls_66m4rnbssgig]
    fn parse_unary_or_primary(&mut self, depth: usize) -> Result<Expr, String> {
        ensure_expr_depth(depth)?;
        if self.is_at_end() {
            return Err("Unexpected end of expression tokens".to_string());
        }
        let unary_op = match &self.peek().kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Exclamation => Some(UnaryOp::Not),
            _ => None,
        };
        if let Some(op) = unary_op {
            self.advance();
            let expr = self.parse_unary_or_primary(depth.saturating_add(1))?;
            Ok(Expr::Unary { op, expr: Box::new(expr) })
        } else {
            self.parse_primary(depth)
        }
    }

    /// Parses literals, identifiers, calls, parenthesized groups, and member chains. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-PRIMARY] | [FLS: fls_66m4rnbssgig]
    fn parse_primary(&mut self, depth: usize) -> Result<Expr, String> {
        ensure_expr_depth(depth)?;
        if self.is_at_end() {
            return Err("Expected primary expression, found EOF".to_string());
        }
        let next_depth = depth.saturating_add(1);
        let tok = self.advance();
        let expr = match &tok.kind {
            TokenKind::IntLit(n) => Expr::Number(*n as f64),
            TokenKind::RealLit(f) => Expr::Number(*f),
            TokenKind::StringLit(s) => Expr::StringLit(s.clone()),
            TokenKind::BoolLit(b) => Expr::BoolLit(*b),
            TokenKind::Ident(name) => {
                let ident = name.clone();
                if self.peek_kind() == Some(&TokenKind::OpenParen) {
                    self.parse_call_expr(ident, next_depth)?
                } else {
                    Expr::Ident(ident)
                }
            }
            TokenKind::OpenParen => {
                let inner = self.parse_binary_expr(0, next_depth)?;
                if self.peek_kind() == Some(&TokenKind::CloseParen) {
                    self.advance();
                    inner
                } else {
                    return Err("Expected matching ')' in parenthesized expression".to_string());
                }
            }
            other => return Err(format!("Unexpected token in expression: {other:?}")),
        };
        self.parse_member_chain(expr, depth)
    }

    /// Parses a parenthesized function argument list for `ident`. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-PRIMARY] | [FLS: fls_66m4rnbssgig]
    fn parse_call_expr(&mut self, ident: String, arg_depth: usize) -> Result<Expr, String> {
        self.advance();
        let mut args = Vec::new();
        while self.peek_kind() != Some(&TokenKind::CloseParen) && !self.is_at_end() {
            args.push(self.parse_binary_expr(0, arg_depth)?);
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        if self.peek_kind() == Some(&TokenKind::CloseParen) {
            self.advance();
            Ok(Expr::Call { func: ident, args })
        } else {
            Err(format!("Expected ')' after function arguments for {ident}"))
        }
    }

    /// Parses zero or more postfix `.field` member accesses. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PARSE-PRIMARY] | [FLS: fls_66m4rnbssgig]
    fn parse_member_chain(&mut self, mut expr: Expr, depth: usize) -> Result<Expr, String> {
        let mut member_depth = depth;
        while self.peek_kind() == Some(&TokenKind::Dot) {
            member_depth = member_depth.saturating_add(1);
            ensure_expr_depth(member_depth)?;
            self.advance();
            if let Some(TokenKind::Ident(field_name)) = self.peek_kind().cloned() {
                self.advance();
                expr = Expr::Member { object: Box::new(expr), member: field_name };
            } else {
                return Err("Expected field identifier following '.'".to_string());
            }
        }
        Ok(expr)
    }

    /// Peeks the current token or returns `EXPR_EOF_TOKEN`. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PEEK] | [FLS: fls_66m4rnbssgig]
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).or_else(|| self.tokens.last()).unwrap_or(&EXPR_EOF_TOKEN)
    }

    /// Peeks the current `TokenKind` when not at EOF. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PEEK-KIND] | [FLS: fls_66m4rnbssgig]
    fn peek_kind(&self) -> Option<&TokenKind> {
        (!self.is_at_end()).then(|| self.tokens.get(self.pos).map(|t| &t.kind)).flatten()
    }

    /// Advances the cursor and returns the consumed token. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-ADVANCE] | [FLS: fls_66m4rnbssgig]
    fn advance(&mut self) -> &Token {
        let idx = self.pos;
        if self.pos < self.tokens.len() {
            self.pos = self.pos.saturating_add(1);
        }
        self.tokens.get(idx).or_else(|| self.tokens.last()).unwrap_or(&EXPR_EOF_TOKEN)
    }

    /// Returns `true` when the cursor is at or past `TokenKind::Eof`. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-IS-AT-END] | [FLS: fls_66m4rnbssgig]
    fn is_at_end(&self) -> bool {
        self.tokens.get(self.pos).is_none_or(|t| t.kind == TokenKind::Eof)
    }

    /// Maps the current lookahead token to a `BinaryOp` if applicable. Realises: [REQ-PARSER-EXPRESSIONS-EXPRESSIONPARSER-PEEK-BINARY-OP] | [FLS: fls_66m4rnbssgig]
    fn peek_binary_op(&self) -> Option<BinaryOp> {
        Some(match self.peek_kind()? {
            TokenKind::Plus => BinaryOp::Add, TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul, TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod, TokenKind::Caret => BinaryOp::Pow,
            TokenKind::EqEq => BinaryOp::Eq, TokenKind::NotEq => BinaryOp::NotEq,
            TokenKind::Lt => BinaryOp::Lt, TokenKind::LtEq => BinaryOp::LtEq,
            TokenKind::Gt => BinaryOp::Gt, TokenKind::GtEq => BinaryOp::GtEq,
            TokenKind::AmpAmp => BinaryOp::And, TokenKind::PipePipe => BinaryOp::Or,
            TokenKind::Ident(s) if s == "and" => BinaryOp::And,
            TokenKind::Ident(s) if s == "or" => BinaryOp::Or,
            _ => return None,
        })
    }
}

/// Returns `(precedence, is_right_associative)` for `op`. Realises: [REQ-PARSER-EXPRESSIONS-BINARY-PRECEDENCE] | [FLS: fls_66m4rnbssgig]
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

/// Convenience function to parse tokens into an `Expr`. Realises: [REQ-PARSER-EXPRESSIONS-PARSE-EXPRESSION] | [FLS: fls_66m4rnbssgig]
pub fn parse_expression(tokens: &[Token]) -> Result<Expr, String> {
    ExpressionParser::new(tokens).parse()
}

//! Mathematical, logical, and formal KaTeX expression parser for SysML v2 constraints.

use crate::lexer::token::{Token, TokenKind};

/// Binary operations supported in SysML v2 constraints and expressions.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "not ",
        }
    }
}

/// Abstract syntax tree node for mathematical and boolean expressions.
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
pub struct ExpressionParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ExpressionParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse complete expression.
    pub fn parse(&mut self) -> Result<Expr, String> {
        let expr = self.parse_binary_expr(0)?;
        Ok(expr)
    }

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

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.pos].kind)
        }
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }

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
pub fn parse_expression(tokens: &[Token]) -> Result<Expr, String> {
    let mut parser = ExpressionParser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::scanner::Scanner;

    #[test]
    fn test_parse_inequality_constraint() {
        let mut scanner = Scanner::new("v <= 60.0");
        let tokens = scanner.scan_all().unwrap();
        let expr = parse_expression(&tokens).unwrap();

        assert_eq!(
            expr,
            Expr::Binary {
                op: BinaryOp::LtEq,
                left: Box::new(Expr::Ident("v".to_string())),
                right: Box::new(Expr::Number(60.0)),
            }
        );
        assert_eq!(expr.to_string_repr(), "v <= 60.0");
    }

    #[test]
    fn test_parse_nested_logical_expression() {
        let mut scanner = Scanner::new("alt >= 10.0 && alt <= 120.0");
        let tokens = scanner.scan_all().unwrap();
        let expr = parse_expression(&tokens).unwrap();

        match expr {
            Expr::Binary { op, left, right } => {
                assert_eq!(op, BinaryOp::And);
                assert_eq!(left.to_string_repr(), "alt >= 10.0");
                assert_eq!(right.to_string_repr(), "alt <= 120.0");
            }
            _ => panic!("Expected binary AND"),
        }
    }

    #[test]
    fn test_parse_member_and_function_call() {
        let mut scanner = Scanner::new("abs(fcc.speed) <= limits.max");
        let tokens = scanner.scan_all().unwrap();
        let expr = parse_expression(&tokens).unwrap();

        assert_eq!(expr.to_string_repr(), "abs(fcc.speed) <= limits.max");
    }
}

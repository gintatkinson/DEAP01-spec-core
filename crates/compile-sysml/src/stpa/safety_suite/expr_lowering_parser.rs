//! Bounded-depth precedence-climbing parser for SysML v2 expression token streams.
//!
//! Safety & Traceability: Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
//! Contract: Parses `SysmlToken` slices into `SysmlExprAst` trees respecting SysML v2 prefix, postfix, and 11-tier infix operator precedence while requiring complete token consumption.
//! Bounds & Faults: Caps recursion depth strictly at `MAX_EXPR_DEPTH = 64` and advances cursor state exclusively via `.get()` and `.saturating_add(1)`, returning `None` on malformed syntax or depth overflow.

use super::expr_lowering::{
    binary_binding_power, SysmlExprAst, SysmlToken, SysmlUnaryOp, MAX_EXPR_DEPTH,
};

/// Cursor-based, slice-indexing-free recursive-descent parser over `&[SysmlToken]` bounded by `MAX_EXPR_DEPTH`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) struct SysmlExprParser<'a> {
    tokens: &'a [SysmlToken],
    pos: usize,
}

impl<'a> SysmlExprParser<'a> {
    /// Inspects the current `SysmlToken` at `self.pos` via `.get(self.pos)` without advancing the cursor.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    fn peek(&self) -> Option<&'a SysmlToken> {
        self.tokens.get(self.pos)
    }

    /// Consumes and returns the current `SysmlToken` at `self.pos` via `.get(self.pos)`, advancing `self.pos` by one with saturating addition.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    fn next_token(&mut self) -> Option<&'a SysmlToken> {
        let tok = self.tokens.get(self.pos)?;
        self.pos = self.pos.saturating_add(1);
        Some(tok)
    }

    /// Parses a prefix-unary or infix-binary SysML subexpression above `min_prec`, returning `None` when `depth > MAX_EXPR_DEPTH`.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    fn parse_expr(&mut self, min_prec: u8, depth: usize) -> Option<SysmlExprAst> {
        if depth > MAX_EXPR_DEPTH {
            return None;
        }

        let mut lhs = match self.peek() {
            Some(SysmlToken::KeywordNot) => {
                let _ = self.next_token();
                let operand = self.parse_expr(50, depth.saturating_add(1))?;
                SysmlExprAst::Unary {
                    op: SysmlUnaryOp::Not,
                    operand: Box::new(operand),
                }
            }
            Some(SysmlToken::Bang | SysmlToken::Tilde) => {
                let _ = self.next_token();
                let operand = self.parse_expr(90, depth.saturating_add(1))?;
                SysmlExprAst::Unary {
                    op: SysmlUnaryOp::Not,
                    operand: Box::new(operand),
                }
            }
            Some(SysmlToken::Plus) => {
                let _ = self.next_token();
                let operand = self.parse_expr(90, depth.saturating_add(1))?;
                SysmlExprAst::Unary {
                    op: SysmlUnaryOp::Plus,
                    operand: Box::new(operand),
                }
            }
            Some(SysmlToken::Minus) => {
                let _ = self.next_token();
                let operand = self.parse_expr(90, depth.saturating_add(1))?;
                SysmlExprAst::Unary {
                    op: SysmlUnaryOp::Minus,
                    operand: Box::new(operand),
                }
            }
            _ => self.parse_postfix_primary(depth)?,
        };

        while let Some((op, lbp, rbp)) = self.peek().and_then(binary_binding_power) {
            if lbp < min_prec {
                break;
            }
            let _ = self.next_token();
            let rhs = self.parse_expr(rbp, depth.saturating_add(1))?;
            lhs = SysmlExprAst::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Some(lhs)
    }

    /// Parses primary atoms (`(expr)`, `Ident`, `Number`, `StringLit`) and left-associative postfix chains (`fn(args)`, `base.field`, `base[idx]`).
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    fn parse_postfix_primary(&mut self, depth: usize) -> Option<SysmlExprAst> {
        if depth > MAX_EXPR_DEPTH {
            return None;
        }

        let mut node = match self.next_token()? {
            SysmlToken::LParen => {
                let inner = self.parse_expr(0, depth.saturating_add(1))?;
                if !matches!(self.next_token(), Some(SysmlToken::RParen)) {
                    return None;
                }
                SysmlExprAst::Paren(Box::new(inner))
            }
            SysmlToken::Ident(name) => SysmlExprAst::Ident(name.clone()),
            SysmlToken::Number(num) => SysmlExprAst::Number(num.clone()),
            SysmlToken::StringLit(lit) => SysmlExprAst::StringLit(lit.clone()),
            _ => return None,
        };

        loop {
            match self.peek() {
                Some(SysmlToken::LParen) => {
                    let _ = self.next_token();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(SysmlToken::RParen)) {
                        let first_arg = self.parse_expr(0, depth.saturating_add(1))?;
                        args.push(first_arg);
                        while matches!(self.peek(), Some(SysmlToken::Comma)) {
                            let _ = self.next_token();
                            let next_arg = self.parse_expr(0, depth.saturating_add(1))?;
                            args.push(next_arg);
                        }
                    }
                    if !matches!(self.next_token(), Some(SysmlToken::RParen)) {
                        return None;
                    }
                    node = SysmlExprAst::Call {
                        callee: Box::new(node),
                        args,
                    };
                }
                Some(SysmlToken::Dot) => {
                    let _ = self.next_token();
                    let Some(SysmlToken::Ident(field)) = self.next_token() else {
                        return None;
                    };
                    node = SysmlExprAst::Field {
                        base: Box::new(node),
                        field: field.clone(),
                    };
                }
                Some(SysmlToken::LBracket) => {
                    let _ = self.next_token();
                    let mut indices = Vec::new();
                    let first_idx = self.parse_expr(0, depth.saturating_add(1))?;
                    indices.push(first_idx);
                    while matches!(self.peek(), Some(SysmlToken::Comma)) {
                        let _ = self.next_token();
                        let next_idx = self.parse_expr(0, depth.saturating_add(1))?;
                        indices.push(next_idx);
                    }
                    if !matches!(self.next_token(), Some(SysmlToken::RBracket)) {
                        return None;
                    }
                    node = SysmlExprAst::Index {
                        base: Box::new(node),
                        indices,
                    };
                }
                _ => break,
            }
        }

        Some(node)
    }
}

/// Parses a complete `SysmlToken` slice into a `SysmlExprAst`, returning `None` if empty, over-deep, or trailing tokens remain unconsumed.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn parse_sysml_expr(tokens: &[SysmlToken]) -> Option<SysmlExprAst> {
    if tokens.is_empty() {
        return None;
    }
    let mut parser = SysmlExprParser { tokens, pos: 0 };
    let ast = parser.parse_expr(0, 0)?;
    if parser.pos == tokens.len() {
        Some(ast)
    } else {
        None
    }
}

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Bounded-depth precedence-climbing parser for SysML v2 expression token streams.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! formal expression parsing objectives for module `crates/compile-sysml/src/stpa/safety_suite/expr_lowering_parser.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Parses `SysmlToken` slices into `SysmlExprAst` trees respecting SysML v2 prefix, postfix,
//!   and 11-tier infix operator precedence while requiring complete token consumption.
//!
//! ## 3. Preconditions
//! - Caller supplies a borrowed `&[SysmlToken]` slice produced by `tokenize_sysml_expr`.
//!
//! ## 4. Postconditions
//! - Returns `Some(SysmlExprAst)` iff the token stream forms a complete valid expression within `MAX_EXPR_DEPTH`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(T) linear in token count T.
//! - Space Complexity: O(D + T) bounded by `MAX_EXPR_DEPTH = 64` call stack and AST node count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Returns `None` on empty slices, syntax errors, trailing tokens, or depth overflow.
//! - State: Local cursor state confined to `SysmlExprParser`.
//! - Action: Advances cursor exclusively via `.get()` and `.saturating_add(1)` with zero panics.
//! - Resource: Bounded stack recursion depth (`<= 64`).
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: PackageDef(expr_lowering_parser)

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

    /// Parses comma-separated argument or index expressions terminated by `closing`.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    fn parse_delimited_list(
        &mut self,
        closing: &SysmlToken,
        allow_empty: bool,
        depth: usize,
    ) -> Option<Vec<SysmlExprAst>> {
        let mut items = Vec::new();
        if allow_empty && self.peek() == Some(closing) {
            let _ = self.next_token();
            return Some(items);
        }
        let first = self.parse_expr(0, depth.saturating_add(1))?;
        items.push(first);
        while matches!(self.peek(), Some(SysmlToken::Comma)) {
            let _ = self.next_token();
            let next_item = self.parse_expr(0, depth.saturating_add(1))?;
            items.push(next_item);
        }
        if self.next_token() == Some(closing) {
            Some(items)
        } else {
            None
        }
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
                    let args = self.parse_delimited_list(&SysmlToken::RParen, true, depth)?;
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
                    let indices = self.parse_delimited_list(&SysmlToken::RBracket, false, depth)?;
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

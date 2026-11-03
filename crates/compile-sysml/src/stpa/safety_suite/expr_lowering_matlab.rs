#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! MATLAB / Simulink Design Verifier (`SLDV`) target renderer and token-stream fallback for SysML v2 expressions.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! formal SLDV assertion synthesis objectives for module `crates/compile-sysml/src/stpa/safety_suite/expr_lowering_matlab.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Projects `SysmlExprAst` trees and fallback `SysmlToken` slices into `%`-comment-free MATLAB/SLDV boolean predicates.
//! - Lowers `implies` to `(~(lhs) || (rhs))`, `xor` to `xor(lhs, rhs)`, `!=`/`~=` to `~=`, and `%` to `mod(lhs, rhs)`.
//!
//! ## 3. Preconditions
//! - Caller supplies validated `SysmlExprAst` nodes or fallback `SysmlToken` slices.
//!
//! ## 4. Postconditions
//! - Emits non-empty MATLAB/SLDV expression strings free of raw SysML word operators and `%` comment characters.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) linear in AST node or token count.
//! - Space Complexity: O(N) bounded by `MAX_AST_RENDER_DEPTH = 512` recursion ceiling.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades to `"true"` when `depth > MAX_AST_RENDER_DEPTH` or token slice is empty.
//! - State: Pure stateless rendering functions.
//! - Action: Uses `.saturating_add(1)` with zero panics or unchecked slice indexing.
//! - Resource: Bounded stack recursion and string buffer allocations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: PackageDef(expr_lowering_matlab)

use super::expr_lowering::{
    is_binary_sysml_expr, SysmlBinOp, SysmlExprAst, SysmlToken, SysmlUnaryOp, MAX_AST_RENDER_DEPTH,
};

/// Renders a binary `SysmlBinOp` node and its operands into syntactically valid MATLAB / SLDV notation.
/// Realises: [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
fn render_binary_matlab(
    op: SysmlBinOp,
    lhs: &SysmlExprAst,
    rhs: &SysmlExprAst,
    next_depth: usize,
) -> String {
    let lhs_m = render_ast_matlab(lhs, next_depth);
    let rhs_m = render_ast_matlab(rhs, next_depth);
    match op {
        SysmlBinOp::Implies => format!("(~({}) || ({}))", lhs_m, rhs_m),
        SysmlBinOp::Or => format!("{} || {}", lhs_m, rhs_m),
        SysmlBinOp::Xor => format!("xor({}, {})", lhs_m, rhs_m),
        SysmlBinOp::And => format!("{} && {}", lhs_m, rhs_m),
        SysmlBinOp::Eq => format!("{} == {}", lhs_m, rhs_m),
        SysmlBinOp::Ne => format!("{} ~= {}", lhs_m, rhs_m),
        SysmlBinOp::Ge => format!("{} >= {}", lhs_m, rhs_m),
        SysmlBinOp::Le => format!("{} <= {}", lhs_m, rhs_m),
        SysmlBinOp::Gt => format!("{} > {}", lhs_m, rhs_m),
        SysmlBinOp::Lt => format!("{} < {}", lhs_m, rhs_m),
        SysmlBinOp::Add => format!("{} + {}", lhs_m, rhs_m),
        SysmlBinOp::Sub => format!("{} - {}", lhs_m, rhs_m),
        SysmlBinOp::Mul => format!("{} * {}", lhs_m, rhs_m),
        SysmlBinOp::Div => format!("{} / {}", lhs_m, rhs_m),
        SysmlBinOp::Rem => format!("mod({}, {})", lhs_m, rhs_m),
        SysmlBinOp::Pow => {
            let lhs_wrapped = if is_binary_sysml_expr(lhs) {
                format!("({})", lhs_m)
            } else {
                lhs_m
            };
            let rhs_wrapped = if matches!(
                rhs,
                SysmlExprAst::Binary {
                    op: SysmlBinOp::Pow,
                    ..
                }
            ) {
                format!("({})", rhs_m)
            } else {
                rhs_m
            };
            format!("{} ^ {}", lhs_wrapped, rhs_wrapped)
        }
    }
}

/// Recursively renders a validated `SysmlExprAst` into a syntactically valid MATLAB / Simulink Design Verifier (`SLDV`) expression bounded by `MAX_AST_RENDER_DEPTH`.
/// Realises: [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn render_ast_matlab(ast: &SysmlExprAst, depth: usize) -> String {
    if depth > MAX_AST_RENDER_DEPTH {
        return "true".to_string();
    }
    let next_depth = depth.saturating_add(1);
    match ast {
        SysmlExprAst::Ident(id) => id.clone(),
        SysmlExprAst::Number(num) => num.clone(),
        SysmlExprAst::StringLit(lit) => format!("'{}'", lit.replace('\'', "''")),
        SysmlExprAst::Paren(inner) => format!("({})", render_ast_matlab(inner, next_depth)),
        SysmlExprAst::Unary { op, operand } => {
            let inner = render_ast_matlab(operand, next_depth);
            let grouped =
                is_binary_sysml_expr(operand) || matches!(operand.as_ref(), SysmlExprAst::Unary { .. });
            match op {
                SysmlUnaryOp::Not if matches!(operand.as_ref(), SysmlExprAst::Paren(_)) => {
                    format!("~{}", inner)
                }
                SysmlUnaryOp::Not => format!("~({})", inner),
                SysmlUnaryOp::Plus if grouped => format!("+({})", inner),
                SysmlUnaryOp::Plus => format!("+{}", inner),
                SysmlUnaryOp::Minus if grouped => format!("-({})", inner),
                SysmlUnaryOp::Minus => format!("-{}", inner),
            }
        }
        SysmlExprAst::Binary { op, lhs, rhs } => render_binary_matlab(*op, lhs, rhs, next_depth),
        SysmlExprAst::Call { callee, args } => {
            let callee_m = render_ast_matlab(callee, next_depth);
            let args_m: Vec<String> = args
                .iter()
                .map(|a| render_ast_matlab(a, next_depth))
                .collect();
            format!("{}({})", callee_m, args_m.join(", "))
        }
        SysmlExprAst::Field { base, field } => {
            let base_m = render_ast_matlab(base, next_depth);
            if is_binary_sysml_expr(base) {
                format!("({}).{}", base_m, field)
            } else {
                format!("{}.{}", base_m, field)
            }
        }
        SysmlExprAst::Index { base, indices } => {
            let base_m = render_ast_matlab(base, next_depth);
            if matches!(base.as_ref(), SysmlExprAst::Number(_)) {
                base_m
            } else {
                let idx_m: Vec<String> = indices
                    .iter()
                    .map(|i| render_ast_matlab(i, next_depth))
                    .collect();
                if is_binary_sysml_expr(base) {
                    format!("({})({})", base_m, idx_m.join(", "))
                } else {
                    format!("{}({})", base_m, idx_m.join(", "))
                }
            }
        }
    }
}

/// Lowers a raw `SysmlToken` stream token-by-token into `%`-free MATLAB-safe tokens when full AST parsing rejects malformed or over-deep input.
/// Realises: [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn fallback_tokens_to_matlab(tokens: &[SysmlToken]) -> String {
    let mut parts = Vec::with_capacity(tokens.len());
    for tok in tokens {
        let piece = match tok {
            SysmlToken::Implies | SysmlToken::Or => "||".to_string(),
            SysmlToken::Xor | SysmlToken::Ne => "~=".to_string(),
            SysmlToken::And => "&&".to_string(),
            SysmlToken::KeywordNot | SysmlToken::Bang | SysmlToken::Tilde => "~".to_string(),
            SysmlToken::Eq => "==".to_string(),
            SysmlToken::Ge => ">=".to_string(),
            SysmlToken::Le => "<=".to_string(),
            SysmlToken::Gt => ">".to_string(),
            SysmlToken::Lt => "<".to_string(),
            SysmlToken::Plus => "+".to_string(),
            SysmlToken::Minus => "-".to_string(),
            SysmlToken::Star => "*".to_string(),
            SysmlToken::Slash | SysmlToken::Percent => "/".to_string(),
            SysmlToken::Pow => "^".to_string(),
            SysmlToken::LParen | SysmlToken::LBracket => "(".to_string(),
            SysmlToken::RParen | SysmlToken::RBracket => ")".to_string(),
            SysmlToken::Comma => ",".to_string(),
            SysmlToken::Dot => ".".to_string(),
            SysmlToken::Ident(id) => id.clone(),
            SysmlToken::Number(n) => n.clone(),
            SysmlToken::StringLit(s) => format!("'{}'", s.replace('\'', "''")),
            SysmlToken::Unknown(_) => String::new(),
        };
        if !piece.is_empty() {
            parts.push(piece);
        }
    }
    if parts.is_empty() {
        "true".to_string()
    } else {
        parts.join(" ")
    }
}

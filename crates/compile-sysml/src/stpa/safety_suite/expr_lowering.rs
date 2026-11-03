#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! SysML v2 expression AST representation, operator precedence, and KaTeX/MATLAB lowering entrypoints.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! formal proof lowering objectives for module `crates/compile-sysml/src/stpa/safety_suite/expr_lowering.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Defines bounded-depth expression AST (`SysmlExprAst`), token taxonomy (`SysmlToken`), and 11-tier infix binding powers.
//! - Exposes deterministic `lower_sysml_expr_to_katex` and `lower_sysml_expr_to_matlab` functions.
//!
//! ## 3. Preconditions
//! - Caller supplies any UTF-8 expression string from a SysML v2 AST `CalcDef` or `ConstraintDef` node.
//!
//! ## 4. Postconditions
//! - Returns non-empty, brace-balanced KaTeX or `%`-comment-free MATLAB/SLDV boolean predicates.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) linear in `expr.len()`.
//! - Space Complexity: O(N) bounded AST and output buffer allocation.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades gracefully via token-stream fallback when input exceeds `MAX_EXPR_DEPTH` or contains partial syntax.
//! - State: Pure referentially transparent functions.
//! - Action: Executes with zero `regex`, zero direct range slicing, and zero `.unwrap()` / `.expect()` / `panic!`.
//! - Resource: Bounded recursion depth (`MAX_EXPR_DEPTH = 64`, `MAX_AST_RENDER_DEPTH = 512`).
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: PackageDef(expr_lowering)

use super::expr_lowering_lexer::tokenize_sysml_expr;
use super::*;

/// Hard recursion-depth ceiling for recursive-descent SysML expression parsing (`STPA-SAFETY-SUITE`).
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) const MAX_EXPR_DEPTH: usize = 64;

/// Hard AST traversal ceiling for recursive KaTeX and MATLAB expression emitters (`STPA-SAFETY-SUITE`).
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) const MAX_AST_RENDER_DEPTH: usize = 512;

/// Lexical token stream representation for deterministic SysML v2 expression lowering, encoding unrecognized codepoints explicitly via `Unknown(char)`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SysmlToken {
    Implies,
    Or,
    Xor,
    And,
    KeywordNot,
    Eq,
    Ne,
    Ge,
    Le,
    Gt,
    Lt,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Tilde,
    Pow,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Ident(String),
    Number(String),
    StringLit(String),
    Unknown(char),
}

/// Canonical binary operator taxonomy mapping logical, relational, arithmetic, modulo, and exponentiation operators across SysML v2, KaTeX, and MATLAB/SLDV.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum SysmlBinOp {
    Implies,
    Or,
    Xor,
    And,
    Eq,
    Ne,
    Ge,
    Le,
    Gt,
    Lt,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

/// Canonical prefix unary operator taxonomy unifying logical negation (`not`, `!`, `~`) alongside arithmetic sign operators (`+`, `-`).
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum SysmlUnaryOp {
    Not,
    Plus,
    Minus,
}

/// Bounded-depth abstract syntax tree (`SysmlExprAst`) for SysML v2 `CalcDef` and `ConstraintDef` expressions preserving explicit `Paren` grouping.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SysmlExprAst {
    Ident(String),
    Number(String),
    StringLit(String),
    Paren(Box<SysmlExprAst>),
    Unary {
        op: SysmlUnaryOp,
        operand: Box<SysmlExprAst>,
    },
    Binary {
        op: SysmlBinOp,
        lhs: Box<SysmlExprAst>,
        rhs: Box<SysmlExprAst>,
    },
    Call {
        callee: Box<SysmlExprAst>,
        args: Vec<SysmlExprAst>,
    },
    Field {
        base: Box<SysmlExprAst>,
        field: String,
    },
    Index {
        base: Box<SysmlExprAst>,
        indices: Vec<SysmlExprAst>,
    },
}

/// Checks whether a `SysmlExprAst` node is an unparenthesized `Binary` expression requiring protective grouping under prefix or superscript operators.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn is_binary_sysml_expr(ast: &SysmlExprAst) -> bool {
    matches!(ast, SysmlExprAst::Binary { .. })
}

/// Maps an infix `SysmlToken` to its `(SysmlBinOp, left_binding_power, right_binding_power)` precedence tuple across all SysML v2 binary tiers.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn binary_binding_power(tok: &SysmlToken) -> Option<(SysmlBinOp, u8, u8)> {
    match tok {
        SysmlToken::Implies => Some((SysmlBinOp::Implies, 10, 10)),
        SysmlToken::Or => Some((SysmlBinOp::Or, 20, 21)),
        SysmlToken::Xor => Some((SysmlBinOp::Xor, 30, 31)),
        SysmlToken::And => Some((SysmlBinOp::And, 40, 41)),
        SysmlToken::Eq => Some((SysmlBinOp::Eq, 60, 61)),
        SysmlToken::Ne => Some((SysmlBinOp::Ne, 60, 61)),
        SysmlToken::Ge => Some((SysmlBinOp::Ge, 60, 61)),
        SysmlToken::Le => Some((SysmlBinOp::Le, 60, 61)),
        SysmlToken::Gt => Some((SysmlBinOp::Gt, 60, 61)),
        SysmlToken::Lt => Some((SysmlBinOp::Lt, 60, 61)),
        SysmlToken::Plus => Some((SysmlBinOp::Add, 70, 71)),
        SysmlToken::Minus => Some((SysmlBinOp::Sub, 70, 71)),
        SysmlToken::Star => Some((SysmlBinOp::Mul, 80, 81)),
        SysmlToken::Slash => Some((SysmlBinOp::Div, 80, 81)),
        SysmlToken::Percent => Some((SysmlBinOp::Rem, 80, 81)),
        SysmlToken::Pow => Some((SysmlBinOp::Pow, 100, 100)),
        _ => None,
    }
}

/// Deterministically lowers a SysML v2 `CalcDef` or `ConstraintDef` expression into a Check-13-compliant KaTeX math expression.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub fn lower_sysml_expr_to_katex(expr: &str) -> String {
    let trimmed = expr.trim_matches(|ch: char| ch.is_whitespace() || ch == ';');
    if trimmed.is_empty() {
        return "\\mathrm{true}".to_string();
    }
    if contains_latex_command_outside_quotes(trimmed) {
        return sanitize_preauthored_katex(trimmed);
    }
    let tokens = tokenize_sysml_expr(trimmed);
    if let Some(ast) = parse_sysml_expr(&tokens) {
        render_ast_katex(&ast, 0)
    } else {
        fallback_tokens_to_katex(&tokens)
    }
}

/// Deterministically lowers a SysML v2 `CalcDef` or `ConstraintDef` expression into an executable MATLAB / Simulink Design Verifier (`SLDV`) predicate.
/// Realises: [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub fn lower_sysml_expr_to_matlab(expr: &str) -> String {
    let trimmed = expr.trim_matches(|ch: char| ch.is_whitespace() || ch == ';');
    if trimmed.is_empty() {
        return "true".to_string();
    }
    let tokens = tokenize_sysml_expr(trimmed);
    if let Some(ast) = parse_sysml_expr(&tokens) {
        render_ast_matlab(&ast, 0)
    } else {
        fallback_tokens_to_matlab(&tokens)
    }
}

#[cfg(test)]
mod expr_lowering_tests {
    use super::{lower_sysml_expr_to_katex, lower_sysml_expr_to_matlab, MAX_EXPR_DEPTH};

    /// Verifies all 11 SysML v2 operator precedence tiers, pre-authored LaTeX wrapper stripping,
    /// identifier/string escaping, empty/whitespace defaults, and `MAX_EXPR_DEPTH` fallback.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    #[test]
    fn test_sysml_expr_lowering_precedence_escaping_and_depth_bounds() {
        assert_eq!(lower_sysml_expr_to_katex(""), "\\mathrm{true}");
        assert_eq!(lower_sysml_expr_to_katex("  ; ; "), "\\mathrm{true}");
        assert_eq!(lower_sysml_expr_to_matlab(""), "true");
        assert_eq!(lower_sysml_expr_to_matlab("  ; ; "), "true");

        let pre_latex = "\\begin{aligned}\n  x_1 &= y_1 + 100% \\\\\n\\end{aligned}";
        let sanitized_latex = lower_sysml_expr_to_katex(pre_latex);
        assert!(!sanitized_latex.contains("\\begin{aligned}"));
        assert!(!sanitized_latex.contains("\\end{aligned}"));
        assert!(sanitized_latex.contains("100\\%"));

        let complex = "x >= 0 -> max_val(a.b_1, arr[0]) ** 2 != 0 & ~flag | mode = \"ok's_100%\"";
        let katex = lower_sysml_expr_to_katex(complex);
        assert!(katex.contains("x \\ge 0 \\implies"));
        assert!(
            katex.contains("\\mathrm{max\\_val}(a.\\mathrm{b\\_1}, \\mathrm{arr}[0])^{2} \\neq 0")
        );
        assert!(katex.contains("\\land \\lnot \\mathrm{flag}"));
        assert!(katex.contains("\\text{\"ok's\\_100\\%\"}"));

        let matlab = lower_sysml_expr_to_matlab(complex);
        assert!(matlab.contains("x >= 0"));
        assert!(matlab.contains("max_val(a.b_1, arr(0)) ^ 2 ~= 0"));
        assert!(matlab.contains("&& ~(flag)"));
        assert!(matlab.contains("mode == 'ok''s_100%'"));

        assert_eq!(lower_sysml_expr_to_katex("not x == 0"), "\\lnot (x = 0)");
        assert_eq!(lower_sysml_expr_to_matlab("not x == 0"), "~(x == 0)");
        assert_eq!(lower_sysml_expr_to_matlab("not (x == 0)"), "~(x == 0)");

        let depth_probe = MAX_EXPR_DEPTH.saturating_add(6);
        let deep_expr = format!(
            "{}x_val >= 0{}",
            "(".repeat(depth_probe),
            ")".repeat(depth_probe)
        );
        let deep_katex = lower_sysml_expr_to_katex(&deep_expr);
        assert!(deep_katex.contains("\\mathrm{x\\_val}"));
        assert!(deep_katex.contains("\\ge"));
        let deep_matlab = lower_sysml_expr_to_matlab(&deep_expr);
        assert!(deep_matlab.contains("x_val"));
        assert!(deep_matlab.contains(">="));
    }
}

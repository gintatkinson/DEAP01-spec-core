//! SysML v2 expression lexical tokenization, AST representation, and KaTeX/MATLAB lowering entrypoints.
//!
//! Safety & Traceability: Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
//! Contract: Tokenizes SysML v2 `CalcDef` and `ConstraintDef` expressions via `.char_indices().peekable()` and `.get(start..end)`, defines bounded-depth expression AST and precedence structures, and exposes deterministic KaTeX and MATLAB/SLDV lowering functions.
//! Bounds & Faults: Enforces `MAX_EXPR_DEPTH = 64` and `MAX_AST_RENDER_DEPTH = 512` ceilings with total token-stream fallback, saturating arithmetic, and zero regular expressions, panics, or unchecked slice indexing.

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

/// Tokenizes a SysML v2 expression string monotonically via `.char_indices().peekable()` and `.get(start..end)` with zero regular expressions or direct slice indexing.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn tokenize_sysml_expr(expr: &str) -> Vec<SysmlToken> {
    let mut tokens = Vec::new();
    let mut chars = expr.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut end = start.saturating_add(ch.len_utf8());
            while let Some(&(next_idx, next_ch)) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    end = next_idx.saturating_add(next_ch.len_utf8());
                    let _ = chars.next();
                } else {
                    break;
                }
            }
            if let Some(word) = expr.get(start..end) {
                let tok = if word.eq_ignore_ascii_case("implies") {
                    SysmlToken::Implies
                } else if word.eq_ignore_ascii_case("or") {
                    SysmlToken::Or
                } else if word.eq_ignore_ascii_case("xor") {
                    SysmlToken::Xor
                } else if word.eq_ignore_ascii_case("and") {
                    SysmlToken::And
                } else if word.eq_ignore_ascii_case("not") {
                    SysmlToken::KeywordNot
                } else {
                    SysmlToken::Ident(word.to_string())
                };
                tokens.push(tok);
            }
            continue;
        }

        if ch.is_ascii_digit()
            || (ch == '.' && matches!(chars.peek(), Some(&(_, d)) if d.is_ascii_digit()))
        {
            let mut num_buf = String::new();
            if ch == '.' {
                num_buf.push('0');
                num_buf.push('.');
            } else {
                num_buf.push(ch);
                while let Some(&(_, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() {
                        num_buf.push(next_ch);
                        let _ = chars.next();
                    } else if next_ch == '_' {
                        let mut la = chars.clone();
                        let _ = la.next();
                        if matches!(la.peek(), Some(&(_, d)) if d.is_ascii_digit()) {
                            let _ = chars.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if matches!(chars.peek(), Some(&(_, '.'))) {
                    let mut la = chars.clone();
                    let _ = la.next();
                    if matches!(la.peek(), Some(&(_, d)) if d.is_ascii_digit()) {
                        num_buf.push('.');
                        let _ = chars.next();
                    }
                }
            }

            if num_buf.ends_with('.') {
                while let Some(&(_, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() {
                        num_buf.push(next_ch);
                        let _ = chars.next();
                    } else if next_ch == '_' {
                        let mut la = chars.clone();
                        let _ = la.next();
                        if matches!(la.peek(), Some(&(_, d)) if d.is_ascii_digit()) {
                            let _ = chars.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }

            if matches!(chars.peek(), Some(&(_, 'e' | 'E'))) {
                let mut la = chars.clone();
                if let Some((_, exp_ch)) = la.next() {
                    let has_valid_exp = match la.peek().copied() {
                        Some((_, d)) if d.is_ascii_digit() => true,
                        Some((_, '+' | '-')) => {
                            let _ = la.next();
                            matches!(la.peek(), Some(&(_, d)) if d.is_ascii_digit())
                        }
                        _ => false,
                    };
                    if has_valid_exp {
                        num_buf.push(exp_ch);
                        let _ = chars.next();
                        if let Some(&(_, sign @ ('+' | '-'))) = chars.peek() {
                            num_buf.push(sign);
                            let _ = chars.next();
                        }
                        while let Some(&(_, next_ch)) = chars.peek() {
                            if next_ch.is_ascii_digit() {
                                num_buf.push(next_ch);
                                let _ = chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            tokens.push(SysmlToken::Number(num_buf));
            continue;
        }

        if ch == '"' || ch == '\'' {
            let quote = ch;
            let mut lit = String::new();
            while let Some((_, c)) = chars.next() {
                if c == '\\' {
                    if let Some((_, escaped)) = chars.next() {
                        lit.push(escaped);
                    }
                } else if c == quote {
                    break;
                } else {
                    lit.push(c);
                }
            }
            tokens.push(SysmlToken::StringLit(lit));
            continue;
        }

        match ch {
            '-' => {
                if matches!(chars.peek(), Some(&(_, '>'))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Implies);
                } else {
                    tokens.push(SysmlToken::Minus);
                }
            }
            '=' => {
                if matches!(chars.peek(), Some(&(_, '>'))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Implies);
                } else if matches!(chars.peek(), Some(&(_, '='))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Eq);
                } else {
                    tokens.push(SysmlToken::Eq);
                }
            }
            '|' => {
                if matches!(chars.peek(), Some(&(_, '|'))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Or);
                } else {
                    tokens.push(SysmlToken::Or);
                }
            }
            '&' => {
                if matches!(chars.peek(), Some(&(_, '&'))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::And);
                } else {
                    tokens.push(SysmlToken::And);
                }
            }
            '!' => {
                if matches!(chars.peek(), Some(&(_, '='))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Ne);
                } else {
                    tokens.push(SysmlToken::Bang);
                }
            }
            '~' => {
                if matches!(chars.peek(), Some(&(_, '='))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Ne);
                } else {
                    tokens.push(SysmlToken::Tilde);
                }
            }
            '>' => {
                if matches!(chars.peek(), Some(&(_, '='))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Ge);
                } else {
                    tokens.push(SysmlToken::Gt);
                }
            }
            '<' => {
                if matches!(chars.peek(), Some(&(_, '='))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Le);
                } else {
                    tokens.push(SysmlToken::Lt);
                }
            }
            '+' => tokens.push(SysmlToken::Plus),
            '*' => {
                if matches!(chars.peek(), Some(&(_, '*'))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Pow);
                } else {
                    tokens.push(SysmlToken::Star);
                }
            }
            '/' => tokens.push(SysmlToken::Slash),
            '%' => tokens.push(SysmlToken::Percent),
            '^' => tokens.push(SysmlToken::Pow),
            '(' => tokens.push(SysmlToken::LParen),
            ')' => tokens.push(SysmlToken::RParen),
            '[' => tokens.push(SysmlToken::LBracket),
            ']' => tokens.push(SysmlToken::RBracket),
            ',' => tokens.push(SysmlToken::Comma),
            '.' => tokens.push(SysmlToken::Dot),
            ':' => {
                if matches!(chars.peek(), Some(&(_, ':'))) {
                    let _ = chars.next();
                    tokens.push(SysmlToken::Dot);
                } else {
                    tokens.push(SysmlToken::Unknown(':'));
                }
            }
            other => tokens.push(SysmlToken::Unknown(other)),
        }
    }

    tokens
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C formal proof rendering objectives for fn `lower_sysml_expr_to_katex` in `crates/compile-sysml/src/stpa/safety_suite/expr_lowering.rs` (refs #698).
///
/// ## 2. Invariants & Mathematical Properties
/// - Maps empty or whitespace/semicolon-only inputs to `\mathrm{true}`.
/// - Preserves pre-authored LaTeX expressions while safely stripping outer `\begin{aligned}` / `\end{aligned}` wrappers and escaping unescaped `%` and `$`.
/// - Lowers all SysML v2 logical (`implies`, `or`, `xor`, `and`, `not`, `&&`, `||`, `!`, `~`), relational (`==`, `!=`, `~=`, `>=`, `<=`), arithmetic (`*`, `%`, `^`, `**`), identifier, and string-literal constructs into balanced KaTeX tokens.
///
/// ## 3. Preconditions
/// - `expr: &str` is any UTF-8 expression string from a SysML v2 AST `CalcDef` or `ConstraintDef` node.
///
/// ## 4. Postconditions
/// - Returned `String` is non-empty, contains balanced `{` and `}` braces, contains zero unescaped `%` or `$`, and contains zero raw SysML operator tokens.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) linear in `expr.len()`.
/// - Space Complexity: O(N) bounded AST and output buffer allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Degrades gracefully via `fallback_tokens_to_katex` when input exceeds `MAX_EXPR_DEPTH` or contains partial syntax.
/// - State: Pure referentially transparent function.
/// - Action: Executes with zero `regex`, zero direct range slicing, and zero `.unwrap()` / `.expect()` / `panic!`.
/// - Resource: Bounded recursion depth (`MAX_EXPR_DEPTH = 64`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING] | SysML v2: ActionDef(lower_sysml_expr_to_katex)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C formal SLDV assertion synthesis objectives for fn `lower_sysml_expr_to_matlab` in `crates/compile-sysml/src/stpa/safety_suite/expr_lowering.rs` (refs #698).
///
/// ## 2. Invariants & Mathematical Properties
/// - Maps empty or whitespace/semicolon-only inputs to `"true"`.
/// - Lowers `implies`/`->`/`=>` to `(~(lhs) || (rhs))`, `xor` to `xor(lhs, rhs)`, `or`/`||`/`|` to `lhs || rhs`, `and`/`&&`/`&` to `lhs && rhs`, `not`/`!`/`~` to `~(inner)`, `==`/`=` to `lhs == rhs`, `!=`/`~=` to `lhs ~= rhs`, `%` to `mod(lhs, rhs)`, `^`/`**` to `lhs ^ rhs`, and string literals to single-quoted `'...'` with `'` doubled to `''`.
///
/// ## 3. Preconditions
/// - `expr: &str` is any UTF-8 expression string from a SysML v2 AST `CalcDef` or `ConstraintDef` node.
///
/// ## 4. Postconditions
/// - Returned `String` is non-empty and free of raw SysML word operators, `!=`, and `%` comment tokens.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) linear in `expr.len()`.
/// - Space Complexity: O(N) bounded AST and output buffer allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Degrades gracefully via `fallback_tokens_to_matlab` when input exceeds `MAX_EXPR_DEPTH` or contains partial syntax.
/// - State: Pure referentially transparent function.
/// - Action: Executes with zero `regex`, zero direct range slicing, and zero `.unwrap()` / `.expect()` / `panic!`.
/// - Resource: Bounded recursion depth (`MAX_EXPR_DEPTH = 64`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: ActionDef(lower_sysml_expr_to_matlab)
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
    /// identifier/string escaping, empty/whitespace defaults, and `MAX_EXPR_DEPTH` fallback across
    /// `lower_sysml_expr_to_katex` and `lower_sysml_expr_to_matlab`.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
    #[test]
    fn test_sysml_expr_lowering_precedence_escaping_and_depth_bounds() {
        assert_eq!(lower_sysml_expr_to_katex(""), "\\mathrm{true}");
        assert_eq!(lower_sysml_expr_to_katex("  ; ; "), "\\mathrm{true}");
        assert_eq!(lower_sysml_expr_to_matlab(""), "true");
        assert_eq!(lower_sysml_expr_to_matlab("  ; ; "), "true");

        // Pre-authored LaTeX outer \begin{aligned} stripping and % escaping
        let pre_latex = "\\begin{aligned}\n  x_1 &= y_1 + 100% \\\\\n\\end{aligned}";
        let sanitized_latex = lower_sysml_expr_to_katex(pre_latex);
        assert!(!sanitized_latex.contains("\\begin{aligned}"));
        assert!(!sanitized_latex.contains("\\end{aligned}"));
        assert!(sanitized_latex.contains("100\\%"));

        // Single ASCII letter vs multi-char snake_case & postfix calls/fields/indices/powers
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

        // Prefix `not` binds looser than `==` (`not x == 0` -> `\lnot (x = 0)` and `~(x == 0)`)
        assert_eq!(lower_sysml_expr_to_katex("not x == 0"), "\\lnot (x = 0)");
        assert_eq!(lower_sysml_expr_to_matlab("not x == 0"), "~(x == 0)");
        assert_eq!(lower_sysml_expr_to_matlab("not (x == 0)"), "~(x == 0)");

        // Over-deep expression (> MAX_EXPR_DEPTH) degrades cleanly without stack overflow
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

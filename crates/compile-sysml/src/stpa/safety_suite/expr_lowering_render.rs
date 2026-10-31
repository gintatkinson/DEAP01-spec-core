//! KaTeX math and MATLAB/SLDV target renderers and total token-stream fallbacks for SysML v2 expressions.
//!
//! Safety & Traceability: Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
//! Contract: Projects `SysmlExprAst` trees and fallback `SysmlToken` slices into brace-balanced Check-13 KaTeX fragments and `%`-comment-free MATLAB/SLDV boolean predicates while preserving pre-authored LaTeX commands safely.
//! Bounds & Faults: Enforces `MAX_AST_RENDER_DEPTH = 512` across recursive AST projections and executes total linear fallback scans with `.strip_prefix()`, `.strip_suffix()`, and `.saturating_add()`.

use super::expr_lowering::{
    is_binary_sysml_expr, SysmlBinOp, SysmlExprAst, SysmlToken, SysmlUnaryOp, MAX_AST_RENDER_DEPTH,
};

/// Escapes KaTeX special characters (`_`, `%`, `&`, `#`, `$`, `{`, `}`, `\`) inside identifier and string-literal payloads to preserve balanced braces.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub(super) fn escape_katex_ident(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().saturating_add(8));
    for ch in raw.chars() {
        match ch {
            '_' => out.push_str("\\_"),
            '%' => out.push_str("\\%"),
            '&' => out.push_str("\\&"),
            '#' => out.push_str("\\#"),
            '$' => out.push_str("\\$"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\\' => out.push_str("\\textbackslash{}"),
            other => out.push(other),
        }
    }
    out
}

/// Formats a SysML identifier for KaTeX math mode, keeping single ASCII letters italic and wrapping multi-character identifiers in `\mathrm{...}`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub(super) fn render_katex_ident(ident: &str) -> String {
    if ident.len() == 1 && ident.chars().all(|ch| ch.is_ascii_alphabetic()) {
        ident.to_string()
    } else {
        format!("\\mathrm{{{}}}", escape_katex_ident(ident))
    }
}

/// Detects whether an expression string already contains pre-authored LaTeX commands (`\` outside single- or double-quoted string literals).
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub(super) fn contains_latex_command_outside_quotes(expr: &str) -> bool {
    let mut in_quote: Option<char> = None;
    let mut escaped_in_quote = false;
    for ch in expr.chars() {
        if let Some(q) = in_quote {
            if escaped_in_quote {
                escaped_in_quote = false;
            } else if ch == '\\' {
                escaped_in_quote = true;
            } else if ch == q {
                in_quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            in_quote = Some(ch);
        } else if ch == '\\' {
            return true;
        }
    }
    false
}

/// Strips outer `$$` and `\begin{aligned}` / `\end{aligned}` wrappers from pre-authored LaTeX expressions and escapes unescaped `%` and `$` characters.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub(super) fn sanitize_preauthored_katex(expr: &str) -> String {
    let mut current = expr.trim();
    if let Some(inner) = current
        .strip_prefix("$$")
        .and_then(|s| s.strip_suffix("$$"))
    {
        current = inner.trim();
    }
    while let Some(inner) = current
        .strip_prefix("\\begin{aligned}")
        .and_then(|s| s.strip_suffix("\\end{aligned}"))
    {
        current = inner.trim();
        if let Some(without_row_break) = current.strip_suffix("\\\\") {
            if !without_row_break.ends_with('\\') {
                current = without_row_break.trim();
            }
        }
    }
    if current.is_empty() {
        return "\\mathrm{true}".to_string();
    }

    let mut out = String::with_capacity(current.len().saturating_add(4));
    let mut prev_escaped = false;
    for ch in current.chars() {
        if (ch == '%' || ch == '$') && !prev_escaped {
            out.push('\\');
            out.push(ch);
        } else {
            out.push(ch);
        }
        prev_escaped = ch == '\\' && !prev_escaped;
    }
    out
}

/// Recursively renders a validated `SysmlExprAst` into a brace-balanced Check-13 KaTeX math expression bounded by `MAX_AST_RENDER_DEPTH`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub(super) fn render_ast_katex(ast: &SysmlExprAst, depth: usize) -> String {
    if depth > MAX_AST_RENDER_DEPTH {
        return "\\mathrm{true}".to_string();
    }
    let next_depth = depth.saturating_add(1);
    match ast {
        SysmlExprAst::Ident(id) => render_katex_ident(id),
        SysmlExprAst::Number(num) => num.clone(),
        SysmlExprAst::StringLit(lit) => format!("\\text{{\"{}\"}}", escape_katex_ident(lit)),
        SysmlExprAst::Paren(inner) => format!("({})", render_ast_katex(inner, next_depth)),
        SysmlExprAst::Unary { op, operand } => {
            let inner = render_ast_katex(operand, next_depth);
            match op {
                SysmlUnaryOp::Not => {
                    if is_binary_sysml_expr(operand) {
                        format!("\\lnot ({})", inner)
                    } else {
                        format!("\\lnot {}", inner)
                    }
                }
                SysmlUnaryOp::Plus => {
                    if is_binary_sysml_expr(operand)
                        || matches!(operand.as_ref(), SysmlExprAst::Unary { .. })
                    {
                        format!("+({})", inner)
                    } else {
                        format!("+{}", inner)
                    }
                }
                SysmlUnaryOp::Minus => {
                    if is_binary_sysml_expr(operand)
                        || matches!(operand.as_ref(), SysmlExprAst::Unary { .. })
                    {
                        format!("-({})", inner)
                    } else {
                        format!("-{}", inner)
                    }
                }
            }
        }
        SysmlExprAst::Binary { op, lhs, rhs } => {
            let lhs_k = render_ast_katex(lhs, next_depth);
            let rhs_k = render_ast_katex(rhs, next_depth);
            match op {
                SysmlBinOp::Implies => format!("{} \\implies {}", lhs_k, rhs_k),
                SysmlBinOp::Or => format!("{} \\lor {}", lhs_k, rhs_k),
                SysmlBinOp::Xor => format!("{} \\oplus {}", lhs_k, rhs_k),
                SysmlBinOp::And => format!("{} \\land {}", lhs_k, rhs_k),
                SysmlBinOp::Eq => format!("{} = {}", lhs_k, rhs_k),
                SysmlBinOp::Ne => format!("{} \\neq {}", lhs_k, rhs_k),
                SysmlBinOp::Ge => format!("{} \\ge {}", lhs_k, rhs_k),
                SysmlBinOp::Le => format!("{} \\le {}", lhs_k, rhs_k),
                SysmlBinOp::Gt => format!("{} > {}", lhs_k, rhs_k),
                SysmlBinOp::Lt => format!("{} < {}", lhs_k, rhs_k),
                SysmlBinOp::Add => format!("{} + {}", lhs_k, rhs_k),
                SysmlBinOp::Sub => format!("{} - {}", lhs_k, rhs_k),
                SysmlBinOp::Mul => format!("{} \\cdot {}", lhs_k, rhs_k),
                SysmlBinOp::Div => format!("{} / {}", lhs_k, rhs_k),
                SysmlBinOp::Rem => format!("{} \\bmod {}", lhs_k, rhs_k),
                SysmlBinOp::Pow => {
                    if is_binary_sysml_expr(lhs)
                        || matches!(lhs.as_ref(), SysmlExprAst::Unary { .. })
                    {
                        format!("({})^{{{}}}", lhs_k, rhs_k)
                    } else {
                        format!("{}^{{{}}}", lhs_k, rhs_k)
                    }
                }
            }
        }
        SysmlExprAst::Call { callee, args } => {
            let callee_k = render_ast_katex(callee, next_depth);
            let args_k: Vec<String> = args
                .iter()
                .map(|a| render_ast_katex(a, next_depth))
                .collect();
            format!("{}({})", callee_k, args_k.join(", "))
        }
        SysmlExprAst::Field { base, field } => {
            let base_k = render_ast_katex(base, next_depth);
            let field_k = render_katex_ident(field);
            if is_binary_sysml_expr(base) {
                format!("({}).{}", base_k, field_k)
            } else {
                format!("{}.{}", base_k, field_k)
            }
        }
        SysmlExprAst::Index { base, indices } => {
            let base_k = render_ast_katex(base, next_depth);
            let idx_k: Vec<String> = indices
                .iter()
                .map(|i| render_ast_katex(i, next_depth))
                .collect();
            if is_binary_sysml_expr(base) {
                format!("({})[{}]", base_k, idx_k.join(", "))
            } else {
                format!("{}[{}]", base_k, idx_k.join(", "))
            }
        }
    }
}

/// Lowers a raw `SysmlToken` stream token-by-token into brace-balanced KaTeX when full AST parsing rejects malformed or over-deep input.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
pub(super) fn fallback_tokens_to_katex(tokens: &[SysmlToken]) -> String {
    let mut parts = Vec::with_capacity(tokens.len());
    for tok in tokens {
        let piece = match tok {
            SysmlToken::Implies => "\\implies".to_string(),
            SysmlToken::Or => "\\lor".to_string(),
            SysmlToken::Xor => "\\oplus".to_string(),
            SysmlToken::And => "\\land".to_string(),
            SysmlToken::KeywordNot | SysmlToken::Bang | SysmlToken::Tilde => "\\lnot".to_string(),
            SysmlToken::Eq => "=".to_string(),
            SysmlToken::Ne => "\\neq".to_string(),
            SysmlToken::Ge => "\\ge".to_string(),
            SysmlToken::Le => "\\le".to_string(),
            SysmlToken::Gt => ">".to_string(),
            SysmlToken::Lt => "<".to_string(),
            SysmlToken::Plus => "+".to_string(),
            SysmlToken::Minus => "-".to_string(),
            SysmlToken::Star => "\\cdot".to_string(),
            SysmlToken::Slash => "/".to_string(),
            SysmlToken::Percent => "\\bmod".to_string(),
            SysmlToken::Pow => "\\wedge".to_string(),
            SysmlToken::LParen => "(".to_string(),
            SysmlToken::RParen => ")".to_string(),
            SysmlToken::LBracket => "[".to_string(),
            SysmlToken::RBracket => "]".to_string(),
            SysmlToken::Comma => ",".to_string(),
            SysmlToken::Dot => ".".to_string(),
            SysmlToken::Ident(id) => render_katex_ident(id),
            SysmlToken::Number(n) => n.clone(),
            SysmlToken::StringLit(s) => format!("\\text{{\"{}\"}}", escape_katex_ident(s)),
            SysmlToken::Unknown(c) => {
                let mut tmp = String::new();
                tmp.push(*c);
                escape_katex_ident(&tmp)
            }
        };
        if !piece.is_empty() {
            parts.push(piece);
        }
    }
    if parts.is_empty() {
        "\\mathrm{true}".to_string()
    } else {
        parts.join(" ")
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
            match op {
                SysmlUnaryOp::Not => {
                    if matches!(operand.as_ref(), SysmlExprAst::Paren(_)) {
                        format!("~{}", inner)
                    } else {
                        format!("~({})", inner)
                    }
                }
                SysmlUnaryOp::Plus => {
                    if is_binary_sysml_expr(operand)
                        || matches!(operand.as_ref(), SysmlExprAst::Unary { .. })
                    {
                        format!("+({})", inner)
                    } else {
                        format!("+{}", inner)
                    }
                }
                SysmlUnaryOp::Minus => {
                    if is_binary_sysml_expr(operand)
                        || matches!(operand.as_ref(), SysmlExprAst::Unary { .. })
                    {
                        format!("-({})", inner)
                    } else {
                        format!("-{}", inner)
                    }
                }
            }
        }
        SysmlExprAst::Binary { op, lhs, rhs } => {
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
                        rhs.as_ref(),
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

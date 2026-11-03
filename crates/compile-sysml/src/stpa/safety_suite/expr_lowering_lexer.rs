#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Cursor-based lexical tokenizer for SysML v2 constraint and calculation expressions.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! formal expression lowering objectives for module `crates/compile-sysml/src/stpa/safety_suite/expr_lowering_lexer.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Advances cursor position monotonically via `.get(..)` and `.saturating_add(1)` over a collected `&[char]` slice.
//! - Preserves case-insensitive logical keyword recognition (`implies`, `or`, `xor`, `and`, `not`), numeric literals,
//!   escaped string literals, and multi-character operators (`->`, `=>`, `==`, `!=`, `~=`, `>=`, `<=`, `||`, `&&`, `**`, `::`).
//!
//! ## 3. Preconditions
//! - Caller supplies any valid UTF-8 expression slice `expr: &str`.
//!
//! ## 4. Postconditions
//! - Returns a complete `Vec<SysmlToken>` stream encoding unrecognized characters via `SysmlToken::Unknown(char)`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) linear in the Unicode scalar count of `expr`.
//! - Space Complexity: O(N) bounded character and token vectors.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Accepts empty, malformed, or arbitrary UTF-8 inputs without error.
//! - State: Local cursor state confined to `tokenize_sysml_expr`.
//! - Action: Zero `.char_indices()`, zero `.peekable()`, zero regular expressions, and zero panics.
//! - Resource: Bounded linear heap allocation.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: PackageDef(expr_lowering_lexer)

use super::expr_lowering::SysmlToken;

/// Checked slice-backed character cursor advancing monotonically via `.get(..)` and `.saturating_add(1)`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
struct LexerCursor<'a> {
    chars: &'a [char],
    pos: usize,
}

impl<'a> LexerCursor<'a> {
    /// Inspects the character at `self.pos` without advancing the cursor.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Inspects the character at `self.pos + 1` without advancing the cursor.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
    fn peek_second(&self) -> Option<char> {
        self.chars.get(self.pos.saturating_add(1)).copied()
    }

    /// Inspects the character at `self.pos + 2` without advancing the cursor.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
    fn peek_third(&self) -> Option<char> {
        self.chars.get(self.pos.saturating_add(2)).copied()
    }

    /// Consumes and returns the character at `self.pos`, advancing `self.pos` by 1.
    /// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
    fn next_char(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos = self.pos.saturating_add(1);
        Some(ch)
    }
}

/// Tokenizes a SysML v2 expression string monotonically via a checked `&[char]` cursor.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
pub(super) fn tokenize_sysml_expr(expr: &str) -> Vec<SysmlToken> {
    let char_vec: Vec<char> = expr.chars().collect();
    let mut cursor = LexerCursor {
        chars: &char_vec,
        pos: 0,
    };
    let mut tokens = Vec::new();

    while let Some(ch) = cursor.next_char() {
        if ch.is_whitespace() {
            continue;
        }
        if ch.is_alphabetic() || ch == '_' {
            tokens.push(lex_ident_or_keyword(ch, &mut cursor));
            continue;
        }
        if ch.is_ascii_digit()
            || (ch == '.' && cursor.peek().is_some_and(|d| d.is_ascii_digit()))
        {
            tokens.push(lex_number_literal(ch, &mut cursor));
            continue;
        }
        if ch == '"' || ch == '\'' {
            tokens.push(lex_string_literal(ch, &mut cursor));
            continue;
        }
        tokens.push(lex_operator_or_punct(ch, &mut cursor));
    }

    tokens
}

/// Scans an identifier or logical word operator (`implies`, `or`, `xor`, `and`, `not`).
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
fn lex_ident_or_keyword(first: char, cursor: &mut LexerCursor<'_>) -> SysmlToken {
    let mut word = String::new();
    word.push(first);
    while let Some(next_ch) = cursor.peek() {
        if next_ch.is_alphanumeric() || next_ch == '_' {
            word.push(next_ch);
            let _ = cursor.next_char();
        } else {
            break;
        }
    }
    if word.eq_ignore_ascii_case("implies") {
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
        SysmlToken::Ident(word)
    }
}

/// Consumes contiguous ASCII digits and digit-bounded `_` separators into `num_buf`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
fn lex_digit_run(cursor: &mut LexerCursor<'_>, num_buf: &mut String) {
    while let Some(next_ch) = cursor.peek() {
        if next_ch.is_ascii_digit() {
            num_buf.push(next_ch);
            let _ = cursor.next_char();
        } else if next_ch == '_' && cursor.peek_second().is_some_and(|d| d.is_ascii_digit()) {
            let _ = cursor.next_char();
        } else {
            break;
        }
    }
}

/// Scans an integer, decimal, or scientific-notation numeric literal into `SysmlToken::Number`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
fn lex_number_literal(first: char, cursor: &mut LexerCursor<'_>) -> SysmlToken {
    let mut num_buf = String::new();
    if first == '.' {
        num_buf.push('0');
        num_buf.push('.');
        lex_digit_run(cursor, &mut num_buf);
    } else {
        num_buf.push(first);
        lex_digit_run(cursor, &mut num_buf);
        if cursor.peek() == Some('.') && cursor.peek_second().is_some_and(|d| d.is_ascii_digit()) {
            let _ = cursor.next_char();
            num_buf.push('.');
            lex_digit_run(cursor, &mut num_buf);
        }
    }
    lex_exponent_suffix(cursor, &mut num_buf);
    SysmlToken::Number(num_buf)
}

/// Appends an optional `e`/`E` scientific-notation exponent suffix onto `num_buf`.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
fn lex_exponent_suffix(cursor: &mut LexerCursor<'_>, num_buf: &mut String) {
    if let Some(exp_ch @ ('e' | 'E')) = cursor.peek() {
        let has_valid_exp = match cursor.peek_second() {
            Some(d) if d.is_ascii_digit() => true,
            Some('+' | '-') => cursor.peek_third().is_some_and(|d| d.is_ascii_digit()),
            _ => false,
        };
        if has_valid_exp {
            num_buf.push(exp_ch);
            let _ = cursor.next_char();
            if let Some(sign @ ('+' | '-')) = cursor.peek() {
                num_buf.push(sign);
                let _ = cursor.next_char();
            }
            while let Some(next_ch) = cursor.peek() {
                if next_ch.is_ascii_digit() {
                    num_buf.push(next_ch);
                    let _ = cursor.next_char();
                } else {
                    break;
                }
            }
        }
    }
}

/// Scans a single- or double-quoted string literal supporting `\` escape sequences.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING]
fn lex_string_literal(quote: char, cursor: &mut LexerCursor<'_>) -> SysmlToken {
    let mut lit = String::new();
    while let Some(c) = cursor.next_char() {
        if c == '\\' {
            if let Some(escaped) = cursor.next_char() {
                lit.push(escaped);
            }
        } else if c == quote {
            break;
        } else {
            lit.push(c);
        }
    }
    SysmlToken::StringLit(lit)
}

/// Scans single- or two-character SysML operators, delimiters, and fallback `Unknown(char)` tokens.
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING]
fn lex_operator_or_punct(ch: char, cursor: &mut LexerCursor<'_>) -> SysmlToken {
    match ch {
        '-' if cursor.peek() == Some('>') => {
            let _ = cursor.next_char();
            SysmlToken::Implies
        }
        '-' => SysmlToken::Minus,
        '=' if cursor.peek() == Some('>') => {
            let _ = cursor.next_char();
            SysmlToken::Implies
        }
        '=' => {
            if cursor.peek() == Some('=') {
                let _ = cursor.next_char();
            }
            SysmlToken::Eq
        }
        '|' => {
            if cursor.peek() == Some('|') {
                let _ = cursor.next_char();
            }
            SysmlToken::Or
        }
        '&' => {
            if cursor.peek() == Some('&') {
                let _ = cursor.next_char();
            }
            SysmlToken::And
        }
        '!' if cursor.peek() == Some('=') => {
            let _ = cursor.next_char();
            SysmlToken::Ne
        }
        '!' => SysmlToken::Bang,
        '~' if cursor.peek() == Some('=') => {
            let _ = cursor.next_char();
            SysmlToken::Ne
        }
        '~' => SysmlToken::Tilde,
        '>' if cursor.peek() == Some('=') => {
            let _ = cursor.next_char();
            SysmlToken::Ge
        }
        '>' => SysmlToken::Gt,
        '<' if cursor.peek() == Some('=') => {
            let _ = cursor.next_char();
            SysmlToken::Le
        }
        '<' => SysmlToken::Lt,
        '+' => SysmlToken::Plus,
        '*' if cursor.peek() == Some('*') => {
            let _ = cursor.next_char();
            SysmlToken::Pow
        }
        '*' => SysmlToken::Star,
        '/' => SysmlToken::Slash,
        '%' => SysmlToken::Percent,
        '^' => SysmlToken::Pow,
        '(' => SysmlToken::LParen,
        ')' => SysmlToken::RParen,
        '[' => SysmlToken::LBracket,
        ']' => SysmlToken::RBracket,
        ',' => SysmlToken::Comma,
        '.' => SysmlToken::Dot,
        ':' if cursor.peek() == Some(':') => {
            let _ = cursor.next_char();
            SysmlToken::Dot
        }
        other => SysmlToken::Unknown(other),
    }
}

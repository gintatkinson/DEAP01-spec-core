//! # Logos CST-Backed Scanner & Precedence-Climbed Expression Parser (`cst_scanner`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Tokenizes UTF-8 SysML v2 / KerML buffers via `deap_core::sysml_cst::lexer::tokenize_sysml` and parses expressions bounded by `MAX_EXPR_DEPTH` (`64`).
//! ## 2. Invariants & Mathematical Properties -- Emits deterministic `Token` streams with 1-indexed line/column spans; rejects deep expression nesting at depth 64.
//! ## 3. Preconditions -- Input slices are valid UTF-8 source strings or `Token` slices.
//! ## 4. Postconditions -- Produces `Vec<Token>` terminating in `TokenKind::Eof` and precedence-climbed `Expr` AST trees.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(N) linear scan and expression parse; Space: O(N).
//! ## 6. Failure Modes & Fault Tolerance -- Rejects unclosed comments/strings and expression depth overflows with `Err(String)`.
//! ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-LEXER-SCANNER-MODULE], [REQ-PARSER-EXPRESSIONS-EXPR] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use deap_core::sysml_cst::lexer::{tokenize_sysml, LexedSysmlToken, SysmlToken};

use super::cst_db::{Span, Token, TokenKind};

/// Lexical scanner translating source text into `Token`s via `deap_core::sysml_cst::lexer::SysmlToken`.
/// Realises: [REQ-LEXER-SCANNER-SCANNER] | [FLS: fls_66m4rnbssgig]
pub struct Scanner<'a> {
    pub(crate) source: &'a str,
    pub(crate) cst_tokens: Vec<LexedSysmlToken<'a>>,
    pub(crate) cst_idx: usize,
    pub(crate) cursor: usize,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl<'a> Scanner<'a> {
    /// Constructs a new `Scanner` backed by `tokenize_sysml`. Realises: [REQ-LEXER-SCANNER-SCANNER-NEW] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self { source, cst_tokens: tokenize_sysml(source), cst_idx: 0, cursor: 0, line: 1, col: 1 }
    }

    /// Scans all tokens from the source until `TokenKind::Eof`. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-ALL] | [FLS: fls_66m4rnbssgig]
    pub fn scan_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                return Ok(tokens);
            }
        }
    }

    /// Fetches the next `Token` from the source stream. Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN] | [FLS: fls_66m4rnbssgig]
    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_line_comments();
        let (start, line, col) = (self.cursor, self.line, self.col);
        if self.cursor >= self.source.len() {
            return Ok(Token::new(TokenKind::Eof, Span::new(start, start, line, col)));
        }
        if self.starts_with("/*") {
            return self.scan_block_comment(start, line, col);
        }
        if let Some(tok) = self.aligned_cst_token() {
            if let Some(mapped) = self.try_fast_cst_token(&tok, start, line, col)? {
                return Ok(mapped);
            }
        }
        match self.peek() {
            ch @ ('"' | '\'') => self.scan_string(ch, start, line, col),
            ch if ch.is_ascii_digit()
                || (ch == '-'
                    && self
                        .source
                        .get(start.saturating_add(1)..)
                        .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))) =>
            {
                self.scan_number(start, line, col)
            }
            ch if is_ident_start(ch) => self.scan_ident_or_keyword(start, line, col),
            _ => self.scan_operator_or_punct(start, line, col),
        }
    }

    /// Returns the `LexedSysmlToken` whose start byte aligns with `self.cursor`. Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN] | [FLS: fls_66m4rnbssgig]
    fn aligned_cst_token(&mut self) -> Option<LexedSysmlToken<'a>> {
        while self.cst_tokens.get(self.cst_idx).is_some_and(|t| t.span.end <= self.cursor) {
            self.cst_idx = self.cst_idx.saturating_add(1);
        }
        self.cst_tokens.get(self.cst_idx).filter(|t| t.span.start == self.cursor).cloned()
    }

    /// Maps a single-span `LexedSysmlToken` directly to `Token`. Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN] | [FLS: fls_66m4rnbssgig]
    fn try_fast_cst_token(&mut self, tok: &LexedSysmlToken<'a>, start: usize, line: usize, col: usize) -> Result<Option<Token>, String> {
        let next_ch = self.source.get(tok.span.end..).and_then(|r| r.chars().next());
        let inner_quoted = || tok.slice.get(1..tok.slice.len().saturating_sub(1)).unwrap_or_default().to_string();
        let kind = match tok.token {
            SysmlToken::StringLit if !tok.slice.contains('\\') => TokenKind::StringLit(inner_quoted()),
            SysmlToken::UnrestrictedName if !tok.slice.contains('\\') => TokenKind::Ident(inner_quoted()),
            SysmlToken::NumberLit if !matches!(next_ch, Some('e' | 'E')) => TokenKind::parse_number_lexeme(tok.slice, line, col)?,
            SysmlToken::DefKw | SysmlToken::PackageKw | SysmlToken::NamespaceKw
            | SysmlToken::DocMetaKw | SysmlToken::StructuralKw | SysmlToken::BehavioralKw
            | SysmlToken::ReqCalcVerificationKw | SysmlToken::KermlModifierKw | SysmlToken::Ident
                if tok.slice != "doc" && tok.slice != "use" && !next_ch.is_some_and(is_ident_continue) =>
            {
                TokenKind::from_keyword_or_ident(tok.slice)
            }
            SysmlToken::Tilde if next_ch == Some('>') => return Ok(None),
            SysmlToken::Operator if tok.slice == "-" && next_ch.is_some_and(|c| c.is_ascii_digit()) => return Ok(None),
            _ => match TokenKind::from_symbol(tok.slice) {
                Some(k) => k,
                None => return Ok(None),
            },
        };
        self.advance_slice(tok.slice);
        self.cst_idx = self.cst_idx.saturating_add(1);
        Ok(Some(Token::new(kind, Span::new(start, self.cursor, line, col))))
    }

    /// Scans a potentially nested `/* ... */` block comment. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-BLOCK-COMMENT] | [FLS: fls_66m4rnbssgig]
    fn scan_block_comment(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        if let Some(tok) = self.aligned_cst_token() {
            let inner = tok.slice.get(2..tok.slice.len().saturating_sub(2)).unwrap_or_default();
            if tok.token == SysmlToken::BlockComment && !inner.contains("/*") {
                self.advance_slice(tok.slice);
                self.cst_idx = self.cst_idx.saturating_add(1);
                return Ok(Token::new(TokenKind::doc_comment_from_raw(inner), Span::new(start, self.cursor, line, col)));
            }
        }
        self.advance_slice("/*");
        let (content_start, mut depth, mut content_end) = (self.cursor, 1usize, self.cursor);
        while self.cursor < self.source.len() && depth > 0 {
            if self.starts_with("/*") {
                depth = depth.saturating_add(1);
                self.advance_slice("/*");
            } else if self.starts_with("*/") {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    content_end = self.cursor;
                }
                self.advance_slice("*/");
            } else {
                self.advance_char();
            }
        }
        if depth > 0 {
            return Err(format!("Unclosed block comment starting at {line}:{col}"));
        }
        let raw = self.source.get(content_start..content_end).unwrap_or_default();
        Ok(Token::new(TokenKind::doc_comment_from_raw(raw), Span::new(start, self.cursor, line, col)))
    }

    /// Scans a single- or double-quoted string literal with escape handling. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-STRING] | [FLS: fls_66m4rnbssgig]
    fn scan_string(&mut self, quote: char, start: usize, line: usize, col: usize) -> Result<Token, String> {
        let desc = if quote == '\'' { "unrestricted identifier" } else { "string literal" };
        self.advance();
        let mut value = String::new();
        while self.cursor < self.source.len() && self.peek() != quote {
            if self.peek() == '\\' {
                self.advance();
                if self.cursor >= self.source.len() {
                    return Err(format!("Unterminated escape in {desc} at {line}:{col}"));
                }
                let esc = match self.advance() {
                    'n' => '\n', 'r' => '\r', 't' => '\t', other => other,
                };
                value.push(esc);
            } else {
                value.push(self.advance_char());
            }
        }
        if self.cursor >= self.source.len() {
            return Err(format!("Unclosed {desc} starting at {line}:{col}"));
        }
        self.advance();
        let kind = if quote == '\'' { TokenKind::Ident(value) } else { TokenKind::StringLit(value) };
        Ok(Token::new(kind, Span::new(start, self.cursor, line, col)))
    }

    /// Scans an integer or floating-point literal. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-NUMBER] | [FLS: fls_66m4rnbssgig]
    fn scan_number(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        if self.peek() == '-' {
            self.advance();
        }
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.source.get(self.cursor.saturating_add(1)..).is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit())) {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        if matches!(self.peek(), 'e' | 'E') {
            self.advance();
            if matches!(self.peek(), '+' | '-') {
                self.advance();
            }
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let kind = TokenKind::parse_number_lexeme(self.source.get(start..self.cursor).unwrap_or_default(), line, col)?;
        Ok(Token::new(kind, Span::new(start, self.cursor, line, col)))
    }

    /// Scans an identifier, keyword, `doc /* ... */` block, or `use case` compound keyword. Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-IDENT-OR-KEYWORD] | [FLS: fls_66m4rnbssgig]
    fn scan_ident_or_keyword(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        self.advance();
        while is_ident_continue(self.peek()) {
            self.advance();
        }
        let text = self.source.get(start..self.cursor).unwrap_or_default();
        if text == "doc" || text == "use" {
            let saved = (self.cursor, self.line, self.col);
            while self.peek().is_ascii_whitespace() {
                self.advance_char();
            }
            if text == "doc" && self.starts_with("/*") {
                return self.scan_block_comment(start, line, col);
            }
            if text == "use" {
                let word_start = self.cursor;
                while is_ident_continue(self.peek()) {
                    self.advance();
                }
                if self.source.get(word_start..self.cursor) == Some("case") {
                    return Ok(Token::new(TokenKind::UseCase, Span::new(start, self.cursor, line, col)));
                }
            }
            (self.cursor, self.line, self.col) = saved;
        }
        Ok(Token::new(TokenKind::from_keyword_or_ident(text), Span::new(start, self.cursor, line, col)))
    }

    /// Scans multi-character or single-character operators and punctuation. Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN] | [FLS: fls_66m4rnbssgig]
    fn scan_operator_or_punct(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        if let Some((two, kind)) = self.source.get(start..start.saturating_add(2)).and_then(|s| Some((s, TokenKind::from_symbol(s)?))) {
            self.advance_slice(two);
            return Ok(Token::new(kind, Span::new(start, self.cursor, line, col)));
        }
        let ch = self.advance();
        let Some(kind) = self.source.get(start..self.cursor).and_then(TokenKind::from_symbol) else {
            return Err(format!("Unexpected character '{ch}' at {line}:{col}"));
        };
        Ok(Token::new(kind, Span::new(start, self.cursor, line, col)))
    }

    /// Skips whitespace and `//` line comments. Realises: [REQ-LEXER-SCANNER-SCANNER-SKIP-WHITESPACE-AND-LINE-COMMENTS] | [FLS: fls_66m4rnbssgig]
    fn skip_whitespace_and_line_comments(&mut self) {
        while let Some(tok) = self.aligned_cst_token() {
            if matches!(tok.token, SysmlToken::Whitespace | SysmlToken::LineComment) {
                self.advance_slice(tok.slice);
                self.cst_idx = self.cst_idx.saturating_add(1);
            } else {
                break;
            }
        }
        while self.cursor < self.source.len() {
            if self.peek().is_ascii_whitespace() {
                self.advance_char();
            } else if self.starts_with("//") {
                while self.cursor < self.source.len() && self.peek() != '\n' {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// Advances `cursor`, `line`, and `col` across `slice`. Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE] | [FLS: fls_66m4rnbssgig]
    fn advance_slice(&mut self, slice: &str) {
        for _ in slice.chars() {
            self.advance_char();
        }
    }

    /// Advances by one character while updating newline line/column tracking. Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE] | [FLS: fls_66m4rnbssgig]
    fn advance_char(&mut self) -> char {
        let ch = self.advance();
        if ch == '\n' {
            self.line = self.line.saturating_add(1);
            self.col = 1;
        }
        ch
    }

    /// Returns `true` when the remaining slice at `cursor` begins with `prefix`. Realises: [REQ-LEXER-SCANNER-SCANNER-STARTS-WITH] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn starts_with(&self, prefix: &str) -> bool {
        self.source.get(self.cursor..).is_some_and(|rest| rest.starts_with(prefix))
    }

    /// Returns the character at `cursor` or `'\0'` on EOF. Realises: [REQ-LEXER-SCANNER-SCANNER-PEEK] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn peek(&self) -> char {
        self.source.get(self.cursor..).and_then(|rest| rest.chars().next()).unwrap_or('\0')
    }

    /// Advances `cursor` by one UTF-8 character and increments `col`. Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn advance(&mut self) -> char {
        let Some(ch) = self.source.get(self.cursor..).and_then(|rest| rest.chars().next()) else {
            return '\0';
        };
        self.cursor = self.cursor.saturating_add(ch.len_utf8());
        self.col = self.col.saturating_add(1);
        ch
    }
}

/// Returns `true` when `c` can start a SysML identifier (`[A-Za-z_@]`). Realises: [REQ-LEXER-SCANNER-IS-IDENT-START] | [FLS: fls_66m4rnbssgig]
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '@'
}

/// Returns `true` when `c` can continue a SysML identifier (`[A-Za-z0-9_-]`). Realises: [REQ-LEXER-SCANNER-IS-IDENT-CONTINUE] | [FLS: fls_66m4rnbssgig]
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

pub use super::cst_expr::{parse_expression, BinaryOp, Expr, ExpressionParser, UnaryOp};

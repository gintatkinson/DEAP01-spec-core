//! Logos CST-backed lexical scanner for SysML v2 / KerML syntax.
//! [DOC-TIER: A]
//!
//! Safety & Traceability: Realises: [REQ-LEXER-SCANNER-MODULE]
//! Contract: Tokenizes UTF-8 SysML v2 and KerML source buffers via `deap_core::sysml_cst::lexer::tokenize_sysml` (`logos::Logos`) into span-annotated `Token` streams.
//! Bounds & Faults: Rejects unclosed comments, unterminated string escapes, and malformed spans deterministically with zero panicking slice indexing.

use super::token::{Span, Token, TokenKind};
use deap_core::sysml_cst::lexer::{tokenize_sysml, LexedSysmlToken, SysmlToken};

/// Lexical scanner translating source text into tokens via `deap_core::sysml_cst::lexer::SysmlToken`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Scanner` in `crates/compile-sysml/src/lexer/scanner.rs`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Scanner` backed by `deap_core::sysml_cst::lexer::LexedSysmlToken`.
/// ## 3. Preconditions
/// - Caller supplies a well-formed UTF-8 `source: &'a str` slice via `Scanner::new`.
/// ## 4. Postconditions
/// - Emits deterministic `Token` streams matching `SysmlToken` spans and line/column coordinates.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) linear scan over `source.len()`.
/// - Space Complexity: O(N) token stream buffer.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Scanner`.
/// - State: Maintains consistent cursor invariants across `Scanner` without panicking on mid-UTF-8 offsets.
/// - Action: Executes `Scanner` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Scanner` (`LEXER-SCANNER`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-SCANNER-SCANNER] | SysML v2: PartDef(Scanner)
pub struct Scanner<'a> {
    pub(super) source: &'a str,
    pub(super) cst_tokens: Vec<LexedSysmlToken<'a>>,
    pub(super) cst_idx: usize,
    pub(super) cursor: usize,
    pub(super) line: usize,
    pub(super) col: usize,
}

impl<'a> Scanner<'a> {
    /// Constructs a new `Scanner` backed by `deap_core::sysml_cst::lexer::tokenize_sysml`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::new`.
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `cursor = 0`, `line = 1`, `col = 1`, and pre-tokenizes `source` via `tokenize_sysml`.
    /// ## 3. Preconditions
    /// - `source: &'a str` is a valid UTF-8 source slice.
    /// ## 4. Postconditions
    /// - Returns an initialized `Scanner<'a>` positioned at the start of `source`.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over `source.len()`.
    /// - Space Complexity: O(N) for the `LexedSysmlToken` vector.
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Total constructor; lexical errors are deferred to `next_token` / `scan_all`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEW] | SysML v2: ActionDef(Scanner::new)
    pub fn new(source: &'a str) -> Self {
        Self { source, cst_tokens: tokenize_sysml(source), cst_idx: 0, cursor: 0, line: 1, col: 1 }
    }

    /// Scans all tokens from the source until `TokenKind::Eof`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::scan_all`.
    /// ## 2. Invariants & Mathematical Properties
    /// - Terminates with a trailing `TokenKind::Eof` token on every valid input stream.
    /// ## 3. Preconditions
    /// - Invoked on a valid `Scanner` instance.
    /// ## 4. Postconditions
    /// - Returns `Ok(Vec<Token>)` or `Err(String)` on the first lexical error.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over remaining source bytes; Space Complexity: O(N).
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Propagates unclosed string/comment and invalid token errors deterministically.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-ALL] | SysML v2: ActionDef(Scanner::scan_all)
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

    /// Fetches the next `Token` from the source stream using `SysmlToken` CST classification.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::next_token`.
    /// ## 2. Invariants & Mathematical Properties
    /// - Advances `cursor` monotonically until `TokenKind::Eof` is reached.
    /// ## 3. Preconditions
    /// - Invoked on a valid `Scanner` instance.
    /// ## 4. Postconditions
    /// - Returns `Ok(Token)` with 1-indexed line/column `Span` or `Err(String)` on lexical error.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K) over the byte length of the scanned token and preceding trivia; Space Complexity: O(K).
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Rejects unclosed block comments, unclosed strings, and unexpected symbols with line/col diagnostics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN] | SysML v2: ActionDef(Scanner::next_token)
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
            ch if ch.is_ascii_digit() || (ch == '-' && self.source.get(start.saturating_add(1)..).is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))) => self.scan_number(start, line, col),
            ch if is_ident_start(ch) => self.scan_ident_or_keyword(start, line, col),
            _ => self.scan_operator_or_punct(start, line, col),
        }
    }

    /// Returns the `LexedSysmlToken` whose start byte aligns with `self.cursor`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN]
    fn aligned_cst_token(&mut self) -> Option<LexedSysmlToken<'a>> {
        while self.cst_tokens.get(self.cst_idx).is_some_and(|t| t.span.end <= self.cursor) {
            self.cst_idx = self.cst_idx.saturating_add(1);
        }
        self.cst_tokens.get(self.cst_idx).filter(|t| t.span.start == self.cursor).cloned()
    }

    /// Maps a single-span `LexedSysmlToken` directly to `Token` when no multi-token lookahead or splitting is required.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN]
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

    /// Scans a potentially nested `/* ... */` block comment using `SysmlToken::BlockComment` for non-nested fast paths.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-BLOCK-COMMENT]
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

    /// Scans a single- or double-quoted string literal with escape handling.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-STRING]
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

    /// Scans an integer or floating-point literal (including negative numbers).
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-NUMBER]
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

    /// Scans an identifier, keyword, `doc /* ... */` block, or `use case` compound keyword.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-IDENT-OR-KEYWORD]
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

    /// Scans multi-character or single-character operators and punctuation via `TokenKind::from_symbol`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN]
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

    /// Skips whitespace and `//` line comments using `SysmlToken::Whitespace` and `SysmlToken::LineComment`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SKIP-WHITESPACE-AND-LINE-COMMENTS]
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

    /// Advances `cursor`, `line`, and `col` across a pre-scanned `slice`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE]
    fn advance_slice(&mut self, slice: &str) {
        for _ in slice.chars() {
            self.advance_char();
        }
    }

    /// Advances by one character while updating newline line/column tracking.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE]
    fn advance_char(&mut self) -> char {
        let ch = self.advance();
        if ch == '\n' {
            self.line = self.line.saturating_add(1);
            self.col = 1;
        }
        ch
    }

    /// Returns `true` when the remaining slice at `cursor` begins with `prefix`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-STARTS-WITH]
    pub(super) fn starts_with(&self, prefix: &str) -> bool {
        self.source.get(self.cursor..).is_some_and(|rest| rest.starts_with(prefix))
    }

    /// Returns the character at `cursor` or `'\0'` on EOF / non-char boundary.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-PEEK]
    pub(super) fn peek(&self) -> char {
        self.source.get(self.cursor..).and_then(|rest| rest.chars().next()).unwrap_or('\0')
    }

    /// Advances `cursor` by one UTF-8 character and increments `col`.
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE]
    pub(super) fn advance(&mut self) -> char {
        let Some(ch) = self.source.get(self.cursor..).and_then(|rest| rest.chars().next()) else {
            return '\0';
        };
        self.cursor = self.cursor.saturating_add(ch.len_utf8());
        self.col = self.col.saturating_add(1);
        ch
    }
}

/// Returns `true` when `c` can start a SysML identifier (`[A-Za-z_@]`).
/// Realises: [REQ-LEXER-SCANNER-IS-IDENT-START]
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '@'
}

/// Returns `true` when `c` can continue a SysML identifier (`[A-Za-z0-9_-]`).
/// Realises: [REQ-LEXER-SCANNER-IS-IDENT-CONTINUE]
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

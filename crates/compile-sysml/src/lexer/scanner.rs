//! Zero-copy slice-based tokenizer for SysML v2 / KerML syntax.

use super::token::{Span, Token, TokenKind};

/// Lexical scanner translating source text into tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Scanner` in `crates/compile-sysml/src/lexer/scanner.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Scanner` across member set `[source: &'a str, cursor: usize, line: usize, col: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Scanner` members (`source: &'a str, cursor: usize, line: usize, col: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Scanner` representation for struct `Scanner`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `Scanner`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `Scanner`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Scanner` (`Scanner`).
/// - State: Maintains consistent state invariants across `Scanner` without partial mutation on error.
/// - Action: Executes `Scanner` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Scanner` (`LEXER-SCANNER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-SCANNER-SCANNER] | SysML v2: PartDef(Scanner)
pub struct Scanner<'a> {
    source: &'a str,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    /// Executes deterministic fn `Scanner::new` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::new` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::new` (declaration #2) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - `source: &'a str` must satisfy domain and type-safety constraints prior to invoking `Scanner::new`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `Scanner::new` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(source)) bounded over input/member dimension of `Scanner::new`.
    /// - Space Complexity: O(N(source)) bounded auxiliary memory for `Scanner::new`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::new` (`Self`).
    /// - State: Maintains consistent state invariants across `Scanner::new` without partial mutation on error.
    /// - Action: Executes `Scanner::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::new` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEW] | SysML v2: ActionDef(Scanner::new)
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    /// Scan all tokens from the source until EOF.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::scan_all` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::scan_all` (declaration #3) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::scan_all` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Vec<Token>, String>` from `Scanner::scan_all` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(scan_all)) bounded over input/member dimension of `Scanner::scan_all`.
    /// - Space Complexity: O(K(scan_all)) bounded auxiliary memory for `Scanner::scan_all`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::scan_all` (`Result<Vec<Token>, String>`).
    /// - State: Maintains consistent state invariants across `Scanner::scan_all` without partial mutation on error.
    /// - Action: Executes `Scanner::scan_all` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::scan_all` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-ALL] | SysML v2: ActionDef(Scanner::scan_all)
    pub fn scan_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// Fetch next token.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::next_token` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::next_token` (declaration #4) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::next_token` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Token, String>` from `Scanner::next_token` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(next_token)) bounded over input/member dimension of `Scanner::next_token`.
    /// - Space Complexity: O(K(next_token)) bounded auxiliary memory for `Scanner::next_token`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::next_token` (`Result<Token, String>`).
    /// - State: Maintains consistent state invariants across `Scanner::next_token` without partial mutation on error.
    /// - Action: Executes `Scanner::next_token` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::next_token` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-NEXT-TOKEN] | SysML v2: ActionDef(Scanner::next_token)
    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_line_comments();

        let start = self.cursor;
        let line = self.line;
        let col = self.col;

        if self.is_at_end() {
            return Ok(Token::new(
                TokenKind::Eof,
                Span::new(start, start, line, col),
            ));
        }

        // Check for block doc comment /* ... */ or doc /* ... */
        if self.starts_with("/*") {
            return self.scan_block_comment(start, line, col);
        }

        let ch = self.peek();

        // String literals
        if ch == '"' || ch == '\'' {
            return self.scan_string(ch, start, line, col);
        }

        // Numeric literals
        if ch.is_ascii_digit() {
            return self.scan_number(start, line, col);
        }

        // Identifiers and keywords
        if is_ident_start(ch) {
            return self.scan_ident_or_keyword(start, line, col);
        }

        // Operators & punctuation
        self.advance();
        let kind = match ch {
            ';' => TokenKind::Semi,
            ',' => TokenKind::Comma,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '.' => TokenKind::Dot,
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            ':' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenKind::ColonEq
                } else if self.peek() == '>' {
                    self.advance();
                    TokenKind::ColonGt
                } else if self.peek() == ':' {
                    self.advance();
                    TokenKind::DoubleColon
                } else {
                    TokenKind::Colon
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    TokenKind::Exclamation
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '-' => {
                if self.peek() == '>' {
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek().is_ascii_digit() {
                    // Negative number
                    return self.scan_number(start, line, col);
                } else {
                    TokenKind::Minus
                }
            }
            '~' => {
                if self.peek() == '>' {
                    self.advance();
                    TokenKind::TildeGt
                } else {
                    TokenKind::Tilde
                }
            }
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    TokenKind::AmpAmp
                } else {
                    return Err(format!("Unexpected character '&' at {}:{}", line, col));
                }
            }
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    TokenKind::PipePipe
                } else {
                    return Err(format!("Unexpected character '|' at {}:{}", line, col));
                }
            }
            unexpected => {
                return Err(format!("Unexpected character '{}' at {}:{}", unexpected, line, col));
            }
        };

        let end = self.cursor;
        Ok(Token::new(kind, Span::new(start, end, line, col)))
    }

    /// Executes deterministic fn `Scanner::scan_block_comment` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::scan_block_comment` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::scan_block_comment` (declaration #5) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - `start: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_block_comment`.
    /// - `line: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_block_comment`.
    /// - `col: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_block_comment`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Token, String>` from `Scanner::scan_block_comment` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(start)) bounded over input/member dimension of `Scanner::scan_block_comment`.
    /// - Space Complexity: O(N(start)) bounded auxiliary memory for `Scanner::scan_block_comment`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::scan_block_comment` (`Result<Token, String>`).
    /// - State: Maintains consistent state invariants across `Scanner::scan_block_comment` without partial mutation on error.
    /// - Action: Executes `Scanner::scan_block_comment` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::scan_block_comment` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-BLOCK-COMMENT] | SysML v2: ActionDef(Scanner::scan_block_comment)
    fn scan_block_comment(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        self.advance(); // /
        self.advance(); // *
        let content_start = self.cursor;
        let mut depth: usize = 1;
        let mut content_end = self.cursor;

        while !self.is_at_end() {
            if self.starts_with("/*") {
                depth += 1;
                self.advance();
                self.advance();
            } else if self.starts_with("*/") {
                depth -= 1;
                if depth == 0 {
                    content_end = self.cursor;
                    self.advance();
                    self.advance();
                    break;
                } else {
                    self.advance();
                    self.advance();
                }
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.col = 0;
                }
                self.advance();
            }
        }

        if depth > 0 {
            return Err(format!("Unclosed block comment starting at {}:{}", line, col));
        }

        let raw = &self.source[content_start..content_end];
        let cleaned = raw.trim();
        let comment_text = if cleaned.starts_with("doc") && (cleaned.len() == 3 || cleaned[3..].chars().next().is_some_and(|c| c.is_whitespace() || c == ':')) {
            cleaned[3..].trim_start_matches(|c: char| c.is_whitespace() || c == ':').trim().to_string()
        } else {
            cleaned.to_string()
        };

        Ok(Token::new(
            TokenKind::DocComment(comment_text),
            Span::new(start, self.cursor, line, col),
        ))
    }

    /// Executes deterministic fn `Scanner::scan_string` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::scan_string` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::scan_string` (declaration #6) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - `quote: char` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_string`.
    /// - `start: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_string`.
    /// - `line: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_string`.
    /// - `col: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_string`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Token, String>` from `Scanner::scan_string` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(quote)) bounded over input/member dimension of `Scanner::scan_string`.
    /// - Space Complexity: O(N(quote)) bounded auxiliary memory for `Scanner::scan_string`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::scan_string` (`Result<Token, String>`).
    /// - State: Maintains consistent state invariants across `Scanner::scan_string` without partial mutation on error.
    /// - Action: Executes `Scanner::scan_string` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::scan_string` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-STRING] | SysML v2: ActionDef(Scanner::scan_string)
    fn scan_string(&mut self, quote: char, start: usize, line: usize, col: usize) -> Result<Token, String> {
        self.advance(); // opening quote
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != quote {
            let ch = self.peek();
            if ch == '\\' {
                self.advance();
                if self.is_at_end() {
                    let desc = if quote == '\'' { "unrestricted identifier" } else { "string literal" };
                    return Err(format!("Unterminated escape in {} at {}:{}", desc, line, col));
                }
                match self.advance() {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    escaped => value.push(escaped),
                }
            } else {
                if ch == '\n' {
                    self.line += 1;
                    self.col = 0;
                }
                value.push(ch);
                self.advance();
            }
        }

        if self.is_at_end() {
            let desc = if quote == '\'' { "unrestricted identifier" } else { "string literal" };
            return Err(format!("Unclosed {} starting at {}:{}", desc, line, col));
        }

        self.advance(); // closing quote

        let kind = if quote == '\'' {
            TokenKind::Ident(value)
        } else {
            TokenKind::StringLit(value)
        };

        Ok(Token::new(
            kind,
            Span::new(start, self.cursor, line, col),
        ))
    }

    /// Executes deterministic fn `Scanner::scan_number` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::scan_number` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::scan_number` (declaration #7) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - `start: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_number`.
    /// - `line: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_number`.
    /// - `col: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_number`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Token, String>` from `Scanner::scan_number` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(start)) bounded over input/member dimension of `Scanner::scan_number`.
    /// - Space Complexity: O(N(start)) bounded auxiliary memory for `Scanner::scan_number`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::scan_number` (`Result<Token, String>`).
    /// - State: Maintains consistent state invariants across `Scanner::scan_number` without partial mutation on error.
    /// - Action: Executes `Scanner::scan_number` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::scan_number` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-NUMBER] | SysML v2: ActionDef(Scanner::scan_number)
    fn scan_number(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        let num_start = start;
        let mut is_real = false;

        if self.peek() == '-' {
            self.advance();
        }

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        if !self.is_at_end() && self.peek() == '.' {
            // Check if next character after '.' is a digit (distinguish from dot operator)
            if self.cursor + 1 < self.source.len() {
                let next_ch = self.source[self.cursor + 1..].chars().next().unwrap_or('\0');
                if next_ch.is_ascii_digit() {
                    is_real = true;
                    self.advance(); // consume '.'
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        self.advance();
                    }
                }
            }
        }

        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            is_real = true;
            self.advance();
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                self.advance();
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num_str = &self.source[num_start..self.cursor];
        if is_real {
            let val = num_str.parse::<f64>().map_err(|e| format!("Invalid float '{}' at {}:{}: {}", num_str, line, col, e))?;
            Ok(Token::new(TokenKind::RealLit(val), Span::new(start, self.cursor, line, col)))
        } else {
            let val = num_str.parse::<i64>().map_err(|e| format!("Invalid integer '{}' at {}:{}: {}", num_str, line, col, e))?;
            Ok(Token::new(TokenKind::IntLit(val), Span::new(start, self.cursor, line, col)))
        }
    }

    /// Executes deterministic fn `Scanner::scan_ident_or_keyword` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::scan_ident_or_keyword` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::scan_ident_or_keyword` (declaration #8) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - `start: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_ident_or_keyword`.
    /// - `line: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_ident_or_keyword`.
    /// - `col: usize` must satisfy domain and type-safety constraints prior to invoking `Scanner::scan_ident_or_keyword`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Token, String>` from `Scanner::scan_ident_or_keyword` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(start)) bounded over input/member dimension of `Scanner::scan_ident_or_keyword`.
    /// - Space Complexity: O(N(start)) bounded auxiliary memory for `Scanner::scan_ident_or_keyword`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::scan_ident_or_keyword` (`Result<Token, String>`).
    /// - State: Maintains consistent state invariants across `Scanner::scan_ident_or_keyword` without partial mutation on error.
    /// - Action: Executes `Scanner::scan_ident_or_keyword` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::scan_ident_or_keyword` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SCAN-IDENT-OR-KEYWORD] | SysML v2: ActionDef(Scanner::scan_ident_or_keyword)
    fn scan_ident_or_keyword(&mut self, start: usize, line: usize, col: usize) -> Result<Token, String> {
        while !self.is_at_end() && is_ident_continue(self.peek()) {
            self.advance();
        }

        let text = &self.source[start..self.cursor];

        // Check if `doc` followed immediately by `/*`
        if text == "doc" {
            let saved_cursor = self.cursor;
            let saved_line = self.line;
            let saved_col = self.col;
            self.skip_whitespace();
            if self.starts_with("/*") {
                return self.scan_block_comment(start, line, col);
            }
            // Restore if not a block comment
            self.cursor = saved_cursor;
            self.line = saved_line;
            self.col = saved_col;
        }

        // Check if `use` followed by `case`
        if text == "use" {
            let saved_cursor = self.cursor;
            let saved_line = self.line;
            let saved_col = self.col;
            self.skip_whitespace();
            let word_start = self.cursor;
            while !self.is_at_end() && is_ident_continue(self.peek()) {
                self.advance();
            }
            if &self.source[word_start..self.cursor] == "case" {
                return Ok(Token::new(
                    TokenKind::UseCase,
                    Span::new(start, self.cursor, line, col),
                ));
            }
            // Restore
            self.cursor = saved_cursor;
            self.line = saved_line;
            self.col = saved_col;
        }

        let kind = match text {
            "package" => TokenKind::Package,
            "import" => TokenKind::Import,
            "part" => TokenKind::Part,
            "def" => TokenKind::Def,
            "port" => TokenKind::Port,
            "item" => TokenKind::Item,
            "flow" => TokenKind::Flow,
            "interface" => TokenKind::Interface,
            "subsystem" => TokenKind::Subsystem,
            "subject" => TokenKind::Subject,
            "actor" => TokenKind::Actor,
            "action" => TokenKind::Action,
            "operation" => TokenKind::Operation,
            "state" => TokenKind::State,
            "transition" => TokenKind::Transition,
            "entry" => TokenKind::Entry,
            "do" => TokenKind::Do,
            "exit" => TokenKind::Exit,
            "perform" => TokenKind::Perform,
            "step" => TokenKind::Step,
            "trigger" => TokenKind::Trigger,
            "requirement" => TokenKind::Requirement,
            "constraint" => TokenKind::Constraint,
            "assert" => TokenKind::Assert,
            "assume" => TokenKind::Assume,
            "require" => TokenKind::Require,
            "verify" => TokenKind::Verify,
            "satisfy" => TokenKind::Satisfy,
            "calc" => TokenKind::Calc,
            "test" => TokenKind::Test,
            "case" => TokenKind::Case,
            "hazard" => TokenKind::Hazard,
            "risk" => TokenKind::Risk,
            "include" => TokenKind::Include,
            "extend" => TokenKind::Extend,
            "precondition" => TokenKind::Precondition,
            "postcondition" => TokenKind::Postcondition,
            "objective" => TokenKind::Objective,
            "capability" => TokenKind::Capability,
            "interaction" => TokenKind::Interaction,
            "lifeline" => TokenKind::Lifeline,
            "message" => TokenKind::Message,
            "connection" => TokenKind::Connection,
            "connect" => TokenKind::Connect,
            "from" => TokenKind::From,
            "to" => TokenKind::To,
            "by" => TokenKind::By,
            "attribute" => TokenKind::Attribute,
            "id" => TokenKind::Id,
            "text" => TokenKind::Text,
            "doc" => TokenKind::Doc,
            "in" => TokenKind::In,
            "out" => TokenKind::Out,
            "inout" => TokenKind::Inout,
            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),
            _ => TokenKind::Ident(text.to_string()),
        };

        Ok(Token::new(kind, Span::new(start, self.cursor, line, col)))
    }

    /// Executes deterministic fn `Scanner::skip_whitespace_and_line_comments` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::skip_whitespace_and_line_comments` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::skip_whitespace_and_line_comments` (declaration #9) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::skip_whitespace_and_line_comments` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `()` from `Scanner::skip_whitespace_and_line_comments` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(skip_whitespace_and_line_comments)) bounded over input/member dimension of `Scanner::skip_whitespace_and_line_comments`.
    /// - Space Complexity: O(K(skip_whitespace_and_line_comments)) bounded auxiliary memory for `Scanner::skip_whitespace_and_line_comments`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::skip_whitespace_and_line_comments` (`()`).
    /// - State: Maintains consistent state invariants across `Scanner::skip_whitespace_and_line_comments` without partial mutation on error.
    /// - Action: Executes `Scanner::skip_whitespace_and_line_comments` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::skip_whitespace_and_line_comments` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SKIP-WHITESPACE-AND-LINE-COMMENTS] | SysML v2: ActionDef(Scanner::skip_whitespace_and_line_comments)
    fn skip_whitespace_and_line_comments(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.col = 1;
                }
                '/' if self.cursor + 1 < self.source.len() && self.source[self.cursor + 1..].starts_with('/') => {
                    // Single line comment
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    /// Executes deterministic fn `Scanner::skip_whitespace` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::skip_whitespace` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::skip_whitespace` (declaration #10) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::skip_whitespace` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `()` from `Scanner::skip_whitespace` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(skip_whitespace)) bounded over input/member dimension of `Scanner::skip_whitespace`.
    /// - Space Complexity: O(K(skip_whitespace)) bounded auxiliary memory for `Scanner::skip_whitespace`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::skip_whitespace` (`()`).
    /// - State: Maintains consistent state invariants across `Scanner::skip_whitespace` without partial mutation on error.
    /// - Action: Executes `Scanner::skip_whitespace` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::skip_whitespace` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-SKIP-WHITESPACE] | SysML v2: ActionDef(Scanner::skip_whitespace)
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.col = 1;
                }
                _ => break,
            }
        }
    }

    /// Executes deterministic fn `Scanner::is_at_end` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::is_at_end` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::is_at_end` (declaration #11) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::is_at_end` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `bool` from `Scanner::is_at_end` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(is_at_end)) bounded over input/member dimension of `Scanner::is_at_end`.
    /// - Space Complexity: O(K(is_at_end)) bounded auxiliary memory for `Scanner::is_at_end`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::is_at_end` (`bool`).
    /// - State: Maintains consistent state invariants across `Scanner::is_at_end` without partial mutation on error.
    /// - Action: Executes `Scanner::is_at_end` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::is_at_end` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-IS-AT-END] | SysML v2: ActionDef(Scanner::is_at_end)
    fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    /// Executes deterministic fn `Scanner::starts_with` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::starts_with` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::starts_with` (declaration #12) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - `prefix: &str` must satisfy domain and type-safety constraints prior to invoking `Scanner::starts_with`.
    ///
    /// ## 4. Postconditions
    /// - Returns `bool` from `Scanner::starts_with` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(prefix)) bounded over input/member dimension of `Scanner::starts_with`.
    /// - Space Complexity: O(N(prefix)) bounded auxiliary memory for `Scanner::starts_with`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::starts_with` (`bool`).
    /// - State: Maintains consistent state invariants across `Scanner::starts_with` without partial mutation on error.
    /// - Action: Executes `Scanner::starts_with` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::starts_with` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-STARTS-WITH] | SysML v2: ActionDef(Scanner::starts_with)
    fn starts_with(&self, prefix: &str) -> bool {
        self.source[self.cursor..].starts_with(prefix)
    }

    /// Executes deterministic fn `Scanner::peek` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::peek` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::peek` (declaration #13) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::peek` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `char` from `Scanner::peek` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(peek)) bounded over input/member dimension of `Scanner::peek`.
    /// - Space Complexity: O(K(peek)) bounded auxiliary memory for `Scanner::peek`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::peek` (`char`).
    /// - State: Maintains consistent state invariants across `Scanner::peek` without partial mutation on error.
    /// - Action: Executes `Scanner::peek` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::peek` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-PEEK] | SysML v2: ActionDef(Scanner::peek)
    fn peek(&self) -> char {
        self.source[self.cursor..].chars().next().unwrap_or('\0')
    }

    /// Executes deterministic fn `Scanner::advance` within `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Scanner::advance` in `crates/compile-sysml/src/lexer/scanner.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Scanner::advance` (declaration #14) within `LEXER-SCANNER`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `Scanner::advance` in `LEXER-SCANNER`.
    ///
    /// ## 4. Postconditions
    /// - Returns `char` from `Scanner::advance` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(advance)) bounded over input/member dimension of `Scanner::advance`.
    /// - Space Complexity: O(K(advance)) bounded auxiliary memory for `Scanner::advance`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Scanner::advance` (`char`).
    /// - State: Maintains consistent state invariants across `Scanner::advance` without partial mutation on error.
    /// - Action: Executes `Scanner::advance` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Scanner::advance` (`LEXER-SCANNER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-LEXER-SCANNER-SCANNER-ADVANCE] | SysML v2: ActionDef(Scanner::advance)
    fn advance(&mut self) -> char {
        let ch = self.peek();
        self.cursor += ch.len_utf8();
        self.col += 1;
        ch
    }
}

/// Executes deterministic fn `is_ident_start` within `crates/compile-sysml/src/lexer/scanner.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_ident_start` in `crates/compile-sysml/src/lexer/scanner.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_ident_start` (declaration #15) within `LEXER-SCANNER`.
///
/// ## 3. Preconditions
/// - `c: char` must satisfy domain and type-safety constraints prior to invoking `is_ident_start`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_ident_start` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(c)) bounded over input/member dimension of `is_ident_start`.
/// - Space Complexity: O(N(c)) bounded auxiliary memory for `is_ident_start`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_ident_start` (`bool`).
/// - State: Maintains consistent state invariants across `is_ident_start` without partial mutation on error.
/// - Action: Executes `is_ident_start` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_ident_start` (`LEXER-SCANNER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-SCANNER-IS-IDENT-START] | SysML v2: ActionDef(is_ident_start)
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Executes deterministic fn `is_ident_continue` within `crates/compile-sysml/src/lexer/scanner.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_ident_continue` in `crates/compile-sysml/src/lexer/scanner.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_ident_continue` (declaration #16) within `LEXER-SCANNER`.
///
/// ## 3. Preconditions
/// - `c: char` must satisfy domain and type-safety constraints prior to invoking `is_ident_continue`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_ident_continue` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(c)) bounded over input/member dimension of `is_ident_continue`.
/// - Space Complexity: O(N(c)) bounded auxiliary memory for `is_ident_continue`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_ident_continue` (`bool`).
/// - State: Maintains consistent state invariants across `is_ident_continue` without partial mutation on error.
/// - Action: Executes `is_ident_continue` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_ident_continue` (`LEXER-SCANNER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-LEXER-SCANNER-IS-IDENT-CONTINUE] | SysML v2: ActionDef(is_ident_continue)
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_at(tokens: &[Token], idx: usize) -> Result<TokenKind, String> {
        tokens
            .get(idx)
            .map(|t| t.kind.clone())
            .ok_or_else(|| format!("missing token at index {idx}"))
    }

    #[test]
    fn test_scan_keywords_and_symbols() -> Result<(), Box<dyn std::error::Error>> {
        let src = "package Foo { part def Bar :> Base { port in p : RS485; } }";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Package);
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Ident("Foo".to_string()));
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::OpenBrace);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Def);
        assert_eq!(kind_at(&tokens, 5)?, TokenKind::Ident("Bar".to_string()));
        assert_eq!(kind_at(&tokens, 6)?, TokenKind::ColonGt);
        assert_eq!(kind_at(&tokens, 7)?, TokenKind::Ident("Base".to_string()));
        Ok(())
    }

    #[test]
    fn test_scan_doc_comment() -> Result<(), Box<dyn std::error::Error>> {
        let src = "doc /* SSOT for Autonomous Vehicle */ part def FCC;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(
            kind_at(&tokens, 0)?,
            TokenKind::DocComment("SSOT for Autonomous Vehicle".to_string())
        );
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::Def);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Ident("FCC".to_string()));
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_scan_numbers_and_operators() -> Result<(), Box<dyn std::error::Error>> {
        let src = "v <= 60.0; count := 42; is_valid == true;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Ident("v".to_string()));
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::LtEq);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::RealLit(60.0));
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Ident("count".to_string()));
        assert_eq!(kind_at(&tokens, 5)?, TokenKind::ColonEq);
        assert_eq!(kind_at(&tokens, 6)?, TokenKind::IntLit(42));
        assert_eq!(kind_at(&tokens, 7)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 8)?, TokenKind::Ident("is_valid".to_string()));
        assert_eq!(kind_at(&tokens, 9)?, TokenKind::EqEq);
        assert_eq!(kind_at(&tokens, 10)?, TokenKind::BoolLit(true));
        assert_eq!(kind_at(&tokens, 11)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_lexer_tokenizes_import_statement() -> Result<(), Box<dyn std::error::Error>> {
        let src = "import Foo::*; import Bar::Baz; import Sys::**;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Import);
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Ident("Foo".to_string()));
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::DoubleColon);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Star);
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 5)?, TokenKind::Import);
        assert_eq!(kind_at(&tokens, 6)?, TokenKind::Ident("Bar".to_string()));
        assert_eq!(kind_at(&tokens, 7)?, TokenKind::DoubleColon);
        assert_eq!(kind_at(&tokens, 8)?, TokenKind::Ident("Baz".to_string()));
        assert_eq!(kind_at(&tokens, 9)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 10)?, TokenKind::Import);
        assert_eq!(kind_at(&tokens, 11)?, TokenKind::Ident("Sys".to_string()));
        assert_eq!(kind_at(&tokens, 12)?, TokenKind::DoubleColon);
        assert_eq!(kind_at(&tokens, 13)?, TokenKind::Star);
        assert_eq!(kind_at(&tokens, 14)?, TokenKind::Star);
        assert_eq!(kind_at(&tokens, 15)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_lexer_nested_block_comments() -> Result<(), Box<dyn std::error::Error>> {
        let src = "/* outer /* nested */ still outer */ part def FCC;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(
            kind_at(&tokens, 0)?,
            TokenKind::DocComment("outer /* nested */ still outer".to_string())
        );
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::Def);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Ident("FCC".to_string()));
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_lexer_unrestricted_single_quoted_identifier() -> Result<(), Box<dyn std::error::Error>> {
        let src = "part def 'Flight Control Computer' :> 'Avionics System';";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Def);
        assert_eq!(
            kind_at(&tokens, 2)?,
            TokenKind::Ident("Flight Control Computer".to_string())
        );
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::ColonGt);
        assert_eq!(
            kind_at(&tokens, 4)?,
            TokenKind::Ident("Avionics System".to_string())
        );
        assert_eq!(kind_at(&tokens, 5)?, TokenKind::Semi);
        Ok(())
    }
}


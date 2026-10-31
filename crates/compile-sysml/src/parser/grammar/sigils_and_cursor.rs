//! Type-sigil lexical pre-normalization, token-stream cursor primitives, and composite type-expression parsing.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Tokenizes UTF-8 SysML v2 / KerML source buffers while preserving exact 1-to-1 byte and column spans across `&`, `?`, and `'` sigils, advances `SysmlParser` token cursors deterministically, and parses contextual identifiers and bounded-depth composite type expressions.
//! Bounds & Faults: Caps recursive type-expression parsing at `MAX_PARSE_DEPTH` (64), returns `GRAMMAR_EOF_TOKEN` on out-of-bounds cursor reads, and uses checked `.get()` slice access and saturating index arithmetic with zero panics.

use super::*;

/// Static end-of-file sentinel token returned when the cursor reaches or exceeds an empty token slice.
/// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK]
pub(crate) static GRAMMAR_EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    span: Span {
        start: 0,
        end: 0,
        line: 0,
        col: 0,
    },
};

/// Exclusive lexical context tracked while masking standalone `&`, `?`, and non-identifier `'` sigils.
/// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-NORMALIZE-TYPE-SIGILS]
enum SigilScanMode {
    Code,
    LineComment,
    BlockComment(usize),
    Quoted(char),
}

/// Returns `true` when `kind` is a single-word KerML / SysML v2 keyword permitted in identifier positions (excluding `import` and multi-word `use case`).
/// Realises: [REQ-SYSML-CONTEXTUAL-IDENTIFIERS]
fn is_contextual_identifier_keyword(kind: &TokenKind) -> bool {
    if *kind == TokenKind::Import {
        return false;
    }
    let lexeme = kind.as_str();
    !lexeme.is_empty() && lexeme.bytes().all(|b| b.is_ascii_lowercase())
}

impl<'a> SysmlParser<'a> {
    /// Tokenizes complete SysML v2 / KerML source text into a `Vec<Token>` while preserving exact source byte and column spans for `&`, `?`, and `'` sigils.
    ///
    /// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-TOKENIZE-SOURCE]
    /// Contract: Masks standalone 1-byte sigils via `normalize_type_sigils`, runs `Scanner::scan_all`, and restores 1-byte `&`, `?`, and `'` tokens as `TokenKind::Ident` while keeping 3-byte single-quoted unrestricted identifiers intact.
    /// Bounds & Faults: Executes in `O(N)` time and `O(N)` space over `source.len()`, propagating scanner diagnostics as typed `ParseError` values without panicking.
    pub fn tokenize_source(source: &str) -> Result<Vec<Token>, ParseError> {
        let normalized = Self::normalize_type_sigils(source);
        let mut scanner = Scanner::new(&normalized);
        let mut tokens = scanner.scan_all().map_err(|msg| ParseError {
            message: msg,
            span: Span::default(),
        })?;
        let bytes = source.as_bytes();
        for tok in &mut tokens {
            if tok.kind == TokenKind::Comma && tok.span.end.saturating_sub(tok.span.start) == 1 {
                match bytes.get(tok.span.start).copied() {
                    Some(b'&') => tok.kind = TokenKind::Ident("&".to_string()),
                    Some(b'?') => tok.kind = TokenKind::Ident("?".to_string()),
                    Some(b'\'') => tok.kind = TokenKind::Ident("'".to_string()),
                    _ => {}
                }
            }
        }
        Ok(tokens)
    }

    /// Pre-processes standalone reference `&`, optional `?`, and lifetime/unclosed `'` sigils outside comments and literals into 1-byte `,` placeholders preserving exact byte and column offsets.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-NORMALIZE-TYPE-SIGILS]
    pub(crate) fn normalize_type_sigils(source: &str) -> String {
        if !source.contains('&') && !source.contains('?') && !source.contains('\'') {
            return source.to_string();
        }
        let chars: Vec<char> = source.chars().collect();
        let mut out = String::with_capacity(source.len());
        let mut i = 0usize;
        let mut mode = SigilScanMode::Code;

        while let Some(&c) = chars.get(i) {
            let next = chars.get(i.saturating_add(1)).copied();
            let step = match mode {
                SigilScanMode::LineComment => {
                    out.push(c);
                    if c == '\n' {
                        mode = SigilScanMode::Code;
                    }
                    1
                }
                SigilScanMode::BlockComment(depth) => {
                    if c == '/' && next == Some('*') {
                        out.push_str("/*");
                        mode = SigilScanMode::BlockComment(depth.saturating_add(1));
                        2
                    } else if c == '*' && next == Some('/') {
                        out.push_str("*/");
                        let next_depth = depth.saturating_sub(1);
                        mode = if next_depth == 0 {
                            SigilScanMode::Code
                        } else {
                            SigilScanMode::BlockComment(next_depth)
                        };
                        2
                    } else {
                        out.push(c);
                        1
                    }
                }
                SigilScanMode::Quoted(quote) => {
                    out.push(c);
                    if c == '\\' {
                        if let Some(escaped) = next {
                            out.push(escaped);
                            2
                        } else {
                            1
                        }
                    } else {
                        if c == quote || (quote == '\'' && c == '\n') {
                            mode = SigilScanMode::Code;
                        }
                        1
                    }
                }
                SigilScanMode::Code => match (c, next) {
                    ('/', Some('/')) => {
                        out.push_str("//");
                        mode = SigilScanMode::LineComment;
                        2
                    }
                    ('/', Some('*')) => {
                        out.push_str("/*");
                        mode = SigilScanMode::BlockComment(1);
                        2
                    }
                    ('&', Some('&')) => {
                        out.push_str("&&");
                        2
                    }
                    ('"', _) => {
                        out.push('"');
                        mode = SigilScanMode::Quoted('"');
                        1
                    }
                    ('\'', _) if Self::is_closed_single_quoted_ident(&chars, i) => {
                        out.push('\'');
                        mode = SigilScanMode::Quoted('\'');
                        1
                    }
                    ('&' | '?' | '\'', _) => {
                        out.push(',');
                        1
                    }
                    _ => {
                        out.push(c);
                        1
                    }
                },
            };
            i = i.saturating_add(step);
        }

        out
    }

    /// Determines whether the single quote at `chars.get(start)` opens a closed single-quoted unrestricted identifier on the same line rather than a lifetime prefix (`&'a T`) or unclosed quote.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-IS-CLOSED-SINGLE-QUOTED-IDENT]
    pub(crate) fn is_closed_single_quoted_ident(chars: &[char], start: usize) -> bool {
        if start > 0 && chars.get(start.saturating_sub(1)) == Some(&'&') {
            return false;
        }
        let first_inner = start.saturating_add(1);
        let mut j = first_inner;
        while let Some(&ch) = chars.get(j) {
            if ch == '\n' || ch == '\r' {
                return false;
            }
            if ch == '\\' {
                j = j.saturating_add(2);
                continue;
            }
            if ch == '\'' {
                let followed_by_ident = chars
                    .get(j.saturating_add(1))
                    .is_some_and(|after| after.is_ascii_alphanumeric() || *after == '_');
                return !followed_by_ident && j > first_inner;
            }
            j = j.saturating_add(1);
        }
        false
    }

    /// Advances the cursor past consecutive line-comment and standalone semicolon trivia tokens.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-SKIP-TRIVIA]
    pub(crate) fn skip_trivia(&mut self) {
        while matches!(
            self.peek_kind(),
            TokenKind::LineComment(_) | TokenKind::Semi
        ) {
            self.advance();
        }
    }

    /// Consumes and returns a leading `TokenKind::DocComment` payload if present at the current cursor position.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-TAKE-DOC-COMMENT]
    pub(crate) fn take_doc_comment(&mut self) -> Option<String> {
        if let TokenKind::DocComment(text) = self.peek_kind().clone() {
            self.advance();
            Some(text)
        } else {
            None
        }
    }

    /// Extracts an inner documentation comment (`/* ... */`, `doc /* ... */`, or `doc = "..."`) inside a braced declaration body and consumes any trailing semicolon.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-TAKE-INNER-DOC-COMMENT]
    pub(crate) fn take_inner_doc_comment(&mut self) -> Option<String> {
        let doc = if let Some(d) = self.take_doc_comment() {
            Some(d)
        } else if self.eat(&TokenKind::Doc) {
            self.skip_trivia();
            if self.eat(&TokenKind::Eq) || self.eat(&TokenKind::Colon) {
                self.skip_trivia();
            }
            if let Some(d) = self.take_doc_comment() {
                Some(d)
            } else {
                match self.peek_kind().clone() {
                    TokenKind::StringLit(s) | TokenKind::Ident(s) => {
                        self.advance();
                        Some(s)
                    }
                    _ => Some(String::new()),
                }
            }
        } else {
            return None;
        };
        while matches!(self.peek_kind(), TokenKind::LineComment(_)) {
            self.advance();
        }
        self.eat(&TokenKind::Semi);
        doc
    }

    /// Checks whether the current token variant matches `kind` by enum discriminant without advancing the cursor.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-CHECK]
    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end()
            && self
                .tokens
                .get(self.pos)
                .is_some_and(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind))
    }

    /// Consumes the current token and returns `true` when its discriminant matches `kind`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-CHECK]
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Returns a reference to the current `Token` at `self.pos`, falling back to the final token or `GRAMMAR_EOF_TOKEN` when empty.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK]
    pub(crate) fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .unwrap_or(&GRAMMAR_EOF_TOKEN)
    }

    /// Returns a reference to the current `TokenKind`, yielding `&TokenKind::Eof` when out of bounds.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK-KIND]
    pub(crate) fn peek_kind(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map_or(&TokenKind::Eof, |t| &t.kind)
    }

    /// Advances `self.pos` by one token when within bounds and returns the consumed token or `GRAMMAR_EOF_TOKEN`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-ADVANCE]
    pub(crate) fn advance(&mut self) -> &Token {
        let idx = self.pos;
        if self.pos < self.tokens.len() {
            self.pos = self.pos.saturating_add(1);
        }
        self.tokens
            .get(idx)
            .or_else(|| self.tokens.last())
            .unwrap_or(&GRAMMAR_EOF_TOKEN)
    }

    /// Returns `true` when `self.pos` is out of bounds or points to `TokenKind::Eof`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-IS-AT-END]
    pub(crate) fn is_at_end(&self) -> bool {
        self.tokens
            .get(self.pos)
            .is_none_or(|t| t.kind == TokenKind::Eof)
    }

    /// Returns `true` when the cursor sits on a statement terminator (`;`, `}`, or `EOF`).
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-READ-UNTIL-SEMI]
    fn at_stmt_end(&self) -> bool {
        self.check(&TokenKind::Semi) || self.check(&TokenKind::CloseBrace) || self.is_at_end()
    }

    /// Constructs a `ParseError` anchored at the current `self.peek()` span.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT]
    fn err_at_peek(&self, message: String) -> ParseError {
        ParseError {
            message,
            span: self.peek().span,
        }
    }

    /// Consumes the current token if its discriminant matches `expected`, or returns a span-annotated `ParseError`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT]
    pub(crate) fn expect(&mut self, expected: &TokenKind) -> Result<&Token, ParseError> {
        if self.check(expected) {
            Ok(self.advance())
        } else {
            Err(self.err_at_peek(format!(
                "Expected '{:?}', found '{:?}'",
                expected,
                self.peek_kind()
            )))
        }
    }

    /// Parses a standard or single-quoted identifier or contextual SysML v2 / KerML keyword permitted in identifier position.
    ///
    /// Safety & Traceability: Realises: [REQ-SYSML-CONTEXTUAL-IDENTIFIERS]
    /// Contract: Accepts non-sigil `TokenKind::Ident` tokens and single-word contextual keywords (`is_contextual_identifier_keyword`) while rejecting standalone 1-byte `&`, `?`, and `'` sigils, operators, literals, and `import`.
    /// Bounds & Faults: Executes in `O(1)` token lookahead and returns a span-accurate `ParseError` without advancing the cursor on mismatch.
    pub fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let tok = self.peek();
        match &tok.kind {
            TokenKind::Ident(s)
                if !((s == "&" || s == "?" || s == "'")
                    && tok.span.end.saturating_sub(tok.span.start) == 1) =>
            {
                let name = s.clone();
                self.advance();
                Ok(name)
            }
            kind if is_contextual_identifier_keyword(kind) => {
                let name = kind.as_str().to_string();
                self.advance();
                Ok(name)
            }
            other => Err(ParseError {
                message: format!("Expected identifier, found '{:?}'", other),
                span: tok.span,
            }),
        }
    }

    /// Alias helper delegating identifier consumption to `SysmlParser::parse_identifier`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-IDENT]
    pub(crate) fn expect_ident(&mut self) -> Result<String, ParseError> {
        self.parse_identifier()
    }

    /// Returns `true` when the current token is a 1-byte normalized type sigil (`&`, `?`, or `'`) rather than a quoted identifier.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    fn peek_is_sigil(&self, sigil: &str) -> bool {
        let tok = self.peek();
        matches!(&tok.kind, TokenKind::Ident(s) if s == sigil)
            && tok.span.end.saturating_sub(tok.span.start) == 1
    }

    /// Parses a simple or composite type expression (`*const/*mut`, `&`, `dyn`, tuples, arrays, generics, multiplicity, `?`) bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    pub(crate) fn parse_type_expr(&mut self) -> Result<String, ParseError> {
        if self.depth >= MAX_PARSE_DEPTH {
            return Err(self.max_depth_error());
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.parse_type_expr_inner();
        self.depth = self.depth.saturating_sub(1);
        result
    }

    /// Parses comma-separated type expressions up to `close` and consumes `close`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    fn parse_comma_type_list(&mut self, close: &TokenKind) -> Result<Vec<String>, ParseError> {
        let mut items = Vec::new();
        while !self.check(close) && !self.is_at_end() {
            items.push(self.parse_type_expr()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(close)?;
        Ok(items)
    }

    /// Dispatches prefix and primary composite type expression forms within an active recursion depth frame.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    fn parse_type_expr_inner(&mut self) -> Result<String, ParseError> {
        if self.eat(&TokenKind::Star) {
            let kw = self.expect_ident()?;
            if kw != "const" && kw != "mut" {
                return Err(
                    self.err_at_peek(format!("Expected 'const' or 'mut' after '*', found '{kw}'"))
                );
            }
            return Ok(format!("*{kw} {}", self.parse_type_expr()?));
        }
        if self.peek_is_sigil("&") || self.check(&TokenKind::AmpAmp) {
            return self.parse_ref_type_expr();
        }
        if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "dyn") {
            self.advance();
            return Ok(format!("dyn {}", self.parse_type_expr()?));
        }

        let mut out = if self.eat(&TokenKind::OpenParen) {
            if self.eat(&TokenKind::CloseParen) {
                "()".to_string()
            } else {
                let elems = self.parse_comma_type_list(&TokenKind::CloseParen)?;
                format!("({})", elems.join(", "))
            }
        } else if self.eat(&TokenKind::OpenBracket) {
            let elem_ty = self.parse_type_expr()?;
            if self.eat(&TokenKind::Semi) {
                let tok = self.advance();
                let len_str = match &tok.kind {
                    TokenKind::IntLit(n) => n.to_string(),
                    TokenKind::Ident(s) => s.clone(),
                    other => {
                        return Err(ParseError {
                            message: format!("Expected array length, found '{:?}'", other),
                            span: tok.span,
                        });
                    }
                };
                self.expect(&TokenKind::CloseBracket)?;
                format!("[{elem_ty}; {len_str}]")
            } else {
                self.expect(&TokenKind::CloseBracket)?;
                format!("[{elem_ty}]")
            }
        } else {
            self.parse_base_or_generic_type_expr()?
        };

        self.append_type_suffixes(&mut out)?;
        Ok(out)
    }

    /// Parses reference type prefixes (`&T`, `&&T`, `&'a T`, `&'a mut T`, `&const T`).
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    fn parse_ref_type_expr(&mut self) -> Result<String, ParseError> {
        let mut out = if self.eat(&TokenKind::AmpAmp) {
            "&&".to_string()
        } else {
            self.advance();
            "&".to_string()
        };
        if self.peek_is_sigil("'") {
            self.advance();
            out.push('\'');
            out.push_str(&self.parse_identifier()?);
            out.push(' ');
        }
        if let TokenKind::Ident(s) = self.peek_kind().clone() {
            if s == "mut" || s == "const" {
                self.advance();
                out.push_str(&s);
                out.push(' ');
            }
        }
        out.push_str(&self.parse_type_expr()?);
        Ok(out)
    }

    /// Parses path-qualified type names `(::)?Ident(::Ident)*` and optional generic argument lists `<T1, T2, ...>`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    fn parse_base_or_generic_type_expr(&mut self) -> Result<String, ParseError> {
        let mut out = String::new();
        if self.eat(&TokenKind::DoubleColon) {
            out.push_str("::");
        }
        out.push_str(&self.expect_ident()?);
        while self.eat(&TokenKind::DoubleColon) {
            out.push_str("::");
            out.push_str(&self.expect_ident()?);
        }
        if self.eat(&TokenKind::Lt) {
            let args = self.parse_comma_type_list(&TokenKind::Gt)?;
            if args.is_empty() {
                return Err(self
                    .err_at_peek("Empty generic type argument list '<>' is invalid".to_string()));
            }
            out.push('<');
            out.push_str(&args.join(", "));
            out.push('>');
        }
        Ok(out)
    }

    /// Appends trailing multiplicity/array suffixes (`Type[]`, `Type[N]`, `Type[1..*]`) and optional `?` markers to `out`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR]
    fn append_type_suffixes(&mut self, out: &mut String) -> Result<(), ParseError> {
        loop {
            if self.eat(&TokenKind::OpenBracket) {
                let mut suffix = String::new();
                while !self.check(&TokenKind::CloseBracket) && !self.at_stmt_end() {
                    let tok = self.advance();
                    match &tok.kind {
                        TokenKind::IntLit(n) => suffix.push_str(&n.to_string()),
                        TokenKind::Ident(s) => suffix.push_str(s),
                        _ => suffix.push_str(tok.kind.as_str()),
                    }
                }
                self.expect(&TokenKind::CloseBracket)?;
                out.push('[');
                out.push_str(&suffix);
                out.push(']');
            } else if self.peek_is_sigil("?") {
                self.advance();
                out.push('?');
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Consumes a simple or dot-separated (`a.b.c`) identifier path.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-IDENT-OR-DOTTED]
    pub(crate) fn expect_ident_or_dotted(&mut self) -> Result<String, ParseError> {
        let mut result = self.expect_ident()?;
        while self.eat(&TokenKind::Dot) {
            result.push('.');
            result.push_str(&self.expect_ident()?);
        }
        Ok(result)
    }

    /// Consumes a double-quoted string literal or an identifier/contextual keyword, returning its unquoted text.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-STRING-OR-IDENT]
    pub(crate) fn expect_string_or_ident(&mut self) -> Result<String, ParseError> {
        if let TokenKind::StringLit(s) = self.peek_kind().clone() {
            self.advance();
            return Ok(s);
        }
        self.parse_identifier().map_err(|_| {
            self.err_at_peek(format!(
                "Expected string or identifier, found '{:?}'",
                self.peek_kind()
            ))
        })
    }

    /// Consumes a required `;` statement terminator or returns a span-accurate `ParseError`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-SEMI]
    pub(crate) fn expect_semi(&mut self) -> Result<(), ParseError> {
        if self.eat(&TokenKind::Semi) {
            Ok(())
        } else {
            Err(self.err_at_peek(format!(
                "Expected ';' statement terminator, found '{:?}'",
                self.peek_kind()
            )))
        }
    }

    /// Consumes tokens up to `;` or enclosing `}` (consuming `;` if present) and joins their display lexemes with spaces.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-READ-UNTIL-SEMI]
    pub(crate) fn read_until_semi(&mut self) -> String {
        let mut pieces = Vec::new();
        while !self.at_stmt_end() {
            let tok = self.advance();
            let piece = match &tok.kind {
                TokenKind::Ident(s) => s.clone(),
                TokenKind::StringLit(s) => format!("\"{s}\""),
                TokenKind::IntLit(n) => n.to_string(),
                TokenKind::RealLit(f) if f.fract() == 0.0 => format!("{f:.1}"),
                TokenKind::RealLit(f) => f.to_string(),
                TokenKind::BoolLit(b) => b.to_string(),
                _ => tok.kind.as_str().to_string(),
            };
            pieces.push(piece);
        }
        self.eat(&TokenKind::Semi);
        pieces.join(" ")
    }

    /// Collects non-comment `Token` values up to `;` or enclosing `}`, consuming `;` if present.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-READ-TOKENS-UNTIL-SEMI]
    pub(crate) fn read_tokens_until_semi(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while !self.at_stmt_end() {
            let tok = self.advance().clone();
            if !matches!(
                tok.kind,
                TokenKind::LineComment(_) | TokenKind::DocComment(_)
            ) {
                out.push(tok);
            }
        }
        self.eat(&TokenKind::Semi);
        out
    }
}

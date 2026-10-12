//! Zero-copy slice-based tokenizer for SysML v2 / KerML syntax.

use super::token::{Span, Token, TokenKind};

/// Lexical scanner translating source text into tokens.
pub struct Scanner<'a> {
    source: &'a str,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    /// Scan all tokens from the source until EOF.
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

    fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.source[self.cursor..].starts_with(prefix)
    }

    fn peek(&self) -> char {
        self.source[self.cursor..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        let ch = self.peek();
        self.cursor += ch.len_utf8();
        self.col += 1;
        ch
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_keywords_and_symbols() {
        let src = "package Foo { part def Bar :> Base { port in p : RS485; } }";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Package);
        assert_eq!(tokens[1].kind, TokenKind::Ident("Foo".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::OpenBrace);
        assert_eq!(tokens[3].kind, TokenKind::Part);
        assert_eq!(tokens[4].kind, TokenKind::Def);
        assert_eq!(tokens[5].kind, TokenKind::Ident("Bar".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::ColonGt);
        assert_eq!(tokens[7].kind, TokenKind::Ident("Base".to_string()));
    }

    #[test]
    fn test_scan_doc_comment() {
        let src = "doc /* SSOT for Autonomous Vehicle */ part def FCC;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all().unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::DocComment("SSOT for Autonomous Vehicle".to_string())
        );
        assert_eq!(tokens[1].kind, TokenKind::Part);
        assert_eq!(tokens[2].kind, TokenKind::Def);
        assert_eq!(tokens[3].kind, TokenKind::Ident("FCC".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Semi);
    }

    #[test]
    fn test_scan_numbers_and_operators() {
        let src = "v <= 60.0; count := 42; is_valid == true;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Ident("v".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::LtEq);
        assert_eq!(tokens[2].kind, TokenKind::RealLit(60.0));
        assert_eq!(tokens[3].kind, TokenKind::Semi);

        assert_eq!(tokens[4].kind, TokenKind::Ident("count".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::ColonEq);
        assert_eq!(tokens[6].kind, TokenKind::IntLit(42));
        assert_eq!(tokens[7].kind, TokenKind::Semi);

        assert_eq!(tokens[8].kind, TokenKind::Ident("is_valid".to_string()));
        assert_eq!(tokens[9].kind, TokenKind::EqEq);
        assert_eq!(tokens[10].kind, TokenKind::BoolLit(true));
        assert_eq!(tokens[11].kind, TokenKind::Semi);
    }

    #[test]
    fn test_lexer_tokenizes_import_statement() {
        let src = "import Foo::*; import Bar::Baz; import Sys::**;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Import);
        assert_eq!(tokens[1].kind, TokenKind::Ident("Foo".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::DoubleColon);
        assert_eq!(tokens[3].kind, TokenKind::Star);
        assert_eq!(tokens[4].kind, TokenKind::Semi);

        assert_eq!(tokens[5].kind, TokenKind::Import);
        assert_eq!(tokens[6].kind, TokenKind::Ident("Bar".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::DoubleColon);
        assert_eq!(tokens[8].kind, TokenKind::Ident("Baz".to_string()));
        assert_eq!(tokens[9].kind, TokenKind::Semi);

        assert_eq!(tokens[10].kind, TokenKind::Import);
        assert_eq!(tokens[11].kind, TokenKind::Ident("Sys".to_string()));
        assert_eq!(tokens[12].kind, TokenKind::DoubleColon);
        assert_eq!(tokens[13].kind, TokenKind::Star);
        assert_eq!(tokens[14].kind, TokenKind::Star);
        assert_eq!(tokens[15].kind, TokenKind::Semi);
    }

    #[test]
    fn test_lexer_nested_block_comments() {
        let src = "/* outer /* nested */ still outer */ part def FCC;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all().unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::DocComment("outer /* nested */ still outer".to_string())
        );
        assert_eq!(tokens[1].kind, TokenKind::Part);
        assert_eq!(tokens[2].kind, TokenKind::Def);
        assert_eq!(tokens[3].kind, TokenKind::Ident("FCC".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Semi);
    }

    #[test]
    fn test_lexer_unrestricted_single_quoted_identifier() {
        let src = "part def 'Flight Control Computer' :> 'Avionics System';";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Part);
        assert_eq!(tokens[1].kind, TokenKind::Def);
        assert_eq!(
            tokens[2].kind,
            TokenKind::Ident("Flight Control Computer".to_string())
        );
        assert_eq!(tokens[3].kind, TokenKind::ColonGt);
        assert_eq!(
            tokens[4].kind,
            TokenKind::Ident("Avionics System".to_string())
        );
        assert_eq!(tokens[5].kind, TokenKind::Semi);
    }
}


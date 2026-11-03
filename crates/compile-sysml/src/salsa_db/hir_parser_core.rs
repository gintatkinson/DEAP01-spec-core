#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Recursive descent parser translating SysML v2 / KerML tokens into typed HIR/AST.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE] | [FLS: fls_66m4rnbssgig]
//! Contract: Parses lexed SysML v2 / KerML token streams into deterministic `PackageDef` ASTs.
//! Bounds & Faults: Caps recursive-descent and package-merge nesting depth at `MAX_PARSE_DEPTH` (64), returning typed `ParseError` diagnostics without panicking.

use super::cst_db::{Span, Token, TokenKind};
pub(crate) use super::hir_parser_cursor::*;
use deap_core::sysml_cst::*;
use std::fmt;

/// Maximum recursive-descent and package-merge nesting depth permitted in `SysmlParser` and `merge_package_defs`.
/// Realises: [REQ-PARSER-GRAMMAR-MAX-PARSE-DEPTH] | [FLS: fls_66m4rnbssgig]
pub(crate) const MAX_PARSE_DEPTH: usize = 64;

/// Diagnostic syntax or filesystem loading error paired with a 1-based source location `Span`.
/// Realises: [REQ-PARSER-GRAMMAR-PARSEERROR] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    /// Formats `ParseError` with 1-based line, column, and diagnostic message.
    /// Realises: [REQ-PARSER-GRAMMAR-PARSEERROR-FMT] | [FLS: fls_66m4rnbssgig]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, col {}: {}",
            self.span.line, self.span.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Depth-bounded recursive-descent cursor translating a SysML v2 / KerML `Token` slice into typed AST definitions.
/// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER] | [FLS: fls_66m4rnbssgig]
pub struct SysmlParser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) pos: usize,
    pub(crate) depth: usize,
}

impl<'a> SysmlParser<'a> {
    /// Initializes a `SysmlParser` cursor positioned at token index `0` and recursion depth `0`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-NEW] | [FLS: fls_66m4rnbssgig]
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            depth: 0,
        }
    }

    /// Constructs a deterministic `ParseError` at the current cursor span when `MAX_PARSE_DEPTH` is reached.
    /// Realises: [REQ-PARSER-GRAMMAR-MAX-PARSE-DEPTH] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn max_depth_error(&self) -> ParseError {
        let tok = self.peek();
        ParseError {
            message: format!("Maximum SysML parse recursion depth ({MAX_PARSE_DEPTH}) exceeded"),
            span: tok.span,
        }
    }

    /// Tokenizes and parses complete SysML v2 / KerML source text into a root `PackageDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-SOURCE] | [FLS: fls_66m4rnbssgig]
    pub fn parse_source(source: &str, default_pkg_name: &str) -> Result<PackageDef, ParseError> {
        let tokens = Self::tokenize_source(source)?;
        let mut parser = SysmlParser::new(&tokens);
        parser.parse_package_or_file(default_pkg_name)
    }

    /// Returns `true` when the current token is a 1-byte normalized type sigil (`&`, `?`, or `'`) rather than a quoted identifier.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn peek_is_sigil(&self, sigil: &str) -> bool {
        let tok = self.peek();
        matches!(&tok.kind, TokenKind::Ident(s) if s == sigil)
            && tok.span.end.saturating_sub(tok.span.start) == 1
    }

    /// Parses a simple or composite type expression (`*const/*mut`, `&`, `dyn`, tuples, arrays, generics, multiplicity, `?`) bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
    fn parse_type_expr_inner(&mut self) -> Result<String, ParseError> {
        if self.eat(&TokenKind::Star) {
            let kw = self.expect_ident()?;
            if kw != "const" && kw != "mut" {
                return Err(self.err_at_peek(format!("Expected 'const' or 'mut' after '*', found '{kw}'")));
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
                format!("({})", self.parse_comma_type_list(&TokenKind::CloseParen)?.join(", "))
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
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
                return Err(self.err_at_peek("Empty generic type argument list '<>' is invalid".to_string()));
            }
            out.push('<');
            out.push_str(&args.join(", "));
            out.push('>');
        }
        Ok(out)
    }

    /// Appends trailing multiplicity/array suffixes (`Type[]`, `Type[N]`, `Type[1..*]`) and optional `?` markers to `out`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TYPE-EXPR] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-IDENT-OR-DOTTED] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn expect_ident_or_dotted(&mut self) -> Result<String, ParseError> {
        let mut result = self.expect_ident()?;
        while self.eat(&TokenKind::Dot) {
            result.push('.');
            result.push_str(&self.expect_ident()?);
        }
        Ok(result)
    }

    /// Consumes a double-quoted string literal or an identifier/contextual keyword, returning its unquoted text.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-STRING-OR-IDENT] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn expect_string_or_ident(&mut self) -> Result<String, ParseError> {
        if let TokenKind::StringLit(s) = self.peek_kind().clone() {
            self.advance();
            return Ok(s);
        }
        self.parse_identifier().map_err(|_| {
            self.err_at_peek(format!("Expected string or identifier, found '{:?}'", self.peek_kind()))
        })
    }

    /// Consumes a required `;` statement terminator or returns a span-accurate `ParseError`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-EXPECT-SEMI] | [FLS: fls_66m4rnbssgig]
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

    /// Parses a `capability` or `capability def` declaration and optional `subsystem` binding into a `CapabilityDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CAPABILITY-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_capability_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<CapabilityDef, ParseError> {
        self.expect(&TokenKind::Capability)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut cap_doc = doc;
        let mut subsystem = None;
        if self.eat(&TokenKind::OpenBrace) {
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                while let Some(d) = self.take_inner_doc_comment() {
                    if cap_doc.is_none() {
                        cap_doc = Some(d);
                    }
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.eat(&TokenKind::Subsystem) {
                    subsystem = Some(self.read_until_semi());
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!("Unexpected token '{:?}' in capability declaration body", tok.kind),
                        span: tok.span,
                    });
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }
        Ok(CapabilityDef {
            name,
            description: cap_doc.clone(),
            subsystem,
            package_ref: None,
            doc: cap_doc,
        })
    }
}

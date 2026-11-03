//! Structural connection, item-flow block, and inline connector parsers for SysML v2 / KerML.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses `connection` / `connection def`, `flow` / `flow def`, and inline `connect [from] <src> to <tgt>;` statements into typed `ConnectionDef` AST nodes with endpoint owner resolution.
//! Bounds & Faults: Uses checked `.get()` token lookahead, `.split_once('.')` endpoint decomposition, and saturating index arithmetic, returning span-annotated `ParseError` diagnostics on malformed connector bodies.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Parses a `connection` / `connection def` or `flow` / `flow def` block into a `ConnectionDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONNECTION-DECL]
    pub(crate) fn parse_connection_decl(
        &mut self,
        doc: Option<String>,
        is_flow: bool,
    ) -> Result<ConnectionDef, ParseError> {
        if is_flow {
            self.expect(&TokenKind::Flow)?;
        } else {
            self.expect(&TokenKind::Connection)?;
        }
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut conn = ConnectionDef {
            name,
            source_port: String::new(),
            target_port: String::new(),
            source_part: None,
            target_part: None,
            doc,
            severity: 1,
            item_flow_ref: None,
            protocol: None,
            latency_ms: None,
            is_flow,
        };
        let mut explicit_source_part: Option<Option<String>> = None;
        let mut explicit_target_part: Option<Option<String>> = None;

        if self.check(&TokenKind::OpenBrace) {
            self.parse_connection_body(
                &mut conn,
                &mut explicit_source_part,
                &mut explicit_target_part,
            )?;
        } else {
            self.expect_semi()?;
        }

        conn.source_part = explicit_source_part.unwrap_or_else(|| {
            conn.source_port
                .split_once('.')
                .map(|(owner, _)| owner.to_string())
        });
        conn.target_part = explicit_target_part.unwrap_or_else(|| {
            conn.target_port
                .split_once('.')
                .map(|(owner, _)| owner.to_string())
        });

        Ok(conn)
    }

    /// Parses the braced body of a `connection` or `flow` declaration into `conn`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONNECTION-DECL]
    fn parse_connection_body(
        &mut self,
        conn: &mut ConnectionDef,
        explicit_source_part: &mut Option<Option<String>>,
        explicit_target_part: &mut Option<Option<String>>,
    ) -> Result<(), ParseError> {
        self.expect(&TokenKind::OpenBrace)?;
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            self.skip_trivia();
            while let Some(inner_doc) = self.take_inner_doc_comment() {
                if conn.doc.is_none() {
                    conn.doc = Some(inner_doc);
                }
                self.skip_trivia();
            }
            if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                break;
            }
            if self.check(&TokenKind::Connect)
                || (self.check(&TokenKind::Flow) && self.peek_next_is_from())
            {
                self.advance();
                if self.check(&TokenKind::From) {
                    self.advance();
                }
                conn.source_port = self.expect_ident_or_dotted()?;
                self.expect(&TokenKind::To)?;
                conn.target_port = self.expect_ident_or_dotted()?;
                self.expect_semi()?;
            } else if self.check(&TokenKind::Attribute) {
                let attr = self.parse_attribute_decl(None)?;
                Self::apply_connection_attribute(
                    conn,
                    explicit_source_part,
                    explicit_target_part,
                    attr,
                );
            } else {
                let tok = self.peek();
                return Err(ParseError {
                    message: format!(
                        "Unexpected token '{:?}' in connection declaration body",
                        tok.kind
                    ),
                    span: tok.span,
                });
            }
        }
        self.expect(&TokenKind::CloseBrace)?;
        Ok(())
    }

    /// Applies a parsed `AttributeDef` inside a `connection` or `flow` body (`protocol`, `latency_ms`, `severity`, `item_flow_ref`, `source_part`, `target_part`) to `conn`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-APPLY-CONNECTION-ATTRIBUTE]
    pub(crate) fn apply_connection_attribute(
        conn: &mut ConnectionDef,
        explicit_source_part: &mut Option<Option<String>>,
        explicit_target_part: &mut Option<Option<String>>,
        attr: AttributeDef,
    ) {
        let unquoted = attr
            .default_value
            .as_ref()
            .map(|v| v.trim_matches('"').to_string());
        match attr.name.as_str() {
            "protocol" => conn.protocol = unquoted,
            "latency_ms" | "latency" => {
                conn.latency_ms = unquoted.and_then(|v| v.replace(' ', "").parse::<f64>().ok());
            }
            "severity" => {
                conn.severity = unquoted
                    .and_then(|v| v.replace(' ', "").parse::<i32>().ok())
                    .unwrap_or(1);
            }
            "item_flow_ref" | "item_flow" => {
                conn.item_flow_ref = unquoted.filter(|v| !v.is_empty());
            }
            "source_part" => {
                *explicit_source_part = Some(unquoted.filter(|v| !v.is_empty()));
            }
            "target_part" => {
                *explicit_target_part = Some(unquoted.filter(|v| !v.is_empty()));
            }
            _ => {}
        }
    }

    /// Parses a standalone inline `connect [from] <source_port> to <target_port>;` statement into a synthesized `ConnectionDef`.
    /// Realises: [REQ-SYSML-PARSER-INLINE-CONNECTOR]
    pub(crate) fn parse_inline_connect_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<ConnectionDef, ParseError> {
        self.expect(&TokenKind::Connect)?;
        if self.check(&TokenKind::From) {
            self.advance();
        }
        let source_port = self.expect_ident_or_dotted()?;

        if self.check(&TokenKind::To) {
            self.advance();
        } else if let TokenKind::Ident(s) = &self.peek().kind {
            if s == "to" {
                self.advance();
            } else {
                let tok = self.peek();
                return Err(ParseError {
                    message: format!(
                        "Expected 'to' keyword in connect statement, found '{:?}'",
                        tok.kind
                    ),
                    span: tok.span,
                });
            }
        } else {
            let tok = self.peek();
            return Err(ParseError {
                message: format!(
                    "Expected 'to' keyword in connect statement, found '{:?}'",
                    tok.kind
                ),
                span: tok.span,
            });
        }

        let target_port = self.expect_ident_or_dotted()?;
        self.expect_semi()?;

        let source_part = source_port
            .split_once('.')
            .map(|(owner, _)| owner.to_string());
        let target_part = target_port
            .split_once('.')
            .map(|(owner, _)| owner.to_string());

        let synth_name = format!(
            "c_{}_to_{}",
            source_port.replace('.', "_"),
            target_port.replace('.', "_")
        );

        Ok(ConnectionDef {
            name: synth_name,
            source_port,
            target_port,
            source_part,
            target_part,
            doc,
            severity: 1,
            item_flow_ref: None,
            protocol: None,
            latency_ms: None,
            is_flow: false,
        })
    }

    /// Returns `true` when the next token at `self.pos + 1` is `TokenKind::From` (distinguishing `flow from a to b;` inside a connection body).
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK-NEXT-IS-FROM]
    pub(crate) fn peek_next_is_from(&self) -> bool {
        self.tokens
            .get(self.pos.saturating_add(1))
            .is_some_and(|t| t.kind == TokenKind::From)
    }

    /// Parses an `attribute` or `attribute def` declaration with optional type expression and default value into an `AttributeDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-ATTRIBUTE-DECL]
    pub(crate) fn parse_attribute_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<AttributeDef, ParseError> {
        self.expect(&TokenKind::Attribute)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut type_name = "String".to_string();
        let mut default_value = None;
        if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
            self.advance();
            type_name = self.parse_type_expr()?;
        }
        if self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq) {
            self.advance();
            default_value = Some(self.read_until_semi());
        } else {
            self.expect_semi()?;
        }
        Ok(AttributeDef {
            name,
            type_name,
            default_value,
            doc,
        })
    }

    /// Parses an `item` or `item def` payload declaration and its inner `attribute` members into an `ItemDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-ITEM-DECL]
    pub(crate) fn parse_item_decl(&mut self, doc: Option<String>) -> Result<ItemDef, ParseError> {
        self.expect(&TokenKind::Item)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut item_doc = doc;
        let mut attributes = Vec::new();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                let mut a_doc = self.take_inner_doc_comment();
                self.skip_trivia();
                while let Some(next_doc) = self.take_inner_doc_comment() {
                    if item_doc.is_none() {
                        item_doc = a_doc;
                    }
                    a_doc = Some(next_doc);
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    if item_doc.is_none() {
                        item_doc = a_doc;
                    }
                    break;
                }
                if self.check(&TokenKind::Attribute) {
                    attributes.push(self.parse_attribute_decl(a_doc)?);
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!(
                            "Unexpected token '{:?}' in item declaration body",
                            tok.kind
                        ),
                        span: tok.span,
                    });
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }
        Ok(ItemDef {
            name,
            doc: item_doc,
            attributes,
        })
    }
}

//! Stereotype annotation, part, port, attribute, and item structural declaration parsers.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses `@`-prefixed SysML v2 stereotype clauses, depth-bounded `part` / `part def` hierarchies, directional and conjugated `port` / `port def` blocks, typed `attribute` declarations, and `item` / `item def` payload definitions.
//! Bounds & Faults: Caps nested `part` recursion at `MAX_PARSE_DEPTH` (64), uses checked `.get()` lookahead with saturating index arithmetic, and fails closed with span-annotated `ParseError` values on invalid body tokens.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Inspects the current token without advancing the cursor and returns the stripped `@`-prefixed stereotype identifier if present.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK-STEREOTYPE-NAME]
    pub(crate) fn peek_stereotype_name(&self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(s) => {
                if let Some(stripped) = s.strip_prefix('@') {
                    if !stripped.is_empty() {
                        return Some(stripped.to_string());
                    }
                    if let Some(next_tok) = self.tokens.get(self.pos.saturating_add(1)) {
                        if let TokenKind::Ident(next_ident) = &next_tok.kind {
                            return Some(next_ident.clone());
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Consumes a call-style `@Anno("V")` or block-style `@Anno { k = "V"; }` stereotype annotation and returns `(name, values)`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-STEREOTYPE-ANNOTATION]
    pub(crate) fn parse_stereotype_annotation(
        &mut self,
    ) -> Result<(String, Vec<String>), ParseError> {
        let name = self.peek_stereotype_name().unwrap_or_default();
        if let TokenKind::Ident(s) = self.peek_kind().clone() {
            self.advance();
            if s == "@" && matches!(self.peek_kind(), TokenKind::Ident(_)) {
                self.advance();
            }
        }
        let mut values = Vec::new();
        self.skip_trivia();
        if self.check(&TokenKind::OpenParen) {
            self.advance();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                match self.peek_kind().clone() {
                    TokenKind::StringLit(v) | TokenKind::Ident(v) => {
                        values.push(v);
                        self.advance();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
            if self.check(&TokenKind::CloseParen) {
                self.advance();
            }
        } else if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                if self.check(&TokenKind::Eq) {
                    self.advance();
                    self.skip_trivia();
                    if let TokenKind::StringLit(v) | TokenKind::Ident(v) = self.peek_kind().clone()
                    {
                        values.push(v);
                        self.advance();
                    }
                } else if let TokenKind::StringLit(v) = self.peek_kind().clone() {
                    values.push(v);
                    self.advance();
                } else {
                    self.advance();
                }
            }
            if self.check(&TokenKind::CloseBrace) {
                self.advance();
            }
        }
        if self.check(&TokenKind::Semi) {
            self.advance();
        }
        Ok((name, values))
    }

    /// Returns `true` when the next token at `self.pos + 1` is `TokenKind::Port` (e.g. `in port`, `out port`, `inout port`).
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK-NEXT-IS-PORT]
    pub(crate) fn peek_next_is_port(&self) -> bool {
        self.tokens
            .get(self.pos.saturating_add(1))
            .is_some_and(|t| t.kind == TokenKind::Port)
    }

    /// Parses a `part` or `part def` declaration bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PART-DECL]
    pub(crate) fn parse_part_decl(&mut self, doc: Option<String>) -> Result<PartDef, ParseError> {
        if self.depth >= MAX_PARSE_DEPTH {
            return Err(self.max_depth_error());
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.parse_part_decl_inner(doc);
        self.depth = self.depth.saturating_sub(1);
        result
    }

    /// Parses the header and nested body of a `part` or `part def` declaration within an active depth frame.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PART-DECL]
    fn parse_part_decl_inner(&mut self, doc: Option<String>) -> Result<PartDef, ParseError> {
        self.expect(&TokenKind::Part)?;
        let is_def = if self.check(&TokenKind::Def) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.expect_ident()?;
        let mut type_name = None;

        if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
            self.advance();
            type_name = Some(self.parse_type_expr()?);
        }

        let mut part = Box::new(PartDef {
            name,
            doc,
            is_def,
            type_name,
            ..Default::default()
        });

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            self.skip_trivia();
            if part.doc.is_none() {
                part.doc = self.take_doc_comment();
            }
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                let mut dummy_pkg = Box::new(PackageDef::default());
                self.parse_container_item(&mut dummy_pkg, Some(&mut part))?;
                Self::merge_dummy_pkg_into_part(&mut part, *dummy_pkg);
                self.skip_trivia();
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(*part)
    }

    /// Merges any top-level declarations collected in `dummy_pkg` into `part`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PART-DECL]
    fn merge_dummy_pkg_into_part(part: &mut PartDef, dummy_pkg: PackageDef) {
        part.parts.extend(dummy_pkg.part_defs);
        part.ports.extend(dummy_pkg.port_defs);
        part.attributes.extend(dummy_pkg.attribute_defs);
        part.actions.extend(dummy_pkg.action_defs);
        part.constraints.extend(dummy_pkg.constraint_defs);
        part.requirements.extend(dummy_pkg.requirement_defs);
        part.states.extend(dummy_pkg.state_defs);
        part.connections.extend(dummy_pkg.connection_defs);
        part.use_cases.extend(dummy_pkg.use_case_defs);
        part.item_defs.extend(dummy_pkg.item_defs);
        part.hazards.extend(dummy_pkg.hazard_defs);
        part.risks.extend(dummy_pkg.risk_defs);
        part.test_cases.extend(dummy_pkg.test_case_defs);
        part.capabilities.extend(dummy_pkg.capability_defs);
        part.operations.extend(dummy_pkg.operation_defs);
        part.interactions.extend(dummy_pkg.interaction_defs);
        part.calcs.extend(dummy_pkg.calc_defs);
        for sat in dummy_pkg.satisfied_requirements {
            if !part.satisfied_requirements.contains(&sat) {
                part.satisfied_requirements.push(sat);
            }
        }
    }

    /// Parses a directional or conjugated `port` / `port def` declaration and its inner attributes and item flows into a `PortDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PORT-DECL]
    pub(crate) fn parse_port_decl(&mut self, doc: Option<String>) -> Result<PortDef, ParseError> {
        let mut direction = "inout".to_string();
        for (tok, dir) in [
            (TokenKind::In, "in"),
            (TokenKind::Out, "out"),
            (TokenKind::Inout, "inout"),
        ] {
            if self.check(&tok) {
                self.advance();
                direction = dir.to_string();
                break;
            }
        }
        self.expect(&TokenKind::Port)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        if direction == "inout" {
            for (tok, dir) in [
                (TokenKind::In, "in"),
                (TokenKind::Out, "out"),
                (TokenKind::Inout, "inout"),
            ] {
                if self.check(&tok) {
                    self.advance();
                    direction = dir.to_string();
                    break;
                }
            }
        }
        let name = self.expect_ident()?;
        let mut is_conjugated = false;
        let type_name = if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
            self.advance();
            if self.check(&TokenKind::Tilde) {
                self.advance();
                is_conjugated = true;
            }
            self.parse_type_expr()?
        } else {
            name.clone()
        };
        let mut port = PortDef {
            name,
            type_name,
            direction,
            is_conjugated,
            doc,
            ..Default::default()
        };
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                let mut f_doc = self.take_inner_doc_comment();
                self.skip_trivia();
                while let Some(next_doc) = self.take_inner_doc_comment() {
                    if port.doc.is_none() {
                        port.doc = f_doc;
                    }
                    f_doc = Some(next_doc);
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    if port.doc.is_none() {
                        port.doc = f_doc;
                    }
                    break;
                }
                if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(f_doc)?;
                    let clean_val = attr
                        .default_value
                        .unwrap_or_default()
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    if attr.name == "protocol_family" {
                        port.protocol_family = clean_val;
                    } else if attr.name == "port_category" {
                        port.port_category = clean_val;
                    } else {
                        port.electrical_attributes.insert(attr.name, clean_val);
                    }
                } else if self.check(&TokenKind::Flow)
                    || self.check(&TokenKind::Item)
                    || self.check(&TokenKind::In)
                    || self.check(&TokenKind::Out)
                    || self.check(&TokenKind::Inout)
                {
                    let mut flow_dir = "out".to_string();
                    if (self.check(&TokenKind::Flow) || self.check(&TokenKind::Item))
                        && !matches!(
                            self.tokens.get(self.pos.saturating_add(1)).map(|t| &t.kind),
                            Some(TokenKind::Colon | TokenKind::ColonGt | TokenKind::Semi)
                        )
                    {
                        self.advance();
                    }
                    for (tok, dir) in [
                        (TokenKind::In, "in"),
                        (TokenKind::Out, "out"),
                        (TokenKind::Inout, "inout"),
                    ] {
                        if self.check(&tok) {
                            self.advance();
                            flow_dir = dir.to_string();
                            break;
                        }
                    }
                    while (self.check(&TokenKind::Flow) || self.check(&TokenKind::Item))
                        && !matches!(
                            self.tokens.get(self.pos.saturating_add(1)).map(|t| &t.kind),
                            Some(TokenKind::Colon | TokenKind::ColonGt | TokenKind::Semi)
                        )
                    {
                        self.advance();
                    }
                    let flow_name = self.expect_ident()?;
                    let flow_type =
                        if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
                            self.advance();
                            self.parse_type_expr()?
                        } else {
                            "Item".to_string()
                        };
                    self.expect_semi()?;
                    port.item_flows.push(FlowDef {
                        name: flow_name,
                        direction: flow_dir,
                        item_type: flow_type,
                        doc: f_doc,
                        ..Default::default()
                    });
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!(
                            "Unexpected token '{:?}' in port declaration body",
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
        Ok(port)
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

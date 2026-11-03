#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Stereotype annotation, part, and port structural declaration parsers.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE] | [FLS: fls_66m4rnbssgig]
//! Contract: Parses `@`-prefixed SysML v2 stereotype clauses, depth-bounded `part` / `part def` hierarchies, and directional/conjugated `port` / `port def` blocks.
//! Bounds & Faults: Caps nested `part` recursion at `MAX_PARSE_DEPTH` (64), uses checked `.get()` lookahead with saturating index arithmetic, and fails closed with span-annotated `ParseError` values.

use super::cst_db::TokenKind;
use super::hir_parser_core::{ParseError, SysmlParser, MAX_PARSE_DEPTH};
use deap_core::sysml_cst::{FlowDef, PackageDef, PartDef, PortDef};

impl<'a> SysmlParser<'a> {
    /// Inspects the current token without advancing the cursor and returns the stripped `@`-prefixed stereotype identifier if present.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK-STEREOTYPE-NAME] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-STEREOTYPE-ANNOTATION] | [FLS: fls_66m4rnbssgig]
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
                    if let TokenKind::StringLit(v) | TokenKind::Ident(v) = self.peek_kind().clone() {
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PEEK-NEXT-IS-PORT] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn peek_next_is_port(&self) -> bool {
        self.tokens
            .get(self.pos.saturating_add(1))
            .is_some_and(|t| t.kind == TokenKind::Port)
    }

    /// Parses a `part` or `part def` declaration bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PART-DECL] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PART-DECL] | [FLS: fls_66m4rnbssgig]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PART-DECL] | [FLS: fls_66m4rnbssgig]
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

    /// Consumes a leading `in`, `out`, or `inout` direction keyword if present at the cursor.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PORT-DECL] | [FLS: fls_66m4rnbssgig]
    fn take_direction_keyword(&mut self) -> Option<&'static str> {
        for (tok, dir) in [
            (TokenKind::In, "in"),
            (TokenKind::Out, "out"),
            (TokenKind::Inout, "inout"),
        ] {
            if self.eat(&tok) {
                return Some(dir);
            }
        }
        None
    }

    /// Parses a directional or conjugated `port` / `port def` declaration and its inner attributes and item flows into a `PortDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PORT-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_port_decl(&mut self, doc: Option<String>) -> Result<PortDef, ParseError> {
        let mut direction = self.take_direction_keyword().unwrap_or("inout").to_string();
        self.expect(&TokenKind::Port)?;
        self.eat(&TokenKind::Def);
        if direction == "inout" {
            if let Some(dir) = self.take_direction_keyword() {
                direction = dir.to_string();
            }
        }
        let name = self.expect_ident()?;
        let mut is_conjugated = false;
        let type_name = if self.eat(&TokenKind::Colon) || self.eat(&TokenKind::ColonGt) {
            if self.eat(&TokenKind::Tilde) {
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
            self.parse_port_body(&mut port)?;
        } else {
            self.expect_semi()?;
        }
        Ok(port)
    }

    /// Parses the braced body of a `port` or `port def` declaration into `port`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PORT-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_port_body(&mut self, port: &mut PortDef) -> Result<(), ParseError> {
        self.expect(&TokenKind::OpenBrace)?;
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
            } else if matches!(
                self.peek_kind(),
                TokenKind::Flow | TokenKind::Item | TokenKind::In | TokenKind::Out | TokenKind::Inout
            ) {
                port.item_flows.push(self.parse_port_flow_member(f_doc)?);
            } else {
                let tok = self.peek();
                return Err(ParseError {
                    message: format!("Unexpected token '{:?}' in port declaration body", tok.kind),
                    span: tok.span,
                });
            }
        }
        self.expect(&TokenKind::CloseBrace)?;
        Ok(())
    }

    /// Parses a single `flow` / `item` member statement inside a `port` declaration body.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PORT-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_port_flow_member(&mut self, f_doc: Option<String>) -> Result<FlowDef, ParseError> {
        let next_is_sep = |s: &Self| {
            matches!(
                s.tokens.get(s.pos.saturating_add(1)).map(|t| &t.kind),
                Some(TokenKind::Colon | TokenKind::ColonGt | TokenKind::Semi)
            )
        };
        if (self.check(&TokenKind::Flow) || self.check(&TokenKind::Item)) && !next_is_sep(self) {
            self.advance();
        }
        let flow_dir = self.take_direction_keyword().unwrap_or("out").to_string();
        while (self.check(&TokenKind::Flow) || self.check(&TokenKind::Item)) && !next_is_sep(self) {
            self.advance();
        }
        let flow_name = self.expect_ident()?;
        let flow_type = if self.eat(&TokenKind::Colon) || self.eat(&TokenKind::ColonGt) {
            self.parse_type_expr()?
        } else {
            "Item".to_string()
        };
        self.expect_semi()?;
        Ok(FlowDef {
            name: flow_name,
            direction: flow_dir,
            item_type: flow_type,
            doc: f_doc,
            ..Default::default()
        })
    }
}

//! Recursive descent parser translating SysML v2 / KerML tokens into typed AST.

use crate::lexer::scanner::Scanner;
use crate::lexer::token::{Span, Token, TokenKind};
use deap_core::sysml_ast::*;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Diagnostic error with source location span.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, col {}: {}",
            self.span.line, self.span.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Recursive descent parser for SysML v2 / KerML syntax.
pub struct SysmlParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> SysmlParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse complete source text into a PackageDef.
    pub fn parse_source(source: &str, default_pkg_name: &str) -> Result<PackageDef, ParseError> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_all().map_err(|msg| ParseError {
            message: msg,
            span: Span::default(),
        })?;
        let mut parser = SysmlParser::new(&tokens);
        parser.parse_package_or_file(default_pkg_name)
    }

    /// Parse a package or top-level file definitions into PackageDef.
    pub fn parse_package_or_file(&mut self, default_pkg_name: &str) -> Result<PackageDef, ParseError> {
        self.skip_trivia();

        // Check if top-level starts with package declaration
        let mut pending_doc = self.take_doc_comment();
        if self.check(&TokenKind::Package) {
            let first_pkg = self.parse_package_decl(pending_doc)?;
            self.skip_trivia();
            if self.is_at_end() {
                return Ok(first_pkg);
            }

            // Multiple packages or items at root level
            let mut root = PackageDef {
                name: default_pkg_name.to_string(),
                packages: vec![first_pkg],
                ..Default::default()
            };

            while !self.is_at_end() {
                self.parse_container_item(&mut root, None)?;
                self.skip_trivia();
            }

            return Ok(root);
        }

        // Flat file without top-level package block
        let mut pkg = PackageDef {
            name: default_pkg_name.to_string(),
            doc: pending_doc.take(),
            ..Default::default()
        };

        while !self.is_at_end() {
            self.parse_container_item(&mut pkg, None)?;
            self.skip_trivia();
        }

        Ok(pkg)
    }

    fn parse_package_decl(&mut self, mut doc: Option<String>) -> Result<PackageDef, ParseError> {
        self.expect(&TokenKind::Package)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::OpenBrace)?;

        self.skip_trivia();
        if doc.is_none() {
            doc = self.take_doc_comment();
        }

        let mut pkg = PackageDef {
            name,
            doc,
            ..Default::default()
        };

        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            self.parse_container_item(&mut pkg, None)?;
            self.skip_trivia();
        }

        self.expect(&TokenKind::CloseBrace)?;
        Ok(pkg)
    }

    /// Parse an import declaration into an ImportDef AST node.
    ///
    /// Realises: [REQ-SYSML-PARSER-IMPORT]
    fn parse_import_decl(&mut self, doc: Option<String>) -> Result<ImportDef, ParseError> {
        self.expect(&TokenKind::Import)?;
        let mut path_segments = Vec::new();
        let mut is_wildcard = false;
        let mut is_recursive = false;

        let mut leading_colon = false;
        if self.check(&TokenKind::DoubleColon) {
            self.advance();
            leading_colon = true;
        }

        let first = self.expect_ident()?;
        path_segments.push(first);

        while self.check(&TokenKind::DoubleColon) {
            self.advance();
            if self.check(&TokenKind::Star) {
                self.advance();
                if self.check(&TokenKind::Star) {
                    self.advance();
                    is_recursive = true;
                    is_wildcard = true;
                } else {
                    is_wildcard = true;
                }
                break;
            } else {
                let seg = self.expect_ident()?;
                path_segments.push(seg);
            }
        }

        self.expect_semi()?;

        let mut path = path_segments.join("::");
        if leading_colon {
            path = format!("::{}", path);
        }

        Ok(ImportDef {
            path,
            is_wildcard,
            is_recursive,
            doc,
        })
    }

    fn parse_container_item(
        &mut self,
        pkg: &mut PackageDef,
        parent_part: Option<&mut PartDef>,
    ) -> Result<(), ParseError> {
        self.skip_trivia();
        let doc = self.take_doc_comment();

        if self.is_at_end() {
            return Ok(());
        }

        match self.peek_kind() {
            TokenKind::Package => {
                let subpkg = self.parse_package_decl(doc)?;
                pkg.packages.push(subpkg);
            }
            TokenKind::Part => {
                let part = self.parse_part_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.parts.push(part);
                } else {
                    pkg.part_defs.push(part);
                }
            }
            TokenKind::Port => {
                let port = self.parse_port_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.ports.push(port);
                } else {
                    pkg.port_defs.push(port);
                }
            }
            TokenKind::In | TokenKind::Out | TokenKind::Inout if self.peek_next_is_port() => {
                let port = self.parse_port_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.ports.push(port);
                } else {
                    pkg.port_defs.push(port);
                }
            }
            TokenKind::Attribute => {
                let attr = self.parse_attribute_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.attributes.push(attr);
                } else {
                    pkg.attribute_defs.push(attr);
                }
            }
            TokenKind::Action => {
                let action = self.parse_action_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.actions.push(action);
                } else {
                    pkg.action_defs.push(action);
                }
            }
            TokenKind::Constraint | TokenKind::Assert => {
                let constraint = self.parse_constraint_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.constraints.push(constraint);
                } else {
                    pkg.constraint_defs.push(constraint);
                }
            }
            TokenKind::Requirement => {
                let req = self.parse_requirement_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.requirements.push(req);
                } else {
                    pkg.requirement_defs.push(req);
                }
            }
            TokenKind::State => {
                let state = self.parse_state_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.states.push(state);
                } else {
                    pkg.state_defs.push(state);
                }
            }
            TokenKind::Import => {
                let imp = self.parse_import_decl(doc)?;
                pkg.imports.push(imp);
            }
            TokenKind::Connection => {
                let conn = self.parse_connection_decl(doc, false)?;
                if let Some(parent) = parent_part {
                    parent.connections.push(conn);
                } else {
                    pkg.connection_defs.push(conn);
                }
            }
            TokenKind::Connect => {
                let conn = self.parse_inline_connect_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.connections.push(conn);
                } else {
                    pkg.connection_defs.push(conn);
                }
            }
            TokenKind::Flow => {
                let conn = self.parse_connection_decl(doc, true)?;
                if let Some(parent) = parent_part {
                    parent.connections.push(conn);
                } else {
                    pkg.connection_defs.push(conn);
                }
            }
            TokenKind::UseCase => {
                let uc = self.parse_use_case_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.use_cases.push(uc);
                } else {
                    pkg.use_case_defs.push(uc);
                }
            }
            TokenKind::Item => {
                let item = self.parse_item_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.item_defs.push(item);
                } else {
                    pkg.item_defs.push(item);
                }
            }
            TokenKind::Hazard => {
                let hz = self.parse_hazard_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.hazards.push(hz);
                } else {
                    pkg.hazard_defs.push(hz);
                }
            }
            TokenKind::Risk => {
                let rk = self.parse_risk_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.risks.push(rk);
                } else {
                    pkg.risk_defs.push(rk);
                }
            }
            TokenKind::Test => {
                let tc = self.parse_test_case_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.test_cases.push(tc);
                } else {
                    pkg.test_case_defs.push(tc);
                }
            }
            TokenKind::Capability => {
                let cap = self.parse_capability_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.capabilities.push(cap);
                } else {
                    pkg.capability_defs.push(cap);
                }
            }
            TokenKind::Operation => {
                let op = self.parse_operation_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.operations.push(op);
                } else {
                    pkg.operation_defs.push(op);
                }
            }
            TokenKind::Interaction => {
                let inter = self.parse_interaction_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.interactions.push(inter);
                } else {
                    pkg.interaction_defs.push(inter);
                }
            }
            _ => {
                // Skip unknown statement or delimiter to maintain fault tolerance
                self.skip_until_semi_or_brace();
            }
        }

        Ok(())
    }

    fn peek_next_is_port(&self) -> bool {
        if self.pos + 1 < self.tokens.len() {
            self.tokens[self.pos + 1].kind == TokenKind::Port
        } else {
            false
        }
    }

    // --- Specific Parsers ---

    fn parse_part_decl(&mut self, doc: Option<String>) -> Result<PartDef, ParseError> {
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
            type_name = Some(self.expect_ident()?);
        }

        let mut part = PartDef {
            name,
            doc,
            is_def,
            type_name,
            ..Default::default()
        };

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            self.skip_trivia();
            if part.doc.is_none() {
                part.doc = self.take_doc_comment();
            }
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                let mut dummy_pkg = PackageDef::default();
                self.parse_container_item(&mut dummy_pkg, Some(&mut part))?;
                // Merge any parsed top-level items into part
                part.parts.extend(dummy_pkg.part_defs);
                part.ports.extend(dummy_pkg.port_defs);
                part.attributes.extend(dummy_pkg.attribute_defs);
                part.actions.extend(dummy_pkg.action_defs);
                part.constraints.extend(dummy_pkg.constraint_defs);
                part.requirements.extend(dummy_pkg.requirement_defs);
                part.states.extend(dummy_pkg.state_defs);
                part.connections.extend(dummy_pkg.connection_defs);
                self.skip_trivia();
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(part)
    }

    fn parse_port_decl(&mut self, doc: Option<String>) -> Result<PortDef, ParseError> {
        let mut direction = "inout".to_string();
        if self.check(&TokenKind::In) {
            self.advance();
            direction = "in".to_string();
        } else if self.check(&TokenKind::Out) {
            self.advance();
            direction = "out".to_string();
        } else if self.check(&TokenKind::Inout) {
            self.advance();
            direction = "inout".to_string();
        }

        self.expect(&TokenKind::Port)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        if direction == "inout" {
            if self.check(&TokenKind::In) {
                self.advance();
                direction = "in".to_string();
            } else if self.check(&TokenKind::Out) {
                self.advance();
                direction = "out".to_string();
            } else if self.check(&TokenKind::Inout) {
                self.advance();
                direction = "inout".to_string();
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
            self.expect_ident()?
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
                let f_doc = self.take_doc_comment();
                if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(f_doc)?;
                    if attr.name == "protocol_family" {
                        port.protocol_family = attr.default_value.unwrap_or_default();
                    } else if attr.name == "port_category" {
                        port.port_category = attr.default_value.unwrap_or_default();
                    } else {
                        port.electrical_attributes
                            .insert(attr.name, attr.default_value.unwrap_or_default());
                    }
                } else if self.check(&TokenKind::Flow) || self.check(&TokenKind::In) || self.check(&TokenKind::Out) {
                    let mut flow_dir = "out".to_string();
                    if self.check(&TokenKind::In) {
                        self.advance();
                        flow_dir = "in".to_string();
                    } else if self.check(&TokenKind::Out) {
                        self.advance();
                        flow_dir = "out".to_string();
                    }
                    if self.check(&TokenKind::Flow) {
                        self.advance();
                    }
                    let flow_name = self.expect_ident()?;
                    let flow_type = if self.check(&TokenKind::Colon) {
                        self.advance();
                        self.expect_ident()?
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
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(port)
    }

    fn parse_attribute_decl(&mut self, doc: Option<String>) -> Result<AttributeDef, ParseError> {
        self.expect(&TokenKind::Attribute)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut type_name = "String".to_string();
        let mut default_value = None;

        if self.check(&TokenKind::Colon) {
            self.advance();
            type_name = self.expect_ident()?;
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

    fn parse_action_decl(&mut self, doc: Option<String>) -> Result<ActionDef, ParseError> {
        self.expect(&TokenKind::Action)?;
        let is_def = if self.check(&TokenKind::Def) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.expect_ident()?;
        let mut in_params = Vec::new();
        let mut out_params = Vec::new();
        let mut performer = None;
        let mut steps = Vec::new();

        if self.check(&TokenKind::OpenParen) {
            self.advance();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                let mut is_out = false;
                if self.check(&TokenKind::In) {
                    self.advance();
                } else if self.check(&TokenKind::Out) {
                    self.advance();
                    is_out = true;
                }
                if self.check(&TokenKind::Attribute) || self.check(&TokenKind::Item) {
                    self.advance();
                }
                let param_name = self.expect_ident()?;
                let param_type = if self.check(&TokenKind::Colon) {
                    self.advance();
                    let mut type_name = self.expect_ident()?;
                    while self.check(&TokenKind::DoubleColon) {
                        self.advance();
                        type_name.push_str("::");
                        type_name.push_str(&self.expect_ident()?);
                    }
                    type_name
                } else {
                    "Real".to_string()
                };
                let param = AttributeDef {
                    name: param_name,
                    type_name: param_type,
                    default_value: None,
                    doc: None,
                };
                if is_out {
                    out_params.push(param);
                } else {
                    in_params.push(param);
                }
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::CloseParen)?;
        }

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                let _ = self.take_doc_comment();
                if self.check(&TokenKind::Perform) {
                    self.advance();
                    performer = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Step) {
                    self.advance();
                    steps.push(self.read_until_semi());
                } else if self.check(&TokenKind::In) || self.check(&TokenKind::Out) {
                    let is_out = self.check(&TokenKind::Out);
                    self.advance();
                    if self.check(&TokenKind::Attribute) || self.check(&TokenKind::Item) {
                        self.advance();
                    }
                    let p_name = self.expect_ident()?;
                    let p_type = if self.check(&TokenKind::Colon) {
                        self.advance();
                        let mut type_name = self.expect_ident()?;
                        while self.check(&TokenKind::DoubleColon) {
                            self.advance();
                            type_name.push_str("::");
                            type_name.push_str(&self.expect_ident()?);
                        }
                        type_name
                    } else {
                        "Real".to_string()
                    };
                    self.expect_semi()?;
                    let p = AttributeDef {
                        name: p_name,
                        type_name: p_type,
                        default_value: None,
                        doc: None,
                    };
                    if is_out {
                        out_params.push(p);
                    } else {
                        in_params.push(p);
                    }
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(ActionDef {
            name,
            doc,
            in_params,
            out_params,
            parameters: Vec::new(),
            performer,
            steps,
            is_def,
        })
    }

    fn parse_constraint_decl(&mut self, doc: Option<String>) -> Result<ConstraintDef, ParseError> {
        let is_assertion = if self.check(&TokenKind::Assert) {
            self.advance();
            true
        } else {
            false
        };

        if self.check(&TokenKind::Constraint) {
            self.advance();
        } else if !is_assertion {
            self.expect(&TokenKind::Constraint)?;
        }

        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut parameters = Vec::new();

        if self.check(&TokenKind::OpenParen) {
            self.advance();
            let mut param_str = String::new();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                let tok = self.advance();
                match &tok.kind {
                    TokenKind::Comma => {
                        if !param_str.trim().is_empty() {
                            parameters.push(param_str.trim().to_string());
                            param_str.clear();
                        }
                    }
                    TokenKind::Ident(s) => {
                        param_str.push_str(s);
                        param_str.push(' ');
                    }
                    TokenKind::Colon => {
                        param_str.push_str(": ");
                    }
                    _ => {
                        param_str.push_str(tok.kind.as_str());
                        param_str.push(' ');
                    }
                }
            }
            if !param_str.trim().is_empty() {
                parameters.push(param_str.trim().to_string());
            }
            self.expect(&TokenKind::CloseParen)?;
        }

        let mut expression = String::new();
        let mut final_doc = doc;
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Doc) {
                    self.advance();
                    self.skip_trivia();
                    if let Some(d) = self.take_doc_comment() {
                        if final_doc.is_none() {
                            final_doc = Some(d);
                        }
                    }
                    continue;
                }
                if let Some(d) = self.take_doc_comment() {
                    if final_doc.is_none() {
                        final_doc = Some(d);
                    }
                    continue;
                }
                let stmt = self.read_until_semi();
                if !stmt.trim().is_empty() {
                    if !expression.is_empty() {
                        expression.push_str(" && ");
                    }
                    expression.push_str(&stmt);
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(ConstraintDef {
            name,
            expression,
            parameters,
            is_assertion,
            doc: final_doc,
            pre_conditions: Vec::new(),
            post_conditions: Vec::new(),
        })
    }

    fn parse_requirement_decl(&mut self, doc: Option<String>) -> Result<RequirementDef, ParseError> {
        self.expect(&TokenKind::Requirement)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut req_id = String::new();
        let mut text = String::new();
        let mut assumes = Vec::new();
        let mut requires = Vec::new();
        let mut derived_from = Vec::new();
        let mut verified_by = Vec::new();
        let mut satisfied_by = Vec::new();
        let mut attributes = Vec::new();
        let mut constraints = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                let sub_doc = self.take_doc_comment();
                if self.check(&TokenKind::Id) {
                    self.advance();
                    if self.check(&TokenKind::Eq) {
                        self.advance();
                    }
                    req_id = self.expect_string_or_ident()?;
                    self.expect_semi()?;
                } else if self.check(&TokenKind::Text) || self.check(&TokenKind::Doc) {
                    self.advance();
                    if self.check(&TokenKind::Eq) {
                        self.advance();
                    }
                    text = self.expect_string_or_ident()?;
                    self.expect_semi()?;
                } else if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(sub_doc)?;
                    attributes.push(attr);
                } else if self.check(&TokenKind::Constraint) || self.check(&TokenKind::Assert) {
                    let con = self.parse_constraint_decl(sub_doc)?;
                    constraints.push(con);
                } else if let Some(d) = sub_doc {
                    if text.is_empty() {
                        text = d;
                    }
                } else if self.check(&TokenKind::Assume) {
                    self.advance();
                    assumes.push(self.read_until_semi());
                } else if self.check(&TokenKind::Require) {
                    self.advance();
                    requires.push(self.read_until_semi());
                } else if match self.peek_kind() {
                    TokenKind::Ident(s) => s == "derived" || s == "derive",
                    _ => false,
                } {
                    self.advance();
                    if self.check(&TokenKind::Requirement) {
                        self.advance();
                    }
                    if self.check(&TokenKind::From) {
                        self.advance();
                    }
                    derived_from.push(self.read_until_semi());
                } else if self.check(&TokenKind::Verify) {
                    self.advance();
                    if self.check(&TokenKind::By) {
                        self.advance();
                    }
                    verified_by.push(self.read_until_semi());
                } else if self.check(&TokenKind::Satisfy) {
                    self.advance();
                    if self.check(&TokenKind::By) {
                        self.advance();
                    }
                    satisfied_by.push(self.read_until_semi());
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(RequirementDef {
            name,
            req_id,
            text,
            doc,
            attributes,
            constraints,
            assumes,
            requires,
            verified_by,
            satisfied_by,
            derived_from,
            derives: Vec::new(),
        })
    }

    fn parse_state_decl(&mut self, doc: Option<String>) -> Result<StateDef, ParseError> {
        self.expect(&TokenKind::State)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut entry_action = None;
        let mut do_action = None;
        let mut exit_action = None;
        let mut transitions = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Entry) {
                    self.advance();
                    entry_action = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Do) {
                    self.advance();
                    do_action = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Exit) {
                    self.advance();
                    exit_action = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Transition) {
                    self.advance();
                    let t_stmt = self.read_until_semi();
                    transitions.push(TransitionDef {
                        name: Some(t_stmt.clone()),
                        source: None,
                        target: None,
                        trigger: None,
                        guard: None,
                        effect: None,
                        doc: None,
                    });
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(StateDef {
            name,
            doc,
            entry_action,
            do_action,
            exit_action,
            transitions,
            sub_states: Vec::new(),
        })
    }

    fn parse_connection_decl(
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
        let mut source_port = String::new();
        let mut target_port = String::new();
        let mut protocol = None;
        let mut latency_ms = None;
        let mut severity = 1;

        let mut conn_doc = doc;
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if let Some(inner_doc) = self.take_doc_comment() {
                    if conn_doc.is_none() {
                        conn_doc = Some(inner_doc);
                    }
                    continue;
                }
                if self.check(&TokenKind::Connect) || (self.check(&TokenKind::Flow) && self.peek_next_is_from()) {
                    self.advance(); // connect / flow
                    if self.check(&TokenKind::From) {
                        self.advance();
                    }
                    source_port = self.expect_ident_or_dotted()?;
                    self.expect(&TokenKind::To)?;
                    target_port = self.expect_ident_or_dotted()?;
                    self.expect_semi()?;
                } else if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(None)?;
                    if attr.name == "protocol" {
                        protocol = attr.default_value.map(|v| v.trim_matches('"').to_string());
                    } else if attr.name == "latency_ms" || attr.name == "latency" {
                        latency_ms = attr.default_value.and_then(|v| v.parse::<f64>().ok());
                    } else if attr.name == "severity" {
                        severity = attr.default_value.and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);
                    }
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        let source_part = if source_port.contains('.') {
            Some(source_port.split('.').next().unwrap().to_string())
        } else {
            None
        };
        let target_part = if target_port.contains('.') {
            Some(target_port.split('.').next().unwrap().to_string())
        } else {
            None
        };

        Ok(ConnectionDef {
            name,
            source_port,
            target_port,
            source_part,
            target_part,
            doc: conn_doc,
            severity,
            item_flow_ref: None,
            protocol,
            latency_ms,
            is_flow,
        })
    }

    /// Parse an inline connector statement into a ConnectionDef AST node.
    ///
    /// Realises: [REQ-SYSML-PARSER-INLINE-CONNECTOR]
    fn parse_inline_connect_decl(&mut self, doc: Option<String>) -> Result<ConnectionDef, ParseError> {
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
                    message: format!("Expected 'to' keyword in connect statement, found '{:?}'", tok.kind),
                    span: tok.span,
                });
            }
        } else {
            let tok = self.peek();
            return Err(ParseError {
                message: format!("Expected 'to' keyword in connect statement, found '{:?}'", tok.kind),
                span: tok.span,
            });
        }

        let target_port = self.expect_ident_or_dotted()?;
        self.expect_semi()?;

        let source_part = if source_port.contains('.') {
            Some(source_port.split('.').next().unwrap().to_string())
        } else {
            None
        };
        let target_part = if target_port.contains('.') {
            Some(target_port.split('.').next().unwrap().to_string())
        } else {
            None
        };

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

    fn peek_next_is_from(&self) -> bool {
        if self.pos + 1 < self.tokens.len() {
            self.tokens[self.pos + 1].kind == TokenKind::From
        } else {
            false
        }
    }

    fn parse_use_case_decl(&mut self, doc: Option<String>) -> Result<UseCaseDef, ParseError> {
        self.expect(&TokenKind::UseCase)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut subject = None;
        let mut actor = None;
        let mut actors = Vec::new();
        let mut objective = None;
        let mut steps = Vec::new();
        let mut includes = Vec::new();
        let mut extends = Vec::new();
        let mut preconditions = Vec::new();
        let mut postconditions = Vec::new();
        let mut uc_doc = doc;

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Doc) {
                    self.advance();
                    self.skip_trivia();
                    if self.check(&TokenKind::Eq) || self.check(&TokenKind::Colon) {
                        self.advance();
                        self.skip_trivia();
                    }
                    let inner = if let Some(d) = self.take_doc_comment() {
                        Some(d)
                    } else if let TokenKind::StringLit(s) = self.peek_kind().clone() {
                        self.advance();
                        Some(s)
                    } else if let TokenKind::Ident(s) = self.peek_kind().clone() {
                        self.advance();
                        Some(s)
                    } else {
                        None
                    };
                    if let Some(inner_text) = inner {
                        if !inner_text.trim().is_empty() {
                            match &mut uc_doc {
                                Some(existing) => {
                                    if !existing.trim().is_empty() {
                                        existing.push_str("\n\n");
                                    }
                                    existing.push_str(&inner_text);
                                }
                                None => {
                                    uc_doc = Some(inner_text);
                                }
                            }
                        }
                    }
                    self.skip_trivia();
                    continue;
                } else if let Some(inner_text) = self.take_doc_comment() {
                    if !inner_text.trim().is_empty() {
                        match &mut uc_doc {
                            Some(existing) => {
                                if !existing.trim().is_empty() {
                                    existing.push_str("\n\n");
                                }
                                existing.push_str(&inner_text);
                            }
                            None => {
                                uc_doc = Some(inner_text);
                            }
                        }
                    }
                    self.skip_trivia();
                    continue;
                } else if self.check(&TokenKind::Subject) {
                    self.advance();
                    subject = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Actor) {
                    self.advance();
                    let act = self.read_until_semi();
                    if actor.is_none() {
                        actor = Some(act.clone());
                    }
                    actors.push(act);
                } else if self.check(&TokenKind::Objective) {
                    self.advance();
                    objective = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Step) {
                    self.advance();
                    steps.push(self.read_until_semi());
                } else if self.check(&TokenKind::Include) {
                    self.advance();
                    includes.push(self.read_until_semi());
                } else if self.check(&TokenKind::Extend) {
                    self.advance();
                    extends.push(self.read_until_semi());
                } else if self.check(&TokenKind::Precondition) {
                    self.advance();
                    preconditions.push(self.read_until_semi());
                } else if self.check(&TokenKind::Postcondition) {
                    self.advance();
                    postconditions.push(self.read_until_semi());
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(UseCaseDef {
            name,
            doc: uc_doc,
            subject,
            actor,
            actors,
            objective,
            includes,
            extends,
            steps,
            preconditions,
            postconditions,
        })
    }

    fn parse_item_decl(&mut self, doc: Option<String>) -> Result<ItemDef, ParseError> {
        self.expect(&TokenKind::Item)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut attributes = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                let a_doc = self.take_doc_comment();
                if self.check(&TokenKind::Attribute) {
                    attributes.push(self.parse_attribute_decl(a_doc)?);
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(ItemDef {
            name,
            doc,
            attributes,
        })
    }

    fn parse_hazard_decl(&mut self, doc: Option<String>) -> Result<HazardDef, ParseError> {
        self.expect(&TokenKind::Hazard)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut severity = 1;
        let mut source_port = None;
        let mut target_port = None;
        let mut part_ref = None;

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(None)?;
                    if attr.name == "severity" {
                        severity = attr.default_value.and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);
                    } else if attr.name == "source_port" {
                        source_port = attr.default_value;
                    } else if attr.name == "target_port" {
                        target_port = attr.default_value;
                    } else if attr.name == "part_ref" {
                        part_ref = attr.default_value;
                    }
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(HazardDef {
            name,
            doc,
            severity,
            source_port,
            target_port,
            part_ref,
        })
    }

    fn parse_risk_decl(&mut self, doc: Option<String>) -> Result<RiskDef, ParseError> {
        self.expect(&TokenKind::Risk)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut severity = 1;
        let mut hazard_ref = None;
        let mut source_port = None;
        let mut target_port = None;

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(None)?;
                    if attr.name == "severity" {
                        severity = attr.default_value.and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);
                    } else if attr.name == "hazard_ref" {
                        hazard_ref = attr.default_value;
                    } else if attr.name == "source_port" {
                        source_port = attr.default_value;
                    } else if attr.name == "target_port" {
                        target_port = attr.default_value;
                    }
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(RiskDef {
            name,
            doc,
            severity,
            hazard_ref,
            source_port,
            target_port,
        })
    }

    fn parse_test_case_decl(&mut self, doc: Option<String>) -> Result<TestCaseDef, ParseError> {
        self.expect(&TokenKind::Test)?;
        if self.check(&TokenKind::Case) {
            self.advance();
        }
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut subject_part = None;
        let mut verified_requirements = Vec::new();
        let mut objective = None;
        let mut test_steps = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Subject) {
                    self.advance();
                    subject_part = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Verify) {
                    self.advance();
                    if self.check(&TokenKind::Requirement) {
                        self.advance();
                    }
                    verified_requirements.push(self.read_until_semi());
                } else if self.check(&TokenKind::Objective) {
                    self.advance();
                    objective = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Step) {
                    self.advance();
                    test_steps.push(self.read_until_semi());
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(TestCaseDef {
            name,
            subject_part,
            verified_requirements,
            objective,
            test_steps,
            doc,
        })
    }

    fn parse_capability_decl(&mut self, doc: Option<String>) -> Result<CapabilityDef, ParseError> {
        self.expect(&TokenKind::Capability)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut subsystem = None;

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Subsystem) {
                    self.advance();
                    subsystem = Some(self.read_until_semi());
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(CapabilityDef {
            name,
            description: doc.clone(),
            subsystem,
            package_ref: None,
            doc,
        })
    }

    fn parse_operation_decl(&mut self, doc: Option<String>) -> Result<OperationDef, ParseError> {
        self.expect(&TokenKind::Operation)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut return_type = None;
        let mut parameters = Vec::new();

        if self.check(&TokenKind::OpenParen) {
            self.advance();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                let mut dir = "in".to_string();
                if self.check(&TokenKind::In) {
                    self.advance();
                } else if self.check(&TokenKind::Out) {
                    self.advance();
                    dir = "out".to_string();
                } else if self.check(&TokenKind::Inout) {
                    self.advance();
                    dir = "inout".to_string();
                }
                let p_name = self.expect_ident()?;
                let p_type = if self.check(&TokenKind::Colon) {
                    self.advance();
                    self.expect_ident()?
                } else {
                    "String".to_string()
                };
                parameters.push(AttributeDef {
                    name: p_name,
                    type_name: p_type,
                    default_value: Some(dir),
                    doc: None,
                });
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::CloseParen)?;
        }

        if self.check(&TokenKind::Colon) {
            self.advance();
            return_type = Some(self.expect_ident()?);
        }

        self.expect_semi()?;

        Ok(OperationDef {
            name,
            direction: "inout".to_string(),
            param_type: "String".to_string(),
            return_type,
            doc,
            parameters,
        })
    }

    fn parse_interaction_decl(&mut self, doc: Option<String>) -> Result<InteractionDef, ParseError> {
        self.expect(&TokenKind::Interaction)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut lifelines = Vec::new();
        let mut messages = Vec::new();
        let mut triggers = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                if self.check(&TokenKind::Lifeline) {
                    self.advance();
                    lifelines.push(self.read_until_semi());
                } else if self.check(&TokenKind::Message) {
                    self.advance();
                    messages.push(self.read_until_semi());
                } else if self.check(&TokenKind::Trigger) {
                    self.advance();
                    triggers.push(self.read_until_semi());
                } else {
                    self.skip_until_semi_or_brace();
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(InteractionDef {
            name,
            lifelines,
            messages,
            triggers,
            doc,
        })
    }

    // --- Helper Methods ---

    fn skip_trivia(&mut self) {
        while !self.is_at_end() {
            match self.peek_kind() {
                TokenKind::LineComment(_) => {
                    self.advance();
                }
                TokenKind::Semi => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn take_doc_comment(&mut self) -> Option<String> {
        if let TokenKind::DocComment(text) = self.peek_kind().clone() {
            self.advance();
            Some(text)
        } else {
            None
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.tokens[self.pos].kind) == std::mem::discriminant(kind)
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        if self.is_at_end() {
            &TokenKind::Eof
        } else {
            &self.tokens[self.pos].kind
        }
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<&Token, ParseError> {
        if self.check(expected) {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(ParseError {
                message: format!("Expected '{:?}', found '{:?}'", expected, tok.kind),
                span: tok.span,
            })
        }
    }

    /// Parse an identifier or contextual keyword accepted in identifier positions.
    ///
    /// Realises: [REQ-SYSML-CONTEXTUAL-IDENTIFIERS]
    pub fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let tok = self.peek();
        match &tok.kind {
            TokenKind::Ident(s) => {
                let name = s.clone();
                self.advance();
                Ok(name)
            }
            // Contextual keywords accepted as identifiers in KerML / SysML v2 syntax
            TokenKind::Text
            | TokenKind::Doc
            | TokenKind::Item
            | TokenKind::State
            | TokenKind::Action
            | TokenKind::Port
            | TokenKind::Requirement
            | TokenKind::Constraint
            | TokenKind::Assert
            | TokenKind::Assume
            | TokenKind::Require
            | TokenKind::Verify
            | TokenKind::Satisfy
            | TokenKind::Calc
            | TokenKind::Test
            | TokenKind::Case
            | TokenKind::Hazard
            | TokenKind::Risk
            | TokenKind::Objective
            | TokenKind::Capability
            | TokenKind::Interaction
            | TokenKind::Lifeline
            | TokenKind::Message
            | TokenKind::Connection
            | TokenKind::Connect
            | TokenKind::From
            | TokenKind::To
            | TokenKind::By
            | TokenKind::Attribute
            | TokenKind::Id
            | TokenKind::In
            | TokenKind::Out
            | TokenKind::Inout
            | TokenKind::Operation
            | TokenKind::Transition
            | TokenKind::Entry
            | TokenKind::Exit
            | TokenKind::Perform
            | TokenKind::Step
            | TokenKind::Trigger
            | TokenKind::Include
            | TokenKind::Extend
            | TokenKind::Precondition
            | TokenKind::Postcondition
            | TokenKind::Subsystem
            | TokenKind::Subject
            | TokenKind::Actor
            | TokenKind::Flow
            | TokenKind::Interface
            | TokenKind::Package
            | TokenKind::Part
            | TokenKind::Def
            | TokenKind::Do => {
                let name = tok.kind.as_str().to_string();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError {
                message: format!("Expected identifier, found '{:?}'", tok.kind),
                span: tok.span,
            }),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        self.parse_identifier()
    }

    fn expect_ident_or_dotted(&mut self) -> Result<String, ParseError> {
        let mut result = self.expect_ident()?;
        while self.check(&TokenKind::Dot) {
            self.advance();
            let next_part = self.expect_ident()?;
            result.push('.');
            result.push_str(&next_part);
        }
        Ok(result)
    }

    fn expect_string_or_ident(&mut self) -> Result<String, ParseError> {
        let tok = self.peek();
        if let TokenKind::StringLit(s) = &tok.kind {
            let val = s.clone();
            self.advance();
            return Ok(val);
        }
        self.parse_identifier().map_err(|_| {
            let tok = self.peek();
            ParseError {
                message: format!("Expected string or identifier, found '{:?}'", tok.kind),
                span: tok.span,
            }
        })
    }

    fn expect_semi(&mut self) -> Result<(), ParseError> {
        if self.check(&TokenKind::Semi) {
            self.advance();
            Ok(())
        } else {
            let tok = self.peek();
            Err(ParseError {
                message: format!("Expected ';' statement terminator, found '{:?}'", tok.kind),
                span: tok.span,
            })
        }
    }

    fn read_until_semi(&mut self) -> String {
        let mut pieces = Vec::new();
        while !self.check(&TokenKind::Semi) && !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let tok = self.advance();
            match &tok.kind {
                TokenKind::Ident(s) => pieces.push(s.clone()),
                TokenKind::StringLit(s) => pieces.push(format!("\"{}\"", s)),
                TokenKind::IntLit(n) => pieces.push(n.to_string()),
                TokenKind::RealLit(f) => {
                    if f.fract() == 0.0 {
                        pieces.push(format!("{:.1}", f));
                    } else {
                        pieces.push(f.to_string());
                    }
                }
                TokenKind::BoolLit(b) => pieces.push(b.to_string()),
                _ => pieces.push(tok.kind.as_str().to_string()),
            }
        }
        if self.check(&TokenKind::Semi) {
            self.advance();
        }
        pieces.join(" ")
    }

    fn skip_until_semi_or_brace(&mut self) {
        let mut depth = 0;
        while !self.is_at_end() {
            match self.peek_kind() {
                TokenKind::OpenBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::CloseBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    self.advance();
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Semi => {
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

/// Discover all .sysml files recursively under `dir`, filtering out hidden and temporary files.
///
/// Returns file paths sorted lexicographically in deterministic order.
pub fn discover_sysml_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !file_name.starts_with('.') && !file_name.starts_with('#')
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "sysml") {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files
}

/// Merge all AST definitions from `src` into `dest`.
///
/// Merges all element definitions (`imports`, `part_defs`, `port_defs`, `attribute_defs`,
/// `action_defs`, `operation_defs`, `capability_defs`, `interaction_defs`, `constraint_defs`,
/// `test_case_defs`, `requirement_defs`, `state_defs`, `use_case_defs`, `item_defs`, `hazard_defs`,
/// `risk_defs`, `connection_defs`).
/// For nested `packages`, if a child package with the same name already exists in
/// `dest.packages`, it is recursively merged; otherwise, it is appended.
pub fn merge_package_defs(dest: &mut PackageDef, src: PackageDef) {
    if dest.doc.is_none() && src.doc.is_some() {
        dest.doc = src.doc;
    }
    if dest.parent_package.is_none() && src.parent_package.is_some() {
        dest.parent_package = src.parent_package;
    }

    for imp in src.imports {
        if !dest.imports.iter().any(|existing| {
            existing.path == imp.path
                && existing.is_wildcard == imp.is_wildcard
                && existing.is_recursive == imp.is_recursive
        }) {
            dest.imports.push(imp);
        }
    }

    dest.part_defs.extend(src.part_defs);
    dest.port_defs.extend(src.port_defs);
    dest.attribute_defs.extend(src.attribute_defs);
    dest.action_defs.extend(src.action_defs);
    dest.operation_defs.extend(src.operation_defs);
    dest.capability_defs.extend(src.capability_defs);
    dest.interaction_defs.extend(src.interaction_defs);
    dest.constraint_defs.extend(src.constraint_defs);
    dest.test_case_defs.extend(src.test_case_defs);
    dest.requirement_defs.extend(src.requirement_defs);
    dest.state_defs.extend(src.state_defs);
    dest.use_case_defs.extend(src.use_case_defs);
    dest.item_defs.extend(src.item_defs);
    dest.hazard_defs.extend(src.hazard_defs);
    dest.risk_defs.extend(src.risk_defs);
    dest.connection_defs.extend(src.connection_defs);

    for src_child in src.packages {
        if let Some(existing) = dest.packages.iter_mut().find(|p| p.name == src_child.name) {
            merge_package_defs(existing, src_child);
        } else {
            dest.packages.push(src_child);
        }
    }
}

/// Parse multiple SysML files and merge their package definitions into a unified root `PackageDef`.
pub fn parse_sysml_tree(files: &[PathBuf]) -> Result<PackageDef, ParseError> {
    if files.is_empty() {
        return Err(ParseError {
            message: "No .sysml files provided for tree parsing".to_string(),
            span: Span::default(),
        });
    }

    let mut root = PackageDef {
        name: "SysML_Model".to_string(),
        ..Default::default()
    };

    for file in files {
        let content = fs::read_to_string(file).map_err(|e| ParseError {
            message: format!("Failed to read SysML file '{}': {}", file.display(), e),
            span: Span::default(),
        })?;

        let parsed = SysmlParser::parse_source(&content, "SysML_Model")?;

        if parsed.name == "SysML_Model" || parsed.name == root.name {
            merge_package_defs(&mut root, parsed);
        } else if let Some(existing) = root.packages.iter_mut().find(|p| p.name == parsed.name) {
            merge_package_defs(existing, parsed);
        } else {
            root.packages.push(parsed);
        }
    }

    Ok(root)
}

/// Discover all SysML files recursively in `dir` and parse them into a unified root `PackageDef`.
pub fn parse_sysml_directory(dir: &Path) -> Result<PackageDef, ParseError> {
    let files = discover_sysml_files(dir);
    if files.is_empty() {
        return Err(ParseError {
            message: format!("No .sysml files found in directory '{}'", dir.display()),
            span: Span::default(),
        });
    }
    parse_sysml_tree(&files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ground_truth_model_fixture() {
        let fixture = include_str!("../../../../tests/fixtures/safety/ground_truth_model.sysml");
        let pkg = SysmlParser::parse_source(fixture, "TestModel").unwrap();

        assert_eq!(pkg.name, "AutonomousVehicle_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("SSOT for Autonomous System Architecture and Safety Model")
        );
        assert_eq!(pkg.attribute_defs.len(), 4);
        assert_eq!(pkg.attribute_defs[0].name, "ruddervatorCount");
        assert_eq!(pkg.attribute_defs[0].type_name, "Integer");
        assert_eq!(pkg.attribute_defs[0].default_value.as_deref(), Some("4"));

        assert_eq!(pkg.part_defs.len(), 2);
        let airframe = &pkg.part_defs[0];
        assert_eq!(airframe.name, "Airframe");
        assert_eq!(airframe.attributes.len(), 1);
        assert_eq!(airframe.attributes[0].name, "massKg");
        assert_eq!(airframe.attributes[0].default_value.as_deref(), Some("25.0"));

        let fcc = &pkg.part_defs[1];
        assert_eq!(fcc.name, "FlightControlComputer");
        assert_eq!(fcc.ports.len(), 2);
        assert_eq!(fcc.ports[0].name, "c2_bus");
        assert_eq!(fcc.ports[0].type_name, "RS485");
        assert_eq!(fcc.ports[1].name, "telemetry");
        assert_eq!(fcc.ports[1].type_name, "MAVLink");
    }

    #[test]
    fn test_parse_grounded_limits_model_fixture() {
        let fixture = include_str!("../../../../tests/fixtures/safety/grounded_limits_model.sysml");
        let pkg = SysmlParser::parse_source(fixture, "TestLimits").unwrap();

        assert_eq!(pkg.name, "GroundedLimits_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("Ground truth limits for flight envelope verification")
        );
        assert_eq!(pkg.attribute_defs.len(), 5);
        assert_eq!(pkg.attribute_defs[0].name, "catapultPressureLimitBar");
        assert_eq!(pkg.attribute_defs[0].default_value.as_deref(), Some("15.0"));
    }

    #[test]
    fn test_parse_complex_system_model() {
        let source = r#"
package FlightControl_SSOT {
    doc /* Primary Flight Control and Mission Computer Architecture */

    attribute maxRollRateDps : Real = 45.0;
    attribute maxPitchAngleDeg : Real = 30.0;

    part def NavigationSubsystem {
        doc /* GPS, IMU and Sensor Fusion Unit */
        port out nav_solution : NavData;
        port in correction_stream : ~NavCorrection;

        action def ComputeNavigationState {
            in rawImu : ImuPacket;
            out filteredState : NavState;
        }

        state def NavFilterState {
            entry InitINS;
            do RunEKF;
            exit ResetCovariance;
        }
    }

    part def ActuatorController {
        port in servo_cmd : ~ServoCommand;
        port out status : ActuatorStatus;

        assert constraint DeflectionBounds {
            deflectionDeg >= -25.0 && deflectionDeg <= 25.0;
        }
    }

    connection def NavToActuatorBus {
        doc /* High speed deterministic bus connection */
        connect NavigationSubsystem.nav_solution to ActuatorController.servo_cmd;
        attribute protocol : String = "CANopen";
        attribute latency_ms : Real = 2.5;
    }

    requirement def Req_Nav_001 {
        id = "REQ-NAV-001";
        text = "Navigation system shall compute attitude at minimum 100 Hz.";
        require latency <= 10.0;
        verify by HardwareInLoopTest;
        satisfy by NavigationSubsystem;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.name, "FlightControl_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("Primary Flight Control and Mission Computer Architecture")
        );
        assert_eq!(pkg.attribute_defs.len(), 2);
        assert_eq!(pkg.part_defs.len(), 2);

        let nav = &pkg.part_defs[0];
        assert_eq!(nav.name, "NavigationSubsystem");
        assert_eq!(nav.doc.as_deref(), Some("GPS, IMU and Sensor Fusion Unit"));
        assert_eq!(nav.ports.len(), 2);
        assert_eq!(nav.ports[0].direction, "out");
        assert_eq!(nav.ports[0].name, "nav_solution");
        assert_eq!(nav.ports[1].direction, "in");
        assert_eq!(nav.ports[1].is_conjugated, true);
        assert_eq!(nav.ports[1].name, "correction_stream");
        assert_eq!(nav.actions.len(), 1);
        assert_eq!(nav.actions[0].name, "ComputeNavigationState");
        assert_eq!(nav.states.len(), 1);
        assert_eq!(nav.states[0].name, "NavFilterState");
        assert_eq!(nav.states[0].entry_action.as_deref(), Some("InitINS"));
        assert_eq!(nav.states[0].do_action.as_deref(), Some("RunEKF"));

        let actuator = &pkg.part_defs[1];
        assert_eq!(actuator.name, "ActuatorController");
        assert_eq!(actuator.constraints.len(), 1);
        assert_eq!(actuator.constraints[0].name, "DeflectionBounds");
        assert_eq!(actuator.constraints[0].is_assertion, true);
        assert!(actuator.constraints[0].expression.contains("deflectionDeg"));

        assert_eq!(pkg.connection_defs.len(), 1);
        let conn = &pkg.connection_defs[0];
        assert_eq!(conn.name, "NavToActuatorBus");
        assert_eq!(conn.source_port, "NavigationSubsystem.nav_solution");
        assert_eq!(conn.target_port, "ActuatorController.servo_cmd");
        assert_eq!(conn.source_part.as_deref(), Some("NavigationSubsystem"));
        assert_eq!(conn.target_part.as_deref(), Some("ActuatorController"));
        assert_eq!(conn.protocol.as_deref(), Some("CANopen"));
        assert_eq!(conn.latency_ms, Some(2.5));

        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = &pkg.requirement_defs[0];
        assert_eq!(req.name, "Req_Nav_001");
        assert_eq!(req.req_id, "REQ-NAV-001");
        assert_eq!(
            req.text,
            "Navigation system shall compute attitude at minimum 100 Hz."
        );
        assert_eq!(req.verified_by, vec!["HardwareInLoopTest"]);
        assert_eq!(req.satisfied_by, vec!["NavigationSubsystem"]);
    }

    #[test]
    fn test_parse_contextual_keywords_as_identifiers() {
        let source = r#"
package TestPkg {
    part def Component {
        attribute text : String = "hello";
        attribute doc : String = "world";
        attribute item : String = "val";
        attribute state : String = "ready";
        attribute action : String = "run";
        attribute port : String = "port1";
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.name, "TestPkg");
        let comp = &pkg.part_defs[0];
        assert_eq!(comp.name, "Component");
        assert_eq!(comp.attributes.len(), 6);
        assert_eq!(comp.attributes[0].name, "text");
        assert_eq!(comp.attributes[1].name, "doc");
        assert_eq!(comp.attributes[2].name, "item");
        assert_eq!(comp.attributes[3].name, "state");
        assert_eq!(comp.attributes[4].name, "action");
        assert_eq!(comp.attributes[5].name, "port");
    }

    #[test]
    fn test_parse_requirement_with_attributes() {
        let source = r#"
package ReqPkg {
    requirement def REQ_0001_Demo {
        id = "REQ-0001";
        text = "The system shall process data.";
        attribute uuidv5 : String = "5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9";
        attribute complexity_class : String = "Class P";
        attribute ac_01_demo : String = "Given: input. When: run. Then: success.";
        verify by AC_01_Demo;
        satisfy by DemoEngine;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = &pkg.requirement_defs[0];
        assert_eq!(req.name, "REQ_0001_Demo");
        assert_eq!(req.req_id, "REQ-0001");
        assert_eq!(req.text, "The system shall process data.");
        assert_eq!(req.attributes.len(), 3);
        assert_eq!(req.attributes[0].name, "uuidv5");
        assert_eq!(
            req.attributes[0].default_value.as_deref(),
            Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"")
        );
        assert_eq!(req.attributes[1].name, "complexity_class");
        assert_eq!(
            req.attributes[1].default_value.as_deref(),
            Some("\"Class P\"")
        );
        assert_eq!(req.attributes[2].name, "ac_01_demo");
        assert_eq!(
            req.attributes[2].default_value.as_deref(),
            Some("\"Given: input. When: run. Then: success.\"")
        );
        assert_eq!(req.verified_by, vec!["AC_01_Demo"]);
        assert_eq!(req.satisfied_by, vec!["DemoEngine"]);
    }

    #[test]
    fn test_parse_requirement_with_assumptions_constraints_derivations() {
        let source = r#"
package ReqPkg {
    requirement def REQ_0032_Test {
        id = "REQ-0032";
        text = "Req with constraints and derivations.";
        assume ValidSchema;
        require Invariant_Alpha;
        require Invariant_Beta;
        derived from REQ_0031_Parent;
        derived from REQ_0019_Sanitization;
        verify by AC_01;
        satisfy by Engine;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = &pkg.requirement_defs[0];
        assert_eq!(req.assumes, vec!["ValidSchema"]);
        assert_eq!(req.requires, vec!["Invariant_Alpha", "Invariant_Beta"]);
        assert_eq!(req.derived_from, vec!["REQ_0031_Parent", "REQ_0019_Sanitization"]);
    }

    #[test]
    fn test_parse_part_with_assert_constraint() {
        let source = r#"
package SubsystemPkg {
    part def Engine {
        assert constraint Invariant_Alpha;
        assert constraint Invariant_Beta;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.part_defs.len(), 1);
        let engine = &pkg.part_defs[0];
        assert_eq!(engine.constraints.len(), 2);
        assert_eq!(engine.constraints[0].name, "Invariant_Alpha");
        assert!(engine.constraints[0].is_assertion);
        assert_eq!(engine.constraints[1].name, "Invariant_Beta");
        assert!(engine.constraints[1].is_assertion);
    }

    #[test]
    fn test_parse_requirement_with_encapsulated_constraints() {
        let source = r#"
package ReqPkg {
    requirement def REQ_0001_Test {
        id = "REQ-0001";
        text = "Requirement with inner constraints.";
        doc /* Formula Alpha */
        constraint def Invariant_Alpha;
        assert constraint Invariant_Beta {
            x > 0;
        }
        require Invariant_Alpha;
        require Invariant_Beta;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = &pkg.requirement_defs[0];
        assert_eq!(req.constraints.len(), 2);
        assert_eq!(req.constraints[0].name, "Invariant_Alpha");
        assert_eq!(req.constraints[0].doc.as_deref(), Some("Formula Alpha"));
        assert!(!req.constraints[0].is_assertion);
        assert_eq!(req.constraints[1].name, "Invariant_Beta");
        assert!(req.constraints[1].is_assertion);
        assert_eq!(req.constraints[1].expression, "x > 0");
    }

    #[test]
    fn test_parse_import_declarations() {
        let source = r#"
package Subsystem_Imports {
    import Subsystem_1::*;
    import Subsystem_2::Engine;
    import Subsystem_3::Core::**;
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.imports.len(), 3);
        assert_eq!(pkg.imports[0].path, "Subsystem_1");
        assert!(pkg.imports[0].is_wildcard);
        assert!(!pkg.imports[0].is_recursive);

        assert_eq!(pkg.imports[1].path, "Subsystem_2::Engine");
        assert!(!pkg.imports[1].is_wildcard);
        assert!(!pkg.imports[1].is_recursive);

        assert_eq!(pkg.imports[2].path, "Subsystem_3::Core");
        assert!(pkg.imports[2].is_wildcard);
        assert!(pkg.imports[2].is_recursive);
    }

    #[test]
    fn test_parse_inline_connector_in_part_def() {
        let source = r#"
part def DEAPCompilerSystem {
    connect a.out to b.in;
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default").unwrap();
        assert_eq!(pkg.part_defs.len(), 1);
        let part = &pkg.part_defs[0];
        assert_eq!(part.name, "DEAPCompilerSystem");
        assert_eq!(part.connections.len(), 1);
        let conn = &part.connections[0];
        assert_eq!(conn.source_port, "a.out");
        assert_eq!(conn.target_port, "b.in");
        assert_eq!(conn.source_part.as_deref(), Some("a"));
        assert_eq!(conn.target_part.as_deref(), Some("b"));
        assert_eq!(conn.name, "c_a_out_to_b_in");
    }

    #[test]
    fn test_parse_multiple_root_packages() {
        let source = r#"
package Subsystem_1 {
    part def Engine;
}
package Subsystem_2 {
    part def Controller;
}
"#;
        let pkg = SysmlParser::parse_source(source, "RootSystem").unwrap();
        assert_eq!(pkg.name, "RootSystem");
        assert_eq!(pkg.packages.len(), 2);
        assert_eq!(pkg.packages[0].name, "Subsystem_1");
        assert_eq!(pkg.packages[0].part_defs.len(), 1);
        assert_eq!(pkg.packages[0].part_defs[0].name, "Engine");
        assert_eq!(pkg.packages[1].name, "Subsystem_2");
        assert_eq!(pkg.packages[1].part_defs.len(), 1);
        assert_eq!(pkg.packages[1].part_defs[0].name, "Controller");
    }

    #[test]
    fn test_parse_multi_file_sysml_tree() {
        let temp_dir = std::env::temp_dir().join(format!(
            "test_sysml_tree_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let conops_dir = temp_dir.join("conops");
        let sub1_dir = temp_dir.join("subsystems/sub1");
        fs::create_dir_all(&conops_dir).unwrap();
        fs::create_dir_all(&sub1_dir).unwrap();

        // 1. conops/actors.sysml
        fs::write(
            conops_dir.join("actors.sysml"),
            r#"
package ConOps {
    part def Operator;
}
"#,
        )
        .unwrap();

        // 2. subsystems/sub1/requirements.sysml
        fs::write(
            sub1_dir.join("requirements.sysml"),
            r#"
package Subsystem_1 {
    requirement def SubsystemSafetyReq {
        doc /* Subsystem shall fail safe */
    }
}
"#,
        )
        .unwrap();

        // 3. subsystems/sub1/architecture.sysml
        fs::write(
            sub1_dir.join("architecture.sysml"),
            r#"
package Subsystem_1 {
    part def SubsystemController;
}
"#,
        )
        .unwrap();

        // 4. model.sysml
        fs::write(
            temp_dir.join("model.sysml"),
            r#"
package DEAP_Compiler_System {
    import Subsystem_1::*;
    part def MainSystem;
}
"#,
        )
        .unwrap();

        // Add hidden and temporary files that should be filtered out
        fs::write(temp_dir.join(".hidden.sysml"), "package Hidden {}").unwrap();
        fs::write(temp_dir.join("#temp.sysml#"), "package Temp {}").unwrap();

        // Test file discovery
        let discovered = discover_sysml_files(&temp_dir);
        assert_eq!(discovered.len(), 4);
        assert!(!discovered.iter().any(|p| p.to_string_lossy().contains(".hidden")));
        assert!(!discovered.iter().any(|p| p.to_string_lossy().contains("#temp")));

        // Test parsing via parse_sysml_tree
        let root = parse_sysml_tree(&discovered).expect("parse_sysml_tree should succeed");

        // Asserts: unified root PackageDef contains all packages
        assert_eq!(root.packages.len(), 3);

        // 1. ConOps package
        let conops = root
            .packages
            .iter()
            .find(|p| p.name == "ConOps")
            .expect("ConOps package found");
        assert_eq!(conops.part_defs.len(), 1);
        assert_eq!(conops.part_defs[0].name, "Operator");

        // 2. Subsystem_1 package: merged from requirements.sysml and architecture.sysml
        let sub1 = root
            .packages
            .iter()
            .find(|p| p.name == "Subsystem_1")
            .expect("Subsystem_1 package found");
        assert_eq!(sub1.requirement_defs.len(), 1);
        assert_eq!(sub1.requirement_defs[0].name, "SubsystemSafetyReq");
        assert_eq!(sub1.part_defs.len(), 1);
        assert_eq!(sub1.part_defs[0].name, "SubsystemController");

        // 3. DEAP_Compiler_System package
        let deap = root
            .packages
            .iter()
            .find(|p| p.name == "DEAP_Compiler_System")
            .expect("DEAP_Compiler_System package found");
        assert_eq!(deap.imports.len(), 1);
        assert_eq!(deap.imports[0].path, "Subsystem_1");
        assert!(deap.imports[0].is_wildcard);
        assert_eq!(deap.part_defs.len(), 1);
        assert_eq!(deap.part_defs[0].name, "MainSystem");

        // Test directory parsing via parse_sysml_directory
        let root_from_dir = parse_sysml_directory(&temp_dir).expect("parse_sysml_directory should succeed");
        assert_eq!(root_from_dir.packages.len(), 3);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_parse_use_case_def_preserves_inner_doc_comments() {
        let source = r#"
package TestPackage {
    use case def UC_01_Vision {
        doc /*
        Actors: HumanEngineer, CIContinuousIntegrationRunner
        Subject: SystemVisionEngine
        Preconditions: Compiler runtime environment is initialized with valid configuration flags.
        Trigger: Compilation pipeline startup command dispatched by operator or CI runner.
        Main Success Scenario:
        1. Operator dispatches compilation command with workspace schema root.
        2. SystemVisionEngine initializes compiler namespace UUID constant NS_DEAP.
        3. InvariantBootstrapEngine asserts zero hardcoded domain concept rules across runtime context.
        4. NamespaceUuidGenerator pre-computes topological UUIDv5 namespace roots.
        5. DomainVocabularyValidator inspects grammar vocabulary tokens against prohibited domain ontologies.
        6. SystemVisionEngine emits immutable CompilerRulePort descriptor to downstream ingestion engines.
        Alternate and Exception Flows:
        5a. Prohibited domain term detected: SystemVisionEngine halts pipeline and emits diagnostic E0100.
        5b. Namespace UUID collision: SystemVisionEngine regenerates topological anchor with depth offset and emits E0101.
        5c. Malformed configuration flags: Compiler logs invalid argument diagnostics and falls back to safe defaults.
        5d. File permission error on workspace: Execution halts safely emitting file access diagnostic.
        5e. Invariant verification timeout: Watchdog timer triggers and logs diagnostic E0199 before graceful termination.
        5f. Non-deterministic execution detected: Checksum validation fails and emits reproducibility diagnostic E0102.
        5g. Memory allocation failure during bootstrap: Engine triggers emergency page compaction.
        5h. Missing root namespace configuration: Fallback to fixed RFC 4122 compiler constant.
        5i. Incompatible schema revision requested: Compiler warns and falls back to backwards-compatible parser mode.
        5j. Circular dependency detected in rule definitions: Rule topological sorter breaks cycle and reports E0103.
        Postconditions: Foundational compiler invariants and UUIDv5 namespaces are established deterministically.
        */
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "TestModel").unwrap();
        assert_eq!(pkg.use_case_defs.len(), 1);
        let uc = &pkg.use_case_defs[0];
        assert_eq!(uc.name, "UC_01_Vision");
        assert!(uc.doc.is_some());
        let doc = uc.doc.as_ref().unwrap();
        assert!(doc.contains("Actors: HumanEngineer, CIContinuousIntegrationRunner"));
        assert!(doc.contains("Subject: SystemVisionEngine"));
        assert!(doc.contains("Preconditions: Compiler runtime environment is initialized"));
        assert!(doc.contains("Trigger: Compilation pipeline startup command"));
        assert!(doc.contains("Main Success Scenario:"));
        assert!(doc.contains("1. Operator dispatches compilation command"));
        assert!(doc.contains("Alternate and Exception Flows:"));
        assert!(doc.contains("5a. Prohibited domain term detected"));
        assert!(doc.contains("5j. Circular dependency detected"));
        assert!(doc.contains("Postconditions: Foundational compiler invariants"));
    }

    #[test]
    fn test_parse_use_case_def_outer_and_inner_doc_concatenation() {
        let source = r#"
doc /* Package doc */
package TestPackage {
    doc /* Outer summary for UC_05 */
    use case def UC_05_Physical_Metrology {
        doc /*
        Actors: CIContinuousIntegrationRunner, HumanEngineer
        Subject: MetrologyFlowEngine
        Preconditions: AST stream is available.
        Trigger: Metrology flow verification command.
        Main Success Scenario:
        1. GrammarLoweringEngine passes AST stream.
        Alternate and Exception Flows:
        5a. SI unit dimension mismatch.
        Postconditions: Units verified.
        */
        subject MetrologyFlowEngine;
        actor CIContinuousIntegrationRunner;
        step GrammarLoweringEngine;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "TestModel").unwrap();
        assert_eq!(pkg.use_case_defs.len(), 1);
        let uc = &pkg.use_case_defs[0];
        assert_eq!(uc.name, "UC_05_Physical_Metrology");
        assert_eq!(uc.subject.as_deref(), Some("MetrologyFlowEngine"));
        assert_eq!(uc.actors.len(), 1);
        assert_eq!(uc.actors[0], "CIContinuousIntegrationRunner");
        assert_eq!(uc.steps.len(), 1);
        assert_eq!(uc.steps[0], "GrammarLoweringEngine");
        assert!(uc.doc.is_some());
        let doc = uc.doc.as_ref().unwrap();
        assert!(doc.contains("Outer summary for UC_05"));
        assert!(doc.contains("Actors: CIContinuousIntegrationRunner, HumanEngineer"));
        assert!(doc.contains("Main Success Scenario:"));
        assert!(doc.contains("5a. SI unit dimension mismatch."));
    }
}



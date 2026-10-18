//! Recursive descent parser translating SysML v2 / KerML tokens into typed AST.

use crate::lexer::scanner::Scanner;
use crate::lexer::token::{Span, Token, TokenKind};
use deap_core::sysml_ast::*;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Sentinel returned when the token stream is empty; never indexed, never mutated.
static GRAMMAR_EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    span: Span {
        start: 0,
        end: 0,
        line: 0,
        col: 0,
    },
};

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
            TokenKind::Satisfy => {
                let req_target = self.parse_satisfy_stmt()?;
                if !req_target.is_empty() {
                    if let Some(parent) = parent_part {
                        if !parent.satisfied_requirements.contains(&req_target) {
                            parent.satisfied_requirements.push(req_target);
                        }
                    } else if !pkg.satisfied_requirements.contains(&req_target) {
                        pkg.satisfied_requirements.push(req_target);
                    }
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
            TokenKind::Calc => {
                let c = self.parse_calc_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.calcs.push(c);
                } else {
                    pkg.calc_defs.push(c);
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
        self.tokens.get(self.pos + 1).is_some_and(|t| t.kind == TokenKind::Port)
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
                    if self.check(&TokenKind::Flow) {
                        self.advance();
                        if self.check(&TokenKind::In) {
                            self.advance();
                            flow_dir = "in".to_string();
                        } else if self.check(&TokenKind::Out) {
                            self.advance();
                            flow_dir = "out".to_string();
                        }
                    } else {
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
            // Array-valued attribute types (`Type[]`), e.g. YANG `leaf-list` lowering.
            if self.check(&TokenKind::OpenBracket) {
                self.advance();
                self.expect(&TokenKind::CloseBracket)?;
                type_name.push_str("[]");
            }
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

        let source_part = source_port.split_once('.').map(|(owner, _)| owner.to_string());
        let target_part = target_port.split_once('.').map(|(owner, _)| owner.to_string());

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

        let source_part = source_port.split_once('.').map(|(owner, _)| owner.to_string());
        let target_part = target_port.split_once('.').map(|(owner, _)| owner.to_string());

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
        self.tokens.get(self.pos + 1).is_some_and(|t| t.kind == TokenKind::From)
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
                    if self.check(&TokenKind::Assert) {
                        self.advance();
                    }
                    if self.check(&TokenKind::Requirement) || self.check(&TokenKind::Constraint) {
                        self.advance();
                    }
                    let targets_raw = self.read_until_semi();
                    for target in targets_raw.split(',') {
                        let trimmed = target.trim();
                        if !trimmed.is_empty() {
                            verified_requirements.push(trimmed.to_string());
                        }
                    }
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

    fn parse_calc_decl(&mut self, doc: Option<String>) -> Result<CalcDef, ParseError> {
        self.expect(&TokenKind::Calc)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut in_params = Vec::new();
        let mut return_type = None;
        let mut final_doc = doc;
        let mut expr_lines = Vec::new();

        if self.check(&TokenKind::OpenParen) {
            self.advance();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                if self.check(&TokenKind::In) || self.check(&TokenKind::Out) || self.check(&TokenKind::Inout) {
                    self.advance();
                }
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
                let mut default_value = None;
                if self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq) {
                    self.advance();
                    let mut pieces = Vec::new();
                    while !self.check(&TokenKind::Comma) && !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
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
                    default_value = Some(pieces.join(" "));
                }
                in_params.push(AttributeDef {
                    name: p_name,
                    type_name: p_type,
                    default_value,
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
            let mut type_name = self.expect_ident()?;
            while self.check(&TokenKind::DoubleColon) {
                self.advance();
                type_name.push_str("::");
                type_name.push_str(&self.expect_ident()?);
            }
            return_type = Some(type_name);
        }

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
                if self.check(&TokenKind::In) || self.check(&TokenKind::Attribute) {
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
                    let default_value = if self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq) {
                        self.advance();
                        Some(self.read_until_semi())
                    } else {
                        if self.check(&TokenKind::Semi) {
                            self.advance();
                        }
                        None
                    };
                    in_params.push(AttributeDef {
                        name: p_name,
                        type_name: p_type,
                        default_value,
                        doc: None,
                    });
                } else {
                    let stmt = self.read_until_semi();
                    let trimmed = stmt.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(rest) = trimmed.strip_prefix("return") {
                        let rest = rest.trim();
                        if let Some((lhs, rhs)) = rest.split_once('=') {
                            if return_type.is_none() {
                                if let Some((_, rtype)) = lhs.split_once(':') {
                                    let first_tok = rtype.split_whitespace().next().unwrap_or("");
                                    if !first_tok.is_empty() {
                                        return_type = Some(first_tok.to_string());
                                    }
                                }
                            }
                            let rhs_trimmed = rhs.trim();
                            if !rhs_trimmed.is_empty() {
                                expr_lines.push(rhs_trimmed.to_string());
                            }
                        } else {
                            let is_math_expr = rest.contains('+') || rest.contains('-') || rest.contains('*') || rest.contains('/') || rest.contains('(');
                            if return_type.is_none() && !is_math_expr && (rest.contains(':') || rest.starts_with("type ") || rest.split_whitespace().count() <= 2) {
                                let type_part = if let Some((_, after_colon)) = rest.split_once(':') {
                                    after_colon.trim()
                                } else {
                                    rest.trim_start_matches("type").trim()
                                };
                                let first_tok = type_part.split_whitespace().next().unwrap_or("");
                                if !first_tok.is_empty() {
                                    return_type = Some(first_tok.to_string());
                                    continue;
                                }
                            }
                            let expr_candidate = rest.trim_start_matches(':').trim();
                            if !expr_candidate.is_empty() {
                                expr_lines.push(expr_candidate.to_string());
                            }
                        }
                    } else {
                        expr_lines.push(trimmed.to_string());
                    }
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        let expression = if expr_lines.is_empty() {
            None
        } else {
            Some(expr_lines.join("\n"))
        };

        Ok(CalcDef {
            name,
            doc: final_doc,
            in_params,
            return_type,
            expression,
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
            self.tokens
                .get(self.pos)
                .is_some_and(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(kind))
        }
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .unwrap_or(&GRAMMAR_EOF_TOKEN)
    }

    fn peek_kind(&self) -> &TokenKind {
        if self.is_at_end() {
            &TokenKind::Eof
        } else {
            self.tokens.get(self.pos).map_or(&TokenKind::Eof, |t| &t.kind)
        }
    }

    fn advance(&mut self) -> &Token {
        let idx = self.pos;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        self.tokens
            .get(idx)
            .or_else(|| self.tokens.last())
            .unwrap_or(&GRAMMAR_EOF_TOKEN)
    }

    fn is_at_end(&self) -> bool {
        self.tokens.get(self.pos).is_none_or(|t| t.kind == TokenKind::Eof)
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

    fn parse_satisfy_stmt(&mut self) -> Result<String, ParseError> {
        self.expect(&TokenKind::Satisfy)?;
        while matches!(self.peek_kind(), TokenKind::DocComment(_) | TokenKind::LineComment(_)) {
            self.advance();
        }
        if self.check(&TokenKind::Requirement) {
            self.advance();
        }
        while matches!(self.peek_kind(), TokenKind::DocComment(_) | TokenKind::LineComment(_)) {
            self.advance();
        }
        let mut raw = String::new();
        while !self.check(&TokenKind::Semi)
            && !self.check(&TokenKind::CloseBrace)
            && !self.check(&TokenKind::By)
            && !self.is_at_end()
        {
            let tok = self.advance();
            match &tok.kind {
                TokenKind::DocComment(_) | TokenKind::LineComment(_) => {}
                TokenKind::Ident(s) => raw.push_str(s),
                TokenKind::StringLit(s) => raw.push_str(s),
                TokenKind::IntLit(n) => {
                    let span_len = tok.span.end.saturating_sub(tok.span.start);
                    if *n < 0 {
                        let d = span_len.saturating_sub(1);
                        raw.push_str(&format!("-{:0width$}", n.abs(), width = d));
                    } else if *n == 0 {
                        if span_len > 1 && !raw.ends_with('-') {
                            let d = span_len.saturating_sub(1);
                            raw.push_str(&format!("-{:0width$}", 0, width = d));
                        } else {
                            raw.push_str(&format!("{:0width$}", 0, width = span_len));
                        }
                    } else {
                        raw.push_str(&format!("{:0width$}", n, width = span_len));
                    }
                }
                TokenKind::Minus => raw.push('-'),
                TokenKind::Dot => raw.push('.'),
                TokenKind::DoubleColon => raw.push_str("::"),
                TokenKind::Slash => raw.push('/'),
                _ => raw.push_str(tok.kind.as_str()),
            }
        }
        if self.check(&TokenKind::By) {
            self.read_until_semi();
        } else {
            self.expect_semi()?;
        }
        Ok(raw.trim().trim_matches('"').trim_matches('\'').trim().to_string())
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
            if path.extension().is_some_and(|ext| ext == "sysml") {
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
/// `risk_defs`, `connection_defs`, `calc_defs`).
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
    dest.calc_defs.extend(src.calc_defs);
    for sat in src.satisfied_requirements {
        if !dest.satisfied_requirements.contains(&sat) {
            dest.satisfied_requirements.push(sat);
        }
    }

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
    fn test_parse_ground_truth_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = r#"
package SystemModel_SSOT {
    doc /* SSOT for Generic System Architecture and Safety Model */

    attribute channelCount : Integer = 4;
    attribute topologyConfiguration : String = "Symmetric";
    attribute primaryOperatingLimit : Real = 12.0;
    attribute secondaryOperatingLimit : Real = 12.0;

    part def StructuralFrame {
        attribute nominalMass : Real = 25.0;
    }

    part def PrimaryControlUnit {
        port command_bus : CommandBus;
        port telemetry : TelemetryLink;
    }
}
"#;
        let pkg = SysmlParser::parse_source(fixture, "TestModel")?;

        assert_eq!(pkg.name, "SystemModel_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("SSOT for Generic System Architecture and Safety Model")
        );
        assert_eq!(pkg.attribute_defs.len(), 4);
        let channel_count = pkg.attribute_defs.first().ok_or("missing attribute 0")?;
        assert_eq!(channel_count.name, "channelCount");
        assert_eq!(channel_count.type_name, "Integer");
        assert_eq!(channel_count.default_value.as_deref(), Some("4"));

        assert_eq!(pkg.part_defs.len(), 2);
        let frame = pkg.part_defs.first().ok_or("missing part 0")?;
        assert_eq!(frame.name, "StructuralFrame");
        assert_eq!(frame.attributes.len(), 1);
        let mass = frame.attributes.first().ok_or("missing frame attribute 0")?;
        assert_eq!(mass.name, "nominalMass");
        assert_eq!(mass.default_value.as_deref(), Some("25.0"));

        let control_unit = pkg.part_defs.get(1).ok_or("missing part 1")?;
        assert_eq!(control_unit.name, "PrimaryControlUnit");
        assert_eq!(control_unit.ports.len(), 2);
        let command_bus = control_unit.ports.first().ok_or("missing port 0")?;
        assert_eq!(command_bus.name, "command_bus");
        assert_eq!(command_bus.type_name, "CommandBus");
        let telemetry = control_unit.ports.get(1).ok_or("missing port 1")?;
        assert_eq!(telemetry.name, "telemetry");
        assert_eq!(telemetry.type_name, "TelemetryLink");
        Ok(())
    }

    #[test]
    fn test_parse_grounded_limits_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = include_str!("../../../../tests/fixtures/safety/grounded_limits_model.sysml");
        let pkg = SysmlParser::parse_source(fixture, "TestLimits")?;

        assert_eq!(pkg.name, "GroundedLimits_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("Ground truth limits for flight envelope verification")
        );
        assert_eq!(pkg.attribute_defs.len(), 5);
        assert_eq!(pkg.attribute_defs.first().ok_or("missing first element of pkg.attribute_defs")?.name, "catapultPressureLimitBar");
        assert_eq!(pkg.attribute_defs.first().ok_or("missing first element of pkg.attribute_defs")?.default_value.as_deref(), Some("15.0"));
        Ok(())
    }

    #[test]
    fn test_parse_complex_system_model() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.name, "FlightControl_SSOT");
        assert_eq!(
            pkg.doc.as_deref(),
            Some("Primary Flight Control and Mission Computer Architecture")
        );
        assert_eq!(pkg.attribute_defs.len(), 2);
        assert_eq!(pkg.part_defs.len(), 2);

        let nav = pkg.part_defs.first().ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(nav.name, "NavigationSubsystem");
        assert_eq!(nav.doc.as_deref(), Some("GPS, IMU and Sensor Fusion Unit"));
        assert_eq!(nav.ports.len(), 2);
        assert_eq!(nav.ports.first().ok_or("missing first element of nav.ports")?.direction, "out");
        assert_eq!(nav.ports.first().ok_or("missing first element of nav.ports")?.name, "nav_solution");
        assert_eq!(nav.ports.get(1).ok_or("missing element 1 of nav.ports")?.direction, "in");
        assert!(nav.ports.get(1).ok_or("missing element 1 of nav.ports")?.is_conjugated);
        assert_eq!(nav.ports.get(1).ok_or("missing element 1 of nav.ports")?.name, "correction_stream");
        assert_eq!(nav.actions.len(), 1);
        assert_eq!(nav.actions.first().ok_or("missing first element of nav.actions")?.name, "ComputeNavigationState");
        assert_eq!(nav.states.len(), 1);
        assert_eq!(nav.states.first().ok_or("missing first element of nav.states")?.name, "NavFilterState");
        assert_eq!(nav.states.first().ok_or("missing first element of nav.states")?.entry_action.as_deref(), Some("InitINS"));
        assert_eq!(nav.states.first().ok_or("missing first element of nav.states")?.do_action.as_deref(), Some("RunEKF"));

        let actuator = pkg.part_defs.get(1).ok_or("missing element 1 of pkg.part_defs")?;
        assert_eq!(actuator.name, "ActuatorController");
        assert_eq!(actuator.constraints.len(), 1);
        assert_eq!(actuator.constraints.first().ok_or("missing first element of actuator.constraints")?.name, "DeflectionBounds");
        assert!(actuator.constraints.first().ok_or("missing first element of actuator.constraints")?.is_assertion);
        assert!(actuator.constraints.first().ok_or("missing first element of actuator.constraints")?.expression.contains("deflectionDeg"));

        assert_eq!(pkg.connection_defs.len(), 1);
        let conn = pkg.connection_defs.first().ok_or("missing first element of pkg.connection_defs")?;
        assert_eq!(conn.name, "NavToActuatorBus");
        assert_eq!(conn.source_port, "NavigationSubsystem.nav_solution");
        assert_eq!(conn.target_port, "ActuatorController.servo_cmd");
        assert_eq!(conn.source_part.as_deref(), Some("NavigationSubsystem"));
        assert_eq!(conn.target_part.as_deref(), Some("ActuatorController"));
        assert_eq!(conn.protocol.as_deref(), Some("CANopen"));
        assert_eq!(conn.latency_ms, Some(2.5));

        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing first element of pkg.requirement_defs")?;
        assert_eq!(req.name, "Req_Nav_001");
        assert_eq!(req.req_id, "REQ-NAV-001");
        assert_eq!(
            req.text,
            "Navigation system shall compute attitude at minimum 100 Hz."
        );
        assert_eq!(req.verified_by, vec!["HardwareInLoopTest"]);
        assert_eq!(req.satisfied_by, vec!["NavigationSubsystem"]);
        Ok(())
    }

    #[test]
    fn test_parse_contextual_keywords_as_identifiers() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.name, "TestPkg");
        let comp = pkg.part_defs.first().ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(comp.name, "Component");
        assert_eq!(comp.attributes.len(), 6);
        assert_eq!(comp.attributes.first().ok_or("missing first element of comp.attributes")?.name, "text");
        assert_eq!(comp.attributes.get(1).ok_or("missing element 1 of comp.attributes")?.name, "doc");
        assert_eq!(comp.attributes.get(2).ok_or("missing element 2 of comp.attributes")?.name, "item");
        assert_eq!(comp.attributes.get(3).ok_or("missing element 3 of comp.attributes")?.name, "state");
        assert_eq!(comp.attributes.get(4).ok_or("missing element 4 of comp.attributes")?.name, "action");
        assert_eq!(comp.attributes.get(5).ok_or("missing element 5 of comp.attributes")?.name, "port");
        Ok(())
    }

    #[test]
    fn test_parse_requirement_with_attributes() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing first element of pkg.requirement_defs")?;
        assert_eq!(req.name, "REQ_0001_Demo");
        assert_eq!(req.req_id, "REQ-0001");
        assert_eq!(req.text, "The system shall process data.");
        assert_eq!(req.attributes.len(), 3);
        assert_eq!(req.attributes.first().ok_or("missing first element of req.attributes")?.name, "uuidv5");
        assert_eq!(
            req.attributes.first().ok_or("missing first element of req.attributes")?.default_value.as_deref(),
            Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"")
        );
        assert_eq!(req.attributes.get(1).ok_or("missing element 1 of req.attributes")?.name, "complexity_class");
        assert_eq!(
            req.attributes.get(1).ok_or("missing element 1 of req.attributes")?.default_value.as_deref(),
            Some("\"Class P\"")
        );
        assert_eq!(req.attributes.get(2).ok_or("missing element 2 of req.attributes")?.name, "ac_01_demo");
        assert_eq!(
            req.attributes.get(2).ok_or("missing element 2 of req.attributes")?.default_value.as_deref(),
            Some("\"Given: input. When: run. Then: success.\"")
        );
        assert_eq!(req.verified_by, vec!["AC_01_Demo"]);
        assert_eq!(req.satisfied_by, vec!["DemoEngine"]);
        Ok(())
    }

    #[test]
    fn test_parse_requirement_with_assumptions_constraints_derivations() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing first element of pkg.requirement_defs")?;
        assert_eq!(req.assumes, vec!["ValidSchema"]);
        assert_eq!(req.requires, vec!["Invariant_Alpha", "Invariant_Beta"]);
        assert_eq!(req.derived_from, vec!["REQ_0031_Parent", "REQ_0019_Sanitization"]);
        Ok(())
    }

    #[test]
    fn test_parse_part_with_assert_constraint() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package SubsystemPkg {
    part def Engine {
        assert constraint Invariant_Alpha;
        assert constraint Invariant_Beta;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.part_defs.len(), 1);
        let engine = pkg.part_defs.first().ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(engine.constraints.len(), 2);
        assert_eq!(engine.constraints.first().ok_or("missing first element of engine.constraints")?.name, "Invariant_Alpha");
        assert!(engine.constraints.first().ok_or("missing first element of engine.constraints")?.is_assertion);
        assert_eq!(engine.constraints.get(1).ok_or("missing element 1 of engine.constraints")?.name, "Invariant_Beta");
        assert!(engine.constraints.get(1).ok_or("missing element 1 of engine.constraints")?.is_assertion);
        Ok(())
    }

    #[test]
    fn test_parse_requirement_with_encapsulated_constraints() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing first element of pkg.requirement_defs")?;
        assert_eq!(req.constraints.len(), 2);
        assert_eq!(req.constraints.first().ok_or("missing first element of req.constraints")?.name, "Invariant_Alpha");
        assert_eq!(req.constraints.first().ok_or("missing first element of req.constraints")?.doc.as_deref(), Some("Formula Alpha"));
        assert!(!req.constraints.first().ok_or("missing first element of req.constraints")?.is_assertion);
        assert_eq!(req.constraints.get(1).ok_or("missing element 1 of req.constraints")?.name, "Invariant_Beta");
        assert!(req.constraints.get(1).ok_or("missing element 1 of req.constraints")?.is_assertion);
        assert_eq!(req.constraints.get(1).ok_or("missing element 1 of req.constraints")?.expression, "x > 0");
        Ok(())
    }

    #[test]
    fn test_parse_import_declarations() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package Subsystem_Imports {
    import Subsystem_1::*;
    import Subsystem_2::Engine;
    import Subsystem_3::Core::**;
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.imports.len(), 3);
        assert_eq!(pkg.imports.first().ok_or("missing first element of pkg.imports")?.path, "Subsystem_1");
        assert!(pkg.imports.first().ok_or("missing first element of pkg.imports")?.is_wildcard);
        assert!(!pkg.imports.first().ok_or("missing first element of pkg.imports")?.is_recursive);

        assert_eq!(pkg.imports.get(1).ok_or("missing element 1 of pkg.imports")?.path, "Subsystem_2::Engine");
        assert!(!pkg.imports.get(1).ok_or("missing element 1 of pkg.imports")?.is_wildcard);
        assert!(!pkg.imports.get(1).ok_or("missing element 1 of pkg.imports")?.is_recursive);

        assert_eq!(pkg.imports.get(2).ok_or("missing element 2 of pkg.imports")?.path, "Subsystem_3::Core");
        assert!(pkg.imports.get(2).ok_or("missing element 2 of pkg.imports")?.is_wildcard);
        assert!(pkg.imports.get(2).ok_or("missing element 2 of pkg.imports")?.is_recursive);
        Ok(())
    }

    #[test]
    fn test_parse_inline_connector_in_part_def() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
part def DEAPCompilerSystem {
    connect a.out to b.in;
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.part_defs.len(), 1);
        let part = pkg.part_defs.first().ok_or("missing first element of pkg.part_defs")?;
        assert_eq!(part.name, "DEAPCompilerSystem");
        assert_eq!(part.connections.len(), 1);
        let conn = part.connections.first().ok_or("missing first element of part.connections")?;
        assert_eq!(conn.source_port, "a.out");
        assert_eq!(conn.target_port, "b.in");
        assert_eq!(conn.source_part.as_deref(), Some("a"));
        assert_eq!(conn.target_part.as_deref(), Some("b"));
        assert_eq!(conn.name, "c_a_out_to_b_in");
        Ok(())
    }

    #[test]
    fn test_parse_multiple_root_packages() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package Subsystem_1 {
    part def Engine;
}
package Subsystem_2 {
    part def Controller;
}
"#;
        let pkg = SysmlParser::parse_source(source, "RootSystem")?;
        assert_eq!(pkg.name, "RootSystem");
        assert_eq!(pkg.packages.len(), 2);
        assert_eq!(pkg.packages.first().ok_or("missing first element of pkg.packages")?.name, "Subsystem_1");
        assert_eq!(pkg.packages.first().ok_or("missing first element of pkg.packages")?.part_defs.len(), 1);
        assert_eq!(pkg.packages.first().ok_or("missing first element of pkg.packages")?.part_defs.first().ok_or("missing first element of part_defs")?.name, "Engine");
        assert_eq!(pkg.packages.get(1).ok_or("missing element 1 of pkg.packages")?.name, "Subsystem_2");
        assert_eq!(pkg.packages.get(1).ok_or("missing element 1 of pkg.packages")?.part_defs.len(), 1);
        assert_eq!(pkg.packages.get(1).ok_or("missing element 1 of pkg.packages")?.part_defs.first().ok_or("missing first element of part_defs")?.name, "Controller");
        Ok(())
    }

    #[test]
    fn test_parse_multi_file_sysml_tree() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!(
            "test_sysml_tree_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                ?
                .as_nanos()
        ));
        let conops_dir = temp_dir.join("conops");
        let sub1_dir = temp_dir.join("subsystems/sub1");
        fs::create_dir_all(&conops_dir)?;
        fs::create_dir_all(&sub1_dir)?;

        // 1. conops/actors.sysml
        fs::write(
            conops_dir.join("actors.sysml"),
            r#"
package ConOps {
    part def Operator;
}
"#,
        )
        ?;

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
        ?;

        // 3. subsystems/sub1/architecture.sysml
        fs::write(
            sub1_dir.join("architecture.sysml"),
            r#"
package Subsystem_1 {
    part def SubsystemController;
}
"#,
        )
        ?;

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
        ?;

        // Add hidden and temporary files that should be filtered out
        fs::write(temp_dir.join(".hidden.sysml"), "package Hidden {}")?;
        fs::write(temp_dir.join("#temp.sysml#"), "package Temp {}")?;

        // Test file discovery
        let discovered = discover_sysml_files(&temp_dir);
        assert_eq!(discovered.len(), 4);
        assert!(!discovered.iter().any(|p| p.to_string_lossy().contains(".hidden")));
        assert!(!discovered.iter().any(|p| p.to_string_lossy().contains("#temp")));

        // Test parsing via parse_sysml_tree
        let root = parse_sysml_tree(&discovered)?;

        // Asserts: unified root PackageDef contains all packages
        assert_eq!(root.packages.len(), 3);

        // 1. ConOps package
        let conops = root
            .packages
            .iter()
            .find(|p| p.name == "ConOps")
            .ok_or("ConOps package missing")?;
        assert_eq!(conops.part_defs.len(), 1);
        assert_eq!(conops.part_defs.first().ok_or("missing first element of conops.part_defs")?.name, "Operator");

        // 2. Subsystem_1 package: merged from requirements.sysml and architecture.sysml
        let sub1 = root
            .packages
            .iter()
            .find(|p| p.name == "Subsystem_1")
            .ok_or("Subsystem_1 package missing")?;
        assert_eq!(sub1.requirement_defs.len(), 1);
        assert_eq!(sub1.requirement_defs.first().ok_or("missing first element of sub1.requirement_defs")?.name, "SubsystemSafetyReq");
        assert_eq!(sub1.part_defs.len(), 1);
        assert_eq!(sub1.part_defs.first().ok_or("missing first element of sub1.part_defs")?.name, "SubsystemController");

        // 3. DEAP_Compiler_System package
        let deap = root
            .packages
            .iter()
            .find(|p| p.name == "DEAP_Compiler_System")
            .ok_or("DEAP_Compiler_System package missing")?;
        assert_eq!(deap.imports.len(), 1);
        assert_eq!(deap.imports.first().ok_or("missing first element of deap.imports")?.path, "Subsystem_1");
        assert!(deap.imports.first().ok_or("missing first element of deap.imports")?.is_wildcard);
        assert_eq!(deap.part_defs.len(), 1);
        assert_eq!(deap.part_defs.first().ok_or("missing first element of deap.part_defs")?.name, "MainSystem");

        // Test directory parsing via parse_sysml_directory
        let root_from_dir = parse_sysml_directory(&temp_dir)?;
        assert_eq!(root_from_dir.packages.len(), 3);

        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_issue_444_merge_package_defs_preserves_calc_defs() -> Result<(), Box<dyn std::error::Error>> {
        use deap_core::sysml_ast::CalcDef;

        let mut dest = PackageDef {
            name: "TrajectorySubsystem".to_string(),
            ..Default::default()
        };
        let mut src = PackageDef {
            name: "TrajectorySubsystem".to_string(),
            ..Default::default()
        };
        src.calc_defs.push(CalcDef {
            name: "ComputeTargetTrajectory".to_string(),
            return_type: Some("TrajectoryVector".to_string()),
            ..Default::default()
        });

        merge_package_defs(&mut dest, src);
        assert_eq!(
            dest.calc_defs.len(),
            1,
            "merge_package_defs must preserve calc_defs from src package"
        );
        assert_eq!(
            dest.calc_defs[0].name,
            "ComputeTargetTrajectory"
        );
        Ok(())
    }

    #[test]
    fn test_parse_use_case_def_preserves_inner_doc_comments() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "TestModel")?;
        assert_eq!(pkg.use_case_defs.len(), 1);
        let uc = pkg.use_case_defs.first().ok_or("missing first element of pkg.use_case_defs")?;
        assert_eq!(uc.name, "UC_01_Vision");
        assert!(uc.doc.is_some());
        let doc = uc.doc.as_ref().ok_or("use case doc missing")?;
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
        Ok(())
    }

    #[test]
    fn test_parse_use_case_def_outer_and_inner_doc_concatenation() -> Result<(), Box<dyn std::error::Error>> {
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
        let pkg = SysmlParser::parse_source(source, "TestModel")?;
        assert_eq!(pkg.use_case_defs.len(), 1);
        let uc = pkg.use_case_defs.first().ok_or("missing first element of pkg.use_case_defs")?;
        assert_eq!(uc.name, "UC_05_Physical_Metrology");
        assert_eq!(uc.subject.as_deref(), Some("MetrologyFlowEngine"));
        assert_eq!(uc.actors.len(), 1);
        assert_eq!(uc.actors.first().ok_or("missing first element of uc.actors")?, "CIContinuousIntegrationRunner");
        assert_eq!(uc.steps.len(), 1);
        assert_eq!(uc.steps.first().ok_or("missing first element of uc.steps")?, "GrammarLoweringEngine");
        assert!(uc.doc.is_some());
        let doc = uc.doc.as_ref().ok_or("use case doc missing")?;
        assert!(doc.contains("Outer summary for UC_05"));
        assert!(doc.contains("Actors: CIContinuousIntegrationRunner, HumanEngineer"));
        assert!(doc.contains("Main Success Scenario:"));
        assert!(doc.contains("5a. SI unit dimension mismatch."));
        Ok(())
    }

    #[test]
    fn test_parse_test_case_def_with_assert_constraint_ids() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package AutonomousSystem {
    assert constraint Assert_UCA_33 {
        interlockState == InterlockStatus::ENGAGED;
    }

    assert constraint Assert_UCA_34 {
        lockConfidence >= 0.98;
    }

    test case def TC_UCA_33_Verification {
        subject SafetyController;
        verify requirement Assert_UCA_33, Assert_UCA_34;
        objective "Verify safety interlock per UCA-33";
        step trigger_interlock;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "TestModel")?;
        assert_eq!(pkg.test_case_defs.len(), 1);
        let tc = pkg.test_case_defs.first().ok_or("missing first element of pkg.test_case_defs")?;
        assert_eq!(tc.name, "TC_UCA_33_Verification");
        assert_eq!(tc.verified_requirements.len(), 2);
        assert_eq!(tc.verified_requirements.first().ok_or("missing first element of tc.verified_requirements")?, "Assert_UCA_33");
        assert_eq!(tc.verified_requirements.get(1).ok_or("missing element 1 of tc.verified_requirements")?, "Assert_UCA_34");
        Ok(())
    }

    #[test]
    fn test_parse_attribute_decl_accepts_array_bracket_type_suffix() {
        let source = r#"
package ArrayModel {
    attribute labelSet : String[];
    attribute scalarValue : Integer = 3;
    part def Holder {
        attribute counts : Integer[];
        attribute weight : Real = 1.5;
    }
}
"#;
        let parsed = SysmlParser::parse_source(source, "ArrayModel");
        assert!(parsed.is_ok(), "Type[] attribute failed to parse: {:?}", parsed.err());
        let pkg = parsed.unwrap_or_default();
        let label_set = pkg.attribute_defs.iter().find(|a| a.name == "labelSet");
        assert_eq!(label_set.map(|a| a.type_name.as_str()), Some("String[]"));
        assert_eq!(label_set.and_then(|a| a.default_value.clone()), None);
        let scalar = pkg.attribute_defs.iter().find(|a| a.name == "scalarValue");
        assert_eq!(scalar.map(|a| a.type_name.as_str()), Some("Integer"));
        let holder = pkg.part_defs.iter().find(|p| p.name == "Holder");
        let counts = holder.and_then(|p| p.attributes.iter().find(|a| a.name == "counts"));
        assert_eq!(counts.map(|a| a.type_name.as_str()), Some("Integer[]"));
        let weight = holder.and_then(|p| p.attributes.iter().find(|a| a.name == "weight"));
        assert_eq!(weight.map(|a| a.type_name.as_str()), Some("Real"));
    }

    #[test]
    fn test_parse_attribute_decl_rejects_unclosed_array_bracket() {
        let source = "package BadArray {\n    attribute labelSet : String[;\n}\n";
        let parsed = SysmlParser::parse_source(source, "BadArray");
        assert!(parsed.is_err(), "unclosed '[' must be a parse error, got {:?}", parsed.ok());
    }

    #[test]
    fn test_issue_460_parse_and_roundtrip_satisfy_requirements() -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let source = r#"
package UserInterfaceSystem {
    satisfy requirement REQ-UI-00;

    part def TouchScreenPanel {
        satisfy requirement REQ-UI-01;
        satisfy requirement REQ-UI-02 by PrimaryTouchSensor;
        port in touchEvent;
    }

    part def NavigationController {
        satisfy requirement REQ-NAV-001;
        satisfy requirement REQ-CORE-002;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "UserInterfaceSystem")?;

        // 1. Package-level satisfy requirement validation
        assert_eq!(
            pkg.satisfied_requirements,
            vec!["REQ-UI-00".to_string()],
            "package-level satisfy requirement REQ-UI-00 must be captured"
        );

        // 2. Part-level satisfy requirement validation (including 'by <Part>' syntax)
        let touch_panel = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "TouchScreenPanel")
            .ok_or("missing TouchScreenPanel")?;
        assert_eq!(
            touch_panel.satisfied_requirements,
            vec!["REQ-UI-01".to_string(), "REQ-UI-02".to_string()],
            "TouchScreenPanel must capture REQ-UI-01 and REQ-UI-02"
        );

        // 3. Multi-digit zero-padded hyphenated requirement IDs
        let nav_ctrl = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "NavigationController")
            .ok_or("missing NavigationController")?;
        assert_eq!(
            nav_ctrl.satisfied_requirements,
            vec!["REQ-NAV-001".to_string(), "REQ-CORE-002".to_string()],
            "zero-padded hyphenated requirement IDs must be preserved exactly"
        );

        // 4. Serialization round-trip validation via SysmlSerializable::to_sysml
        let serialized = pkg.to_sysml(0);
        assert!(
            serialized.contains("satisfy requirement REQ-UI-00;"),
            "serialized sysml must contain package-level satisfy requirement REQ-UI-00:\n{}",
            serialized
        );
        assert!(
            serialized.contains("satisfy requirement REQ-UI-01;"),
            "serialized sysml must contain part-level satisfy requirement REQ-UI-01:\n{}",
            serialized
        );
        assert!(
            serialized.contains("satisfy requirement REQ-UI-02;"),
            "serialized sysml must contain part-level satisfy requirement REQ-UI-02:\n{}",
            serialized
        );
        assert!(
            serialized.contains("satisfy requirement REQ-NAV-001;"),
            "serialized sysml must contain REQ-NAV-001:\n{}",
            serialized
        );
        assert!(
            serialized.contains("satisfy requirement REQ-CORE-002;"),
            "serialized sysml must contain REQ-CORE-002:\n{}",
            serialized
        );

        // 5. Re-parse serialized output and assert AST idempotency
        let reparsed = SysmlParser::parse_source(&serialized, "UserInterfaceSystem")?;
        assert_eq!(
            reparsed.satisfied_requirements, pkg.satisfied_requirements,
            "reparsed package satisfied_requirements must match original"
        );

        let reparsed_touch = reparsed
            .part_defs
            .iter()
            .find(|p| p.name == "TouchScreenPanel")
            .ok_or("missing reparsed TouchScreenPanel")?;
        assert_eq!(
            reparsed_touch.satisfied_requirements, touch_panel.satisfied_requirements,
            "reparsed part satisfied_requirements must match original"
        );

        let reparsed_nav = reparsed
            .part_defs
            .iter()
            .find(|p| p.name == "NavigationController")
            .ok_or("missing reparsed NavigationController")?;
        assert_eq!(
            reparsed_nav.satisfied_requirements, nav_ctrl.satisfied_requirements,
            "reparsed nav controller satisfied_requirements must match original"
        );

        Ok(())
    }

    #[test]
    fn test_issue_460_parse_satisfy_syntax_variants_and_deduplication() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package SyntaxVariantsModel {
    part def DisplaySubsystem {
        // Syntax variant: without 'requirement' keyword
        satisfy REQ-UI-01;
        // Duplicate satisfy statement should be deduplicated
        satisfy requirement REQ-UI-01;
        // Quoted requirement identifier
        satisfy requirement "REQ-UI-02";
        // Hyphenated ID with multiple segments and zero-padding
        satisfy requirement REQ-FMS-DISP-0042;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "SyntaxVariantsModel")?;
        let display = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "DisplaySubsystem")
            .ok_or("missing DisplaySubsystem")?;

        assert_eq!(
            display.satisfied_requirements,
            vec![
                "REQ-UI-01".to_string(),
                "REQ-UI-02".to_string(),
                "REQ-FMS-DISP-0042".to_string(),
            ],
            "syntax variants and zero-padded multi-hyphen identifiers must be parsed with duplicate deduplication"
        );
        Ok(())
    }

    #[test]
    fn test_issue_460_parse_satisfy_with_interspersed_block_comments() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package CommentVariantsModel {
    part def CommentSubsystem {
        // Block comment between satisfy and requirement
        satisfy /* comment 1 */ requirement REQ-UI-01;
        // Block comment between satisfy and ID (no requirement keyword)
        satisfy /* comment 2 */ REQ-UI-02;
        // Block comment between requirement and ID
        satisfy requirement /* comment 3 */ REQ-UI-03;
        // Block comment before semicolon
        satisfy requirement REQ-UI-04 /* comment 4 */;
        // Multiple block comments interspersed
        satisfy /* c1 */ /* c2 */ requirement /* c3 */ REQ-UI-05 /* c4 */;
        // Block comment with 'by' clause
        satisfy requirement REQ-UI-06 /* comment 5 */ by SubsystemImpl;
        // Qualified path
        satisfy requirement Core::UI::REQ-07;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "CommentVariantsModel")?;
        let subsystem = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "CommentSubsystem")
            .ok_or("missing CommentSubsystem")?;

        assert_eq!(
            subsystem.satisfied_requirements,
            vec![
                "REQ-UI-01".to_string(),
                "REQ-UI-02".to_string(),
                "REQ-UI-03".to_string(),
                "REQ-UI-04".to_string(),
                "REQ-UI-05".to_string(),
                "REQ-UI-06".to_string(),
                "Core::UI::REQ-07".to_string(),
            ],
            "interspersed block comments must not leak <dynamic> or prevent requirement keyword recognition"
        );
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_between_satisfy_and_requirement() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy // single-line comment\n requirement REQ-01;", "REQ-01"),
            ("satisfy /* single-line block comment */ requirement REQ-02;", "REQ-02"),
            ("satisfy /* multi\n   line\n   block comment */ requirement REQ-03;", "REQ-03"),
            ("satisfy /* nested /* block */ comment */ requirement REQ-04;", "REQ-04"),
            ("satisfy doc /* doc comment style */ requirement REQ-05;", "REQ-05"),
            ("satisfy /* doc: comment style */ requirement REQ-06;", "REQ-06"),
            ("satisfy /* c1 */ /* c2 */ requirement REQ-07;", "REQ-07"),
            ("satisfy /* c1 */ // line\n /* c2 */ requirement REQ-08;", "REQ-08"),
            ("satisfy /* satisfy requirement by */ requirement REQ-09;", "REQ-09"),
            ("satisfy /**/ requirement REQ-10;", "REQ-10"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs[0].satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comment between satisfy and requirement: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_between_requirement_and_identifier() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement // line comment\n REQ-01;", "REQ-01"),
            ("satisfy requirement /* single block */ REQ-02;", "REQ-02"),
            ("satisfy requirement /* multi\nline\nblock */ REQ-03;", "REQ-03"),
            ("satisfy requirement doc /* doc comment */ REQ-04;", "REQ-04"),
            ("satisfy requirement /* doc: docstring */ REQ-05;", "REQ-05"),
            ("satisfy requirement /* c1 */ /* c2 */ REQ-06;", "REQ-06"),
            ("satisfy requirement /* c1 */ // line\n /* c2 */ REQ-07;", "REQ-07"),
            // Direct satisfy without 'requirement' keyword
            ("satisfy /* comment without req keyword */ REQ-08;", "REQ-08"),
            ("satisfy // line without req keyword\n REQ-09;", "REQ-09"),
            ("satisfy /* multi\nline without req */ REQ-10;", "REQ-10"),
            // Quoted and unrestricted identifiers with comments
            ("satisfy requirement /* c1 */ \"REQ-UI-01\";", "REQ-UI-01"),
            ("satisfy requirement /* c1 */ 'REQ-UI-02';", "REQ-UI-02"),
            ("satisfy /* c1 */ \"REQ-UI-03\";", "REQ-UI-03"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs[0].satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comment between requirement and ID: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_within_identifier_tokens_and_hyphens() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement REQ /* c */ -01;", "REQ-01"),
            ("satisfy requirement REQ- /* c */ 01;", "REQ-01"),
            ("satisfy requirement REQ /* c1 */ - /* c2 */ UI /* c3 */ - /* c4 */ 01;", "REQ-UI-01"),
            ("satisfy requirement REQ- // line comment\n 01;", "REQ-01"),
            ("satisfy requirement REQ /* multi\nline */ -01;", "REQ-01"),
            ("satisfy requirement REQ /* nested /* c */ */ -01;", "REQ-01"),
            ("satisfy requirement REQ /* c1 */ - /* c2 */ FMS /* c3 */ - /* c4 */ DISP /* c5 */ - /* c6 */ 0042;", "REQ-FMS-DISP-0042"),
            ("satisfy requirement Core /* c1 */ ::UI::REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core:: /* c2 */ UI::REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core::UI /* c3 */ ::REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core::UI:: /* c4 */ REQ-01;", "Core::UI::REQ-01"),
            ("satisfy requirement Core /* c1 */ :: /* c2 */ UI /* c3 */ :: /* c4 */ REQ /* c5 */ - /* c6 */ 01;", "Core::UI::REQ-01"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs[0].satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comments within identifier tokens: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_before_terminating_semicolon() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement REQ-01 // line comment\n ;", "REQ-01"),
            ("satisfy requirement REQ-02 /* block comment */;", "REQ-02"),
            ("satisfy requirement REQ-03 /* multi\nline\ncomment */;", "REQ-03"),
            ("satisfy requirement REQ-04 /* nested /* comment */ */;", "REQ-04"),
            ("satisfy requirement REQ-05 /* c1 */ /* c2 */;", "REQ-05"),
            ("satisfy requirement REQ-06 /* c1 */ // line\n /* c2 */;", "REQ-06"),
            ("satisfy requirement REQ-07 /* ;;; */;", "REQ-07"),
            ("satisfy requirement REQ-08 /* { } */;", "REQ-08"),
            ("satisfy requirement \"REQ-09\" /* block comment */;", "REQ-09"),
            ("satisfy requirement 'REQ-10' /* block comment */;", "REQ-10"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs[0].satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comments before terminating semicolon: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_surrounding_by_clause() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            ("satisfy requirement REQ-01 /* comment before by */ by NavCtrl;", "REQ-01"),
            ("satisfy requirement REQ-02 // line before by\n by NavCtrl;", "REQ-02"),
            ("satisfy requirement REQ-03 /* multi\nline */ by NavCtrl;", "REQ-03"),
            ("satisfy requirement REQ-04 by /* comment between by and part */ NavCtrl;", "REQ-04"),
            ("satisfy requirement REQ-05 by // line between by and part\n NavCtrl;", "REQ-05"),
            ("satisfy requirement REQ-06 by /* multi\nline */ NavCtrl;", "REQ-06"),
            ("satisfy requirement REQ-07 by NavCtrl /* comment after part before semi */;", "REQ-07"),
            ("satisfy requirement REQ-08 by NavCtrl // line after part before semi\n ;", "REQ-08"),
            ("satisfy requirement REQ-09 by NavCtrl /* multi\nline */;", "REQ-09"),
            ("satisfy requirement REQ-10 /* c1 */ by /* c2 */ NavCtrl /* c3 */;", "REQ-10"),
            ("satisfy requirement REQ-11 // l1\n by // l2\n NavCtrl // l3\n ;", "REQ-11"),
            ("satisfy requirement REQ-12 /* by */ by /* satisfy */ NavCtrl /* ; */;", "REQ-12"),
            ("satisfy requirement \"REQ-UI-01\" /* c1 */ by /* c2 */ NavCtrl /* c3 */;", "REQ-UI-01"),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs[0].satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comments surrounding 'by <Part>' clause: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_package_and_part_def_roundtrip_stress() -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let source = r#"
package AdversarialCommentsPackage {
    // Package-level satisfy statement with inline block comment
    satisfy /* pkg comment 1 */ requirement REQ-PKG-01 /* pkg comment 2 */;

    // Package-level satisfy statement with 'by' clause and comments
    satisfy requirement REQ-PKG-02 /* before by */ by CorePlatform /* after by */;

    part def CockpitSubsystem {
        // Position 1: between satisfy and requirement
        satisfy /* block comment */ requirement REQ-UI-01;
        satisfy // line comment
            requirement REQ-UI-02;
        satisfy /* multi
            line
            comment */ requirement REQ-UI-03;

        // Position 2: between requirement and ID
        satisfy requirement /* block comment */ REQ-UI-04;
        satisfy /* without req keyword */ REQ-UI-05;

        // Position 3: between ID tokens and hyphens
        satisfy requirement REQ /* c1 */ - /* c2 */ UI /* c3 */ - /* c4 */ 06;
        satisfy requirement Core /* c1 */ :: /* c2 */ UI /* c3 */ :: /* c4 */ REQ-UI-07;

        // Position 4: before terminating semicolon
        satisfy requirement REQ-UI-08 /* comment before semi */;

        // Position 5: before and after 'by <Part>'
        satisfy requirement REQ-UI-09 /* c1 */ by /* c2 */ TouchDisplay /* c3 */;

        // Deduplication: statement with comments matches clean statement
        satisfy requirement REQ-UI-01;
        satisfy /* dupe comment */ requirement REQ-UI-01;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "AdversarialCommentsPackage")?;

        // 1. Package-level satisfy requirement validation
        assert_eq!(
            pkg.satisfied_requirements,
            vec!["REQ-PKG-01".to_string(), "REQ-PKG-02".to_string()],
            "package-level satisfy requirements must be extracted cleanly without comment artifacts"
        );

        // 2. Part-level satisfy requirement validation and deduplication
        let cockpit = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "CockpitSubsystem")
            .ok_or("missing CockpitSubsystem")?;

        let expected_part_reqs = vec![
            "REQ-UI-01".to_string(),
            "REQ-UI-02".to_string(),
            "REQ-UI-03".to_string(),
            "REQ-UI-04".to_string(),
            "REQ-UI-05".to_string(),
            "REQ-UI-06".to_string(),
            "Core::UI::REQ-UI-07".to_string(),
            "REQ-UI-08".to_string(),
            "REQ-UI-09".to_string(),
        ];
        assert_eq!(
            cockpit.satisfied_requirements, expected_part_reqs,
            "cockpit satisfied_requirements must match expected list with deduplication"
        );

        // 3. Serialization round-trip validation via SysmlSerializable
        let serialized = pkg.to_sysml(0);
        assert!(serialized.contains("satisfy requirement REQ-PKG-01;"));
        assert!(serialized.contains("satisfy requirement REQ-PKG-02;"));
        assert!(serialized.contains("satisfy requirement REQ-UI-01;"));
        assert!(serialized.contains("satisfy requirement REQ-UI-06;"));
        assert!(serialized.contains("satisfy requirement Core::UI::REQ-UI-07;"));

        // 4. AST Idempotency after re-parsing serialized output
        let reparsed = SysmlParser::parse_source(&serialized, "AdversarialCommentsPackage")?;
        assert_eq!(reparsed.satisfied_requirements, pkg.satisfied_requirements);
        let reparsed_cockpit = reparsed
            .part_defs
            .iter()
            .find(|p| p.name == "CockpitSubsystem")
            .ok_or("missing reparsed CockpitSubsystem")?;
        assert_eq!(reparsed_cockpit.satisfied_requirements, cockpit.satisfied_requirements);

        Ok(())
    }
}



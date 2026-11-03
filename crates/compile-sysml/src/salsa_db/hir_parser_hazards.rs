#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Safety hazard, risk, verification test-case, operation, and interaction declaration parsers for SysML v2 / KerML.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE] | [FLS: fls_66m4rnbssgig]
//! Contract: Parses `hazard` / `hazard def`, `risk` / `risk def`, `test case` / `test case def`, `operation`, and `interaction` blocks into deterministic AST nodes.
//! Bounds & Faults: Executes linear token consumption with zero direct slice indexing and fails closed with span-annotated `ParseError` diagnostics on unrecognized body statements.

use super::cst_db::TokenKind;
use super::hir_parser_core::{ParseError, SysmlParser};
use deap_core::sysml_cst::{
    AttributeDef, HazardDef, InteractionDef, OperationDef, RiskDef, TestCaseDef,
};

impl<'a> SysmlParser<'a> {
    /// Parses a `hazard` or `hazard def` declaration and its severity and port/part attributes into a `HazardDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-HAZARD-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_hazard_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<HazardDef, ParseError> {
        self.expect(&TokenKind::Hazard)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut hazard_doc = doc;
        let mut severity = 1;
        let mut source_port = None;
        let mut target_port = None;
        let mut part_ref = None;

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.drain_first_inner_doc(&mut hazard_doc);
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(None)?;
                    let unquoted = attr.default_value.map(|v| v.trim().trim_matches('"').to_string());
                    match attr.name.as_str() {
                        "severity" => {
                            severity = unquoted.and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);
                        }
                        "source_port" => source_port = unquoted,
                        "target_port" => target_port = unquoted,
                        "part_ref" => part_ref = unquoted,
                        _ => {}
                    }
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!("Unexpected token '{:?}' in hazard declaration body", tok.kind),
                        span: tok.span,
                    });
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(HazardDef {
            name,
            doc: hazard_doc,
            severity,
            source_port,
            target_port,
            part_ref,
        })
    }

    /// Parses a `risk` or `risk def` declaration and its severity, hazard reference, and port attributes into a `RiskDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-RISK-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_risk_decl(&mut self, doc: Option<String>) -> Result<RiskDef, ParseError> {
        self.expect(&TokenKind::Risk)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut risk_doc = doc;
        let mut severity = 1;
        let mut hazard_ref = None;
        let mut source_port = None;
        let mut target_port = None;

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.drain_first_inner_doc(&mut risk_doc);
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.check(&TokenKind::Attribute) {
                    let attr = self.parse_attribute_decl(None)?;
                    let unquoted = attr.default_value.map(|v| v.trim().trim_matches('"').to_string());
                    match attr.name.as_str() {
                        "severity" => {
                            severity = unquoted.and_then(|v| v.parse::<i32>().ok()).unwrap_or(1);
                        }
                        "hazard_ref" => hazard_ref = unquoted,
                        "source_port" => source_port = unquoted,
                        "target_port" => target_port = unquoted,
                        _ => {}
                    }
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!("Unexpected token '{:?}' in risk declaration body", tok.kind),
                        span: tok.span,
                    });
                }
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(RiskDef {
            name,
            doc: risk_doc,
            severity,
            hazard_ref,
            source_port,
            target_port,
        })
    }

    /// Parses a `test case` or `test case def` declaration and its `subject`, `verify`, `objective`, and `step` clauses into a `TestCaseDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TEST-CASE-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_test_case_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<TestCaseDef, ParseError> {
        self.expect(&TokenKind::Test)?;
        if self.check(&TokenKind::Case) {
            self.advance();
        }
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut tc_doc = doc;
        let mut subject_part = None;
        let mut verified_requirements = Vec::new();
        let mut objective = None;
        let mut test_steps = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.drain_first_inner_doc(&mut tc_doc);
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
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
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!("Unexpected token '{:?}' in test case declaration body", tok.kind),
                        span: tok.span,
                    });
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
            doc: tc_doc,
        })
    }

    /// Parses an `operation` or `operation def` signature with typed `in`/`out`/`inout` parameters and optional return type into an `OperationDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-OPERATION-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_operation_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<OperationDef, ParseError> {
        self.expect(&TokenKind::Operation)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut return_type = None;
        let mut parameters = Vec::new();

        if self.eat(&TokenKind::OpenParen) {
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                let dir = if self.eat(&TokenKind::In) {
                    "in"
                } else if self.eat(&TokenKind::Out) {
                    "out"
                } else if self.eat(&TokenKind::Inout) {
                    "inout"
                } else {
                    "in"
                }
                .to_string();
                let p_name = self.expect_ident()?;
                let p_type = if self.eat(&TokenKind::Colon) {
                    self.parse_type_expr()?
                } else {
                    "String".to_string()
                };
                parameters.push(AttributeDef {
                    name: p_name,
                    type_name: p_type,
                    default_value: Some(dir),
                    doc: None,
                });
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::CloseParen)?;
        }
        if self.eat(&TokenKind::Colon) {
            return_type = Some(self.parse_type_expr()?);
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

    /// Parses an `interaction` or `interaction def` block containing `lifeline`, `message`, and `trigger` statements into an `InteractionDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-INTERACTION-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_interaction_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<InteractionDef, ParseError> {
        self.expect(&TokenKind::Interaction)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut inter_doc = doc;
        let mut lifelines = Vec::new();
        let mut messages = Vec::new();
        let mut triggers = Vec::new();

        if self.eat(&TokenKind::OpenBrace) {
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                while let Some(d) = self.take_inner_doc_comment() {
                    if inter_doc.is_none() {
                        inter_doc = Some(d);
                    }
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.eat(&TokenKind::Lifeline) {
                    lifelines.push(self.read_until_semi());
                } else if self.eat(&TokenKind::Message) {
                    messages.push(self.read_until_semi());
                } else if self.eat(&TokenKind::Trigger) {
                    triggers.push(self.read_until_semi());
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!("Unexpected token '{:?}' in interaction declaration body", tok.kind),
                        span: tok.span,
                    });
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
            doc: inter_doc,
        })
    }
}

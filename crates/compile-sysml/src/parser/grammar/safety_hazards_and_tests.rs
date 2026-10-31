//! Safety hazard, risk, and verification test-case declaration parsers for SysML v2 / KerML.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses `hazard` / `hazard def`, `risk` / `risk def`, and `test case` / `test case def` blocks into deterministic `HazardDef`, `RiskDef`, and `TestCaseDef` AST nodes.
//! Bounds & Faults: Executes linear token consumption with zero direct slice indexing and fails closed with span-annotated `ParseError` diagnostics on unrecognized body statements.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Parses a `hazard` or `hazard def` declaration and its severity and port/part attributes into a `HazardDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-HAZARD-DECL]
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
                    let unquoted = attr
                        .default_value
                        .map(|v| v.trim().trim_matches('"').to_string());
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
                        message: format!(
                            "Unexpected token '{:?}' in hazard declaration body",
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-RISK-DECL]
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
                    let unquoted = attr
                        .default_value
                        .map(|v| v.trim().trim_matches('"').to_string());
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
                        message: format!(
                            "Unexpected token '{:?}' in risk declaration body",
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TEST-CASE-DECL]
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
                        message: format!(
                            "Unexpected token '{:?}' in test case declaration body",
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

        Ok(TestCaseDef {
            name,
            subject_part,
            verified_requirements,
            objective,
            test_steps,
            doc: tc_doc,
        })
    }
}

//! Constraint, requirement, and requirement-satisfaction parsers for SysML v2 / KerML.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses `constraint` / `assert constraint`, `requirement` / `requirement def`, and `satisfy` declarations into deterministic `ConstraintDef`, `RequirementDef`, and satisfaction target AST nodes.
//! Bounds & Faults: Executes linear token consumption with saturating span arithmetic and zero direct slice indexing, failing closed with span-annotated `ParseError` diagnostics on unrecognized body statements.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Advances past leading trivia and captures the first inner doc comment into `slot` when unset.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-TAKE-INNER-DOC-COMMENT]
    pub(super) fn drain_first_inner_doc(&mut self, slot: &mut Option<String>) {
        self.skip_trivia();
        while let Some(d) = self.take_inner_doc_comment() {
            if slot.is_none() {
                *slot = Some(d);
            }
            self.skip_trivia();
        }
    }

    /// Parses parenthesized `(in/out [attribute|item] name [: Type], ...)` constraint parameters into `parameters`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONSTRAINT-DECL]
    fn parse_constraint_header_params(
        &mut self,
        parameters: &mut Vec<String>,
    ) -> Result<(), ParseError> {
        self.advance();
        while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
            self.skip_trivia();
            if self.check(&TokenKind::CloseParen) {
                break;
            }
            let mut prefix = String::new();
            if self.check(&TokenKind::In) {
                self.advance();
                prefix.push_str("in ");
            } else if self.check(&TokenKind::Out) {
                self.advance();
                prefix.push_str("out ");
            } else if self.check(&TokenKind::Inout) {
                self.advance();
                prefix.push_str("inout ");
            }
            if self.check(&TokenKind::Attribute) {
                self.advance();
                prefix.push_str("attribute ");
            } else if self.check(&TokenKind::Item) {
                self.advance();
                prefix.push_str("item ");
            }
            let p_name = self.expect_ident()?;
            let param_formatted =
                if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
                    self.advance();
                    let p_type = self.parse_type_expr()?;
                    format!("{prefix}{p_name}: {p_type}")
                } else {
                    format!("{prefix}{p_name}")
                };
            parameters.push(param_formatted);
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&TokenKind::CloseParen)?;
        Ok(())
    }

    /// Parses a `constraint` / `constraint def` or `assert constraint` declaration into a `ConstraintDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONSTRAINT-DECL]
    pub(crate) fn parse_constraint_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<ConstraintDef, ParseError> {
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
            self.parse_constraint_header_params(&mut parameters)?;
        }

        let mut expression = String::new();
        let mut final_doc = doc;
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.drain_first_inner_doc(&mut final_doc);
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
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

    /// Applies `@TriggersHazard`, `@SafetyRealises`, and `@Layer` stereotype values onto `req`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-REQUIREMENT-DECL]
    fn apply_requirement_stereotype(
        req: &mut RequirementDef,
        stereo_name: &str,
        values: Vec<String>,
    ) {
        if stereo_name == "TriggersHazard" {
            for val in values {
                if !val.is_empty() && !req.triggers_hazards.contains(&val) {
                    req.triggers_hazards.push(val);
                }
            }
        } else if stereo_name == "SafetyRealises" {
            for val in values {
                if !val.is_empty() && !req.safety_realises.contains(&val) {
                    req.safety_realises.push(val);
                }
            }
        } else if stereo_name == "Layer" {
            if let Some(first_val) = values.into_iter().find(|v| !v.is_empty()) {
                req.layer = Some(first_val);
            }
        }
    }

    /// Parses a single body statement inside a `requirement` or `requirement def` block and populates `req`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-REQUIREMENT-DECL]
    fn parse_requirement_member(
        &mut self,
        req: &mut RequirementDef,
        sub_doc: Option<String>,
    ) -> Result<(), ParseError> {
        if self.check(&TokenKind::Id) {
            self.advance();
            if self.check(&TokenKind::Eq) {
                self.advance();
            }
            req.req_id = self.expect_string_or_ident()?;
            self.expect_semi()?;
        } else if self.check(&TokenKind::Text) || self.check(&TokenKind::Doc) {
            self.advance();
            if self.check(&TokenKind::Eq) {
                self.advance();
            }
            req.text = self.expect_string_or_ident()?;
            self.expect_semi()?;
        } else if self.check(&TokenKind::Attribute) {
            req.attributes.push(self.parse_attribute_decl(sub_doc)?);
        } else if self.check(&TokenKind::Constraint) || self.check(&TokenKind::Assert) {
            req.constraints.push(self.parse_constraint_decl(sub_doc)?);
        } else if self.peek_stereotype_name().is_some() {
            let (stereo_name, values) = self.parse_stereotype_annotation()?;
            Self::apply_requirement_stereotype(req, &stereo_name, values);
        } else if self.check(&TokenKind::Assume) {
            self.advance();
            req.assumes.push(self.read_until_semi());
        } else if self.check(&TokenKind::Require) {
            self.advance();
            req.requires.push(self.read_until_semi());
        } else if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "derived" || s == "derive")
        {
            self.advance();
            if self.check(&TokenKind::Requirement) {
                self.advance();
            }
            if self.check(&TokenKind::From) {
                self.advance();
            }
            req.derived_from
                .push(self.read_until_semi().replace(" :: ", "::"));
        } else if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "derives") {
            self.advance();
            if self.check(&TokenKind::Requirement) {
                self.advance();
            }
            req.derives
                .push(self.read_until_semi().replace(" :: ", "::"));
        } else if self.check(&TokenKind::Verify) {
            self.advance();
            if self.check(&TokenKind::By) {
                self.advance();
            }
            req.verified_by
                .push(self.read_until_semi().replace(" :: ", "::"));
        } else if self.check(&TokenKind::Satisfy) {
            self.advance();
            if self.check(&TokenKind::By) {
                self.advance();
            }
            req.satisfied_by
                .push(self.read_until_semi().replace(" :: ", "::"));
        } else if self.check(&TokenKind::Subject) {
            self.advance();
            self.read_until_semi();
        } else {
            let tok = self.peek();
            return Err(ParseError {
                message: format!(
                    "Unexpected token '{:?}' in requirement declaration body",
                    tok.kind
                ),
                span: tok.span,
            });
        }
        Ok(())
    }

    /// Parses a `requirement` or `requirement def` declaration and its inner traceability, attribute, and constraint clauses into a `RequirementDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-REQUIREMENT-DECL]
    pub(crate) fn parse_requirement_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<RequirementDef, ParseError> {
        self.expect(&TokenKind::Requirement)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }

        let name = self.expect_ident()?;
        let mut req = RequirementDef {
            name,
            doc,
            ..Default::default()
        };

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                let mut sub_doc = self.take_inner_doc_comment();
                self.skip_trivia();
                while let Some(next_doc) = self.take_inner_doc_comment() {
                    if let Some(prev_d) = sub_doc {
                        if req.text.is_empty() {
                            req.text = prev_d;
                        }
                    }
                    sub_doc = Some(next_doc);
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    if let Some(d) = sub_doc {
                        if req.text.is_empty() {
                            req.text = d;
                        }
                    }
                    break;
                }
                if !self.check(&TokenKind::Attribute)
                    && !self.check(&TokenKind::Constraint)
                    && !self.check(&TokenKind::Assert)
                {
                    if let Some(d) = sub_doc.take() {
                        if req.text.is_empty() {
                            req.text = d;
                        }
                    }
                }
                self.parse_requirement_member(&mut req, sub_doc)?;
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }

        Ok(req)
    }

    /// Parses a `satisfy [requirement] <TargetId> [by <Part>];` statement preserving zero-padded hyphenated identifiers across interspersed comments.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-SATISFY-STMT]
    pub(crate) fn parse_satisfy_stmt(&mut self) -> Result<String, ParseError> {
        self.expect(&TokenKind::Satisfy)?;
        while matches!(
            self.peek_kind(),
            TokenKind::DocComment(_) | TokenKind::LineComment(_)
        ) {
            self.advance();
        }
        if self.check(&TokenKind::Requirement) {
            self.advance();
        }
        while matches!(
            self.peek_kind(),
            TokenKind::DocComment(_) | TokenKind::LineComment(_)
        ) {
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
        Ok(raw
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim()
            .to_string())
    }
}

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Use case and calculation (`calc` / `calc def`) declaration parsers for SysML v2 / KerML.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE] | [FLS: fls_66m4rnbssgig]
//! Contract: Parses `use case` / `use case def` and `calc` / `calc def` declarations (including typed, bare-type, and expression `return` clauses) into deterministic AST nodes.
//! Bounds & Faults: Bounded linear token consumption with backtracking cursor restoration on ambiguous `calc` return statements and span-annotated `ParseError` diagnostics on invalid body tokens.

use super::cst_db::TokenKind;
use super::hir_parser_core::{ParseError, SysmlParser};
use deap_core::sysml_cst::{AttributeDef, CalcDef, UseCaseDef};

impl<'a> SysmlParser<'a> {
    /// Parses a `use case` or `use case def` declaration and its `subject`, `actor`, `objective`, `step`, `include`, `extend`, `precondition`, and `postcondition` clauses.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-USE-CASE-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_use_case_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<UseCaseDef, ParseError> {
        self.expect(&TokenKind::UseCase)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut uc = UseCaseDef { name, doc, ..Default::default() };
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                while let Some(inner_text) = self.take_inner_doc_comment() {
                    if !inner_text.trim().is_empty() {
                        let slot = uc.doc.get_or_insert_with(String::new);
                        if !slot.trim().is_empty() {
                            slot.push_str("\n\n");
                        }
                        slot.push_str(&inner_text);
                    }
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                self.parse_use_case_member(&mut uc)?;
            }
            self.expect(&TokenKind::CloseBrace)?;
        } else {
            self.expect_semi()?;
        }
        Ok(uc)
    }

    /// Parses a single body statement inside a `use case` or `use case def` block and populates `uc`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-USE-CASE-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_use_case_member(&mut self, uc: &mut UseCaseDef) -> Result<(), ParseError> {
        if self.check(&TokenKind::Subject) {
            self.advance();
            uc.subject = Some(self.read_until_semi());
        } else if self.check(&TokenKind::Actor) {
            self.advance();
            let act = self.read_until_semi();
            if uc.actor.is_none() {
                uc.actor = Some(act.clone());
            }
            uc.actors.push(act);
        } else if self.check(&TokenKind::Objective) {
            self.advance();
            uc.objective = Some(self.read_until_semi());
        } else if self.check(&TokenKind::Step) {
            self.advance();
            uc.steps.push(self.read_until_semi());
        } else if self.check(&TokenKind::Include) {
            self.advance();
            if self.check(&TokenKind::UseCase) {
                self.advance();
            }
            uc.includes.push(self.read_until_semi());
        } else if self.check(&TokenKind::Extend) {
            self.advance();
            if self.check(&TokenKind::UseCase) {
                self.advance();
            }
            uc.extends.push(self.read_until_semi());
        } else if self.check(&TokenKind::Precondition) {
            self.advance();
            uc.preconditions.push(self.read_until_semi());
        } else if self.check(&TokenKind::Postcondition) {
            self.advance();
            uc.postconditions.push(self.read_until_semi());
        } else {
            let tok = self.peek();
            return Err(ParseError {
                message: format!("Unexpected token '{:?}' in use case declaration body", tok.kind),
                span: tok.span,
            });
        }
        Ok(())
    }

    /// Parses parenthesized `(in/out name : Type = default, ...)` calculation header parameters into `in_params`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CALC-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_calc_header_params(
        &mut self,
        in_params: &mut Vec<AttributeDef>,
    ) -> Result<(), ParseError> {
        self.advance();
        while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
            if self.check(&TokenKind::In)
                || self.check(&TokenKind::Out)
                || self.check(&TokenKind::Inout)
            {
                self.advance();
            }
            if self.check(&TokenKind::Attribute) || self.check(&TokenKind::Item) {
                self.advance();
            }
            let p_name = self.expect_ident()?;
            let p_type = if self.check(&TokenKind::Colon) {
                self.advance();
                self.parse_type_expr()?
            } else {
                "Real".to_string()
            };
            let mut default_value = None;
            if self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq) {
                self.advance();
                let mut pieces = Vec::new();
                while !self.check(&TokenKind::Comma)
                    && !self.check(&TokenKind::CloseParen)
                    && !self.is_at_end()
                {
                    let tok = self.advance();
                    match &tok.kind {
                        TokenKind::Ident(s) => pieces.push(s.clone()),
                        TokenKind::StringLit(s) => pieces.push(format!("\"{}\"", s)),
                        TokenKind::IntLit(n) => pieces.push(n.to_string()),
                        TokenKind::RealLit(f) if f.fract() == 0.0 => pieces.push(format!("{:.1}", f)),
                        TokenKind::RealLit(f) => pieces.push(f.to_string()),
                        TokenKind::BoolLit(b) => pieces.push(b.to_string()),
                        _ => pieces.push(tok.kind.as_str().to_string()),
                    }
                }
                default_value = Some(pieces.join(" "));
            }
            in_params.push(AttributeDef { name: p_name, type_name: p_type, default_value, doc: None });
            self.eat(&TokenKind::Comma);
        }
        self.expect(&TokenKind::CloseParen)?;
        Ok(())
    }

    /// Parses a parameter or `attribute` declaration statement inside a `calc` or `calc def` body into `in_params`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CALC-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_calc_body_param(
        &mut self,
        in_params: &mut Vec<AttributeDef>,
    ) -> Result<(), ParseError> {
        self.advance();
        if self.check(&TokenKind::Attribute) || self.check(&TokenKind::Item) {
            self.advance();
        }
        let p_name = self.expect_ident()?;
        let p_type = if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
            self.advance();
            self.parse_type_expr()?
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
        Ok(())
    }

    /// Parses `<Type> [= <expr>];` following a typed `return` separator inside a `calc` body.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CALC-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_calc_typed_return_tail(
        &mut self,
        return_type: &mut Option<String>,
        expr_lines: &mut Vec<String>,
    ) -> Result<(), ParseError> {
        let rtype = self.parse_type_expr()?;
        if return_type.is_none() {
            *return_type = Some(rtype);
        }
        if self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq) {
            self.advance();
            let rhs = self.read_until_semi();
            let rhs_trimmed = rhs.trim();
            if !rhs_trimmed.is_empty() {
                expr_lines.push(rhs_trimmed.to_string());
            }
        } else {
            self.expect_semi()?;
        }
        Ok(())
    }

    /// Parses a `return` statement inside a `calc` body.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CALC-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_calc_return_stmt(
        &mut self,
        in_params: &[AttributeDef],
        return_type: &mut Option<String>,
        expr_lines: &mut Vec<String>,
    ) -> Result<(), ParseError> {
        self.advance();
        self.skip_trivia();
        if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
            self.advance();
            return self.parse_calc_typed_return_tail(return_type, expr_lines);
        }
        if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "type") {
            self.advance();
            if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
                self.advance();
            }
            return self.parse_calc_typed_return_tail(return_type, expr_lines);
        }

        let saved_pos = self.pos;
        if self.check(&TokenKind::In)
            || self.check(&TokenKind::Out)
            || self.check(&TokenKind::Inout)
        {
            self.advance();
        }
        if self.check(&TokenKind::Attribute) || self.check(&TokenKind::Item) {
            self.advance();
        }
        let has_ident = self.parse_identifier().is_ok();
        if has_ident && (self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt)) {
            self.advance();
            return self.parse_calc_typed_return_tail(return_type, expr_lines);
        }
        if has_ident && (self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq)) {
            self.advance();
            let rhs = self.read_until_semi();
            let rhs_trimmed = rhs.trim();
            if !rhs_trimmed.is_empty() {
                expr_lines.push(rhs_trimmed.to_string());
            }
            return Ok(());
        }

        self.pos = saved_pos;
        let starts_with_call = self.parse_identifier().is_ok() && self.check(&TokenKind::OpenParen);
        self.pos = saved_pos;
        let mut parsed_bare_type = false;
        if return_type.is_none() && !starts_with_call {
            if let Ok(rtype) = self.parse_type_expr() {
                if self.check(&TokenKind::Semi) && Self::is_bare_calc_return_type(&rtype, in_params)
                {
                    self.advance();
                    *return_type = Some(rtype);
                    parsed_bare_type = true;
                }
            }
        }
        if !parsed_bare_type {
            self.pos = saved_pos;
            let expr_str = self.read_until_semi();
            let expr_trimmed = expr_str.trim();
            if !expr_trimmed.is_empty() {
                expr_lines.push(expr_trimmed.to_string());
            }
        }
        Ok(())
    }

    /// Parses a `calc` or `calc def` declaration header, parameters, return type, and body expressions into a `CalcDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CALC-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_calc_decl(&mut self, doc: Option<String>) -> Result<CalcDef, ParseError> {
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
            self.parse_calc_header_params(&mut in_params)?;
        }
        if self.check(&TokenKind::Colon) {
            self.advance();
            return_type = Some(self.parse_type_expr()?);
        }
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                while let Some(d) = self.take_inner_doc_comment() {
                    if final_doc.is_none() {
                        final_doc = Some(d);
                    }
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.check(&TokenKind::In)
                    || self.check(&TokenKind::Out)
                    || self.check(&TokenKind::Inout)
                    || self.check(&TokenKind::Attribute)
                {
                    self.parse_calc_body_param(&mut in_params)?;
                } else if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "return") {
                    self.parse_calc_return_stmt(&in_params, &mut return_type, &mut expr_lines)?;
                } else {
                    let stmt = self.read_until_semi();
                    let trimmed = stmt.trim();
                    if !trimmed.is_empty() {
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

    /// Classifies whether a bare `return <candidate>;` inside a `calc def` body is a type declaration rather than a value expression.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-IS-BARE-CALC-RETURN-TYPE] | [FLS: fls_66m4rnbssgig]
    fn is_bare_calc_return_type(rtype: &str, in_params: &[AttributeDef]) -> bool {
        let trimmed = rtype.trim();
        let base = trimmed.split('[').next().unwrap_or(trimmed).trim();
        let is_param = in_params.iter().any(|p| p.name == base);
        let is_composite = trimmed.starts_with(['&', '*', '(', '[']);
        let is_indexed = !is_composite && trimmed.contains('[') && !trimmed.ends_with("[]");
        let is_primitive = matches!(
            base,
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                | "f32" | "f64" | "bool" | "str" | "char"
        );
        let is_named_type = base.starts_with(|c: char| c.is_ascii_uppercase())
            || base.contains("::")
            || is_primitive;
        !is_param && !is_indexed && (is_composite || is_named_type)
    }
}

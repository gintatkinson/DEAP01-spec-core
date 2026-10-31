//! Behavioral action, state machine, operation, and interaction declaration parsers for SysML v2 / KerML.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses `action` / `action def`, recursive `state` / `state def`, `operation` / `operation def`, and `interaction` / `interaction def` declarations into deterministic AST nodes.
//! Bounds & Faults: Enforces `MAX_PARSE_DEPTH` (64) on nested `state` hierarchies, uses checked `.get()` lookahead with saturating index arithmetic, and fails closed with span-annotated `ParseError` diagnostics on invalid body tokens.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Parses parenthesized `(in/out name : Type = default, ...)` action header parameters into `in_params` and `out_params`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-ACTION-HEADER-PARAMS]
    fn parse_action_header_params(
        &mut self,
        in_params: &mut Vec<AttributeDef>,
        out_params: &mut Vec<AttributeDef>,
    ) -> Result<(), ParseError> {
        self.expect(&TokenKind::OpenParen)?;
        while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
            let mut is_out = false;
            if self.check(&TokenKind::In) {
                self.advance();
            } else if self.check(&TokenKind::Out) || self.check(&TokenKind::Inout) {
                self.advance();
                is_out = true;
            }
            if self.check(&TokenKind::Attribute) || self.check(&TokenKind::Item) {
                self.advance();
            }
            let param_name = self.expect_ident()?;
            let param_type = if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
                self.advance();
                self.parse_type_expr()?
            } else {
                "Real".to_string()
            };
            let default_value = if self.check(&TokenKind::Eq) || self.check(&TokenKind::ColonEq) {
                self.advance();
                let mut pieces = Vec::new();
                while !self.check(&TokenKind::Comma)
                    && !self.check(&TokenKind::CloseParen)
                    && !self.is_at_end()
                {
                    let tok = self.advance();
                    let piece = match &tok.kind {
                        TokenKind::Ident(s) => s.clone(),
                        TokenKind::StringLit(s) => format!("\"{}\"", s),
                        TokenKind::IntLit(n) => n.to_string(),
                        TokenKind::RealLit(f) if f.fract() == 0.0 => format!("{:.1}", f),
                        TokenKind::RealLit(f) => f.to_string(),
                        TokenKind::BoolLit(b) => b.to_string(),
                        _ => tok.kind.as_str().to_string(),
                    };
                    pieces.push(piece);
                }
                Some(pieces.join(" "))
            } else {
                None
            };
            let param = AttributeDef {
                name: param_name,
                type_name: param_type,
                default_value,
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
        Ok(())
    }

    /// Parses a semicolon-terminated `in`/`out`/`inout` parameter statement inside an action declaration body.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-ACTION-BODY-PARAM]
    fn parse_action_body_param(
        &mut self,
        in_params: &mut Vec<AttributeDef>,
        out_params: &mut Vec<AttributeDef>,
    ) -> Result<(), ParseError> {
        let is_out = self.check(&TokenKind::Out) || self.check(&TokenKind::Inout);
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
            self.expect_semi()?;
            None
        };
        let p = AttributeDef {
            name: p_name,
            type_name: p_type,
            default_value,
            doc: None,
        };
        if is_out {
            out_params.push(p);
        } else {
            in_params.push(p);
        }
        Ok(())
    }

    /// Parses an `action` or `action def` declaration header, parameter lists, performer, and execution steps into an `ActionDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-ACTION-DECL]
    pub(crate) fn parse_action_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<ActionDef, ParseError> {
        self.expect(&TokenKind::Action)?;
        let is_def = self.check(&TokenKind::Def);
        if is_def {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut action_doc = doc;
        let mut in_params = Vec::new();
        let mut out_params = Vec::new();
        let mut parameters = Vec::new();
        let mut performer = None;
        let mut steps = Vec::new();

        if self.check(&TokenKind::OpenParen) {
            self.parse_action_header_params(&mut in_params, &mut out_params)?;
        }
        if self.check(&TokenKind::Colon) || self.check(&TokenKind::ColonGt) {
            self.advance();
            let _return_type = self.parse_type_expr()?;
        }

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                self.skip_trivia();
                while let Some(d) = self.take_inner_doc_comment() {
                    if action_doc.is_none() {
                        action_doc = Some(d);
                    }
                    self.skip_trivia();
                }
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.check(&TokenKind::Perform) {
                    self.advance();
                    performer = Some(self.read_until_semi());
                } else if self.check(&TokenKind::Step) {
                    self.advance();
                    steps.push(self.read_until_semi());
                } else if self.check(&TokenKind::Attribute) {
                    parameters.push(self.parse_attribute_decl(None)?);
                } else if self.check(&TokenKind::In)
                    || self.check(&TokenKind::Out)
                    || self.check(&TokenKind::Inout)
                {
                    self.parse_action_body_param(&mut in_params, &mut out_params)?;
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!(
                            "Unexpected token '{:?}' in action declaration body",
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

        Ok(ActionDef {
            name,
            doc: action_doc,
            in_params,
            out_params,
            parameters,
            performer,
            steps,
            is_def,
        })
    }

    /// Collects inner documentation comments or `doc = "...";` property statements inside a `state` body.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-TAKE-STATE-MEMBER-DOC]
    fn take_state_member_doc(&mut self, state_doc: &mut Option<String>) -> Option<String> {
        self.skip_trivia();
        let mut inner_doc = None;
        while self.check(&TokenKind::Doc) || matches!(self.peek_kind(), TokenKind::DocComment(_)) {
            let is_prop_doc = self.check(&TokenKind::Doc)
                && !matches!(
                    self.tokens.get(self.pos.saturating_add(1)).map(|t| &t.kind),
                    Some(TokenKind::DocComment(_))
                );
            if let Some(next_doc) = self.take_inner_doc_comment() {
                if is_prop_doc {
                    if state_doc.is_none() {
                        *state_doc = Some(next_doc);
                    }
                } else {
                    if inner_doc.is_some() && state_doc.is_none() {
                        *state_doc = inner_doc;
                    }
                    inner_doc = Some(next_doc);
                }
            }
            self.skip_trivia();
        }
        if self.check(&TokenKind::CloseBrace)
            || self.is_at_end()
            || (!self.check(&TokenKind::State) && !self.check(&TokenKind::Transition))
        {
            if let Some(d) = inner_doc.take() {
                if state_doc.is_none() {
                    *state_doc = Some(d);
                }
            }
        }
        inner_doc
    }

    /// Parses an `entry`, `do`, or `exit` state action clause up to its terminating semicolon.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-STATE-ACTION-CLAUSE]
    fn parse_state_action_clause(&mut self) -> String {
        self.advance();
        let raw_tokens = self.read_tokens_until_semi();
        let tokens: Vec<Token> = raw_tokens
            .into_iter()
            .filter(|t| {
                !matches!(
                    t.kind,
                    TokenKind::Eof
                        | TokenKind::Semi
                        | TokenKind::LineComment(_)
                        | TokenKind::DocComment(_)
                )
            })
            .collect();
        let slice = tokens.as_slice();
        let action_slice = if slice.first().is_some_and(|t| t.kind == TokenKind::Action)
            && slice.get(1).is_some_and(|t| {
                !matches!(
                    t.kind,
                    TokenKind::OpenParen
                        | TokenKind::CloseParen
                        | TokenKind::OpenBracket
                        | TokenKind::CloseBracket
                        | TokenKind::Dot
                        | TokenKind::DoubleColon
                        | TokenKind::Comma
                        | TokenKind::Semi
                )
            }) {
            slice.get(1..).unwrap_or_default()
        } else {
            slice
        };
        Self::format_transition_expr(action_slice)
    }

    /// Parses a `state` or `state def` declaration and nested sub-states bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-STATE-DECL]
    pub(crate) fn parse_state_decl(&mut self, doc: Option<String>) -> Result<StateDef, ParseError> {
        if self.depth >= MAX_PARSE_DEPTH {
            return Err(self.max_depth_error());
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.parse_state_decl_inner(doc);
        self.depth = self.depth.saturating_sub(1);
        result
    }

    /// Parses the header, actions, transitions, and nested sub-states of a `state` block within an active depth frame.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-STATE-DECL]
    fn parse_state_decl_inner(&mut self, doc: Option<String>) -> Result<StateDef, ParseError> {
        self.expect(&TokenKind::State)?;
        if self.check(&TokenKind::Def) {
            self.advance();
        }
        let name = self.expect_ident()?;
        let mut state_doc = doc;
        let mut entry_action = None;
        let mut do_action = None;
        let mut exit_action = None;
        let mut transitions = Vec::new();
        let mut sub_states = Vec::new();

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                let inner_doc = self.take_state_member_doc(&mut state_doc);
                if self.check(&TokenKind::CloseBrace) || self.is_at_end() {
                    break;
                }
                if self.check(&TokenKind::Entry) {
                    entry_action = Some(self.parse_state_action_clause());
                } else if self.check(&TokenKind::Do) {
                    do_action = Some(self.parse_state_action_clause());
                } else if self.check(&TokenKind::Exit) {
                    exit_action = Some(self.parse_state_action_clause());
                } else if self.check(&TokenKind::State) {
                    sub_states.push(self.parse_state_decl(inner_doc)?);
                } else if self.check(&TokenKind::Transition) {
                    self.advance();
                    let t_tokens = self.read_tokens_until_semi();
                    let mut t_def = Self::parse_transition_tokens(&t_tokens);
                    if inner_doc.is_some() {
                        t_def.doc = inner_doc;
                    }
                    transitions.push(t_def);
                } else {
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!(
                            "Unexpected token '{:?}' in state declaration body",
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

        Ok(StateDef {
            name,
            doc: state_doc,
            entry_action,
            do_action,
            exit_action,
            transitions,
            sub_states,
        })
    }

    /// Parses an `operation` or `operation def` signature with typed `in`/`out`/`inout` parameters and optional return type into an `OperationDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-OPERATION-DECL]
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
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::CloseParen)?;
        }

        if self.check(&TokenKind::Colon) {
            self.advance();
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-INTERACTION-DECL]
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

        if self.check(&TokenKind::OpenBrace) {
            self.advance();
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
                    let tok = self.peek();
                    return Err(ParseError {
                        message: format!(
                            "Unexpected token '{:?}' in interaction declaration body",
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

        Ok(InteractionDef {
            name,
            lifelines,
            messages,
            triggers,
            doc: inter_doc,
        })
    }
}

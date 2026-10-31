//! State-machine transition clause token parsers and canonical SysML v2 transition formatters.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses `transition` statements (`first`/`from`, `accept`, `if`, `do`, `then`/`to`, and shorthand `<src>_to_<tgt>` identifiers) into `TransitionDef` nodes and formats `TransitionDef` clauses with bijective round-trip fidelity.
//! Bounds & Faults: Total non-panicking token and string processing using `.get()`, `.split_once()`, and saturating index/depth arithmetic with zero direct slice indexing.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Returns `true` when `tok` is a SysML v2 transition clause keyword (`first`, `from`, `then`, `to`, `accept`, `if`, `do`).
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-IS-TRANSITION-CLAUSE-KEYWORD]
    fn is_transition_clause_keyword(tok: &str) -> bool {
        matches!(
            tok,
            "first" | "from" | "then" | "to" | "accept" | "if" | "do"
        )
    }

    /// Returns `true` when `tok` is an unquoted lexer token representing a SysML v2 transition clause keyword.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-IS-TRANSITION-KEYWORD-TOKEN]
    fn is_transition_keyword_token(tok: &Token) -> bool {
        match &tok.kind {
            TokenKind::StringLit(_)
            | TokenKind::IntLit(_)
            | TokenKind::RealLit(_)
            | TokenKind::BoolLit(_)
            | TokenKind::LineComment(_)
            | TokenKind::DocComment(_)
            | TokenKind::Eof => false,
            TokenKind::Ident(s) => Self::is_transition_clause_keyword(s.as_str()),
            other => Self::is_transition_clause_keyword(other.as_str()),
        }
    }

    /// Renders a single lexer `Token` into its canonical textual lexeme for transition clause reconstruction.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-TOKEN-LEXEME]
    fn token_lexeme(tok: &Token) -> String {
        match &tok.kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::StringLit(s) => format!("\"{}\"", s),
            TokenKind::IntLit(n) => n.to_string(),
            TokenKind::RealLit(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            TokenKind::BoolLit(b) => b.to_string(),
            TokenKind::Eof | TokenKind::LineComment(_) | TokenKind::DocComment(_) => String::new(),
            _ => tok.kind.as_str().to_string(),
        }
    }

    /// Parses a simple, `::`-qualified, or `.`-dotted state endpoint reference from `tokens` starting at `*idx`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TRANSITION-ENDPOINT]
    fn parse_transition_endpoint(tokens: &[Token], idx: &mut usize) -> Option<String> {
        let mut out = String::new();
        let tok = tokens.get(*idx)?;
        if Self::is_transition_keyword_token(tok) {
            return None;
        }
        if tok.kind == TokenKind::DoubleColon {
            out.push_str("::");
            *idx = idx.saturating_add(1);
        }

        let first_seg = tokens.get(*idx)?;
        let lex = Self::token_lexeme(first_seg);
        if lex.is_empty() {
            return None;
        }
        out.push_str(&lex);
        *idx = idx.saturating_add(1);

        while let Some(sep_tok) = tokens.get(*idx) {
            let sep_str = match sep_tok.kind {
                TokenKind::DoubleColon => "::",
                TokenKind::Dot => ".",
                _ => break,
            };
            out.push_str(sep_str);
            *idx = idx.saturating_add(1);
            if let Some(seg_tok) = tokens.get(*idx) {
                let seg_lex = Self::token_lexeme(seg_tok);
                if !seg_lex.is_empty() {
                    out.push_str(&seg_lex);
                    *idx = idx.saturating_add(1);
                }
            } else {
                break;
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    /// Formats a slice of transition or state-action expression tokens with tight punctuation around `::`, `.`, `()`, `[]`, and `,`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-FORMAT-TRANSITION-EXPR]
    pub(super) fn format_transition_expr(tokens: &[Token]) -> String {
        let mut out = String::new();
        for (i, tok) in tokens.iter().enumerate() {
            let lex = Self::token_lexeme(tok);
            if lex.is_empty() {
                continue;
            }
            if i > 0 && !out.is_empty() {
                let prev_kind = tokens.get(i.saturating_sub(1)).map(|t| &t.kind);
                let curr_kind = &tok.kind;
                let prev_is_callable_target = prev_kind.is_some_and(|pk| {
                    matches!(
                        pk,
                        TokenKind::Ident(_) | TokenKind::CloseParen | TokenKind::CloseBracket
                    ) || pk
                        .as_str()
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                });
                let no_space_after_prev = matches!(
                    prev_kind,
                    Some(
                        TokenKind::DoubleColon
                            | TokenKind::Dot
                            | TokenKind::OpenParen
                            | TokenKind::OpenBracket
                    )
                );
                let no_space_before_curr =
                    matches!(
                        curr_kind,
                        TokenKind::DoubleColon
                            | TokenKind::Dot
                            | TokenKind::CloseParen
                            | TokenKind::CloseBracket
                            | TokenKind::Comma
                            | TokenKind::Semi
                    ) || (matches!(curr_kind, TokenKind::OpenParen | TokenKind::OpenBracket)
                        && prev_is_callable_target);

                if !no_space_after_prev && !no_space_before_curr {
                    out.push(' ');
                }
            }
            out.push_str(&lex);
        }
        out
    }

    /// Consumes a balanced `accept`, `if`, or `do` expression clause up to the next top-level transition keyword and returns its formatted text.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TRANSITION-TOKENS]
    fn parse_transition_clause_expr(tokens: &[Token], idx: &mut usize, kw: &str) -> Option<String> {
        *idx = idx.saturating_add(1);
        let start_idx = *idx;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        while let Some(inner_tok) = tokens.get(*idx) {
            let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if at_top_level && Self::is_transition_keyword_token(inner_tok) {
                break;
            }
            match inner_tok.kind {
                TokenKind::OpenParen => paren_depth = paren_depth.saturating_add(1),
                TokenKind::CloseParen => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::OpenBracket => bracket_depth = bracket_depth.saturating_add(1),
                TokenKind::CloseBracket => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::OpenBrace => brace_depth = brace_depth.saturating_add(1),
                TokenKind::CloseBrace => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
            *idx = idx.saturating_add(1);
        }
        let slice = tokens.get(start_idx..*idx)?;
        let expr_slice = if kw == "do"
            && slice.first().is_some_and(|t| t.kind == TokenKind::Action)
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
        let formatted = Self::format_transition_expr(expr_slice);
        if formatted.is_empty() {
            None
        } else {
            Some(formatted)
        }
    }

    /// Infers `source`, `target`, and optional `trigger` from shorthand `<src>_to_<tgt>[_on_<trig>]` transition identifiers when explicit endpoints are absent.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TRANSITION-TOKENS]
    fn infer_shorthand_transition_endpoints(
        raw_name: &str,
        source: &mut Option<String>,
        target: &mut Option<String>,
        trigger: &mut Option<String>,
    ) {
        if let Some((before_to, after_to_raw)) = raw_name.split_once("_to_") {
            let src_part = before_to.trim();
            let after_to = after_to_raw.trim();
            let (tgt_part, trig_part) =
                if let Some((tgt_raw, trig_raw)) = after_to.split_once("_on_") {
                    (tgt_raw.trim(), trig_raw.trim())
                } else {
                    (after_to, "")
                };
            if !src_part.is_empty() && !tgt_part.is_empty() {
                *source = Some(src_part.to_string());
                *target = Some(tgt_part.to_string());
                if trigger.is_none() && !trig_part.is_empty() {
                    *trigger = Some(trig_part.to_string());
                }
            }
        }
    }

    /// Parses a slice of lexer `Token` items from a SysML v2 `transition` statement into a `TransitionDef` without fabricating synthetic names on unnamed transitions.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TRANSITION-TOKENS]
    pub(super) fn parse_transition_tokens(raw_tokens: &[Token]) -> TransitionDef {
        let tokens: Vec<Token> = raw_tokens
            .iter()
            .filter(|t| {
                !matches!(
                    t.kind,
                    TokenKind::Eof
                        | TokenKind::Semi
                        | TokenKind::LineComment(_)
                        | TokenKind::DocComment(_)
                )
            })
            .cloned()
            .collect();

        let mut name = None;
        let mut source = None;
        let mut target = None;
        let mut trigger = None;
        let mut guard = None;
        let mut effect = None;

        let mut idx = 0usize;
        if let Some(first_tok) = tokens.first() {
            if !Self::is_transition_keyword_token(first_tok) {
                if let Some(ep_or_name) = Self::parse_transition_endpoint(&tokens, &mut idx) {
                    name = Some(ep_or_name);
                }
            }
        }

        while let Some(tok) = tokens.get(idx) {
            let kw = if Self::is_transition_keyword_token(tok) {
                Self::token_lexeme(tok)
            } else {
                idx = idx.saturating_add(1);
                continue;
            };

            match kw.as_str() {
                "first" | "from" => {
                    idx = idx.saturating_add(1);
                    if let Some(ep) = Self::parse_transition_endpoint(&tokens, &mut idx) {
                        source = Some(ep);
                    }
                }
                "then" | "to" => {
                    idx = idx.saturating_add(1);
                    if let Some(ep) = Self::parse_transition_endpoint(&tokens, &mut idx) {
                        target = Some(ep);
                    }
                }
                "accept" | "if" | "do" => {
                    if let Some(formatted) =
                        Self::parse_transition_clause_expr(&tokens, &mut idx, kw.as_str())
                    {
                        match kw.as_str() {
                            "accept" => trigger = Some(formatted),
                            "if" => guard = Some(formatted),
                            "do" => effect = Some(formatted),
                            _ => {}
                        }
                    }
                }
                _ => {
                    idx = idx.saturating_add(1);
                }
            }
        }

        if source.is_none() && target.is_none() {
            if let Some(raw_name) = name.as_deref() {
                Self::infer_shorthand_transition_endpoints(
                    raw_name,
                    &mut source,
                    &mut target,
                    &mut trigger,
                );
            }
        }

        if name.is_none()
            && source.is_none()
            && target.is_none()
            && trigger.is_none()
            && guard.is_none()
            && effect.is_none()
            && !tokens.is_empty()
        {
            let fallback = Self::format_transition_expr(&tokens);
            if !fallback.is_empty() {
                name = Some(fallback);
            }
        }

        TransitionDef {
            name,
            source,
            target,
            trigger,
            guard,
            effect,
            doc: None,
        }
    }

    /// Decomposes a raw SysML v2 transition statement body into a structured `TransitionDef`.
    ///
    /// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-TRANSITION-CLAUSE]
    /// Contract: Tokenizes `raw` via `SysmlParser::tokenize_source` and delegates to `SysmlParser::parse_transition_tokens` so `first <src> then <tgt>` and `from <src> to <tgt>` extract structured endpoints without fabricating synthetic names on unnamed transitions.
    /// Bounds & Faults: Linear `O(N)` over `raw.len()`; total over `&str`, degrading cleanly to a trimmed name-only `TransitionDef` if lexical tokenization fails.
    pub fn parse_transition_clause(raw: &str) -> TransitionDef {
        match Self::tokenize_source(raw) {
            Ok(tokens) => Self::parse_transition_tokens(&tokens),
            Err(_) => {
                let trimmed = raw.trim();
                TransitionDef {
                    name: if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    },
                    source: None,
                    target: None,
                    trigger: None,
                    guard: None,
                    effect: None,
                    doc: None,
                }
            }
        }
    }

    /// Formats a structured `TransitionDef` into a canonical SysML v2 transition statement body (`[name] [first <src>] [accept <trig>] [if <grd>] [do <eff>] [then <tgt>]`).
    ///
    /// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-FORMAT-TRANSITION-CLAUSE]
    /// Contract: Preserves round-trip equivalence (`SysmlParser::parse_transition_clause(&SysmlParser::format_transition_clause(t))`) across `name`, `source`, `target`, `trigger`, `guard`, and `effect` fields.
    /// Bounds & Faults: Linear `O(K)` over the total byte length `K` of `TransitionDef` fields; total pure function with zero panics.
    pub fn format_transition_clause(t: &TransitionDef) -> String {
        let mut parts = Vec::new();
        if let Some(n) = t.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            if !n.contains(char::is_whitespace) && !Self::is_transition_clause_keyword(n) {
                parts.push(n.to_string());
            }
        }
        if let Some(s) = t.source.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("first {s}"));
        }
        if let Some(trig) = t
            .trigger
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            parts.push(format!("accept {trig}"));
        }
        if let Some(grd) = t.guard.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("if {grd}"));
        }
        if let Some(eff) = t.effect.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("do {eff}"));
        }
        if let Some(tg) = t.target.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("then {tg}"));
        }
        if parts.is_empty() {
            t.name.as_deref().unwrap_or("").trim().to_string()
        } else {
            parts.join(" ")
        }
    }
}

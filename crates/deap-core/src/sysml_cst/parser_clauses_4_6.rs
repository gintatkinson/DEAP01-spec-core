//! # SysML v2 / KerML Rowan CST Parser -- Clauses 4 to 6
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Recursive Rowan `GreenNodeBuilder` CST construction for OMG SysML v2.0 + KerML 1.0 Clauses 4--6 (`BP-03`, `BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Hierarchical CST fidelity and `MAX_CST_PARSE_DEPTH` (`64`, `DEAP-E1002`) guard.
//! ## 3. Preconditions -- Invoked by `SysmlCstBuilder::parse_sequence` on a valid `LexedSysmlToken` stream.
//! ## 4. Postconditions -- Advances `self.pos` across the declaration/statement and records `SysmlConstructSummary`.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(K) over declaration tokens; Space: O(min(D, 64)) recursion frames.
//! ## 6. Failure Modes & Fault Tolerance -- Unclosed `{` blocks emit `DEAP-E1001`; depth overflow emits `DEAP-E1002` and recovers iteratively.
//! ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use super::lexer::{LexedSysmlToken, SysmlToken};
use super::parser::SysmlConstructSummary;
use super::parser_clauses_1_3::{SysmlCstBuilder, MAX_CST_PARSE_DEPTH};
use super::syntax_kind::SysmlSyntaxKind;

/// Returns `true` when `slice` is a leading KerML/SysML modifier that precedes a declaration keyword.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Allows lookahead classification to skip prefix modifiers.
/// ## 2. Invariants & Mathematical Properties -- Pure membership check over static modifier keywords.
/// ## 3. Preconditions -- `slice` is a token lexeme.
/// ## 4. Postconditions -- Returns `true` iff `slice` is a prefix modifier.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance -- Total pure function with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn is_leading_modifier(slice: &str) -> bool {
    matches!(
        slice,
        "abstract" | "readonly" | "derived" | "end" | "in" | "out" | "inout" | "ref"
            | "private" | "protected" | "public" | "standard" | "library" | "ordered" | "nonunique"
    )
}

impl<'a> SysmlCstBuilder<'a> {
    /// Classifies Clauses 4--6 constructs (`CalcDef`, `ConstraintDef`, `RequirementDef`, `UseCaseDef`, `VerificationDef`, `ViewDef`, `StatementNode`).
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Maps OMG SysML v2 Clauses 4--6 keywords to composite `SysmlSyntaxKind` variants (`BP-03`).
    /// ## 2. Invariants & Mathematical Properties -- Handles compound `use case [def]` and `test case [def]` keywords deterministically.
    /// ## 3. Preconditions -- `lead` and `second` are non-trivia lookahead slices; `has_def` indicates `def` presence.
    /// ## 4. Postconditions -- Returns the matching composite `SysmlSyntaxKind` for Clauses 4--6.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Defaults to `SysmlSyntaxKind::StatementNode` for general statements.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn classify_clause_4_6_kind(lead: &str, second: Option<&str>, has_def: bool) -> SysmlSyntaxKind {
        match lead {
            "calc" => if has_def { SysmlSyntaxKind::CalcDef } else { SysmlSyntaxKind::CalcUsage },
            "constraint" | "assert" | "assume" | "require" => {
                if has_def { SysmlSyntaxKind::ConstraintDef } else { SysmlSyntaxKind::ConstraintUsage }
            }
            "requirement" | "satisfy" => {
                if has_def { SysmlSyntaxKind::RequirementDef } else { SysmlSyntaxKind::RequirementUsage }
            }
            "concern" | "frame" => {
                if has_def { SysmlSyntaxKind::ConcernDef } else { SysmlSyntaxKind::ConcernUsage }
            }
            "use" if second == Some("case") => {
                if has_def { SysmlSyntaxKind::UseCaseDef } else { SysmlSyntaxKind::UseCaseUsage }
            }
            "include" => SysmlSyntaxKind::UseCaseUsage,
            "analysis" => if has_def { SysmlSyntaxKind::AnalysisDef } else { SysmlSyntaxKind::AnalysisUsage },
            "verification" | "verify" => {
                if has_def { SysmlSyntaxKind::VerificationDef } else { SysmlSyntaxKind::VerificationUsage }
            }
            "test" if second == Some("case") => {
                if has_def { SysmlSyntaxKind::VerificationDef } else { SysmlSyntaxKind::VerificationUsage }
            }
            "view" => if has_def { SysmlSyntaxKind::ViewDef } else { SysmlSyntaxKind::ViewUsage },
            "viewpoint" => if has_def { SysmlSyntaxKind::ViewpointDef } else { SysmlSyntaxKind::ViewpointUsage },
            "rendering" => if has_def { SysmlSyntaxKind::RenderingDef } else { SysmlSyntaxKind::RenderingUsage },
            _ => SysmlSyntaxKind::StatementNode,
        }
    }

    /// Inspects the token stream at `self.pos` and parses the next composite CST construct at `depth`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Dispatches standalone doc comments, lexical errors, and Clauses 1--6 constructs.
    /// ## 2. Invariants & Mathematical Properties -- Always wraps the construct in a composite `SysmlSyntaxKind` node and advances `self.pos`.
    /// ## 3. Preconditions -- `self.pos < self.tokens.len()`.
    /// ## 4. Postconditions -- Emits a complete composite CST node into `self.builder`.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(K) over construct tokens; Space: O(min(D, 64)).
    /// ## 6. Failure Modes & Fault Tolerance -- Wraps `SysmlToken::Error` tokens in `SysmlSyntaxKind::ErrorNode`.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_next_construct(&mut self, depth: usize) {
        let Some(curr) = self.tokens.get(self.pos).cloned() else {
            return;
        };
        if curr.token == SysmlToken::BlockComment {
            self.builder.start_node(SysmlSyntaxKind::DocCommentNode.into());
            self.emit_current_token();
            self.builder.finish_node();
            return;
        }
        if curr.token == SysmlToken::Error {
            self.builder.start_node(SysmlSyntaxKind::ErrorNode.into());
            self.emit_current_token();
            self.builder.finish_node();
            return;
        }
        let (node_kind, summary) = self.classify_construct_at_cursor();
        if node_kind == SysmlSyntaxKind::DocCommentNode && self.try_parse_doc_comment_node() {
            return;
        }
        if let Some(s) = summary {
            self.constructs.push(s);
        }
        self.builder.start_node(node_kind.into());
        self.parse_header_and_tail(depth);
        self.builder.finish_node();
    }

    /// Parses `doc /* ... */` or `comment /* ... */` when not followed by `=` property assignment.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Ensures `doc /* ... */` prefixes form self-contained `DocCommentNode` CST nodes before declarations.
    /// ## 2. Invariants & Mathematical Properties -- Consumes `doc` + trivia + `BlockComment` + optional `;`.
    /// ## 3. Preconditions -- Current token is `doc` or `comment`.
    /// ## 4. Postconditions -- Returns `true` if a block-comment doc node was emitted, or `false` for `doc = "...";`.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Total lookahead without mutating cursor when `false`.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    fn try_parse_doc_comment_node(&mut self) -> bool {
        let mut idx = self.pos.saturating_add(1);
        while let Some(t) = self.tokens.get(idx) {
            if matches!(t.token, SysmlToken::Whitespace | SysmlToken::LineComment) {
                idx = idx.saturating_add(1);
            } else if t.token == SysmlToken::BlockComment {
                self.builder.start_node(SysmlSyntaxKind::DocCommentNode.into());
                while self.pos <= idx {
                    self.emit_current_token();
                }
                self.builder.finish_node();
                return true;
            } else {
                break;
            }
        }
        false
    }

    /// Determines the composite `SysmlSyntaxKind` and optional `SysmlConstructSummary` at `self.pos`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Unifies Clauses 1--3 and Clauses 4--6 lookahead classification (`BP-03`).
    /// ## 2. Invariants & Mathematical Properties -- Skips prefix modifiers and inspects `def` and identifier tokens.
    /// ## 3. Preconditions -- `self.pos < self.tokens.len()`.
    /// ## 4. Postconditions -- Returns `(SysmlSyntaxKind, Option<SysmlConstructSummary>)`.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(1) bounded to 8 lookahead tokens; Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Total classification with checked `.get()` access.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    fn classify_construct_at_cursor(&self) -> (SysmlSyntaxKind, Option<SysmlConstructSummary>) {
        let lookahead = self.peek_non_trivia(8);
        let has_library = lookahead.iter().any(|(_, t)| t.slice == "library");
        let mut cursor = 0usize;
        while let Some((_, tok)) = lookahead.get(cursor) {
            if is_leading_modifier(tok.slice) {
                cursor = cursor.saturating_add(1);
            } else {
                break;
            }
        }
        let Some((_, lead_tok)) = lookahead.get(cursor).cloned() else {
            return (SysmlSyntaxKind::StatementNode, None);
        };
        let second_tok = lookahead.get(cursor.saturating_add(1)).map(|(_, t)| t.clone());
        let third_tok = lookahead.get(cursor.saturating_add(2)).map(|(_, t)| t.clone());
        let is_compound = matches!((lead_tok.slice, second_tok.as_ref().map(|t| t.slice)), ("use" | "test", Some("case")));
        let def_offset = if is_compound { 2 } else { 1 };
        let has_def = lookahead
            .get(cursor.saturating_add(def_offset))
            .is_some_and(|(_, t)| t.token == SysmlToken::DefKw || t.slice == "def");
        let name_offset = def_offset.saturating_add(usize::from(has_def));
        let second_slice = second_tok.as_ref().map(|t| t.slice);
        let kind = Self::classify_clause_1_3_kind(lead_tok.slice, second_slice, has_def, has_library)
            .unwrap_or_else(|| Self::classify_clause_4_6_kind(lead_tok.slice, second_slice, has_def));
        let name_tok = lookahead
            .get(cursor.saturating_add(name_offset))
            .map(|(_, t)| t.clone())
            .or(third_tok)
            .or(second_tok);
        (kind, Self::build_construct_summary(kind, lead_tok, name_tok, has_def))
    }

    /// Builds a `SysmlConstructSummary` when `kind` is a named declaration/usage construct.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Populates construct summaries directly during Rowan CST node creation (`BP-03`).
    /// ## 2. Invariants & Mathematical Properties -- Strips surrounding `'` quotes on `UnrestrictedName` identifiers.
    /// ## 3. Preconditions -- `lead_tok` is the construct's primary keyword token.
    /// ## 4. Postconditions -- Returns `Some(SysmlConstructSummary)` for named constructs or `None` for anonymous statements.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Total function with zero panics.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    fn build_construct_summary(
        kind: SysmlSyntaxKind,
        lead_tok: LexedSysmlToken<'a>,
        name_tok: Option<LexedSysmlToken<'a>>,
        is_definition: bool,
    ) -> Option<SysmlConstructSummary> {
        if matches!(
            kind,
            SysmlSyntaxKind::StatementNode | SysmlSyntaxKind::DocCommentNode | SysmlSyntaxKind::ErrorNode
        ) {
            return None;
        }
        let tok = name_tok?;
        let name = if matches!(
            tok.token,
            SysmlToken::Ident
                | SysmlToken::UnrestrictedName
                | SysmlToken::StructuralKw
                | SysmlToken::BehavioralKw
                | SysmlToken::ReqCalcVerificationKw
        ) {
            tok.slice.trim_matches('\'').to_string()
        } else {
            return None;
        };
        Some(SysmlConstructSummary {
            kind,
            name,
            is_definition,
            byte_range: lead_tok.span.start..tok.span.end,
        })
    }

    /// Parses the declaration header, specialization/multiplicity/expression clauses, and `;` or `{ ... }` body.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Enforces `MAX_CST_PARSE_DEPTH` (`64`, `DEAP-E1002`) and unclosed brace detection (`DEAP-E1001`).
    /// ## 2. Invariants & Mathematical Properties -- Emits `SpecializationClause`, `MultiplicityBounds`, and `ExpressionNode` child CST nodes.
    /// ## 3. Preconditions -- An enclosing declaration/statement node has been started on `self.builder`.
    /// ## 4. Postconditions -- Consumes tokens through the terminating `;` or matching `}`.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(K); Space: O(min(D, 64)).
    /// ## 6. Failure Modes & Fault Tolerance -- Wraps depth-overflow blocks in `ErrorNode` iteratively without stack overflow.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_header_and_tail(&mut self, depth: usize) {
        while let Some(tok) = self.tokens.get(self.pos).cloned() {
            match tok.token {
                SysmlToken::Semi => {
                    self.emit_current_token();
                    break;
                }
                SysmlToken::RBrace => break,
                SysmlToken::LBrace => {
                    self.parse_braced_body(depth, tok.span.start);
                    break;
                }
                SysmlToken::ColonGt | SysmlToken::ColonGtGt | SysmlToken::ColonColonGt | SysmlToken::Colon => {
                    self.parse_specialization_clause();
                }
                SysmlToken::LBracket => self.parse_multiplicity_bounds(),
                SysmlToken::ColonEq => self.parse_expression_clause(),
                SysmlToken::Operator if tok.slice == "=" => self.parse_expression_clause(),
                _ => self.emit_current_token(),
            }
        }
    }

    /// Parses a `{ ... }` block at `depth`, enforcing `MAX_CST_PARSE_DEPTH` (`64`).
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Guards against stack overflow (`DEAP-E1002`) and unclosed braces (`DEAP-E1001`).
    /// ## 2. Invariants & Mathematical Properties -- Recurses only when `depth.saturating_add(1) < MAX_CST_PARSE_DEPTH`.
    /// ## 3. Preconditions -- Current token at `self.pos` is `SysmlToken::LBrace`.
    /// ## 4. Postconditions -- Consumes `{`, all inner declarations, and matching `}` (or records `DEAP-E1001` at EOF).
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(K); Space: O(min(D, 64)).
    /// ## 6. Failure Modes & Fault Tolerance -- Iterative balanced recovery on depth overflow (`DEAP-E1002`).
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
    fn parse_braced_body(&mut self, depth: usize, open_byte: usize) {
        let next_depth = depth.saturating_add(1);
        if next_depth >= MAX_CST_PARSE_DEPTH {
            self.diagnostics.push(format!(
                "[DEAP-E1002] MAX_AST_DEPTH ({MAX_CST_PARSE_DEPTH}) exceeded at byte {open_byte}"
            ));
            self.builder.start_node(SysmlSyntaxKind::ErrorNode.into());
            let mut brace_depth = 0usize;
            while let Some(tok) = self.tokens.get(self.pos).cloned() {
                if tok.token == SysmlToken::LBrace {
                    brace_depth = brace_depth.saturating_add(1);
                } else if tok.token == SysmlToken::RBrace {
                    brace_depth = brace_depth.saturating_sub(1);
                    self.emit_current_token();
                    if brace_depth == 0 {
                        break;
                    }
                    continue;
                }
                self.emit_current_token();
            }
            self.builder.finish_node();
            return;
        }
        self.emit_current_token();
        self.parse_sequence(true, next_depth);
        if self.tokens.get(self.pos).is_some_and(|t| t.token == SysmlToken::RBrace) {
            self.emit_current_token();
        } else {
            self.diagnostics.push(format!(
                "[DEAP-E1001] Unclosed block '{{' opened at byte {open_byte}"
            ));
        }
    }

    /// Parses a specialization/typing clause (`:>`, `:>>`, `::>`, `:`) into `SysmlSyntaxKind::SpecializationClause`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Models KerML specialization and feature typing subtrees in the Rowan CST.
    /// ## 2. Invariants & Mathematical Properties -- Stops before `[`, `=`, `:=`, `{`, `;`, or `}`.
    /// ## 3. Preconditions -- Current token is `:`, `:>`, `:>>`, or `::>`.
    /// ## 4. Postconditions -- Emits a `SpecializationClause` node.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(K); Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Total function with checked token lookahead.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    fn parse_specialization_clause(&mut self) {
        self.builder.start_node(SysmlSyntaxKind::SpecializationClause.into());
        self.emit_current_token();
        while let Some(tok) = self.tokens.get(self.pos).cloned() {
            if matches!(
                tok.token,
                SysmlToken::LBracket | SysmlToken::ColonEq | SysmlToken::LBrace | SysmlToken::RBrace | SysmlToken::Semi
            ) || (tok.token == SysmlToken::Operator && tok.slice == "=")
            {
                break;
            }
            self.emit_current_token();
        }
        self.builder.finish_node();
    }

    /// Parses a `[lo..hi]` multiplicity clause into `SysmlSyntaxKind::MultiplicityBounds`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Models SysML v2 multiplicity bounds in the Rowan CST.
    /// ## 2. Invariants & Mathematical Properties -- Consumes `[` through matching `]` (or stops before `;`, `{`, `}`).
    /// ## 3. Preconditions -- Current token is `SysmlToken::LBracket`.
    /// ## 4. Postconditions -- Emits a `MultiplicityBounds` node.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(K); Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Total function with zero panics.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    fn parse_multiplicity_bounds(&mut self) {
        self.builder.start_node(SysmlSyntaxKind::MultiplicityBounds.into());
        self.emit_current_token();
        while let Some(tok) = self.tokens.get(self.pos).cloned() {
            if matches!(tok.token, SysmlToken::Semi | SysmlToken::LBrace | SysmlToken::RBrace) {
                break;
            }
            self.emit_current_token();
            if tok.token == SysmlToken::RBracket {
                break;
            }
        }
        self.builder.finish_node();
    }

    /// Parses a value assignment / expression (`= expr` or `:= expr`) into `SysmlSyntaxKind::ExpressionNode`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope -- Captures Clause 6 value and constraint expressions as typed Rowan CST nodes.
    /// ## 2. Invariants & Mathematical Properties -- Consumes `=` / `:=` and expression tokens up to `;`, `{`, or `}`.
    /// ## 3. Preconditions -- Current token is `=` or `:=`.
    /// ## 4. Postconditions -- Emits an `ExpressionNode` CST child.
    /// ## 5. Algorithmic Complexity Bounds -- Time: O(K); Space: O(1).
    /// ## 6. Failure Modes & Fault Tolerance -- Total function with zero panics.
    /// ## 7. Realization & Traceability -- Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    fn parse_expression_clause(&mut self) {
        self.builder.start_node(SysmlSyntaxKind::ExpressionNode.into());
        self.emit_current_token();
        while let Some(tok) = self.tokens.get(self.pos).cloned() {
            if matches!(tok.token, SysmlToken::Semi | SysmlToken::LBrace | SysmlToken::RBrace) {
                break;
            }
            self.emit_current_token();
        }
        self.builder.finish_node();
    }
}

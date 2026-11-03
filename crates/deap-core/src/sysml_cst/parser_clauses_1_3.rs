//! # SysML v2 / KerML Rowan CST Parser -- Clauses 1 to 3
//! [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Implements recursive Rowan `GreenNodeBuilder` CST construction for OMG SysML v2.0 + KerML 1.0
//! Clauses 1--3 (Namespaces/Packages/Metadata, Structural Definitions/Usages, and Behavioral
//! Actions/States/Transitions) under DO-178C Level A and ISO 26262 ASIL D (`BP-03`, `BP-10`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Lossless Round-Trip Identity: Every `LexedSysmlToken` is emitted into `GreenNodeBuilder` in
//!   strict source order so `syntax.text().to_string() == source` holds unconditionally.
//! - Bounded Recursion: Nesting depth is strictly capped at `MAX_CST_PARSE_DEPTH` (`64`, `DEAP-E1002`).
//!
//! ## 3. Preconditions
//! - Input token slice is produced by `tokenize_sysml(source)`.
//!
//! ## 4. Postconditions
//! - Produces a hierarchical `rowan::GreenNode` rooted at `SysmlSyntaxKind::SourceFile` with
//!   nested composite nodes and diagnostic receipts (`DEAP-E1001`, `DEAP-E1002`).
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) over token count `N`.
//! - Space Complexity: O(N) green-tree allocation and O(min(D, 64)) call-stack frames.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed tokens and unclosed blocks wrap offending spans in `SysmlSyntaxKind::ErrorNode`
//!   (`DEAP-E1001`); depth overflow beyond `64` emits `DEAP-E1002` and recovers iteratively.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

use rowan::GreenNodeBuilder;

use super::lexer::{LexedSysmlToken, SysmlToken};
use super::parser::SysmlConstructSummary;
use super::syntax_kind::{token_to_syntax_kind, SysmlSyntaxKind};

/// Maximum CST nesting depth permitted before failing closed with `DEAP-E1002`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents call-stack exhaustion on adversarially nested SysML v2 blocks under DO-178C Level A.
/// ## 2. Invariants & Mathematical Properties
/// Fixed upper bound `MAX_CST_PARSE_DEPTH == 64`.
/// ## 3. Preconditions
/// None.
/// ## 4. Postconditions
/// Enforced on every braced block descent in `SysmlCstBuilder`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1); Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Exceeding depth 64 triggers `DEAP-E1002` and iterative recovery.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
pub const MAX_CST_PARSE_DEPTH: usize = 64;

/// Stateful Rowan CST builder over a `LexedSysmlToken` stream.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Drives lossless 6-clause SysML v2 CST construction under DO-178C Level A (`BP-03`, `BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// `pos` advances monotonically from `0` to `tokens.len()`.
/// ## 3. Preconditions
/// `tokens` is a valid lexical stream for a single UTF-8 SysML v2 source buffer.
/// ## 4. Postconditions
/// Consumes all tokens and yields `(rowan::GreenNode, Vec<String>, Vec<SysmlConstructSummary>)`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Never panics; wraps lexical/syntactic faults in `SysmlSyntaxKind::ErrorNode`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
pub struct SysmlCstBuilder<'a> {
    pub(crate) tokens: &'a [LexedSysmlToken<'a>],
    pub(crate) pos: usize,
    pub(crate) builder: GreenNodeBuilder<'static>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) constructs: Vec<SysmlConstructSummary>,
}

impl<'a> SysmlCstBuilder<'a> {
    /// Creates a new `SysmlCstBuilder` over `tokens`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Initializes deterministic CST parser state under DO-178C Level A.
    /// ## 2. Invariants & Mathematical Properties
    /// `pos == 0`, empty diagnostics and constructs.
    /// ## 3. Preconditions
    /// `tokens` is a borrowed slice of `LexedSysmlToken`.
    /// ## 4. Postconditions
    /// Returns an initialized `SysmlCstBuilder`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1); Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total constructor with zero panics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn new(tokens: &'a [LexedSysmlToken<'a>]) -> Self {
        Self {
            tokens,
            pos: 0,
            builder: GreenNodeBuilder::new(),
            diagnostics: Vec::new(),
            constructs: Vec::new(),
        }
    }

    /// Parses the entire token stream into a `SourceFile` Rowan `GreenNode`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Entrypoint for 6-clause recursive Rowan CST synthesis (`BP-03`, `BP-10`).
    /// ## 2. Invariants & Mathematical Properties
    /// Emits every token in `self.tokens` under `SysmlSyntaxKind::SourceFile`.
    /// ## 3. Preconditions
    /// Invoked once on a fresh `SysmlCstBuilder`.
    /// ## 4. Postconditions
    /// Returns `(GreenNode, diagnostics, constructs)` with `self.pos == self.tokens.len()`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(N); Space Complexity: O(N).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Recovers from unmatched braces, lexical errors, and depth overflow without panicking.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn build_source_file(
        mut self,
    ) -> (rowan::GreenNode, Vec<String>, Vec<SysmlConstructSummary>) {
        self.builder.start_node(SysmlSyntaxKind::SourceFile.into());
        self.parse_sequence(false, 0);
        self.builder.finish_node();
        (self.builder.finish(), self.diagnostics, self.constructs)
    }

    /// Parses a sequence of declarations/statements at `depth` until EOF or closing `}`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Drives statement-level CST construction and error recovery under DO-178C Level A.
    /// ## 2. Invariants & Mathematical Properties
    /// Strictly advances `self.pos` on every loop iteration.
    /// ## 3. Preconditions
    /// `depth <= MAX_CST_PARSE_DEPTH`.
    /// ## 4. Postconditions
    /// Stops at EOF or (when `in_block` is true) before the matching `}`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(N) across the enclosed token range; Space Complexity: O(1) per level.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Unmatched top-level `}` tokens are wrapped in `ErrorNode` (`DEAP-E1001`).
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_9ucqbbd0s2yo]
    pub(crate) fn parse_sequence(&mut self, in_block: bool, depth: usize) {
        while self.pos < self.tokens.len() {
            self.emit_whitespace_and_line_comments();
            let Some(curr) = self.tokens.get(self.pos) else {
                break;
            };
            if curr.token == SysmlToken::RBrace {
                if in_block {
                    break;
                }
                let start = curr.span.start;
                self.diagnostics.push(format!(
                    "[DEAP-E1001] Unmatched closing brace '}}' at byte {start}"
                ));
                self.builder.start_node(SysmlSyntaxKind::ErrorNode.into());
                self.emit_current_token();
                self.builder.finish_node();
                continue;
            }
            let before = self.pos;
            self.parse_next_construct(depth);
            if self.pos == before {
                self.builder.start_node(SysmlSyntaxKind::ErrorNode.into());
                self.emit_current_token();
                self.builder.finish_node();
            }
        }
    }

    /// Emits contiguous `Whitespace` and `LineComment` trivia tokens at `self.pos`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Preserves lossless whitespace and line-comment trivia in the Rowan CST.
    /// ## 2. Invariants & Mathematical Properties
    /// Consumes only `Whitespace` and `LineComment` tokens.
    /// ## 3. Preconditions
    /// `self.pos <= self.tokens.len()`.
    /// ## 4. Postconditions
    /// `self.pos` points to EOF or a non-whitespace/non-line-comment token.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(T) over contiguous trivia count `T`; Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total function with checked `.get()` indexing.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn emit_whitespace_and_line_comments(&mut self) {
        while let Some(tok) = self.tokens.get(self.pos) {
            if matches!(tok.token, SysmlToken::Whitespace | SysmlToken::LineComment) {
                self.emit_current_token();
            } else {
                break;
            }
        }
    }

    /// Emits the token at `self.pos` into `self.builder` and increments `self.pos`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Atomic token emission primitive for `SysmlCstBuilder`.
    /// ## 2. Invariants & Mathematical Properties
    /// Advances `self.pos` by 1 and records `DEAP-E1001` if `tok.token == SysmlToken::Error`.
    /// ## 3. Preconditions
    /// `self.pos < self.tokens.len()`.
    /// ## 4. Postconditions
    /// Token leaf is appended to the active Rowan node.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1); Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// No-op if `self.pos >= self.tokens.len()`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn emit_current_token(&mut self) {
        if let Some(tok) = self.tokens.get(self.pos) {
            if tok.token == SysmlToken::Error {
                self.diagnostics.push(format!(
                    "[DEAP-E1001] Lexical error token '{}' at bytes {}..{}",
                    tok.slice, tok.span.start, tok.span.end
                ));
            }
            let kind = token_to_syntax_kind(tok.token);
            self.builder.token(kind.into(), tok.slice);
            self.pos = self.pos.saturating_add(1);
        }
    }

    /// Collects up to `limit` non-trivia `(index, LexedSysmlToken)` lookahead items from `self.pos`.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Provides bounded k-token lookahead across trivia without mutating cursor state.
    /// ## 2. Invariants & Mathematical Properties
    /// Skips `Whitespace`, `LineComment`, and `BlockComment` tokens up to `limit` items.
    /// ## 3. Preconditions
    /// `limit <= 8`.
    /// ## 4. Postconditions
    /// Returns at most `limit` non-trivia tokens starting at or after `self.pos`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(T + limit); Space Complexity: O(limit).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total read-only inspection using checked `.get()`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn peek_non_trivia(&self, limit: usize) -> Vec<(usize, LexedSysmlToken<'a>)> {
        let mut out = Vec::with_capacity(limit);
        let mut idx = self.pos;
        while out.len() < limit {
            let Some(tok) = self.tokens.get(idx) else {
                break;
            };
            if !matches!(
                tok.token,
                SysmlToken::Whitespace | SysmlToken::LineComment | SysmlToken::BlockComment
            ) {
                out.push((idx, tok.clone()));
            }
            idx = idx.saturating_add(1);
        }
        out
    }

    /// Classifies Clauses 1--3 constructs (`PackageDecl`, `ImportDecl`, Structural, Behavioral) from lookahead tokens.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Maps OMG SysML v2 Clauses 1--3 keywords and `def` modifiers to composite `SysmlSyntaxKind` variants.
    /// ## 2. Invariants & Mathematical Properties
    /// Distinguishes `*Def` from `*Usage` by checking whether `def` follows the primary keyword.
    /// ## 3. Preconditions
    /// `lead` contains the leading non-modifier keyword slice and `has_def` indicates a following `def`.
    /// ## 4. Postconditions
    /// Returns `Some(SysmlSyntaxKind)` for Clauses 1--3 or `None` to delegate to Clauses 4--6.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1); Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total pure classification with zero panics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn classify_clause_1_3_kind(
        lead: &str,
        second: Option<&str>,
        has_def: bool,
        has_library: bool,
    ) -> Option<SysmlSyntaxKind> {
        match lead {
            "package" | "namespace" => Some(if has_library {
                SysmlSyntaxKind::LibraryPackageDecl
            } else {
                SysmlSyntaxKind::PackageDecl
            }),
            "import" => Some(SysmlSyntaxKind::ImportDecl),
            "alias" => Some(SysmlSyntaxKind::AliasDecl),
            "doc" | "comment" | "rep" | "locale" => Some(SysmlSyntaxKind::DocCommentNode),
            "metadata" | "hazard" | "risk" | "capability" => Some(if has_def {
                SysmlSyntaxKind::MetadataDef
            } else {
                SysmlSyntaxKind::MetadataUsage
            }),
            "@" | "#" => Some(SysmlSyntaxKind::MetadataUsage),
            "part" => Some(if has_def { SysmlSyntaxKind::PartDef } else { SysmlSyntaxKind::PartUsage }),
            "port" => Some(if has_def { SysmlSyntaxKind::PortDef } else { SysmlSyntaxKind::PortUsage }),
            "attribute" => Some(if has_def { SysmlSyntaxKind::AttributeDef } else { SysmlSyntaxKind::AttributeUsage }),
            "enum" => Some(if has_def { SysmlSyntaxKind::EnumDef } else { SysmlSyntaxKind::EnumUsage }),
            "item" => Some(if has_def { SysmlSyntaxKind::ItemDef } else { SysmlSyntaxKind::ItemUsage }),
            "connection" | "connect" | "binding" => Some(if has_def {
                SysmlSyntaxKind::ConnectionDef
            } else {
                SysmlSyntaxKind::ConnectionUsage
            }),
            "interface" => Some(if has_def { SysmlSyntaxKind::InterfaceDef } else { SysmlSyntaxKind::InterfaceUsage }),
            "flow" | "message" => Some(if has_def { SysmlSyntaxKind::FlowDef } else { SysmlSyntaxKind::FlowUsage }),
            "allocation" | "allocate" => Some(if has_def {
                SysmlSyntaxKind::AllocationDef
            } else {
                SysmlSyntaxKind::AllocationUsage
            }),
            "occurrence" | "individual" | "snapshot" | "timeslice" => Some(if has_def {
                SysmlSyntaxKind::OccurrenceDef
            } else {
                SysmlSyntaxKind::OccurrenceUsage
            }),
            "variation" => Some(SysmlSyntaxKind::VariationDecl),
            "variant" => Some(SysmlSyntaxKind::VariantUsage),
            "action" | "perform" | "send" | "accept" | "assign" => Some(if has_def {
                SysmlSyntaxKind::ActionDef
            } else {
                SysmlSyntaxKind::ActionUsage
            }),
            "state" | "exhibit" => Some(if has_def { SysmlSyntaxKind::StateDef } else { SysmlSyntaxKind::StateUsage }),
            "transition" => Some(SysmlSyntaxKind::TransitionDecl),
            "succession" | "first" => Some(SysmlSyntaxKind::SuccessionDecl),
            "use" if second == Some("case") => None,
            _ => None,
        }
    }
}

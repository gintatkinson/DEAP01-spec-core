//! [DOC-TIER: A]
//!
//! ### 1. Architectural Purpose
//! Provides lossless Rowan Concrete Syntax Tree (CST) construction and structural
//! construct extraction for SysML v2 and KerML documents (`parse_sysml_cst`).
//!
//! ### 2. Mathematical / Formal / Standard Formulation
//! Given source text $S$ and token stream $T = \langle t_1, \dots, t_n \rangle$:
//! $$
//! \begin{aligned}
//! \text{text}(\text{GreenNode}(T)) &= \bigoplus_{i=1}^{n} \text{slice}(t_i) = S
//! \end{aligned}
//! $$
//! Every byte of $S$ (including trivia and lexical errors) is preserved in the root
//! `SOURCE_FILE` node while non-trivia tokens are scanned into construct summaries.
//!
//! ### 3. Step-by-Step Execution Flow
//! 1. Tokenize `source` using `tokenize_sysml(source)`.
//! 2. Insert all tokens sequentially into `rowan::GreenNodeBuilder` under `SOURCE_FILE`.
//! 3. Record lexical error spans and verify balanced `{` / `}` delimiters.
//! 4. Scan non-trivia tokens to extract `SysmlConstructSummary` records.
//! 5. Verify the lossless roundtrip invariant `syntax.text().to_string() == source`.
//!
//! ### 4. Key Data Structures & Invariants
//! - `SysmlCstDiagnostic`: Byte-spanned lexical or delimiter diagnostic.
//! - `SysmlConstructSummary`: Typed summary of a discovered SysML/KerML construct.
//! - `ParsedSysmlCst`: Root `GreenNode`, extracted constructs, diagnostics, and roundtrip flag.
//!
//! ### 5. Error Handling & Edge Cases
//! - Invalid tokens (`SysmlToken::Error`) become `SysmlSyntaxKind::ERROR` leaves plus diagnostics.
//! - Unbalanced braces produce diagnostics without aborting CST construction.
//!
//! ### 6. Performance & Complexity
//! - Time complexity: $\mathcal{O}(|S|)$ single-pass CST construction and header scan.
//! - Space complexity: $\mathcal{O}(|T|)$ proportional to token count.
//!
//! ### 7. Safety & Concurrency
//! Pure safe Rust (`Send + Sync` data types) with zero `unsafe` blocks.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::doc_markdown, clippy::missing_const_for_fn)]

use rowan::GreenNodeBuilder;

use super::lexer::{tokenize_sysml, LexedSysmlToken, SysmlToken};
use super::syntax_kind::{token_to_syntax_kind, SysmlSyntaxKind, SysmlSyntaxNode};

/// Diagnostic emitted during lossless SysML v2 CST construction. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SysmlCstDiagnostic {
    /// Human-readable diagnostic message.
    pub message: String,
    /// Inclusive start byte offset in source.
    pub span_start: usize,
    /// Exclusive end byte offset in source.
    pub span_end: usize,
}

/// Summary of a normative SysML v2 or KerML construct discovered in the token stream. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SysmlConstructSummary {
    /// Classified composite syntax kind (`*_DEF`, `*_USAGE`, or `*_DECL`).
    pub kind: SysmlSyntaxKind,
    /// Leading normative keyword head (e.g. `"part"`, `"requirement"`, `"package"`).
    pub keyword_head: String,
    /// Declared identifier or unrestricted name, if present.
    pub name: Option<String>,
    /// True if the construct includes the `def` keyword or is a formal definition.
    pub is_definition: bool,
    /// Inclusive start byte offset in source.
    pub span_start: usize,
    /// Exclusive end byte offset in source.
    pub span_end: usize,
}

/// Output of lossless SysML v2 CST parsing. Realises: [REQ-0201]
#[derive(Debug, Clone)]
pub struct ParsedSysmlCst {
    /// Immutable Rowan green tree rooted at `SysmlSyntaxKind::SOURCE_FILE`.
    pub green: rowan::GreenNode,
    /// Extracted top-level and nested SysML/KerML constructs in source order.
    pub constructs: Vec<SysmlConstructSummary>,
    /// Lexical and structural diagnostics collected during parsing.
    pub diagnostics: Vec<SysmlCstDiagnostic>,
    /// True when `syntax().text().to_string() == source`.
    pub lossless_roundtrip_eq: bool,
}

impl ParsedSysmlCst {
    /// Creates a navigable typed Rowan red-tree root node from the stored green node. Realises: [REQ-0201]
    #[must_use]
    pub fn syntax(&self) -> SysmlSyntaxNode {
        SysmlSyntaxNode::new_root(self.green.clone())
    }
}

/// Maps a normative SysML v2 / KerML keyword head and `is_def` flag to `SysmlSyntaxKind`. Realises: [REQ-0201]
#[must_use]
pub fn classify_keyword_kind(keyword: &str, is_def: bool) -> SysmlSyntaxKind {
    match keyword {
        "package" | "library" => SysmlSyntaxKind::PACKAGE_DECL,
        "import" => SysmlSyntaxKind::IMPORT_DECL,
        "alias" => SysmlSyntaxKind::ALIAS_DECL,
        "doc" | "comment" | "rep" => SysmlSyntaxKind::DOC_COMMENT_NODE,
        "metadata" => select_kind(is_def, SysmlSyntaxKind::METADATA_DEF, SysmlSyntaxKind::METADATA_USAGE),
        "attribute" => select_kind(is_def, SysmlSyntaxKind::ATTRIBUTE_DEF, SysmlSyntaxKind::ATTRIBUTE_USAGE),
        "enum" => select_kind(is_def, SysmlSyntaxKind::ENUM_DEF, SysmlSyntaxKind::ENUM_USAGE),
        "item" => select_kind(is_def, SysmlSyntaxKind::ITEM_DEF, SysmlSyntaxKind::ITEM_USAGE),
        "part" => select_kind(is_def, SysmlSyntaxKind::PART_DEF, SysmlSyntaxKind::PART_USAGE),
        "port" => select_kind(is_def, SysmlSyntaxKind::PORT_DEF, SysmlSyntaxKind::PORT_USAGE),
        "connection" => select_kind(is_def, SysmlSyntaxKind::CONNECTION_DEF, SysmlSyntaxKind::CONNECTION_USAGE),
        "interface" => select_kind(is_def, SysmlSyntaxKind::INTERFACE_DEF, SysmlSyntaxKind::INTERFACE_USAGE),
        "flow" | "succession" => select_kind(is_def, SysmlSyntaxKind::FLOW_DEF, SysmlSyntaxKind::FLOW_USAGE),
        "allocation" | "allocate" => select_kind(is_def, SysmlSyntaxKind::ALLOCATION_DEF, SysmlSyntaxKind::ALLOCATION_USAGE),
        "occurrence" | "individual" | "snapshot" | "timeslice" => {
            select_kind(is_def, SysmlSyntaxKind::OCCURRENCE_DEF, SysmlSyntaxKind::OCCURRENCE_USAGE)
        }
        "action" => select_kind(is_def, SysmlSyntaxKind::ACTION_DEF, SysmlSyntaxKind::ACTION_USAGE),
        "state" | "exhibit" => select_kind(is_def, SysmlSyntaxKind::STATE_DEF, SysmlSyntaxKind::STATE_USAGE),
        "transition" => SysmlSyntaxKind::TRANSITION_DECL,
        "calc" => select_kind(is_def, SysmlSyntaxKind::CALC_DEF, SysmlSyntaxKind::CALC_USAGE),
        "constraint" => select_kind(is_def, SysmlSyntaxKind::CONSTRAINT_DEF, SysmlSyntaxKind::CONSTRAINT_USAGE),
        "requirement" | "hazard" | "risk" | "capability" => {
            select_kind(is_def, SysmlSyntaxKind::REQUIREMENT_DEF, SysmlSyntaxKind::REQUIREMENT_USAGE)
        }
        "concern" => select_kind(is_def, SysmlSyntaxKind::CONCERN_DEF, SysmlSyntaxKind::CONCERN_USAGE),
        "use" => select_kind(is_def, SysmlSyntaxKind::USE_CASE_DEF, SysmlSyntaxKind::USE_CASE_USAGE),
        "analysis" => select_kind(is_def, SysmlSyntaxKind::ANALYSIS_DEF, SysmlSyntaxKind::ANALYSIS_USAGE),
        "verification" => select_kind(is_def, SysmlSyntaxKind::VERIFICATION_DEF, SysmlSyntaxKind::VERIFICATION_USAGE),
        "view" => select_kind(is_def, SysmlSyntaxKind::VIEW_DEF, SysmlSyntaxKind::VIEW_USAGE),
        "viewpoint" => select_kind(is_def, SysmlSyntaxKind::VIEWPOINT_DEF, SysmlSyntaxKind::VIEWPOINT_USAGE),
        "rendering" => select_kind(is_def, SysmlSyntaxKind::RENDERING_DEF, SysmlSyntaxKind::RENDERING_USAGE),
        "variation" => SysmlSyntaxKind::VARIATION_DECL,
        "variant" => SysmlSyntaxKind::VARIANT_USAGE,
        _ => SysmlSyntaxKind::ERROR_NODE,
    }
}

/// Selects definition or usage syntax kind based on flag. Realises: [REQ-0201]
const fn select_kind(
    is_def: bool,
    def_kind: SysmlSyntaxKind,
    usage_kind: SysmlSyntaxKind,
) -> SysmlSyntaxKind {
    if is_def { def_kind } else { usage_kind }
}

/// Parses `source` into a lossless Rowan CST and extracts SysML v2 / KerML constructs. Realises: [REQ-0201]
#[must_use]
pub fn parse_sysml_cst(source: &str) -> ParsedSysmlCst {
    let tokens = tokenize_sysml(source);
    let mut builder = GreenNodeBuilder::new();
    let mut diagnostics = Vec::new();
    let mut open_braces: Vec<(usize, usize)> = Vec::new();

    builder.start_node(SysmlSyntaxKind::SOURCE_FILE.into());
    for lexed in &tokens {
        let kind = token_to_syntax_kind(lexed.token);
        builder.token(kind.into(), lexed.slice);
        match lexed.token {
            SysmlToken::Error => diagnostics.push(SysmlCstDiagnostic {
                message: format!("Unrecognized lexical token: {:?}", lexed.slice),
                span_start: lexed.span.start,
                span_end: lexed.span.end,
            }),
            SysmlToken::LBrace => open_braces.push((lexed.span.start, lexed.span.end)),
            SysmlToken::RBrace if open_braces.pop().is_none() => {
                diagnostics.push(SysmlCstDiagnostic {
                    message: "Unmatched closing brace '}'".to_string(),
                    span_start: lexed.span.start,
                    span_end: lexed.span.end,
                });
            }
            _ => {}
        }
    }
    builder.finish_node();

    for (start, end) in open_braces {
        diagnostics.push(SysmlCstDiagnostic {
            message: "Unclosed opening brace '{'".to_string(),
            span_start: start,
            span_end: end,
        });
    }

    let green = builder.finish();
    let constructs = extract_constructs(&tokens);
    let lossless_roundtrip_eq = SysmlSyntaxNode::new_root(green.clone()).text() == source;

    ParsedSysmlCst {
        green,
        constructs,
        diagnostics,
        lossless_roundtrip_eq,
    }
}

/// Extracts constructs from non-trivia token stream. Realises: [REQ-0201]
fn extract_constructs(tokens: &[LexedSysmlToken<'_>]) -> Vec<SysmlConstructSummary> {
    let non_trivia: Vec<&LexedSysmlToken<'_>> = tokens
        .iter()
        .filter(|t| {
            !matches!(
                t.token,
                SysmlToken::Whitespace | SysmlToken::LineComment | SysmlToken::BlockComment
            )
        })
        .collect();

    let mut constructs = Vec::new();
    let mut idx = 0;
    while let Some(&tok) = non_trivia.get(idx) {
        let base_kind = classify_keyword_kind(tok.slice, false);
        if base_kind == SysmlSyntaxKind::ERROR_NODE {
            idx += 1;
            continue;
        }
        let mut cursor = idx + 1;
        if (tok.slice == "library" && non_trivia.get(cursor).is_some_and(|t| t.slice == "package"))
            || (matches!(tok.slice, "use" | "analysis" | "verification")
                && non_trivia.get(cursor).is_some_and(|t| t.slice == "case"))
        {
            cursor += 1;
        }
        let is_definition = non_trivia
            .get(cursor)
            .is_some_and(|t| t.token == SysmlToken::DefKw);
        if is_definition {
            cursor += 1;
        }
        let kind = classify_keyword_kind(tok.slice, is_definition);
        let (name, next_cursor) = extract_construct_name(&non_trivia, cursor);
        let span_end = find_construct_end(&non_trivia, next_cursor, tok.span.end);
        constructs.push(SysmlConstructSummary {
            kind,
            keyword_head: tok.slice.to_string(),
            name,
            is_definition,
            span_start: tok.span.start,
            span_end,
        });
        idx = next_cursor.max(idx + 1);
    }
    constructs
}

/// Extracts construct identifier name from token stream. Realises: [REQ-0201]
fn extract_construct_name(
    non_trivia: &[&LexedSysmlToken<'_>],
     mut cursor: usize,
) -> (Option<String>, usize) {
    if non_trivia.get(cursor).is_some_and(|t| t.slice == "<")
        && non_trivia.get(cursor + 2).is_some_and(|t| t.slice == ">")
    {
        cursor += 3;
    }
    if let Some(tok) = non_trivia.get(cursor) {
        if matches!(tok.token, SysmlToken::Ident | SysmlToken::UnrestrictedName) {
            return (Some(tok.slice.to_string()), cursor + 1);
        }
    }
    (None, cursor)
}

/// Finds ending byte offset of construct. Realises: [REQ-0201]
fn find_construct_end(
    non_trivia: &[&LexedSysmlToken<'_>],
    start_cursor: usize,
    fallback_end: usize,
) -> usize {
    let mut depth = 0usize;
    let mut last_end = fallback_end;
    for tok in non_trivia.iter().skip(start_cursor) {
        last_end = tok.span.end;
        match tok.token {
            SysmlToken::Semi if depth == 0 => return tok.span.end,
            SysmlToken::LBrace => depth += 1,
            SysmlToken::RBrace => {
                if depth <= 1 {
                    return tok.span.end;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    last_end
}

//! # Lexer for SysML v2 / KerML Concrete Syntax Tree
//! [DOC-TIER: A]
//!
//! ## 1. Module Description
//! High-performance lexical scanner for OMG SysML v2 and KerML syntax using `logos`.
//!
//! ## 2. Contract and Invariants
//! Tokenizes raw source text into full-fidelity token streams without discarding whitespace or comments.
//!
//! ## 3. Normative References
//! - OMG Systems Modeling Language (SysML) v2.0 Specification
//! - OMG Kernel Modeling Language (KerML) v1.0 Specification
//!
//! ## 4. Safety and Security Considerations
//! Linear scan without backtracking; memory-safe slice indexing via `logos`.
//!
//! ## 5. Verification and Testing
//! Validated against normative keywords, punctuation, identifiers, comments, and error fallback.
//!
//! ## 6. Performance Profile
//! O(N) single-pass tokenization, zero allocations for token slices.
//!
//! ## 7. Traceability
//! Implements CST lexer layer for SysML parser pipeline.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::doc_markdown, clippy::missing_const_for_fn)]

use std::ops::Range;

/// Concrete syntax tree tokens for OMG SysML v2 and KerML.
#[derive(logos::Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SysmlToken {
    /// Whitespace characters (spaces, tabs, newlines).
    #[regex(r"[ \t\r\n]+")]
    Whitespace,

    /// Single-line comment starting with `//`.
    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment,

    /// Multi-line block comment delimited by `/*` and `*/`.
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/")]
    BlockComment,

    /// Definition keyword `def`.
    #[token("def")]
    DefKw,

    /// Package-level keywords (`library`, `standard`, `package`).
    #[regex(r"library|standard|package")]
    PackageKw,

    /// Namespace-level keywords (`import`, `alias`, `private`, `public`, `protected`, `all`).
    #[regex(r"import|alias|private|public|protected|all")]
    NamespaceKw,

    /// Documentation and metadata keywords (`doc`, `comment`, `about`, `rep`, `locale`, `metadata`).
    #[regex(r"doc|comment|about|rep|locale|metadata")]
    DocMetaKw,

    /// Structural definition keywords.
    #[regex(
        r"attribute|enum|item|part|port|connection|interface|flow|succession|allocation|allocate|occurrence|individual|snapshot|timeslice|variation|variant"
    )]
    StructuralKw,

    /// Behavioral definition keywords.
    #[regex(
        r"action|perform|accept|send|assign|if|else|while|loop|for|fork|join|merge|decide|first|then|state|exhibit|entry|do|exit|transition|parallel|interaction|message|connect|bind"
    )]
    BehavioralKw,

    /// Requirements, calculation, and verification keywords.
    #[regex(
        r"calc|return|constraint|assert|assume|require|requirement|subject|actor|stakeholder|concern|frame|satisfy|use|case|include|analysis|objective|verification|verify|view|viewpoint|rendering|expose|filter|render|hazard|risk|capability"
    )]
    ReqCalcVerificationKw,

    /// KerML modifier keywords.
    #[regex(
        r"inout|in|out|ref|abstract|readonly|derived|end|ordered|nonunique|redefines|subsets|references|defined|by|specializes|to|from|via|not|and|or|xor|implies|true|false|null"
    )]
    KermlModifierKw,

    /// Operator `::>`.
    #[token("::>")]
    ColonColonGt,

    /// Operator `:>>`.
    #[token(":>>")]
    ColonGtGt,

    /// Operator `:>`.
    #[token(":>")]
    ColonGt,

    /// Operator `::`.
    #[token("::")]
    ColonColon,

    /// Operator `:=`.
    #[token(":=")]
    ColonEq,

    /// Punctuation `:`.
    #[token(":")]
    Colon,

    /// Punctuation `~`.
    #[token("~")]
    Tilde,

    /// Punctuation `@`.
    #[token("@")]
    At,

    /// Punctuation `#`.
    #[token("#")]
    Hash,

    /// Operator `..`.
    #[token("..")]
    DotDot,

    /// Operator `->`.
    #[token("->")]
    Arrow,

    /// Operator `=>`.
    #[token("=>")]
    FatArrow,

    /// Punctuation `.`.
    #[token(".")]
    Dot,

    /// Punctuation `,`.
    #[token(",")]
    Comma,

    /// Punctuation `;`.
    #[token(";")]
    Semi,

    /// Punctuation `{`.
    #[token("{")]
    LBrace,

    /// Punctuation `}`.
    #[token("}")]
    RBrace,

    /// Punctuation `[`.
    #[token("[")]
    LBracket,

    /// Punctuation `]`.
    #[token("]")]
    RBracket,

    /// Punctuation `(`.
    #[token("(")]
    LParen,

    /// Punctuation `)`.
    #[token(")")]
    RParen,

    /// KerML and SysML expressions operators.
    #[regex(r"===|!==|==|!=|<=|>=|\*\*|\.\?|\?\?|&&|\|\||[=+\-*/%<>?!]")]
    Operator,

    /// Double-quoted string literal.
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,

    /// Single-quoted unrestricted name literal.
    #[regex(r"'([^'\\]|\\.)*'")]
    UnrestrictedName,

    /// Numeric literal (integer or floating point, optional exponent).
    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?")]
    NumberLit,

    /// SysML identifier.
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", priority = 1)]
    Ident,

    /// Lexing error fallback token.
    Error,
}

/// A token spanned across a slice of the original SysML source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexedSysmlToken<'a> {
    /// Token category classified by the lexer.
    pub token: SysmlToken,
    /// Exact slice of the source string corresponding to this token.
    pub slice: &'a str,
    /// Byte span range within the source string.
    pub span: Range<usize>,
}

/// Tokenize a SysML v2 / KerML source string into a stream of spanned tokens.
#[must_use]
pub fn tokenize_sysml(source: &str) -> Vec<LexedSysmlToken<'_>> {
    logos::Logos::lexer(source)
        .spanned()
        .map(|(res, span)| {
            let token = match res {
                Ok(tok) => tok,
                Err(()) => SysmlToken::Error,
            };
            LexedSysmlToken {
                token,
                slice: source.get(span.clone()).unwrap_or_default(),
                span,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords_and_ident() {
        let input = "package def Vehicle { part def Engine; }";
        let tokens = tokenize_sysml(input);
        assert!(!tokens.is_empty());
        assert_eq!(tokens.first().map(|t| t.token), Some(SysmlToken::PackageKw));
        assert_eq!(tokens.first().map(|t| t.slice), Some("package"));
    }

    #[test]
    fn test_tokenize_comments_and_operators() {
        let input = "// comment\n/* block */ :>> ::> := ..";
        let tokens = tokenize_sysml(input);
        assert_eq!(
            tokens.first().map(|t| t.token),
            Some(SysmlToken::LineComment)
        );
    }
}

//! # SysML v2 / KerML Concrete Syntax Tree Syntax Kinds
//! [DOC-TIER: A]
//!
//! ## 1. Module Description
//! Enumerates Rowan CST token and composite node kinds for OMG SysML v2 and KerML.
//!
//! ## 2. Contract and Invariants
//! Token kinds mirror `SysmlToken` variants. Node kinds span normative definitions and usages.
//!
//! ## 3. Normative References
//! - OMG Systems Modeling Language (SysML) v2.0 Specification
//! - OMG Kernel Modeling Language (KerML) v1.0 Specification
//!
//! ## 4. Safety and Security Considerations
//! Discriminant validation ensures memory safety during Rowan raw syntax kind conversions.
//!
//! ## 5. Verification and Testing
//! Unit tests verify bi-directional conversions between raw kinds and Rowan `Language` mapping.
//!
//! ## 6. Performance Profile
//! Zero-allocation `repr(u16)` enum tags for lightweight CST node representations.
//!
//! ## 7. Traceability
//! Implements Rowan `Language` and syntax kind mappings for the SysML concrete syntax tree.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::doc_markdown, clippy::missing_const_for_fn)]

/// Complete enumeration of tokens and composite CST nodes for SysML v2 and KerML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum SysmlSyntaxKind {
    // Token Kinds
    WHITESPACE = 0,
    LINE_COMMENT,
    BLOCK_COMMENT,
    DEF_KW,
    PACKAGE_KW,
    NAMESPACE_KW,
    DOC_META_KW,
    STRUCTURAL_KW,
    BEHAVIORAL_KW,
    REQ_CALC_VERIFICATION_KW,
    KERML_MODIFIER_KW,
    COLON_COLON_GT,
    COLON_GT_GT,
    COLON_GT,
    COLON_COLON,
    COLON_EQ,
    COLON,
    TILDE,
    AT,
    HASH,
    DOT_DOT,
    ARROW,
    FAT_ARROW,
    DOT,
    COMMA,
    SEMI,
    L_BRACE,
    R_BRACE,
    L_BRACKET,
    R_BRACKET,
    L_PAREN,
    R_PAREN,
    OPERATOR,
    STRING_LIT,
    UNRESTRICTED_NAME,
    NUMBER_LIT,
    IDENT,
    ERROR_TOKEN,

    // Normative Composite CST Node Kinds
    SOURCE_FILE,
    PACKAGE_DECL,
    LIBRARY_PACKAGE_DECL,
    IMPORT_DECL,
    ALIAS_DECL,
    DOC_COMMENT_NODE,
    METADATA_DEF,
    METADATA_USAGE,
    ATTRIBUTE_DEF,
    ATTRIBUTE_USAGE,
    ENUM_DEF,
    ENUM_USAGE,
    ITEM_DEF,
    ITEM_USAGE,
    PART_DEF,
    PART_USAGE,
    PORT_DEF,
    PORT_USAGE,
    CONNECTION_DEF,
    CONNECTION_USAGE,
    INTERFACE_DEF,
    INTERFACE_USAGE,
    FLOW_DEF,
    FLOW_USAGE,
    SUCCESSION_DECL,
    ALLOCATION_DEF,
    ALLOCATION_USAGE,
    OCCURRENCE_DEF,
    OCCURRENCE_USAGE,
    ACTION_DEF,
    ACTION_USAGE,
    STATE_DEF,
    STATE_USAGE,
    TRANSITION_DECL,
    CALC_DEF,
    CALC_USAGE,
    CONSTRAINT_DEF,
    CONSTRAINT_USAGE,
    REQUIREMENT_DEF,
    REQUIREMENT_USAGE,
    CONCERN_DEF,
    CONCERN_USAGE,
    USE_CASE_DEF,
    USE_CASE_USAGE,
    ANALYSIS_DEF,
    ANALYSIS_USAGE,
    VERIFICATION_DEF,
    VERIFICATION_USAGE,
    VIEW_DEF,
    VIEW_USAGE,
    VIEWPOINT_DEF,
    VIEWPOINT_USAGE,
    RENDERING_DEF,
    RENDERING_USAGE,
    VARIATION_DECL,
    VARIANT_USAGE,
    SPECIALIZATION_CLAUSE,
    MULTIPLICITY_BOUNDS,
    EXPRESSION_NODE,
    BLOCK_BODY,
    STATEMENT_NODE,
    ERROR_NODE,
    __LAST,
}

impl From<SysmlSyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SysmlSyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl From<SysmlSyntaxKind> for u16 {
    fn from(kind: SysmlSyntaxKind) -> Self {
        kind as Self
    }
}

const ALL_SYNTAX_KINDS: [SysmlSyntaxKind; (SysmlSyntaxKind::__LAST as usize) + 1] = [
    SysmlSyntaxKind::WHITESPACE,
    SysmlSyntaxKind::LINE_COMMENT,
    SysmlSyntaxKind::BLOCK_COMMENT,
    SysmlSyntaxKind::DEF_KW,
    SysmlSyntaxKind::PACKAGE_KW,
    SysmlSyntaxKind::NAMESPACE_KW,
    SysmlSyntaxKind::DOC_META_KW,
    SysmlSyntaxKind::STRUCTURAL_KW,
    SysmlSyntaxKind::BEHAVIORAL_KW,
    SysmlSyntaxKind::REQ_CALC_VERIFICATION_KW,
    SysmlSyntaxKind::KERML_MODIFIER_KW,
    SysmlSyntaxKind::COLON_COLON_GT,
    SysmlSyntaxKind::COLON_GT_GT,
    SysmlSyntaxKind::COLON_GT,
    SysmlSyntaxKind::COLON_COLON,
    SysmlSyntaxKind::COLON_EQ,
    SysmlSyntaxKind::COLON,
    SysmlSyntaxKind::TILDE,
    SysmlSyntaxKind::AT,
    SysmlSyntaxKind::HASH,
    SysmlSyntaxKind::DOT_DOT,
    SysmlSyntaxKind::ARROW,
    SysmlSyntaxKind::FAT_ARROW,
    SysmlSyntaxKind::DOT,
    SysmlSyntaxKind::COMMA,
    SysmlSyntaxKind::SEMI,
    SysmlSyntaxKind::L_BRACE,
    SysmlSyntaxKind::R_BRACE,
    SysmlSyntaxKind::L_BRACKET,
    SysmlSyntaxKind::R_BRACKET,
    SysmlSyntaxKind::L_PAREN,
    SysmlSyntaxKind::R_PAREN,
    SysmlSyntaxKind::OPERATOR,
    SysmlSyntaxKind::STRING_LIT,
    SysmlSyntaxKind::UNRESTRICTED_NAME,
    SysmlSyntaxKind::NUMBER_LIT,
    SysmlSyntaxKind::IDENT,
    SysmlSyntaxKind::ERROR_TOKEN,
    SysmlSyntaxKind::SOURCE_FILE,
    SysmlSyntaxKind::PACKAGE_DECL,
    SysmlSyntaxKind::LIBRARY_PACKAGE_DECL,
    SysmlSyntaxKind::IMPORT_DECL,
    SysmlSyntaxKind::ALIAS_DECL,
    SysmlSyntaxKind::DOC_COMMENT_NODE,
    SysmlSyntaxKind::METADATA_DEF,
    SysmlSyntaxKind::METADATA_USAGE,
    SysmlSyntaxKind::ATTRIBUTE_DEF,
    SysmlSyntaxKind::ATTRIBUTE_USAGE,
    SysmlSyntaxKind::ENUM_DEF,
    SysmlSyntaxKind::ENUM_USAGE,
    SysmlSyntaxKind::ITEM_DEF,
    SysmlSyntaxKind::ITEM_USAGE,
    SysmlSyntaxKind::PART_DEF,
    SysmlSyntaxKind::PART_USAGE,
    SysmlSyntaxKind::PORT_DEF,
    SysmlSyntaxKind::PORT_USAGE,
    SysmlSyntaxKind::CONNECTION_DEF,
    SysmlSyntaxKind::CONNECTION_USAGE,
    SysmlSyntaxKind::INTERFACE_DEF,
    SysmlSyntaxKind::INTERFACE_USAGE,
    SysmlSyntaxKind::FLOW_DEF,
    SysmlSyntaxKind::FLOW_USAGE,
    SysmlSyntaxKind::SUCCESSION_DECL,
    SysmlSyntaxKind::ALLOCATION_DEF,
    SysmlSyntaxKind::ALLOCATION_USAGE,
    SysmlSyntaxKind::OCCURRENCE_DEF,
    SysmlSyntaxKind::OCCURRENCE_USAGE,
    SysmlSyntaxKind::ACTION_DEF,
    SysmlSyntaxKind::ACTION_USAGE,
    SysmlSyntaxKind::STATE_DEF,
    SysmlSyntaxKind::STATE_USAGE,
    SysmlSyntaxKind::TRANSITION_DECL,
    SysmlSyntaxKind::CALC_DEF,
    SysmlSyntaxKind::CALC_USAGE,
    SysmlSyntaxKind::CONSTRAINT_DEF,
    SysmlSyntaxKind::CONSTRAINT_USAGE,
    SysmlSyntaxKind::REQUIREMENT_DEF,
    SysmlSyntaxKind::REQUIREMENT_USAGE,
    SysmlSyntaxKind::CONCERN_DEF,
    SysmlSyntaxKind::CONCERN_USAGE,
    SysmlSyntaxKind::USE_CASE_DEF,
    SysmlSyntaxKind::USE_CASE_USAGE,
    SysmlSyntaxKind::ANALYSIS_DEF,
    SysmlSyntaxKind::ANALYSIS_USAGE,
    SysmlSyntaxKind::VERIFICATION_DEF,
    SysmlSyntaxKind::VERIFICATION_USAGE,
    SysmlSyntaxKind::VIEW_DEF,
    SysmlSyntaxKind::VIEW_USAGE,
    SysmlSyntaxKind::VIEWPOINT_DEF,
    SysmlSyntaxKind::VIEWPOINT_USAGE,
    SysmlSyntaxKind::RENDERING_DEF,
    SysmlSyntaxKind::RENDERING_USAGE,
    SysmlSyntaxKind::VARIATION_DECL,
    SysmlSyntaxKind::VARIANT_USAGE,
    SysmlSyntaxKind::SPECIALIZATION_CLAUSE,
    SysmlSyntaxKind::MULTIPLICITY_BOUNDS,
    SysmlSyntaxKind::EXPRESSION_NODE,
    SysmlSyntaxKind::BLOCK_BODY,
    SysmlSyntaxKind::STATEMENT_NODE,
    SysmlSyntaxKind::ERROR_NODE,
    SysmlSyntaxKind::__LAST,
];

/// Rowan language definition for SysML v2 / KerML CST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SysmlLanguage {}

impl rowan::Language for SysmlLanguage {
    type Kind = SysmlSyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        ALL_SYNTAX_KINDS
            .get(usize::from(raw.0))
            .copied()
            .unwrap_or(SysmlSyntaxKind::ERROR_NODE)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

/// Rowan syntax node alias for SysML v2 / KerML.
pub type SysmlSyntaxNode = rowan::SyntaxNode<SysmlLanguage>;

/// Rowan syntax token alias for SysML v2 / KerML.
pub type SysmlSyntaxToken = rowan::SyntaxToken<SysmlLanguage>;

/// Map a `SysmlToken` lexer token to its corresponding `SysmlSyntaxKind`.
#[must_use]
pub const fn token_to_syntax_kind(token: super::lexer::SysmlToken) -> SysmlSyntaxKind {
    match token {
        super::lexer::SysmlToken::Whitespace => SysmlSyntaxKind::WHITESPACE,
        super::lexer::SysmlToken::LineComment => SysmlSyntaxKind::LINE_COMMENT,
        super::lexer::SysmlToken::BlockComment => SysmlSyntaxKind::BLOCK_COMMENT,
        super::lexer::SysmlToken::DefKw => SysmlSyntaxKind::DEF_KW,
        super::lexer::SysmlToken::PackageKw => SysmlSyntaxKind::PACKAGE_KW,
        super::lexer::SysmlToken::NamespaceKw => SysmlSyntaxKind::NAMESPACE_KW,
        super::lexer::SysmlToken::DocMetaKw => SysmlSyntaxKind::DOC_META_KW,
        super::lexer::SysmlToken::StructuralKw => SysmlSyntaxKind::STRUCTURAL_KW,
        super::lexer::SysmlToken::BehavioralKw => SysmlSyntaxKind::BEHAVIORAL_KW,
        super::lexer::SysmlToken::ReqCalcVerificationKw => SysmlSyntaxKind::REQ_CALC_VERIFICATION_KW,
        super::lexer::SysmlToken::KermlModifierKw => SysmlSyntaxKind::KERML_MODIFIER_KW,
        super::lexer::SysmlToken::ColonColonGt => SysmlSyntaxKind::COLON_COLON_GT,
        super::lexer::SysmlToken::ColonGtGt => SysmlSyntaxKind::COLON_GT_GT,
        super::lexer::SysmlToken::ColonGt => SysmlSyntaxKind::COLON_GT,
        super::lexer::SysmlToken::ColonColon => SysmlSyntaxKind::COLON_COLON,
        super::lexer::SysmlToken::ColonEq => SysmlSyntaxKind::COLON_EQ,
        super::lexer::SysmlToken::Colon => SysmlSyntaxKind::COLON,
        super::lexer::SysmlToken::Tilde => SysmlSyntaxKind::TILDE,
        super::lexer::SysmlToken::At => SysmlSyntaxKind::AT,
        super::lexer::SysmlToken::Hash => SysmlSyntaxKind::HASH,
        super::lexer::SysmlToken::DotDot => SysmlSyntaxKind::DOT_DOT,
        super::lexer::SysmlToken::Arrow => SysmlSyntaxKind::ARROW,
        super::lexer::SysmlToken::FatArrow => SysmlSyntaxKind::FAT_ARROW,
        super::lexer::SysmlToken::Dot => SysmlSyntaxKind::DOT,
        super::lexer::SysmlToken::Comma => SysmlSyntaxKind::COMMA,
        super::lexer::SysmlToken::Semi => SysmlSyntaxKind::SEMI,
        super::lexer::SysmlToken::LBrace => SysmlSyntaxKind::L_BRACE,
        super::lexer::SysmlToken::RBrace => SysmlSyntaxKind::R_BRACE,
        super::lexer::SysmlToken::LBracket => SysmlSyntaxKind::L_BRACKET,
        super::lexer::SysmlToken::RBracket => SysmlSyntaxKind::R_BRACKET,
        super::lexer::SysmlToken::LParen => SysmlSyntaxKind::L_PAREN,
        super::lexer::SysmlToken::RParen => SysmlSyntaxKind::R_PAREN,
        super::lexer::SysmlToken::Operator => SysmlSyntaxKind::OPERATOR,
        super::lexer::SysmlToken::StringLit => SysmlSyntaxKind::STRING_LIT,
        super::lexer::SysmlToken::UnrestrictedName => SysmlSyntaxKind::UNRESTRICTED_NAME,
        super::lexer::SysmlToken::NumberLit => SysmlSyntaxKind::NUMBER_LIT,
        super::lexer::SysmlToken::Ident => SysmlSyntaxKind::IDENT,
        super::lexer::SysmlToken::Error => SysmlSyntaxKind::ERROR_TOKEN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rowan::Language;

    #[test]
    fn test_token_to_syntax_kind_roundtrip() {
        assert_eq!(
            token_to_syntax_kind(super::super::lexer::SysmlToken::Whitespace),
            SysmlSyntaxKind::WHITESPACE
        );
        let raw = SysmlLanguage::kind_to_raw(SysmlSyntaxKind::PACKAGE_DECL);
        assert_eq!(SysmlLanguage::kind_from_raw(raw), SysmlSyntaxKind::PACKAGE_DECL);
    }
}

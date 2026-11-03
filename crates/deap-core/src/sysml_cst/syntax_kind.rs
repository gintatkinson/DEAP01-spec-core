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

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

/// Complete enumeration of tokens and composite CST nodes for SysML v2 and KerML. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SysmlSyntaxKind {
    // Token Kinds
    Whitespace = 0,
    LineComment,
    BlockComment,
    DefKw,
    PackageKw,
    NamespaceKw,
    DocMetaKw,
    StructuralKw,
    BehavioralKw,
    ReqCalcVerificationKw,
    KermlModifierKw,
    ColonColonGt,
    ColonGtGt,
    ColonGt,
    ColonColon,
    ColonEq,
    Colon,
    Tilde,
    At,
    Hash,
    DotDot,
    Arrow,
    FatArrow,
    Dot,
    Comma,
    Semi,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Operator,
    StringLit,
    UnrestrictedName,
    NumberLit,
    Ident,
    ErrorToken,

    // Normative Composite CST Node Kinds
    SourceFile,
    PackageDecl,
    LibraryPackageDecl,
    ImportDecl,
    AliasDecl,
    DocCommentNode,
    MetadataDef,
    MetadataUsage,
    AttributeDef,
    AttributeUsage,
    EnumDef,
    EnumUsage,
    ItemDef,
    ItemUsage,
    PartDef,
    PartUsage,
    PortDef,
    PortUsage,
    ConnectionDef,
    ConnectionUsage,
    InterfaceDef,
    InterfaceUsage,
    FlowDef,
    FlowUsage,
    SuccessionDecl,
    AllocationDef,
    AllocationUsage,
    OccurrenceDef,
    OccurrenceUsage,
    ActionDef,
    ActionUsage,
    StateDef,
    StateUsage,
    TransitionDecl,
    CalcDef,
    CalcUsage,
    ConstraintDef,
    ConstraintUsage,
    RequirementDef,
    RequirementUsage,
    ConcernDef,
    ConcernUsage,
    UseCaseDef,
    UseCaseUsage,
    AnalysisDef,
    AnalysisUsage,
    VerificationDef,
    VerificationUsage,
    ViewDef,
    ViewUsage,
    ViewpointDef,
    ViewpointUsage,
    RenderingDef,
    RenderingUsage,
    VariationDecl,
    VariantUsage,
    SpecializationClause,
    MultiplicityBounds,
    ExpressionNode,
    BlockBody,
    StatementNode,
    ErrorNode,
    Last,
}

impl From<SysmlSyntaxKind> for rowan::SyntaxKind {
    /// Converts SysmlSyntaxKind to rowan::SyntaxKind. Realises: [REQ-0201]
    fn from(kind: SysmlSyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl From<SysmlSyntaxKind> for u16 {
    /// Converts SysmlSyntaxKind to raw u16. Realises: [REQ-0201]
    fn from(kind: SysmlSyntaxKind) -> Self {
        kind as Self
    }
}

const ALL_SYNTAX_KINDS: [SysmlSyntaxKind; (SysmlSyntaxKind::Last as usize) + 1] = [
    SysmlSyntaxKind::Whitespace, SysmlSyntaxKind::LineComment, SysmlSyntaxKind::BlockComment,
    SysmlSyntaxKind::DefKw, SysmlSyntaxKind::PackageKw, SysmlSyntaxKind::NamespaceKw,
    SysmlSyntaxKind::DocMetaKw, SysmlSyntaxKind::StructuralKw, SysmlSyntaxKind::BehavioralKw,
    SysmlSyntaxKind::ReqCalcVerificationKw, SysmlSyntaxKind::KermlModifierKw,
    SysmlSyntaxKind::ColonColonGt, SysmlSyntaxKind::ColonGtGt, SysmlSyntaxKind::ColonGt,
    SysmlSyntaxKind::ColonColon, SysmlSyntaxKind::ColonEq, SysmlSyntaxKind::Colon,
    SysmlSyntaxKind::Tilde, SysmlSyntaxKind::At, SysmlSyntaxKind::Hash,
    SysmlSyntaxKind::DotDot, SysmlSyntaxKind::Arrow, SysmlSyntaxKind::FatArrow,
    SysmlSyntaxKind::Dot, SysmlSyntaxKind::Comma, SysmlSyntaxKind::Semi,
    SysmlSyntaxKind::LBrace, SysmlSyntaxKind::RBrace, SysmlSyntaxKind::LBracket,
    SysmlSyntaxKind::RBracket, SysmlSyntaxKind::LParen, SysmlSyntaxKind::RParen,
    SysmlSyntaxKind::Operator, SysmlSyntaxKind::StringLit, SysmlSyntaxKind::UnrestrictedName,
    SysmlSyntaxKind::NumberLit, SysmlSyntaxKind::Ident, SysmlSyntaxKind::ErrorToken,
    SysmlSyntaxKind::SourceFile, SysmlSyntaxKind::PackageDecl, SysmlSyntaxKind::LibraryPackageDecl,
    SysmlSyntaxKind::ImportDecl, SysmlSyntaxKind::AliasDecl, SysmlSyntaxKind::DocCommentNode,
    SysmlSyntaxKind::MetadataDef, SysmlSyntaxKind::MetadataUsage, SysmlSyntaxKind::AttributeDef,
    SysmlSyntaxKind::AttributeUsage, SysmlSyntaxKind::EnumDef, SysmlSyntaxKind::EnumUsage,
    SysmlSyntaxKind::ItemDef, SysmlSyntaxKind::ItemUsage, SysmlSyntaxKind::PartDef,
    SysmlSyntaxKind::PartUsage, SysmlSyntaxKind::PortDef, SysmlSyntaxKind::PortUsage,
    SysmlSyntaxKind::ConnectionDef, SysmlSyntaxKind::ConnectionUsage, SysmlSyntaxKind::InterfaceDef,
    SysmlSyntaxKind::InterfaceUsage, SysmlSyntaxKind::FlowDef, SysmlSyntaxKind::FlowUsage,
    SysmlSyntaxKind::SuccessionDecl, SysmlSyntaxKind::AllocationDef, SysmlSyntaxKind::AllocationUsage,
    SysmlSyntaxKind::OccurrenceDef, SysmlSyntaxKind::OccurrenceUsage, SysmlSyntaxKind::ActionDef,
    SysmlSyntaxKind::ActionUsage, SysmlSyntaxKind::StateDef, SysmlSyntaxKind::StateUsage,
    SysmlSyntaxKind::TransitionDecl, SysmlSyntaxKind::CalcDef, SysmlSyntaxKind::CalcUsage,
    SysmlSyntaxKind::ConstraintDef, SysmlSyntaxKind::ConstraintUsage, SysmlSyntaxKind::RequirementDef,
    SysmlSyntaxKind::RequirementUsage, SysmlSyntaxKind::ConcernDef, SysmlSyntaxKind::ConcernUsage,
    SysmlSyntaxKind::UseCaseDef, SysmlSyntaxKind::UseCaseUsage, SysmlSyntaxKind::AnalysisDef,
    SysmlSyntaxKind::AnalysisUsage, SysmlSyntaxKind::VerificationDef, SysmlSyntaxKind::VerificationUsage,
    SysmlSyntaxKind::ViewDef, SysmlSyntaxKind::ViewUsage, SysmlSyntaxKind::ViewpointDef,
    SysmlSyntaxKind::ViewpointUsage, SysmlSyntaxKind::RenderingDef, SysmlSyntaxKind::RenderingUsage,
    SysmlSyntaxKind::VariationDecl, SysmlSyntaxKind::VariantUsage, SysmlSyntaxKind::SpecializationClause,
    SysmlSyntaxKind::MultiplicityBounds, SysmlSyntaxKind::ExpressionNode, SysmlSyntaxKind::BlockBody,
    SysmlSyntaxKind::StatementNode, SysmlSyntaxKind::ErrorNode, SysmlSyntaxKind::Last,
];

/// Rowan language definition for SysML v2 / KerML CST. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SysmlLanguage {}

impl rowan::Language for SysmlLanguage {
    type Kind = SysmlSyntaxKind;

    /// Converts rowan::SyntaxKind to SysmlSyntaxKind. Realises: [REQ-0201]
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        ALL_SYNTAX_KINDS
            .get(usize::from(raw.0))
            .copied()
            .unwrap_or(SysmlSyntaxKind::ErrorNode)
    }

    /// Converts SysmlSyntaxKind to rowan::SyntaxKind. Realises: [REQ-0201]
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

/// Rowan syntax node alias for SysML v2 / KerML.
pub type SysmlSyntaxNode = rowan::SyntaxNode<SysmlLanguage>;

/// Rowan syntax token alias for SysML v2 / KerML. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
pub type SysmlSyntaxToken = rowan::SyntaxToken<SysmlLanguage>;

/// Rowan syntax element alias (node or token) for SysML v2 / KerML. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
pub type SysmlSyntaxElement = rowan::SyntaxElement<SysmlLanguage>;

impl SysmlSyntaxKind {
    /// Alias for [`Self::SourceFile`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const SOURCE_FILE: Self = Self::SourceFile;
    /// Alias for [`Self::PackageDecl`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const PACKAGE_DEF: Self = Self::PackageDecl;
    /// Alias for [`Self::PartDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const PART_DEF: Self = Self::PartDef;
    /// Alias for [`Self::ActionDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const ACTION_DEF: Self = Self::ActionDef;
    /// Alias for [`Self::StateDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const STATE_DEF: Self = Self::StateDef;
    /// Alias for [`Self::PortDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const PORT_DEF: Self = Self::PortDef;
    /// Alias for [`Self::ConstraintDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const CONSTRAINT_DEF: Self = Self::ConstraintDef;
    /// Alias for [`Self::RequirementDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const REQUIREMENT_DEF: Self = Self::RequirementDef;
    /// Alias for [`Self::UseCaseDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const USE_CASE_DEF: Self = Self::UseCaseDef;
    /// Alias for [`Self::VerificationDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const TEST_CASE_DEF: Self = Self::VerificationDef;
    /// Alias for [`Self::CalcDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const CALC_DEF: Self = Self::CalcDef;
    /// Alias for [`Self::ConnectionDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const CONNECTION_DEF: Self = Self::ConnectionDef;
    /// Alias for [`Self::EnumDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const ENUM_DEF: Self = Self::EnumDef;
    /// Alias for [`Self::AttributeDef`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const ATTRIBUTE_DEF: Self = Self::AttributeDef;
    /// Alias for [`Self::DocCommentNode`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const DOC_COMMENT: Self = Self::DocCommentNode;
    /// Alias for [`Self::ErrorNode`]. Realises: [REQ-0201] [FLS: fls_8tsynkj2cufj]
    pub const ERROR_NODE: Self = Self::ErrorNode;

    /// Returns `true` when `self` is a composite CST node kind rather than a leaf token kind.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Distinguishes composite Rowan syntax nodes from leaf lexical tokens under DO-178C Level A.
    /// ## 2. Invariants & Mathematical Properties
    /// `self as u16 >= Self::SourceFile as u16 && self != Self::Last`.
    /// ## 3. Preconditions
    /// `self` is a valid `SysmlSyntaxKind` variant.
    /// ## 4. Postconditions
    /// Returns `true` iff `self` is a composite CST node kind.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1); Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total pure discriminant comparison with zero panics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub const fn is_composite_node(self) -> bool {
        (self as u16) >= (Self::SourceFile as u16) && (self as u16) < (Self::Last as u16)
    }

    /// Returns `true` when `self` is a whitespace or comment trivia token kind.
    /// [DOC-TIER: A]
    /// ## 1. Safety Intent & Regulatory Scope
    /// Identifies lossless trivia tokens for CST lookahead under DO-178C Level A.
    /// ## 2. Invariants & Mathematical Properties
    /// Matches `Whitespace`, `LineComment`, and `BlockComment`.
    /// ## 3. Preconditions
    /// `self` is a valid `SysmlSyntaxKind` variant.
    /// ## 4. Postconditions
    /// Returns `true` iff `self` is trivia.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1); Space Complexity: O(1).
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total pure pattern match with zero panics.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0201] | [FLS: fls_9ucqbbd0s2yo]
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::LineComment | Self::BlockComment)
    }
}

/// Map a `SysmlToken` lexer token to its corresponding `SysmlSyntaxKind`. Realises: [REQ-0201]
#[must_use]
pub const fn token_to_syntax_kind(token: super::lexer::SysmlToken) -> SysmlSyntaxKind {
    match token {
        super::lexer::SysmlToken::Whitespace => SysmlSyntaxKind::Whitespace,
        super::lexer::SysmlToken::LineComment => SysmlSyntaxKind::LineComment,
        super::lexer::SysmlToken::BlockComment => SysmlSyntaxKind::BlockComment,
        super::lexer::SysmlToken::DefKw => SysmlSyntaxKind::DefKw,
        super::lexer::SysmlToken::PackageKw => SysmlSyntaxKind::PackageKw,
        super::lexer::SysmlToken::NamespaceKw => SysmlSyntaxKind::NamespaceKw,
        super::lexer::SysmlToken::DocMetaKw => SysmlSyntaxKind::DocMetaKw,
        super::lexer::SysmlToken::StructuralKw => SysmlSyntaxKind::StructuralKw,
        super::lexer::SysmlToken::BehavioralKw => SysmlSyntaxKind::BehavioralKw,
        super::lexer::SysmlToken::ReqCalcVerificationKw => SysmlSyntaxKind::ReqCalcVerificationKw,
        super::lexer::SysmlToken::KermlModifierKw => SysmlSyntaxKind::KermlModifierKw,
        super::lexer::SysmlToken::ColonColonGt => SysmlSyntaxKind::ColonColonGt,
        super::lexer::SysmlToken::ColonGtGt => SysmlSyntaxKind::ColonGtGt,
        super::lexer::SysmlToken::ColonGt => SysmlSyntaxKind::ColonGt,
        super::lexer::SysmlToken::ColonColon => SysmlSyntaxKind::ColonColon,
        super::lexer::SysmlToken::ColonEq => SysmlSyntaxKind::ColonEq,
        super::lexer::SysmlToken::Colon => SysmlSyntaxKind::Colon,
        super::lexer::SysmlToken::Tilde => SysmlSyntaxKind::Tilde,
        super::lexer::SysmlToken::At => SysmlSyntaxKind::At,
        super::lexer::SysmlToken::Hash => SysmlSyntaxKind::Hash,
        super::lexer::SysmlToken::DotDot => SysmlSyntaxKind::DotDot,
        super::lexer::SysmlToken::Arrow => SysmlSyntaxKind::Arrow,
        super::lexer::SysmlToken::FatArrow => SysmlSyntaxKind::FatArrow,
        super::lexer::SysmlToken::Dot => SysmlSyntaxKind::Dot,
        super::lexer::SysmlToken::Comma => SysmlSyntaxKind::Comma,
        super::lexer::SysmlToken::Semi => SysmlSyntaxKind::Semi,
        super::lexer::SysmlToken::LBrace => SysmlSyntaxKind::LBrace,
        super::lexer::SysmlToken::RBrace => SysmlSyntaxKind::RBrace,
        super::lexer::SysmlToken::LBracket => SysmlSyntaxKind::LBracket,
        super::lexer::SysmlToken::RBracket => SysmlSyntaxKind::RBracket,
        super::lexer::SysmlToken::LParen => SysmlSyntaxKind::LParen,
        super::lexer::SysmlToken::RParen => SysmlSyntaxKind::RParen,
        super::lexer::SysmlToken::Operator => SysmlSyntaxKind::Operator,
        super::lexer::SysmlToken::StringLit => SysmlSyntaxKind::StringLit,
        super::lexer::SysmlToken::UnrestrictedName => SysmlSyntaxKind::UnrestrictedName,
        super::lexer::SysmlToken::NumberLit => SysmlSyntaxKind::NumberLit,
        super::lexer::SysmlToken::Ident => SysmlSyntaxKind::Ident,
        super::lexer::SysmlToken::Error => SysmlSyntaxKind::ErrorToken,
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
            SysmlSyntaxKind::Whitespace
        );
        let raw = SysmlLanguage::kind_to_raw(SysmlSyntaxKind::PackageDecl);
        assert_eq!(SysmlLanguage::kind_from_raw(raw), SysmlSyntaxKind::PackageDecl);
    }
}

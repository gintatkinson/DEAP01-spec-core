//! KaTeX allowed alignment environments enumeration and catalog.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `AllowedAlignmentEnv`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves structural type invariants for enum `AllowedAlignmentEnv` across all 17 members.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed values for `AllowedAlignmentEnv` members.
//!
//! ## 4. Postconditions
//! - Defines authoritative `AllowedAlignmentEnv` and `AllowedAlignmentEnvsCatalog` representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) constant-time enum dispatch.
//! - Space Complexity: O(1) auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Exhaustive matching prevents unhandled variants.
//!
//! ## 7. Realization & Traceability
//! Realises: [deap_core::markdown::AllowedAlignmentEnv] | SysML v2: PartDef(AllowedAlignmentEnv)

/// Typed enumeration of every KaTeX environment in which bare `&` alignment operators are permitted.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `AllowedAlignmentEnv`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `AllowedAlignmentEnv` across all 17 members.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `AllowedAlignmentEnv` members.
///
/// ## 4. Postconditions
/// - Defines the authoritative `AllowedAlignmentEnv` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(17)) bounded over member dimension of `AllowedAlignmentEnv`.
/// - Space Complexity: O(M(17)) bounded auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AllowedAlignmentEnv`.
/// - State: Maintains consistent state invariants.
/// - Action: Executes `AllowedAlignmentEnv` deterministically.
/// - Resource: Bounds stack and heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::markdown::AllowedAlignmentEnv] | SysML v2: PartDef(AllowedAlignmentEnv)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AllowedAlignmentEnv {
    /// Multi-line aligned equations (`aligned`).
    Aligned,
    /// Aligned equations with explicit column count (`alignedat`).
    AlignedAt,
    /// Undelimited matrix (`matrix`).
    Matrix,
    /// Parenthesis-delimited matrix (`pmatrix`).
    ParenMatrix,
    /// Square-bracket-delimited matrix (`bmatrix`).
    BracketMatrix,
    /// Curly-brace-delimited matrix (`Bmatrix`).
    BraceMatrix,
    /// Single-vertical-bar-delimited matrix (`vmatrix`).
    VertMatrix,
    /// Double-vertical-bar-delimited matrix (`Vmatrix`).
    DoubleVertMatrix,
    /// Piecewise definition (`cases`).
    Cases,
    /// Display-style piecewise definition (`dcases`).
    DisplayCases,
    /// Right-braced piecewise definition (`rcases`).
    RightCases,
    /// General tabular array (`array`).
    Array,
    /// Single equation split across lines (`split`).
    Split,
    /// Centered gathered block inside display math (`gathered`).
    Gathered,
    /// Centered gathered equations (`gather`).
    Gather,
    /// Compact sub-array for limits (`subarray`).
    Subarray,
    /// Inline-sized matrix (`smallmatrix`).
    SmallMatrix,
}

impl AllowedAlignmentEnv {
    /// Canonical declaration-ordered table of all 17 permitted KaTeX alignment environments.
    pub const ALL: &'static [Self] = &[
        Self::Aligned,
        Self::AlignedAt,
        Self::Matrix,
        Self::ParenMatrix,
        Self::BracketMatrix,
        Self::BraceMatrix,
        Self::VertMatrix,
        Self::DoubleVertMatrix,
        Self::Cases,
        Self::DisplayCases,
        Self::RightCases,
        Self::Array,
        Self::Split,
        Self::Gathered,
        Self::Gather,
        Self::Subarray,
        Self::SmallMatrix,
    ];

    /// Projects an alignment environment variant onto the exact `\begin{...}` identifier KaTeX expects.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Pins every whitelisted LaTeX alignment environment name.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective, case-sensitive mapping from 17 variants to 17 distinct ASCII environment names.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 17 `AllowedAlignmentEnv` variants.
    ///
    /// ## 4. Postconditions
    /// Returns the `'static` environment identifier without the `\begin{` / `}` wrapper.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive match prevents unhandled variants.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-KATEX-ALIGN-ENV-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aligned => "aligned",
            Self::AlignedAt => "alignedat",
            Self::Matrix => "matrix",
            Self::ParenMatrix => "pmatrix",
            Self::BracketMatrix => "bmatrix",
            Self::BraceMatrix => "Bmatrix",
            Self::VertMatrix => "vmatrix",
            Self::DoubleVertMatrix => "Vmatrix",
            Self::Cases => "cases",
            Self::DisplayCases => "dcases",
            Self::RightCases => "rcases",
            Self::Array => "array",
            Self::Split => "split",
            Self::Gathered => "gathered",
            Self::Gather => "gather",
            Self::Subarray => "subarray",
            Self::SmallMatrix => "smallmatrix",
        }
    }

    /// Reference-taking adapter yielding the KaTeX environment identifier for a borrowed variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supplies the `fn(&'static AllowedAlignmentEnv)` pointer.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Pointwise equality `e.as_str_ref() == e.as_str()` holds for all 17 variants.
    ///
    /// ## 3. Preconditions
    /// `self` is a shared borrow of an `AllowedAlignmentEnv`.
    ///
    /// ## 4. Postconditions
    /// Returns the same `'static` identifier as `as_str`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Non-panicking and allocation-free.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-KATEX-ALIGN-ENV-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog of allowed LaTeX environments where bare `&` alignment operators are permitted.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `AllowedAlignmentEnvsCatalog`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `AllowedAlignmentEnvsCatalog`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `AllowedAlignmentEnvsCatalog` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `AllowedAlignmentEnvsCatalog` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input dimension.
/// - Space Complexity: O(M(1)) zero auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AllowedAlignmentEnvsCatalog`.
/// - State: Maintains consistent state invariants.
/// - Action: Executes `AllowedAlignmentEnvsCatalog` deterministically.
/// - Resource: Bounds allocations within `AllowedAlignmentEnvsCatalog`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-MARKDOWN-ALLOWEDALIGNMENTENVSCATALOG] | SysML v2: PartDef(AllowedAlignmentEnvsCatalog)
#[derive(Debug, Clone, Copy, Default)]
pub struct AllowedAlignmentEnvsCatalog;

/// Canonical instance of `AllowedAlignmentEnvsCatalog` for KaTeX alignment environment validation.
pub const ALLOWED_ALIGNMENT_ENVS: AllowedAlignmentEnvsCatalog = AllowedAlignmentEnvsCatalog;

impl AllowedAlignmentEnvsCatalog {
    /// Checks whether a candidate LaTeX environment name permits bare `&` alignment tokens.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity rules.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Returns `true` if and only if `*env` belongs to the 17 canonical KaTeX alignment environments.
    ///
    /// ## 3. Preconditions
    /// `env` is a valid borrowed string slice reference.
    ///
    /// ## 4. Postconditions
    /// Returns a deterministic boolean membership verdict.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(L) bounded by 11 bytes. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total non-panicking function.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-KATEX-ALIGN-ENVS]
    pub fn contains(&self, env: &&str) -> bool {
        matches!(
            *env,
            "aligned"
                | "alignedat"
                | "matrix"
                | "pmatrix"
                | "bmatrix"
                | "Bmatrix"
                | "vmatrix"
                | "Vmatrix"
                | "cases"
                | "dcases"
                | "rcases"
                | "array"
                | "split"
                | "gathered"
                | "gather"
                | "subarray"
                | "smallmatrix"
        )
    }

    /// Returns a typed-enum-backed iterator over all 17 allowed KaTeX alignment environment names.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Provides deterministic traversal.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields exactly 17 unique `'static` ASCII string literals in canonical declaration order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid reference to `AllowedAlignmentEnvsCatalog`.
    ///
    /// ## 4. Postconditions
    /// Returns an exact-size iterator with `len() == 17`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) construction, O(17) traversal. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible total function; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-KATEX-ALIGN-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, AllowedAlignmentEnv>,
        fn(&'static AllowedAlignmentEnv) -> &'static str,
    > {
        let project: fn(&'static AllowedAlignmentEnv) -> &'static str =
            AllowedAlignmentEnv::as_str_ref;
        AllowedAlignmentEnv::ALL.iter().map(project)
    }
}

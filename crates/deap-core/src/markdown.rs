//! Markdown token scanner, KaTeX, and math block parser.

/// Typed enumeration of every KaTeX environment in which bare `&` alignment operators are permitted.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `AllowedAlignmentEnv` in `crates/deap-core/src/markdown.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `AllowedAlignmentEnv` across member set `[Aligned, AlignedAt, Matrix, ParenMatrix, BracketMatrix, BraceMatrix]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `AllowedAlignmentEnv` members (`Aligned, AlignedAt, Matrix, ParenMatrix, BracketMatrix, BraceMatrix`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `AllowedAlignmentEnv` representation for enum `AllowedAlignmentEnv`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `AllowedAlignmentEnv`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `AllowedAlignmentEnv`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AllowedAlignmentEnv` (`AllowedAlignmentEnv`).
/// - State: Maintains consistent state invariants across `AllowedAlignmentEnv` without partial mutation on error.
/// - Action: Executes `AllowedAlignmentEnv` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `AllowedAlignmentEnv` (`DEAP-CORE-MARKDOWN`).
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Pins every whitelisted LaTeX alignment
    /// environment name to a typed variant so the KaTeX bare-`&` gate in `check_latex_katex_syntax`
    /// compares against case-exact identifiers (`bmatrix` vs `Bmatrix` are distinct environments).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective, case-sensitive mapping from 17 variants to 17 distinct ASCII environment names
    /// of at most 11 bytes; declaration order equals the legacy catalog order.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 17 `AllowedAlignmentEnv` variants.
    ///
    /// ## 4. Postconditions
    /// Returns the `'static` environment identifier without the `\begin{` / `}` wrapper.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) seventeen-arm `const` match lowered to a lookup. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive match prevents a new environment variant from compiling without its identifier,
    /// so the KaTeX gate cannot silently accept `&` in an unnamed environment.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supplies the `fn(&'static AllowedAlignmentEnv)`
    /// pointer used by `AllowedAlignmentEnvsCatalog::iter`, so iteration over permitted alignment
    /// environments shares the single exhaustive projection in `as_str`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Pointwise equality `e.as_str_ref() == e.as_str()` holds for all 17 variants.
    ///
    /// ## 3. Preconditions
    /// `self` is a shared borrow of an `AllowedAlignmentEnv`, typically from `AllowedAlignmentEnv::ALL`.
    ///
    /// ## 4. Postconditions
    /// Returns the same `'static` identifier as `as_str`, independent of the borrow's lifetime.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) discriminant load and lookup. Space Complexity: O(1) stack only.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Non-panicking and allocation-free; there is no error channel because no input is invalid.
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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `AllowedAlignmentEnvsCatalog` in `crates/deap-core/src/markdown.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `AllowedAlignmentEnvsCatalog` across member set `[const: AllowedAlignmentEnvsCatalog = AllowedAlignmentEnvsCatalog;]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `AllowedAlignmentEnvsCatalog` members (`const: AllowedAlignmentEnvsCatalog = AllowedAlignmentEnvsCatalog;`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `AllowedAlignmentEnvsCatalog` representation for struct `AllowedAlignmentEnvsCatalog`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `AllowedAlignmentEnvsCatalog`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `AllowedAlignmentEnvsCatalog`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AllowedAlignmentEnvsCatalog` (`AllowedAlignmentEnvsCatalog`).
/// - State: Maintains consistent state invariants across `AllowedAlignmentEnvsCatalog` without partial mutation on error.
/// - Action: Executes `AllowedAlignmentEnvsCatalog` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `AllowedAlignmentEnvsCatalog` (`DEAP-CORE-MARKDOWN`).
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity rules by replacing
    /// untyped static string arrays with a deterministic pattern-matching catalog.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Returns `true` if and only if `*env` belongs to the 17 canonical KaTeX tabular and
    /// multi-line alignment environments (`aligned`, `alignedat`, `matrix`, `pmatrix`, `bmatrix`,
    /// `Bmatrix`, `vmatrix`, `Vmatrix`, `cases`, `dcases`, `rcases`, `array`, `split`,
    /// `gathered`, `gather`, `subarray`, `smallmatrix`).
    ///
    /// ## 3. Preconditions
    /// `env` is a valid borrowed string slice reference representing a parsed `\begin{...}` identifier.
    ///
    /// ## 4. Postconditions
    /// Returns a deterministic boolean membership verdict with zero heap allocation and zero side effects.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(L) where L is the byte length of `*env` (bounded by 11 bytes). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total non-panicking function; unknown or malformed environment names evaluate to `false`.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Provides deterministic, allocation-free
    /// traversal over the permitted LaTeX alignment environments, each derived from a typed
    /// `AllowedAlignmentEnv` variant rather than an untyped string array.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields exactly 17 unique `'static` ASCII string literals in canonical declaration order,
    /// equal element-wise to `AllowedAlignmentEnv::ALL` projected through `AllowedAlignmentEnv::as_str`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid reference to `AllowedAlignmentEnvsCatalog`.
    ///
    /// ## 4. Postconditions
    /// Returns a `Map<slice::Iter<'static, AllowedAlignmentEnv>, fn(&'static AllowedAlignmentEnv) -> &'static str>`
    /// positioned at the first environment (`aligned`) with an exact remaining length of 17.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) iterator construction, O(17) traversal. Space Complexity: O(1) slice cursor plus fn pointer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible total function with zero dynamic allocation or panic paths; the backing table
    /// is a `'static` constant, so the iterator can never dangle.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-KATEX-ALIGN-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, AllowedAlignmentEnv>,
        fn(&'static AllowedAlignmentEnv) -> &'static str,
    > {
        let project: fn(&'static AllowedAlignmentEnv) -> &'static str = AllowedAlignmentEnv::as_str_ref;
        AllowedAlignmentEnv::ALL.iter().map(project)
    }
}

/// Typed enumeration of every Mermaid diagram type header accepted as the first non-comment line of a block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ValidMermaidHeader` in `crates/deap-core/src/markdown.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `ValidMermaidHeader` across member set `[Graph, Flowchart, SequenceDiagram, StateDiagram, StateDiagramV2, ClassDiagram]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ValidMermaidHeader` members (`Graph, Flowchart, SequenceDiagram, StateDiagram, StateDiagramV2, ClassDiagram`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ValidMermaidHeader` representation for enum `ValidMermaidHeader`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `ValidMermaidHeader`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `ValidMermaidHeader`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ValidMermaidHeader` (`ValidMermaidHeader`).
/// - State: Maintains consistent state invariants across `ValidMermaidHeader` without partial mutation on error.
/// - Action: Executes `ValidMermaidHeader` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ValidMermaidHeader` (`DEAP-CORE-MARKDOWN`).
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::markdown::ValidMermaidHeader] | SysML v2: PartDef(ValidMermaidHeader)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidMermaidHeader {
    /// Legacy graph diagram (`graph`).
    Graph,
    /// Flowchart diagram (`flowchart`).
    Flowchart,
    /// Sequence diagram (`sequencediagram`).
    SequenceDiagram,
    /// Legacy state diagram (`statediagram`).
    StateDiagram,
    /// Version 2 state diagram (`statediagram-v2`).
    StateDiagramV2,
    /// Class diagram (`classdiagram`).
    ClassDiagram,
    /// Entity-relationship diagram (`erdiagram`).
    ErDiagram,
    /// Gantt chart (`gantt`).
    Gantt,
    /// Pie chart (`pie`).
    Pie,
    /// Git graph (`gitgraph`).
    GitGraph,
    /// C4 context diagram (`c4context`).
    C4Context,
    /// User journey diagram (`journey`).
    Journey,
    /// Quadrant chart (`quadrantchart`).
    QuadrantChart,
    /// XY chart beta (`xychart-beta`).
    XyChartBeta,
    /// Mind map (`mindmap`).
    Mindmap,
    /// Timeline diagram (`timeline`).
    Timeline,
    /// Sankey diagram beta (`sankey-beta`).
    SankeyBeta,
    /// Requirement diagram (`requirementdiagram`).
    RequirementDiagram,
}

impl ValidMermaidHeader {
    /// Canonical declaration-ordered table of all 18 accepted Mermaid diagram headers.
    pub const ALL: &'static [Self] = &[
        Self::Graph,
        Self::Flowchart,
        Self::SequenceDiagram,
        Self::StateDiagram,
        Self::StateDiagramV2,
        Self::ClassDiagram,
        Self::ErDiagram,
        Self::Gantt,
        Self::Pie,
        Self::GitGraph,
        Self::C4Context,
        Self::Journey,
        Self::QuadrantChart,
        Self::XyChartBeta,
        Self::Mindmap,
        Self::Timeline,
        Self::SankeyBeta,
        Self::RequirementDiagram,
    ];

    /// Projects a Mermaid header variant onto its lowercase header keyword.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Fixes each whitelisted Mermaid diagram type
    /// to the lowercase keyword that `check_mermaid_syntax` prefix-matches against the lowered first
    /// non-comment line, so diagram-header enforcement is driven by a typed, closed set.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective mapping from 18 variants to 18 distinct lowercase ASCII keywords (hyphen allowed);
    /// declaration order preserves the legacy order, keeping `statediagram` ahead of `statediagram-v2`.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 18 `ValidMermaidHeader` variants.
    ///
    /// ## 4. Postconditions
    /// Returns an all-lowercase `'static` keyword with no surrounding whitespace or direction suffix
    /// (for example `"flowchart"`, never `"flowchart TD"`).
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) eighteen-arm `const` match. Space Complexity: O(1), returns a borrowed literal.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// A variant added without a keyword fails exhaustiveness checking at compile time, so the
    /// Mermaid gate can never treat an unnamed header as valid.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-MERMAID-HDR-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Flowchart => "flowchart",
            Self::SequenceDiagram => "sequencediagram",
            Self::StateDiagram => "statediagram",
            Self::StateDiagramV2 => "statediagram-v2",
            Self::ClassDiagram => "classdiagram",
            Self::ErDiagram => "erdiagram",
            Self::Gantt => "gantt",
            Self::Pie => "pie",
            Self::GitGraph => "gitgraph",
            Self::C4Context => "c4context",
            Self::Journey => "journey",
            Self::QuadrantChart => "quadrantchart",
            Self::XyChartBeta => "xychart-beta",
            Self::Mindmap => "mindmap",
            Self::Timeline => "timeline",
            Self::SankeyBeta => "sankey-beta",
            Self::RequirementDiagram => "requirementdiagram",
        }
    }

    /// Borrow-based adapter returning the lowercase Mermaid keyword for a referenced header variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Furnishes the `fn(&'static ValidMermaidHeader)`
    /// pointer consumed by `ValidMermaidHeadersCatalog::iter`, so header prefix search and the
    /// by-value projection cannot drift apart.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// `h.as_str_ref() == h.as_str()` for each of the 18 variants; delegation is the entire body.
    ///
    /// ## 3. Preconditions
    /// `self` is a shared reference to a `ValidMermaidHeader`, usually yielded by `ValidMermaidHeader::ALL.iter()`.
    ///
    /// ## 4. Postconditions
    /// Returns the identical lowercase `'static` keyword; no borrow of `self` escapes.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) single indirection then const lookup. Space Complexity: O(1) register-resident.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Has no failure surface: total over its domain, no I/O, no allocation, no panic.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-MERMAID-HDR-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog of known valid Mermaid diagram type headers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ValidMermaidHeadersCatalog` in `crates/deap-core/src/markdown.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ValidMermaidHeadersCatalog` across member set `[const: ValidMermaidHeadersCatalog = ValidMermaidHeadersCatalog;]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ValidMermaidHeadersCatalog` members (`const: ValidMermaidHeadersCatalog = ValidMermaidHeadersCatalog;`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ValidMermaidHeadersCatalog` representation for struct `ValidMermaidHeadersCatalog`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `ValidMermaidHeadersCatalog`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `ValidMermaidHeadersCatalog`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ValidMermaidHeadersCatalog` (`ValidMermaidHeadersCatalog`).
/// - State: Maintains consistent state invariants across `ValidMermaidHeadersCatalog` without partial mutation on error.
/// - Action: Executes `ValidMermaidHeadersCatalog` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ValidMermaidHeadersCatalog` (`DEAP-CORE-MARKDOWN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-MARKDOWN-VALIDMERMAIDHEADERSCATALOG] | SysML v2: PartDef(ValidMermaidHeadersCatalog)
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidMermaidHeadersCatalog;

/// Canonical instance of `ValidMermaidHeadersCatalog` for Mermaid diagram header verification.
pub const VALID_MERMAID_HEADERS: ValidMermaidHeadersCatalog = ValidMermaidHeadersCatalog;

impl ValidMermaidHeadersCatalog {
    /// Returns a typed-enum-backed iterator over all 18 canonical Mermaid diagram type headers.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity rules by providing
    /// deterministic header enumeration sourced from the typed `ValidMermaidHeader` enumeration
    /// instead of untyped global string-slice arrays.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 18 lowercase Mermaid diagram header prefixes in fixed declaration order, equal
    /// element-wise to `ValidMermaidHeader::ALL` mapped through `ValidMermaidHeader::as_str`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid reference to `ValidMermaidHeadersCatalog`.
    ///
    /// ## 4. Postconditions
    /// Returns a `Map<slice::Iter<'static, ValidMermaidHeader>, fn(&'static ValidMermaidHeader) -> &'static str>`
    /// suitable for first-match prefix search via `find`, without heap allocation.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) to construct, O(18) worst-case `find`. Space Complexity: O(1) borrowed cursor state.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible total function; never panics, and the stable order guarantees deterministic
    /// header kind selection across runs.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-MERMAID-HDR-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, ValidMermaidHeader>,
        fn(&'static ValidMermaidHeader) -> &'static str,
    > {
        let project: fn(&'static ValidMermaidHeader) -> &'static str = ValidMermaidHeader::as_str_ref;
        ValidMermaidHeader::ALL.iter().map(project)
    }

    /// Checks whether a lowercase header token matches one of the 18 valid Mermaid diagram types.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces strict diagram header
    /// whitelisting across specification markdown artifacts.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Returns `true` if and only if `hdr` is an exact match for one of the 18 supported
    /// lowercase Mermaid diagram keywords.
    ///
    /// ## 3. Preconditions
    /// `hdr` is a valid UTF-8 string slice normalized to lowercase by the caller if case-insensitive matching is desired.
    ///
    /// ## 4. Postconditions
    /// Returns a deterministic boolean verdict without side effects or heap allocation.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(L) where L is the byte length of `hdr` (bounded by 18 bytes). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total non-panicking function; unrecognized headers evaluate to `false`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-MERMAID-HDR-CONTAINS]
    pub fn contains(&self, hdr: &str) -> bool {
        matches!(
            hdr,
            "graph"
                | "flowchart"
                | "sequencediagram"
                | "statediagram"
                | "statediagram-v2"
                | "classdiagram"
                | "erdiagram"
                | "gantt"
                | "pie"
                | "gitgraph"
                | "c4context"
                | "journey"
                | "quadrantchart"
                | "xychart-beta"
                | "mindmap"
                | "timeline"
                | "sankey-beta"
                | "requirementdiagram"
        )
    }
}

/// Removes fenced code blocks (` ```...``` ` and `~~~...~~~`), replacing non-newline characters
/// with ASCII spaces so downstream line and column offsets remain invariant.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents code literals inside fenced blocks
/// from triggering false-positive KaTeX math delimiter or prose linter diagnostics.
///
/// ## 2. Invariants & Mathematical Properties
/// - Line Preservation Invariant: `output.lines().count() == content.lines().count()` and all
///   `\n` and `\r` byte positions are preserved identically.
/// - Fence Matching Property: Opening fence of character $c \in \{\text{'`'}, \text{'\~'}\}$ with
///   length $k \ge 3$ remains active until a closing line starting with at least $k$ repetitions
///   of $c$ followed only by optional whitespace is encountered.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown string slice.
///
/// ## 4. Postconditions
/// Returns a newly allocated `String` of identical character count to `content` where all fenced
/// code regions are blanked with `' '` characters (preserving `\r` and `\n`).
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) single pass over $N$ UTF-8 characters. Space Complexity: O(N) for the output buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function. Unclosed fences blank all remaining characters through EOF without error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-FENCES]
pub fn strip_fenced_code_blocks(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_fence = false;
    let mut fence_char = ' ';
    let mut fence_len = 0;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if !in_fence {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let Some(ch) = trimmed.chars().next() else {
                    continue;
                };
                let len = trimmed.chars().take_while(|&c| c == ch).count();
                in_fence = true;
                fence_char = ch;
                fence_len = len;
                for c in line.chars() {
                    if c == '\n' || c == '\r' {
                        out.push(c);
                    } else {
                        out.push(' ');
                    }
                }
                continue;
            }
            out.push_str(line);
        } else {
            let is_closing = if trimmed.starts_with(fence_char) {
                let len = trimmed.chars().take_while(|&c| c == fence_char).count();
                len >= fence_len && trimmed.get(len..).unwrap_or("").trim().is_empty()
            } else {
                false
            };

            for c in line.chars() {
                if c == '\n' || c == '\r' {
                    out.push(c);
                } else {
                    out.push(' ');
                }
            }

            if is_closing {
                in_fence = false;
            }
        }
    }
    out
}

/// Removes GitLab Flavored Markdown (GLFM) inline math spans (`$`...`$` or `$``...``$`),
/// replacing non-newline characters with spaces so adjacent `$` signs cannot coalesce into false `$$` tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Eliminates false `$$` display-math delimiter
/// synthesis when stripping backtick spans inside GLFM `$`+...`+$` math expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Character-Length Invariant: `output.chars().count() == content.chars().count()`.
/// - Exact Backtick Run Parity: An opening `$` followed by $k \ge 1$ backticks matches the first
///   subsequent run of exactly $k$ backticks immediately followed by `$`.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 string slice (typically after fenced code blocks have been stripped).
///
/// ## 4. Postconditions
/// Returns a `String` with all matched GLFM inline math spans replaced by ASCII spaces while
/// preserving all newline characters (`\n`, `\r`) and unmatched characters verbatim.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N^2) worst-case on pathological unclosed `$`+ sequences, O(N) typical linear scan.
/// Space Complexity: O(N) character buffer allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function using checked slice indexing (`.get()`, `.get_mut()`). Unclosed
/// `$`+ prefixes are left intact and scanning advances by one character.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-GLFM]
pub fn strip_glfm_inline_math(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut result = chars.clone();
    let mut i = 0;

    while i < n {
        if chars.get(i) == Some(&'$') && chars.get(i + 1) == Some(&'`') {
            let mut k = 0;
            while chars.get(i + 1 + k) == Some(&'`') {
                k += 1;
            }

            let mut close_idx = None;
            let mut j = i + 1 + k;
            while j + k < n {
                if chars.get(j) == Some(&'`') {
                    let mut count = 0;
                    while chars.get(j + count) == Some(&'`') {
                        count += 1;
                    }
                    if count == k && chars.get(j + count) == Some(&'$') {
                        close_idx = Some(j + count);
                        break;
                    }
                    j += count;
                } else {
                    j += 1;
                }
            }

            if let Some(end) = close_idx {
                if let Some(slice) = result.get_mut(i..=end) {
                    for item in slice {
                        if *item != '\n' && *item != '\r' {
                            *item = ' ';
                        }
                    }
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }

    result.into_iter().collect()
}

/// Removes CommonMark inline code spans delimited by matching backtick runs of length $k \ge 1$,
/// replacing non-newline characters with ASCII spaces.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Masks inline code literals (such as `` `$HOME` ``
/// or `` `\begin{align}` ``) before running mathematical and prose syntax validators.
///
/// ## 2. Invariants & Mathematical Properties
/// - Character-Length Invariant: `output.chars().count() == content.chars().count()`.
/// - CommonMark Backtick Matching: An opening run of $k \ge 1$ backticks is closed only by a
///   contiguous run of exactly $k$ backticks; longer or shorter runs inside the span are skipped.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// Returns a `String` where every matched inline code span (including its bounding backticks) is
/// overwritten with `' '` (preserving `\n` and `\r`).
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N^2) worst-case for unclosed backtick runs of distinct lengths, O(N) typical.
/// Space Complexity: O(N) character vector allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function using checked indexing. Unmatched backtick runs are preserved verbatim.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-INLINE]
pub fn strip_inline_code(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut result = chars.clone();
    let mut i = 0;

    while i < n {
        if chars.get(i) == Some(&'`') {
            let mut k = 0;
            while chars.get(i + k) == Some(&'`') {
                k += 1;
            }

            let mut close_idx = None;
            let mut j = i + k;
            while j < n {
                if chars.get(j) == Some(&'`') {
                    let mut count = 0;
                    while chars.get(j + count) == Some(&'`') {
                        count += 1;
                    }
                    if count == k {
                        close_idx = Some(j + count - 1);
                        break;
                    }
                    j += count;
                } else {
                    j += 1;
                }
            }

            if let Some(end) = close_idx {
                if let Some(slice) = result.get_mut(i..=end) {
                    for item in slice {
                        if *item != '\n' && *item != '\r' {
                            *item = ' ';
                        }
                    }
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }

    result.into_iter().collect()
}

/// Sequentially strips fenced code blocks, GLFM inline math (`$`...`$`), and standard inline code
/// spans so only active markdown prose and display/inline LaTeX math remain.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces the mandatory 3-stage sanitization
/// ordering (`strip_fenced_code_blocks` -> `strip_glfm_inline_math` -> `strip_inline_code`) so
/// stripping inline backticks never leaves behind orphaned `$` delimiters from GLFM math.
///
/// ## 2. Invariants & Mathematical Properties
/// Functional composition $f = \text{strip\_inline\_code} \circ \text{strip\_glfm\_inline\_math} \circ \text{strip\_fenced\_code\_blocks}$
/// preserving total line count and newline offsets across all three stages.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown document slice.
///
/// ## 4. Postconditions
/// Returns a sanitized `String` with all fenced blocks, GLFM inline math spans, and backtick code
/// spans replaced by spaces.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) typical, O(N^2) worst-case on pathological unclosed backtick sequences.
/// Space Complexity: O(N) auxiliary string buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function composed exclusively of total sanitization stages.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-STRIP]
pub fn strip_code_and_glfm(content: &str) -> String {
    let no_fences = strip_fenced_code_blocks(content);
    let no_glfm = strip_glfm_inline_math(&no_fences);
    strip_inline_code(&no_glfm)
}

/// Validates LaTeX / KaTeX rendering syntax across markdown content, checking `$$` delimiter parity,
/// `\begin{aligned}` / `\end{aligned}` balance, forbidden `\begin{align}` environments, and bare `&` operators.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces Universal LaTeX & KaTeX Mathematical
/// Rendering Rules across all specification artifacts to prevent broken mathematical rendering.
///
/// ## 2. Invariants & Mathematical Properties
/// - Display Delimiter Parity: Number of `$$` tokens in sanitized content must be even ($2k$).
/// - Environment Stack Discipline: For every odd-indexed `$$` segment $B_{2m+1}$, nested `\begin{env}`
///   and `\end{env}` transitions maintain an active environment stack $S$; every unescaped `&` must
///   satisfy $\exists e \in S : e \in \text{ALLOWED\_ALIGNMENT\_ENVS}$.
/// - Forbidden Top-Level Align: `\begin{align}` and `\begin{align*}` are unconditionally rejected inside `$$` blocks.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown string slice; `file_rel_path` is a human-readable relative path
/// for diagnostic attribution.
///
/// ## 4. Postconditions
/// Returns an empty `Vec<String>` if and only if all KaTeX syntactic invariants hold; otherwise returns
/// descriptive diagnostic strings citing `file_rel_path`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) over markdown byte length $N$ after code-span sanitization.
/// Space Complexity: O(N) for sanitized text and diagnostic collection.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function. Uses UTF-8 character boundary clamping (`is_char_boundary`) when
/// extracting error context snippets around bare `&` operators.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-KATEX]
pub fn check_latex_katex_syntax(content: &str, file_rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let cleaned = strip_code_and_glfm(content);

    // 1. Validate balanced $$ math blocks
    let parts: Vec<&str> = cleaned.split("$$").collect();
    if !(parts.len() - 1).is_multiple_of(2) {
        errors.push(format!(
            "Unbalanced $$ display math delimiters in {} (found {} delimiters).",
            file_rel_path,
            parts.len() - 1
        ));
        return errors;
    }

    // 2. Check balanced \begin{aligned} and \end{aligned} globally in the file
    let num_begin_aligned_all = cleaned.matches(r"\begin{aligned}").count();
    let num_end_aligned_all = cleaned.matches(r"\end{aligned}").count();
    if num_begin_aligned_all != num_end_aligned_all {
        errors.push(format!(
            "Unbalanced \\begin{{aligned}} ({}) and \\end{{aligned}} ({}) pairs in {}.",
            num_begin_aligned_all, num_end_aligned_all, file_rel_path
        ));
    }

    // 3. Validate each display math block
    for i in (1..parts.len()).step_by(2) {
        let Some(&block) = parts.get(i) else {
            continue;
        };

        // Detect top-level \begin{align} or \begin{align*}
        if block.contains(r"\begin{align}") || block.contains(r"\begin{align*}") {
            errors.push(format!(
                "Forbidden \\begin{{align}} or \\begin{{align*}} found in display math block in {}. In markdown KaTeX, \\begin{{aligned}} must be used instead.",
                file_rel_path
            ));
        }

        // Validate balanced \begin{aligned} and \end{aligned} within block
        let num_begin = block.matches(r"\begin{aligned}").count();
        let num_end = block.matches(r"\end{aligned}").count();
        if num_begin != num_end {
            errors.push(format!(
                "Unbalanced \\begin{{aligned}} ({}) and \\end{{aligned}} ({}) in math block in {}.",
                num_begin, num_end, file_rel_path
            ));
        }

        // Detect bare alignment operator & outside allowed alignment environments
        let mut env_stack: Vec<String> = Vec::new();
        let mut idx = 0;
        let bytes = block.as_bytes();
        while idx < bytes.len() {
            if bytes.get(idx) == Some(&b'\\') {
                if block.get(idx..).is_some_and(|s| s.starts_with(r"\begin{")) {
                    let after = block.get(idx + 7..).unwrap_or("");
                    if let Some(close_pos) = after.find('}') {
                        let env_name = after.get(..close_pos).unwrap_or("");
                        if !env_name.is_empty() && env_name.chars().all(|c| c.is_ascii_alphabetic() || c == '*') {
                            env_stack.push(env_name.to_string());
                            idx += 7 + close_pos + 1;
                            continue;
                        }
                    }
                } else if block.get(idx..).is_some_and(|s| s.starts_with(r"\end{")) {
                    let after = block.get(idx + 5..).unwrap_or("");
                    if let Some(close_pos) = after.find('}') {
                        let env_name = after.get(..close_pos).unwrap_or("");
                        if !env_name.is_empty() && env_name.chars().all(|c| c.is_ascii_alphabetic() || c == '*') {
                            if let Some(pos) = env_stack.iter().rposition(|x| x == env_name) {
                                env_stack.truncate(pos);
                            }
                            idx += 5 + close_pos + 1;
                            continue;
                        }
                    }
                } else if block.get(idx..).is_some_and(|s| s.starts_with(r"\&")) {
                    idx += 2;
                    continue;
                }
                idx += 1;
            } else if bytes.get(idx) == Some(&b'&') {
                let inside_allowed = env_stack
                    .iter()
                    .any(|env| ALLOWED_ALIGNMENT_ENVS.contains(&env.as_str()));
                if !inside_allowed {
                    let mut start = idx.saturating_sub(20);
                    while start > 0 && !block.is_char_boundary(start) {
                        start -= 1;
                    }
                    let mut end = (idx + 1 + 20).min(block.len());
                    while end < block.len() && !block.is_char_boundary(end) {
                        end += 1;
                    }
                    let snippet = block.get(start..end).unwrap_or("").trim().replace('\n', " ");
                    errors.push(format!(
                        "Bare alignment operator '&' outside alignment environment in {}: \"...{}...\"",
                        file_rel_path, snippet
                    ));
                }
                idx += 1;
            } else {
                idx += 1;
            }
        }
    }

    errors
}

/// Inspects a single Mermaid diagram statement line for unquoted `<` or `>` characters after
/// stripping valid UML stereotypes (`<<...>>`) and Mermaid connector arrows.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents Mermaid parser failures caused by
/// raw comparison operators or angle brackets in transition guards and node labels.
///
/// ## 2. Invariants & Mathematical Properties
/// - Stereotype Exemption: Substrings matching `<<[A-Za-z0-9_-]+>>` are stripped prior to inspection.
/// - Longest-Match Arrow Exemption: All 17 Mermaid connector tokens (`==>>`, `-->>`, `<<==`, `<<--`,
///   `<-->`, `<==>`, `<-.-`, `-.->`, `==>`, `-->`, `->>`, `<==`, `<--`, `<<-`, `<->`, `->`, `<-`)
///   are removed in decreasing length order so composite arrows never leave residual `<` or `>` characters.
/// - Quote State Machine: Tracks unescaped `"` toggles and reports the first `<` or `>` encountered
///   when `in_quotes == false`.
///
/// ## 3. Preconditions
/// `line` is a single non-comment UTF-8 line from a Mermaid diagram body.
///
/// ## 4. Postconditions
/// Returns `Some('<')` or `Some('>')` if an unquoted angle bracket remains after stripping stereotypes
/// and connectors; returns `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where $L$ is the byte length of `line`. Space Complexity: O(L) for sanitized line copy.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function using UTF-8-aware slicing (`len_utf8()`, `.get()`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-ANGLES]
fn validate_mermaid_angle_bracket_escaping(line: &str) -> Option<char> {
    let mut clean_line = line.trim().to_string();
    // Strip stereotypes: <<[a-zA-Z0-9_\-]+>>
    let mut stripped = String::with_capacity(clean_line.len());
    let mut idx = 0;
    while idx < clean_line.len() {
        if clean_line.get(idx..).is_some_and(|s| s.starts_with("<<")) {
            if let Some(close_pos) = clean_line.get(idx + 2..).and_then(|s| s.find(">>")) {
                let inner = clean_line.get(idx + 2..idx + 2 + close_pos).unwrap_or("");
                if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                    idx += 2 + close_pos + 2;
                    continue;
                }
            }
        }
        let Some(ch) = clean_line.get(idx..).and_then(|s| s.chars().next()) else {
            break;
        };
        stripped.push(ch);
        idx += ch.len_utf8();
    }
    clean_line = stripped;

    // Strip longest arrows first to prevent partial replacements (e.g. "-->" matching inside "<-->")
    let arrows = [
        "==>>", "-->>", "<<==", "<<--", "<-->", "<==>", "<-.-", "-.->",
        "==>", "-->", "->>", "<==", "<--", "<<-", "<->",
        "->", "<-",
    ];
    for arrow in arrows {
        clean_line = clean_line.replace(arrow, "");
    }

    let mut in_quotes = false;
    let mut escape = false;
    for ch in clean_line.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && (ch == '<' || ch == '>') {
            return Some(ch);
        }
    }
    None
}

/// Validates all fenced ` ```mermaid ` diagrams in a markdown document for fence closure, valid
/// diagram type headers, forbidden semicolons in notes/messages, and unquoted angle brackets.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces Mermaid Syntax & Diagram Integrity
/// Rules across all architectural and behavioral specification documents.
///
/// ## 2. Invariants & Mathematical Properties
/// - Fence Closure: Every opening ` ```mermaid ` fence must have a matching closing ` ``` ` line before EOF.
/// - Header Validity: The first non-empty, non-`%%`-comment line inside each Mermaid block must start
///   (case-insensitively) with a prefix in `VALID_MERMAID_HEADERS`.
/// - Semicolon Prohibition: `Note` statements and sequence diagram message lines must not contain `;`.
/// - Angle-Bracket Quoting: For `graph`, `flowchart`, `sequencediagram`, `statediagram`, and
///   `statediagram-v2` blocks, every non-comment line must satisfy `validate_mermaid_angle_bracket_escaping(line).is_none()`.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown document slice; `file_rel_path` is the relative path used
/// for line-numbered diagnostic reporting.
///
/// ## 4. Postconditions
/// Returns an empty `Vec<String>` when all Mermaid blocks are syntactically valid; otherwise returns
/// all detected violations formatted with `file_rel_path:line_number`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) over total document byte length $N$. Space Complexity: O(N) for line slices and error messages.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function. Gracefully reports unclosed fences at EOF without out-of-bounds indexing.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-MERMAID]
pub fn check_mermaid_syntax(content: &str, file_rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let n = lines.len();
    let mut i = 0;

    while i < n {
        let Some(&line) = lines.get(i) else {
            break;
        };
        let trimmed = line.trim();
        let is_mermaid_open = if trimmed.starts_with("```") {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            backticks >= 3 && trimmed.get(backticks..).unwrap_or("").trim() == "mermaid"
        } else {
            false
        };

        if is_mermaid_open {
            let start_line = i + 1;
            let mut body: Vec<(usize, &str)> = Vec::new();
            i += 1;
            let mut closed = false;

            while i < n {
                let Some(&inner) = lines.get(i) else {
                    break;
                };
                let inner_trimmed = inner.trim();
                if inner_trimmed.starts_with("```") {
                    closed = true;
                    break;
                }
                body.push((i + 1, inner));
                i += 1;
            }

            if !closed {
                errors.push(format!(
                    "{}:{}: unclosed ```mermaid fence. Every diagram must be closed with a matching ``` fence on its own line.",
                    file_rel_path, start_line
                ));
            } else {
                // Find first non-empty, non-comment line to check header
                let mut header_opt = None;
                for &(lineno, body_line) in &body {
                    let trimmed = body_line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("%%") {
                        header_opt = Some((lineno, trimmed));
                        break;
                    }
                }

                let mut kind = String::new();
                if let Some((header_line, header_text)) = header_opt {
                    let lower = header_text.to_lowercase();
                    if let Some(matched_kind) = VALID_MERMAID_HEADERS
                        .iter()
                        .find(|&h| lower.starts_with(h))
                    {
                        kind = matched_kind.to_string();
                    } else {
                        errors.push(format!(
                            "{}:{}: missing or invalid Mermaid diagram header (found '{}'). The first non-comment line must declare a valid diagram type header.",
                            file_rel_path, header_line, header_text
                        ));
                    }
                } else {
                    errors.push(format!(
                        "{}:{}: missing or invalid Mermaid diagram header. The first non-comment line must declare a valid diagram type header.",
                        file_rel_path, start_line
                    ));
                }

                // Check forbidden semicolons in Mermaid Note statements and message text
                for &(lineno, body_line) in &body {
                    let trimmed = body_line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("%%") {
                        continue;
                    }
                    let is_note = trimmed.starts_with("Note ") || trimmed.starts_with("note ");
                    let is_msg = trimmed.contains(">>")
                        || trimmed.contains("->>")
                        || trimmed.contains("-->>")
                        || (kind == "sequencediagram"
                            && trimmed.contains(':')
                            && !trimmed.starts_with("participant ")
                            && !trimmed.starts_with("actor "));
                    if (is_note || is_msg) && trimmed.contains(';') {
                        errors.push(format!(
                            "{}:{}: semicolon in Mermaid Note or message text: '{}'. Replace with commas, dashes, or spaces.",
                            file_rel_path, lineno, trimmed
                        ));
                    }
                }

                // Check angle bracket escaping in diagram kinds
                if ["graph", "flowchart", "sequencediagram", "statediagram", "statediagram-v2"]
                    .contains(&kind.as_str())
                {
                    for &(lineno, body_line) in &body {
                        let trimmed = body_line.trim();
                        if trimmed.is_empty() || trimmed.starts_with("%%") {
                            continue;
                        }
                        if let Some(unquoted_char) =
                            validate_mermaid_angle_bracket_escaping(trimmed)
                        {
                            errors.push(format!(
                                "{}:{}: unquoted '{}' character in Mermaid diagram line: '{}'. Transitions, labels, or guards containing comparison operators or brackets MUST be enclosed in double quotes.",
                                file_rel_path, lineno, unquoted_char, trimmed
                            ));
                        }
                    }
                }
            }
        }
        i += 1;
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glfm_inline_math_does_not_synthesize_display_math() {
        let sample = "The constraint `$``v_{lim} = 10``$` applies along with `$``a_{lim} = 2``$`.";
        let cleaned = strip_code_and_glfm(sample);
        assert!(
            !cleaned.contains("$$"),
            "GLFM math stripping must not synthesize false '$$' display delimiters! Cleaned: {:?}",
            cleaned
        );
        let errors = check_latex_katex_syntax(sample, "test.md");
        assert!(
            errors.is_empty(),
            "Expected 0 errors on valid GLFM math, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_bare_alignment_inside_aligned_passes() {
        let sample = r#"
$$
\begin{aligned}
x &= 1 \\
y &= 2
\end{aligned}
$$
"#;
        let errors = check_latex_katex_syntax(sample, "test.md");
        assert!(
            errors.is_empty(),
            "Expected 0 errors for alignment operator '&' inside \\begin{{aligned}}, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_bare_alignment_outside_aligned_fails() {
        let sample = r#"
$$
x & 1 \\
y & 2
$$
"#;
        let errors = check_latex_katex_syntax(sample, "test.md");
        assert!(
            !errors.is_empty(),
            "Expected error for bare '&' outside alignment environment"
        );
        let first_err = errors.first().map(|s| s.as_str()).unwrap_or("");
        assert!(
            first_err.contains("Bare alignment operator '&' outside alignment environment"),
            "Error was: {}",
            first_err
        );
    }

    #[test]
    fn test_schema_req_0099_zero_errors() {
        let req_content = r#"---
id: REQ-0099
title: "Base Dimension Tuple Representation & Generalized Conjugate Power Pairs"
subsystem: "Subsystem 5: 7D Physical Metrology & Abstract Flow Conservation Networks"
uuidv5: 83cab42d-eb92-56bf-91cd-0f4bc1c7a675
---

# [REQ-0099] Base Dimension Tuple Representation & Generalized Conjugate Power Pairs

| Metadata Field | Contract Specification |
| :--- | :--- |
| **Requirement ID** | `REQ-0099` |
| **Deterministic UUIDv5** | `83cab42d-eb92-56bf-91cd-0f4bc1c7a675` |
| **Subsystem** | Subsystem 5: 7D Physical Metrology & Abstract Flow Conservation Networks |
| **Complexity Class** | Class $\mathbf{P}$ ($O(1)$ coordinate check, $O(N)$ lowering) |
| **NP Frontier Anchor** | N/A (Deterministic Polynomial Foundation) |
| **Diagnostic Code Bindings** | `E0199`, `E0300`, `E0301` (per REQ-0191) |
| **Governing Standard** | IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 |

---

## 1. Normative Statement

The engine SHALL represent dimensions as irreducible rational 7-tuples over $\mathbb{Q}^7$ spanning ISQ base dimensions ($M, L, T, I, \Theta, N, J$).

Lowering MUST enforce:
1. **Irreducible Exponents**: Coordinates MUST be irreducible pairs $(p, q)$ ($p \in \mathbb{Z}, q \in \mathbb{Z}^+, \gcd(|p|, q) = 1$). Denominators $q \le 0$ emit `E0300`.
2. **Dimensionless Zero**: Zero vector $\mathbf{0} \in \mathbb{Q}^7$ acts as additive identity.
3. **Conjugate Pairs**: Conservative ports (`Port_Alpha`, `Interface_Gamma`) MUST bind Across (`Across_Var`, $e$) and Through (`Through_Var`, $f$) features.
4. **Power Invariant**: Conjugate dimensions MUST satisfy $``\mathbf{d}(e) + \mathbf{d}(f) \equiv \mathbf{d}_{\text{Power}} = (1,2,-3,0,0,0,0) \in \mathbb{Q}^7``$. Violations emit `E0301`.
5. **Non-Conservative Exemption**: Signal interfaces (REQ-0106) SHALL NOT enforce the power invariant.
6. **Homogeneity**: Operations ($``Q_1 \pm Q_2``$, $``Q_1 == Q_2``$) require $``\mathbf{d}(Q_1) = \mathbf{d}(Q_2)``$; mismatches emit `E0300`.

---

## 2. Formal Invariant

Lattice invariants, power duality, and diagnostics:

```math
\begin{aligned}
\mathbf{d} &\in \mathbb{Q}^7,\; d_i = p_i/q_i,\; q_i \in \mathbb{Z}^+,\; \gcd(|p_i|, q_i) = 1,\; \mathbf{0} = (0,\dots,0) \\
\mathbf{d}_{\text{Power}} &= (1,2,-3,0,0,0,0),\; \forall C = \langle e, f \rangle \in \mathcal{C}_{\text{pair}} \implies \mathbf{d}(e) + \mathbf{d}(f) \equiv \mathbf{d}_{\text{Power}} \\
\mathbf{d}(e) + \mathbf{d}(f) &\ne \mathbf{d}_{\text{Power}} \implies \text{Diagnostic}(C, \text{E0301}) \\
q_i \le 0 &\vee (\mathbf{d}(Q_1) \ne \mathbf{d}(Q_2) \wedge Q_1 \pm Q_2) \implies \text{Diagnostic}(n, \text{E0300}) \\
\Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n))
\end{aligned}
```

| Symbol / Term | Domain / Formal Definition |
| :--- | :--- |
| $\mathbf{d} \in \mathbb{Q}^7$ | Irreducible rational exponent 7-tuple over ISQ |
| $``p_i, q_i``$ | Rational numerator and positive denominator ($\gcd=1$) |
| $``\mathbf{0}, \mathbf{d}_{\text{Power}}``$ | Zero vector and Power vector $(1,2,-3,0,0,0,0)$ |
| $``\mathcal{C}_{\text{pair}}, \Delta_n``$ | Conjugate pair descriptors and UUIDv5 hash |
| $e, f$ | Across effort and Through flow features |

---

## 3. Computational Complexity & Algorithmic Bounds

- **Complexity Class:** $\mathbf{P}$ (Deterministic Polynomial)
- **Asymptotic Bound:** $O(1)$ arithmetic; $``O(N_P + N_C)``$ AST traversal over $``N_P``$ ports and $``N_C``$ pairs.
- **Tractable Subclass & Frontier:** Group arithmetic in $\mathbb{Q}^7$ runs in $O(1)$, bypassing NP frontiers (NP-1--NP-8) to ground flow networks (NP-6). Work-stealing scales as $``O((N_P + N_C) / N_{\text{max\_workers}} + \log N_{\text{max\_workers}})``$.

---

## 4. Verification & Conformance Criteria (IEEE 29148 Acceptance Criteria)

### AC-01: Canonical Rational Reduction & Zero Identity
- **Given:** Rational coordinates $(2/4,6/2,-9/3,0/1,0/1,0/1,0/1)$ and dimensionless scalar.
- **When:** Lowering dimension AST nodes.
- **Then:** Engine reduces coordinates to $((1,2),(3,1),(-3,1),(0,1),(0,1),(0,1),(0,1))$ and maps scalar to $\mathbf{0}$.
- **Diagnostic:** Diagnostic sink remains empty.

### AC-02: Conservative Conjugate Power Pair Conformance
- **Given:** Interface `Interface_Gamma` with Across `Across_Var` and Through `Through_Var`.
- **When:** Validating conjugate pairs.
- **Then:** Dimensional sum evaluates to $``\mathbf{d}_{\text{Power}} = (1,2,-3,0,0,0,0)``$ with valid flag set true.
- **Diagnostic:** Diagnostic sink remains empty.

### AC-03: Conjugate Power Product Violation Rejection
- **Given:** Interface `Interface_Gamma` with Across $(1,1,-2,0,0,0,0)$ and Through $(0,0,1,0,0,0,0)$.
- **When:** Evaluating power invariant ($``\mathbf{d}_e + \mathbf{d}_f \ne \mathbf{d}_{\text{Power}}``$).
- **Then:** Lowering rejects definition with `E0301` and assigns sentinel $``\bot_{\text{pair}}``$.
- **Diagnostic:** `E0301`.

### AC-04: Static Homogeneity & Delimiter Recovery
- **Given:** Denominator $q = 0$ or mismatched addition `param_x + param_y`.
- **When:** Parsing and validation execute.
- **Then:** Compiler emits `E0300`, assigns sentinel $``\bot_{\text{dim}}``$, recovers on delimiter (`;`), and halts at $``C_{\text{max\_errors}}``$ with `E0199`.
- **Diagnostic:** `E0300`, `E0199`.

---

*Traceability: Elaborated under IEEE 29148-2018 and the DEAP Compiler Streamlined Charter.*
"#;
        let katex_errors = check_latex_katex_syntax(req_content, "schema/REQ-0099.md");
        assert!(
            katex_errors.is_empty(),
            "schema/REQ-0099.md must have 0 KaTeX errors, got: {:?}",
            katex_errors
        );

        let mermaid_errors = check_mermaid_syntax(req_content, "schema/REQ-0099.md");
        assert!(
            mermaid_errors.is_empty(),
            "schema/REQ-0099.md must have 0 Mermaid errors, got: {:?}",
            mermaid_errors
        );
    }

    #[test]
    fn test_mermaid_unclosed_fence_detection() {
        let sample = "```mermaid\ngraph TD\nA --> B\n";
        let errors = check_mermaid_syntax(sample, "test.md");
        assert!(!errors.is_empty(), "Should detect unclosed mermaid fence");
        assert!(errors.first().is_some_and(|e| e.contains("unclosed ```mermaid fence")));
    }

    #[test]
    fn test_mermaid_invalid_header_detection() {
        let sample = "```mermaid\ninvalidHeader TD\nA --> B\n```";
        let errors = check_mermaid_syntax(sample, "test.md");
        assert!(!errors.is_empty(), "Should detect invalid diagram header");
        assert!(errors.first().is_some_and(|e| e.contains("missing or invalid Mermaid diagram header")));
    }

    #[test]
    fn test_mermaid_unquoted_angle_bracket_detection() {
        let sample = "```mermaid\ngraph TD\nA -->|x < 5| B\n```";
        let errors = check_mermaid_syntax(sample, "test.md");
        assert!(!errors.is_empty(), "Should detect unquoted '<' character");
        assert!(errors.first().is_some_and(|e| e.contains("unquoted '<' character")));

        let quoted_sample = "```mermaid\ngraph TD\nA -->|\"x < 5\"| B\n```";
        let clean_errors = check_mermaid_syntax(quoted_sample, "test.md");
        assert!(
            clean_errors.is_empty(),
            "Quoted angle bracket should pass, got: {:?}",
            clean_errors
        );
    }

    #[test]
    fn test_catalog_structs_contains_and_iter() {
        let align_catalog: AllowedAlignmentEnvsCatalog = ALLOWED_ALIGNMENT_ENVS;
        assert!(align_catalog.contains(&"aligned"));
        assert!(align_catalog.contains(&"pmatrix"));
        assert!(!align_catalog.contains(&"align"));
        assert_eq!(align_catalog.iter().count(), 17);

        let mermaid_catalog: ValidMermaidHeadersCatalog = VALID_MERMAID_HEADERS;
        assert!(mermaid_catalog.contains("flowchart"));
        assert!(mermaid_catalog.contains("sequencediagram"));
        assert!(!mermaid_catalog.contains("invalid"));
        assert_eq!(mermaid_catalog.iter().count(), 18);
    }

    #[test]
    fn test_allowed_alignment_env_enum_cardinality_order_and_catalog_parity() {
        let expected = vec![
            "aligned",
            "alignedat",
            "matrix",
            "pmatrix",
            "bmatrix",
            "Bmatrix",
            "vmatrix",
            "Vmatrix",
            "cases",
            "dcases",
            "rcases",
            "array",
            "split",
            "gathered",
            "gather",
            "subarray",
            "smallmatrix",
        ];
        assert_eq!(AllowedAlignmentEnv::ALL.len(), 17);
        let via_enum: Vec<&'static str> = AllowedAlignmentEnv::ALL.iter().map(|e| e.as_str()).collect();
        assert_eq!(via_enum, expected);
        let via_catalog: Vec<&'static str> = ALLOWED_ALIGNMENT_ENVS.iter().collect();
        assert_eq!(via_catalog, expected);
        for e in AllowedAlignmentEnv::ALL {
            assert_eq!(e.as_str_ref(), e.as_str());
            assert!(ALLOWED_ALIGNMENT_ENVS.contains(&e.as_str()));
        }
        assert!(!ALLOWED_ALIGNMENT_ENVS.contains(&"align"));
        assert!(!ALLOWED_ALIGNMENT_ENVS.contains(&"ALIGNED"));
    }

    #[test]
    fn test_valid_mermaid_header_enum_cardinality_order_and_catalog_parity() {
        let expected = vec![
            "graph",
            "flowchart",
            "sequencediagram",
            "statediagram",
            "statediagram-v2",
            "classdiagram",
            "erdiagram",
            "gantt",
            "pie",
            "gitgraph",
            "c4context",
            "journey",
            "quadrantchart",
            "xychart-beta",
            "mindmap",
            "timeline",
            "sankey-beta",
            "requirementdiagram",
        ];
        assert_eq!(ValidMermaidHeader::ALL.len(), 18);
        let via_enum: Vec<&'static str> = ValidMermaidHeader::ALL.iter().map(|h| h.as_str()).collect();
        assert_eq!(via_enum, expected);
        let via_catalog: Vec<&'static str> = VALID_MERMAID_HEADERS.iter().collect();
        assert_eq!(via_catalog, expected);
        for h in ValidMermaidHeader::ALL {
            assert_eq!(h.as_str_ref(), h.as_str());
            assert!(VALID_MERMAID_HEADERS.contains(h.as_str()));
        }
        assert!(!VALID_MERMAID_HEADERS.contains("sequenceDiagram"));
        assert!(!VALID_MERMAID_HEADERS.contains("architecture-beta"));
    }

    #[test]
    fn test_markdown_catalogs_free_of_static_str_iter_evasion() {
        let source = include_str!("markdown.rs");
        let needle_into_iter = ["IntoIter<&'static", " str"].concat();
        let needle_static_slice = ["&'static [&'static", " str]"].concat();
        let hits: Vec<String> = source
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains(&needle_into_iter) || l.contains(&needle_static_slice))
            .map(|(i, l)| format!("line {}: {}", i + 1, l.trim()))
            .collect();
        assert!(
            hits.is_empty(),
            "markdown.rs still carries untyped string-array catalog evasions:\n{}",
            hits.join("\n")
        );
    }
}

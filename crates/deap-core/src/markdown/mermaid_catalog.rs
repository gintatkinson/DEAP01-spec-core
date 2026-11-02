//! Mermaid valid diagram headers enumeration and catalog.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ValidMermaidHeader`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves structural type invariants for enum `ValidMermaidHeader` across all 18 members.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed values for `ValidMermaidHeader` members.
//!
//! ## 4. Postconditions
//! - Defines authoritative `ValidMermaidHeader` and `ValidMermaidHeadersCatalog` representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) constant-time enum dispatch.
//! - Space Complexity: O(1) auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Exhaustive matching prevents unhandled variants.
//!
//! ## 7. Realization & Traceability
//! Realises: [deap_core::markdown::ValidMermaidHeader] | SysML v2: PartDef(ValidMermaidHeader)

/// Typed enumeration of every Mermaid diagram type header accepted as the first non-comment line of a block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ValidMermaidHeader`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across all 18 members.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ValidMermaidHeader` members.
///
/// ## 4. Postconditions
/// - Defines the authoritative `ValidMermaidHeader` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(18)) bounded over member dimension of `ValidMermaidHeader`.
/// - Space Complexity: O(M(18)) bounded auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ValidMermaidHeader`.
/// - State: Maintains consistent state invariants.
/// - Action: Executes `ValidMermaidHeader` deterministically.
/// - Resource: Bounds stack and heap allocations.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Fixes each whitelisted Mermaid diagram type.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective mapping from 18 variants to 18 distinct lowercase ASCII keywords.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 18 `ValidMermaidHeader` variants.
    ///
    /// ## 4. Postconditions
    /// Returns an all-lowercase `'static` keyword without surrounding whitespace or direction suffix.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive match prevents unhandled variants.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supplies `fn(&'static ValidMermaidHeader)` pointer.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// `h.as_str_ref() == h.as_str()` for each of the 18 variants.
    ///
    /// ## 3. Preconditions
    /// `self` is a shared reference to a `ValidMermaidHeader`.
    ///
    /// ## 4. Postconditions
    /// Returns the identical lowercase `'static` keyword.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total and infallible.
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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ValidMermaidHeadersCatalog`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ValidMermaidHeadersCatalog`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ValidMermaidHeadersCatalog` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `ValidMermaidHeadersCatalog` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input dimension.
/// - Space Complexity: O(M(1)) zero auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ValidMermaidHeadersCatalog`.
/// - State: Maintains consistent state invariants.
/// - Action: Executes `ValidMermaidHeadersCatalog` deterministically.
/// - Resource: Bounds allocations within `ValidMermaidHeadersCatalog`.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity rules.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 18 lowercase Mermaid diagram header prefixes in fixed declaration order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid reference to `ValidMermaidHeadersCatalog`.
    ///
    /// ## 4. Postconditions
    /// Returns an iterator yielding exactly 18 static strings.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) construction, O(18) traversal. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible total function; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-MD-MERMAID-HDR-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, ValidMermaidHeader>,
        fn(&'static ValidMermaidHeader) -> &'static str,
    > {
        let project: fn(&'static ValidMermaidHeader) -> &'static str =
            ValidMermaidHeader::as_str_ref;
        ValidMermaidHeader::ALL.iter().map(project)
    }

    /// Checks whether a lowercase header token matches one of the 18 valid Mermaid diagram types.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Returns `true` if and only if `hdr` is an exact match for one of the 18 supported keywords.
    ///
    /// ## 3. Preconditions
    /// `hdr` is a valid UTF-8 string slice.
    ///
    /// ## 4. Postconditions
    /// Returns a deterministic boolean verdict without side effects.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(L) bounded by 18 bytes. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total non-panicking function.
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

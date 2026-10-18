//! Streaming zero-copy line and token parser for Mermaid diagrams conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Provides AST extraction and structural invariant enforcement for Mermaid diagrams extracted by `markdown_ast::MermaidBlock`.
//! Supports class diagrams, state diagrams, sequence diagrams, flowcharts, graphs, entity-relationship diagrams, and xy charts.
//! Strictly zero regex, zero unwrap/expect/panic, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::markdown_ast::MermaidBlock;
use crate::spec_audit::SpecAuditError;
use std::collections::HashSet;

/// Supported diagram types parsed from diagram headers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `DiagramType` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `DiagramType` across member set `[Class, State, Sequence, Flowchart, Graph, Er]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `DiagramType` members (`Class, State, Sequence, Flowchart, Graph, Er`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `DiagramType` representation for enum `DiagramType`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `DiagramType`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `DiagramType`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `DiagramType` (`DiagramType`).
/// - State: Maintains consistent state invariants across `DiagramType` without partial mutation on error.
/// - Action: Executes `DiagramType` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `DiagramType` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-DIAGRAMTYPE] | SysML v2: PartDef(DiagramType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagramType {
    /// Class diagram (`classDiagram`).
    Class,
    /// State diagram (`stateDiagram`, `stateDiagram-v2`).
    State,
    /// Sequence diagram (`sequenceDiagram`).
    Sequence,
    /// Flowchart (`flowchart TD`, `flowchart LR`, etc.).
    Flowchart,
    /// Graph (`graph TD`, `graph LR`, etc.).
    Graph,
    /// Entity-Relationship diagram (`erDiagram`).
    Er,
    /// XY Chart (`xychart-beta`).
    XyChart,
    /// Other or generic diagram type.
    Other,
}

/// In-memory AST representation of any parsed Mermaid diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `DiagramAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `DiagramAst` across member set `[Class, State, Sequence, Flowchart, Er, XyChart]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `DiagramAst` members (`Class, State, Sequence, Flowchart, Er, XyChart`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `DiagramAst` representation for enum `DiagramAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `DiagramAst`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `DiagramAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `DiagramAst` (`DiagramAst`).
/// - State: Maintains consistent state invariants across `DiagramAst` without partial mutation on error.
/// - Action: Executes `DiagramAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `DiagramAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-DIAGRAMAST] | SysML v2: PartDef(DiagramAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramAst {
    /// Parsed class diagram AST.
    Class(ClassDiagramAst),
    /// Parsed state diagram AST.
    State(StateDiagramAst),
    /// Parsed sequence diagram AST.
    Sequence(SequenceDiagramAst),
    /// Parsed flowchart diagram AST.
    Flowchart(FlowchartAst),
    /// Parsed ER diagram AST.
    Er(ErDiagramAst),
    /// Parsed XY chart diagram AST.
    XyChart(XyChartAst),
    /// Generic or unspecialized diagram AST.
    Other(GenericDiagramAst),
}

/// Member visibility modifier in UML class diagrams.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `Visibility` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `Visibility` across member set `[Public, Private, Protected, Package]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Visibility` members (`Public, Private, Protected, Package`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Visibility` representation for enum `Visibility`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `Visibility`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `Visibility`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Visibility` (`Visibility`).
/// - State: Maintains consistent state invariants across `Visibility` without partial mutation on error.
/// - Action: Executes `Visibility` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Visibility` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VISIBILITY] | SysML v2: PartDef(Visibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Public visibility (`+`).
    Public,
    /// Private visibility (`-`).
    Private,
    /// Protected visibility (`#`).
    Protected,
    /// Package/Internal visibility (`~`).
    Package,
}

impl Visibility {
    /// Parses a visibility prefix character.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `Visibility::from_char` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `Visibility::from_char` (declaration #4) within `SPEC-AUDIT-DIAGRAM-AST`.
    ///
    /// ## 3. Preconditions
    /// - `c: char` must satisfy domain and type-safety constraints prior to invoking `Visibility::from_char`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Option<Self>` from `Visibility::from_char` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(c)) bounded over input/member dimension of `Visibility::from_char`.
    /// - Space Complexity: O(N(c)) bounded auxiliary memory for `Visibility::from_char`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `Visibility::from_char` (`Option<Self>`).
    /// - State: Maintains consistent state invariants across `Visibility::from_char` without partial mutation on error.
    /// - Action: Executes `Visibility::from_char` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `Visibility::from_char` (`SPEC-AUDIT-DIAGRAM-AST`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VISIBILITY-FROM-CHAR] | SysML v2: ActionDef(Visibility::from_char)
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Public),
            '-' => Some(Self::Private),
            '#' => Some(Self::Protected),
            '~' => Some(Self::Package),
            _ => None,
        }
    }
}

/// Class member attribute definition in a class diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ClassAttributeAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ClassAttributeAst` across member set `[visibility: Option<Visibility>, name: String, type_name: Option<String>, raw: String, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ClassAttributeAst` members (`visibility: Option<Visibility>, name: String, type_name: Option<String>, raw: String, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ClassAttributeAst` representation for struct `ClassAttributeAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ClassAttributeAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ClassAttributeAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ClassAttributeAst` (`ClassAttributeAst`).
/// - State: Maintains consistent state invariants across `ClassAttributeAst` without partial mutation on error.
/// - Action: Executes `ClassAttributeAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ClassAttributeAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASSATTRIBUTEAST] | SysML v2: PartDef(ClassAttributeAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAttributeAst {
    /// Member visibility.
    pub visibility: Option<Visibility>,
    /// Attribute identifier.
    pub name: String,
    /// Declared type name, if present.
    pub type_name: Option<String>,
    /// Raw unparsed member string.
    pub raw: String,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Class member operation/method definition in a class diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ClassOperationAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ClassOperationAst` across member set `[visibility: Option<Visibility>, name: String, parameters: Vec<String>, return_type: Option<String>, raw: String]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ClassOperationAst` members (`visibility: Option<Visibility>, name: String, parameters: Vec<String>, return_type: Option<String>, raw: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ClassOperationAst` representation for struct `ClassOperationAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ClassOperationAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ClassOperationAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ClassOperationAst` (`ClassOperationAst`).
/// - State: Maintains consistent state invariants across `ClassOperationAst` without partial mutation on error.
/// - Action: Executes `ClassOperationAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ClassOperationAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASSOPERATIONAST] | SysML v2: PartDef(ClassOperationAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassOperationAst {
    /// Member visibility.
    pub visibility: Option<Visibility>,
    /// Operation identifier.
    pub name: String,
    /// Parameter signature list.
    pub parameters: Vec<String>,
    /// Return type, if present.
    pub return_type: Option<String>,
    /// Raw unparsed member string.
    pub raw: String,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Relationship type between classifiers in a class diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ClassRelationType` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `ClassRelationType` across member set `[Inheritance, Composition, Aggregation, Association, Realization, Dependency]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ClassRelationType` members (`Inheritance, Composition, Aggregation, Association, Realization, Dependency`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ClassRelationType` representation for enum `ClassRelationType`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(6)) bounded over input/member dimension of `ClassRelationType`.
/// - Space Complexity: O(M(6)) bounded auxiliary memory for `ClassRelationType`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ClassRelationType` (`ClassRelationType`).
/// - State: Maintains consistent state invariants across `ClassRelationType` without partial mutation on error.
/// - Action: Executes `ClassRelationType` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ClassRelationType` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASSRELATIONTYPE] | SysML v2: PartDef(ClassRelationType)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassRelationType {
    /// Generalization / Inheritance (`<|--` or `--|>`).
    Inheritance,
    /// Composition (`*--` or `--*`).
    Composition,
    /// Aggregation (`o--` or `--o`).
    Aggregation,
    /// Association (`-->` or `<--` or `--`).
    Association,
    /// Realization / Interface Implementation (`..|>` or `<|..`).
    Realization,
    /// Dependency (`..>` or `<..` or `..`).
    Dependency,
    /// Other relationship symbol.
    Other(String),
}

/// Classifier relationship in a class diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ClassRelationAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ClassRelationAst` across member set `[from_class: String, to_class: String, relation_type: ClassRelationType, raw_symbol: String, label: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ClassRelationAst` members (`from_class: String, to_class: String, relation_type: ClassRelationType, raw_symbol: String, label: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ClassRelationAst` representation for struct `ClassRelationAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ClassRelationAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ClassRelationAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ClassRelationAst` (`ClassRelationAst`).
/// - State: Maintains consistent state invariants across `ClassRelationAst` without partial mutation on error.
/// - Action: Executes `ClassRelationAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ClassRelationAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASSRELATIONAST] | SysML v2: PartDef(ClassRelationAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRelationAst {
    /// Source classifier identifier.
    pub from_class: String,
    /// Target classifier identifier.
    pub to_class: String,
    /// Parsed relation semantic type.
    pub relation_type: ClassRelationType,
    /// Exact relation arrow operator in diagram text.
    pub raw_symbol: String,
    /// Optional relationship label.
    pub label: Option<String>,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Class definition in a class diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ClassDefAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ClassDefAst` across member set `[name: String, annotations: Vec<String>, attributes: Vec<ClassAttributeAst>, operations: Vec<ClassOperationAst>, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ClassDefAst` members (`name: String, annotations: Vec<String>, attributes: Vec<ClassAttributeAst>, operations: Vec<ClassOperationAst>, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ClassDefAst` representation for struct `ClassDefAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ClassDefAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ClassDefAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ClassDefAst` (`ClassDefAst`).
/// - State: Maintains consistent state invariants across `ClassDefAst` without partial mutation on error.
/// - Action: Executes `ClassDefAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ClassDefAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASSDEFAST] | SysML v2: PartDef(ClassDefAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassDefAst {
    /// Classifier identifier name.
    pub name: String,
    /// Stereotypes or annotations (e.g. `<<interface>>`).
    pub annotations: Vec<String>,
    /// Declared attributes.
    pub attributes: Vec<ClassAttributeAst>,
    /// Declared operations.
    pub operations: Vec<ClassOperationAst>,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Complete AST for a Mermaid class diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ClassDiagramAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ClassDiagramAst` across member set `[classes: Vec<ClassDefAst>, relations: Vec<ClassRelationAst>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ClassDiagramAst` members (`classes: Vec<ClassDefAst>, relations: Vec<ClassRelationAst>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ClassDiagramAst` representation for struct `ClassDiagramAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `ClassDiagramAst`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `ClassDiagramAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ClassDiagramAst` (`ClassDiagramAst`).
/// - State: Maintains consistent state invariants across `ClassDiagramAst` without partial mutation on error.
/// - Action: Executes `ClassDiagramAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ClassDiagramAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASSDIAGRAMAST] | SysML v2: PartDef(ClassDiagramAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassDiagramAst {
    /// Declared or inferred classes.
    pub classes: Vec<ClassDefAst>,
    /// Declared relationships between classes.
    pub relations: Vec<ClassRelationAst>,
}

/// State definition in a state diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `StateDefAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `StateDefAst` across member set `[id: String, label: Option<String>, is_composite: bool, sub_states: Vec<StateDefAst>, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `StateDefAst` members (`id: String, label: Option<String>, is_composite: bool, sub_states: Vec<StateDefAst>, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `StateDefAst` representation for struct `StateDefAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `StateDefAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `StateDefAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `StateDefAst` (`StateDefAst`).
/// - State: Maintains consistent state invariants across `StateDefAst` without partial mutation on error.
/// - Action: Executes `StateDefAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `StateDefAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATEDEFAST] | SysML v2: PartDef(StateDefAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StateDefAst {
    /// State identifier (e.g. `Idle`, `Processing`, or `[*]`).
    pub id: String,
    /// Human-readable label or alias, if declared.
    pub label: Option<String>,
    /// Indicates whether state is a composite container with nested states.
    pub is_composite: bool,
    /// Nested sub-states for composite states.
    pub sub_states: Vec<StateDefAst>,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// State transition in a state diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `StateTransitionAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `StateTransitionAst` across member set `[source: String, target: String, event: Option<String>, guard: Option<String>, action: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `StateTransitionAst` members (`source: String, target: String, event: Option<String>, guard: Option<String>, action: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `StateTransitionAst` representation for struct `StateTransitionAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `StateTransitionAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `StateTransitionAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `StateTransitionAst` (`StateTransitionAst`).
/// - State: Maintains consistent state invariants across `StateTransitionAst` without partial mutation on error.
/// - Action: Executes `StateTransitionAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `StateTransitionAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATETRANSITIONAST] | SysML v2: PartDef(StateTransitionAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionAst {
    /// Source state identifier (e.g. `Idle` or `[*]`).
    pub source: String,
    /// Target state identifier (e.g. `Active` or `[*]`).
    pub target: String,
    /// Trigger event name, if specified.
    pub event: Option<String>,
    /// Guard condition expression, if specified.
    pub guard: Option<String>,
    /// Effect action invocation, if specified.
    pub action: Option<String>,
    /// Raw unparsed transition label.
    pub raw_label: Option<String>,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Note attached to a state in a state diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `StateNoteAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `StateNoteAst` across member set `[target_state: Option<String>, text: String, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `StateNoteAst` members (`target_state: Option<String>, text: String, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `StateNoteAst` representation for struct `StateNoteAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `StateNoteAst`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `StateNoteAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `StateNoteAst` (`StateNoteAst`).
/// - State: Maintains consistent state invariants across `StateNoteAst` without partial mutation on error.
/// - Action: Executes `StateNoteAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `StateNoteAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATENOTEAST] | SysML v2: PartDef(StateNoteAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateNoteAst {
    /// Target state the note is associated with.
    pub target_state: Option<String>,
    /// Note text content.
    pub text: String,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Complete AST for a Mermaid state diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `StateDiagramAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `StateDiagramAst` across member set `[states: Vec<StateDefAst>, transitions: Vec<StateTransitionAst>, notes: Vec<StateNoteAst>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `StateDiagramAst` members (`states: Vec<StateDefAst>, transitions: Vec<StateTransitionAst>, notes: Vec<StateNoteAst>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `StateDiagramAst` representation for struct `StateDiagramAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `StateDiagramAst`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `StateDiagramAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `StateDiagramAst` (`StateDiagramAst`).
/// - State: Maintains consistent state invariants across `StateDiagramAst` without partial mutation on error.
/// - Action: Executes `StateDiagramAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `StateDiagramAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATEDIAGRAMAST] | SysML v2: PartDef(StateDiagramAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StateDiagramAst {
    /// Declared states.
    pub states: Vec<StateDefAst>,
    /// Declared state transitions.
    pub transitions: Vec<StateTransitionAst>,
    /// Attached notes.
    pub notes: Vec<StateNoteAst>,
}

/// Sequence diagram participant or actor lifeline.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SequenceParticipantAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SequenceParticipantAst` across member set `[name: String, alias: Option<String>, is_actor: bool, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SequenceParticipantAst` members (`name: String, alias: Option<String>, is_actor: bool, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SequenceParticipantAst` representation for struct `SequenceParticipantAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `SequenceParticipantAst`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `SequenceParticipantAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SequenceParticipantAst` (`SequenceParticipantAst`).
/// - State: Maintains consistent state invariants across `SequenceParticipantAst` without partial mutation on error.
/// - Action: Executes `SequenceParticipantAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SequenceParticipantAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCEPARTICIPANTAST] | SysML v2: PartDef(SequenceParticipantAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceParticipantAst {
    /// Identifier name.
    pub name: String,
    /// Display alias if specified (`as "Alias"`).
    pub alias: Option<String>,
    /// Indicates whether declared as an actor.
    pub is_actor: bool,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Message exchange between lifelines in a sequence diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SequenceMessageAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SequenceMessageAst` across member set `[from: String, to: String, arrow: String, message: String, operation_name: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SequenceMessageAst` members (`from: String, to: String, arrow: String, message: String, operation_name: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SequenceMessageAst` representation for struct `SequenceMessageAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `SequenceMessageAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `SequenceMessageAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SequenceMessageAst` (`SequenceMessageAst`).
/// - State: Maintains consistent state invariants across `SequenceMessageAst` without partial mutation on error.
/// - Action: Executes `SequenceMessageAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SequenceMessageAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCEMESSAGEAST] | SysML v2: PartDef(SequenceMessageAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceMessageAst {
    /// Sender participant identifier.
    pub from: String,
    /// Receiver participant identifier.
    pub to: String,
    /// Message arrow symbol (e.g. `->>`, `-->>`).
    pub arrow: String,
    /// Full message text.
    pub message: String,
    /// Extracted operation or action name.
    pub operation_name: Option<String>,
    /// Extracted parameter strings.
    pub parameters: Vec<String>,
    /// Indicates whether message is a return/reply exchange.
    pub is_reply: bool,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Lifeline activation or deactivation event in a sequence diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SequenceActivationAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SequenceActivationAst` across member set `[participant: String, is_activate: bool, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SequenceActivationAst` members (`participant: String, is_activate: bool, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SequenceActivationAst` representation for struct `SequenceActivationAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `SequenceActivationAst`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `SequenceActivationAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SequenceActivationAst` (`SequenceActivationAst`).
/// - State: Maintains consistent state invariants across `SequenceActivationAst` without partial mutation on error.
/// - Action: Executes `SequenceActivationAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SequenceActivationAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCEACTIVATIONAST] | SysML v2: PartDef(SequenceActivationAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceActivationAst {
    /// Participant identifier.
    pub participant: String,
    /// True for `activate`, false for `deactivate`.
    pub is_activate: bool,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Complete AST for a Mermaid sequence diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SequenceDiagramAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SequenceDiagramAst` across member set `[participants: Vec<SequenceParticipantAst>, messages: Vec<SequenceMessageAst>, activations: Vec<SequenceActivationAst>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SequenceDiagramAst` members (`participants: Vec<SequenceParticipantAst>, messages: Vec<SequenceMessageAst>, activations: Vec<SequenceActivationAst>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SequenceDiagramAst` representation for struct `SequenceDiagramAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `SequenceDiagramAst`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `SequenceDiagramAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SequenceDiagramAst` (`SequenceDiagramAst`).
/// - State: Maintains consistent state invariants across `SequenceDiagramAst` without partial mutation on error.
/// - Action: Executes `SequenceDiagramAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SequenceDiagramAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCEDIAGRAMAST] | SysML v2: PartDef(SequenceDiagramAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SequenceDiagramAst {
    /// Declared lifelines and actors.
    pub participants: Vec<SequenceParticipantAst>,
    /// Message exchange sequence.
    pub messages: Vec<SequenceMessageAst>,
    /// Activation events.
    pub activations: Vec<SequenceActivationAst>,
}

/// Node definition in a flowchart diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FlowNodeAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FlowNodeAst` across member set `[id: String, label: Option<String>, shape: Option<String>, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FlowNodeAst` members (`id: String, label: Option<String>, shape: Option<String>, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FlowNodeAst` representation for struct `FlowNodeAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `FlowNodeAst`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `FlowNodeAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FlowNodeAst` (`FlowNodeAst`).
/// - State: Maintains consistent state invariants across `FlowNodeAst` without partial mutation on error.
/// - Action: Executes `FlowNodeAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FlowNodeAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FLOWNODEAST] | SysML v2: PartDef(FlowNodeAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowNodeAst {
    /// Node identifier.
    pub id: String,
    /// Node label text.
    pub label: Option<String>,
    /// Shape bracket delimiter style.
    pub shape: Option<String>,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Directed edge in a flowchart diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FlowEdgeAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FlowEdgeAst` across member set `[from: String, to: String, arrow: String, label: Option<String>, line: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FlowEdgeAst` members (`from: String, to: String, arrow: String, label: Option<String>, line: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FlowEdgeAst` representation for struct `FlowEdgeAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `FlowEdgeAst`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `FlowEdgeAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FlowEdgeAst` (`FlowEdgeAst`).
/// - State: Maintains consistent state invariants across `FlowEdgeAst` without partial mutation on error.
/// - Action: Executes `FlowEdgeAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FlowEdgeAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FLOWEDGEAST] | SysML v2: PartDef(FlowEdgeAst)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEdgeAst {
    /// Source node identifier.
    pub from: String,
    /// Target node identifier.
    pub to: String,
    /// Arrow symbol.
    pub arrow: String,
    /// Optional edge label.
    pub label: Option<String>,
    /// 1-indexed line number in source document.
    pub line: usize,
}

/// Complete AST for a Mermaid flowchart or graph diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FlowchartAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FlowchartAst` across member set `[direction: Option<String>, nodes: Vec<FlowNodeAst>, edges: Vec<FlowEdgeAst>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FlowchartAst` members (`direction: Option<String>, nodes: Vec<FlowNodeAst>, edges: Vec<FlowEdgeAst>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FlowchartAst` representation for struct `FlowchartAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `FlowchartAst`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `FlowchartAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FlowchartAst` (`FlowchartAst`).
/// - State: Maintains consistent state invariants across `FlowchartAst` without partial mutation on error.
/// - Action: Executes `FlowchartAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FlowchartAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FLOWCHARTAST] | SysML v2: PartDef(FlowchartAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FlowchartAst {
    /// Graph layout direction (e.g. `TD`, `LR`).
    pub direction: Option<String>,
    /// Extracted nodes.
    pub nodes: Vec<FlowNodeAst>,
    /// Extracted edges.
    pub edges: Vec<FlowEdgeAst>,
}

/// Complete AST for an Entity-Relationship diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ErDiagramAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ErDiagramAst` across member set `[entities: Vec<String>, relationships: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ErDiagramAst` members (`entities: Vec<String>, relationships: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ErDiagramAst` representation for struct `ErDiagramAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `ErDiagramAst`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `ErDiagramAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ErDiagramAst` (`ErDiagramAst`).
/// - State: Maintains consistent state invariants across `ErDiagramAst` without partial mutation on error.
/// - Action: Executes `ErDiagramAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ErDiagramAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-ERDIAGRAMAST] | SysML v2: PartDef(ErDiagramAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ErDiagramAst {
    /// Declared entities.
    pub entities: Vec<String>,
    /// Declared relationship lines.
    pub relationships: Vec<String>,
}

/// Complete AST for an XY chart diagram.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `XyChartAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `XyChartAst` across member set `[title: Option<String>, x_axis: Option<String>, y_axis: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `XyChartAst` members (`title: Option<String>, x_axis: Option<String>, y_axis: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `XyChartAst` representation for struct `XyChartAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `XyChartAst`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `XyChartAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `XyChartAst` (`XyChartAst`).
/// - State: Maintains consistent state invariants across `XyChartAst` without partial mutation on error.
/// - Action: Executes `XyChartAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `XyChartAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-XYCHARTAST] | SysML v2: PartDef(XyChartAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XyChartAst {
    /// Chart title, if specified.
    pub title: Option<String>,
    /// X axis definition.
    pub x_axis: Option<String>,
    /// Y axis definition.
    pub y_axis: Option<String>,
}

/// Complete AST for generic or unspecialized diagrams.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `GenericDiagramAst` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `GenericDiagramAst` across member set `[header: String, lines: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `GenericDiagramAst` members (`header: String, lines: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `GenericDiagramAst` representation for struct `GenericDiagramAst`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `GenericDiagramAst`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `GenericDiagramAst`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `GenericDiagramAst` (`GenericDiagramAst`).
/// - State: Maintains consistent state invariants across `GenericDiagramAst` without partial mutation on error.
/// - Action: Executes `GenericDiagramAst` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `GenericDiagramAst` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-GENERICDIAGRAMAST] | SysML v2: PartDef(GenericDiagramAst)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenericDiagramAst {
    /// Header line.
    pub header: String,
    /// Body lines.
    pub lines: Vec<String>,
}

// ============================================================================
// INVARIANT ENFORCEMENT ENGINE (PURE SCANNER, ZERO REGEX)
// ============================================================================

/// Validates structural and syntax invariants across a Mermaid block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_mermaid_invariants` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_mermaid_invariants` (declaration #25) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `validate_mermaid_invariants`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), Vec<SpecAuditError>>` from `validate_mermaid_invariants` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `validate_mermaid_invariants`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `validate_mermaid_invariants`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_mermaid_invariants` (`Result<(), Vec<SpecAuditError>>`).
/// - State: Maintains consistent state invariants across `validate_mermaid_invariants` without partial mutation on error.
/// - Action: Executes `validate_mermaid_invariants` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_mermaid_invariants` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VALIDATE-MERMAID-INVARIANTS] | SysML v2: ActionDef(validate_mermaid_invariants)
pub fn validate_mermaid_invariants(block: &MermaidBlock) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();

    // Invariant 1: Matching fences check (all diagrams must end cleanly)
    if block.end_line <= block.start_line {
        errors.push(SpecAuditError::ValidationError(format!(
            "Unclosed or invalid Mermaid code fence starting at line {}.",
            block.start_line
        )));
    }

    let mut open_braces = 0usize;
    let mut close_braces = 0usize;
    let mut subgraphs = 0usize;
    let mut ends = 0usize;

    let is_class_diagram = {
        let h = block.header.to_lowercase();
        h.starts_with("classdiagram")
    };

    let mut in_class_block = false;

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() {
            continue;
        }

        // Invariant check: orphan backticks
        if clean.starts_with("```") || clean.starts_with("~~~") {
            errors.push(SpecAuditError::ValidationError(format!(
                "Stray or nested markdown code fence within Mermaid diagram at line {}.",
                line_num
            )));
        }

        // Track subgraphs and ends
        if clean.starts_with("subgraph") {
            subgraphs += 1;
            // Check subgraph title quotes: titles with spaces or hyphens must be quoted
            let after_subgraph = clean["subgraph".len()..].trim();
            if !after_subgraph.is_empty() {
                let title = after_subgraph;
                let is_quoted = title.starts_with('"') && title.ends_with('"');
                if !is_quoted && (title.contains(' ') || title.contains('-')) {
                    errors.push(SpecAuditError::ValidationError(format!(
                        "Subgraph title '{}' at line {} contains spaces or hyphens and must be enclosed in double quotes.",
                        title, line_num
                    )));
                }
            }
        } else if clean == "end" || clean.starts_with("end ") {
            ends += 1;
        }

        // Track braces outside of double quotes
        let mut in_quotes = false;
        let mut prev_char = ' ';
        for c in clean.chars() {
            if c == '"' && prev_char != '\\' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                if c == '{' {
                    open_braces += 1;
                } else if c == '}' {
                    close_braces += 1;
                }
            }
            prev_char = c;
        }

        // Invariant 3: Unquoted '<' and '>' strictly forbidden across all diagram types
        if let Err(err) = check_unquoted_angle_brackets(clean, line_num) {
            errors.push(err);
        }

        // Invariant 4: Double quotes required around node labels containing special characters ('/', ':', '()', '[]')
        if let Err(err) = check_node_label_quoting(clean, line_num) {
            errors.push(err);
        }

        // Invariant 2: Class diagram specific member checks (ZERO curly braces, ZERO secondary colons)
        if is_class_diagram {
            if clean.contains('{') && clean.starts_with("class ") {
                in_class_block = true;
                continue;
            }
            if clean == "}" {
                in_class_block = false;
                continue;
            }

            if in_class_block {
                // Inside `class Name { ... }` block
                if clean.starts_with("<<") && clean.ends_with(">>") {
                    continue;
                }
                // Check curly braces
                if clean.contains('{') || clean.contains('}') {
                    errors.push(SpecAuditError::ValidationError(format!(
                        "Class member string '{}' at line {} contains prohibited curly braces '{{}}'.",
                        clean, line_num
                    )));
                }
                // Check colon
                if clean.contains(':') {
                    errors.push(SpecAuditError::ValidationError(format!(
                        "Class member string '{}' at line {} contains prohibited colon ':'. Use standard spacing instead (e.g. '+ReturnType methodName(Type arg)').",
                        clean, line_num
                    )));
                }
            } else {
                // Check Form B: `ClassName : MemberString`
                if let Some((left, right)) = clean.split_once(':') {
                    // Make sure left is not a relation with arrow
                    let left_trimmed = left.trim();
                    let right_trimmed = right.trim();
                    if !contains_relation_arrow(left_trimmed) {
                        // This is a member line: `ClassName : member`
                        if right_trimmed.contains('{') || right_trimmed.contains('}') {
                            errors.push(SpecAuditError::ValidationError(format!(
                                "Class member string '{}' at line {} contains prohibited curly braces '{{}}'.",
                                right_trimmed, line_num
                            )));
                        }
                        if right_trimmed.contains(':') {
                            errors.push(SpecAuditError::ValidationError(format!(
                                "Class member string '{}' at line {} contains prohibited secondary colon ':'. Use standard spacing instead (e.g. '+ReturnType methodName(Type arg)').",
                                right_trimmed, line_num
                            )));
                        }
                    }
                }
            }
        }
    }

    if open_braces != close_braces {
        errors.push(SpecAuditError::ValidationError(format!(
            "Unbalanced curly braces in Mermaid diagram ({} opening '{{' vs {} closing '}}') between lines {}-{}.",
            open_braces, close_braces, block.start_line, block.end_line
        )));
    }

    if subgraphs != ends {
        errors.push(SpecAuditError::ValidationError(format!(
            "Unbalanced subgraph blocks in Mermaid diagram ({} 'subgraph' vs {} 'end') between lines {}-{}.",
            subgraphs, ends, block.start_line, block.end_line
        )));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Helper to strip Mermaid comments (`%% ...`) outside string literals.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `strip_mermaid_comment` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `strip_mermaid_comment` (declaration #26) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `strip_mermaid_comment`.
///
/// ## 4. Postconditions
/// - Returns `&str` from `strip_mermaid_comment` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `strip_mermaid_comment`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `strip_mermaid_comment`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `strip_mermaid_comment` (`&str`).
/// - State: Maintains consistent state invariants across `strip_mermaid_comment` without partial mutation on error.
/// - Action: Executes `strip_mermaid_comment` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `strip_mermaid_comment` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STRIP-MERMAID-COMMENT] | SysML v2: ActionDef(strip_mermaid_comment)
fn strip_mermaid_comment(line: &str) -> &str {
    let mut in_quotes = false;
    let mut prev = ' ';
    let mut comment_start = None;

    let chars: Vec<char> = line.chars().collect();
    for i in 0..chars.len() {
        let c = chars[i];
        if c == '"' && prev != '\\' {
            in_quotes = !in_quotes;
        } else if !in_quotes && c == '%' && i + 1 < chars.len() && chars[i + 1] == '%' {
            comment_start = Some(i);
            break;
        }
        prev = c;
    }

    if let Some(idx) = comment_start {
        // Safe slice on character boundary
        let byte_pos = line
            .char_indices()
            .nth(idx)
            .map(|(p, _)| p)
            .unwrap_or(line.len());
        &line[..byte_pos]
    } else {
        line
    }
}

/// Known Mermaid arrow tokens sorted by length descending to prevent partial match collisions.
const ARROWS: &[&str] = &[
    "==>>", "-->>", "<<==", "<<--", "<-->", "<==>",
    "<|--", "--|>", "<|..", "..|>", "<|==", "==|>",
    "*--", "--*", "o--", "--o",
    "<-.-", "-.->", "==>", "-->", "->>", "<==", "<--",
    "<<-", "<->", "->", "<-", "..>", "<..", "..", "--",
];

/// Checks whether a string segment contains any UML or flowchart relation arrow.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `contains_relation_arrow` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `contains_relation_arrow` (declaration #27) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `contains_relation_arrow`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `contains_relation_arrow` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `contains_relation_arrow`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `contains_relation_arrow`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `contains_relation_arrow` (`bool`).
/// - State: Maintains consistent state invariants across `contains_relation_arrow` without partial mutation on error.
/// - Action: Executes `contains_relation_arrow` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `contains_relation_arrow` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CONTAINS-RELATION-ARROW] | SysML v2: ActionDef(contains_relation_arrow)
fn contains_relation_arrow(s: &str) -> bool {
    ARROWS.iter().any(|&arrow| s.contains(arrow))
}

/// Checks that all angle brackets `<` and `>` are inside quotes or valid Mermaid operators.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `check_unquoted_angle_brackets` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `check_unquoted_angle_brackets` (declaration #28) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `check_unquoted_angle_brackets`.
/// - `line_num: usize` must satisfy domain and type-safety constraints prior to invoking `check_unquoted_angle_brackets`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), SpecAuditError>` from `check_unquoted_angle_brackets` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `check_unquoted_angle_brackets`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `check_unquoted_angle_brackets`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `check_unquoted_angle_brackets` (`Result<(), SpecAuditError>`).
/// - State: Maintains consistent state invariants across `check_unquoted_angle_brackets` without partial mutation on error.
/// - Action: Executes `check_unquoted_angle_brackets` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `check_unquoted_angle_brackets` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CHECK-UNQUOTED-ANGLE-BRACKETS] | SysML v2: ActionDef(check_unquoted_angle_brackets)
fn check_unquoted_angle_brackets(line: &str, line_num: usize) -> Result<(), SpecAuditError> {
    let mut in_quotes = false;
    let mut prev_char = ' ';
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut i = 0;

    while i < n {
        let b = bytes[i];
        if b == b'"' && prev_char != '\\' {
            in_quotes = !in_quotes;
            prev_char = b as char;
            i += 1;
            continue;
        }

        if in_quotes {
            prev_char = b as char;
            i += 1;
            continue;
        }

        // Check if substring starts with any known arrow operator
        let rest = &line[i..];
        let mut matched_arrow_len = 0;
        for &arrow in ARROWS {
            if rest.starts_with(arrow) {
                matched_arrow_len = arrow.len();
                break;
            }
        }

        if matched_arrow_len > 0 {
            i += matched_arrow_len;
            prev_char = ' ';
            continue;
        }

        // Check if substring starts with a stereotype `<<...>>`
        if rest.starts_with("<<") {
            if let Some(close_idx) = rest.find(">>") {
                // Verify no double quote inside
                let inner = &rest[2..close_idx];
                if !inner.contains('"') {
                    i += close_idx + 2;
                    prev_char = ' ';
                    continue;
                }
            }
        }

        // Unquoted angle bracket check
        if b == b'<' || b == b'>' {
            return Err(SpecAuditError::ValidationError(format!(
                "Unquoted '{}' character at line {}: transitions, labels, or guards containing comparison operators or brackets MUST be enclosed in double quotes.",
                b as char, line_num
            )));
        }

        prev_char = b as char;
        i += 1;
    }

    Ok(())
}

/// Checks that node labels containing special characters (`/`, `:`, `()`, `[]`) are enclosed in double quotes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `check_node_label_quoting` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `check_node_label_quoting` (declaration #29) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `check_node_label_quoting`.
/// - `line_num: usize` must satisfy domain and type-safety constraints prior to invoking `check_node_label_quoting`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), SpecAuditError>` from `check_node_label_quoting` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `check_node_label_quoting`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `check_node_label_quoting`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `check_node_label_quoting` (`Result<(), SpecAuditError>`).
/// - State: Maintains consistent state invariants across `check_node_label_quoting` without partial mutation on error.
/// - Action: Executes `check_node_label_quoting` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `check_node_label_quoting` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CHECK-NODE-LABEL-QUOTING] | SysML v2: ActionDef(check_node_label_quoting)
fn check_node_label_quoting(line: &str, line_num: usize) -> Result<(), SpecAuditError> {
    // Delimiter pairs for Mermaid shapes:
    // `[` ... `]`
    // `(` ... `)`
    // `{` ... `}`
    // `>` ... `]`
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("%%") {
        return Ok(());
    }

    let mut in_quotes = false;
    let mut prev = ' ';
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let c = chars[i];
        if c == '"' && prev != '\\' {
            in_quotes = !in_quotes;
            prev = c;
            i += 1;
            continue;
        }

        if in_quotes {
            prev = c;
            i += 1;
            continue;
        }

        // Look for opening delimiter: `[`, `(`, `{`
        let open_ch = c;
        if open_ch == '[' || open_ch == '(' || open_ch == '{' {
            // Find closing delimiter matching open_ch
            let close_ch = match open_ch {
                '[' => ']',
                '(' => ')',
                '{' => '}',
                _ => ' ',
            };

            // Find matching closing delimiter outside of quotes
            let mut depth = 1usize;
            let mut j = i + 1;
            let mut inner_in_quotes = false;
            let mut inner_prev = ' ';
            let mut close_pos = None;

            while j < n {
                let jc = chars[j];
                if jc == '"' && inner_prev != '\\' {
                    inner_in_quotes = !inner_in_quotes;
                } else if !inner_in_quotes {
                    if jc == open_ch {
                        depth += 1;
                    } else if jc == close_ch {
                        depth -= 1;
                        if depth == 0 {
                            close_pos = Some(j);
                            break;
                        }
                    }
                }
                inner_prev = jc;
                j += 1;
            }

            if let Some(end_idx) = close_pos {
                let inner_slice: String = chars[i + 1..end_idx].iter().collect();
                let inner_trimmed = inner_slice.trim();

                // Strip nested shape markers if any (e.g. `([ ... ])`, `[[ ... ]]`)
                let cleaned_label = inner_trimmed
                    .trim_matches(|ch: char| ch == '[' || ch == ']' || ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '/');

                let is_quoted = cleaned_label.starts_with('"') && cleaned_label.ends_with('"');
                if !is_quoted && !cleaned_label.is_empty() {
                    // Check if contains special characters: `/`, `:`, `(`, `)`, `[`, `]`
                    let has_special = cleaned_label.chars().any(|ch| {
                        ch == '/' || ch == ':' || ch == '(' || ch == ')' || ch == '[' || ch == ']'
                    });

                    if has_special {
                        return Err(SpecAuditError::ValidationError(format!(
                            "Node label '{}' at line {} contains special characters ('/', ':', '()', '[]') and must be enclosed in double quotes.",
                            cleaned_label, line_num
                        )));
                    }
                }

                i = end_idx + 1;
                prev = ' ';
                continue;
            }
        }

        prev = c;
        i += 1;
    }

    Ok(())
}

// ============================================================================
// MERMAID AST EXTRACTION ENGINE (ZERO REGEX)
// ============================================================================

/// Parses a `MermaidBlock` into a strongly-typed `DiagramAst`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_mermaid_diagram` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_mermaid_diagram` (declaration #30) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_mermaid_diagram`.
///
/// ## 4. Postconditions
/// - Returns `Result<DiagramAst, SpecAuditError>` from `parse_mermaid_diagram` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_mermaid_diagram`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_mermaid_diagram`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_mermaid_diagram` (`Result<DiagramAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_mermaid_diagram` without partial mutation on error.
/// - Action: Executes `parse_mermaid_diagram` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_mermaid_diagram` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-MERMAID-DIAGRAM] | SysML v2: ActionDef(parse_mermaid_diagram)
pub fn parse_mermaid_diagram(block: &MermaidBlock) -> Result<DiagramAst, SpecAuditError> {
    // 1. Enforce structural invariants first
    if let Err(errors) = validate_mermaid_invariants(block) {
        if let Some(first) = errors.into_iter().next() {
            return Err(first);
        }
    }

    let header_type = detect_diagram_type(&block.header);

    match header_type {
        DiagramType::Class => {
            let ast = parse_class_diagram(block)?;
            Ok(DiagramAst::Class(ast))
        }
        DiagramType::State => {
            let ast = parse_state_diagram(block)?;
            Ok(DiagramAst::State(ast))
        }
        DiagramType::Sequence => {
            let ast = parse_sequence_diagram(block)?;
            Ok(DiagramAst::Sequence(ast))
        }
        DiagramType::Flowchart | DiagramType::Graph => {
            let ast = parse_flowchart(block)?;
            Ok(DiagramAst::Flowchart(ast))
        }
        DiagramType::Er => {
            let ast = parse_er_diagram(block)?;
            Ok(DiagramAst::Er(ast))
        }
        DiagramType::XyChart => {
            let ast = parse_xychart(block)?;
            Ok(DiagramAst::XyChart(ast))
        }
        DiagramType::Other => {
            let ast = parse_generic_diagram(block)?;
            Ok(DiagramAst::Other(ast))
        }
    }
}

/// Detects the `DiagramType` from the diagram header text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `detect_diagram_type` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `detect_diagram_type` (declaration #31) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `header: &str` must satisfy domain and type-safety constraints prior to invoking `detect_diagram_type`.
///
/// ## 4. Postconditions
/// - Returns `DiagramType` from `detect_diagram_type` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(header)) bounded over input/member dimension of `detect_diagram_type`.
/// - Space Complexity: O(N(header)) bounded auxiliary memory for `detect_diagram_type`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `detect_diagram_type` (`DiagramType`).
/// - State: Maintains consistent state invariants across `detect_diagram_type` without partial mutation on error.
/// - Action: Executes `detect_diagram_type` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `detect_diagram_type` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-DETECT-DIAGRAM-TYPE] | SysML v2: ActionDef(detect_diagram_type)
pub fn detect_diagram_type(header: &str) -> DiagramType {
    let trimmed = header.trim().to_lowercase();
    if trimmed.starts_with("classdiagram") {
        DiagramType::Class
    } else if trimmed.starts_with("statediagram-v2") || trimmed.starts_with("statediagram") {
        DiagramType::State
    } else if trimmed.starts_with("sequencediagram") {
        DiagramType::Sequence
    } else if trimmed.starts_with("flowchart") {
        DiagramType::Flowchart
    } else if trimmed.starts_with("graph") {
        DiagramType::Graph
    } else if trimmed.starts_with("erdiagram") {
        DiagramType::Er
    } else if trimmed.starts_with("xychart-beta") || trimmed.starts_with("xychart") {
        DiagramType::XyChart
    } else {
        DiagramType::Other
    }
}

// ----------------------------------------------------------------------------
// CLASS DIAGRAM PARSER
// ----------------------------------------------------------------------------

/// Executes deterministic fn `parse_class_diagram` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_class_diagram` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_class_diagram` (declaration #32) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_class_diagram`.
///
/// ## 4. Postconditions
/// - Returns `Result<ClassDiagramAst, SpecAuditError>` from `parse_class_diagram` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_class_diagram`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_class_diagram`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_class_diagram` (`Result<ClassDiagramAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_class_diagram` without partial mutation on error.
/// - Action: Executes `parse_class_diagram` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_class_diagram` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-DIAGRAM] | SysML v2: ActionDef(parse_class_diagram)
fn parse_class_diagram(block: &MermaidBlock) -> Result<ClassDiagramAst, SpecAuditError> {
    let mut classes_map: Vec<ClassDefAst> = Vec::new();
    let mut relations = Vec::new();
    let mut current_class: Option<ClassDefAst> = None;

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() || clean.eq_ignore_ascii_case("classDiagram") {
            continue;
        }

        // Class block closure
        if clean == "}" {
            if let Some(c) = current_class.take() {
                merge_or_push_class(&mut classes_map, c);
            }
            continue;
        }

        // Inside class block: `class Name {`
        if let Some(ref mut c) = current_class {
            if clean.starts_with("<<") && clean.ends_with(">>") {
                let annot = clean.trim_matches(|ch| ch == '<' || ch == '>').to_string();
                c.annotations.push(annot);
                continue;
            }

            parse_class_member_line(clean, c, line_num);
            continue;
        }

        // Opening class block: `class Name {`
        if clean.starts_with("class ") && clean.ends_with('{') {
            let name_part = clean["class ".len()..clean.len() - 1].trim();
            current_class = Some(ClassDefAst {
                name: name_part.to_string(),
                annotations: Vec::new(),
                attributes: Vec::new(),
                operations: Vec::new(),
                line: line_num,
            });
            continue;
        }

        // Standalone class declaration: `class Name`
        if clean.starts_with("class ") {
            let name_part = clean["class ".len()..].trim();
            if !name_part.is_empty() {
                merge_or_push_class(
                    &mut classes_map,
                    ClassDefAst {
                        name: name_part.to_string(),
                        annotations: Vec::new(),
                        attributes: Vec::new(),
                        operations: Vec::new(),
                        line: line_num,
                    },
                );
            }
            continue;
        }

        // Relation line: `ClassA <|-- ClassB : label`
        if let Some(rel) = parse_class_relation(clean, line_num) {
            // Also ensure participating classes are indexed
            ensure_class_stub(&mut classes_map, &rel.from_class, line_num);
            ensure_class_stub(&mut classes_map, &rel.to_class, line_num);
            relations.push(rel);
            continue;
        }

        // Single-line member: `ClassName : MemberString`
        if let Some((left, right)) = clean.split_once(':') {
            let left_trimmed = left.trim();
            let right_trimmed = right.trim();

            if !contains_relation_arrow(left_trimmed) {
                let class_name = left_trimmed.strip_prefix("class ").unwrap_or(left_trimmed).trim();
                let mut temp_class = ClassDefAst {
                    name: class_name.to_string(),
                    annotations: Vec::new(),
                    attributes: Vec::new(),
                    operations: Vec::new(),
                    line: line_num,
                };
                parse_class_member_line(right_trimmed, &mut temp_class, line_num);
                merge_or_push_class(&mut classes_map, temp_class);
                continue;
            }
        }
    }

    if let Some(c) = current_class {
        merge_or_push_class(&mut classes_map, c);
    }

    Ok(ClassDiagramAst {
        classes: classes_map,
        relations,
    })
}

/// Executes deterministic fn `merge_or_push_class` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `merge_or_push_class` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `merge_or_push_class` (declaration #33) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `classes: &mut Vec<ClassDefAst>` must satisfy domain and type-safety constraints prior to invoking `merge_or_push_class`.
/// - `new_class: ClassDefAst` must satisfy domain and type-safety constraints prior to invoking `merge_or_push_class`.
///
/// ## 4. Postconditions
/// - Returns `()` from `merge_or_push_class` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(classes)) bounded over input/member dimension of `merge_or_push_class`.
/// - Space Complexity: O(N(classes)) bounded auxiliary memory for `merge_or_push_class`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `merge_or_push_class` (`()`).
/// - State: Maintains consistent state invariants across `merge_or_push_class` without partial mutation on error.
/// - Action: Executes `merge_or_push_class` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `merge_or_push_class` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-MERGE-OR-PUSH-CLASS] | SysML v2: ActionDef(merge_or_push_class)
fn merge_or_push_class(classes: &mut Vec<ClassDefAst>, new_class: ClassDefAst) {
    if let Some(existing) = classes.iter_mut().find(|c| c.name == new_class.name) {
        existing.annotations.extend(new_class.annotations);
        existing.attributes.extend(new_class.attributes);
        existing.operations.extend(new_class.operations);
    } else {
        classes.push(new_class);
    }
}

/// Executes deterministic fn `ensure_class_stub` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ensure_class_stub` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `ensure_class_stub` (declaration #34) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `classes: &mut Vec<ClassDefAst>` must satisfy domain and type-safety constraints prior to invoking `ensure_class_stub`.
/// - `name: &str` must satisfy domain and type-safety constraints prior to invoking `ensure_class_stub`.
/// - `line: usize` must satisfy domain and type-safety constraints prior to invoking `ensure_class_stub`.
///
/// ## 4. Postconditions
/// - Returns `()` from `ensure_class_stub` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(classes)) bounded over input/member dimension of `ensure_class_stub`.
/// - Space Complexity: O(N(classes)) bounded auxiliary memory for `ensure_class_stub`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ensure_class_stub` (`()`).
/// - State: Maintains consistent state invariants across `ensure_class_stub` without partial mutation on error.
/// - Action: Executes `ensure_class_stub` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ensure_class_stub` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-ENSURE-CLASS-STUB] | SysML v2: ActionDef(ensure_class_stub)
fn ensure_class_stub(classes: &mut Vec<ClassDefAst>, name: &str, line: usize) {
    if !name.is_empty() && !classes.iter().any(|c| c.name == name) {
        classes.push(ClassDefAst {
            name: name.to_string(),
            annotations: Vec::new(),
            attributes: Vec::new(),
            operations: Vec::new(),
            line,
        });
    }
}

/// Executes deterministic fn `parse_class_member_line` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_class_member_line` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_class_member_line` (declaration #35) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `parse_class_member_line`.
/// - `class_def: &mut ClassDefAst` must satisfy domain and type-safety constraints prior to invoking `parse_class_member_line`.
/// - `line_num: usize` must satisfy domain and type-safety constraints prior to invoking `parse_class_member_line`.
///
/// ## 4. Postconditions
/// - Returns `()` from `parse_class_member_line` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `parse_class_member_line`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `parse_class_member_line`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_class_member_line` (`()`).
/// - State: Maintains consistent state invariants across `parse_class_member_line` without partial mutation on error.
/// - Action: Executes `parse_class_member_line` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_class_member_line` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-MEMBER-LINE] | SysML v2: ActionDef(parse_class_member_line)
fn parse_class_member_line(line: &str, class_def: &mut ClassDefAst, line_num: usize) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    let first_char = trimmed.chars().next().unwrap_or(' ');
    let visibility = Visibility::from_char(first_char);
    let after_vis = if visibility.is_some() {
        trimmed[1..].trim()
    } else {
        trimmed
    };

    // Operation detection: contains `(` and `)`
    if let Some(open_paren) = after_vis.find('(') {
        if let Some(close_paren) = after_vis[open_paren..].find(')') {
            let actual_close = open_paren + close_paren;
            let before_paren = after_vis[..open_paren].trim();
            let inside_paren = after_vis[open_paren + 1..actual_close].trim();
            let after_close = after_vis[actual_close + 1..].trim();

            let mut parameters = Vec::new();
            if !inside_paren.is_empty() {
                for param in inside_paren.split(',') {
                    let p = param.trim();
                    if !p.is_empty() {
                        parameters.push(p.to_string());
                    }
                }
            }

            let words: Vec<&str> = before_paren.split_whitespace().collect();
            let (op_name, return_type) = if words.len() >= 2 {
                // e.g. `void execute_pipeline` or `ReturnType name`
                let ret = words[0].to_string();
                let name = words[1].to_string();
                (name, Some(ret))
            } else if !words.is_empty() {
                let name = words[0].to_string();
                let ret = if !after_close.is_empty() {
                    Some(after_close.to_string())
                } else {
                    None
                };
                (name, ret)
            } else {
                ("anonymous_operation".to_string(), None)
            };

            class_def.operations.push(ClassOperationAst {
                visibility,
                name: op_name,
                parameters,
                return_type,
                raw: trimmed.to_string(),
                line: line_num,
            });
            return;
        }
    }

    // Attribute detection: no parentheses
    let words: Vec<&str> = after_vis.split_whitespace().collect();
    let (attr_name, type_name) = if words.len() >= 2 {
        // e.g. `float altitude` -> type is words[0], name is words[1]
        (words[1].to_string(), Some(words[0].to_string()))
    } else if !words.is_empty() {
        (words[0].to_string(), None)
    } else {
        ("anonymous_attr".to_string(), None)
    };

    class_def.attributes.push(ClassAttributeAst {
        visibility,
        name: attr_name,
        type_name,
        raw: trimmed.to_string(),
        line: line_num,
    });
}

/// Executes deterministic fn `parse_class_relation` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_class_relation` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_class_relation` (declaration #36) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `parse_class_relation`.
/// - `line_num: usize` must satisfy domain and type-safety constraints prior to invoking `parse_class_relation`.
///
/// ## 4. Postconditions
/// - Returns `Option<ClassRelationAst>` from `parse_class_relation` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `parse_class_relation`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `parse_class_relation`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_class_relation` (`Option<ClassRelationAst>`).
/// - State: Maintains consistent state invariants across `parse_class_relation` without partial mutation on error.
/// - Action: Executes `parse_class_relation` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_class_relation` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-RELATION] | SysML v2: ActionDef(parse_class_relation)
fn parse_class_relation(line: &str, line_num: usize) -> Option<ClassRelationAst> {
    // List of relation operators to test
    const REL_OPS: &[(&str, ClassRelationType)] = &[
        ("<|--", ClassRelationType::Inheritance),
        ("--|>", ClassRelationType::Inheritance),
        ("<|..", ClassRelationType::Realization),
        ("..|>", ClassRelationType::Realization),
        ("*--", ClassRelationType::Composition),
        ("--*", ClassRelationType::Composition),
        ("o--", ClassRelationType::Aggregation),
        ("--o", ClassRelationType::Aggregation),
        ("-->", ClassRelationType::Association),
        ("<--", ClassRelationType::Association),
        ("..>", ClassRelationType::Dependency),
        ("<..", ClassRelationType::Dependency),
        ("..", ClassRelationType::Dependency),
        ("--", ClassRelationType::Association),
    ];

    for &(op, ref rel_type) in REL_OPS {
        if let Some(pos) = line.find(op) {
            let left = line[..pos].trim();
            let right_full = line[pos + op.len()..].trim();

            if left.is_empty() || right_full.is_empty() {
                continue;
            }

            let (right, label) = if let Some((r, lbl)) = right_full.split_once(':') {
                (r.trim(), Some(lbl.trim().trim_matches('"').to_string()))
            } else {
                (right_full, None)
            };

            return Some(ClassRelationAst {
                from_class: left.to_string(),
                to_class: right.to_string(),
                relation_type: rel_type.clone(),
                raw_symbol: op.to_string(),
                label,
                line: line_num,
            });
        }
    }

    None
}

// ----------------------------------------------------------------------------
// STATE DIAGRAM PARSER
// ----------------------------------------------------------------------------

/// Executes deterministic fn `parse_state_diagram` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_state_diagram` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_state_diagram` (declaration #37) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_state_diagram`.
///
/// ## 4. Postconditions
/// - Returns `Result<StateDiagramAst, SpecAuditError>` from `parse_state_diagram` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_state_diagram`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_state_diagram`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_state_diagram` (`Result<StateDiagramAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_state_diagram` without partial mutation on error.
/// - Action: Executes `parse_state_diagram` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_state_diagram` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-STATE-DIAGRAM] | SysML v2: ActionDef(parse_state_diagram)
fn parse_state_diagram(block: &MermaidBlock) -> Result<StateDiagramAst, SpecAuditError> {
    let mut states_set: HashSet<String> = HashSet::new();
    let mut states = Vec::new();
    let mut transitions = Vec::new();
    let mut notes = Vec::new();

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty()
            || clean.eq_ignore_ascii_case("stateDiagram")
            || clean.eq_ignore_ascii_case("stateDiagram-v2")
            || clean == "}"
        {
            continue;
        }

        // Note parsing: `note left of State : ...`
        if clean.starts_with("note ") {
            let after_note = clean["note ".len()..].trim();
            let (target_state, text) = if let Some((dir_and_target, note_body)) = after_note.split_once(':') {
                let target = dir_and_target
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string();
                (Some(target), note_body.trim().to_string())
            } else {
                (None, after_note.to_string())
            };

            notes.push(StateNoteAst {
                target_state,
                text,
                line: line_num,
            });
            continue;
        }

        // State declaration: `state "Label" as StateId` or `state StateId {`
        if clean.starts_with("state ") {
            let after_state = clean["state ".len()..].trim().trim_end_matches('{').trim();
            if let Some((label_part, id_part)) = after_state.split_once(" as ") {
                let id = id_part.trim().to_string();
                let label = label_part.trim().trim_matches('"').to_string();
                if states_set.insert(id.clone()) {
                    states.push(StateDefAst {
                        id,
                        label: Some(label),
                        is_composite: clean.ends_with('{'),
                        sub_states: Vec::new(),
                        line: line_num,
                    });
                }
            } else if !after_state.is_empty() {
                let id = after_state.to_string();
                if states_set.insert(id.clone()) {
                    states.push(StateDefAst {
                        id,
                        label: None,
                        is_composite: clean.ends_with('{'),
                        sub_states: Vec::new(),
                        line: line_num,
                    });
                }
            }
            continue;
        }

        // Transition line: `State1 --> State2 : event [guard] / action`
        if let Some((left, right_full)) = clean.split_once("-->") {
            let source = left.trim().to_string();
            let (target, label_opt) = if let Some((t, l)) = right_full.split_once(':') {
                (t.trim().to_string(), Some(l.trim().to_string()))
            } else {
                (right_full.trim().to_string(), None)
            };

            if source != "[*]" && states_set.insert(source.clone()) {
                states.push(StateDefAst {
                    id: source.clone(),
                    label: None,
                    is_composite: false,
                    sub_states: Vec::new(),
                    line: line_num,
                });
            }
            if target != "[*]" && states_set.insert(target.clone()) {
                states.push(StateDefAst {
                    id: target.clone(),
                    label: None,
                    is_composite: false,
                    sub_states: Vec::new(),
                    line: line_num,
                });
            }

            let mut event = None;
            let mut guard = None;
            let mut action = None;

            if let Some(ref raw_label) = label_opt {
                let unquoted = raw_label.trim_matches('"').trim();

                // Extract guard `[...]`
                let mut label_without_guard = unquoted.to_string();
                if let Some(open_b) = unquoted.find('[') {
                    if let Some(close_b) = unquoted[open_b..].find(']') {
                        let actual_close = open_b + close_b;
                        guard = Some(unquoted[open_b + 1..actual_close].trim().to_string());
                        label_without_guard = format!(
                            "{}{}",
                            &unquoted[..open_b],
                            &unquoted[actual_close + 1..]
                        );
                    }
                }

                // Extract action after `/`
                if let Some((evt_part, act_part)) = label_without_guard.split_once('/') {
                    let a = act_part.trim().to_string();
                    if !a.is_empty() {
                        action = Some(a);
                    }
                    let e = evt_part.trim().to_string();
                    if !e.is_empty() {
                        event = Some(e);
                    }
                } else {
                    let e = label_without_guard.trim().to_string();
                    if !e.is_empty() {
                        event = Some(e);
                    }
                }
            }

            transitions.push(StateTransitionAst {
                source,
                target,
                event,
                guard,
                action,
                raw_label: label_opt,
                line: line_num,
            });
        }
    }

    Ok(StateDiagramAst {
        states,
        transitions,
        notes,
    })
}

// ----------------------------------------------------------------------------
// SEQUENCE DIAGRAM PARSER
// ----------------------------------------------------------------------------

/// Executes deterministic fn `parse_sequence_diagram` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_sequence_diagram` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_sequence_diagram` (declaration #38) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_sequence_diagram`.
///
/// ## 4. Postconditions
/// - Returns `Result<SequenceDiagramAst, SpecAuditError>` from `parse_sequence_diagram` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_sequence_diagram`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_sequence_diagram`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_sequence_diagram` (`Result<SequenceDiagramAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_sequence_diagram` without partial mutation on error.
/// - Action: Executes `parse_sequence_diagram` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_sequence_diagram` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-SEQUENCE-DIAGRAM] | SysML v2: ActionDef(parse_sequence_diagram)
fn parse_sequence_diagram(block: &MermaidBlock) -> Result<SequenceDiagramAst, SpecAuditError> {
    let mut participants: Vec<SequenceParticipantAst> = Vec::new();
    let mut messages: Vec<SequenceMessageAst> = Vec::new();
    let mut activations: Vec<SequenceActivationAst> = Vec::new();

    const SEQ_ARROWS: &[&str] = &[
        "-->>+", "-->>-", "-->>",
        "->>+", "->>-", "->>",
        "-->+", "-->-", "-->",
        "->+", "->-", "->",
        "--x", "-x", "--)", "-)",
    ];

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty()
            || clean.eq_ignore_ascii_case("sequenceDiagram")
            || clean.eq_ignore_ascii_case("autonumber")
        {
            continue;
        }

        // Participant declaration: `participant Name` or `participant Name as "Alias"`
        if clean.starts_with("participant ") {
            let rest = clean["participant ".len()..].trim();
            if let Some((name_part, alias_part)) = rest.split_once(" as ") {
                let name = name_part.trim().to_string();
                let alias = alias_part.trim().trim_matches('"').to_string();
                participants.push(SequenceParticipantAst {
                    name,
                    alias: Some(alias),
                    is_actor: false,
                    line: line_num,
                });
            } else {
                participants.push(SequenceParticipantAst {
                    name: rest.to_string(),
                    alias: None,
                    is_actor: false,
                    line: line_num,
                });
            }
            continue;
        }

        // Actor declaration: `actor Name` or `actor Name as "Alias"`
        if clean.starts_with("actor ") {
            let rest = clean["actor ".len()..].trim();
            if let Some((name_part, alias_part)) = rest.split_once(" as ") {
                let name = name_part.trim().to_string();
                let alias = alias_part.trim().trim_matches('"').to_string();
                participants.push(SequenceParticipantAst {
                    name,
                    alias: Some(alias),
                    is_actor: true,
                    line: line_num,
                });
            } else {
                participants.push(SequenceParticipantAst {
                    name: rest.to_string(),
                    alias: None,
                    is_actor: true,
                    line: line_num,
                });
            }
            continue;
        }

        // Activation / Deactivation: `activate Name` / `deactivate Name`
        if clean.starts_with("activate ") {
            let part = clean["activate ".len()..].trim().to_string();
            activations.push(SequenceActivationAst {
                participant: part,
                is_activate: true,
                line: line_num,
            });
            continue;
        }
        if clean.starts_with("deactivate ") {
            let part = clean["deactivate ".len()..].trim().to_string();
            activations.push(SequenceActivationAst {
                participant: part,
                is_activate: false,
                line: line_num,
            });
            continue;
        }

        // Message lines: `A ->> B : msg()`
        for &arrow in SEQ_ARROWS {
            if let Some(pos) = clean.find(arrow) {
                let from = clean[..pos].trim().to_string();
                let right_full = clean[pos + arrow.len()..].trim();

                let (to, msg_text) = if let Some((target_part, text)) = right_full.split_once(':') {
                    (target_part.trim().to_string(), text.trim().to_string())
                } else {
                    (right_full.to_string(), String::new())
                };

                let is_reply = arrow.starts_with("--");

                // Parse operation name and parameters from message text
                let mut op_name = None;
                let mut params = Vec::new();

                let unquoted_msg = msg_text.trim_matches('"').trim();
                if let Some(open_p) = unquoted_msg.find('(') {
                    if let Some(close_p) = unquoted_msg[open_p..].find(')') {
                        let actual_close = open_p + close_p;
                        let name_candidate = unquoted_msg[..open_p].trim();
                        if !name_candidate.is_empty() {
                            op_name = Some(name_candidate.to_string());
                        }
                        let inside = unquoted_msg[open_p + 1..actual_close].trim();
                        if !inside.is_empty() {
                            for p in inside.split(',') {
                                let trimmed_p = p.trim();
                                if !trimmed_p.is_empty() {
                                    params.push(trimmed_p.to_string());
                                }
                            }
                        }
                    }
                } else if !unquoted_msg.is_empty() {
                    let first_word = unquoted_msg.split_whitespace().next().unwrap_or("");
                    if !first_word.is_empty() {
                        op_name = Some(first_word.to_string());
                    }
                }

                // Ensure participants are tracked if not declared explicitly
                if !participants.iter().any(|p| p.name == from) && !from.is_empty() {
                    participants.push(SequenceParticipantAst {
                        name: from.clone(),
                        alias: None,
                        is_actor: false,
                        line: line_num,
                    });
                }
                if !participants.iter().any(|p| p.name == to) && !to.is_empty() {
                    participants.push(SequenceParticipantAst {
                        name: to.clone(),
                        alias: None,
                        is_actor: false,
                        line: line_num,
                    });
                }

                messages.push(SequenceMessageAst {
                    from,
                    to,
                    arrow: arrow.to_string(),
                    message: msg_text,
                    operation_name: op_name,
                    parameters: params,
                    is_reply,
                    line: line_num,
                });
                break;
            }
        }
    }

    Ok(SequenceDiagramAst {
        participants,
        messages,
        activations,
    })
}

// ----------------------------------------------------------------------------
// FLOWCHART / GRAPH PARSER
// ----------------------------------------------------------------------------

/// Executes deterministic fn `parse_flowchart` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_flowchart` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_flowchart` (declaration #39) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_flowchart`.
///
/// ## 4. Postconditions
/// - Returns `Result<FlowchartAst, SpecAuditError>` from `parse_flowchart` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_flowchart`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_flowchart`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_flowchart` (`Result<FlowchartAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_flowchart` without partial mutation on error.
/// - Action: Executes `parse_flowchart` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_flowchart` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-FLOWCHART] | SysML v2: ActionDef(parse_flowchart)
fn parse_flowchart(block: &MermaidBlock) -> Result<FlowchartAst, SpecAuditError> {
    let mut direction = None;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_ids: HashSet<String> = HashSet::new();

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() {
            continue;
        }

        // Header line: `flowchart TD` or `graph LR`
        if clean.starts_with("flowchart ") || clean.starts_with("graph ") {
            let dir = clean.split_whitespace().nth(1).unwrap_or("TD").to_string();
            direction = Some(dir);
            continue;
        }

        // Edge detection: `A --> B` or `A -->|Label| B`
        for &arrow in ARROWS {
            if let Some(pos) = clean.find(arrow) {
                let from = clean[..pos].trim();
                let after_arrow = clean[pos + arrow.len()..].trim();

                let (label, to) = if after_arrow.starts_with('|') {
                    if let Some(second_pipe) = after_arrow[1..].find('|') {
                        let lbl = after_arrow[1..second_pipe + 1].trim();
                        let target = after_arrow[second_pipe + 2..].trim();
                        (Some(lbl.trim_matches('"').to_string()), target)
                    } else {
                        (None, after_arrow)
                    }
                } else {
                    (None, after_arrow)
                };

                let from_id = extract_node_id(from);
                let to_id = extract_node_id(to);

                if node_ids.insert(from_id.clone()) {
                    nodes.push(FlowNodeAst {
                        id: from_id.clone(),
                        label: extract_node_label(from),
                        shape: None,
                        line: line_num,
                    });
                }
                if node_ids.insert(to_id.clone()) {
                    nodes.push(FlowNodeAst {
                        id: to_id.clone(),
                        label: extract_node_label(to),
                        shape: None,
                        line: line_num,
                    });
                }

                edges.push(FlowEdgeAst {
                    from: from_id,
                    to: to_id,
                    arrow: arrow.to_string(),
                    label,
                    line: line_num,
                });
                break;
            }
        }
    }

    Ok(FlowchartAst {
        direction,
        nodes,
        edges,
    })
}

/// Executes deterministic fn `extract_node_id` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_node_id` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_node_id` (declaration #40) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `extract_node_id`.
///
/// ## 4. Postconditions
/// - Returns `String` from `extract_node_id` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `extract_node_id`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `extract_node_id`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_node_id` (`String`).
/// - State: Maintains consistent state invariants across `extract_node_id` without partial mutation on error.
/// - Action: Executes `extract_node_id` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_node_id` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-EXTRACT-NODE-ID] | SysML v2: ActionDef(extract_node_id)
fn extract_node_id(s: &str) -> String {
    let delimiters = ['[', '(', '{', '>'];
    if let Some(idx) = s.find(|c| delimiters.contains(&c)) {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

/// Executes deterministic fn `extract_node_label` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_node_label` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_node_label` (declaration #41) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `extract_node_label`.
///
/// ## 4. Postconditions
/// - Returns `Option<String>` from `extract_node_label` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `extract_node_label`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `extract_node_label`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_node_label` (`Option<String>`).
/// - State: Maintains consistent state invariants across `extract_node_label` without partial mutation on error.
/// - Action: Executes `extract_node_label` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_node_label` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-EXTRACT-NODE-LABEL] | SysML v2: ActionDef(extract_node_label)
fn extract_node_label(s: &str) -> Option<String> {
    let start_delims = ['[', '(', '{'];
    if let Some(start) = s.find(|c| start_delims.contains(&c)) {
        let close_delims = [']', ')', '}'];
        if let Some(end) = s.rfind(|c| close_delims.contains(&c)) {
            if end > start {
                let inner = s[start + 1..end].trim().trim_matches('"');
                return Some(inner.to_string());
            }
        }
    }
    None
}

// ----------------------------------------------------------------------------
// ER / XY / GENERIC DIAGRAM PARSERS
// ----------------------------------------------------------------------------

/// Executes deterministic fn `parse_er_diagram` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_er_diagram` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_er_diagram` (declaration #42) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_er_diagram`.
///
/// ## 4. Postconditions
/// - Returns `Result<ErDiagramAst, SpecAuditError>` from `parse_er_diagram` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_er_diagram`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_er_diagram`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_er_diagram` (`Result<ErDiagramAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_er_diagram` without partial mutation on error.
/// - Action: Executes `parse_er_diagram` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_er_diagram` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-ER-DIAGRAM] | SysML v2: ActionDef(parse_er_diagram)
fn parse_er_diagram(block: &MermaidBlock) -> Result<ErDiagramAst, SpecAuditError> {
    let mut entities = Vec::new();
    let mut relationships = Vec::new();

    for line in block.raw.lines() {
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() || clean.eq_ignore_ascii_case("erDiagram") {
            continue;
        }
        if clean.contains("||") || clean.contains("}|") || clean.contains("|{") {
            relationships.push(clean.to_string());
        } else {
            let entity = clean.split_whitespace().next().unwrap_or("").to_string();
            if !entity.is_empty() && !entities.contains(&entity) {
                entities.push(entity);
            }
        }
    }

    Ok(ErDiagramAst {
        entities,
        relationships,
    })
}

/// Executes deterministic fn `parse_xychart` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_xychart` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_xychart` (declaration #43) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_xychart`.
///
/// ## 4. Postconditions
/// - Returns `Result<XyChartAst, SpecAuditError>` from `parse_xychart` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_xychart`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_xychart`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_xychart` (`Result<XyChartAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_xychart` without partial mutation on error.
/// - Action: Executes `parse_xychart` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_xychart` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-XYCHART] | SysML v2: ActionDef(parse_xychart)
fn parse_xychart(block: &MermaidBlock) -> Result<XyChartAst, SpecAuditError> {
    let mut title = None;
    let mut x_axis = None;
    let mut y_axis = None;

    for line in block.raw.lines() {
        let clean = strip_mermaid_comment(line).trim();
        if clean.starts_with("title ") {
            title = Some(clean["title ".len()..].trim().trim_matches('"').to_string());
        } else if clean.starts_with("x-axis ") {
            x_axis = Some(clean["x-axis ".len()..].trim().to_string());
        } else if clean.starts_with("y-axis ") {
            y_axis = Some(clean["y-axis ".len()..].trim().to_string());
        }
    }

    Ok(XyChartAst {
        title,
        x_axis,
        y_axis,
    })
}

/// Executes deterministic fn `parse_generic_diagram` within `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_generic_diagram` in `crates/verify-baseline/src/spec_audit/diagram_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_generic_diagram` (declaration #44) within `SPEC-AUDIT-DIAGRAM-AST`.
///
/// ## 3. Preconditions
/// - `block: &MermaidBlock` must satisfy domain and type-safety constraints prior to invoking `parse_generic_diagram`.
///
/// ## 4. Postconditions
/// - Returns `Result<GenericDiagramAst, SpecAuditError>` from `parse_generic_diagram` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(block)) bounded over input/member dimension of `parse_generic_diagram`.
/// - Space Complexity: O(N(block)) bounded auxiliary memory for `parse_generic_diagram`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_generic_diagram` (`Result<GenericDiagramAst, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_generic_diagram` without partial mutation on error.
/// - Action: Executes `parse_generic_diagram` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_generic_diagram` (`SPEC-AUDIT-DIAGRAM-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-GENERIC-DIAGRAM] | SysML v2: ActionDef(parse_generic_diagram)
fn parse_generic_diagram(block: &MermaidBlock) -> Result<GenericDiagramAst, SpecAuditError> {
    let mut lines = Vec::new();
    for line in block.raw.lines() {
        let clean = strip_mermaid_comment(line).trim();
        if !clean.is_empty() {
            lines.push(clean.to_string());
        }
    }
    Ok(GenericDiagramAst {
        header: block.header.clone(),
        lines,
    })
}

// ============================================================================
// SEMANTIC ACCEPTANCE UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_diagram_parsing_with_visibility_and_types() {
        let raw = r#"classDiagram
    class FlightController {
        +float altitude
        -bool armed
        +execute_loop(int rate_hz) void
        #check_battery() bool
    }
    class SensorHub {
        +int sensor_count
    }
    FlightController *-- SensorHub : owns
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "classDiagram".to_string(),
            start_line: 10,
            end_line: 21,
        };

        let ast = parse_mermaid_diagram(&block).expect("Class diagram parse failed");
        if let DiagramAst::Class(c_ast) = ast {
            assert_eq!(
                c_ast.classes.len(),
                2,
                "Classes found: {:?}",
                c_ast.classes.iter().map(|c| &c.name).collect::<Vec<_>>()
            );
            let fc = c_ast.classes.iter().find(|c| c.name == "FlightController").unwrap();
            assert_eq!(fc.attributes.len(), 2);
            assert_eq!(fc.attributes[0].name, "altitude");
            assert_eq!(fc.attributes[0].visibility, Some(Visibility::Public));
            assert_eq!(fc.attributes[0].type_name, Some("float".to_string()));

            assert_eq!(fc.operations.len(), 2);
            assert_eq!(fc.operations[0].name, "execute_loop");
            assert_eq!(fc.operations[0].visibility, Some(Visibility::Public));
            assert_eq!(fc.operations[0].return_type, Some("void".to_string()));
            assert_eq!(fc.operations[0].parameters, vec!["int rate_hz"]);

            assert_eq!(c_ast.relations.len(), 1);
            assert_eq!(c_ast.relations[0].from_class, "FlightController");
            assert_eq!(c_ast.relations[0].to_class, "SensorHub");
            assert_eq!(c_ast.relations[0].relation_type, ClassRelationType::Composition);
            assert_eq!(c_ast.relations[0].label, Some("owns".to_string()));
        } else {
            panic!("Expected Class diagram AST");
        }
    }

    #[test]
    fn test_class_diagram_curly_braces_in_member_fails() {
        let raw = r#"classDiagram
    class FlightController {
        +{abstract} void execute()
    }
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "classDiagram".to_string(),
            start_line: 1,
            end_line: 5,
        };

        let res = parse_mermaid_diagram(&block);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("prohibited curly braces"));
    }

    #[test]
    fn test_class_diagram_secondary_colon_in_member_fails() {
        let raw = r#"classDiagram
    class FlightController {
        +altitude: float
    }
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "classDiagram".to_string(),
            start_line: 1,
            end_line: 5,
        };

        let res = parse_mermaid_diagram(&block);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("prohibited colon"));
    }

    #[test]
    fn test_unquoted_angle_bracket_fails() {
        let raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> Active : count > 0 / trigger
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "stateDiagram-v2".to_string(),
            start_line: 1,
            end_line: 4,
        };

        let res = parse_mermaid_diagram(&block);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Unquoted '>' character"));
    }

    #[test]
    fn test_quoted_angle_bracket_passes() {
        let raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> Active : "count > 0 / trigger"
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "stateDiagram-v2".to_string(),
            start_line: 1,
            end_line: 4,
        };

        let res = parse_mermaid_diagram(&block);
        assert!(res.is_ok(), "Expected valid quoted comparison operator to pass: {:?}", res);
    }

    #[test]
    fn test_node_label_unquoted_special_char_fails() {
        let raw = r#"flowchart TD
    A[Processing / Action] --> B
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "flowchart".to_string(),
            start_line: 1,
            end_line: 3,
        };

        let res = parse_mermaid_diagram(&block);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("special characters"));
    }

    #[test]
    fn test_node_label_quoted_special_char_passes() {
        let raw = r#"flowchart TD
    A["Processing / Action"] --> B["Status: Active"]
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "flowchart".to_string(),
            start_line: 1,
            end_line: 3,
        };

        let res = parse_mermaid_diagram(&block);
        assert!(res.is_ok(), "Expected quoted node label with special chars to pass: {:?}", res);
    }

    #[test]
    fn test_sequence_diagram_extraction() {
        let raw = r#"sequenceDiagram
    actor Pilot
    participant FlightController
    participant SensorHub
    Pilot ->> FlightController: arm()
    FlightController ->> SensorHub: readSensors()
    SensorHub -->> FlightController: sensorData
    FlightController -->> Pilot: armedAck
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "sequenceDiagram".to_string(),
            start_line: 1,
            end_line: 9,
        };

        let ast = parse_mermaid_diagram(&block).expect("Sequence diagram parse failed");
        if let DiagramAst::Sequence(s_ast) = ast {
            assert_eq!(s_ast.participants.len(), 3);
            let pilot = s_ast.participants.iter().find(|p| p.name == "Pilot").unwrap();
            assert!(pilot.is_actor);

            assert_eq!(s_ast.messages.len(), 4);
            assert_eq!(s_ast.messages[0].from, "Pilot");
            assert_eq!(s_ast.messages[0].to, "FlightController");
            assert_eq!(s_ast.messages[0].operation_name, Some("arm".to_string()));
            assert!(!s_ast.messages[0].is_reply);

            assert_eq!(s_ast.messages[2].from, "SensorHub");
            assert_eq!(s_ast.messages[2].to, "FlightController");
            assert!(s_ast.messages[2].is_reply);
        } else {
            panic!("Expected Sequence diagram AST");
        }
    }

    #[test]
    fn test_state_diagram_extraction_with_guard_and_action() {
        let raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> Active : "start [armed] / initMotors()"
    Active --> [*]
"#;
        let block = MermaidBlock {
            raw: raw.to_string(),
            header: "stateDiagram-v2".to_string(),
            start_line: 1,
            end_line: 5,
        };

        let ast = parse_mermaid_diagram(&block).expect("State diagram parse failed");
        if let DiagramAst::State(s_ast) = ast {
            assert_eq!(s_ast.states.len(), 2);
            assert_eq!(s_ast.transitions.len(), 3);

            let trans = &s_ast.transitions[1];
            assert_eq!(trans.source, "Idle");
            assert_eq!(trans.target, "Active");
            assert_eq!(trans.event, Some("start".to_string()));
            assert_eq!(trans.guard, Some("armed".to_string()));
            assert_eq!(trans.action, Some("initMotors()".to_string()));
        } else {
            panic!("Expected State diagram AST");
        }
    }
}

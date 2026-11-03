//! Streaming zero-copy line and token parser for Mermaid diagrams conforming to DO-178C Level A and ISO 26262 ASIL D.
//! Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]

use crate::spec_audit::diagram_class_parser::parse_class_diagram;
pub use crate::spec_audit::diagram_invariants::validate_mermaid_invariants;
use crate::spec_audit::diagram_parsers::{
    parse_er_diagram, parse_flowchart, parse_generic_diagram, parse_sequence_diagram,
    parse_state_diagram, parse_xychart,
};
use crate::spec_audit::markdown_ast::MermaidBlock;
use crate::spec_audit::SpecAuditError;

/// Supported diagram types parsed from diagram headers.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-DIAGRAMTYPE]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagramType {
    /// Class diagram (`classDiagram`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Class,
    /// State diagram (`stateDiagram`, `stateDiagram-v2`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    State,
    /// Sequence diagram (`sequenceDiagram`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Sequence,
    /// Flowchart (`flowchart TD`, `flowchart LR`, etc.).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Flowchart,
    /// Graph (`graph TD`, `graph LR`, etc.).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Graph,
    /// Entity-Relationship diagram (`erDiagram`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Er,
    /// XY Chart (`xychart-beta`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    XyChart,
    /// Other or generic diagram type.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Other,
}

/// In-memory AST representation of any parsed Mermaid diagram.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-DIAGRAMAST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramAst {
    /// Parsed class diagram AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Class(ClassDiagramAst),
    /// Parsed state diagram AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    State(StateDiagramAst),
    /// Parsed sequence diagram AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Sequence(SequenceDiagramAst),
    /// Parsed flowchart diagram AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Flowchart(FlowchartAst),
    /// Parsed ER diagram AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Er(ErDiagramAst),
    /// Parsed XY chart AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    XyChart(XyChartAst),
    /// Generic diagram AST.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Other(GenericDiagramAst),
}

/// UML member visibility specifier.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VISIBILITY]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Public visibility (`+`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Public,
    /// Private visibility (`-`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Private,
    /// Protected visibility (`#`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Protected,
    /// Package/Internal visibility (`~`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Package,
}

impl Visibility {
    /// Parses a visibility symbol (`+`, `-`, `#`, `~`) into [`Visibility`].
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FROM-CHAR]
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

/// Attribute declaration inside a `classDiagram` class block.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASS-ATTRIBUTE-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAttributeAst {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_name: Option<String>,
    pub raw: String,
    pub line: usize,
}

/// Operation declaration inside a `classDiagram` class block.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASS-OPERATION-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassOperationAst {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub raw: String,
    pub line: usize,
}

/// UML class relation kind.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASS-RELATION-TYPE]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassRelationType {
    /// Generalization / Inheritance (`<|--` or `--|>`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Inheritance,
    /// Composition (`*--` or `--*`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Composition,
    /// Aggregation (`o--` or `--o`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Aggregation,
    /// Association (`-->` or `<--` or `--`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Association,
    /// Realization (`<|..` or `..|>`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Realization,
    /// Dependency (`..>` or `<..` or `..`).
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Dependency,
    /// Other relation symbol.
    /// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]
    Other(String),
}

/// Relation edge between two classes in a `classDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASS-RELATION-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRelationAst {
    pub from_class: String,
    pub to_class: String,
    pub relation_type: ClassRelationType,
    pub raw_symbol: String,
    pub label: Option<String>,
    pub line: usize,
}

/// Parsed class definition node in a `classDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASS-DEF-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassDefAst {
    pub name: String,
    pub annotations: Vec<String>,
    pub attributes: Vec<ClassAttributeAst>,
    pub operations: Vec<ClassOperationAst>,
    pub line: usize,
}

/// Parsed `classDiagram` AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CLASS-DIAGRAM-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassDiagramAst {
    pub classes: Vec<ClassDefAst>,
    pub relations: Vec<ClassRelationAst>,
}

/// State definition in a state diagram.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATE-DEF-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StateDefAst {
    pub id: String,
    pub label: Option<String>,
    pub is_composite: bool,
    pub sub_states: Vec<StateDefAst>,
    pub line: usize,
}

/// Transition edge in a `stateDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATE-TRANSITION-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionAst {
    pub source: String,
    pub target: String,
    pub event: Option<String>,
    pub guard: Option<String>,
    pub action: Option<String>,
    pub raw_label: Option<String>,
    pub line: usize,
}

/// Note annotation in a `stateDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATE-NOTE-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateNoteAst {
    pub target_state: Option<String>,
    pub text: String,
    pub line: usize,
}

/// Parsed `stateDiagram` AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STATE-DIAGRAM-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StateDiagramAst {
    pub states: Vec<StateDefAst>,
    pub transitions: Vec<StateTransitionAst>,
    pub notes: Vec<StateNoteAst>,
}

/// Participant or actor lifeline in a `sequenceDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCE-PARTICIPANT-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceParticipantAst {
    pub name: String,
    pub alias: Option<String>,
    pub is_actor: bool,
    pub line: usize,
}

/// Message exchange in a `sequenceDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCE-MESSAGE-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceMessageAst {
    pub from: String,
    pub to: String,
    pub arrow: String,
    pub message: String,
    pub operation_name: Option<String>,
    pub parameters: Vec<String>,
    pub is_reply: bool,
    pub line: usize,
}

/// Lifeline activation/deactivation directive in a `sequenceDiagram`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCE-ACTIVATION-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceActivationAst {
    pub participant: String,
    pub is_activate: bool,
    pub line: usize,
}

/// Parsed `sequenceDiagram` AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-SEQUENCE-DIAGRAM-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SequenceDiagramAst {
    pub participants: Vec<SequenceParticipantAst>,
    pub messages: Vec<SequenceMessageAst>,
    pub activations: Vec<SequenceActivationAst>,
}

/// Flowchart node AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FLOW-NODE-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowNodeAst {
    pub id: String,
    pub label: Option<String>,
    pub shape: Option<String>,
    pub line: usize,
}

/// Flowchart directed edge AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FLOW-EDGE-AST]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEdgeAst {
    pub from: String,
    pub to: String,
    pub arrow: String,
    pub label: Option<String>,
    pub line: usize,
}

/// Parsed `flowchart` / `graph` AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-FLOWCHART-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FlowchartAst {
    pub direction: Option<String>,
    pub nodes: Vec<FlowNodeAst>,
    pub edges: Vec<FlowEdgeAst>,
}

/// Parsed `erDiagram` AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-ER-DIAGRAM-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ErDiagramAst {
    pub entities: Vec<String>,
    pub relationships: Vec<String>,
}

/// Parsed `xychart-beta` AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-XY-CHART-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XyChartAst {
    pub title: Option<String>,
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
}

/// Parsed generic Mermaid diagram AST.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-GENERIC-DIAGRAM-AST]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenericDiagramAst {
    pub header: String,
    pub lines: Vec<String>,
}

/// Classifies a Mermaid header string into [`DiagramType`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-DETECT-DIAGRAM-TYPE]
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

/// Validates and parses a [`MermaidBlock`] into a typed [`DiagramAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-MERMAID-DIAGRAM]
pub fn parse_mermaid_diagram(block: &MermaidBlock) -> Result<DiagramAst, SpecAuditError> {
    if let Err(errors) = validate_mermaid_invariants(block) {
        if let Some(first) = errors.into_iter().next() {
            return Err(first);
        }
    }

    let header_type = detect_diagram_type(&block.header);
    match header_type {
        DiagramType::Class => Ok(DiagramAst::Class(parse_class_diagram(block)?)),
        DiagramType::State => Ok(DiagramAst::State(parse_state_diagram(block)?)),
        DiagramType::Sequence => Ok(DiagramAst::Sequence(parse_sequence_diagram(block)?)),
        DiagramType::Flowchart | DiagramType::Graph => {
            Ok(DiagramAst::Flowchart(parse_flowchart(block)?))
        }
        DiagramType::Er => Ok(DiagramAst::Er(parse_er_diagram(block)?)),
        DiagramType::XyChart => Ok(DiagramAst::XyChart(parse_xychart(block)?)),
        DiagramType::Other => Ok(DiagramAst::Other(parse_generic_diagram(block)?)),
    }
}

#[cfg(test)]
#[path = "diagram_ast_tests.rs"]
mod tests;

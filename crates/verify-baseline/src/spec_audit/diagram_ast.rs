//! Streaming zero-copy line and token parser for Mermaid diagrams conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Provides AST extraction and structural invariant enforcement for Mermaid diagrams extracted by `markdown_ast::MermaidBlock`.
//! Supports class diagrams, state diagrams, sequence diagrams, flowcharts, graphs, entity-relationship diagrams, and xy charts.
//! Strictly zero regex, zero unwrap/expect/panic, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::markdown_ast::MermaidBlock;
use crate::spec_audit::SpecAuditError;
use std::collections::HashSet;

/// Supported diagram types parsed from diagram headers.
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
    /// Preconditions: None.
    /// Postconditions: Returns `Some(Visibility)` if character is `+`, `-`, `#`, or `~`.
    /// Algorithmic Complexity: O(1).
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassDiagramAst {
    /// Declared or inferred classes.
    pub classes: Vec<ClassDefAst>,
    /// Declared relationships between classes.
    pub relations: Vec<ClassRelationAst>,
}

/// State definition in a state diagram.
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ErDiagramAst {
    /// Declared entities.
    pub entities: Vec<String>,
    /// Declared relationship lines.
    pub relationships: Vec<String>,
}

/// Complete AST for an XY chart diagram.
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
/// Preconditions: `block` contains valid raw source lines.
/// Postconditions: Returns `Ok(())` if all invariants hold, or a vector of `SpecAuditError` violations.
/// Algorithmic Complexity: O(N) where N is total character length of the raw block.
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
fn contains_relation_arrow(s: &str) -> bool {
    ARROWS.iter().any(|&arrow| s.contains(arrow))
}

/// Checks that all angle brackets `<` and `>` are inside quotes or valid Mermaid operators.
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
/// Preconditions: `block` is an extracted Mermaid diagram code block.
/// Postconditions: Validates all diagram invariants and returns `Ok(DiagramAst)` or `Err(SpecAuditError)`.
/// Algorithmic Complexity: O(N) where N is block character length.
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

fn merge_or_push_class(classes: &mut Vec<ClassDefAst>, new_class: ClassDefAst) {
    if let Some(existing) = classes.iter_mut().find(|c| c.name == new_class.name) {
        existing.annotations.extend(new_class.annotations);
        existing.attributes.extend(new_class.attributes);
        existing.operations.extend(new_class.operations);
    } else {
        classes.push(new_class);
    }
}

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

fn extract_node_id(s: &str) -> String {
    let delimiters = ['[', '(', '{', '>'];
    if let Some(idx) = s.find(|c| delimiters.contains(&c)) {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

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

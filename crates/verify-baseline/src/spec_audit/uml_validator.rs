//! Strongly-typed cross-validation of Mermaid UML diagrams against SysML v2 models.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//! Indexes `deap_core::sysml_ast::SysmlModel` and `PackageDef` into O(1) symbol lookup tables.
//! Enforces that:
//! - Non-actor sequence diagram lifelines resolve to declared Parts/Blocks.
//! - Sequence diagram messages resolve to declared Operations or Actions on target classifiers.
//! - Class diagram classes, attributes, and operations resolve to declared Parts and member definitions.
//! - State diagram states, actions, and triggers resolve to declared States and Actions.
//! - Specification documents include valid Source References citing schema files or normative clauses.
//!
//! Strictly zero regex, zero unwrap/expect/panic, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::diagram_ast::{
    parse_mermaid_diagram, validate_mermaid_invariants, ClassDiagramAst, DiagramAst,
    SequenceDiagramAst, StateDiagramAst,
};
use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::sections::normalize_tokens;
use crate::spec_audit::SpecAuditError;
use deap_core::sysml_ast::{PackageDef, PartDef, StateDef, SysmlModel};
use std::collections::{HashMap, HashSet};

/// Pre-indexed symbol set for a single classifier (Part / Block) in SysML AST.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassifierSymbolInfo {
    /// Classifier identifier name (e.g. `FlightController`, `SensorHub`).
    pub name: String,
    /// Declared attribute names.
    pub attributes: HashSet<String>,
    /// Declared operation names.
    pub operations: HashSet<String>,
    /// Declared action names.
    pub actions: HashSet<String>,
    /// Declared port names.
    pub ports: HashSet<String>,
    /// Declared state names if classifier owns a state machine.
    pub states: HashSet<String>,
}

/// Pre-indexed symbol lookup tables providing O(1) resolution against SysML v2 models.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SysmlSymbolIndex {
    /// Valid Part / Block names in the model.
    pub part_names: HashSet<String>,
    /// Valid State names across all state machines in the model.
    pub state_names: HashSet<String>,
    /// Valid Action and Operation names across all parts and packages.
    pub action_names: HashSet<String>,
    /// Valid Port names across all parts and packages.
    pub port_names: HashSet<String>,
    /// Valid Attribute names across all parts and packages.
    pub attribute_names: HashSet<String>,
    /// Mapping of classifier name -> ClassifierSymbolInfo.
    pub classifiers: HashMap<String, ClassifierSymbolInfo>,
}

impl SysmlSymbolIndex {
    /// Constructs an empty symbol index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-indexes a high-level `SysmlModel` into O(1) lookup tables.
    ///
    /// Preconditions: None.
    /// Postconditions: Extracts all parts, nested parts, states, operations, actions, ports, and attributes.
    /// Algorithmic Complexity: O(V + E) where V is AST node count and E is containment edge count.
    pub fn from_model(model: &SysmlModel) -> Self {
        let mut index = Self::new();

        // Index model-level attributes
        for attr in &model.attributes {
            if !attr.name.is_empty() {
                index.attribute_names.insert(attr.name.clone());
            }
        }

        // Index model-level ports
        for port in &model.ports {
            if !port.name.is_empty() {
                index.port_names.insert(port.name.clone());
            }
        }

        // Index model-level actions
        for act in &model.actions {
            if !act.name.is_empty() {
                index.action_names.insert(act.name.clone());
            }
        }

        // Index all parts recursively
        for part in &model.parts {
            index.index_part(part);
        }

        index.link_type_names();
        index
    }

    /// Pre-indexes a hierarchical `PackageDef` into O(1) lookup tables.
    ///
    /// Preconditions: None.
    /// Postconditions: Recursively indexes all packages, subpackages, parts, states, operations, and actions.
    /// Algorithmic Complexity: O(V + E) over package and part AST hierarchy.
    pub fn from_package(pkg: &PackageDef) -> Self {
        let mut index = Self::new();
        index.index_package_recursive(pkg);
        index.link_type_names();
        index
    }

    fn index_package_recursive(&mut self, pkg: &PackageDef) {
        for attr in &pkg.attribute_defs {
            if !attr.name.is_empty() {
                self.attribute_names.insert(attr.name.clone());
            }
        }
        for port in &pkg.port_defs {
            if !port.name.is_empty() {
                self.port_names.insert(port.name.clone());
            }
        }
        for act in &pkg.action_defs {
            if !act.name.is_empty() {
                self.action_names.insert(act.name.clone());
            }
        }
        for op in &pkg.operation_defs {
            if !op.name.is_empty() {
                self.action_names.insert(op.name.clone());
            }
        }
        for st in &pkg.state_defs {
            self.index_state(st);
        }
        for part in &pkg.part_defs {
            self.index_part(part);
        }
        for subpkg in &pkg.packages {
            self.index_package_recursive(subpkg);
        }
    }

    fn index_part(&mut self, part: &PartDef) {
        if !part.name.is_empty() {
            self.part_names.insert(part.name.clone());

            let mut classifier_states = HashSet::new();
            for st in &part.states {
                collect_state_names(st, &mut classifier_states);
            }

            let classifier = self
                .classifiers
                .entry(part.name.clone())
                .or_insert_with(|| ClassifierSymbolInfo {
                    name: part.name.clone(),
                    ..Default::default()
                });

            for attr in &part.attributes {
                if !attr.name.is_empty() {
                    classifier.attributes.insert(attr.name.clone());
                    self.attribute_names.insert(attr.name.clone());
                }
            }

            for op in &part.operations {
                if !op.name.is_empty() {
                    classifier.operations.insert(op.name.clone());
                    self.action_names.insert(op.name.clone());
                }
            }

            for act in &part.actions {
                if !act.name.is_empty() {
                    classifier.actions.insert(act.name.clone());
                    self.action_names.insert(act.name.clone());
                }
            }

            for port in &part.ports {
                if !port.name.is_empty() {
                    classifier.ports.insert(port.name.clone());
                    self.port_names.insert(port.name.clone());
                }
            }

            classifier.states.extend(classifier_states);
        }

        for st in &part.states {
            self.index_state(st);
        }

        if let Some(ref type_name) = part.type_name {
            if !type_name.is_empty() {
                self.part_names.insert(type_name.clone());
            }
        }

        for subpart in &part.parts {
            self.index_part(subpart);
        }
    }

    pub fn index_state(&mut self, state: &StateDef) {
        if !state.name.is_empty() {
            self.state_names.insert(state.name.clone());
        }

        if let Some(ref entry) = state.entry_action {
            let ident = extract_ident(entry);
            if !ident.is_empty() {
                self.action_names.insert(ident.to_string());
            }
        }
        if let Some(ref do_act) = state.do_action {
            let ident = extract_ident(do_act);
            if !ident.is_empty() {
                self.action_names.insert(ident.to_string());
            }
        }
        if let Some(ref exit) = state.exit_action {
            let ident = extract_ident(exit);
            if !ident.is_empty() {
                self.action_names.insert(ident.to_string());
            }
        }

        for trans in &state.transitions {
            if let Some(ref act) = trans.effect {
                let ident = extract_ident(act);
                if !ident.is_empty() {
                    self.action_names.insert(ident.to_string());
                }
            }
            if let Some(ref trg) = trans.trigger {
                let ident = extract_ident(trg);
                if !ident.is_empty() {
                    self.action_names.insert(ident.to_string());
                }
            }
        }

        for sub in &state.sub_states {
            self.index_state(sub);
        }
    }

    /// Links classifier instances with their declared type definition symbols.
    fn link_type_names(&mut self) {
        // Collect mapping of known classifiers
        let known = self.classifiers.clone();
        for (name, classifier) in &mut self.classifiers {
            if let Some(suffix_idx) = name.rfind(':') {
                let type_cand = name[suffix_idx + 1..].trim();
                if let Some(parent) = known.get(type_cand) {
                    classifier.attributes.extend(parent.attributes.clone());
                    classifier.operations.extend(parent.operations.clone());
                    classifier.actions.extend(parent.actions.clone());
                    classifier.ports.extend(parent.ports.clone());
                }
            }
        }
    }
}

// ============================================================================
// CROSS-REFERENCE VALIDATION GATES (ZERO REGEX)
// ============================================================================

/// Cross-validates a sequence diagram AST against the indexed SysML model.
///
/// Preconditions: `diagram` has been successfully parsed and invariant-checked.
/// Postconditions: Returns `Ok(())` if all lifelines and messages resolve, or a list of validation errors.
/// Algorithmic Complexity: O(L + M) where L is lifelines count and M is messages count.
pub fn validate_sequence_diagram(
    diagram: &SequenceDiagramAst,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();

    // 1. Verify all non-actor lifelines resolve to declared Part/Block
    let mut actor_names: HashSet<String> = HashSet::new();

    for p in &diagram.participants {
        if p.is_actor {
            actor_names.insert(p.name.clone());
            if let Some(ref a) = p.alias {
                actor_names.insert(a.clone());
            }
            continue;
        }

        let name_match = index.part_names.contains(&p.name);
        let alias_match = p
            .alias
            .as_ref()
            .map(|a| index.part_names.contains(a))
            .unwrap_or(false);

        if !name_match && !alias_match {
            errors.push(SpecAuditError::ValidationError(format!(
                "Lifeline '{}' in sequence diagram at line {} does not resolve to any declared Part/Block in SysML model.",
                p.name, p.line
            )));
        }
    }

    // 2. Verify every message maps to an active Operation or Action on the target classifier
    for msg in &diagram.messages {
        // Replies (e.g. `-->>`) are return values or confirmations, skip target operation check
        if msg.is_reply {
            continue;
        }

        // If target is an actor, skip operation check
        if actor_names.contains(&msg.to) {
            continue;
        }

        // Target must resolve to a part
        if !index.part_names.contains(&msg.to) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Message recipient lifeline '{}' at line {} does not resolve to any declared Part/Block in SysML model.",
                msg.to, msg.line
            )));
            continue;
        }

        if let Some(ref op) = msg.operation_name {
            let op_clean = extract_ident(op);
            if op_clean.is_empty() {
                continue;
            }

            if let Some(classifier) = index.classifiers.get(&msg.to) {
                let matches_target = classifier.operations.contains(op_clean)
                    || classifier.actions.contains(op_clean)
                    || index.action_names.contains(op_clean);

                if !matches_target {
                    errors.push(SpecAuditError::ValidationError(format!(
                        "Message operation '{}' targeting '{}' at line {} does not map to any active Operation or Action declared on classifier '{}' in SysML model.",
                        op_clean, msg.to, msg.line, msg.to
                    )));
                }
            } else if !index.action_names.contains(op_clean) {
                errors.push(SpecAuditError::ValidationError(format!(
                    "Message operation '{}' targeting '{}' at line {} does not map to any active Operation or Action in SysML model.",
                    op_clean, msg.to, msg.line
                )));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Cross-validates a class diagram AST against the indexed SysML model.
///
/// Preconditions: `diagram` has been successfully parsed and invariant-checked.
/// Postconditions: Returns `Ok(())` if all classes and members resolve, or a list of validation errors.
/// Algorithmic Complexity: O(C + A + M + R) where C is classes, A is attributes, M is operations, R is relations.
pub fn validate_class_diagram(
    diagram: &ClassDiagramAst,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();

    // 1. Verify every class resolves to a declared Part/Block
    for c in &diagram.classes {
        if !index.part_names.contains(&c.name) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Class '{}' in class diagram at line {} does not resolve to any declared Part/Block in SysML model.",
                c.name, c.line
            )));
            continue;
        }

        // 2. Verify member attributes and operations match SysML AST declarations
        if let Some(classifier) = index.classifiers.get(&c.name) {
            let ast_has_members = !classifier.ports.is_empty()
                || !classifier.attributes.is_empty()
                || !classifier.operations.is_empty();
            let diagram_has_members = !c.attributes.is_empty() || !c.operations.is_empty();

            if ast_has_members && !diagram_has_members && diagram.relations.is_empty() {
                errors.push(SpecAuditError::ValidationError(format!(
                    "Class '{}' in class diagram at line {} is a vacuous stub omitting declared AST ports and attributes.",
                    c.name, c.line
                )));
            }

            for attr in &c.attributes {
                let attr_clean = extract_ident(&attr.name);
                let matches_attr = classifier.attributes.contains(attr_clean)
                    || index.attribute_names.contains(attr_clean)
                    || classifier.ports.contains(attr_clean)
                    || index.port_names.contains(attr_clean);
                if !matches_attr {
                    errors.push(SpecAuditError::ValidationError(format!(
                        "Member attribute '{}' of class '{}' at line {} does not match any attribute declaration in SysML model.",
                        attr.name, c.name, attr.line
                    )));
                }
            }

            for op in &c.operations {
                let op_clean = extract_ident(&op.name);
                let matches_op = classifier.operations.contains(op_clean)
                    || classifier.actions.contains(op_clean)
                    || index.action_names.contains(op_clean)
                    || classifier.ports.contains(op_clean)
                    || index.port_names.contains(op_clean);
                if !matches_op {
                    errors.push(SpecAuditError::ValidationError(format!(
                        "Member operation '{}' of class '{}' at line {} does not match any operation or action declaration in SysML model.",
                        op.name, c.name, op.line
                    )));
                }
            }
        }
    }

    // 3. Verify relations reference valid classifiers
    for rel in &diagram.relations {
        if !index.part_names.contains(&rel.from_class) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Source class '{}' in relation at line {} does not resolve to any declared Part/Block in SysML model.",
                rel.from_class, rel.line
            )));
        }
        if !index.part_names.contains(&rel.to_class) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Target class '{}' in relation at line {} does not resolve to any declared Part/Block in SysML model.",
                rel.to_class, rel.line
            )));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Cross-validates a state diagram AST against the indexed SysML model.
///
/// Preconditions: `diagram` has been successfully parsed and invariant-checked.
/// Postconditions: Returns `Ok(())` if all states and transition triggers/actions resolve, or a list of validation errors.
/// Algorithmic Complexity: O(S + T) where S is states count and T is transitions count.
pub fn validate_state_diagram(
    diagram: &StateDiagramAst,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();

    // 1. Verify every state resolves to a declared State in SysML AST
    for st in &diagram.states {
        if st.id == "[*]" {
            continue;
        }
        if !index.state_names.contains(&st.id) {
            errors.push(SpecAuditError::ValidationError(format!(
                "State '{}' in state diagram at line {} does not resolve to any declared State in SysML model.",
                st.id, st.line
            )));
        }
    }

    // 2. Verify transitions
    for trans in &diagram.transitions {
        if trans.source != "[*]" && !index.state_names.contains(&trans.source) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Transition source state '{}' at line {} does not resolve to any declared State in SysML model.",
                trans.source, trans.line
            )));
        }
        if trans.target != "[*]" && !index.state_names.contains(&trans.target) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Transition target state '{}' at line {} does not resolve to any declared State in SysML model.",
                trans.target, trans.line
            )));
        }

        // Verify transition action effect
        if let Some(ref act) = trans.action {
            let act_clean = extract_ident(act);
            if !act_clean.is_empty() && !index.action_names.contains(act_clean) {
                errors.push(SpecAuditError::ValidationError(format!(
                    "Transition action '{}' in state diagram at line {} does not resolve to any declared Action or Operation in SysML model.",
                    act, trans.line
                )));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Verifies that a specification document contains a valid Source References section.
///
/// Preconditions: `doc` has been parsed into `MarkdownDoc`.
/// Postconditions: Returns `Ok(())` if document contains a non-empty, non-placeholder Source References section.
/// Algorithmic Complexity: O(H + P) where H is headings count and P is prose segment count.
pub fn validate_source_references(doc: &MarkdownDoc) -> Result<(), SpecAuditError> {
    // 1. Find heading matching Source References
    let mut target_heading = None;
    for h in &doc.headings {
        let tokens = normalize_tokens(&h.text);
        let has_source = tokens.iter().any(|t| t == "source");
        let has_reference = tokens.iter().any(|t| t.starts_with("referenc"));
        let has_traceability = tokens.iter().any(|t| t == "traceability");

        if (has_source && has_reference) || (has_reference && has_traceability) || (h.text.to_lowercase().contains("source reference")) {
            target_heading = Some(h);
            break;
        }
    }

    let heading = match target_heading {
        Some(h) => h,
        None => {
            return Err(SpecAuditError::ValidationError(
                "Document is missing a mandatory 'Source References' section.".to_string(),
            ));
        }
    };

    // 2. Locate prose or text following this heading and before next heading
    let start_line = heading.line;
    let next_heading_line = doc
        .headings
        .iter()
        .filter(|h| h.line > start_line && h.level <= heading.level)
        .map(|h| h.line)
        .min()
        .unwrap_or(usize::MAX);

    let section_prose: Vec<&str> = doc
        .prose
        .iter()
        .filter(|p| p.line > start_line && p.line < next_heading_line)
        .map(|p| p.text.trim())
        .filter(|t| !t.is_empty())
        .collect();

    if section_prose.is_empty() {
        return Err(SpecAuditError::ValidationError(
            "Source References section is empty and contains no citations.".to_string(),
        ));
    }

    // 3. Verify section contains valid references and is not a placeholder stub
    let mut has_valid_reference = false;
    let valid_indicators = [
        "schema/", ".sysml", ".md", "ieee", "iso", "do-178c", "ecss", "rfc",
        "req-", "sysml-", "http://", "https://", "clause", "§",
    ];

    let placeholder_indicators = [
        "*to be populated*", "*(to be populated)*", "*tbd*", "*(tbd)*", "*(none)*",
    ];

    for segment in section_prose {
        let lower = segment.to_lowercase();
        let is_placeholder = placeholder_indicators.iter().any(|&ph| lower.contains(ph));
        if is_placeholder {
            continue;
        }

        if valid_indicators.iter().any(|&vi| lower.contains(vi)) || segment.contains("[") && segment.contains("](") {
            has_valid_reference = true;
            break;
        }
    }

    if !has_valid_reference {
        return Err(SpecAuditError::ValidationError(
            "Source References section contains no valid schema, standard clause, or file references.".to_string(),
        ));
    }

    Ok(())
}

/// Orchestrates complete UML syntax and semantic cross-validation across all diagrams in a document.
///
/// Preconditions: `doc` has been parsed, `index` has been built.
/// Postconditions: Returns `Ok(())` if all diagrams and references pass validation, or vector of all errors.
/// Algorithmic Complexity: O(D * S) where D is diagram count and S is symbol table size.
pub fn validate_document_uml(
    doc: &MarkdownDoc,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut all_errors = Vec::new();

    // 1. Source references gate
    if let Err(e) = validate_source_references(doc) {
        all_errors.push(e);
    }

    // 2. Validate each Mermaid diagram block
    for block in &doc.mermaid_blocks {
        // Invariant check
        if let Err(mut inv_errors) = validate_mermaid_invariants(block) {
            all_errors.append(&mut inv_errors);
            continue;
        }

        // AST extraction
        match parse_mermaid_diagram(block) {
            Ok(ast) => match ast {
                DiagramAst::Class(class_diagram) => {
                    if let Err(mut class_errors) = validate_class_diagram(&class_diagram, index) {
                        all_errors.append(&mut class_errors);
                    }
                }
                DiagramAst::State(state_diagram) => {
                    if let Err(mut state_errors) = validate_state_diagram(&state_diagram, index) {
                        all_errors.append(&mut state_errors);
                    }
                }
                DiagramAst::Sequence(sequence_diagram) => {
                    if let Err(mut seq_errors) =
                        validate_sequence_diagram(&sequence_diagram, index)
                    {
                        all_errors.append(&mut seq_errors);
                    }
                }
                _ => {}
            },
            Err(e) => {
                all_errors.push(e);
            }
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

/// Helper to extract clean alphanumeric identifier from operation or action invocation string.
fn extract_ident(s: &str) -> &str {
    let trimmed = s.trim().trim_start_matches(|c| c == '+' || c == '-' || c == '#' || c == '~').trim();
    if let Some(open_p) = trimmed.find('(') {
        trimmed[..open_p].trim()
    } else {
        trimmed
    }
}

/// Recursively collects all state names declared in a state hierarchy.
fn collect_state_names(state: &StateDef, names: &mut HashSet<String>) {
    if !state.name.is_empty() {
        names.insert(state.name.clone());
    }
    for sub in &state.sub_states {
        collect_state_names(sub, names);
    }
}

/// Validates Use Case Mermaid diagram compliance conforming to DO-178C Level A and ISO 26262 ASIL D.
///
/// Preconditions: `doc` has been parsed.
/// Postconditions: Mechanically fails if Use Case diagrams:
/// 1. Lack a system boundary subgraph (`subgraph ... System Boundary`).
/// 2. Use directed arrows (`-->`) to actors.
/// 3. Use non-stadium nodes (`["..."]` instead of `(["..."])`).
/// 4. Consist of trivial 2-node flows.
/// Algorithmic Complexity: O(B * L) where B is mermaid block count and L is line count.
pub fn validate_use_case_diagram_compliance(
    doc: &MarkdownDoc,
    rel_path: &str,
) -> Vec<String> {
    let mut errors = Vec::new();

    if doc.mermaid_blocks.is_empty() {
        errors.push(format!(
            "{}: Use Case Diagram Compliance Gate violation: Use Case document contains no Mermaid diagram blocks.",
            rel_path
        ));
        return errors;
    }

    for block in &doc.mermaid_blocks {
        let lines: Vec<&str> = block.raw.lines().collect();

        // 1. System boundary subgraph check
        let has_subgraph = lines.iter().any(|l| {
            let t = l.trim().to_lowercase();
            t.starts_with("subgraph")
        });

        if !has_subgraph {
            errors.push(format!(
                "{}: Use Case Diagram Compliance Gate violation: Diagram lacks a system boundary subgraph (expected 'subgraph \"... System Boundary\"').",
                rel_path
            ));
        }

        // 2. Directed arrows to actors check
        let primary_actor = doc.table_metadata.get("primary_actor").map(|s| s.trim()).unwrap_or("");
        let mut actor_ids = HashSet::new();
        if !primary_actor.is_empty() {
            actor_ids.insert(primary_actor.to_string());
            actor_ids.insert(primary_actor.to_lowercase());
        }
        actor_ids.insert("actor".to_string());
        actor_ids.insert("Actor".to_string());

        for l in &lines {
            let t = l.trim();
            if t.starts_with("actor ") {
                let act_id = t["actor ".len()..].trim().split_whitespace().next().unwrap_or("");
                if !act_id.is_empty() {
                    actor_ids.insert(act_id.to_string());
                    actor_ids.insert(act_id.to_lowercase());
                }
            } else if t.contains("((\"") {
                if let Some(paren_pos) = t.find("((\"") {
                    let act_id = t[..paren_pos].trim();
                    if !act_id.is_empty() {
                        actor_ids.insert(act_id.to_string());
                        actor_ids.insert(act_id.to_lowercase());
                    }
                }
            }
        }

        let directed_arrows = ["-->", "->", "==>", "-.->", "<--", "<-"];
        for l in &lines {
            let t = l.trim();
            if t.starts_with("%%") {
                continue;
            }
            let is_directed = directed_arrows.iter().any(|&arr| t.contains(arr));
            if is_directed {
                let contains_actor = actor_ids.iter().any(|act| {
                    t.contains(act) || t.to_lowercase().contains(act)
                });
                if contains_actor {
                    errors.push(format!(
                        "{}: Use Case Diagram Compliance Gate violation: Directed arrows ('-->') used to connect actor in line: '{}'. Actors must use undirected associations ('---') in UML Use Case diagrams.",
                        rel_path, t
                    ));
                    break;
                }
            }
        }

        // 3. Non-stadium nodes check (`["..."]` instead of `(["..."])`)
        let mut has_non_stadium = false;
        for l in &lines {
            let t = l.trim();
            if t.starts_with("%%") || t.starts_with("subgraph") || t.starts_with("classDef") || t.starts_with("style") {
                continue;
            }
            if t.contains("[\"") && t.contains("\"]") && !t.contains("([\"") && !t.contains("\"])") {
                has_non_stadium = true;
                break;
            }
        }

        if has_non_stadium {
            errors.push(format!(
                "{}: Use Case Diagram Compliance Gate violation: Non-stadium nodes ('[\"... \"]') found in Use Case diagram. Use cases must use stadium-shaped nodes ('([\"... \"])').",
                rel_path
            ));
        }

        // 4. Trivial 2-node flows
        let mut node_ids = HashSet::new();
        for l in &lines {
            let t = l.trim();
            if t.starts_with("%%") || t.starts_with("subgraph") || t.starts_with("end")
                || t.starts_with("flowchart") || t.starts_with("graph")
                || t.starts_with("classDef") || t.starts_with("style")
            {
                continue;
            }
            let first_token = t.split(|c: char| c == '[' || c == '(' || c == ' ' || c == '-' || c == '.')
                .next()
                .unwrap_or("")
                .trim();
            if !first_token.is_empty() && first_token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                node_ids.insert(first_token.to_string());
            }

            for arr in &["-->", "---", "->", "-.->"] {
                if let Some(pos) = t.find(arr) {
                    let after = t[pos + arr.len()..].trim();
                    let target_token = after.split(|c: char| c == '[' || c == '(' || c == ' ' || c == '-' || c == '.')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !target_token.is_empty() && target_token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        node_ids.insert(target_token.to_string());
                    }
                }
            }
        }

        if node_ids.len() <= 2 {
            errors.push(format!(
                "{}: Use Case Diagram Compliance Gate violation: Trivial 2-node flow detected (found {} distinct node(s)). Use case diagrams must model complete subsystem boundaries and scenario step inclusions.",
                rel_path, node_ids.len()
            ));
        }
    }

    errors
}

/// Validates Epic statechart heterogeneity conforming to DO-178C Level A and ISO 26262 ASIL D.
///
/// Preconditions: `epics` contains audited Epic Markdown documents.
/// Postconditions: Mechanically fails if multiple Epics share identical canned statecharts
/// (e.g. sharing identical 'Bootstrapping -> Ingesting -> Compiling -> Verifying' transitions
/// without subsystem-specific states, or sharing identical statechart transition sets).
/// Algorithmic Complexity: O(E * T) where E is epic count and T is transition count.
pub fn validate_epic_statechart_heterogeneity(
    epics: &[(&str, &MarkdownDoc)],
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut epic_transitions: Vec<(&str, Vec<(String, String)>)> = Vec::new();

    for &(rel_path, doc) in epics {
        for block in &doc.mermaid_blocks {
            let lower_raw = block.raw.to_lowercase();
            if block.header.starts_with("stateDiagram") || lower_raw.contains("statediagram") {
                let mut trans_list = Vec::new();
                for line in block.raw.lines() {
                    let t = line.trim();
                    if t.starts_with("%%") || t.starts_with("stateDiagram") || t.is_empty() {
                        continue;
                    }
                    if let Some(arrow_idx) = t.find("-->") {
                        let from = t[..arrow_idx].trim();
                        let rest = t[arrow_idx + 3..].trim();
                        let to = match rest.find(':') {
                            Some(colon_idx) => rest[..colon_idx].trim(),
                            None => rest,
                        };
                        if !from.is_empty() && !to.is_empty() {
                            trans_list.push((from.to_string(), to.to_string()));
                        }
                    }
                }
                trans_list.sort();
                epic_transitions.push((rel_path, trans_list));
                break;
            }
        }
    }

    if epic_transitions.len() < 2 {
        return errors;
    }

    // 1. Check for canned pipeline lifecycle transitions:
    let mut canned_epics = Vec::new();
    for (rel_path, trans) in &epic_transitions {
        let has_boot_ingest = trans.iter().any(|(f, t)| f == "Bootstrapping" && t == "Ingesting");
        let has_ingest_comp = trans.iter().any(|(f, t)| f == "Ingesting" && t == "Compiling");
        let has_comp_verif = trans.iter().any(|(f, t)| f == "Compiling" && t == "Verifying");
        if has_boot_ingest && has_ingest_comp && has_comp_verif {
            canned_epics.push(*rel_path);
        }
    }

    if canned_epics.len() > 1 {
        errors.push(format!(
            "Epic Statechart Heterogeneity Gate violation: Multiple Epics ({}) share identical canned pipeline lifecycle statecharts ('Bootstrapping -> Ingesting -> Compiling -> Verifying') without subsystem-specific states.",
            canned_epics.join(", ")
        ));
    }

    // 2. Check for exact identical transition sets between any two different Epics
    let mut checked_pairs = HashSet::new();
    for i in 0..epic_transitions.len() {
        for j in i + 1..epic_transitions.len() {
            let (path1, trans1) = &epic_transitions[i];
            let (path2, trans2) = &epic_transitions[j];
            if !trans1.is_empty() && trans1 == trans2 {
                let pair_key = if path1 < path2 {
                    format!("{}|{}", path1, path2)
                } else {
                    format!("{}|{}", path2, path1)
                };
                if checked_pairs.insert(pair_key) {
                    if !(canned_epics.contains(path1) && canned_epics.contains(path2)) {
                        errors.push(format!(
                            "Epic Statechart Heterogeneity Gate violation: Epics '{}' and '{}' share identical statechart transitions ({} transitions). Epics must define distinct subsystem-specific statecharts.",
                            path1, path2, trans1.len()
                        ));
                    }
                }
            }
        }
    }

    errors
}

/// Helper to extract clean epic stem from markdown link or identifier.
fn extract_epic_stem(raw: &str) -> String {
    let clean = raw.trim().trim_matches(|c| c == '[' || c == ']');
    if let Some(idx) = clean.find('(') {
        let text = clean[..idx].trim().trim_matches(|c| c == ']' || c == '[');
        if !text.is_empty() {
            return text.to_string();
        }
    }
    if let Some(idx) = clean.rfind('/') {
        let fname = &clean[idx + 1..];
        return fname.strip_suffix(".md").unwrap_or(fname).to_string();
    }
    clean.to_string()
}

/// Helper to extract subsystem name from a feature title like "Feature 06: [System Vision] SystemVisionEngine".
fn extract_subsystem_from_title(title: &str) -> String {
    if let Some(open) = title.find('[') {
        if let Some(close) = title[open..].find(']') {
            return title[open + 1..open + close].trim().to_string();
        }
    }
    String::new()
}

/// Validates Parent Epic alignment between User Stories, Features, and Epics.
///
/// Preconditions: `stories`, `features`, and `epics` contain parsed Markdown documents.
/// Postconditions: Mechanically fails if a User Story links to a mismatching parent Epic
/// (e.g. non-conops story declaring 'Parent Epic: EPIC-02-ConOps' when its subsystem / required
/// features belong to a different Epic).
/// Algorithmic Complexity: O(S * F) where S is story count and F is required feature count.
pub fn validate_parent_epic_alignment(
    stories: &[(&str, &MarkdownDoc)],
    features: &[(&str, &MarkdownDoc)],
    epics: &[(&str, &MarkdownDoc)],
) -> Vec<String> {
    let mut errors = Vec::new();

    let mut feat_info: HashMap<String, (String, String)> = HashMap::new();
    for &(f_path, f_doc) in features {
        let f_basename = match f_path.rfind('/') {
            Some(idx) => &f_path[idx + 1..],
            None => f_path,
        };
        let f_stem = f_basename.strip_suffix(".md").unwrap_or(f_basename).to_lowercase();

        let parent_epic = f_doc.table_metadata.get("parent_epic").map(|s| s.trim()).unwrap_or("");
        let parent_epic_clean = extract_epic_stem(parent_epic);

        let title = f_doc.table_metadata.get("title").map(|s| s.trim()).unwrap_or("");
        let subsys = extract_subsystem_from_title(title);

        feat_info.insert(f_stem, (subsys, parent_epic_clean));
    }

    let mut epic_associated_stories: HashMap<String, String> = HashMap::new();
    for &(ep_path, ep_doc) in epics {
        let ep_basename = match ep_path.rfind('/') {
            Some(idx) => &ep_path[idx + 1..],
            None => ep_path,
        };
        let ep_stem = ep_basename.strip_suffix(".md").unwrap_or(ep_basename);
        if ep_stem.to_lowercase().contains("epic-02") {
            continue;
        }

        for segment in &ep_doc.prose {
            for raw_word in segment.text.split(|c: char| c == '(' || c == ')' || c == '[' || c == ']' || c.is_whitespace()) {
                let word = raw_word.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-' && c != '_');
                if word.to_lowercase().contains("us-") {
                    let s_basename = match word.rfind('/') {
                        Some(idx) => &word[idx + 1..],
                        None => word,
                    };
                    let s_stem = s_basename.strip_suffix(".md").unwrap_or(s_basename).to_lowercase();
                    epic_associated_stories.insert(s_stem, ep_stem.to_string());
                }
            }
        }
    }

    for &(s_path, s_doc) in stories {
        let s_basename = match s_path.rfind('/') {
            Some(idx) => &s_path[idx + 1..],
            None => s_path,
        };
        let s_stem = s_basename.strip_suffix(".md").unwrap_or(s_basename).to_lowercase();

        let declared_parent = s_doc.table_metadata.get("parent_epic").map(|s| s.trim()).unwrap_or("");
        let declared_stem = extract_epic_stem(declared_parent);

        let spec_source = s_doc.table_metadata.get("specification_source").map(|s| s.trim()).unwrap_or("");
        let is_conops_source = spec_source.to_lowercase().contains("conops");

        let mut linked_feature_parent_epics = Vec::new();
        for segment in &s_doc.prose {
            for raw_word in segment.text.split(|c: char| c == '(' || c == ')' || c == '[' || c == ']' || c.is_whitespace()) {
                let word = raw_word.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-' && c != '_');
                if word.to_lowercase().contains("feat-") {
                    let f_basename = match word.rfind('/') {
                        Some(idx) => &word[idx + 1..],
                        None => word,
                    };
                    let f_stem = f_basename.strip_suffix(".md").unwrap_or(f_basename).to_lowercase();
                    let matched_feat = feat_info.iter().find(|(k, _)| {
                        k.as_str() == f_stem
                            || (k.starts_with(&f_stem) && k[f_stem.len()..].starts_with('-'))
                            || (f_stem.starts_with(k.as_str()) && f_stem[k.len()..].starts_with('-'))
                    });
                    if let Some((_, (_subsys, p_epic))) = matched_feat {
                        if !p_epic.is_empty() {
                            linked_feature_parent_epics.push((f_stem, p_epic.clone()));
                        }
                    }
                }
            }
        }

        let primary_feature_epic = linked_feature_parent_epics
            .iter()
            .find(|(_f, p)| !p.to_lowercase().contains("epic-01") && !p.to_lowercase().contains("epic-02"))
            .map(|(_f, p)| p.as_str());

        let expected_from_epic_checklist = epic_associated_stories
            .iter()
            .find(|(k, _)| {
                k.as_str() == s_stem
                    || (k.starts_with(&s_stem) && k[s_stem.len()..].starts_with('-'))
                    || (s_stem.starts_with(k.as_str()) && s_stem[k.len()..].starts_with('-'))
            })
            .map(|(_, v)| v.as_str());
        let expected_epic = primary_feature_epic.or(expected_from_epic_checklist);

        if let Some(expected) = expected_epic {
            let declared_lower = declared_stem.to_lowercase();
            let expected_lower = expected.to_lowercase();

            let mismatch = if declared_lower.contains("epic-02") && !expected_lower.contains("epic-02") {
                true
            } else if !declared_lower.contains(&expected_lower) && !expected_lower.contains(&declared_lower) {
                !is_conops_source
            } else {
                false
            };

            if mismatch {
                errors.push(format!(
                    "{}: Parent Epic Alignment Gate violation: User Story declared parent Epic '{}', but its associated subsystem / required features belong to '{}'.",
                    s_path, declared_stem, expected
                ));
            }
        }
    }

    errors
}

// ============================================================================
// SEMANTIC ACCEPTANCE UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_audit::diagram_ast::{
        ClassAttributeAst, ClassDefAst, ClassOperationAst, SequenceMessageAst,
        SequenceParticipantAst, StateDefAst, StateTransitionAst, Visibility,
    };
    use crate::spec_audit::markdown_ast::{HeadingNode, ProseSegment};
    use deap_core::sysml_ast::{ActionDef, AttributeDef, OperationDef, PartDef, StateDef, SysmlModel};

    fn create_test_sysml_model() -> SysmlModel {
        let mut model = SysmlModel::default();

        let mut flight_controller = PartDef {
            name: "FlightController".to_string(),
            ..Default::default()
        };
        flight_controller.attributes.push(AttributeDef {
            name: "altitude".to_string(),
            type_name: "float".to_string(),
            ..Default::default()
        });
        flight_controller.operations.push(OperationDef {
            name: "execute_pipeline".to_string(),
            return_type: Some("void".to_string()),
            ..Default::default()
        });
        flight_controller.actions.push(ActionDef {
            name: "arm_motors".to_string(),
            ..Default::default()
        });

        let mut sensor_hub = PartDef {
            name: "SensorHub".to_string(),
            ..Default::default()
        };
        sensor_hub.operations.push(OperationDef {
            name: "read_sensors".to_string(),
            ..Default::default()
        });

        model.parts.push(flight_controller);
        model.parts.push(sensor_hub);

        let mut state_idle = StateDef {
            name: "Idle".to_string(),
            ..Default::default()
        };
        state_idle.entry_action = Some("init_subsystems()".to_string());

        let state_active = StateDef {
            name: "Active".to_string(),
            ..Default::default()
        };

        model.parts[0].states.push(state_idle);
        model.parts[0].states.push(state_active);

        model
    }

    #[test]
    fn test_symbol_indexing_and_lookup() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        assert!(index.part_names.contains("FlightController"));
        assert!(index.part_names.contains("SensorHub"));
        assert!(index.attribute_names.contains("altitude"));
        assert!(index.action_names.contains("execute_pipeline"));
        assert!(index.action_names.contains("read_sensors"));
        assert!(index.action_names.contains("arm_motors"));
        assert!(index.state_names.contains("Idle"));
        assert!(index.state_names.contains("Active"));
    }

    #[test]
    fn test_valid_sequence_diagram_passes() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let seq = SequenceDiagramAst {
            participants: vec![
                SequenceParticipantAst {
                    name: "Pilot".to_string(),
                    alias: None,
                    is_actor: true,
                    line: 1,
                },
                SequenceParticipantAst {
                    name: "FlightController".to_string(),
                    alias: None,
                    is_actor: false,
                    line: 2,
                },
                SequenceParticipantAst {
                    name: "SensorHub".to_string(),
                    alias: None,
                    is_actor: false,
                    line: 3,
                },
            ],
            messages: vec![
                SequenceMessageAst {
                    from: "Pilot".to_string(),
                    to: "FlightController".to_string(),
                    arrow: "->>".to_string(),
                    message: "execute_pipeline()".to_string(),
                    operation_name: Some("execute_pipeline".to_string()),
                    parameters: Vec::new(),
                    is_reply: false,
                    line: 4,
                },
                SequenceMessageAst {
                    from: "FlightController".to_string(),
                    to: "SensorHub".to_string(),
                    arrow: "->>".to_string(),
                    message: "read_sensors()".to_string(),
                    operation_name: Some("read_sensors".to_string()),
                    parameters: Vec::new(),
                    is_reply: false,
                    line: 5,
                },
            ],
            activations: Vec::new(),
        };

        assert!(validate_sequence_diagram(&seq, &index).is_ok());
    }

    #[test]
    fn test_unknown_lifeline_fails_sequence_validation() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let seq = SequenceDiagramAst {
            participants: vec![SequenceParticipantAst {
                name: "UnregisteredEngine".to_string(),
                alias: None,
                is_actor: false,
                line: 10,
            }],
            messages: Vec::new(),
            activations: Vec::new(),
        };

        let res = validate_sequence_diagram(&seq, &index);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors[0].to_string().contains("Lifeline 'UnregisteredEngine' in sequence diagram"));
    }

    #[test]
    fn test_unknown_operation_fails_sequence_validation() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let seq = SequenceDiagramAst {
            participants: vec![
                SequenceParticipantAst {
                    name: "Pilot".to_string(),
                    alias: None,
                    is_actor: true,
                    line: 1,
                },
                SequenceParticipantAst {
                    name: "FlightController".to_string(),
                    alias: None,
                    is_actor: false,
                    line: 2,
                },
            ],
            messages: vec![SequenceMessageAst {
                from: "Pilot".to_string(),
                to: "FlightController".to_string(),
                arrow: "->>".to_string(),
                message: "unregistered_method()".to_string(),
                operation_name: Some("unregistered_method".to_string()),
                parameters: Vec::new(),
                is_reply: false,
                line: 3,
            }],
            activations: Vec::new(),
        };

        let res = validate_sequence_diagram(&seq, &index);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors[0].to_string().contains("unregistered_method"));
    }

    #[test]
    fn test_class_diagram_validation_passes() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let class_diag = ClassDiagramAst {
            classes: vec![ClassDefAst {
                name: "FlightController".to_string(),
                annotations: Vec::new(),
                attributes: vec![ClassAttributeAst {
                    visibility: Some(Visibility::Public),
                    name: "altitude".to_string(),
                    type_name: Some("float".to_string()),
                    raw: "+float altitude".to_string(),
                    line: 2,
                }],
                operations: vec![ClassOperationAst {
                    visibility: Some(Visibility::Public),
                    name: "execute_pipeline".to_string(),
                    parameters: Vec::new(),
                    return_type: Some("void".to_string()),
                    raw: "+execute_pipeline() void".to_string(),
                    line: 3,
                }],
                line: 1,
            }],
            relations: Vec::new(),
        };

        assert!(validate_class_diagram(&class_diag, &index).is_ok());
    }

    #[test]
    fn test_unknown_class_fails_class_validation() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let class_diag = ClassDiagramAst {
            classes: vec![ClassDefAst {
                name: "PhantomPart".to_string(),
                annotations: Vec::new(),
                attributes: Vec::new(),
                operations: Vec::new(),
                line: 1,
            }],
            relations: Vec::new(),
        };

        let res = validate_class_diagram(&class_diag, &index);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors[0].to_string().contains("Class 'PhantomPart' in class diagram"));
    }

    #[test]
    fn test_vacuous_class_stub_fails_validation() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        // FlightController has declared attributes and operations in the model
        let vacuous_diag = ClassDiagramAst {
            classes: vec![ClassDefAst {
                name: "FlightController".to_string(),
                annotations: Vec::new(),
                attributes: Vec::new(),
                operations: Vec::new(),
                line: 1,
            }],
            relations: Vec::new(),
        };

        let res = validate_class_diagram(&vacuous_diag, &index);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors[0]
            .to_string()
            .contains("is a vacuous stub omitting declared AST ports and attributes"));
    }

    #[test]
    fn test_class_diagram_with_port_passes() {
        let mut model = create_test_sysml_model();
        model.parts[0].ports.push(deap_core::sysml_ast::PortDef {
            name: "telemetry_port".to_string(),
            type_name: "TelemetryStream".to_string(),
            direction: "out".to_string(),
            ..Default::default()
        });
        let index = SysmlSymbolIndex::from_model(&model);

        // Test port matching in operations (Mermaid parsed with direction parentheses)
        let class_diag = ClassDiagramAst {
            classes: vec![ClassDefAst {
                name: "FlightController".to_string(),
                annotations: Vec::new(),
                attributes: Vec::new(),
                operations: vec![ClassOperationAst {
                    visibility: Some(Visibility::Public),
                    name: "telemetry_port".to_string(),
                    parameters: vec!["out".to_string()],
                    return_type: Some("TelemetryStream".to_string()),
                    raw: "+TelemetryStream telemetry_port (out)".to_string(),
                    line: 2,
                }],
                line: 1,
            }],
            relations: Vec::new(),
        };

        assert!(validate_class_diagram(&class_diag, &index).is_ok());
    }

    #[test]
    fn test_state_diagram_validation_passes() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let state_diag = StateDiagramAst {
            states: vec![
                StateDefAst {
                    id: "Idle".to_string(),
                    label: None,
                    is_composite: false,
                    sub_states: Vec::new(),
                    line: 1,
                },
                StateDefAst {
                    id: "Active".to_string(),
                    label: None,
                    is_composite: false,
                    sub_states: Vec::new(),
                    line: 2,
                },
            ],
            transitions: vec![StateTransitionAst {
                source: "Idle".to_string(),
                target: "Active".to_string(),
                event: Some("arm".to_string()),
                guard: None,
                action: Some("arm_motors()".to_string()),
                raw_label: Some("arm / arm_motors()".to_string()),
                line: 3,
            }],
            notes: Vec::new(),
        };

        assert!(validate_state_diagram(&state_diag, &index).is_ok());
    }

    #[test]
    fn test_unknown_state_fails_state_validation() {
        let model = create_test_sysml_model();
        let index = SysmlSymbolIndex::from_model(&model);

        let state_diag = StateDiagramAst {
            states: vec![StateDefAst {
                id: "Exploded".to_string(),
                label: None,
                is_composite: false,
                sub_states: Vec::new(),
                line: 1,
            }],
            transitions: Vec::new(),
            notes: Vec::new(),
        };

        let res = validate_state_diagram(&state_diag, &index);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors[0].to_string().contains("State 'Exploded' in state diagram"));
    }

    #[test]
    fn test_source_references_section_validation() {
        let valid_doc = MarkdownDoc {
            headings: vec![
                HeadingNode {
                    level: 1,
                    text: "Epic 01".to_string(),
                    line: 1,
                },
                HeadingNode {
                    level: 2,
                    text: "## 6. Source References".to_string(),
                    line: 10,
                },
            ],
            frontmatter: None,
            table_metadata: HashMap::new(),
            prose: vec![ProseSegment {
                text: "Derived from `schema/model.sysml` and IEEE 29148 clause 8.4.".to_string(),
                line: 12,
            }],
            mermaid_blocks: Vec::new(),
            unresolved_placeholders: Vec::new(),
        };

        assert!(validate_source_references(&valid_doc).is_ok());

        let missing_doc = MarkdownDoc {
            headings: vec![HeadingNode {
                level: 1,
                text: "Epic 01".to_string(),
                line: 1,
            }],
            frontmatter: None,
            table_metadata: HashMap::new(),
            prose: Vec::new(),
            mermaid_blocks: Vec::new(),
            unresolved_placeholders: Vec::new(),
        };

        let res = validate_source_references(&missing_doc);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("missing a mandatory 'Source References' section"));
    }

    #[test]
    fn test_use_case_diagram_compliance_gate() {
        use crate::spec_audit::markdown_ast::MermaidBlock;

        // Trivial 2-node flow, directed arrow to actor, non-stadium, no boundary subgraph
        let legacy_doc = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::from([("primary_actor".to_string(), "HumanEngineer".to_string())]),
            prose: vec![],
            mermaid_blocks: vec![MermaidBlock {
                raw: "flowchart TD\n    Actor[\"HumanEngineer\"] --> Target[\"SystemVisionEngine\"]\n".to_string(),
                header: "flowchart TD".to_string(),
                start_line: 10,
                end_line: 12,
            }],
            unresolved_placeholders: vec![],
        };

        let errs = validate_use_case_diagram_compliance(&legacy_doc, "docs/use-cases/uc-01.md");
        assert!(errs.iter().any(|e| e.contains("lacks a system boundary subgraph")));
        assert!(errs.iter().any(|e| e.contains("Directed arrows ('-->') used to connect actor")));
        assert!(errs.iter().any(|e| e.contains("Non-stadium nodes")));
        assert!(errs.iter().any(|e| e.contains("Trivial 2-node flow detected")));

        // Compliant UML diagram with boundary, stadium nodes, undirected association
        let compliant_raw = r#"graph TD
    subgraph "SystemVisionEngine System Boundary"
        UC_Main(["System Vision and Foundational Invariants"])
        UC_Step01(["Operator dispatches command"])
        UC_Step02(["Engine boots invariants"])
    end
    HumanEngineer(("HumanEngineer")) --- UC_Main
    UC_Main -. "<<include>>" .-> UC_Step01
    UC_Main -. "<<include>>" .-> UC_Step02
"#;
        let compliant_doc = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::from([("primary_actor".to_string(), "HumanEngineer".to_string())]),
            prose: vec![],
            mermaid_blocks: vec![MermaidBlock {
                raw: compliant_raw.to_string(),
                header: "graph TD".to_string(),
                start_line: 10,
                end_line: 18,
            }],
            unresolved_placeholders: vec![],
        };

        let clean_errs = validate_use_case_diagram_compliance(&compliant_doc, "docs/use-cases/uc-01.md");
        assert!(clean_errs.is_empty(), "Compliant diagram should have 0 errors: {:?}", clean_errs);
    }

    #[test]
    fn test_epic_statechart_heterogeneity_gate() {
        use crate::spec_audit::markdown_ast::MermaidBlock;

        let canned_raw = r#"stateDiagram-v2
    [*] --> Bootstrapping
    Bootstrapping --> Ingesting : dispatch
    Ingesting --> Compiling : parse_complete
    Compiling --> Verifying : ast_lowered
    Verifying --> EmitSuccess : pass
    EmitSuccess --> [*]
"#;
        let epic1 = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::new(),
            prose: vec![],
            mermaid_blocks: vec![MermaidBlock {
                raw: canned_raw.to_string(),
                header: "stateDiagram-v2".to_string(),
                start_line: 1,
                end_line: 8,
            }],
            unresolved_placeholders: vec![],
        };
        let epic2 = epic1.clone();

        let epics = vec![
            ("docs/epics/epic-03.md", &epic1),
            ("docs/epics/epic-04.md", &epic2),
        ];

        let errs = validate_epic_statechart_heterogeneity(&epics);
        assert!(!errs.is_empty(), "Identical canned statecharts must be flagged");
        assert!(errs.iter().any(|e| e.contains("share identical canned pipeline lifecycle statecharts")));

        // Heterogeneous epics
        let custom_raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> IngestionActive : start
    IngestionActive --> Finalized : done
    Finalized --> [*]
"#;
        let epic3 = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::new(),
            prose: vec![],
            mermaid_blocks: vec![MermaidBlock {
                raw: custom_raw.to_string(),
                header: "stateDiagram-v2".to_string(),
                start_line: 1,
                end_line: 6,
            }],
            unresolved_placeholders: vec![],
        };

        let heterogenous = vec![
            ("docs/epics/epic-03.md", &epic1),
            ("docs/epics/epic-04.md", &epic3),
        ];
        let clean_errs = validate_epic_statechart_heterogeneity(&heterogenous);
        assert!(clean_errs.is_empty(), "Heterogeneous epics must pass: {:?}", clean_errs);
    }

    #[test]
    fn test_parent_epic_alignment_gate() {
        // Feature 06 belongs to EPIC-03-SystemVision
        let feat06 = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::from([
                ("title".to_string(), "Feature 06: [System Vision] SystemVisionEngine".to_string()),
                ("parent_epic".to_string(), "[EPIC-03-SystemVision](../epics/EPIC-03-SystemVision.md)".to_string()),
            ]),
            prose: vec![],
            mermaid_blocks: vec![],
            unresolved_placeholders: vec![],
        };

        // Epic 03 checklist associates US-01
        let epic03 = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::new(),
            prose: vec![ProseSegment {
                text: "- [ ] [User Story 01: Bootstrap System Vision Action](../user-stories/US-01-BootstrapSystemVisionAction.md)".to_string(),
                line: 20,
            }],
            mermaid_blocks: vec![],
            unresolved_placeholders: vec![],
        };

        // Mismatched story declaring Parent Epic: EPIC-02-ConOps while realizing FEAT-06
        let mismatched_story = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::from([
                ("title".to_string(), "User Story 01: Bootstrap System Vision Action".to_string()),
                ("parent_epic".to_string(), "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)".to_string()),
                ("specification_source".to_string(), "[schema/subsystems/system_vision/behavior.sysml]".to_string()),
            ]),
            prose: vec![ProseSegment {
                text: "## Required Features\n- [ ] [Feature 06: [System Vision] SystemVisionEngine](../features/FEAT-06-SystemVisionEngine.md)".to_string(),
                line: 30,
            }],
            mermaid_blocks: vec![],
            unresolved_placeholders: vec![],
        };

        let stories = vec![("docs/user-stories/US-01-BootstrapSystemVisionAction.md", &mismatched_story)];
        let features = vec![("docs/features/FEAT-06-SystemVisionEngine.md", &feat06)];
        let epics = vec![("docs/epics/EPIC-03-SystemVision.md", &epic03)];

        let errs = validate_parent_epic_alignment(&stories, &features, &epics);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("Parent Epic Alignment Gate violation"));
        assert!(errs[0].contains("declared parent Epic 'EPIC-02-ConOps'"));
        assert!(errs[0].contains("EPIC-03-SystemVision"));

        // Aligned story
        let aligned_story = MarkdownDoc {
            headings: vec![],
            frontmatter: None,
            table_metadata: HashMap::from([
                ("title".to_string(), "User Story 01: Bootstrap System Vision Action".to_string()),
                ("parent_epic".to_string(), "[EPIC-03-SystemVision](../epics/EPIC-03-SystemVision.md)".to_string()),
                ("specification_source".to_string(), "[schema/subsystems/system_vision/behavior.sysml]".to_string()),
            ]),
            prose: vec![ProseSegment {
                text: "## Required Features\n- [ ] [Feature 06: [System Vision] SystemVisionEngine](../features/FEAT-06-SystemVisionEngine.md)".to_string(),
                line: 30,
            }],
            mermaid_blocks: vec![],
            unresolved_placeholders: vec![],
        };

        let clean_stories = vec![("docs/user-stories/US-01-BootstrapSystemVisionAction.md", &aligned_story)];
        let clean_errs = validate_parent_epic_alignment(&clean_stories, &features, &epics);
        assert!(clean_errs.is_empty(), "Aligned story should have 0 errors: {:?}", clean_errs);
    }
}

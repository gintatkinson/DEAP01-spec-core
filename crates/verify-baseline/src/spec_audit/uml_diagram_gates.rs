//! UML sequence, class, state, and source reference validation gates.
//! Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR]

use std::collections::BTreeSet;

use crate::spec_audit::diagram_ast::{
    parse_mermaid_diagram, validate_mermaid_invariants, ClassDiagramAst, DiagramAst,
    SequenceDiagramAst, StateDiagramAst,
};
use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::sections::normalize_tokens;
use crate::spec_audit::uml_validator::{extract_ident, SysmlSymbolIndex};
use crate::spec_audit::SpecAuditError;

/// Cross-validates a sequence diagram AST against the indexed SysML model.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-SEQUENCE-DIAGRAM]
pub fn validate_sequence_diagram(
    diagram: &SequenceDiagramAst,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();
    let mut actor_names: BTreeSet<String> = BTreeSet::new();

    for p in &diagram.participants {
        if p.is_actor {
            actor_names.insert(p.name.clone());
            if let Some(ref a) = p.alias {
                actor_names.insert(a.clone());
            }
            continue;
        }
        let name_match = index.part_names.contains(&p.name);
        let alias_match = p.alias.as_ref().is_some_and(|a| index.part_names.contains(a));
        if !name_match && !alias_match {
            errors.push(SpecAuditError::ValidationError(format!(
                "Lifeline '{}' in sequence diagram at line {} does not resolve to any declared Part/Block in SysML model.",
                p.name, p.line
            )));
        }
    }

    for msg in &diagram.messages {
        if msg.is_reply || actor_names.contains(&msg.to) {
            continue;
        }
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
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-CLASS-DIAGRAM]
pub fn validate_class_diagram(
    diagram: &ClassDiagramAst,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();

    for c in &diagram.classes {
        if !index.part_names.contains(&c.name) {
            errors.push(SpecAuditError::ValidationError(format!(
                "Class '{}' in class diagram at line {} does not resolve to any declared Part/Block in SysML model.",
                c.name, c.line
            )));
            continue;
        }
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
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-STATE-DIAGRAM]
pub fn validate_state_diagram(
    diagram: &StateDiagramAst,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();

    for st in &diagram.states {
        if st.id != "[*]" && !index.state_names.contains(&st.id) {
            errors.push(SpecAuditError::ValidationError(format!(
                "State '{}' in state diagram at line {} does not resolve to any declared State in SysML model.",
                st.id, st.line
            )));
        }
    }

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
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-SOURCE-REFERENCES]
pub fn validate_source_references(doc: &MarkdownDoc) -> Result<(), SpecAuditError> {
    let mut target_heading = None;
    for h in &doc.headings {
        let tokens = normalize_tokens(&h.text);
        let has_source = tokens.iter().any(|t| t == "source");
        let has_reference = tokens.iter().any(|t| t.starts_with("referenc"));
        let has_traceability = tokens.iter().any(|t| t == "traceability");
        if (has_source && has_reference)
            || (has_reference && has_traceability)
            || h.text.to_lowercase().contains("source reference")
        {
            target_heading = Some(h);
            break;
        }
    }

    let Some(heading) = target_heading else {
        return Err(SpecAuditError::ValidationError(
            "Document is missing a mandatory 'Source References' section.".to_string(),
        ));
    };

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

    let valid_indicators = [
        "schema/", ".sysml", ".md", "ieee", "iso", "do-178c", "ecss", "rfc", "req-", "sysml-",
        "http://", "https://", "clause", "§",
    ];
    let placeholder_indicators = [
        "*to be populated*",
        "*(to be populated)*",
        "*tbd*",
        "*(tbd)*",
        "*(none)*",
    ];

    for segment in section_prose {
        let lower = segment.to_lowercase();
        if placeholder_indicators.iter().any(|&ph| lower.contains(ph)) {
            continue;
        }
        if valid_indicators.iter().any(|&vi| lower.contains(vi))
            || (segment.contains('[') && segment.contains("]("))
        {
            return Ok(());
        }
    }

    Err(SpecAuditError::ValidationError(
        "Source References section contains no valid schema, standard clause, or file references."
            .to_string(),
    ))
}

/// Orchestrates complete UML syntax and semantic cross-validation across all diagrams in a document.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-DOCUMENT-UML]
pub fn validate_document_uml(
    doc: &MarkdownDoc,
    index: &SysmlSymbolIndex,
) -> Result<(), Vec<SpecAuditError>> {
    let mut all_errors = Vec::new();
    if let Err(e) = validate_source_references(doc) {
        all_errors.push(e);
    }

    for block in &doc.mermaid_blocks {
        if let Err(mut inv_errors) = validate_mermaid_invariants(block) {
            all_errors.append(&mut inv_errors);
            continue;
        }
        match parse_mermaid_diagram(block) {
            Ok(DiagramAst::Class(class_diagram)) => {
                if let Err(mut errs) = validate_class_diagram(&class_diagram, index) {
                    all_errors.append(&mut errs);
                }
            }
            Ok(DiagramAst::State(state_diagram)) => {
                if let Err(mut errs) = validate_state_diagram(&state_diagram, index) {
                    all_errors.append(&mut errs);
                }
            }
            Ok(DiagramAst::Sequence(sequence_diagram)) => {
                if let Err(mut errs) = validate_sequence_diagram(&sequence_diagram, index) {
                    all_errors.append(&mut errs);
                }
            }
            Ok(_) => {}
            Err(e) => all_errors.push(e),
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

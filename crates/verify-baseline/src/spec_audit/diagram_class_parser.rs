//! Mermaid `classDiagram` parser for `spec_audit::diagram_ast`.
//! Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]

use crate::spec_audit::diagram_ast::{
    ClassAttributeAst, ClassDefAst, ClassDiagramAst, ClassOperationAst, ClassRelationAst,
    ClassRelationType, Visibility,
};
use crate::spec_audit::diagram_invariants::{contains_relation_arrow, strip_mermaid_comment};
use crate::spec_audit::markdown_ast::MermaidBlock;
use crate::spec_audit::SpecAuditError;

/// Parses a Mermaid `classDiagram` block into [`ClassDiagramAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-DIAGRAM]
pub(crate) fn parse_class_diagram(block: &MermaidBlock) -> Result<ClassDiagramAst, SpecAuditError> {
    let mut classes_map: Vec<ClassDefAst> = Vec::new();
    let mut relations = Vec::new();
    let mut current_class: Option<ClassDefAst> = None;

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() || clean.eq_ignore_ascii_case("classDiagram") {
            continue;
        }
        if clean == "}" {
            if let Some(c) = current_class.take() {
                merge_or_push_class(&mut classes_map, c);
            }
            continue;
        }
        if let Some(ref mut c) = current_class {
            if clean.starts_with("<<") && clean.ends_with(">>") {
                let annot = clean.trim_matches(|ch| ch == '<' || ch == '>').to_string();
                c.annotations.push(annot);
                continue;
            }
            parse_class_member_line(clean, c, line_num);
            continue;
        }
        if parse_class_line_outside_block(
            clean,
            line_num,
            &mut classes_map,
            &mut relations,
            &mut current_class,
        ) {
            continue;
        }
    }

    if let Some(c) = current_class {
        merge_or_push_class(&mut classes_map, c);
    }

    Ok(ClassDiagramAst { classes: classes_map, relations })
}

/// Handles top-level `classDiagram` lines outside an open `class Name {` block.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-DIAGRAM]
fn parse_class_line_outside_block(
    clean: &str,
    line_num: usize,
    classes_map: &mut Vec<ClassDefAst>,
    relations: &mut Vec<ClassRelationAst>,
    current_class: &mut Option<ClassDefAst>,
) -> bool {
    if clean.starts_with("class ") && clean.ends_with('{') {
        let name_part = clean["class ".len()..clean.len() - 1].trim();
        *current_class = Some(ClassDefAst {
            name: name_part.to_string(),
            annotations: Vec::new(),
            attributes: Vec::new(),
            operations: Vec::new(),
            line: line_num,
        });
        return true;
    }

    if let Some(after_class) = clean.strip_prefix("class ") {
        let name_part = after_class.trim();
        if !name_part.is_empty() {
            merge_or_push_class(
                classes_map,
                ClassDefAst {
                    name: name_part.to_string(),
                    annotations: Vec::new(),
                    attributes: Vec::new(),
                    operations: Vec::new(),
                    line: line_num,
                },
            );
        }
        return true;
    }

    if let Some(rel) = parse_class_relation(clean, line_num) {
        ensure_class_stub(classes_map, &rel.from_class, line_num);
        ensure_class_stub(classes_map, &rel.to_class, line_num);
        relations.push(rel);
        return true;
    }

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
            merge_or_push_class(classes_map, temp_class);
            return true;
        }
    }
    false
}

/// Merges a parsed [`ClassDefAst`] into an existing class stub or pushes a new entry.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-MERGE-OR-PUSH-CLASS]
fn merge_or_push_class(classes: &mut Vec<ClassDefAst>, new_class: ClassDefAst) {
    if let Some(existing) = classes.iter_mut().find(|c| c.name == new_class.name) {
        existing.annotations.extend(new_class.annotations);
        existing.attributes.extend(new_class.attributes);
        existing.operations.extend(new_class.operations);
    } else {
        classes.push(new_class);
    }
}

/// Ensures a class stub exists in `classes` for `name`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-ENSURE-CLASS-STUB]
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

/// Parses an attribute or operation signature line inside a `classDiagram` class block.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-MEMBER-LINE]
fn parse_class_member_line(line: &str, class_def: &mut ClassDefAst, line_num: usize) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    let first_char = trimmed.chars().next().unwrap_or(' ');
    let visibility = Visibility::from_char(first_char);
    let after_vis = if visibility.is_some() { trimmed[1..].trim() } else { trimmed };

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
            let (op_name, return_type) = match (words.first(), words.get(1)) {
                (Some(&w0), Some(&w1)) => (w1.to_string(), Some(w0.to_string())),
                (Some(&w0), None) => {
                    let ret =
                        if !after_close.is_empty() { Some(after_close.to_string()) } else { None };
                    (w0.to_string(), ret)
                }
                _ => ("anonymous_operation".to_string(), None),
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

    let words: Vec<&str> = after_vis.split_whitespace().collect();
    let (attr_name, type_name) = match (words.first(), words.get(1)) {
        (Some(&w0), Some(&w1)) => (w1.to_string(), Some(w0.to_string())),
        (Some(&w0), None) => (w0.to_string(), None),
        _ => ("anonymous_attr".to_string(), None),
    };

    class_def.attributes.push(ClassAttributeAst {
        visibility,
        name: attr_name,
        type_name,
        raw: trimmed.to_string(),
        line: line_num,
    });
}

/// Parses a class relation line (`ClassA <|-- ClassB : label`) into [`ClassRelationAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-CLASS-RELATION]
fn parse_class_relation(line: &str, line_num: usize) -> Option<ClassRelationAst> {
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

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Feature Mermaid Class Diagram and Operation Bullet Codecs
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
//! Synthesizes Check 13B compliant Mermaid `classDiagram` blocks and bijective `ActionDef`
//! operation signature and bullet codecs under ISO/IEC/IEEE 15288:2023.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PartDef`, `ActionDef`, or Markdown bullet slices are valid UTF-8.
//! - **Postconditions**: Emits Mermaid class blocks and bijective Markdown operation bullets.
//! - **Invariants**: Operation signatures emit zero secondary colons or curly braces in Mermaid members.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Malformed Markdown operation bullets return `None` fail-closed without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N)` formatting and parsing over member slices and bullet lines.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `feature_emitter`, `epic_diagrams`, and `reverse::extractor`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by forward/reverse round-trip unit tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]

use super::*;

/// Appends a single `PartDef` Mermaid `class` declaration (ports, attributes, operations, and actions) to `lines`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
pub(crate) fn append_part_class_block(lines: &mut Vec<String>, part: &PartDef) {
    let has_members = !part.ports.is_empty()
        || !part.attributes.is_empty()
        || !part.operations.is_empty()
        || !part.actions.is_empty();
    if has_members {
        lines.push(format!("    class {} {{", part.name));
        for port in &part.ports {
            let dir_prefix = if port.direction == "in" {
                "in"
            } else if port.direction == "out" {
                "out"
            } else {
                "inout"
            };
            let p_type = if port.type_name.is_empty() {
                "Port"
            } else {
                &port.type_name
            };
            lines.push(format!(
                "        +{} {} ({})",
                p_type, port.name, dir_prefix
            ));
        }
        for attr in &part.attributes {
            let a_type = if attr.type_name.is_empty() {
                "String"
            } else {
                &attr.type_name
            };
            lines.push(format!("        +{} {}", a_type, attr.name));
        }
        let mut emitted_ops = BTreeSet::new();
        for op in &part.operations {
            let ret = op.return_type.as_deref().unwrap_or("void");
            if emitted_ops.insert(op.name.as_str()) {
                lines.push(format!("        +{}() {}", op.name, ret));
            }
        }
        for act in &part.actions {
            if emitted_ops.insert(act.name.as_str()) {
                lines.push(format!(
                    "        {}",
                    format_mermaid_operation_signature(act)
                ));
            }
        }
        lines.push("    }".to_string());
    } else {
        lines.push(format!("    class {}", part.name));
    }
}

/// Generates a Check 13B compliant Mermaid `classDiagram` lowered from `PartDef` members and subparts.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-CLASS-DIAGRAM]
pub(crate) fn generate_class_diagram(part: &PartDef) -> String {
    let mut lines = vec!["classDiagram".to_string()];
    append_part_class_block(&mut lines, part);
    for subpart in &part.parts {
        let target_type = subpart.type_name.as_deref().unwrap_or(&subpart.name);
        lines.push(format!(
            "    {} --> {} : {}",
            part.name, target_type, subpart.name
        ));
    }
    lines.join("\n")
}

/// Formats an `ActionDef` as a Mermaid `classDiagram` operation signature preserving `in_params` and `out_params`.
/// Realises: [REQ-0604]
pub fn format_mermaid_operation_signature(action: &ActionDef) -> String {
    let mut params = Vec::new();
    for p in &action.in_params {
        params.push(format!("in_{}_{}", p.name, p.type_name));
    }
    for p in &action.out_params {
        params.push(format!("out_{}_{}", p.name, p.type_name));
    }
    let ret = action
        .out_params
        .first()
        .map(|p| p.type_name.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or("void");
    format!(
        "+{name}({sig}) {ret}",
        name = action.name,
        sig = params.join(", ")
    )
}

/// Formats an `ActionDef` as a Feature Markdown operation bullet preserving `in_params` and `out_params`.
/// Realises: [REQ-0604]
pub fn format_markdown_operation_bullet(action: &ActionDef) -> String {
    let mut params = Vec::new();
    for p in &action.in_params {
        params.push(format!("in {}: {}", p.name, p.type_name));
    }
    for p in &action.out_params {
        params.push(format!("out {}: {}", p.name, p.type_name));
    }
    let ret = action
        .out_params
        .first()
        .map(|p| p.type_name.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or("void");
    format!(
        "- `+{name}({sig}) : {ret}`",
        name = action.name,
        sig = params.join(", ")
    )
}

/// Parses one comma-delimited `[in|out] name: Type` parameter token into `in_params` or `out_params`.
/// Realises: [REQ-0604]
fn parse_operation_bullet_parameter(
    raw_param: &str,
    in_params: &mut Vec<AttributeDef>,
    out_params: &mut Vec<AttributeDef>,
) -> Option<()> {
    let p_trim = raw_param.trim();
    if p_trim.is_empty() {
        return None;
    }
    let (dir, rest) = if let Some(r) = p_trim.strip_prefix("in ") {
        ("in", r.trim())
    } else if let Some(r) = p_trim.strip_prefix("out ") {
        ("out", r.trim())
    } else {
        ("in", p_trim)
    };
    let (p_name, p_type) = rest.split_once(':')?;
    let p_name = p_name.trim();
    let p_type = p_type.trim();
    if p_name.is_empty() || p_type.is_empty() {
        return None;
    }
    let attr = AttributeDef {
        name: p_name.to_string(),
        type_name: p_type.to_string(),
        ..Default::default()
    };
    if dir == "out" {
        out_params.push(attr);
    } else {
        in_params.push(attr);
    }
    Some(())
}

/// Parses a Feature Markdown operation bullet (`` - `+ActionName(in p: T, out q: U) : ReturnType` ``) back into an `ActionDef`.
/// Realises: [REQ-0604]
pub fn parse_markdown_operation_bullet(line: &str) -> Option<ActionDef> {
    let trimmed = line
        .trim()
        .trim_start_matches('-')
        .trim()
        .trim_matches('`')
        .trim();
    let after_plus = trimmed.strip_prefix('+')?;
    let open_paren = after_plus.find('(')?;
    let close_paren = after_plus.find(')')?;
    if close_paren < open_paren {
        return None;
    }
    let name = after_plus.get(..open_paren)?.trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    let params_str = after_plus
        .get(open_paren.saturating_add(1)..close_paren)?
        .trim();
    let after_close = after_plus.get(close_paren.saturating_add(1)..)?.trim();
    let ret_str = after_close.strip_prefix(':')?.trim();
    if ret_str.is_empty() {
        return None;
    }

    let mut in_params = Vec::new();
    let mut out_params = Vec::new();
    if !params_str.is_empty() {
        for raw_param in params_str.split(',') {
            parse_operation_bullet_parameter(raw_param, &mut in_params, &mut out_params)?;
        }
    }
    if out_params.is_empty() && ret_str != "void" {
        out_params.push(AttributeDef {
            name: "result".to_string(),
            type_name: ret_str.to_string(),
            ..Default::default()
        });
    }

    Some(ActionDef {
        name: name.to_string(),
        in_params,
        out_params,
        ..Default::default()
    })
}

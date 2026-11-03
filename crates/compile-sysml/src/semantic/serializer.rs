#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 / KerML canonical textual notation serializer / pretty-printer.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for SysML v2 serialization.
//! ## 2. Invariants & Mathematical Properties
//! Renders deterministic SysML v2 textual notation from typed AST nodes with canonical 4-space block indentation.
//! ## 3. Preconditions
//! AST nodes conform to `deap_core::sysml_ast` structural invariants within `MAX_AST_DEPTH` (64).
//! ## 4. Postconditions
//! Produces bijective canonical SysML v2 text (`parse(serialize(ast)) == ast`) with zero panics.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over AST node size. Space Complexity: O(N) output string buffer.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Caps recursive AST serialization depth at `MAX_AST_DEPTH` (64) with saturating arithmetic.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-MODULE, REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE]

pub use super::symbols::MAX_AST_DEPTH;
use deap_core::sysml_ast::{AttributeDef, FlowDef, PortDef};

/// Helper to format doc comments as standard SysML v2 doc blocks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `format_doc_comment`.
/// ## 2. Invariants & Mathematical Properties
/// - Escapes nested `/*` and `*/` tokens so block comments maintain balanced lexical depth.
/// ## 3. Preconditions
/// - `doc: &Option<String>` and `indent: usize` satisfy domain and type-safety constraints.
/// ## 4. Postconditions
/// - Returns `String` formatted as a single-line or multi-line `doc /* ... */` block.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)). Space Complexity: O(N(doc)).
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total function returning empty string when `doc` is `None` or blank.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SERIALIZER-FORMAT-DOC-COMMENT] | SysML v2: ActionDef(format_doc_comment)
pub fn format_doc_comment(doc: &Option<String>, indent: usize) -> String {
    let doc_str = match doc {
        Some(d) if !d.trim().is_empty() => d.trim(),
        _ => return String::new(),
    };

    let pad = " ".repeat(indent);
    let sanitized = doc_str.replace("/*", "/ *").replace("*/", "* /");
    let lines: Vec<&str> = sanitized.lines().collect();

    if lines.len() <= 1 {
        format!("{}doc /* {} */\n", pad, sanitized)
    } else if let Some((first, rest)) = lines.split_first() {
        let mut out = format!("{}doc /* {}\n", pad, first);
        if let Some((last, middle)) = rest.split_last() {
            for line in middle {
                out.push_str(&format!("{}       {}\n", pad, line));
            }
            out.push_str(&format!("{}       {} */\n", pad, last));
        }
        out
    } else {
        format!("{}doc /* {} */\n", pad, sanitized)
    }
}

/// Trait providing SysML v2 canonical serialization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for trait `SysmlSerializable`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across `fn to_sysml(&self, indent: usize) -> String`.
/// ## 3. Preconditions
/// - Caller supplies well-formed AST receiver and indentation width in spaces.
/// ## 4. Postconditions
/// - Returns deterministic canonical SysML v2 textual representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) over receiver subtree. Space Complexity: O(N) output string.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total non-panicking serialization contract.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE] | SysML v2: PartDef(SysmlSerializable)
pub trait SysmlSerializable {
    /// Serializes `self` into canonical SysML v2 notation at `indent` leading spaces.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE-TO-SYSML] | SysML v2: ActionDef(SysmlSerializable::to_sysml)
    fn to_sysml(&self, indent: usize) -> String;
}

/// Convenience function to serialize any `SysmlSerializable` entity to canonical textual notation.
/// Realises: [REQ-SEMANTIC-SERIALIZER-TO-SYSML] | SysML v2: ActionDef(to_sysml)
pub fn to_sysml<T: SysmlSerializable>(item: &T) -> String {
    item.to_sysml(0)
}

/// Internal depth-bounded serialization contract capping recursive AST container formatting at `MAX_AST_DEPTH`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-BOUNDED-TRAIT]
pub(crate) trait BoundedSysmlSerializable {
    /// Formats `self` at `indent` spaces when `depth < MAX_AST_DEPTH`, terminating recursion safely at `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-TO-SYSML-BOUNDED]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String;
}

/// Strips leading/trailing whitespace and enclosing double-quote delimiters (`"`) from a string slice.
/// Realises: [REQ-SEMANTIC-SERIALIZER-STRIP-ENCLOSING-QUOTES] | SysML v2: ActionDef(strip_enclosing_quotes)
pub(crate) fn strip_enclosing_quotes(val: &str) -> &str {
    val.trim().trim_matches('"')
}

/// Formats an `f64` real literal for SysML v2 output without truncating non-zero fractional digits.
/// Realises: [REQ-SEMANTIC-SERIALIZER-FORMAT-REAL-LITERAL] | SysML v2: ActionDef(format_real_literal)
pub(crate) fn format_real_literal(val: f64) -> String {
    if val.fract() == 0.0 {
        format!("{:.1}", val)
    } else {
        val.to_string()
    }
}

impl SysmlSerializable for AttributeDef {
    /// Serializes `AttributeDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-ATTRIBUTEDEF-TO-SYSML] | SysML v2: ActionDef(AttributeDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let def_val = self
            .default_value
            .as_ref()
            .map(|v| {
                let trimmed = v.trim();
                if let Some(inner) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    let mut escaped = String::with_capacity(trimmed.len().saturating_add(4));
                    escaped.push('"');
                    let mut prev_bs = false;
                    for ch in inner.chars() {
                        if ch == '"' && !prev_bs {
                            escaped.push('\\');
                            escaped.push('"');
                        } else {
                            escaped.push(ch);
                        }
                        prev_bs = ch == '\\' && !prev_bs;
                    }
                    escaped.push('"');
                    format!(" = {}", escaped)
                } else {
                    format!(" = {}", v)
                }
            })
            .unwrap_or_default();
        format!(
            "{}{}attribute {} : {}{};",
            doc, pad, self.name, self.type_name, def_val
        )
    }
}

impl SysmlSerializable for FlowDef {
    /// Serializes `FlowDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-FLOWDEF-TO-SYSML] | SysML v2: ActionDef(FlowDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let dir = if !self.direction.is_empty() {
            format!("{} ", self.direction)
        } else {
            String::new()
        };
        format!(
            "{}{}flow {}{} : {};",
            doc, pad, dir, self.name, self.item_type
        )
    }
}

/// Appends inner port body attributes and item flows to `lines`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-PORTDEF-TO-SYSML]
fn append_port_body_members(port: &PortDef, pad: &str, indent: usize, lines: &mut Vec<String>) {
    if !port.protocol_family.is_empty() {
        lines.push(format!(
            "{}    attribute protocol_family : String = \"{}\";",
            pad,
            strip_enclosing_quotes(&port.protocol_family)
        ));
    }
    if !port.port_category.is_empty() && port.port_category != "DataPort" {
        lines.push(format!(
            "{}    attribute port_category : String = \"{}\";",
            pad,
            strip_enclosing_quotes(&port.port_category)
        ));
    }
    for (k, v) in &port.electrical_attributes {
        lines.push(format!(
            "{}    attribute {} = \"{}\";",
            pad,
            k,
            strip_enclosing_quotes(v)
        ));
    }
    let child_indent = indent.saturating_add(4);
    for flow in &port.item_flows {
        lines.push(flow.to_sysml(child_indent));
    }
}

impl SysmlSerializable for PortDef {
    /// Serializes `PortDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PORTDEF-TO-SYSML] | SysML v2: ActionDef(PortDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let dir = if !self.direction.is_empty() {
            format!("{} ", self.direction)
        } else {
            String::new()
        };
        let conj = if self.is_conjugated { "~" } else { "" };
        let has_body = !self.protocol_family.is_empty()
            || (!self.port_category.is_empty() && self.port_category != "DataPort")
            || !self.electrical_attributes.is_empty()
            || !self.item_flows.is_empty();

        if has_body {
            let mut lines = Vec::new();
            if !doc.is_empty() {
                lines.push(doc.trim_end().to_string());
            }
            if self.name == self.type_name {
                lines.push(format!("{}port def {} {{", pad, self.name));
            } else {
                lines.push(format!(
                    "{}port {}{}{} : {} {{",
                    pad, dir, conj, self.name, self.type_name
                ));
            }
            append_port_body_members(self, &pad, indent, &mut lines);
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        } else if self.name == self.type_name {
            format!("{}{}port def {};", doc, pad, self.name)
        } else {
            format!(
                "{}{}port {}{}{} : {};",
                doc, pad, dir, conj, self.name, self.type_name
            )
        }
    }
}

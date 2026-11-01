//! Logical interface bindings, typed attribute parsers, and depth-bounded structural part AST extractors for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-INTERFACES-AND-ITEMS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `InterfaceDef` for subsystem `AstSlice` projections and provides zero-regex `parse_attribute_defs` and depth-bounded `parse_part_defs` structural extractors with nested block isolation.
//! Bounds & Faults: All character scans use bounds-checked `.get` slicing and saturating offset arithmetic; nested `PartDef` traversal is strictly bounded by `MAX_AST_DEPTH = 64` with zero panics or unbounded call-stack growth.

use super::identifiers::{
    extract_doc_comment_before, extract_ident, extract_type_ident, find_matching_brace,
    is_word_boundary_after, is_word_boundary_before, mask_string_literals, skip_ws, AttributeDef,
};
use super::{
    parse_action_defs, parse_calc_defs, parse_connection_defs, parse_constraint_defs,
    parse_hazard_defs, parse_port_defs, parse_requirement_defs, parse_safety_and_layer_annotations,
    parse_satisfied_requirements, parse_state_defs, strip_sysml_comments, PartDef,
};
use serde::{Deserialize, Serialize};

/// Maximum permitted recursive `PartDef` nesting depth per DEAP Rust Coding Standard Section 5.
/// Realises: [REQ-SYSML-AST-MAX-DEPTH-BOUND]
const MAX_AST_DEPTH: usize = 64;

/// Represents a logical interface connection binding between a source port and a target port in an AST slice.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-INTERFACEDEF]
/// Contract: Preserves logical interface `name`, `source_port`, `target_port`, and optional `doc` metadata for token-budgeted subsystem `AstSlice` projections.
/// Bounds & Faults: O(1) structural value record supporting deterministic `Eq` deduplication and `serde` round-trip serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InterfaceDef {
    pub name: String,
    pub source_port: String,
    pub target_port: String,
    #[serde(default)]
    pub doc: Option<String>,
}

/// Parses an optional `: <Type>` annotation and `:= <val>;` / `= <val>;` / `;` tail of an attribute statement.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ATTRIBUTE-DEFS]
fn parse_attribute_type_and_value(
    clean: &str,
    masked: &str,
    start: usize,
) -> Option<(String, Option<String>, usize)> {
    let mut curr = start;
    let mut type_name = "String".to_string();
    let mut has_type = false;

    let rem_colon = masked.get(curr..).unwrap_or("");
    if rem_colon.starts_with(':') && !rem_colon.starts_with(":=") {
        curr = curr.saturating_add(1);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        let (t_name, type_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
        if t_name.is_empty() {
            return None;
        }
        type_name = t_name.to_string();
        has_type = true;
        curr = curr.saturating_add(type_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }

    let rem_assign = masked.get(curr..).unwrap_or("");
    if rem_assign.starts_with(":=") || rem_assign.starts_with('=') {
        let op_len = if rem_assign.starts_with(":=") { 2 } else { 1 };
        curr = curr.saturating_add(op_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        let semi = masked.get(curr..).and_then(|s| s.find(';'))?;
        let val_end = curr.saturating_add(semi);
        let val = clean.get(curr..val_end).unwrap_or("").trim().to_string();
        Some((type_name, Some(val), val_end.saturating_add(1)))
    } else if has_type && rem_assign.starts_with(';') {
        Some((type_name, None, curr.saturating_add(1)))
    } else {
        None
    }
}

/// Parses a single `attribute [def] <name> ...;` declaration beginning at byte offset `kw_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ATTRIBUTE-DEFS]
fn parse_single_attribute_after_keyword(
    clean: &str,
    masked: &str,
    kw_pos: usize,
) -> Option<(AttributeDef, usize)> {
    let doc = extract_doc_comment_before(clean, kw_pos);
    let mut curr = kw_pos.saturating_add(9);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let (type_name, default_value, next_idx) = parse_attribute_type_and_value(clean, masked, curr)?;
    Some((
        AttributeDef {
            name: name.to_string(),
            type_name,
            default_value,
            doc,
        },
        next_idx,
    ))
}

/// Parses `AttributeDef` declarations and valued property assignments from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ATTRIBUTE-DEFS]
/// Contract: Extracts every `attribute` and `attribute def` declaration with an explicit `: <Type>` annotation and/or `=` / `:=` default value in appearance order.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using checked `.get` slicing; malformed statements are skipped without panicking.
pub fn parse_attribute_defs(content: &str) -> Vec<AttributeDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut attrs = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("attribute")) {
        let kw_pos = idx.saturating_add(pos);
        let after_kw = kw_pos.saturating_add(9);
        if !is_word_boundary_before(&masked, kw_pos) || !is_word_boundary_after(&masked, after_kw) {
            idx = after_kw;
            continue;
        }
        if let Some((attr, next_idx)) =
            parse_single_attribute_after_keyword(&clean, &masked, kw_pos)
        {
            attrs.push(attr);
            idx = next_idx;
        } else {
            idx = after_kw;
        }
    }
    attrs
}

/// Specifies which nested SysML v2 block categories are masked out when scanning shallow part bodies.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-NESTEDBLOCKMASKMODE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NestedBlockMaskMode {
    /// Masks nested `part` and `requirement` blocks so their members are not attributed to the parent part.
    PartAndRequirement,
    /// Masks nested `port` blocks so port-internal attributes are not attributed to the parent part.
    PortOnly,
}

/// Matches the byte length of a nested block keyword at `idx` when permitted by `mode`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-NESTED-BLOCKS]
fn match_nested_block_keyword(
    masked: &str,
    idx: usize,
    mode: NestedBlockMaskMode,
) -> Option<usize> {
    if !is_word_boundary_before(masked, idx) {
        return None;
    }
    let rem = masked.get(idx..)?;
    match mode {
        NestedBlockMaskMode::PartAndRequirement => {
            if rem.starts_with("part") && is_word_boundary_after(masked, idx.saturating_add(4)) {
                Some(4)
            } else if rem.starts_with("requirement")
                && is_word_boundary_after(masked, idx.saturating_add(11))
            {
                Some(11)
            } else {
                None
            }
        }
        NestedBlockMaskMode::PortOnly => {
            if rem.starts_with("port") && is_word_boundary_after(masked, idx.saturating_add(4)) {
                Some(4)
            } else {
                None
            }
        }
    }
}

/// Advances past modifiers, identifier, and optional type annotation to locate an opening `{` byte offset.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-NESTED-BLOCKS]
fn advance_to_nested_block_open_brace(masked: &str, after_kw: usize) -> Option<usize> {
    let mut curr = after_kw;
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    for dir_kw in ["inout", "out", "in"] {
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with(dir_kw) && is_word_boundary_after(s, dir_kw.len()))
        {
            curr = curr.saturating_add(dir_kw.len());
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
            break;
        }
    }
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
        curr = curr.saturating_add(1);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let rem_colon = masked.get(curr..).unwrap_or("");
    if rem_colon.starts_with(":>") || rem_colon.starts_with(':') {
        let sep_len = if rem_colon.starts_with(":>") { 2 } else { 1 };
        curr = curr.saturating_add(sep_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
            curr = curr.saturating_add(1);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
        let (_, t_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
        curr = curr.saturating_add(t_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }

    if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
        Some(curr)
    } else {
        None
    }
}

/// Masks brace-enclosed bodies of nested blocks matching `mode` with spaces while preserving byte offsets and newlines.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-NESTED-BLOCKS]
fn mask_nested_blocks(body: &str, mode: NestedBlockMaskMode) -> String {
    let masked = mask_string_literals(body);
    let mut out = body.as_bytes().to_vec();
    let mut idx: usize = 0;

    while idx < masked.len() {
        let Some(kw_len) = match_nested_block_keyword(&masked, idx, mode) else {
            let rem = masked.get(idx..).unwrap_or("");
            let step = rem.chars().next().map(char::len_utf8).unwrap_or(1);
            idx = idx.saturating_add(step);
            continue;
        };

        let after_kw = idx.saturating_add(kw_len);
        if let Some(brace_pos) = advance_to_nested_block_open_brace(&masked, after_kw) {
            if let Some(close_pos) = find_matching_brace(body, brace_pos) {
                if let Some(slice) = out.get_mut(brace_pos.saturating_add(1)..close_pos) {
                    for byte in slice {
                        if *byte != b'\n' {
                            *byte = b' ';
                        }
                    }
                }
                idx = close_pos.saturating_add(1);
                continue;
            }
        }
        idx = after_kw;
    }
    String::from_utf8(out).unwrap_or_else(|_| body.to_string())
}

/// Parsed header and raw inner body slice for a single SysML v2 `part` or `part def` declaration.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PART-DEFS]
struct ParsedPartHeader<'a> {
    name: &'a str,
    doc: Option<String>,
    is_def: bool,
    type_name: Option<String>,
    body: &'a str,
    next_idx: usize,
}

/// Parses a `part [def] <name> [: <Type>] ({ ... } | ;)` header starting at byte offset `part_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PART-DEFS]
fn parse_part_header_and_body<'a>(
    clean: &'a str,
    masked: &'a str,
    part_pos: usize,
) -> Option<ParsedPartHeader<'a>> {
    let doc = extract_doc_comment_before(clean, part_pos);
    let mut curr = part_pos.saturating_add(4);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let mut is_def = false;
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        is_def = true;
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }

    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let mut type_name = None;
    let rem_colon = masked.get(curr..).unwrap_or("");
    if rem_colon.starts_with(":>") || rem_colon.starts_with(':') {
        let sep_len = if rem_colon.starts_with(":>") { 2 } else { 1 };
        curr = curr.saturating_add(sep_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        let (t_name, t_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
        if !t_name.is_empty() {
            type_name = Some(t_name.to_string());
            curr = curr.saturating_add(t_len);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
    }

    let rem_body = masked.get(curr..).unwrap_or("");
    let (body, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr);
        let inner = match close_pos {
            Some(cp) => clean.get(curr.saturating_add(1)..cp).unwrap_or(""),
            None => "",
        };
        let next = match close_pos {
            Some(cp) => cp.saturating_add(1),
            None => curr.saturating_add(1),
        };
        (inner, next)
    } else if rem_body.starts_with(';') {
        ("", curr.saturating_add(1))
    } else {
        return None;
    };

    Some(ParsedPartHeader {
        name,
        doc,
        is_def,
        type_name,
        body,
        next_idx,
    })
}

/// Populates a `PartDef` and its shallow and depth-bounded nested members from a `ParsedPartHeader`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PART-DEFS]
fn build_part_def_from_header(header: ParsedPartHeader<'_>, depth: usize) -> PartDef {
    let shallow_body = mask_nested_blocks(header.body, NestedBlockMaskMode::PartAndRequirement);
    let attr_body = mask_nested_blocks(&shallow_body, NestedBlockMaskMode::PortOnly);

    let attributes = parse_attribute_defs(&attr_body);
    let ports = parse_port_defs(&shallow_body);
    let actions = parse_action_defs(&shallow_body);
    let constraints = parse_constraint_defs(&shallow_body);
    let requirements = parse_requirement_defs(header.body);
    let satisfied_requirements = parse_satisfied_requirements(&shallow_body);
    let connections = parse_connection_defs(&shallow_body);
    let states = parse_state_defs(&shallow_body);
    let calcs = parse_calc_defs(&shallow_body);
    let hazards = parse_hazard_defs(&shallow_body);
    let (triggers_hazards, safety_realises, layer) =
        parse_safety_and_layer_annotations(&shallow_body);
    let nested_parts = if header.body.is_empty() {
        Vec::new()
    } else {
        parse_part_defs_at_depth(header.body, depth.saturating_add(1))
    };

    PartDef {
        name: header.name.to_string(),
        doc: header.doc,
        is_def: header.is_def,
        type_name: header.type_name,
        attributes,
        ports,
        actions,
        constraints,
        requirements,
        satisfied_requirements,
        parts: nested_parts,
        operations: Vec::new(),
        capabilities: Vec::new(),
        interactions: Vec::new(),
        test_cases: Vec::new(),
        states,
        use_cases: Vec::new(),
        item_defs: Vec::new(),
        hazards,
        risks: Vec::new(),
        connections,
        calcs,
        triggers_hazards,
        safety_realises,
        layer,
    }
}

/// Depth-bounded helper extracting `PartDef` declarations up to `MAX_AST_DEPTH`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PART-DEFS]
fn parse_part_defs_at_depth(content: &str, depth: usize) -> Vec<PartDef> {
    if depth > MAX_AST_DEPTH {
        return Vec::new();
    }
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut parts = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("part")) {
        let part_pos = idx.saturating_add(pos);
        let after_part = part_pos.saturating_add(4);
        if !is_word_boundary_before(&masked, part_pos)
            || !is_word_boundary_after(&masked, after_part)
        {
            idx = after_part;
            continue;
        }
        if let Some(header) = parse_part_header_and_body(&clean, &masked, part_pos) {
            idx = header.next_idx;
            parts.push(build_part_def_from_header(header, depth));
        } else {
            idx = after_part;
        }
    }
    parts
}

/// Parses structural `PartDef` definitions and nested part usages from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PART-DEFS]
/// Contract: Extracts every `part def` and `part` usage in appearance order, isolating shallow attributes/ports/connections from nested sub-parts and bounding nested traversal to `MAX_AST_DEPTH = 64`.
/// Bounds & Faults: Executes in O(N * D) time where `D <= 64` using checked `.get` slicing and saturating arithmetic; exceeding `MAX_AST_DEPTH` stops further descent cleanly without panicking.
pub fn parse_part_defs(content: &str) -> Vec<PartDef> {
    parse_part_defs_at_depth(content, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `InterfaceDef` serde round-trip and `Eq` structural comparison.
    #[test]
    fn interface_def_roundtrips_json_with_optional_doc() -> Result<(), serde_json::Error> {
        let iface = InterfaceDef {
            name: "NavToFlightCtrl".to_string(),
            source_port: "nav.state_out".to_string(),
            target_port: "fcu.state_in".to_string(),
            doc: Some("Deterministic state bus".to_string()),
        };
        let encoded = serde_json::to_string(&iface)?;
        let decoded: InterfaceDef = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, iface);

        let minimal: InterfaceDef =
            serde_json::from_str(r#"{"name":"Bus","source_port":"a","target_port":"b"}"#)?;
        assert!(minimal.doc.is_none());
        Ok(())
    }

    /// Verifies `parse_attribute_defs` across typed, `:=` assigned, `=` untyped, and malformed statements.
    #[test]
    fn parse_attribute_defs_handles_typed_assigned_and_malformed_variants(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            doc /* Nominal bus voltage */
            attribute def BusVoltage : ScalarValues::Real = 28.0;
            attribute cycleCount := 100;
            attribute statusLabel : String;
            // Untyped without assignment or missing semicolon must be skipped cleanly
            attribute bareUntyped;
            attribute missingColonType : = 5;
        "#;
        let attrs = parse_attribute_defs(src);
        assert_eq!(attrs.len(), 3);

        let a0 = attrs.first().ok_or("missing BusVoltage")?;
        assert_eq!(a0.name, "BusVoltage");
        assert_eq!(a0.type_name, "ScalarValues::Real");
        assert_eq!(a0.default_value.as_deref(), Some("28.0"));
        assert_eq!(a0.doc.as_deref(), Some("Nominal bus voltage"));

        let a1 = attrs.get(1).ok_or("missing cycleCount")?;
        assert_eq!(a1.name, "cycleCount");
        assert_eq!(a1.type_name, "String");
        assert_eq!(a1.default_value.as_deref(), Some("100"));

        let a2 = attrs.get(2).ok_or("missing statusLabel")?;
        assert_eq!(a2.name, "statusLabel");
        assert_eq!(a2.type_name, "String");
        assert!(a2.default_value.is_none());
        Ok(())
    }

    /// Verifies `parse_part_defs` isolates port-internal and child-part attributes from parent parts.
    #[test]
    fn parse_part_defs_isolates_nested_port_and_subpart_attributes(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            part def FlightComputer {
                attribute parentRateHz : Real = 200.0;
                in port sensor_in : SensorPort {
                    attribute protocol_family : String = "SPI";
                    attribute portInternalGain : Real = 1.5;
                }
                part watchdog : WatchdogTimer {
                    attribute childTimeoutMs : Integer = 25;
                }
            }
        "#;
        let parts = parse_part_defs(src);
        assert_eq!(parts.len(), 1);
        let parent = parts.first().ok_or("missing FlightComputer")?;
        assert_eq!(parent.attributes.len(), 1);
        assert_eq!(
            parent.attributes.first().map(|a| a.name.as_str()),
            Some("parentRateHz")
        );
        assert_eq!(parent.ports.len(), 1);
        assert_eq!(parent.parts.len(), 1);
        let child = parent.parts.first().ok_or("missing watchdog")?;
        assert_eq!(child.name, "watchdog");
        assert!(!child.is_def);
        assert_eq!(child.type_name.as_deref(), Some("WatchdogTimer"));
        assert_eq!(child.attributes.len(), 1);
        assert_eq!(
            child.attributes.first().map(|a| a.name.as_str()),
            Some("childTimeoutMs")
        );
        Ok(())
    }

    /// Verifies `parse_part_defs` parses up to `MAX_AST_DEPTH` (64) and stops cleanly at `MAX_AST_DEPTH + 1` (65) and deep inputs.
    #[test]
    fn parse_part_defs_enforces_max_ast_depth_and_plus_one_bounds() -> Result<(), String> {
        let mut nested_sysml = format!("part Leaf_{};", MAX_AST_DEPTH.saturating_add(1));
        for level in (0..=MAX_AST_DEPTH).rev() {
            nested_sysml = format!("part def Node_{level} {{ {nested_sysml} }}");
        }
        let parsed = parse_part_defs(&nested_sysml);
        let mut cursor = parsed.first().ok_or("missing root Node_0")?;
        for expected_level in 0..=MAX_AST_DEPTH {
            assert_eq!(cursor.name, format!("Node_{expected_level}"));
            if expected_level < MAX_AST_DEPTH {
                cursor = cursor
                    .parts
                    .first()
                    .ok_or_else(|| format!("missing child below Node_{expected_level}"))?;
            }
        }
        assert!(
            cursor.parts.is_empty(),
            "Child at depth MAX_AST_DEPTH + 1 must be truncated cleanly without panicking"
        );

        // Deep 160-level input completes deterministically without stack overflow.
        let mut very_deep = String::from("part DeepLeaf;");
        for level in (0..160_usize).rev() {
            very_deep = format!("part def Deep_{level} {{ {very_deep} }}");
        }
        let deep_parsed = parse_part_defs(&very_deep);
        assert_eq!(deep_parsed.len(), 1);
        Ok(())
    }
}

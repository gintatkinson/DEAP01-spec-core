//! Foundational SysML v2 identifier, span, directionality, and attribute literal primitives.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-IDENTIFIERS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Provides deterministic, zero-regex lexical identifier classifiers, directionality and cardinality defaults, `AttributeDef` and `FlowDef` value records, annotation/metrology literal extractors, and offset-preserving brace and string-masking scanners.
//! Bounds & Faults: All routines execute in linear O(N) or constant O(1) time with checked `.get` indexing, saturating offset arithmetic, and zero panics on malformed or multi-byte UTF-8 inputs.

use serde::{Deserialize, Serialize};

/// Returns `true` as the default boolean definition flag for SysML v2 AST nodes.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-TRUE]
pub(super) fn default_true() -> bool {
    true
}

/// Returns `1` as the default baseline severity or multiplicity count for SysML v2 AST nodes.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-ONE]
pub(super) fn default_one() -> i32 {
    1
}

/// Returns `"out"` as the default flow direction for `FlowDef` deserialization.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-DIRECTION-OUT]
fn default_direction_out() -> String {
    "out".to_string()
}

/// Returns `"Item"` as the default payload item type for `FlowDef` deserialization.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-ITEM-TYPE]
fn default_item_type() -> String {
    "Item".to_string()
}

/// Returns `"inout"` as the default bidirectional port and operation direction in SysML v2.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-DIRECTION-INOUT]
pub(super) fn default_direction_inout() -> String {
    "inout".to_string()
}

/// Returns `"DataPort"` as the default logical port category in SysML v2.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-PORT-CATEGORY]
pub(super) fn default_port_category() -> String {
    "DataPort".to_string()
}

/// Represents a typed attribute definition or valued property instance in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-ATTRIBUTEDEF]
/// Contract: Captures the canonical identifier `name`, resolved `type_name`, optional literal `default_value`, and attached `doc` comment of a SysML v2 attribute declaration.
/// Bounds & Faults: O(1) structural record construction and deterministic `serde` round-trip serialization with zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeDef {
    pub name: String,
    #[serde(default)]
    pub type_name: String,
    pub default_value: Option<String>,
    pub doc: Option<String>,
}

/// Represents a directed item flow or discrete signal payload specification on a SysML v2 port.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-FLOWDEF]
/// Contract: Preserves item flow `name`, `direction` (defaulting to `"out"`), `item_type` (defaulting to `"Item"`), optional `rate_hz`, physical `unit`, `valid_range`, and `default_value`.
/// Bounds & Faults: O(1) structural record representation with deterministic `serde` defaults when optional wire fields are omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FlowDef {
    pub name: String,
    #[serde(default = "default_direction_out")]
    pub direction: String,
    #[serde(default = "default_item_type")]
    pub item_type: String,
    pub doc: Option<String>,
    pub rate_hz: Option<f64>,
    pub unit: Option<String>,
    pub valid_range: Option<String>,
    pub default_value: Option<String>,
}

/// Classifies whether an ASCII byte belongs to a SysML v2 identifier (`[A-Za-z0-9_]`).
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/IS-IDENT-CHAR]
pub(super) fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Checks whether byte offset `pos` in `text` is preceded by start-of-input or a non-identifier byte.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/WORD-BOUNDARY-BEFORE]
pub(super) fn is_word_boundary_before(text: &str, pos: usize) -> bool {
    match pos.checked_sub(1) {
        None => true,
        Some(prev) => !text
            .as_bytes()
            .get(prev)
            .copied()
            .is_some_and(is_ident_char),
    }
}

/// Checks whether byte offset `pos` in `text` is followed by end-of-input or a non-identifier byte.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/WORD-BOUNDARY-AFTER]
pub(super) fn is_word_boundary_after(text: &str, pos: usize) -> bool {
    if pos >= text.len() {
        true
    } else {
        !text.as_bytes().get(pos).copied().is_some_and(is_ident_char)
    }
}

/// Counts contiguous leading ASCII whitespace bytes in `text`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/SKIP-WS]
pub(super) fn skip_ws(text: &str) -> usize {
    text.bytes().take_while(|b| b.is_ascii_whitespace()).count()
}

/// Extracts a leading ASCII identifier slice and its byte length from `text`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-IDENT]
pub(super) fn extract_ident(text: &str) -> (&str, usize) {
    let len = text.bytes().take_while(|&b| is_ident_char(b)).count();
    (text.get(..len).unwrap_or(""), len)
}

/// Extracts a leading SysML v2 qualified type identifier (`Ident` or `Pkg::Ident`) and its byte length.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-TYPE-IDENT]
pub(super) fn extract_type_ident(s: &str) -> (&str, usize) {
    let mut len: usize = 0;
    let bytes = s.as_bytes();
    while let Some(&b) = bytes.get(len) {
        if is_ident_char(b) {
            len = len.saturating_add(1);
        } else if s.get(len..).is_some_and(|rest| rest.starts_with("::")) {
            len = len.saturating_add(2);
        } else {
            break;
        }
    }
    (s.get(..len).unwrap_or(""), len)
}

/// Checks whether a lowercase token matches a reserved schema meta-keyword or primitive scalar name.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/VALID-SUBSYSTEM-IDENT]
fn is_reserved_subsystem_keyword(lower: &str) -> bool {
    matches!(
        lower,
        "true"
            | "false"
            | "null"
            | "none"
            | "na"
            | "string"
            | "int"
            | "integer"
            | "boolean"
            | "float"
            | "description"
            | "version"
            | "name"
            | "id"
            | "type"
            | "subsystem"
            | "component"
            | "part"
            | "module"
            | "parameters"
            | "attributes"
            | "properties"
            | "specs"
            | "metadata"
            | "title"
            | "status"
            | "real"
            | "primary"
            | "secondary"
            | "optional"
            | "required"
            | "level"
            | "segment"
            | "layer"
            | "architecture"
            | "purpose"
            | "function"
            | "item"
            | "flow"
            | "port"
            | "interface"
            | "doc"
            | "package"
            | "def"
    )
}

/// Validates whether `s` is a well-formed, non-reserved identifier for a subsystem, component, or part.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/VALID-SUBSYSTEM-IDENT]
/// Contract: Returns `true` iff `1 <= s.len() <= 128`, `s` starts with an ASCII letter or `_`, contains only `[A-Za-z0-9_]`, and does not match any case-insensitive reserved schema keyword.
/// Bounds & Faults: O(|s|) deterministic lexical check; returns `false` without panicking on empty, oversized, or non-ASCII inputs.
pub fn is_valid_subsystem_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let Some(first) = s.chars().next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    if is_reserved_subsystem_keyword(&s.to_ascii_lowercase()) {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Extracts an optional `doc /* ... */` documentation block immediately preceding `keyword_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-DOC-COMMENT]
pub(super) fn extract_doc_comment_before(text: &str, keyword_pos: usize) -> Option<String> {
    let before = text.get(..keyword_pos)?.trim_end();
    if !before.ends_with("*/") {
        return None;
    }
    let open = before.rfind("/*")?;
    let doc_prefix = before.get(..open)?.trim_end();
    if !doc_prefix.ends_with("doc") {
        return None;
    }
    let doc_start = doc_prefix.len().saturating_sub(3);
    if !is_word_boundary_before(doc_prefix, doc_start) {
        return None;
    }
    let body_start = open.saturating_add(2);
    let body_end = before.len().saturating_sub(2);
    let inner = before.get(body_start..body_end)?;
    Some(inner.trim().to_string())
}

/// Extracts the inner string slice of a leading double-quoted literal `"..."`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-QUOTED-STRING]
pub(super) fn extract_quoted_string(s: &str) -> Option<&str> {
    let rest = s.strip_prefix('"')?;
    let end = rest.find('"')?;
    rest.get(..end)
}

/// Extracts optional SI unit (`[m/s]`) and numeric range bounds (`[min..max]`) from an attribute string.
/// Realises: [REQ-SYSML-AST-EXTRACT-SI-UNIT-BOUNDS]
pub(super) fn extract_si_unit_and_bounds(raw: &str) -> (Option<String>, Option<(f64, f64)>) {
    let mut si_unit = None;
    let mut range_bounds = None;
    let mut cursor: usize = 0;

    while let Some(open_b) = raw.get(cursor..).and_then(|s| s.find('[')) {
        let start = cursor.saturating_add(open_b).saturating_add(1);
        let Some(close_b) = raw.get(start..).and_then(|s| s.find(']')) else {
            break;
        };
        let end = start.saturating_add(close_b);
        if let Some(inside) = raw.get(start..end) {
            let trimmed = inside.trim();
            if let Some((min_s, max_s)) = trimmed.split_once("..") {
                if let (Ok(min_v), Ok(max_v)) =
                    (min_s.trim().parse::<f64>(), max_s.trim().parse::<f64>())
                {
                    range_bounds = Some((min_v, max_v));
                }
            } else if !trimmed.is_empty() && si_unit.is_none() {
                si_unit = Some(trimmed.to_string());
            }
        }
        cursor = end.saturating_add(1);
    }

    (si_unit, range_bounds)
}

/// Lexical state machine states for double-quoted string scanning with backslash escapes.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-STRING-LITERALS]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteScanState {
    Outside,
    InString,
    Escaped,
}

/// Locates the byte offset of the closing `}` matching the block starting at or after `open_pos`.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/FIND-MATCHING-BRACE]
/// Contract: Scans UTF-8 characters from `open_pos`, ignoring braces inside escaped double-quoted string literals, and returns `Some(close_byte_offset)` when brace depth returns to `0`.
/// Bounds & Faults: O(N) single-pass scan with saturating depth counters; returns `None` cleanly on out-of-bounds `open_pos`, leading unmatched `}`, or unclosed braces.
pub fn find_matching_brace(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth: usize = 0;
    let mut state = QuoteScanState::Outside;
    let chars = s.get(open_pos..)?.char_indices();
    for (idx, ch) in chars {
        match state {
            QuoteScanState::Escaped => {
                state = QuoteScanState::InString;
            }
            QuoteScanState::InString => {
                if ch == '\\' {
                    state = QuoteScanState::Escaped;
                } else if ch == '"' {
                    state = QuoteScanState::Outside;
                }
            }
            QuoteScanState::Outside => match ch {
                '"' => state = QuoteScanState::InString,
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    if depth == 0 {
                        return None;
                    }
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(open_pos.saturating_add(idx));
                    }
                }
                _ => {}
            },
        }
    }
    None
}

/// Replaces a `// ...` line comment starting at `start` with spaces while preserving `\n`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-STRING-LITERALS]
fn mask_line_comment_bytes(out: &mut [u8], start: usize) -> usize {
    if let Some(slot) = out.get_mut(start) {
        *slot = b' ';
    }
    let second = start.saturating_add(1);
    if let Some(slot) = out.get_mut(second) {
        *slot = b' ';
    }
    let mut idx = start.saturating_add(2);
    while let Some(&b) = out.get(idx) {
        if b == b'\n' {
            break;
        }
        if let Some(slot) = out.get_mut(idx) {
            *slot = b' ';
        }
        idx = idx.saturating_add(1);
    }
    idx
}

/// Replaces a `/* ... */` block comment starting at `start` with spaces while preserving `\n`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-STRING-LITERALS]
fn mask_block_comment_bytes(out: &mut [u8], start: usize) -> usize {
    if let Some(slot) = out.get_mut(start) {
        *slot = b' ';
    }
    let second = start.saturating_add(1);
    if let Some(slot) = out.get_mut(second) {
        *slot = b' ';
    }
    let mut idx = start.saturating_add(2);
    while let Some(&b) = out.get(idx) {
        let next_idx = idx.saturating_add(1);
        if b == b'*' && out.get(next_idx) == Some(&b'/') {
            if let Some(slot) = out.get_mut(idx) {
                *slot = b' ';
            }
            if let Some(slot) = out.get_mut(next_idx) {
                *slot = b' ';
            }
            return idx.saturating_add(2);
        }
        if b != b'\n' {
            if let Some(slot) = out.get_mut(idx) {
                *slot = b' ';
            }
        }
        idx = next_idx;
    }
    idx
}

/// Masks string literal contents and line/block comments with ASCII spaces while preserving byte offsets.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-STRING-LITERALS]
/// Contract: Guarantees `mask_string_literals(s).len() == s.len()` and preserves every newline byte position so keyword indices in the masked buffer align 1:1 with `s`.
/// Bounds & Faults: O(N) time and O(N) space over `s.len()` using checked `.get` / `.get_mut` byte access with zero panics on unclosed quotes or comments.
pub fn mask_string_literals(s: &str) -> String {
    let mut out = s.as_bytes().to_vec();
    let mut i: usize = 0;
    let mut state = QuoteScanState::Outside;
    while i < out.len() {
        let cur = out.get(i).copied().unwrap_or(0);
        match state {
            QuoteScanState::Escaped => {
                state = QuoteScanState::InString;
                if cur != b'\n' {
                    if let Some(slot) = out.get_mut(i) {
                        *slot = b' ';
                    }
                }
                i = i.saturating_add(1);
            }
            QuoteScanState::InString => {
                if cur == b'\\' {
                    state = QuoteScanState::Escaped;
                    if let Some(slot) = out.get_mut(i) {
                        *slot = b' ';
                    }
                } else if cur == b'"' {
                    state = QuoteScanState::Outside;
                } else if cur != b'\n' {
                    if let Some(slot) = out.get_mut(i) {
                        *slot = b' ';
                    }
                }
                i = i.saturating_add(1);
            }
            QuoteScanState::Outside => {
                let next_i = i.saturating_add(1);
                if cur == b'"' {
                    state = QuoteScanState::InString;
                    i = next_i;
                } else if cur == b'/' && out.get(next_i) == Some(&b'/') {
                    i = mask_line_comment_bytes(&mut out, i);
                } else if cur == b'/' && out.get(next_i) == Some(&b'*') {
                    i = mask_block_comment_bytes(&mut out, i);
                } else {
                    i = next_i;
                }
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies default serde helper functions and `FlowDef` deserialization defaults.
    #[test]
    fn test_directionality_and_flow_serde_defaults() -> Result<(), serde_json::Error> {
        assert!(default_true());
        assert_eq!(default_one(), 1);
        assert_eq!(default_direction_inout(), "inout");
        assert_eq!(default_port_category(), "DataPort");

        let flow: FlowDef = serde_json::from_str(r#"{"name":"telemetry_frame"}"#)?;
        assert_eq!(flow.name, "telemetry_frame");
        assert_eq!(flow.direction, "out");
        assert_eq!(flow.item_type, "Item");
        Ok(())
    }

    /// Verifies identifier extraction, qualified type paths, word boundaries, and subsystem validation.
    #[test]
    fn test_identifier_and_subsystem_validation_boundaries() {
        assert_eq!(extract_ident("SensorNode_01 : Port"), ("SensorNode_01", 13));
        assert_eq!(extract_ident(":NoIdent"), ("", 0));
        assert_eq!(
            extract_type_ident("ScalarValues::Real = 1.0"),
            ("ScalarValues::Real", 18)
        );
        assert!(is_word_boundary_before("part def X", 0));
        assert!(is_word_boundary_before("part def X", 5));
        assert!(!is_word_boundary_before("subpart", 3));
        assert!(is_word_boundary_after("part", 4));
        assert!(is_word_boundary_after("part", 99));

        assert!(is_valid_subsystem_identifier("FlightGuidance_01"));
        assert!(is_valid_subsystem_identifier("_InternalBus"));
        assert!(!is_valid_subsystem_identifier(""));
        assert!(!is_valid_subsystem_identifier("9StartsWithDigit"));
        assert!(!is_valid_subsystem_identifier("Subsystem"));
        assert!(!is_valid_subsystem_identifier("package"));
        assert!(!is_valid_subsystem_identifier("has-hyphen"));

        let max_valid = "A".repeat(128);
        let too_long = "A".repeat(129);
        assert!(is_valid_subsystem_identifier(&max_valid));
        assert!(!is_valid_subsystem_identifier(&too_long));
    }

    /// Verifies doc comment, quoted string, and SI unit/bounds literal extraction.
    #[test]
    fn test_literal_and_si_bounds_extractors() {
        let src = "  doc /* Pitch rate controller. */  part def PitchCtrl";
        let kw_pos = src.find("part").unwrap_or(0);
        assert_eq!(
            extract_doc_comment_before(src, kw_pos).as_deref(),
            Some("Pitch rate controller.")
        );
        assert_eq!(
            extract_doc_comment_before("mydoc /* not kw */ part", 19),
            None
        );
        assert_eq!(extract_doc_comment_before(src, 999), None);

        assert_eq!(extract_quoted_string("\"REQ-01\";"), Some("REQ-01"));
        assert_eq!(extract_quoted_string("\"unclosed"), None);

        let (unit, bounds) = extract_si_unit_and_bounds("85.0 [m/s] [-10.5..350.25]");
        assert_eq!(unit.as_deref(), Some("m/s"));
        assert_eq!(bounds, Some((-10.5, 350.25)));
        assert_eq!(extract_si_unit_and_bounds("unclosed [m/s"), (None, None));
    }

    /// Verifies brace matching and byte-length-preserving string/comment masking on escaped and UTF-8 inputs.
    #[test]
    fn test_brace_matching_and_offset_preserving_masking() {
        let block = r#"part def A { attribute s = "escaped \" } brace"; part b { } }"#;
        let open_pos = block.find('{').unwrap_or(0);
        assert_eq!(find_matching_brace(block, open_pos), Some(block.len() - 1));
        assert_eq!(find_matching_brace("} stray close {", 0), None);
        assert_eq!(find_matching_brace("{ unclosed", 0), None);
        assert_eq!(find_matching_brace("{ ok }", 999), None);

        let raw = "part A {\n  // comment }\n  /* block\n  still comment */\n  attribute label = \"utf8_\\n_\\\"_text\";\n}\n";
        let masked = mask_string_literals(raw);
        assert_eq!(masked.len(), raw.len());
        let raw_newlines: Vec<usize> = raw.match_indices('\n').map(|(i, _)| i).collect();
        let masked_newlines: Vec<usize> = masked.match_indices('\n').map(|(i, _)| i).collect();
        assert_eq!(masked_newlines, raw_newlines);
        assert!(!masked.contains("comment"));
        assert!(!masked.contains("utf8_"));
    }
}

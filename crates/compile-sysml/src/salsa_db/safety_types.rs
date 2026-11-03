//! # CST-Backed Type Normalization & Safety Annotation Target Extraction (`safety_types`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Provides CST-backed type normalization, generic type decomposition, and `@SafetyRealises`/`@TriggersHazard` target extraction (`BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Caps composite-type decomposition at `MAX_AST_DEPTH` (`64`).
//! ## 3. Preconditions -- Input type expressions and requirement prose are valid UTF-8 slices.
//! ## 4. Postconditions -- Produces normalized type names, generic arguments, and `AnnotationSpan` target lists.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(N); Space: O(N).
//! ## 6. Failure Modes & Fault Tolerance -- Zero recursion and zero panics on malformed type or annotation syntax.
//! ## 7. Realization & Traceability -- Realises: [REQ-0603], [REQ-SEMANTIC-VALIDATOR-MODULE] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use deap_core::sysml_cst::lexer::{tokenize_sysml, SysmlToken};
use deap_core::sysml_cst::PackageDef;

/// Maximum AST and composite-type nesting depth permitted during SysML v2 semantic validation. Realises: [REQ-SEMANTIC-VALIDATOR-MAX-AST-DEPTH] | [FLS: fls_66m4rnbssgig]
pub(crate) const MAX_AST_DEPTH: usize = 64;

/// Deterministically normalizes a source-language type expression by stripping reference, pointer, `dyn`, bracket multiplicity, and nullable `?` wrappers.
/// Realises: [REQ-SEMANTIC-VALIDATOR-NORMALIZE-SOURCE-TYPE] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn normalize_source_type(raw: &str) -> &str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "";
    }
    let mut s = trimmed;
    loop {
        let prev = s;
        while let Some(stripped) = s.strip_suffix('?') {
            s = stripped.trim_end();
        }
        if let Some(rest) = strip_pointer_or_ref_prefix(s) {
            s = rest;
        }
        if let Some(rest) = s.strip_prefix("dyn").filter(|r| r.starts_with(char::is_whitespace)) {
            s = rest.trim_start();
        }
        if let Some(unwrapped) = strip_trailing_bracket_wrapper(s) {
            s = unwrapped;
        }
        if s == prev {
            break;
        }
    }
    if s.is_empty() { trimmed } else { s }
}

/// Checks whether `rest` starts on a valid type-modifier token boundary. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
fn is_modifier_boundary(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace() || matches!(c, '*' | '&' | '[' | '('))
}

/// Strips a leading pointer or reference qualifier (`*const`, `*mut`, `&`, `&'a mut`) from `s`. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
fn strip_pointer_or_ref_prefix(s: &str) -> Option<&str> {
    for prefix in ["*const", "*mut", "&mut", "&const"] {
        if let Some(rest) = s.strip_prefix(prefix).filter(|r| is_modifier_boundary(r)) {
            return Some(rest.trim_start());
        }
    }
    let mut rest = s.strip_prefix('&')?.trim_start();
    if let Some(after_quote) = rest.strip_prefix('\'') {
        let lt_len: usize = after_quote.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').map(char::len_utf8).sum();
        rest = after_quote.get(lt_len..)?.trim_start();
        if let Some(after) = ["mut", "const"].into_iter().find_map(|kw| rest.strip_prefix(kw).filter(|r| is_modifier_boundary(r))) {
            rest = after.trim_start();
        }
    }
    Some(rest.trim_start())
}

/// Strips a trailing `[...]` multiplicity suffix or unwraps a slice/array `[T; N]` wrapper. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
fn strip_trailing_bracket_wrapper(s: &str) -> Option<&str> {
    if !s.ends_with(']') {
        return None;
    }
    let (mut depth, mut open_idx, mut pos) = (0usize, None, s.len());
    for ch in s.chars().rev() {
        pos = pos.saturating_sub(ch.len_utf8());
        if ch == ']' {
            depth = depth.saturating_add(1);
        } else if ch == '[' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                open_idx = Some(pos);
                break;
            }
        }
    }
    let open = open_idx?;
    if open > 0 {
        return Some(s.get(..open)?.trim_end());
    }
    let inner = s.strip_prefix('[')?.strip_suffix(']')?.trim();
    let (mut d, mut idx) = (0usize, 0usize);
    for ch in inner.chars() {
        match ch {
            '[' | '<' | '(' => d = d.saturating_add(1),
            ']' | '>' | ')' => d = d.saturating_sub(1),
            ';' if d == 0 => return Some(inner.get(..idx)?.trim()),
            _ => {}
        }
        idx = idx.saturating_add(ch.len_utf8());
    }
    Some(inner)
}

/// Unwraps balanced outer `<...>` generic delimiters when they enclose the entire string `s`. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
fn unwrap_outer_angle_brackets(s: &str) -> &str {
    if !(s.starts_with('<') && s.ends_with('>')) {
        return s;
    }
    let (mut depth, mut closed_early, mut idx, last) = (0usize, false, 0usize, s.len().saturating_sub(1));
    for ch in s.chars() {
        if ch == '<' {
            depth = depth.saturating_add(1);
        } else if ch == '>' {
            depth = depth.saturating_sub(1);
            if depth == 0 && idx < last {
                closed_early = true;
                break;
            }
        }
        idx = idx.saturating_add(ch.len_utf8());
    }
    if depth == 0 && !closed_early { s.strip_prefix('<').and_then(|r| r.strip_suffix('>')).map_or(s, str::trim) } else { s }
}

/// Deterministically splits comma-separated top-level type arguments inside generic angle brackets `<...>`. Realises: [REQ-SEMANTIC-VALIDATOR-SPLIT-TOP-LEVEL-TYPE-ARGS] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn split_top_level_type_args(args: &str) -> Vec<&str> {
    let s = unwrap_outer_angle_brackets(args.trim());
    if s.is_empty() {
        return Vec::new();
    }
    let (mut result, mut angle, mut paren, mut bracket, mut start, mut idx) = (Vec::new(), 0usize, 0usize, 0usize, 0usize, 0usize);
    for ch in s.chars() {
        let next_idx = idx.saturating_add(ch.len_utf8());
        match ch {
            '<' => angle = angle.saturating_add(1),
            '>' => angle = angle.saturating_sub(1),
            '(' => paren = paren.saturating_add(1),
            ')' => paren = paren.saturating_sub(1),
            '[' => bracket = bracket.saturating_add(1),
            ']' => bracket = bracket.saturating_sub(1),
            ',' if angle == 0 && paren == 0 && bracket == 0 => {
                if let Some(t) = s.get(start..idx).map(str::trim).filter(|t| !t.is_empty()) {
                    result.push(t);
                }
                start = next_idx;
            }
            _ => {}
        }
        idx = next_idx;
    }
    if let Some(tail) = s.get(start..).map(str::trim).filter(|t| !t.is_empty()) {
        result.push(tail);
    }
    result
}

/// Checks whether `type_name` decomposes exclusively into SysML v2, Rust, or Dart/Flutter primitive and builtin types. Realises: [REQ-SEMANTIC-VALIDATOR-IS-PRIMITIVE-OR-BUILTIN-TYPE] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn is_primitive_or_builtin_type(type_name: &str) -> bool {
    check_type_resolvable_with(type_name, 0, |_| false)
}

/// Iteratively checks whether `type_name` decomposes into builtin or `is_user_symbol`-resolvable components up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-IS-RESOLVABLE-TYPE] | [FLS: fls_66m4rnbssgig]
pub(crate) fn check_type_resolvable_with(type_name: &str, start_depth: usize, is_user_symbol: impl Fn(&str) -> bool) -> bool {
    let mut stack: Vec<(&str, usize)> = vec![(type_name, start_depth)];
    while let Some((current_raw, depth)) = stack.pop() {
        let clean = normalize_source_type(current_raw);
        if depth >= MAX_AST_DEPTH || clean.is_empty() {
            return false;
        }
        if clean == "()" {
            continue;
        }
        if let Some(tuple_inner) = extract_enclosing_tuple_inner(clean) {
            let inner = tuple_inner.trim();
            if !inner.is_empty() {
                let args = split_top_level_type_args(inner);
                if args.is_empty() {
                    return false;
                }
                for arg in args.into_iter().rev() {
                    stack.push((arg, depth.saturating_add(1)));
                }
            }
            continue;
        }
        match decompose_generic_type(clean) {
            GenericDecomposition::Malformed => return false,
            GenericDecomposition::Split { outer, inner } => {
                let (clean_outer, args) = (normalize_source_type(outer), split_top_level_type_args(inner));
                if clean_outer.is_empty() || args.is_empty() || (!is_atomic_builtin_type_name(clean_outer) && !is_user_symbol(clean_outer)) {
                    return false;
                }
                for arg in args.into_iter().rev() {
                    stack.push((arg, depth.saturating_add(1)));
                }
            }
            GenericDecomposition::Atomic if !is_atomic_builtin_type_name(clean) && !is_user_symbol(clean) => return false,
            GenericDecomposition::Atomic => {}
        }
    }
    true
}

/// Structural classification of a normalized type expression into atomic, generic, or malformed forms. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_8tsynkj2cufj]
pub(crate) enum GenericDecomposition<'a> {
    Atomic,
    Split { outer: &'a str, inner: &'a str },
    Malformed,
}

/// Splits a normalized type string into its outer generic constructor and inner angle-bracket arguments. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
pub(crate) fn decompose_generic_type(clean: &str) -> GenericDecomposition<'_> {
    let Some(lt_idx) = clean.find('<') else { return GenericDecomposition::Atomic };
    let Some(gt_idx) = clean.rfind('>') else { return GenericDecomposition::Malformed };
    if gt_idx <= lt_idx || !clean.get(gt_idx.saturating_add(1)..).is_some_and(|t| t.trim().is_empty()) {
        return GenericDecomposition::Malformed;
    }
    match (clean.get(..lt_idx), clean.get(lt_idx.saturating_add(1)..gt_idx)) {
        (Some(outer), Some(inner)) => GenericDecomposition::Split { outer: outer.trim(), inner },
        _ => GenericDecomposition::Malformed,
    }
}

/// Extracts the inner substring of a balanced top-level tuple type `(A, B, ...)`. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
pub(crate) fn extract_enclosing_tuple_inner(clean: &str) -> Option<&str> {
    if !clean.starts_with('(') || !clean.ends_with(')') {
        return None;
    }
    let (mut depth, mut idx, last) = (0usize, 0usize, clean.len().saturating_sub(1));
    for ch in clean.chars() {
        if ch == '(' {
            depth = depth.saturating_add(1);
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 && idx < last {
                return None;
            }
        }
        idx = idx.saturating_add(ch.len_utf8());
    }
    (depth == 0).then(|| clean.strip_prefix('(').and_then(|s| s.strip_suffix(')'))).flatten()
}

/// Checks whether an atomic type identifier matches a known SysML v2, Rust, or Dart/Flutter builtin. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
pub(crate) fn is_atomic_builtin_type_name(clean: &str) -> bool {
    ["ScalarValues::", "ISQ::", "SI::", "std::", "core::", "alloc::"].iter().any(|p| clean.starts_with(p))
        || matches!(
            clean.rsplit("::").next().unwrap_or(clean).trim(),
            "Boolean" | "bool" | "boolean" | "Integer" | "int" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                | "Natural" | "natural" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "Real" | "real"
                | "Float" | "float" | "double" | "num" | "f32" | "f64" | "String" | "string" | "str" | "char"
                | "void" | "unit" | "dynamic" | "Object" | "ScalarValues" | "Vec" | "Option" | "Result" | "Box"
                | "Arc" | "Rc" | "Cow" | "Mutex" | "RwLock" | "BTreeMap" | "BTreeSet" | "HashMap" | "HashSet"
                | "Path" | "PathBuf" | "Duration" | "Instant" | "SystemTime" | "DateTime" | "Error" | "List"
                | "Map" | "Set" | "Iterable" | "Future" | "Stream" | "Widget" | "StatefulWidget"
                | "StatelessWidget" | "State" | "BuildContext" | "Key" | "GlobalKey" | "LocalKey" | "ValueKey"
                | "VoidCallback" | "ValueChanged" | "ValueGetter" | "ValueSetter" | "WidgetBuilder"
                | "IndexedWidgetBuilder" | "TransitionBuilder" | "Color" | "IconData" | "EdgeInsets"
                | "EdgeInsetsGeometry" | "BoxConstraints" | "Size" | "Offset" | "Rect" | "TextStyle"
                | "ThemeData" | "TextEditingController" | "ScrollController" | "FocusNode" | "AnimationController"
        )
}

/// Computes the qualified package scope name for `pkg` under `parent_scope`. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_66m4rnbssgig]
pub(crate) fn compute_scoped_package_name(pkg: &PackageDef, parent_scope: &str) -> String {
    let empty_root = pkg.name == "SysML_Model"
        && pkg.attribute_defs.is_empty() && pkg.port_defs.is_empty() && pkg.action_defs.is_empty()
        && pkg.constraint_defs.is_empty() && pkg.calc_defs.is_empty() && pkg.requirement_defs.is_empty()
        && pkg.state_defs.is_empty() && pkg.connection_defs.is_empty() && pkg.use_case_defs.is_empty()
        && pkg.item_defs.is_empty() && pkg.hazard_defs.is_empty() && pkg.risk_defs.is_empty()
        && pkg.test_case_defs.is_empty() && pkg.capability_defs.is_empty() && pkg.operation_defs.is_empty()
        && pkg.interaction_defs.is_empty() && pkg.part_defs.is_empty();
    match (parent_scope.is_empty(), pkg.name.is_empty() || empty_root) {
        (true, true) if !pkg.packages.is_empty() => String::new(),
        (true, _) => pkg.name.clone(),
        (false, _) if pkg.name.is_empty() => parent_scope.to_string(),
        _ => format!("{parent_scope}::{}", pkg.name),
    }
}

/// Parsed safety annotation target list or unclosed-delimiter diagnostic marker. Realises: [REQ-SEMANTIC-MOD] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AnnotationSpan {
    Targets(Vec<String>),
    UnclosedDelimiter,
}

/// Extracts `@SafetyRealises` or `@TriggersHazard` target identifiers from requirement prose. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY] | [FLS: fls_66m4rnbssgig]
pub(crate) fn extract_annotation_targets(text: &str, tag: &str) -> Vec<AnnotationSpan> {
    if tag.is_empty() {
        return Vec::new();
    }
    let (mut results, lower_text, lower_tag, mut search_idx) = (Vec::new(), text.to_ascii_lowercase(), tag.to_ascii_lowercase(), 0usize);
    while let Some(pos) = lower_text.get(search_idx..).and_then(|w| w.find(&lower_tag)) {
        let tag_start = search_idx.saturating_add(pos);
        let idx = tag_start.saturating_add(lower_tag.len());
        let Some(rest) = text.get(idx..) else { break };
        let after_ws = rest.trim_start();
        let ws_end = rest.len().saturating_sub(after_ws.len());
        if after_ws.starts_with('(') {
            let Some(rparen_tok) = tokenize_sysml(after_ws).into_iter().find(|t| t.token == SysmlToken::RParen) else {
                results.push(AnnotationSpan::UnclosedDelimiter);
                break;
            };
            let Some(inside) = after_ws.get(1..rparen_tok.span.start) else {
                results.push(AnnotationSpan::UnclosedDelimiter);
                break;
            };
            results.push(AnnotationSpan::Targets(inside.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()));
            search_idx = idx.saturating_add(ws_end).saturating_add(rparen_tok.span.end);
            continue;
        } else if let Some(after_colon) = after_ws.strip_prefix(':') {
            let id_rest = after_colon.trim_start();
            let id_start = after_colon.len().saturating_sub(id_rest.len());
            let id_len: usize = id_rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').map(char::len_utf8).sum();
            if let Some(raw_id) = (id_len > 0).then(|| id_rest.get(..id_len)).flatten() {
                results.push(AnnotationSpan::Targets(vec![raw_id.trim().to_string()]));
                search_idx = idx.saturating_add(ws_end).saturating_add(1).saturating_add(id_start).saturating_add(id_len);
                continue;
            }
        }
        let step = text.get(tag_start..).and_then(|s| s.chars().next()).map_or(1, char::len_utf8);
        search_idx = tag_start.saturating_add(step);
    }
    results
}

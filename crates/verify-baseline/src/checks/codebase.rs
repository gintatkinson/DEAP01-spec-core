//! Baseline verification checks 38 through 41 for presentation anti-hardcoding, zero mocking, UI contracts, and solver bridge parity.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and MIL-STD-1472H standards.
//! Enforces deterministic static AST verification preventing hardcoded presentation strings,
//! mock/fake test bypasses, unverified viewport contracts, and duplicate solver logic across Flutter and Rust tiers.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure Schema & Topology Grounding: Extracts identifiers dynamically at runtime from schema and topology files.
//! - Zero Regex Usage: Enforces deterministic linear character and token traversal exclusively.
//! - Bijective Persistence & SSOT: Validates 1:1 entity parity in SQLite persistence seeds and thin C-ABI delegation.
//!
//! ## 3. Preconditions
//! - `repo_root` is an accessible filesystem path pointing to the root of the workspace.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` when all checked invariants hold.
//! - Returns `Err(Vec<String>)` containing formatted diagnostic violation messages upon any defect.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(F * L) where F is the file count and L is the average line length.
//! - Space Complexity: O(E + V) where E is the entity identifier count and V is the violation diagnostic count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Gracefully returns `Ok(())` when optional directories (such as `app_flutter`) are absent.
//! - Collects all detected violations per gate rather than failing on the first violation.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS], [DO-330-TQL-4-CHECKS-CODEBASE]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 38: Presentation Anti-Hardcoding AST Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H human-machine interface verification mandates.
/// Mechanically rejects hardcoded domain entity labels, static telemetry strings, and static row builder booleans.
///
/// ## 2. Invariants & Mathematical Properties
/// - Dynamic Entity Set: Identifiers are dynamically discovered from topology datasets and SysML schemas.
/// - Zero Regex: Linear character scanners detect un-interpolated `Text(...)` literals and static row flags.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid workspace path.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if no hardcoded presentation literals exist or if target UI features are absent.
/// - Returns `Err(Vec<String>)` with file and line diagnostics for every hardcoded presentation occurrence.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F * L + N * log N) over feature files and extracted entity names.
/// - Space Complexity: O(N + V) for extracted entity sets and collected violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Ok(())` when `app_flutter/lib/features` is absent.
/// - Bounded token inspection using safe string slicing.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-PRESENTATION-ANTI-HARDCODING]
pub fn check_38_presentation_anti_hardcoding_ast_gate(
    repo_root: &Path,
) -> Result<(), Vec<String>> {
    let features_dir = repo_root.join("app_flutter").join("lib").join("features");
    if !features_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();
    let entity_ids = extract_entity_identifiers_from_topology_and_schema(repo_root);

    scan_dart_features_for_hardcoding(repo_root, &entity_ids, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Dynamically extracts entity identifiers and names from topology JSON and SysML schema files.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A pure schema-driven compilation mandates (Check 19 domain cleanliness).
/// Eliminates hardcoded domain constants in verification tooling by reading declared entities dynamically.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Regex Extraction: Traverses JSON structure and schema text line-by-line without regular expressions.
/// - Keyword Filtering: Discards reserved language keywords and tokens shorter than 3 characters.
///
/// ## 3. Preconditions
/// - `repo_root` points to the workspace root directory.
///
/// ## 4. Postconditions
/// - Returns a deduplicated `BTreeSet<String>` containing valid discovered entity identifiers.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(S + T * log T) where S is schema size and T is topology node count.
/// - Space Complexity: O(N) where N is the set of extracted entity strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing files return empty set elements without throwing unhandled errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-EXTRACT-TOPOLOGY-IDENTIFIERS]
fn extract_entity_identifiers_from_topology_and_schema(repo_root: &Path) -> BTreeSet<String> {
    let mut entity_ids = BTreeSet::new();

    let topology_path = repo_root.join("app_flutter").join("assets").join("topology_data.json");
    if let Ok(content) = fs::read_to_string(&topology_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            extract_strings_from_json_value(&val, &mut entity_ids);
        }
    }

    let schema_paths = [
        repo_root.join(".pipeline").join("schema.sysml"),
        repo_root.join("schema").join("schema.sysml"),
    ];

    for sp in &schema_paths {
        if let Ok(content) = fs::read_to_string(sp) {
            for line in content.lines() {
                let trimmed = line.trim();
                let decl_prefixes = [
                    "part def ", "item def ", "port def ", "action def ",
                    "calc def ", "attribute def ", "state def ", "part ",
                    "item ", "port ",
                ];
                for prefix in &decl_prefixes {
                    if let Some(rest) = trimmed.strip_prefix(prefix) {
                        let ident_token: String = rest
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                            .collect();
                        let clean = ident_token.trim();
                        if clean.len() >= 3 && !is_generic_schema_keyword(clean) {
                            entity_ids.insert(clean.to_string());
                        }
                    }
                }
            }
        }
    }

    entity_ids
}

/// Recursively harvests identifier string values from a JSON AST without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A data traversal requirements for configuration and topology parsing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Traverses objects and arrays recursively up to bounded recursion depth.
///
/// ## 3. Preconditions
/// - `val` is a valid parsed JSON value reference; `entities` is an accumulator set.
///
/// ## 4. Postconditions
/// - Appends all non-keyword string values associated with entity keys to `entities`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(J) where J is total JSON node count.
/// - Space Complexity: O(D) where D is max object nesting depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-object/non-array JSON variants terminate recursion safely.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-EXTRACT-JSON-VALUES]
fn extract_strings_from_json_value(val: &serde_json::Value, entities: &mut BTreeSet<String>) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let lower_k = k.to_lowercase();
                if lower_k == "id"
                    || lower_k == "name"
                    || lower_k == "label"
                    || lower_k == "callsign"
                    || lower_k == "title"
                    || lower_k == "parentid"
                    || lower_k == "parent_id"
                    || lower_k == "source"
                    || lower_k == "target"
                    || lower_k == "sourceid"
                    || lower_k == "targetid"
                {
                    if let Some(s) = v.as_str() {
                        insert_entity_and_stems(s, entities);
                    }
                } else if lower_k == "properties" {
                    if let serde_json::Value::Object(prop_map) = v {
                        for prop_val in prop_map.values() {
                            if let Some(ps) = prop_val.as_str() {
                                if ps.contains('-') || ps.contains('_') {
                                    insert_entity_and_stems(ps, entities);
                                }
                            }
                        }
                    }
                }
                extract_strings_from_json_value(v, entities);
            }
        }
        serde_json::Value::Array(arr) => {
            for elem in arr {
                extract_strings_from_json_value(elem, entities);
            }
        }
        _ => {}
    }
}

/// Inserts an entity identifier and its parenthetical/hyphenated base stems into `entities`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H entity stem disambiguation mandates.
/// Ensures compound topology labels (e.g. `"Unit-Alpha (P1)"`) and hierarchical parent IDs
/// (e.g. `"ZONE-NORTH-01"`) register both full strings and stems (`"Unit-Alpha"`, `"NORTH-01"`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Regex: Decomposes strings using deterministic `.split_once()` character boundaries.
/// - Enforces minimum length >= 3 and keyword exclusion on all inserted stems.
///
/// ## 3. Preconditions
/// - `raw` is a UTF-8 string slice; `entities` is a mutable `BTreeSet<String>`.
///
/// ## 4. Postconditions
/// - Inserts the trimmed identifier, any `(`-stripped prefix stem, and any hyphenated sub-stem into `entities`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L \log E)$ where $L$ is string length and $E$ is entity set size.
/// - Space Complexity: $O(L)$ for stem string allocations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: short or keyword strings are safely ignored.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-EXTRACT-JSON-VALUES]
fn insert_entity_and_stems(raw: &str, entities: &mut BTreeSet<String>) {
    let trimmed = raw.trim();
    if trimmed.len() >= 3 && !is_generic_schema_keyword(trimmed) {
        entities.insert(trimmed.to_string());
    }
    let base = if let Some((before_paren, _)) = trimmed.split_once('(') {
        let stem = before_paren.trim();
        if stem.len() >= 3 && !is_generic_schema_keyword(stem) {
            entities.insert(stem.to_string());
        }
        stem
    } else {
        trimmed
    };
    if let Some((_, sub_stem)) = base.split_once('-') {
        let sub_clean = sub_stem.trim();
        if sub_clean.contains('-')
            && sub_clean.len() >= 4
            && !is_generic_schema_keyword(sub_clean)
        {
            entities.insert(sub_clean.to_string());
        }
    }
}

/// Determines whether an identifier string is a generic language or schema keyword.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-330 TQL-4 lexical disambiguation standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Case-insensitive comparison against closed list of generic type and language tokens.
///
/// ## 3. Preconditions
/// - `ident` is a non-empty UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `true` if `ident` matches a generic keyword; `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K) where K is the count of reserved keywords.
/// - Space Complexity: O(1) static lookup.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure function with zero side effects.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-IS-GENERIC-SCHEMA-KEYWORD]
fn is_generic_schema_keyword(ident: &str) -> bool {
    let lower = ident.to_lowercase();
    matches!(
        lower.as_str(),
        "int"
            | "integer"
            | "bool"
            | "boolean"
            | "real"
            | "string"
            | "true"
            | "false"
            | "null"
            | "node"
            | "link"
            | "edge"
            | "type"
            | "item"
            | "part"
            | "port"
            | "def"
            | "action"
            | "calc"
            | "state"
            | "name"
    )
}

/// Scans Dart source files under `app_flutter/lib/features` for presentation hardcoding violations.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H anti-fabrication standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Only inspects `.dart` files under the specified directory path.
///
/// ## 3. Preconditions
/// - `repo_root` is valid workspace root; `entity_ids` is pre-populated identifier set.
///
/// ## 4. Postconditions
/// - Appends diagnostic violation strings to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F * L) over scanned Dart lines.
/// - Space Complexity: O(V) for violation accumulation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unreadable files are skipped safely with error suppression or diagnostic logging.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-SCAN-DART-FEATURES]
fn scan_dart_features_for_hardcoding(
    repo_root: &Path,
    entity_ids: &BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    let features_dir = repo_root.join("app_flutter").join("lib").join("features");
    for entry_res in WalkDir::new(&features_dir) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(path) {
                    let rel = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path)
                        .display()
                        .to_string();
                    let stripped_content = strip_dart_comments(&content);
                    for (idx, line) in stripped_content.lines().enumerate() {
                        let line_no = idx.saturating_add(1);
                        check_presentation_line(line, &rel, line_no, entity_ids, violations);
                    }
                }
            }
        }
    }
}

/// Checks an individual Dart source line for hardcoded presentation strings or static boolean flags.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H presentation inspection rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Discards line comments before analysis.
/// - Flags un-interpolated `Text('...')` or `const Text('...')` referencing known entity identifiers.
/// - Flags static telemetry indicators ('RULES: 0 VIOLATIONS', 'PASS', 'FAIL', etc.).
/// - Flags static row builder arguments (`isActive: true/false`, `isSelected: true/false`).
///
/// ## 3. Preconditions
/// - `line` is a valid UTF-8 string; `entity_ids` contains extracted entity names.
///
/// ## 4. Postconditions
/// - Populates `violations` if hardcoded presentation literals are present.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L + N) where L is line length and N is entity count.
/// - Space Complexity: O(V) for violation reporting.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Safe non-panicking string inspection using `.find()` and `.get()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
fn check_presentation_line(
    line: &str,
    file_rel: &str,
    line_no: usize,
    entity_ids: &BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    let stripped = if let Some((code, _)) = line.split_once("//") {
        code.trim()
    } else {
        line.trim()
    };

    if stripped.is_empty()
        || stripped.starts_with("import ")
        || stripped.starts_with("export ")
        || stripped.starts_with("part ")
        || stripped.starts_with("library ")
    {
        return;
    }

    for flag_name in ["isActive", "isSelected"] {
        let key_colon = format!("{}:", flag_name);
        let mut search_from = 0usize;
        while let Some(rel_idx) = stripped.get(search_from..).and_then(|s| s.find(&key_colon)) {
            let after_colon_idx = search_from
                .saturating_add(rel_idx)
                .saturating_add(key_colon.len());
            let after_colon = stripped.get(after_colon_idx..).unwrap_or("").trim_start();
            for bool_lit in ["true", "false"] {
                if let Some(rest) = after_colon.strip_prefix(bool_lit) {
                    let boundary_ok = rest
                        .chars()
                        .next()
                        .map(|c| !c.is_ascii_alphanumeric() && c != '_')
                        .unwrap_or(true);
                    if boundary_ok {
                        violations.push(format!(
                            "Check 38 violation: Hardcoded static boolean flag '{}: {}' found in {}:{}. Row states must bind dynamically.",
                            flag_name, bool_lit, file_rel, line_no
                        ));
                    }
                }
            }
            search_from = after_colon_idx;
        }
    }

    let literals = extract_uninterpolated_string_literals(stripped);
    for (start_idx, inner_raw) in literals {
        let prefix_trimmed = stripped.get(..start_idx).unwrap_or("").trim_end();
        if prefix_trimmed.ends_with("Key(") {
            continue;
        }
        let inner = inner_raw.trim();
        if inner.is_empty() {
            continue;
        }

        let mut matched_entity = false;
        for entity in entity_ids {
            if inner == entity.as_str() || inner.contains(entity.as_str()) {
                violations.push(format!(
                    "Check 38 violation: Hardcoded un-interpolated domain entity literal '{}' in Text widget in {}:{}.",
                    entity, file_rel, line_no
                ));
                matched_entity = true;
                break;
            }
        }

        if is_hardcoded_telemetry_or_designator_literal(inner) {
            violations.push(format!(
                "Check 38 violation: Hardcoded static telemetry indicator found in {}:{}: '{}'. Telemetry status must bind to live solver or monitor state.",
                file_rel, line_no, stripped
            ));
        } else if matched_entity {
            // Entity violation already recorded above.
        }
    }
}

/// Extracts all un-interpolated single-quoted or double-quoted string literals from a Dart source line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Check 34 Zero-Regex lexical scanning mandates.
/// Replaces naive `split_once(')')` slicing so strings containing inner parentheses or trailing widget
/// arguments are extracted losslessly.
///
/// ## 2. Invariants & Mathematical Properties
/// - Respects `\` character escapes inside string literals.
/// - Excludes any literal containing `$` (Dart string interpolation).
///
/// ## 3. Preconditions
/// - `line` is a comment-stripped UTF-8 Dart source line.
///
/// ## 4. Postconditions
/// - Returns `Vec<(usize, String)>` pairing each literal's opening quote byte index with its inner text.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ single linear pass over `line`.
/// - Space Complexity: $O(L)$ for extracted literal slices.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unclosed quotes at end of line are safely ignored without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
fn extract_uninterpolated_string_literals(line: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch == '\'' || ch == '"' {
            let quote = ch;
            let mut buf = String::new();
            let mut closed = false;
            while let Some((_, c)) = chars.next() {
                if c == '\\' {
                    buf.push(c);
                    if let Some((_, esc)) = chars.next() {
                        buf.push(esc);
                    }
                } else if c == quote {
                    closed = true;
                    break;
                } else {
                    buf.push(c);
                }
            }
            if closed && !buf.contains('$') {
                results.push((idx, buf));
            }
        }
    }

    results
}

/// Evaluates whether an un-interpolated presentation string literal matches a structural telemetry,
/// status readout, badge, or hyphenated entity designator pattern without regular expressions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, Check 19 (Zero Hardcoded Domain Concepts), and Check 34 (Zero Regex).
/// Detects static telemetry readouts and hardcoded entity designators using pure character-class predicates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Regex and Zero Domain Constants: inspects character classes (digits, `%`, `/`, `:`, `[]`, `()`, `-`).
/// - Rejects standalone status tokens (`PASS`, `FAIL`), percentage literals (`<digit>%`), slash-ratios (`<digit> / <digit>`),
///   bracketed uppercase badges (`[AIR-GAP]`), colon-prefixed uppercase headers, parenthesized priority/timer tags,
///   hyphenated alphanumeric designators, and numeric count + uppercase/unit tokens.
///
/// ## 3. Preconditions
/// - `inner` is a trimmed, non-empty, un-interpolated string literal body.
///
/// ## 4. Postconditions
/// - Returns `true` if `inner` exhibits any hardcoded telemetry or entity designator structure; `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in the byte/character length of `inner`.
/// - Space Complexity: $O(T)$ where $T$ is the count of whitespace-delimited tokens in `inner`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure total function over UTF-8 string slices.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
fn is_hardcoded_telemetry_or_designator_literal(inner: &str) -> bool {
    let trimmed = inner.trim();
    if trimmed == "PASS" || trimmed == "FAIL" {
        return true;
    }

    // 1. Percentage literal: <digit>%
    if trimmed
        .as_bytes()
        .windows(2)
        .any(|w| w.first().is_some_and(u8::is_ascii_digit) && w.get(1) == Some(&b'%'))
    {
        return true;
    }

    // 2. Bracketed uppercase badge: [<UPPER/HYPHEN>]
    if trimmed.len() >= 4 && trimmed.starts_with('[') && trimmed.ends_with(']') {
        let badge_body = trimmed.get(1..trimmed.len().saturating_sub(1)).unwrap_or("");
        let upper_count = badge_body.chars().filter(|c| c.is_ascii_uppercase()).count();
        let all_badge_chars = badge_body
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_');
        if upper_count >= 2 && all_badge_chars {
            return true;
        }
    }

    // 3. Colon-prefixed uppercase or designator status/telemetry readout: <UPPER_OR_ID>: <val>
    if let Some((left, right)) = trimmed.split_once(':') {
        let left_clean = left.trim_start_matches('(').trim();
        let right_clean = right.trim_end_matches(')').trim();
        if !left_clean.is_empty() && !right_clean.is_empty() {
            let upper_count = left_clean.chars().filter(|c| c.is_ascii_uppercase()).count();
            let has_lower = left_clean.chars().any(|c| c.is_ascii_lowercase());
            if upper_count >= 2 && !has_lower {
                return true;
            }
        }
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();

    // 4. Slash-ratio telemetry: <digit> / <digit>
    for window in tokens.windows(3) {
        if let (Some(&w0), Some(&w1), Some(&w2)) = (window.first(), window.get(1), window.get(2)) {
            let left_is_num = w0.chars().all(|c| c.is_ascii_digit()) && !w0.is_empty();
            let right_is_num = w2.chars().all(|c| c.is_ascii_digit()) && !w2.is_empty();
            if left_is_num && w1 == "/" && right_is_num {
                return true;
            }
        }
    }

    // 5. Numeric count followed by uppercase plural/status word (e.g. "2 NODES", "0 VIOLATIONS")
    for window in tokens.windows(2) {
        if let (Some(&w0), Some(&w1)) = (window.first(), window.get(1)) {
            let left_is_num = w0.chars().all(|c| c.is_ascii_digit()) && !w0.is_empty();
            let right_clean = w1.trim_matches(|c: char| !c.is_ascii_alphabetic());
            let right_is_upper_word =
                right_clean.len() >= 3 && right_clean.chars().all(|c| c.is_ascii_uppercase());
            if left_is_num && right_is_upper_word {
                return true;
            }
        }
    }

    // 6. Per-token inspection: parenthesized tags, hyphenated designators, and numeric multipliers/units
    for raw_tok in &tokens {
        if raw_tok.starts_with("(P")
            && raw_tok.ends_with(')')
            && raw_tok.len() >= 4
            && raw_tok
                .get(2..raw_tok.len().saturating_sub(1))
                .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false)
        {
            return true;
        }

        let clean_tok = raw_tok.trim_matches(|c: char| {
            c == '(' || c == ')' || c == '[' || c == ']' || c == ',' || c == ';' || c == ':'
        });
        if clean_tok.is_empty() {
            continue;
        }

        // Numeric multiplier or unit token (e.g. "12x", "15km")
        let digit_prefix_len = clean_tok
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .count();
        if digit_prefix_len >= 1 && digit_prefix_len < clean_tok.len() {
            let unit_suffix = clean_tok.get(digit_prefix_len..).unwrap_or("");
            if matches!(unit_suffix, "x" | "km" | "m" | "s" | "ms" | "Hz" | "dB") {
                return true;
            }
        }

        // Hyphenated designator / callsign token (e.g. "Unit-Alpha", "ZONE-ALPHA-01", "NORTH-01", "SENSOR-01")
        if clean_tok.contains('-') && clean_tok.len() >= 4 {
            let starts_upper = clean_tok
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false);
            let segments: Vec<&str> = clean_tok.split('-').collect();
            let valid_segments = segments.len() >= 2
                && segments.iter().all(|seg| {
                    !seg.is_empty()
                        && seg
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '_')
                })
                && segments.get(1..).unwrap_or(&[]).iter().any(|seg| {
                    seg.chars()
                        .next()
                        .map(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                        .unwrap_or(false)
                });
            if starts_upper && valid_segments {
                return true;
            }
        }
    }

    false
}

/// Check 39: Zero-Mocking Test AST & SQLite Disposal Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Section 1.9 of the Project Constitution (Zero-Mocking Live Persistence Mandate).
/// Mechanically rejects mock classes, fake implementations, empty stubs, and leaking SQLite database connections.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Mock Classes: Bans any class declaration prefixed with Mock, _Mock, Fake, or _Fake in tests.
/// - Mandatory Teardown: Every test suite allocating SQLite databases in `setUpAll` must dispose them in `tearDownAll`.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid workspace path.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if all tests run against real persistence with proper disposal, or if test directories are absent.
/// - Returns `Err(Vec<String>)` listing all forbidden mock tokens, empty stubs, and missing teardown handlers.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(T * L) where T is the count of test files and L is lines per file.
/// - Space Complexity: O(V) for collected violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Skips non-existent test directories cleanly without error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-ZERO-MOCKING-TEST-AST-GATE]
pub fn check_39_zero_mocking_test_ast_gate(
    repo_root: &Path,
) -> Result<(), Vec<String>> {
    let test_dirs = [
        repo_root.join("app_flutter").join("test"),
        repo_root.join("app_flutter").join("integration_test"),
    ];

    let mut violations = Vec::new();

    for tdir in &test_dirs {
        if tdir.is_dir() {
            for entry_res in WalkDir::new(tdir) {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                        let rel = path
                            .strip_prefix(repo_root)
                            .unwrap_or(path)
                            .display()
                            .to_string();
                        scan_test_file_for_mocking(path, &rel, &mut violations);
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Strips `//` line comments and `/* ... */` block comments from Dart source while preserving newlines.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Check 34 Zero-Regex static analysis mandates.
/// Prevents commented-out code (`// tearDownAll`, `// await tester.tap`) from bypassing AST gates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves exact line count (`\n` characters) so diagnostic line numbers remain 1:1 with source files.
/// - Preserves `//` and `/*` sequences inside single-quoted and double-quoted string literals.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 Dart source slice.
///
/// ## 4. Postconditions
/// - Returns a `String` with all line and block comments replaced by empty space/newlines.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ single linear pass in character count.
/// - Space Complexity: $O(N)$ output string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: never panics or fails on arbitrary UTF-8 input.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS]
pub fn strip_dart_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push('\n');
            }
        } else if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block_comment = false;
            } else if ch == '\n' {
                out.push('\n');
            }
        } else if in_single {
            out.push(ch);
            if ch == '\\' {
                if let Some(esc) = chars.next() {
                    out.push(esc);
                }
            } else if ch == '\'' {
                in_single = false;
            }
        } else if in_double {
            out.push(ch);
            if ch == '\\' {
                if let Some(esc) = chars.next() {
                    out.push(esc);
                }
            } else if ch == '"' {
                in_double = false;
            }
        } else if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            in_line_comment = true;
        } else if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_block_comment = true;
        } else {
            if ch == '\'' {
                in_single = true;
            } else if ch == '"' {
                in_double = true;
            }
            out.push(ch);
        }
    }
    out
}

/// Extracts a Dart class identifier from a line regardless of Dart 3 class modifiers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Section 1.9 Zero-Mocking class declaration detection.
///
/// ## 2. Invariants & Mathematical Properties
/// - Handles `class Foo`, `final class Foo`, `base class Foo`, `sealed class Foo`, `abstract interface class Foo`.
///
/// ## 3. Preconditions
/// - `trimmed_line` is a comment-stripped, whitespace-trimmed Dart source line.
///
/// ## 4. Postconditions
/// - Returns `Some(class_name)` if a class declaration is present on the line; `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in line length.
/// - Space Complexity: $O(1)$ zero-allocation slice return.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `None` if no identifier follows `class`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
fn extract_dart_class_name(trimmed_line: &str) -> Option<&str> {
    let header = trimmed_line.split('{').next().unwrap_or(trimmed_line);
    let mut tokens = header.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "class" {
            if let Some(raw_name) = tokens.next() {
                let clean = raw_name.split('<').next().unwrap_or(raw_name).trim();
                if !clean.is_empty() {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// Extracts a receiver method call `<receiver>.<method>(` from a comment-stripped Dart line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 39/40 direct state mutation detection mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `Some("<receiver>.<method>(")` when a non-lifecycle method invocation is found on `receiver_prefix`.
/// - Ignores benign lifecycle and inspection methods (`dispose`, `close`, `addListener`, `removeListener`, `toString`).
///
/// ## 3. Preconditions
/// - `line` is a comment-stripped Dart line; `receiver_prefix` ends with `.` (e.g. `"bridge."`).
///
/// ## 4. Postconditions
/// - Returns `Some(call_str)` if a mutating method invocation is found; `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in line length.
/// - Space Complexity: $O(M)$ proportional to method identifier length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Safe slice indexing with zero panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE], [REQ-CHECK-40-SCAN-TEST-GESTURES]
fn extract_receiver_method_call(line: &str, receiver_prefix: &str) -> Option<String> {
    let idx = line.find(receiver_prefix)?;
    if idx > 0 {
        let prev = line.as_bytes().get(idx.saturating_sub(1)).copied().unwrap_or(b' ');
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return None;
        }
    }
    let after = line.get(idx.saturating_add(receiver_prefix.len())..)?;
    let method_name: String = after
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if method_name.is_empty() {
        return None;
    }
    let rest = after.get(method_name.len()..)?.trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let benign = ["dispose", "close", "addListener", "removeListener", "toString"];
    if benign.contains(&method_name.as_str()) {
        return None;
    }
    Some(format!("{}{}((", receiver_prefix, method_name).trim_end_matches('(').to_string() + "(")
}

/// Scans comment-stripped `testWidgets` blocks for direct receiver mutations without `tester.` gestures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H interactive gesture verification mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Partitions `stripped` into `testWidgets` blocks by brace depth.
/// - Flags any `receivers` method call in a `testWidgets` block that lacks a `tester.` gesture.
///
/// ## 3. Preconditions
/// - `stripped` is comment-stripped Dart test source; `receivers` lists receiver prefixes (e.g. `["bridge."]`).
///
/// ## 4. Postconditions
/// - Returns `Vec<(usize, String)>` of `(line_number, matched_call)` for every un-gestured direct mutation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L \times R)$ where $L$ is line count and $R$ is receiver prefix count.
/// - Space Complexity: $O(M)$ for collected mutation records within a block.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function over arbitrary comment-stripped source text.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE], [REQ-CHECK-40-SCAN-TEST-GESTURES]
fn find_ungestured_test_widgets_mutations(
    stripped: &str,
    receivers: &[&str],
) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut in_block = false;
    let mut brace_depth: isize = 0;
    let mut has_gesture = false;
    let mut block_mutations: Vec<(usize, String)> = Vec::new();
    let gestures = [
        "tester.tap(",
        "tester.drag(",
        "tester.enterText(",
        "tester.longPress(",
        "tester.sendKeyEvent(",
    ];

    for (idx, line) in stripped.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        if !in_block && line.contains("testWidgets(") {
            in_block = true;
            brace_depth = 0;
            has_gesture = false;
            block_mutations.clear();
        }
        if in_block {
            if gestures.iter().any(|g| line.contains(g)) {
                has_gesture = true;
            }
            for rec in receivers {
                if let Some(call) = extract_receiver_method_call(line, rec) {
                    block_mutations.push((line_no, call));
                }
            }
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth = brace_depth.saturating_add(1);
                } else if ch == '}' {
                    brace_depth = brace_depth.saturating_sub(1);
                }
            }
            if brace_depth <= 0 && line.contains("});") {
                if !has_gesture {
                    results.append(&mut block_mutations);
                }
                in_block = false;
            }
        }
    }
    if in_block && !has_gesture {
        results.append(&mut block_mutations);
    }
    results
}

/// Inspects a single comment-stripped line for mock/fake/stub declarations, forbidden `implements`, and tautologies.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Section 1.9 Zero-Mocking Live Persistence Mandate.
///
/// ## 2. Invariants & Mathematical Properties
/// - Detects `Mock*`, `_Mock*`, `Fake*`, `_Fake*`, `Stub*`, `_Stub*` regardless of Dart 3 class modifiers.
/// - Detects `implements *Repository`, `*DataSource`, `*Transport`, `*Service`, `*Store`, `*Client`.
/// - Detects tautological assertions (`expect(true, isTrue)`, `expect(false, isFalse)`).
///
/// ## 3. Preconditions
/// - `trimmed` is a comment-stripped, trimmed Dart line.
///
/// ## 4. Postconditions
/// - Appends any detected line-level Check 39 violations to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in line length.
/// - Space Complexity: $O(V)$ for appended violation diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking token inspection.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
fn check_line_for_mocking_violations(
    trimmed: &str,
    file_rel: &str,
    line_no: usize,
    violations: &mut Vec<String>,
) {
    if let Some(cls) = extract_dart_class_name(trimmed) {
        let is_mock_cls = cls.starts_with("Mock")
            || cls.starts_with("_Mock")
            || cls.starts_with("Fake")
            || cls.starts_with("_Fake")
            || cls.starts_with("Stub")
            || cls.starts_with("_Stub")
            || cls.contains("Mock")
            || cls.contains("Fake")
            || cls.contains("Stub");
        if is_mock_cls {
            violations.push(format!(
                "Check 39 violation: Forbidden mock or fake class declaration in {}:{}: '{}'. Live persistence required.",
                file_rel, line_no, trimmed
            ));
        }
    }
    for ext in ["extends Mock", "extends Fake", "with Mock", "with Fake"] {
        if trimmed.contains(ext) {
            violations.push(format!(
                "Check 39 violation: Forbidden inheritance from Mock/Fake in {}:{}: '{}'.",
                file_rel, line_no, trimmed
            ));
        }
    }
    if let Some((_, after_imp)) = trimmed.split_once("implements ") {
        let suffixes = ["Repository", "DataSource", "Transport", "Service", "Store", "Client"];
        for raw_tok in after_imp.split(|c: char| c == ',' || c == '{' || c.is_whitespace()) {
            let tok = raw_tok.trim();
            if suffixes.iter().any(|s| tok.ends_with(s)) {
                violations.push(format!(
                    "Check 39 violation: Forbidden mock repository/datasource implementation in {}:{}: '{}'.",
                    file_rel, line_no, trimmed
                ));
            }
        }
    }
    if trimmed.contains("TestThemeService") || trimmed.contains("FakeThemeService") {
        violations.push(format!(
            "Check 39 violation: Forbidden mock theme service in {}:{}: '{}'.",
            file_rel, line_no, trimmed
        ));
    }
    for stub in [
        "Result.success([])",
        "TopologyData(coordinateMapping: {}, nodes: [], links: [])",
        "TopologyData(nodes: [], links: [])",
        "expect(true, isTrue)",
        "expect(false, isFalse)",
    ] {
        if trimmed.contains(stub) {
            violations.push(format!(
                "Check 39 violation: Hardcoded empty result stub or tautological assertion in {}:{}: '{}'.",
                file_rel, line_no, trimmed
            ));
        }
    }
}

/// Inspects a single test file for forbidden mock classes, empty stubs, and leaking SQLite connections.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A live persistence testing mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips comments first via `strip_dart_comments` so commented-out `tearDownAll` or mocks are accurately analyzed.
/// - Rejects Dart 3 modified mock/fake/stub classes, service/repository `implements`, tautologies, and un-gestured mutations.
/// - Validates paired presence of uncommented `tearDownAll` containing `.close()` or `.dispose()`.
///
/// ## 3. Preconditions
/// - `file_path` exists on disk; `file_rel` is workspace-relative path string.
///
/// ## 4. Postconditions
/// - Appends diagnostic violation strings to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ where $L$ is lines in the file.
/// - Space Complexity: $O(V)$ for diagnostic accumulation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Files failing read operations do not panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
fn scan_test_file_for_mocking(file_path: &Path, file_rel: &str, violations: &mut Vec<String>) {
    let raw_content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let content = strip_dart_comments(&raw_content);

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            check_line_for_mocking_violations(trimmed, file_rel, line_no, violations);
        }
    }

    for (line_no, call) in find_ungestured_test_widgets_mutations(
        &content,
        &["bridge.", "controller.", "notifier."],
    ) {
        violations.push(format!(
            "Check 39 violation: Direct state mutation '{}' without tester gesture in {}:{}.",
            call, file_rel, line_no
        ));
    }

    let has_sqlite_setup = content.contains("openDatabase(")
        || content.contains("DatabaseInitializer")
        || content.contains("DomainSeedStrategy")
        || content.contains("seedGeneratedDomainDatabase");

    if has_sqlite_setup && content.contains("setUpAll(") {
        let has_valid_teardown = content.contains("tearDownAll(")
            && (content.contains(".close()") || content.contains(".dispose()"));
        if !has_valid_teardown {
            violations.push(format!(
                "Check 39 violation: {}: setUpAll initializes SQLite database without matching tearDownAll (.close()/.dispose()) disposal.",
                file_rel
            ));
        }
    }
}

/// Check 40: Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H spatial layout and interactive verification standards.
/// Enforces multi-resolution contract test suites, real gesture actuation, and HUD non-overlap invariants.
///
/// ## 2. Invariants & Mathematical Properties
/// - 6-Viewport Completeness: Verifies presence of FHD_1080P_1X through XDR_6K_2X in generated contract tests.
/// - Real Gesture Mandate: Rejects shallow probing helpers and direct solver bridge state mutations.
/// - Pairwise HUD Separation: Mandates non-overlap assertions for test suites mounting tactical HUD overlays.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid workspace path.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if generated UI contract tests and interactive gesture rules pass or if Flutter tests are absent.
/// - Returns `Err(Vec<String>)` detailing missing viewports, shallow helpers, direct bridge mutations, or unverified HUD overlays.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(T * L) across Flutter test suites.
/// - Space Complexity: O(V) for collected violation messages.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Ok(())` if `app_flutter/test` is absent.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-INTERACTIVE-GESTURE-HUD-NON-OVERLAP]
pub fn check_40_interactive_gesture_and_hud_non_overlap_gate(
    repo_root: &Path,
) -> Result<(), Vec<String>> {
    let test_dir = repo_root.join("app_flutter").join("test");
    if !test_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();

    let contract_path = test_dir.join("generated").join("schema_ui_contract_test.g.dart");
    if contract_path.is_file() {
        let rel = contract_path
            .strip_prefix(repo_root)
            .unwrap_or(&contract_path)
            .display()
            .to_string();
        verify_ui_contract_test_viewports(&contract_path, &rel, &mut violations);
    } else {
        violations.push(
            "Check 40 violation: Missing mandatory generated UI contract test app_flutter/test/generated/schema_ui_contract_test.g.dart."
                .to_string(),
        );
    }

    let test_dirs = [
        test_dir,
        repo_root.join("app_flutter").join("integration_test"),
    ];

    for tdir in &test_dirs {
        if tdir.is_dir() {
            for entry_res in WalkDir::new(tdir) {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                        let rel = path
                            .strip_prefix(repo_root)
                            .unwrap_or(path)
                            .display()
                            .to_string();
                        scan_test_file_for_interactive_gestures(path, &rel, &mut violations);
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Verifies that the generated schema UI contract test sweeps all 6 canonical viewports and 4 physical resolutions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to MIL-STD-1472H multi-resolution spatial layout verification mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips comments first via `strip_dart_comments`.
/// - Verifies all 6 canonical viewports (`FHD_1080P_1X`..`XDR_6K_2X`), all 4 physical resolutions,
///   live `tester.getRect(`, uncommented `tester.tap(`, `assertPairwiseHudNonOverlap`, and `.overlaps(`.
/// - Rejects synthetic `Rect.fromLTWH(` literals in active contract code.
///
/// ## 3. Preconditions
/// - `contract_path` points to existing generated test file on disk.
///
/// ## 4. Postconditions
/// - Appends violation diagnostic if any viewport, resolution, or live spatial check is missing or mocked.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(C)$ where $C$ is contract file length.
/// - Space Complexity: $O(C)$ for comment-stripped buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Reads safely and logs diagnostics on missing content.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-VERIFY-VIEWPORTS]
fn verify_ui_contract_test_viewports(contract_path: &Path, file_rel: &str, violations: &mut Vec<String>) {
    let raw_content = match fs::read_to_string(contract_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let content = strip_dart_comments(&raw_content);

    for vp in [
        "FHD_1080P_1X",
        "QHD_1440P_1X",
        "UHD_4K_1X",
        "UHD_4K_2X",
        "XDR_6K_1X",
        "XDR_6K_2X",
    ] {
        if !content.contains(vp) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing canonical viewport '{}'.",
                file_rel, vp
            ));
        }
    }

    for (w, h) in [("1920", "1080"), ("2560", "1440"), ("3840", "2160"), ("6016", "3384")] {
        if !content.contains(w) || !content.contains(h) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing physical resolution {}x{}.",
                file_rel, w, h
            ));
        }
    }

    for required_token in [
        "tester.getRect(",
        "tester.tap(",
        "assertPairwiseHudNonOverlap",
        ".overlaps(",
    ] {
        if !content.contains(required_token) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing active '{}' invocation.",
                file_rel, required_token
            ));
        }
    }

    if content.contains("Rect.fromLTWH(") {
        violations.push(format!(
            "Check 40 violation: Generated UI contract test {} uses synthetic Rect.fromLTWH( instead of live tester.getRect(.",
            file_rel
        ));
    }
}

/// Scans a Dart test file for shallow probing helpers, direct bridge mutations, and missing HUD assertions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H interactive gesture actuation requirements.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips comments first via `strip_dart_comments`.
/// - Rejects `probeWidgetKey` and shallow `evaluate().isNotEmpty`.
/// - Rejects domain-agnostic direct `bridge.<method>(` mutations in `testWidgets` blocks lacking a `tester.` gesture.
/// - Rejects HUD mounts lacking `assertPairwiseHudNonOverlap` or `.overlaps(`.
///
/// ## 3. Preconditions
/// - `file_path` is an existing file path; `file_rel` is relative path string.
///
/// ## 4. Postconditions
/// - Appends violation diagnostics to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ where $L$ is lines in the file.
/// - Space Complexity: $O(V)$ for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking line iteration.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-SCAN-TEST-GESTURES]
fn scan_test_file_for_interactive_gestures(
    file_path: &Path,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let raw_content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let content = strip_dart_comments(&raw_content);

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();

        if trimmed.contains("probeWidgetKey(") || trimmed.contains("probeWidgetKey ") {
            violations.push(format!(
                "Check 40 violation: Shallow widget probing helper detected in {}:{}: probeWidgetKey is banned.",
                file_rel, line_no
            ));
        }

        if trimmed.contains("evaluate().isNotEmpty") {
            violations.push(format!(
                "Check 40 violation: Shallow evaluate().isNotEmpty check detected in {}:{}. Tests must pump gestures and assert state transitions.",
                file_rel, line_no
            ));
        }
    }

    for (line_no, mut_call) in find_ungestured_test_widgets_mutations(&content, &["bridge."]) {
        violations.push(format!(
            "Check 40 violation: Direct solver bridge mutation detected in {}:{}: '{}'. UI tests must actuate controls via tester.tap(find.byKey(...)).",
            file_rel, line_no, mut_call
        ));
    }

    let mounts_hud = content.contains("HudOverlay")
        || content.contains("TacticalHud")
        || content.contains("HudPanel")
        || content.contains("hud_panel")
        || content.contains("hud_overlay");

    if mounts_hud {
        let has_non_overlap =
            content.contains("assertPairwiseHudNonOverlap") || content.contains(".overlaps(");
        if !has_non_overlap {
            violations.push(format!(
                "Check 40 violation: {}: Test file mounts Tactical HUD overlays but lacks assertPairwiseHudNonOverlap or Rect.overlaps assertion.",
                file_rel
            ));
        }
    }
}

/// Check 41: Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D single source of truth computational kernel requirements.
/// Guarantees that presentation solver bridges delegate exclusively to native Rust C-ABI routines,
/// and enforces 100% bijective entity parity between topology datasets and SQLite persistence seeds.
///
/// ## 2. Invariants & Mathematical Properties
/// - SSOT Computational Boundary: Bans duplicate Pareto and candidate synthesis math in Dart when a Rust solver exists.
/// - Bijective Entity Seed Parity: 100% of declared topology entities must be seeded into SQLite `instances` and `properties`.
/// - 5 Canonical Tables: Enforces presence of type_definitions, type_attributes, type_relations, properties, and instances.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid workspace path.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if solver bridge is thin and SQLite seed is bijectively complete, or if files are absent.
/// - Returns `Err(Vec<String>)` reporting duplicate Dart solver math or missing SQLite seed entities/tables.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(E * log S + B * L) where E is topology entity count and S is seed entity count.
/// - Space Complexity: O(E + V) for entity sets and violation diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Ok(())` if `app_flutter` or topology files are absent.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-SOLVER-BRIDGE-SQLITE-SEED-PARITY]
pub fn check_41_solver_bridge_and_sqlite_seed_parity_gate(
    repo_root: &Path,
) -> Result<(), Vec<String>> {
    let app_flutter_dir = repo_root.join("app_flutter");
    if !app_flutter_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();

    check_solver_bridge_ssot_delegation(repo_root, &mut violations);
    check_bijective_sqlite_seed_parity(repo_root, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Verifies that Dart solver bridge files are thin delegation adapters without duplicate solver math.
/// Audits Dart solver bridge files for duplicate optimization mathematics and thin-bridge ceiling violations.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A single source of truth kernel integrity mandates.
/// Exposes a standalone verification entrypoint for Check 41 duplicate solver math auditing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects any `app_flutter/lib/data/*bridge*.dart` file exceeding 1200 lines when a native solver crate exists.
/// - Rejects candidate synthesis, composite scoring, constraint evaluation, and HOTL veto state mutation in Dart bridges.
///
/// ## 3. Preconditions
/// - `repo_root` is the workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if all Dart solver bridges are thin delegation shims, or `Err(violations)` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(B \times L)$ across Dart bridge files and lines.
/// - Space Complexity: $O(V)$ for accumulated violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Ok(())` when no solver crate or `app_flutter/lib/data` directory exists.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-SSOT-DELEGATION]
pub fn check_no_duplicate_dart_solver_math(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    check_solver_bridge_ssot_delegation(repo_root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Enforces that Dart solver bridge sources delegate optimization math exclusively to the Rust solver.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A single source of truth kernel integrity mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Triggers when a native Rust solver crate exists under `crates/`.
/// - Rejects bridge files exceeding the 1200-line thin-bridge ceiling.
/// - Rejects candidate generator functions, composite scoring, constraint evaluation, and HOTL veto logic in Dart bridge sources.
///
/// ## 3. Preconditions
/// - `repo_root` is workspace root directory; `violations` is diagnostic accumulator.
///
/// ## 4. Postconditions
/// - Appends violation diagnostic if duplicate solver math or bloated bridge files are detected.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(B * L) across Dart bridge files.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Gracefully exits if no native solver crate or Dart bridge files exist.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-SSOT-DELEGATION]
fn check_solver_bridge_ssot_delegation(repo_root: &Path, violations: &mut Vec<String>) {
    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        return;
    }

    let upstream_crates = [
        "deap-core",
        "compile-sysml",
        "ingest-sysml",
        "verify-baseline",
        "assemble-conops",
        "reconcile-backlog",
    ];

    let mut has_solver_crate = false;
    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !upstream_crates.contains(&name.as_str()) {
                    let src_solver = entry.path().join("src").join("solver.rs");
                    let src_lib = entry.path().join("src").join("lib.rs");
                    if src_solver.is_file() || src_lib.is_file() {
                        has_solver_crate = true;
                        break;
                    }
                }
            }
        }
    }

    if !has_solver_crate {
        return;
    }

    let data_dir = repo_root.join("app_flutter").join("lib").join("data");
    if !data_dir.is_dir() {
        return;
    }

    let duplicate_math_tokens = [
        "_buildCandidates",
        "buildCandidates(",
        "alphaCovOffset",
        "reserveBoostPct",
        "extraDensityPenalty",
        "paretoScore",
        "violationCount: 0",
        "synthesize_candidates(",
        "synthesizeCandidates(",
        "compute_composite_score(",
        "computeCompositeScore(",
        "evaluate_constraints(",
        "evaluateConstraints(",
        "_hotlVetoActive = true",
    ];

    for entry_res in WalkDir::new(&data_dir) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy();
            if file_name.ends_with(".dart") && file_name.contains("bridge") {
                if let Ok(content) = fs::read_to_string(path) {
                    let rel = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path)
                        .display()
                        .to_string();
                    let line_count = content.lines().count();
                    if line_count > 1200 {
                        violations.push(format!(
                            "Check 41 violation: Dart solver bridge {} exceeds 1200-line thin-bridge ceiling ({} lines). Dart solver bridge must be a thin C-ABI delegation shim.",
                            rel, line_count
                        ));
                    }
                    for (idx, line) in content.lines().enumerate() {
                        let line_no = idx.saturating_add(1);
                        let trimmed = line.trim();
                        for token in &duplicate_math_tokens {
                            if trimmed.contains(token) {
                                violations.push(format!(
                                    "Check 41 violation: Duplicate solver mathematics detected in {}:{}: '{}'. Dart solver bridge must be a thin C-ABI delegation shim.",
                                    rel, line_no, token
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Enforces 100% bijective entity parity between topology datasets and SQLite persistence seeds.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D data integrity and persistence parity standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every node ID in `topology_data.json` must be seeded in SQLite `instances` and `properties`.
/// - Seed script must populate all 5 canonical tables: `type_definitions`, `type_attributes`, `type_relations`, `properties`, `instances`.
///
/// ## 3. Preconditions
/// - `repo_root` points to workspace root.
///
/// ## 4. Postconditions
/// - Appends violation diagnostics for any missing entity IDs or omitted canonical tables.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(E * log S) where E is topology entity count and S is seed statements.
/// - Space Complexity: O(E + V) for extracted sets and violation reports.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Exits safely if topology JSON or seed files are absent.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-BIJECTIVE-SEED-PARITY]
fn check_bijective_sqlite_seed_parity(repo_root: &Path, violations: &mut Vec<String>) {
    let topology_path = repo_root.join("app_flutter").join("assets").join("topology_data.json");
    if !topology_path.is_file() {
        return;
    }

    let content = match fs::read_to_string(&topology_path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!(
                "Check 41 violation: Failed to read app_flutter/assets/topology_data.json: {}.",
                err
            ));
            return;
        }
    };

    let val = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(v) => v,
        Err(err) => {
            violations.push(format!(
                "Check 41 violation: Malformed JSON in app_flutter/assets/topology_data.json: {}.",
                err
            ));
            return;
        }
    };

    let nodes_opt = val
        .get("nodes")
        .and_then(|n| n.as_array())
        .or_else(|| val.as_array());
    let nodes = match nodes_opt {
        Some(arr) => arr,
        None => {
            violations.push(
                "Check 41 violation: app_flutter/assets/topology_data.json is missing a valid 'nodes' array."
                    .to_string(),
            );
            return;
        }
    };

    let mut topology_entity_ids = BTreeSet::new();
    let mut topology_type_names = BTreeSet::new();
    let mut topology_property_keys = BTreeSet::new();

    for node in nodes {
        if let Some(id_str) = node.get("id").and_then(|i| i.as_str()) {
            let trimmed = id_str.trim();
            if !trimmed.is_empty() {
                topology_entity_ids.insert(trimmed.to_string());
            }
        }
        if let Some(type_str) = node
            .get("typeName")
            .or_else(|| node.get("type_name"))
            .or_else(|| node.get("type"))
            .and_then(|t| t.as_str())
        {
            let trimmed = type_str.trim();
            if !trimmed.is_empty() {
                topology_type_names.insert(trimmed.to_string());
            }
        }
        if let Some(props_obj) = node.get("properties").and_then(|p| p.as_object()) {
            for prop_key in props_obj.keys() {
                let trimmed = prop_key.trim();
                if !trimmed.is_empty() {
                    topology_property_keys.insert(trimmed.to_string());
                }
            }
        }
    }

    let seeds_dir = repo_root.join("app_flutter").join("lib").join("data").join("seeds");
    if !seeds_dir.is_dir() {
        violations.push(
            "Check 41 violation: SQLite seed directory app_flutter/lib/data/seeds is missing while topology_data.json is present."
                .to_string(),
        );
        return;
    }

    let mut seed_files_content = String::new();
    let mut has_generated_seed = false;
    let mut strategy_content: Option<String> = None;

    if let Ok(entries) = fs::read_dir(&seeds_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                    if let Ok(c) = fs::read_to_string(&path) {
                        if !c.trim().is_empty() {
                            let fname = entry.file_name().to_string_lossy().to_string();
                            if fname == "generated_domain_seed.g.dart" {
                                has_generated_seed = true;
                            } else if fname == "domain_seed_strategy.dart" {
                                strategy_content = Some(c.clone());
                            }
                            seed_files_content.push_str(&c);
                            seed_files_content.push('\n');
                        }
                    }
                }
            }
        }
    }

    if seed_files_content.trim().is_empty() {
        violations.push(
            "Check 41 violation: No non-empty .dart SQLite seed files found in app_flutter/lib/data/seeds."
                .to_string(),
        );
        return;
    }

    if let Some(strat) = &strategy_content {
        for placeholder in &["space_$", "field_1", "field_50"] {
            if strat.contains(placeholder) {
                violations.push(format!(
                    "Check 41 violation: Synthetic placeholder pattern '{}' detected in app_flutter/lib/data/seeds/domain_seed_strategy.dart.",
                    placeholder
                ));
            }
        }
        if has_generated_seed {
            let delegates_to_generated = strat.contains("seedGeneratedDomainDatabase")
                || strat.contains("seedDatabase");
            let contains_all_entities = !topology_entity_ids.is_empty()
                && topology_entity_ids.iter().all(|id| strat.contains(id));
            if !delegates_to_generated && !contains_all_entities {
                violations.push(
                    "Check 41 violation: domain_seed_strategy.dart neither delegates to seedGeneratedDomainDatabase nor seeds all topology entities directly."
                        .to_string(),
                );
            }
        }
    }

    for entity_id in &topology_entity_ids {
        if !seed_files_content.contains(entity_id) {
            violations.push(format!(
                "Check 41 violation: Bijective SQLite seed parity failure: entity '{}' from topology_data.json is missing from SQLite domain seed.",
                entity_id
            ));
        }
    }

    for type_name in &topology_type_names {
        if !seed_files_content.contains(type_name) {
            violations.push(format!(
                "Check 41 violation: Bijective SQLite seed parity failure: entity type '{}' from topology_data.json is missing from SQLite domain seed.",
                type_name
            ));
        }
    }

    for prop_key in &topology_property_keys {
        if !seed_files_content.contains(prop_key) {
            violations.push(format!(
                "Check 41 violation: Bijective SQLite seed parity failure: property key '{}' from topology_data.json is missing from SQLite domain seed.",
                prop_key
            ));
        }
    }

    let canonical_tables = [
        "type_definitions",
        "type_attributes",
        "type_relations",
        "properties",
        "instances",
    ];

    for tbl in &canonical_tables {
        if !seed_files_content.contains(tbl) {
            violations.push(format!(
                "Check 41 violation: SQLite domain seed does not populate canonical table '{}'.",
                tbl
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    struct ScopedTempDir {
        path: PathBuf,
    }

    impl ScopedTempDir {
        fn new(prefix: &str) -> Result<Self, std::io::Error> {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let dir = std::env::temp_dir().join(format!("test_codebase_{}_{}_{}", prefix, nanos, count));
            fs::create_dir_all(&dir)?;
            Ok(Self { path: dir })
        }
    }

    impl Drop for ScopedTempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_check_38_positive_clean_presentation() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_38_pos")?;
        let feature_dir = tmp.path.join("app_flutter/lib/features/workspace");
        fs::create_dir_all(&feature_dir)?;

        let clean_dart = r#"
class TacticalWorkspaceView extends StatelessWidget {
  final WorkspaceViewModel vm;
  const TacticalWorkspaceView({required this.vm, super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(vm.displayTitle),
        Text(vm.activeTelemetrySummary),
        EntityRow(
          label: vm.selectedEntityLabel,
          isActive: vm.isEntityActive,
        ),
      ],
    );
  }
}
"#;
        fs::write(feature_dir.join("workspace_view.dart"), clean_dart)?;

        let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
        assert!(res.is_ok(), "Positive clean presentation must pass: {:?}", res.err());
        Ok(())
    }

    #[test]
    fn test_check_38_negative_hardcoded_literals() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_38_neg")?;
        let assets_dir = tmp.path.join("app_flutter/assets");
        let feature_dir = tmp.path.join("app_flutter/lib/features/tactical");
        fs::create_dir_all(&assets_dir)?;
        fs::create_dir_all(&feature_dir)?;

        let topology_json = r#"{
  "nodes": [
    {"id": "NODE_ALPHA_01", "name": "Alpha Node", "type": "Sensor"}
  ]
}"#;
        fs::write(assets_dir.join("topology_data.json"), topology_json)?;

        let defective_dart = r#"
class DefectiveView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('NODE_ALPHA_01'),
        const Text('RULES: 0 VIOLATIONS'),
        EntityItem(isActive: true),
      ],
    );
  }
}
"#;
        fs::write(feature_dir.join("defective_view.dart"), defective_dart)?;

        let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
        assert!(res.is_err(), "Defective hardcoded presentation must fail Check 38");
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("NODE_ALPHA_01")));
        assert!(errs.iter().any(|e| e.contains("RULES: 0 VIOLATIONS")));
        assert!(errs.iter().any(|e| e.contains("isActive: true")));
        Ok(())
    }

    #[test]
    fn test_check_39_positive_clean_persistence_test() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_39_pos")?;
        let test_dir = tmp.path.join("app_flutter/test/data");
        fs::create_dir_all(&test_dir)?;

        let clean_test = r#"
void main() {
  late Database db;
  setUpAll(() async {
    db = await openDatabase(inMemoryDatabasePath);
    await seedGeneratedDomainDatabase(db);
  });

  tearDownAll(() async {
    await db.close();
  });

  test('valid live persistence test', () async {
    final rows = await db.query('instances');
    expect(rows, isNotEmpty);
  });
}
"#;
        fs::write(test_dir.join("live_db_test.dart"), clean_test)?;

        let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
        assert!(res.is_ok(), "Clean persistence test must pass Check 39: {:?}", res.err());
        Ok(())
    }

    #[test]
    fn test_check_39_negative_mocking_and_leaking_db() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_39_neg")?;
        let test_dir = tmp.path.join("app_flutter/test/features");
        fs::create_dir_all(&test_dir)?;

        let defective_test = r#"
class MockDataRepository extends Mock implements TreeRepository {}
class _FakeTacticalThemeService extends Fake {}

void main() {
  setUpAll(() async {
    await openDatabase(':memory:');
    // Notice missing tearDownAll!
  });

  test('mocked test', () {
    final emptyRes = const Result.success([]);
    expect(emptyRes, isNotNull);
  });
}
"#;
        fs::write(test_dir.join("mock_test.dart"), defective_test)?;

        let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
        assert!(res.is_err(), "Mocking and leaking SQLite must fail Check 39");
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("Mock") || e.contains("mocking")));
        assert!(errs.iter().any(|e| e.contains("Result.success")));
        assert!(errs.iter().any(|e| e.contains("tearDownAll")));
        Ok(())
    }

    #[test]
    fn test_check_40_positive_ui_contracts() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_40_pos")?;
        let test_gen_dir = tmp.path.join("app_flutter/test/generated");
        let test_dir = tmp.path.join("app_flutter/test/widgets");
        fs::create_dir_all(&test_gen_dir)?;
        fs::create_dir_all(&test_dir)?;

        let generated_contract = r#"
// Generated schema UI contract test
const kCanonicalViewports = [
  CanonicalViewport(identifier: 'FHD_1080P_1X', physicalWidth: 1920, physicalHeight: 1080),
  CanonicalViewport(identifier: 'QHD_1440P_1X', physicalWidth: 2560, physicalHeight: 1440),
  CanonicalViewport(identifier: 'UHD_4K_1X', physicalWidth: 3840, physicalHeight: 2160),
  CanonicalViewport(identifier: 'UHD_4K_2X', physicalWidth: 3840, physicalHeight: 2160),
  CanonicalViewport(identifier: 'XDR_6K_1X', physicalWidth: 6016, physicalHeight: 3384),
  CanonicalViewport(identifier: 'XDR_6K_2X', physicalWidth: 6016, physicalHeight: 3384),
];

void assertPairwiseHudNonOverlap(List<Rect> rects) {
  final overlaps = rects[0].overlaps(rects[1]);
}

void main() {
  testWidgets('assert zero RenderFlex overflow across all viewports', (tester) async {
    final rect = tester.getRect(find.byKey(const ValueKey('hud')));
    assertPairwiseHudNonOverlap([rect, rect]);
    await tester.tap(find.byKey(const ValueKey('row')));
  });
}
"#;
        fs::write(test_gen_dir.join("schema_ui_contract_test.g.dart"), generated_contract)?;

        let interactive_test = r#"
void main() {
  testWidgets('interactive tap test with HUD non-overlap', (tester) async {
    await tester.tap(find.byKey(const Key('tactical_control_button')));
    await tester.pumpAndSettle();
    assertPairwiseHudNonOverlap(tester);
  });
}
"#;
        fs::write(test_dir.join("interactive_test.dart"), interactive_test)?;

        let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
        assert!(res.is_ok(), "Positive contract test must pass Check 40: {:?}", res.err());
        Ok(())
    }

    #[test]
    fn test_check_40_negative_shallow_probing_and_direct_bridge_mutation() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_40_neg")?;
        let test_gen_dir = tmp.path.join("app_flutter/test/generated");
        let test_dir = tmp.path.join("app_flutter/test/widgets");
        fs::create_dir_all(&test_gen_dir)?;
        fs::create_dir_all(&test_dir)?;

        // Incomplete viewports: missing XDR_6K_2X
        let incomplete_contract = r#"
const kViewports = [
  'FHD_1080P_1X',
  'QHD_1440P_1X',
  'UHD_4K_1X',
  'UHD_4K_2X',
  'XDR_6K_1X',
];
"#;
        fs::write(test_gen_dir.join("schema_ui_contract_test.g.dart"), incomplete_contract)?;

        let defective_test = r#"
void probeWidgetKey(Key key) {
  find.byKey(key).evaluate().isNotEmpty;
}

void main() {
  testWidgets('defective test mounting TacticalHud without non-overlap assert', (tester) async {
    await tester.pumpWidget(const TacticalHud());
    bridge.relockInterlock();
  });
}
"#;
        fs::write(test_dir.join("defective_widget_test.dart"), defective_test)?;

        let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
        assert!(res.is_err(), "Incomplete viewports and shallow probing must fail Check 40");
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("XDR_6K_2X")));
        assert!(errs.iter().any(|e| e.contains("probeWidgetKey")));
        assert!(errs.iter().any(|e| e.contains("bridge.relockInterlock")));
        assert!(errs.iter().any(|e| e.contains("assertPairwiseHudNonOverlap")));
        Ok(())
    }

    #[test]
    fn test_check_41_positive_c_abi_and_seed_parity() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_41_pos")?;
        let assets_dir = tmp.path.join("app_flutter/assets");
        let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
        let data_dir = tmp.path.join("app_flutter/lib/data");
        fs::create_dir_all(&assets_dir)?;
        fs::create_dir_all(&seeds_dir)?;
        fs::create_dir_all(&data_dir)?;

        let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Primary Sensor"},
    {"id": "CONTROLLER_02", "name": "Main Controller"}
  ]
}"#;
        fs::write(assets_dir.join("topology_data.json"), topology_json)?;

        let seed_dart = r#"
// Generated domain seed
Future<void> seedDatabase(Database db) async {
  // 5 canonical tables:
  // type_definitions, type_attributes, type_relations, properties, instances
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO type_relations VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('SENSOR_01'), ('CONTROLLER_02')");
  await db.execute("INSERT INTO properties VALUES (...)");
}
"#;
        fs::write(seeds_dir.join("generated_domain_seed.g.dart"), seed_dart)?;

        let bridge_dart = r#"
class SolverBridge {
  final DynamicLibrary _dylib;
  SolverBridge(this._dylib);

  Uint8List evaluate(Uint8List payload) {
    // Pure thin C-ABI delegation shim calling native cdylib
    return _callNativeFfi(payload);
  }
}
"#;
        fs::write(data_dir.join("solver_bridge.dart"), bridge_dart)?;

        let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
        assert!(res.is_ok(), "Positive C-ABI and seed parity must pass Check 41: {:?}", res.err());
        Ok(())
    }

    #[test]
    fn test_check_41_negative_duplicate_math_and_missing_seeds() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = ScopedTempDir::new("check_41_neg")?;
        let solver_crate_dir = tmp.path.join("crates/domain-solver/src");
        let assets_dir = tmp.path.join("app_flutter/assets");
        let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
        let data_dir = tmp.path.join("app_flutter/lib/data");
        fs::create_dir_all(&solver_crate_dir)?;
        fs::create_dir_all(&assets_dir)?;
        fs::create_dir_all(&seeds_dir)?;
        fs::create_dir_all(&data_dir)?;

        fs::write(solver_crate_dir.join("lib.rs"), "// solver crate")?;
        fs::write(solver_crate_dir.join("solver.rs"), "// solver arithmetic")?;

        let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Primary Sensor"},
    {"id": "SENSOR_02_MISSING", "name": "Missing Sensor"}
  ]
}"#;
        fs::write(assets_dir.join("topology_data.json"), topology_json)?;

        // Seed file misses SENSOR_02_MISSING and misses type_relations table
        let defective_seed_dart = r#"
Future<void> seedDatabase(Database db) async {
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('SENSOR_01')");
  await db.execute("INSERT INTO properties VALUES (...)");
}
"#;
        fs::write(seeds_dir.join("generated_domain_seed.g.dart"), defective_seed_dart)?;

        // Bridge file has duplicate candidate generator and Pareto scoring
        let defective_bridge_dart = r#"
class SolverBridge {
  List<Candidate> _buildCandidates() {
    final paretoScore = alphaCovOffset * 0.8 + reserveBoostPct * 0.2;
    return [Candidate(score: paretoScore, violationCount: 0)];
  }
}
"#;
        fs::write(data_dir.join("domain_solver_bridge.dart"), defective_bridge_dart)?;

        let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
        assert!(res.is_err(), "Duplicate math and missing seed entities must fail Check 41");
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("SENSOR_02_MISSING")));
        assert!(errs.iter().any(|e| e.contains("type_relations")));
        assert!(errs.iter().any(|e| e.contains("_buildCandidates")));
        Ok(())
    }
}

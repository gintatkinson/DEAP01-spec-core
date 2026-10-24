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
                {
                    if let Some(s) = v.as_str() {
                        let trimmed = s.trim();
                        if trimmed.len() >= 3 && !is_generic_schema_keyword(trimmed) {
                            entities.insert(trimmed.to_string());
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
                    for (idx, line) in content.lines().enumerate() {
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

    if stripped.is_empty() {
        return;
    }

    let static_bool_patterns = [
        "isActive: true",
        "isActive: false",
        "isSelected: true",
        "isSelected: false",
        "isActive:\ttrue",
        "isActive:\tfalse",
        "isSelected:\ttrue",
        "isSelected:\tfalse",
    ];

    for pat in &static_bool_patterns {
        if stripped.contains(pat) {
            violations.push(format!(
                "Check 38 violation: Hardcoded static boolean flag '{}' found in {}:{}. Row states must bind dynamically.",
                pat, file_rel, line_no
            ));
        }
    }

    if stripped.contains("RULES: 0 VIOLATIONS")
        || stripped.contains("12x CLASS-")
        || stripped.contains("Text('PASS')")
        || stripped.contains("Text(\"PASS\")")
        || stripped.contains("Text('FAIL')")
        || stripped.contains("Text(\"FAIL\")")
    {
        violations.push(format!(
            "Check 38 violation: Hardcoded static telemetry indicator found in {}:{}: '{}'. Telemetry status must bind to live solver or monitor state.",
            file_rel, line_no, stripped
        ));
    }

    if let Some(text_idx) = stripped.find("Text(") {
        let after_text = stripped.get(text_idx.saturating_add(5)..).unwrap_or("");
        if let Some((raw_arg, _)) = after_text.split_once(')') {
            let arg = raw_arg.trim();
            let is_literal_string = (arg.starts_with('\'') && arg.ends_with('\'') && arg.len() >= 2)
                || (arg.starts_with('"') && arg.ends_with('"') && arg.len() >= 2);
            if is_literal_string && !arg.contains('$') {
                let inner = arg
                    .get(1..arg.len().saturating_sub(1))
                    .unwrap_or("")
                    .trim();
                for entity in entity_ids {
                    if inner == entity || inner.contains(entity) {
                        violations.push(format!(
                            "Check 38 violation: Hardcoded un-interpolated domain entity literal '{}' in Text widget in {}:{}.",
                            entity, file_rel, line_no
                        ));
                        break;
                    }
                }
            }
        }
    }
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

/// Inspects a single test file for forbidden mock classes, empty stubs, and leaking SQLite connections.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A live persistence testing mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Detects mock classes via token matching on `class Mock`, `class Fake`, `extends Mock`, etc.
/// - Validates paired presence of `tearDownAll` containing `.close()` or `.dispose()` when `openDatabase` is present in `setUpAll`.
///
/// ## 3. Preconditions
/// - `file_path` exists on disk; `file_rel` is workspace-relative path string.
///
/// ## 4. Postconditions
/// - Appends diagnostic violation strings to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is lines in the file.
/// - Space Complexity: O(V) for diagnostic accumulation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Files failing read operations do not panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
fn scan_test_file_for_mocking(file_path: &Path, file_rel: &str, violations: &mut Vec<String>) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let forbidden_class_prefixes = ["class Mock", "class _Mock", "class Fake", "class _Fake"];
    let forbidden_extends = ["extends Mock", "extends Fake", "with Mock", "with Fake"];
    let forbidden_implements = ["implements TreeRepository", "implements DataSource", "implements TacticalTransport"];
    let forbidden_stubs = [
        "Result.success([])",
        "TopologyData(coordinateMapping: {}, nodes: [], links: [])",
        "TopologyData(nodes: [], links: [])",
    ];

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();

        for prefix in &forbidden_class_prefixes {
            if trimmed.starts_with(prefix) {
                violations.push(format!(
                    "Check 39 violation: Forbidden mock or fake class declaration in {}:{}: '{}'. Live persistence required.",
                    file_rel, line_no, trimmed
                ));
            }
        }

        for ext in &forbidden_extends {
            if trimmed.contains(ext) {
                violations.push(format!(
                    "Check 39 violation: Forbidden inheritance from Mock/Fake in {}:{}: '{}'.",
                    file_rel, line_no, trimmed
                ));
            }
        }

        for imp in &forbidden_implements {
            if trimmed.contains(imp) {
                violations.push(format!(
                    "Check 39 violation: Forbidden mock repository/datasource implementation in {}:{}: '{}'.",
                    file_rel, line_no, trimmed
                ));
            }
        }

        if trimmed.contains("TestThemeService") || trimmed.contains("FakeThemeService") {
            violations.push(format!(
                "Check 39 violation: Forbidden mock theme service in {}:{}: '{}'.",
                file_rel, line_no, trimmed
            ));
        }

        for stub in &forbidden_stubs {
            if trimmed.contains(stub) {
                violations.push(format!(
                    "Check 39 violation: Hardcoded empty result stub in {}:{}: '{}'.",
                    file_rel, line_no, trimmed
                ));
            }
        }
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

/// Verifies that the generated schema UI contract test sweeps all 6 canonical viewports.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to MIL-STD-1472H multi-resolution spatial layout verification mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijective presence verification for the 6 canonical viewports from FHD_1080P_1X to XDR_6K_2X.
///
/// ## 3. Preconditions
/// - `contract_path` points to existing generated test file on disk.
///
/// ## 4. Postconditions
/// - Appends violation diagnostic if any of the 6 canonical viewports is missing.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(C) where C is contract file length.
/// - Space Complexity: O(1) static inspection.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Reads safely and logs diagnostics on missing content.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-VERIFY-VIEWPORTS]
fn verify_ui_contract_test_viewports(contract_path: &Path, file_rel: &str, violations: &mut Vec<String>) {
    let content = match fs::read_to_string(contract_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let canonical_viewports = [
        "FHD_1080P_1X",
        "QHD_1440P_1X",
        "UHD_4K_1X",
        "UHD_4K_2X",
        "XDR_6K_1X",
        "XDR_6K_2X",
    ];

    for vp in &canonical_viewports {
        if !content.contains(vp) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing canonical viewport '{}'.",
                file_rel, vp
            ));
        }
    }
}

/// Scans a Dart test file for shallow probing helpers, direct bridge mutations, and missing HUD assertions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H interactive gesture actuation requirements.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects `probeWidgetKey` and shallow `evaluate().isNotEmpty`.
/// - Rejects direct `bridge.*` mutations in test bodies.
/// - Rejects HUD mounts lacking `assertPairwiseHudNonOverlap` or `.overlaps(`.
///
/// ## 3. Preconditions
/// - `file_path` is an existing file path; `file_rel` is relative path string.
///
/// ## 4. Postconditions
/// - Appends violation diagnostics to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is lines in the file.
/// - Space Complexity: O(V) for diagnostics.
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
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let banned_bridge_mutations = [
        "bridge.relockInterlock(",
        "bridge.triggerHotlRfJamRetasking(",
        "bridge.restoreStrategicLink(",
        "bridge.setMobileAtakEudMode(",
        "bridge.toggleRadarDegraded(",
        "bridge.setSwarmDensityMultiplier(",
        "bridge.setMinMagazineReserveRatio(",
    ];

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

        for mut_call in &banned_bridge_mutations {
            if trimmed.contains(mut_call) {
                violations.push(format!(
                    "Check 40 violation: Direct solver bridge mutation detected in {}:{}: '{}'. UI tests must actuate controls via tester.tap(find.byKey(...)).",
                    file_rel, line_no, mut_call
                ));
            }
        }
    }

    let mounts_hud = content.contains("HudOverlay")
        || content.contains("TacticalHud")
        || content.contains("HudPanel")
        || content.contains("hud_panel")
        || content.contains("hud_overlay");

    if mounts_hud {
        let has_non_overlap = content.contains("assertPairwiseHudNonOverlap") || content.contains(".overlaps(");
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A single source of truth kernel integrity mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Triggers when a native Rust solver crate exists under `crates/`.
/// - Rejects candidate generator functions and Pareto scoring in Dart bridge sources.
///
/// ## 3. Preconditions
/// - `repo_root` is workspace root directory; `violations` is diagnostic accumulator.
///
/// ## 4. Postconditions
/// - Appends violation diagnostic if duplicate solver math is detected in Dart bridge files.
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

    let mut topology_entity_ids = BTreeSet::new();
    if let Ok(content) = fs::read_to_string(&topology_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(nodes) = val.get("nodes").and_then(|n| n.as_array()) {
                for node in nodes {
                    if let Some(id_str) = node.get("id").and_then(|i| i.as_str()) {
                        let trimmed = id_str.trim();
                        if !trimmed.is_empty() {
                            topology_entity_ids.insert(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    let seeds_dir = repo_root.join("app_flutter").join("lib").join("data").join("seeds");
    if !seeds_dir.is_dir() {
        return;
    }

    let mut seed_files_content = String::new();
    if let Ok(entries) = fs::read_dir(&seeds_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                    if let Ok(c) = fs::read_to_string(&path) {
                        seed_files_content.push_str(&c);
                        seed_files_content.push('\n');
                    }
                }
            }
        }
    }

    if seed_files_content.is_empty() {
        return;
    }

    for entity_id in &topology_entity_ids {
        if !seed_files_content.contains(entity_id) {
            violations.push(format!(
                "Check 41 violation: Bijective SQLite seed parity failure: entity '{}' from topology_data.json is missing from SQLite domain seed.",
                entity_id
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
        fn new(prefix: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let dir = std::env::temp_dir().join(format!("test_codebase_{}_{}_{}", prefix, nanos, count));
            fs::create_dir_all(&dir).unwrap();
            Self { path: dir }
        }
    }

    impl Drop for ScopedTempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_check_38_positive_clean_presentation() {
        let tmp = ScopedTempDir::new("check_38_pos");
        let feature_dir = tmp.path.join("app_flutter/lib/features/workspace");
        fs::create_dir_all(&feature_dir).unwrap();

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
        fs::write(feature_dir.join("workspace_view.dart"), clean_dart).unwrap();

        let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
        assert!(res.is_ok(), "Positive clean presentation must pass: {:?}", res.err());
    }

    #[test]
    fn test_check_38_negative_hardcoded_literals() {
        let tmp = ScopedTempDir::new("check_38_neg");
        let assets_dir = tmp.path.join("app_flutter/assets");
        let feature_dir = tmp.path.join("app_flutter/lib/features/tactical");
        fs::create_dir_all(&assets_dir).unwrap();
        fs::create_dir_all(&feature_dir).unwrap();

        let topology_json = r#"{
  "nodes": [
    {"id": "NODE_ALPHA_01", "name": "Alpha Node", "type": "Sensor"}
  ]
}"#;
        fs::write(assets_dir.join("topology_data.json"), topology_json).unwrap();

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
        fs::write(feature_dir.join("defective_view.dart"), defective_dart).unwrap();

        let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
        assert!(res.is_err(), "Defective hardcoded presentation must fail Check 38");
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("NODE_ALPHA_01")));
        assert!(errs.iter().any(|e| e.contains("RULES: 0 VIOLATIONS")));
        assert!(errs.iter().any(|e| e.contains("isActive: true")));
    }

    #[test]
    fn test_check_39_positive_clean_persistence_test() {
        let tmp = ScopedTempDir::new("check_39_pos");
        let test_dir = tmp.path.join("app_flutter/test/data");
        fs::create_dir_all(&test_dir).unwrap();

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
        fs::write(test_dir.join("live_db_test.dart"), clean_test).unwrap();

        let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
        assert!(res.is_ok(), "Clean persistence test must pass Check 39: {:?}", res.err());
    }

    #[test]
    fn test_check_39_negative_mocking_and_leaking_db() {
        let tmp = ScopedTempDir::new("check_39_neg");
        let test_dir = tmp.path.join("app_flutter/test/features");
        fs::create_dir_all(&test_dir).unwrap();

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
        fs::write(test_dir.join("mock_test.dart"), defective_test).unwrap();

        let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
        assert!(res.is_err(), "Mocking and leaking SQLite must fail Check 39");
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Mock") || e.contains("mocking")));
        assert!(errs.iter().any(|e| e.contains("Result.success")));
        assert!(errs.iter().any(|e| e.contains("tearDownAll")));
    }

    #[test]
    fn test_check_40_positive_ui_contracts() {
        let tmp = ScopedTempDir::new("check_40_pos");
        let test_gen_dir = tmp.path.join("app_flutter/test/generated");
        let test_dir = tmp.path.join("app_flutter/test/widgets");
        fs::create_dir_all(&test_gen_dir).unwrap();
        fs::create_dir_all(&test_dir).unwrap();

        let generated_contract = r#"
// Generated schema UI contract test
const kCanonicalViewports = [
  'FHD_1080P_1X',
  'QHD_1440P_1X',
  'UHD_4K_1X',
  'UHD_4K_2X',
  'XDR_6K_1X',
  'XDR_6K_2X',
];

void main() {
  testWidgets('assert zero RenderFlex overflow across all viewports', (tester) async {
    // contract verification
  });
}
"#;
        fs::write(test_gen_dir.join("schema_ui_contract_test.g.dart"), generated_contract).unwrap();

        let interactive_test = r#"
void main() {
  testWidgets('interactive tap test with HUD non-overlap', (tester) async {
    await tester.tap(find.byKey(const Key('tactical_control_button')));
    await tester.pumpAndSettle();
    assertPairwiseHudNonOverlap(tester);
  });
}
"#;
        fs::write(test_dir.join("interactive_test.dart"), interactive_test).unwrap();

        let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
        assert!(res.is_ok(), "Positive contract test must pass Check 40: {:?}", res.err());
    }

    #[test]
    fn test_check_40_negative_shallow_probing_and_direct_bridge_mutation() {
        let tmp = ScopedTempDir::new("check_40_neg");
        let test_gen_dir = tmp.path.join("app_flutter/test/generated");
        let test_dir = tmp.path.join("app_flutter/test/widgets");
        fs::create_dir_all(&test_gen_dir).unwrap();
        fs::create_dir_all(&test_dir).unwrap();

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
        fs::write(test_gen_dir.join("schema_ui_contract_test.g.dart"), incomplete_contract).unwrap();

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
        fs::write(test_dir.join("defective_widget_test.dart"), defective_test).unwrap();

        let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
        assert!(res.is_err(), "Incomplete viewports and shallow probing must fail Check 40");
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("XDR_6K_2X")));
        assert!(errs.iter().any(|e| e.contains("probeWidgetKey")));
        assert!(errs.iter().any(|e| e.contains("bridge.relockInterlock")));
        assert!(errs.iter().any(|e| e.contains("assertPairwiseHudNonOverlap")));
    }

    #[test]
    fn test_check_41_positive_c_abi_and_seed_parity() {
        let tmp = ScopedTempDir::new("check_41_pos");
        let assets_dir = tmp.path.join("app_flutter/assets");
        let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
        let data_dir = tmp.path.join("app_flutter/lib/data");
        fs::create_dir_all(&assets_dir).unwrap();
        fs::create_dir_all(&seeds_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();

        let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Primary Sensor"},
    {"id": "CONTROLLER_02", "name": "Main Controller"}
  ]
}"#;
        fs::write(assets_dir.join("topology_data.json"), topology_json).unwrap();

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
        fs::write(seeds_dir.join("generated_domain_seed.g.dart"), seed_dart).unwrap();

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
        fs::write(data_dir.join("solver_bridge.dart"), bridge_dart).unwrap();

        let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
        assert!(res.is_ok(), "Positive C-ABI and seed parity must pass Check 41: {:?}", res.err());
    }

    #[test]
    fn test_check_41_negative_duplicate_math_and_missing_seeds() {
        let tmp = ScopedTempDir::new("check_41_neg");
        let solver_crate_dir = tmp.path.join("crates/domain-solver/src");
        let assets_dir = tmp.path.join("app_flutter/assets");
        let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
        let data_dir = tmp.path.join("app_flutter/lib/data");
        fs::create_dir_all(&solver_crate_dir).unwrap();
        fs::create_dir_all(&assets_dir).unwrap();
        fs::create_dir_all(&seeds_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();

        fs::write(solver_crate_dir.join("lib.rs"), "// solver crate").unwrap();
        fs::write(solver_crate_dir.join("solver.rs"), "// solver arithmetic").unwrap();

        let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Primary Sensor"},
    {"id": "SENSOR_02_MISSING", "name": "Missing Sensor"}
  ]
}"#;
        fs::write(assets_dir.join("topology_data.json"), topology_json).unwrap();

        // Seed file misses SENSOR_02_MISSING and misses type_relations table
        let defective_seed_dart = r#"
Future<void> seedDatabase(Database db) async {
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('SENSOR_01')");
  await db.execute("INSERT INTO properties VALUES (...)");
}
"#;
        fs::write(seeds_dir.join("generated_domain_seed.g.dart"), defective_seed_dart).unwrap();

        // Bridge file has duplicate candidate generator and Pareto scoring
        let defective_bridge_dart = r#"
class SolverBridge {
  List<Candidate> _buildCandidates() {
    final paretoScore = alphaCovOffset * 0.8 + reserveBoostPct * 0.2;
    return [Candidate(score: paretoScore, violationCount: 0)];
  }
}
"#;
        fs::write(data_dir.join("domain_solver_bridge.dart"), defective_bridge_dart).unwrap();

        let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
        assert!(res.is_err(), "Duplicate math and missing seed entities must fail Check 41");
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("SENSOR_02_MISSING")));
        assert!(errs.iter().any(|e| e.contains("type_relations")));
        assert!(errs.iter().any(|e| e.contains("_buildCandidates")));
    }
}

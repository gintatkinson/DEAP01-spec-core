//! Unit tests for `source_ast` language extraction and CST parser integration. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D deterministic verification.
//! Validates Rust and Dart source code parsing, AST extraction, and syntax error detection.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Tests run deterministically without reliance on global state or ambient files.
//! - Exercises edge cases in delimiter nesting, string escaping, and docstring bindings.
//!
//! ## 3. Preconditions
//! - Valid UTF-8 test inputs.
//!
//! ## 4. Postconditions
//! - Asserts exact AST extraction and syntax validation outcomes.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Bounded execution time per test case.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Test failures panic or return structured `Result<(), String>` errors.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608], [REQ-FEAT-42-VALIDATE-SYNTAX-LANG]

    use super::*;

    /// Tests Dart stateful widget AST extraction and SysML lowering. Realises: [REQ-0201]
    #[test]
    fn test_parse_dart_stateful_widget_and_lower_to_sysml() -> Result<(), String> {
        let dart_src = r#"import 'package:flutter/material.dart';

/// Realises: [REQ-UI-01]
class WidgetAlpha extends StatefulWidget {
  final String title;
  final int refreshRateHz;
  final VoidCallback onRefresh;
  final ValueChanged<int> onRateChanged;

  const WidgetAlpha({
    super.key,
    required this.title,
    required this.refreshRateHz,
    required this.onRefresh,
    required this.onRateChanged,
  });

  @override
  State<WidgetAlpha> createState() => _WidgetAlphaState();
}

class _WidgetAlphaState extends State<WidgetAlpha> {
  bool isExpanded = false;

  void togglePanel(bool nextState) {
    assert(nextState == true || nextState == false);
    isExpanded = nextState;
  }
}
"#;
        let items = parse_dart_source_ast(dart_src, "lib/widget_alpha.dart")?;
        let pkg = lower_source_ast_to_package(&items, "UiAlphaPkg");
        let panel = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "WidgetAlpha")
            .ok_or_else(|| "WidgetAlpha PartDef".to_string())?;
        if !panel.attributes.iter().any(|a| a.name == "title") {
            return Err("Missing 'title' attribute on WidgetAlpha".to_string());
        }
        if !panel.attributes.iter().any(|a| a.name == "refreshRateHz") {
            return Err("Missing 'refreshRateHz' attribute on WidgetAlpha".to_string());
        }
        if !panel.ports.iter().any(|p| p.name == "onRefresh") {
            return Err("Missing 'onRefresh' port on WidgetAlpha".to_string());
        }
        if !panel.ports.iter().any(|p| p.name == "onRateChanged") {
            return Err("Missing 'onRateChanged' port on WidgetAlpha".to_string());
        }
        if !panel
            .satisfied_requirements
            .contains(&"REQ-UI-01".to_string())
        {
            return Err("Missing 'REQ-UI-01' satisfied requirement on WidgetAlpha".to_string());
        }
        if panel.states.is_empty() {
            return Err("Expected non-empty states on WidgetAlpha".to_string());
        }
        Ok(())
    }

    /// Tests Rust struct and impl AST extraction and SysML lowering. Realises: [REQ-0201]
    #[test]
    fn test_parse_rust_struct_and_impl_and_lower_to_sysml() -> Result<(), String> {
        let rust_src = r#"/// Realises: [REQ-BUF-01]
pub struct ComponentBuffer {
    pub capacity: usize,
    pub count: usize,
}

impl ComponentBuffer {
    pub fn push_item(&mut self, value: u32) -> Result<usize, String> {
        debug_assert!(self.count <= self.capacity);
        Ok(self.count)
    }
}
"#;
        let items = parse_rust_source_ast(rust_src, "src/buffer.rs")?;
        let pkg = lower_source_ast_to_package(&items, "BufferPkg");
        let buf = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "ComponentBuffer")
            .ok_or_else(|| "ComponentBuffer PartDef".to_string())?;
        if buf.attributes.len() != 2 {
            return Err(format!(
                "Expected 2 attributes on ComponentBuffer, got {}",
                buf.attributes.len()
            ));
        }
        if !buf
            .actions
            .iter()
            .any(|a| a.name == "push_item" && a.in_params.len() == 1 && a.out_params.len() == 1)
        {
            return Err("Missing expected 'push_item' action on ComponentBuffer".to_string());
        }
        if buf.constraints.is_empty() {
            return Err("Expected non-empty constraints on ComponentBuffer".to_string());
        }
        if !buf
            .satisfied_requirements
            .contains(&"REQ-BUF-01".to_string())
        {
            return Err(
                "Missing 'REQ-BUF-01' satisfied requirement on ComponentBuffer".to_string(),
            );
        }
        Ok(())
    }

    /// Tests syntax error diagnostic for unclosed brace delimiters. Realises: [REQ-FEAT-42-VALIDATE-SYNTAX-LANG]
    #[test]
    fn test_syntax_error_unclosed_brace_fails_closed_with_line_and_col() -> Result<(), String> {
        let bad_rust = "pub struct Broken {\n    pub x: u32,\n";
        let err = match parse_rust_source_ast(bad_rust, "src/broken.rs") {
            Ok(_) => return Err("Expected syntax error for unclosed brace".to_string()),
            Err(e) => e,
        };
        if !err.contains("src/broken.rs:1:19") || !err.contains("unclosed delimiter '{'") {
            return Err(format!("Expected line/col diagnostic, got: {err}"));
        }
        Ok(())
    }

    /// Tests Dart raw strings and interpolation with nested quotes. Realises: [REQ-FEAT-42-VALIDATE-SYNTAX-LANG]
    #[test]
    fn test_validate_dart_raw_strings_and_interpolated_nested_quotes() -> Result<(), String> {
        let dart_src = r#"
class PathFormatter {
  String normalize(Map<String, String> map) {
    final sep = r'\';
    final rawPath = r"C:\path\";
    final msg = "hello ${map["world"]}!";
    final msg2 = 'hello ${map['world']}!';
    return "$sep$rawPath$msg$msg2";
  }
}
"#;
        validate_source_syntax_for_language(
            dart_src,
            "lib/path_formatter.dart",
            SourceLanguage::Dart,
        )?;

        let rust_utf8_char =
            "pub fn is_up_arrow(ch: char) -> bool {\n    ch == '↑' || ch == '{'\n}\n";
        validate_source_syntax_for_language(rust_utf8_char, "src/arrow.rs", SourceLanguage::Rust)?;
        Ok(())
    }

    /// Verifies comment stripping, delimiter depth tracking with char literals, and Check 34 range-slice absence.
    ///
    /// # Summary
    /// Tests `strip_line_comment_preserving_strings` and `update_ast_delimiter_depths` on character literals,
    /// hashed raw strings, and unclosed `${` strings, and verifies zero raw range-slice indexing or `#[allow(...)]` in production code.
    ///
    /// # Preconditions
    /// - Invoked under `cargo test -p deap-core --lib`.
    ///
    /// # Postconditions
    /// - Verifies exact stripped slices, delimiter depth counters, positive/negative `contains_direct_range_slice` probes, and absence of raw range slices or `#[allow(clippy::type_complexity)]` in production code.
    ///
    /// # Invariants
    /// - `strip_line_comment_preserving_strings` preserves `'/'`, `"http://..."`, and `r#"http://... // keep"#` while stripping trailing `//`.
    /// - `update_ast_delimiter_depths` ignores `'{'` and `'('` inside character literals, hashed raw strings, and unclosed `"${"` literals.
    /// - Production code prior to `#[cfg(test)]` contains zero raw range-slice indexing expressions and zero `#[allow(clippy::type_complexity)]` attributes.
    ///
    /// # Failure Modes
    /// - Returns `Err(String)` if comment stripping, delimiter depth tracking, or Check 34 self-verification checks fail.
    ///
    /// # Side Effects
    /// - None (pure in-memory unit test).
    ///
    /// # Determinism
    /// - Deterministic single-threaded unit test execution.
    ///
    /// # Parameters
    /// - None (unit test).
    ///
    /// # Returns
    /// - `Result<(), String>` (`Ok(())` on check success).
    ///
    /// # Errors
    /// - Returns `Err(String)` if any invariant check fails.
    ///
    /// # Panics
    /// - Never panics (`#![forbid(clippy::indexing_slicing)]` and fallible `Result` return compliant).
    ///
    /// # Safety
    /// - Safe pure lexical unit test (`#![forbid(unsafe_code)]` compliant).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34 (`check_zero_regex`) non-panicking slice invariants (refs #492).
    /// Ensures `strip_line_comment_preserving_strings` and `update_ast_delimiter_depths` preserve
    /// character literals, hashed raw strings, and unclosed `${` tokens and execute without direct range-slice indexing.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `strip_line_comment_preserving_strings` preserves `'/'`, `"http://..."`, and `r#"... // ..."#` while stripping trailing `//`.
    /// - `update_ast_delimiter_depths` ignores `'{'` and `'('` inside character literals, `r#"..."#`, and unclosed `"${"` literals.
    /// - Production code prior to `#[cfg(test)]` contains zero raw range-slice indexing expressions.
    ///
    /// ## 3. Preconditions
    /// - Invoked under `cargo test -p deap-core --lib`.
    ///
    /// ## 4. Postconditions
    /// - Verifies exact stripped slices, delimiter depth counters, and absence of raw range slices in production code.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is byte length of `source_ast.rs`.
    /// - Space Complexity: O(1) auxiliary heap allocations.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Fails test execution immediately via `Err(String)` if any raw range-slice token or `#[allow(clippy::type_complexity)]` remains in production code.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SOURCE-AST-STRIP-LINE-COMMENT], [REQ-FEAT-42-UPDATE-AST-DELIMITER-DEPTHS]
    #[test]
    fn test_strip_comment_and_delimiter_depths_zero_range_slice_indexing() -> Result<(), String> {
        let stripped = strip_line_comment_preserving_strings(
            "let ch = '/'; let url = \"http://example.com\"; // trailing comment",
        );
        if stripped != "let ch = '/'; let url = \"http://example.com\"; " {
            return Err(format!("Unexpected stripped line: {stripped}"));
        }

        let stripped_raw = strip_line_comment_preserving_strings(
            "let raw = r#\"http://example.com // keep\"#; // strip",
        );
        if stripped_raw != "let raw = r#\"http://example.com // keep\"#; " {
            return Err(format!("Unexpected stripped_raw line: {stripped_raw}"));
        }

        let stripped_raw_quote = strip_line_comment_preserving_strings(
            r##"let q = r#"quote " // not a comment"#; // comment"##,
        );
        if stripped_raw_quote != r##"let q = r#"quote " // not a comment"#; "## {
            return Err(format!(
                "Unexpected stripped_raw_quote line: {stripped_raw_quote}"
            ));
        }

        let stripped_raw_backslash =
            strip_line_comment_preserving_strings(r##"let p = r#"C:\"#; // comment"##);
        if stripped_raw_backslash != r##"let p = r#"C:\"#; "## {
            return Err(format!(
                "Unexpected stripped_raw_backslash line: {stripped_raw_backslash}"
            ));
        }

        let mut brace_depth: usize = 0;
        let mut paren_depth: usize = 0;
        update_ast_delimiter_depths(
            "if ch == '{' || ch == '(' || ch == '↑' {",
            &mut brace_depth,
            &mut paren_depth,
        );
        if brace_depth != 1 || paren_depth != 0 {
            return Err(format!(
                "Unexpected delimiter depths: brace={brace_depth}, paren={paren_depth}"
            ));
        }

        let mut brace_depth_interp: usize = 0;
        let mut paren_depth_interp: usize = 0;
        update_ast_delimiter_depths(
            "let msg = \"unclosed ${ ( in rust\"; if ready {",
            &mut brace_depth_interp,
            &mut paren_depth_interp,
        );
        if brace_depth_interp != 1 || paren_depth_interp != 0 {
            return Err(format!(
                "Unexpected interp depths: brace={brace_depth_interp}, paren={paren_depth_interp}"
            ));
        }

        let mut brace_depth_starts_with: usize = 0;
        let mut paren_depth_starts_with: usize = 0;
        update_ast_delimiter_depths(
            "if s.starts_with(\"${\") {",
            &mut brace_depth_starts_with,
            &mut paren_depth_starts_with,
        );
        if brace_depth_starts_with != 1 || paren_depth_starts_with != 0 {
            return Err(format!(
                "Unexpected starts_with depths: brace={brace_depth_starts_with}, paren={paren_depth_starts_with}"
            ));
        }

        let mut brace_depth_starts_with_pair: usize = 0;
        let mut paren_depth_starts_with_pair: usize = 0;
        update_ast_delimiter_depths(
            "if s.starts_with(\"${\") && other == \"}\" {",
            &mut brace_depth_starts_with_pair,
            &mut paren_depth_starts_with_pair,
        );
        if brace_depth_starts_with_pair != 1 || paren_depth_starts_with_pair != 0 {
            return Err(format!(
                "Unexpected starts_with_pair depths: brace={brace_depth_starts_with_pair}, paren={paren_depth_starts_with_pair}"
            ));
        }

        let dollar_brace_rust = r#"pub fn check_token(s: &str, other: &str) -> bool {
    if s.starts_with("${") && other == "}" {
        return true;
    }
    false
}

pub struct AfterDollarBrace {
    pub ok: bool,
}

pub fn after_dollar_brace() -> bool {
    true
}
"#;
        let parsed_items = parse_rust_source_ast(dollar_brace_rust, "src/dollar_brace.rs")?;
        if !parsed_items
            .iter()
            .any(|item| item.kind == SourceItemKind::Struct && item.name == "AfterDollarBrace")
        {
            return Err("Expected AfterDollarBrace struct after ${ literal".to_string());
        }
        if !parsed_items
            .iter()
            .any(|item| item.kind == SourceItemKind::Function && item.name == "after_dollar_brace")
        {
            return Err("Expected after_dollar_brace function after ${ literal".to_string());
        }

        let mut brace_depth_raw: usize = 0;
        let mut paren_depth_raw: usize = 0;
        update_ast_delimiter_depths(
            "let raw = r#\"{ ( [\"#; if ok {",
            &mut brace_depth_raw,
            &mut paren_depth_raw,
        );
        if brace_depth_raw != 1 || paren_depth_raw != 0 {
            return Err(format!(
                "Unexpected raw depths: brace={brace_depth_raw}, paren={paren_depth_raw}"
            ));
        }

        let positive_probes = [
            "let x = &s[0..1];",
            "let p = r\"C:\\\\\"; let y = &s[0..1];",
            "let z = arr[find(\"]\") + s[0..1]];",
            "let w = call()[start..end];",
        ];
        for probe in positive_probes {
            if !contains_direct_range_slice(probe) {
                return Err(format!(
                    "Expected contains_direct_range_slice to return true for: {probe}"
                ));
            }
        }

        let negative_probes = [
            "let a = map[\"a..b\"];",
            "let b = map[r#\"a..b\"#];",
            "let c = r#\"quote \" then s[0..1]\"#;",
            "let d = arr[find(\"..\")];",
            "let r = 0..10;",
        ];
        for probe in negative_probes {
            if contains_direct_range_slice(probe) {
                return Err(format!(
                    "Expected contains_direct_range_slice to return false for: {probe}"
                ));
            }
        }

        let self_source = include_str!("source_ast.rs");
        let cfg_test_mod_marker = ["\n#[cfg(test)]", "\nmod tests"].concat();
        let prod_source = match self_source.split_once(&cfg_test_mod_marker) {
            Some((prod, _)) => prod,
            None => self_source,
        };
        for (line_idx, raw_line) in prod_source.lines().enumerate() {
            let trimmed = raw_line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }
            let code = strip_line_comment_preserving_strings(trimmed);
            if contains_direct_range_slice(code) {
                return Err(format!(
                    "Check 34 violation: direct range-slice indexing found at line {}: {}",
                    line_idx.saturating_add(1),
                    trimmed
                ));
            }
        }
        if prod_source.contains("#[allow(clippy::type_complexity)]") {
            return Err(
                "Check 34 violation: #[allow(clippy::type_complexity)] must be eliminated via type aliases"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// Verifies `scan_rust_line_preserving_lex_state` on multi-lifetime signatures, char literals, block comments, and raw strings.
    ///
    /// # Summary
    /// Tests `scan_rust_line_preserving_lex_state` and `parse_rust_source_ast` on multiple long lifetime parameters,
    /// escaped and Unicode character literals, mid-line closing block comments, and multi-hash raw strings (`r##"..."##`).
    ///
    /// # Preconditions
    /// - Invoked under `cargo test -p deap-core --lib`.
    ///
    /// # Postconditions
    /// - Confirms `RustStringScannerState` returns to `Normal`, char literals are blanked out, lifetimes are preserved,
    ///   and declarations after mid-line `*/` or `r##"..."##` are extracted accurately.
    ///
    /// # Invariants
    /// - Never treats a pair of long lifetime parameters (`'lifetime_alpha, 'lifetime_beta`) as a character literal.
    /// - Never drops code tokens appearing after `*/` on the closing line of a multiline block comment.
    ///
    /// # Failure Modes
    /// - Returns `Err(String)` if lexical state transitions or AST extraction fail.
    ///
    /// # Side Effects
    /// - None (pure in-memory unit test).
    ///
    /// # Determinism
    /// - Deterministic single-threaded execution.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34/Check 36 shared Rust lexical state machine verification.
    /// Proves that `scan_rust_line_preserving_lex_state` handles lifetimes, character literals, nested block comments,
    /// and multi-hash raw strings without magic byte-offset limits.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Character literals (`'{'`, `'\''`, `'\u{1F600}'`) are blanked into spaces.
    /// - Lifetimes (`'lifetime_alpha`, `'lifetime_beta`) retain their leading `'` and identifier tokens.
    /// - Multi-hash raw strings (`r##"..."#"##`) remain in `RawString(2)` until `"##` is encountered.
    ///
    /// ## 3. Preconditions
    /// - Executed by `cargo test -p deap-core`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when all scanner and AST extraction checks pass.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over the test fixture lines.
    /// - Space Complexity: O(N) for sanitized strings and extracted AST items.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Fails immediately via `Err(String)` on state mismatch or missing AST items.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SOURCE-AST-SCAN-RUST-LINE-LEX-STATE], [REQ-SOURCE-AST-RUST-SCANNER-STATE]
    #[test]
    fn test_scan_rust_line_preserving_lex_state_char_literals_and_lifetimes() -> Result<(), String>
    {
        let mut state = RustStringScannerState::Normal;
        let line = "pub fn pick<'lifetime_alpha, 'lifetime_beta>(x: &'lifetime_alpha str, c: char) -> bool where 'lifetime_alpha: 'lifetime_beta { c == '{' || c == '\\'' || c == '\\u{1F600}' }";
        let sanitized = scan_rust_line_preserving_lex_state(line, &mut state);
        if state != RustStringScannerState::Normal {
            return Err(format!("Expected Normal scanner state, got {state:?}"));
        }
        if !sanitized.contains("pub fn pick<'lifetime_alpha, 'lifetime_beta>") {
            return Err(format!(
                "Missing lifetime signature in sanitized line: {sanitized}"
            ));
        }
        if sanitized.contains("'{'") || sanitized.contains("1F600") {
            return Err(format!(
                "Character literals were not blanked in sanitized line: {sanitized}"
            ));
        }

        let rust_src = r###"/* multiline comment start
still in comment */ pub struct AfterComment {
    pub raw_field: usize,
}

pub fn with_multi_hash_raw<'lifetime_alpha, 'lifetime_beta>(
    input: &'lifetime_alpha str,
) -> &'lifetime_beta str
where
    'lifetime_alpha: 'lifetime_beta,
{
    let _raw = r##"inner "# still raw
/// fake doc inside raw string
pub struct FakeInsideRaw {}
closing now "##;
    input
}
"###;
        let items = parse_rust_source_ast(rust_src, "src/lex_test.rs")?;
        if !items.iter().any(|i| i.name == "AfterComment") {
            return Err("Missing 'AfterComment' item after multiline block comment".to_string());
        }
        if !items.iter().any(|i| i.name == "with_multi_hash_raw") {
            return Err("Missing 'with_multi_hash_raw' function item".to_string());
        }
        if items.iter().any(|i| i.name == "FakeInsideRaw") {
            return Err("Unexpected 'FakeInsideRaw' parsed from inside raw string".to_string());
        }
        Ok(())
    }

    /// Verifies semantic type aliases `ExtractedMethodSignature` and `ComponentMethodTable` for REQ-0601 AC-04.
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34/Check 36 verification for semantic type aliases (`REQ-0601`).
    /// ## 2. Invariants & Mathematical Properties
    /// - `extract_method_signature` returns `None` for non-callable `Struct` AST items and `Some` for callable `Function`, `Method`, and `Constructor` items.
    /// - `extract_component_method_table` and `lower_source_ast_to_package` preserve normalized parameters and return types.
    /// ## 3. Preconditions
    /// - Executed by `cargo test -p deap-core` on an abstract Rust component fixture (`ComponentAlpha`).
    /// ## 4. Postconditions
    /// - Negative probe (`Struct` -> `None`), `Function`/`Method`/`Constructor` signature checks, `ComponentMethodTable` lookup, and `PartDef` action lowering checks pass without indexing or `unwrap()`.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N log C) over parsed AST items. Space Complexity: O(N) fixture allocation.
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `Err(String)` immediately if signature extraction or table lookup deviates.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0601], [REQ-FEAT-42-EXTRACT-METHOD-SIGNATURE], [REQ-FEAT-42-EXTRACT-COMPONENT-METHOD-TABLE]
    #[test]
    fn test_req_0601_ac04_semantic_type_aliases() -> Result<(), String> {
        let rust_src = r#"pub struct ComponentAlpha {
    pub threshold: i64,
}

impl ComponentAlpha {
    pub fn process_signal(&self, input_val: i64) -> bool {
        input_val > 0
    }
}
"#;
        let mut items = parse_rust_source_ast(rust_src, "src/component_alpha.rs")?;
        let struct_item = items
            .iter()
            .find(|i| i.name == "ComponentAlpha")
            .ok_or_else(|| "Missing ComponentAlpha struct AST item".to_string())?;
        if extract_method_signature(struct_item).is_some() {
            return Err("Expected None from extract_method_signature for Struct item".to_string());
        }

        let method_item = items
            .iter()
            .find(|i| i.name == "process_signal")
            .ok_or_else(|| "Missing process_signal method AST item".to_string())?
            .clone();
        let extracted: ExtractedMethodSignature = extract_method_signature(&method_item)
            .ok_or_else(|| {
                "Expected Some(ExtractedMethodSignature) for process_signal".to_string()
            })?;
        if extracted.0 != vec![("input_val".to_string(), "Integer".to_string())] {
            return Err(format!("Unexpected in_params: {:?}", extracted.0));
        }
        if extracted.1 != vec![("result".to_string(), "Boolean".to_string())] {
            return Err(format!("Unexpected out_params: {:?}", extracted.1));
        }
        if extracted.2 != Some("Boolean".to_string()) {
            return Err(format!("Unexpected return_type: {:?}", extracted.2));
        }

        let mut top_fn_item = method_item.clone();
        top_fn_item.kind = SourceItemKind::Function;
        top_fn_item.name = "reset_global".to_string();
        top_fn_item.owner_name = None;
        top_fn_item.return_type = Some("()".to_string());
        let top_extracted = extract_method_signature(&top_fn_item).ok_or_else(|| {
            "Expected Some(ExtractedMethodSignature) for top-level Function".to_string()
        })?;
        if !top_extracted.1.is_empty() || top_extracted.2.is_some() {
            return Err(format!(
                "Expected empty out_params and None ret_type for unit Function, got {top_extracted:?}"
            ));
        }

        let mut ctor_item = method_item;
        ctor_item.kind = SourceItemKind::Constructor;
        ctor_item.name = "new".to_string();
        ctor_item.owner_name = Some("ComponentAlpha".to_string());
        ctor_item.return_type = Some("ComponentAlpha".to_string());
        let ctor_extracted = extract_method_signature(&ctor_item)
            .ok_or_else(|| "Expected Some(ExtractedMethodSignature) for Constructor".to_string())?;
        if ctor_extracted.2 != Some("ComponentAlpha".to_string()) {
            return Err(format!(
                "Expected Some(\"ComponentAlpha\") return type for Constructor, got {:?}",
                ctor_extracted.2
            ));
        }

        items.push(top_fn_item);
        items.push(ctor_item);

        let table: ComponentMethodTable = extract_component_method_table(&items);
        let top_methods = table
            .get("TopLevel")
            .ok_or_else(|| "Missing TopLevel entry in ComponentMethodTable".to_string())?;
        if top_methods.len() != 1 || top_methods.first() != Some(&top_extracted) {
            return Err(format!(
                "Unexpected TopLevel methods entry: {top_methods:?}"
            ));
        }

        let methods = table
            .get("ComponentAlpha")
            .ok_or_else(|| "Missing ComponentAlpha entry in ComponentMethodTable".to_string())?;
        if methods.len() != 2
            || methods.first() != Some(&extracted)
            || methods.get(1) != Some(&ctor_extracted)
        {
            return Err(format!(
                "Unexpected ComponentAlpha methods entry: {methods:?}"
            ));
        }

        let pkg = lower_source_ast_to_package(&items, "PkgAlpha");
        let part = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "ComponentAlpha")
            .ok_or_else(|| "Missing ComponentAlpha PartDef in PkgAlpha".to_string())?;
        if !part.actions.iter().any(|a| {
            a.name == "process_signal" && a.in_params.len() == 1 && a.out_params.len() == 1
        }) {
            return Err("Missing 'process_signal' action on ComponentAlpha PartDef".to_string());
        }
        Ok(())
    }

    /// Verifies that `discover_source_files` allows `.`-prefixed root directories while excluding `.cargo`, `target`, `node_modules`, and `tests` subdirectories.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_discover_source_files_excludes_hidden_caches_and_target`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Ensures bounded O(F log F) source discovery that never traverses hidden dependency caches or build/test trees.
    ///
    /// ## 3. Preconditions
    /// - Writable system temporary directory exists.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when only canonical `src/lib.rs` is discovered and `.cargo/`, `target/`, and `tests/` files are excluded.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(F log F) over fixture entries.
    /// - Space Complexity: O(F) auxiliary memory.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates filesystem errors via `Result<(), String>`.
    /// - State: Removes temporary directory before returning.
    /// - Action: Zero panics or unwraps.
    /// - Resource: Bounded temporary fixture directory.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-DISCOVER-SOURCE-FILES]
    #[test]
    fn test_discover_source_files_excludes_hidden_caches_and_target() -> Result<(), String> {
        let temp_dir =
            std::env::temp_dir().join(format!(".deap_test_src_discover_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let src_dir = temp_dir.join("src");
        let cargo_cache_dir = temp_dir.join(".cargo").join("registry").join("src");
        let target_dir = temp_dir.join("target").join("debug");
        let tests_dir = temp_dir.join("tests");
        std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&cargo_cache_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&tests_dir).map_err(|e| e.to_string())?;

        let valid_rs = src_dir.join("lib.rs");
        std::fs::write(&valid_rs, "pub struct ValidComponent;\n").map_err(|e| e.to_string())?;
        std::fs::write(cargo_cache_dir.join("cached.rs"), "pub struct CachedDep;\n")
            .map_err(|e| e.to_string())?;
        std::fs::write(target_dir.join("build_gen.rs"), "pub struct BuildGen;\n")
            .map_err(|e| e.to_string())?;
        std::fs::write(
            tests_dir.join("integration.rs"),
            "pub struct IntegrationFixture;\n",
        )
        .map_err(|e| e.to_string())?;

        let discovered = discover_source_files(&temp_dir, SourceLanguage::Rust);
        let _ = std::fs::remove_dir_all(&temp_dir);
        let files = discovered?;
        if files != vec![valid_rs] {
            return Err(format!("Expected only src/lib.rs, got {files:?}"));
        }
        Ok(())
    }

    /// Verifies that `parse_rust_source_ast` handles nested `/* ... /* ... */ ... */` block comments and `br"..."` / `br#"..."#` raw byte string literals (`#555`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_555_nested_block_comments_and_raw_byte_strings`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Nested block comments track nesting depth so unbalanced delimiters after an inner `*/` do not leak into top-level syntax validation.
    /// - Unclosed nested block comments (`depth > 0` at EOF) deterministically return an `"unclosed block comment"` syntax error.
    /// - Raw byte strings `br"\"` and `br#"unclosed " quote"#` are recognized as raw literals without escaping the closing quote or failing on interior quotes.
    ///
    /// ## 3. Preconditions
    /// - In-memory Rust source fixtures containing nested block comments and raw byte string literals.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when `ByteStringOwner` is extracted from valid source and unclosed nested block comments are rejected.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over fixture byte length.
    /// - Space Complexity: O(I) for extracted AST items.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` on assertion mismatch.
    /// - State: Stateless in-memory test.
    /// - Action: Zero panics, unwraps, or expectations.
    /// - Resource: Bounded stack and heap fixture strings.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-PARSE-RUST-AST]
    #[test]
    fn test_issue_555_nested_block_comments_and_raw_byte_strings() -> Result<(), String> {
        let valid_src = concat!(
            "/* outer /* inner */ { unmatched brace inside outer comment */\n",
            "pub fn use_raw_bytes() -> &'static [u8] {\n",
            "    let a = br\"\\\";\n",
            "    let b = br#\"unclosed \" quote\"#;\n",
            "    let _ = a;\n",
            "    b\n",
            "}\n",
            "pub struct ByteStringOwner {\n",
            "    pub id: u32,\n",
            "}\n",
        );
        let items = parse_rust_source_ast(valid_src, "issue_555_valid.rs")?;
        if !items
            .iter()
            .any(|item| item.kind == SourceItemKind::Struct && item.name == "ByteStringOwner")
        {
            return Err(format!(
                "Expected Struct ByteStringOwner in extracted AST items, got {items:?}"
            ));
        }

        let unclosed_src = "/* outer /* inner */ pub struct Broken {}\n";
        match parse_rust_source_ast(unclosed_src, "issue_555_unclosed.rs") {
            Err(err) if err.contains("unclosed block comment") => Ok(()),
            other => Err(format!(
                "Expected Err containing 'unclosed block comment', got {other:?}"
            )),
        }
    }

    /// Verifies that `classify_rust_item_line_ast` and `parse_rust_source_ast` extract `unsafe trait`, `unsafe fn`, and `extern "<ABI>" fn` declarations (`#556`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_556_classify_rust_item_line_ast_extracts_unsafe_and_extern_items`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `unsafe trait` declarations (with or without visibility modifiers) are classified as `SourceItemKind::Trait`.
    /// - `unsafe fn`, `const unsafe fn`, `async unsafe fn`, `extern "<ABI>" fn`, `unsafe extern "<ABI>" fn`, and `extern fn` declarations are classified as `SourceItemKind::Function`.
    ///
    /// ## 3. Preconditions
    /// - In-memory Rust item declaration lines and source fixtures containing `unsafe` and `extern` qualifiers.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when all `unsafe` and `extern` traits and functions are deterministically classified and extracted.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over fixture line length.
    /// - Space Complexity: O(I) for extracted identifiers and AST items.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` on classification mismatch.
    /// - State: Stateless in-memory test.
    /// - Action: Zero panics, unwraps, or expectations.
    /// - Resource: Bounded stack and heap fixture strings.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CLASSIFY-RUST-ITEM-AST]
    #[test]
    fn test_issue_556_classify_rust_item_line_ast_extracts_unsafe_and_extern_items(
    ) -> Result<(), String> {
        let cases = [
            (
                "pub unsafe trait Sendable {",
                SourceItemKind::Trait,
                "Sendable",
            ),
            (
                "unsafe trait RawMarker {",
                SourceItemKind::Trait,
                "RawMarker",
            ),
            (
                "pub unsafe fn raw_ptr_read() {",
                SourceItemKind::Function,
                "raw_ptr_read",
            ),
            (
                "unsafe fn internal_unchecked() {",
                SourceItemKind::Function,
                "internal_unchecked",
            ),
            (
                "pub const unsafe fn const_unchecked() {",
                SourceItemKind::Function,
                "const_unchecked",
            ),
            (
                "pub async unsafe fn async_unchecked() {",
                SourceItemKind::Function,
                "async_unchecked",
            ),
            (
                "pub extern \"C\" fn c_abi_entry() {",
                SourceItemKind::Function,
                "c_abi_entry",
            ),
            (
                "pub unsafe extern \"C\" fn unsafe_c_abi_entry() {",
                SourceItemKind::Function,
                "unsafe_c_abi_entry",
            ),
            (
                "pub const unsafe extern \"C\" fn const_unsafe_c_fn() {",
                SourceItemKind::Function,
                "const_unsafe_c_fn",
            ),
            (
                "extern fn default_abi_fn() {",
                SourceItemKind::Function,
                "default_abi_fn",
            ),
        ];

        for (line, expected_kind, expected_name) in cases {
            let actual = classify_rust_item_line_ast(line);
            let expected = Some((expected_kind, expected_name.to_string()));
            if actual != expected {
                return Err(format!(
                    "Expected {expected:?} for line '{line}', got {actual:?}"
                ));
            }
        }

        let src = concat!(
            "pub unsafe trait HardwareBus {\n",
            "    unsafe fn transfer(&self);\n",
            "}\n",
            "pub unsafe fn write_mmio_reg(addr: usize, val: u32) {\n",
            "    let _ = (addr, val);\n",
            "}\n",
            "pub unsafe extern \"C\" fn ffi_dispatch_irq(irq: u32) {\n",
            "    let _ = irq;\n",
            "}\n",
        );
        let items = parse_rust_source_ast(src, "issue_556.rs")?;
        let summary: Vec<(SourceItemKind, &str)> = items
            .iter()
            .map(|item| (item.kind, item.name.as_str()))
            .collect();
        let expected_summary = vec![
            (SourceItemKind::Trait, "HardwareBus"),
            (SourceItemKind::Method, "transfer"),
            (SourceItemKind::Function, "write_mmio_reg"),
            (SourceItemKind::Function, "ffi_dispatch_irq"),
        ];
        if summary != expected_summary {
            return Err(format!("Expected {expected_summary:?}, got {summary:?}"));
        }
        Ok(())
    }

    /// Verifies that `lower_source_ast_to_package` lowers `State<WidgetName>` fields, methods (`ActionDef`), `satisfied_requirements`, and `assert(...)` invariants (`ConstraintDef`) onto the owning `WidgetName` `PartDef` regardless of declaration order (`#557`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_557_dart_state_methods_and_asserts_lowered_onto_owning_widget_part_def`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - When `State<WidgetName>` appears before `WidgetName` in a Dart translation unit, `WidgetName` still receives all `State<WidgetName>` attributes, ports, state definitions, methods, requirement satisfactions, and assertion constraints, and retains its own doc comment.
    /// - Pass 2 forwards `State<WidgetName>` methods (`ActionDef`), `satisfied_requirements`, and `assert(...)` `ConstraintDef` invariants onto `WidgetName` without duplicating action or constraint names.
    ///
    /// ## 3. Preconditions
    /// - In-memory Dart source fixture declaring `_TelemetryPanelState extends State<TelemetryPanel>` prior to `TelemetryPanel extends StatefulWidget`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when `TelemetryPanel` in `pkg.part_defs` contains the expected `refreshHz` attribute, `updateTelemetry` action, `REQ-UI-TELEM-01` satisfaction, and `sample >= 0.0` constraint invariant.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(I * P) over AST items I and package part definitions P.
    /// - Space Complexity: O(I) for lowered package definitions.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` on missing part, attribute, action, requirement, or constraint.
    /// - State: Stateless in-memory test.
    /// - Action: Zero panics, unwraps, or expectations.
    /// - Resource: Bounded stack and heap fixture strings.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-LOWER-AST-TO-PACKAGE]
    #[test]
    fn test_issue_557_dart_state_methods_and_asserts_lowered_onto_owning_widget_part_def(
    ) -> Result<(), String> {
        let src = concat!(
            "class _TelemetryPanelState extends State<TelemetryPanel> {\n",
            "  int refreshHz = 60;\n",
            "\n",
            "  /// Realises: [REQ-UI-TELEM-01]\n",
            "  void updateTelemetry(double sample) {\n",
            "    assert(sample >= 0.0);\n",
            "  }\n",
            "}\n",
            "\n",
            "/// Primary telemetry panel widget.\n",
            "class TelemetryPanel extends StatefulWidget {\n",
            "  final String title;\n",
            "}\n",
        );
        let ast = parse_dart_source_ast(src, "issue_557.dart")?;
        let pkg = lower_source_ast_to_package(&ast, "UiPkg");
        let panel = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "TelemetryPanel")
            .ok_or_else(|| "Expected TelemetryPanel PartDef in pkg.part_defs".to_string())?;

        if !panel.attributes.iter().any(|a| a.name == "refreshHz") {
            return Err(format!(
                "Expected TelemetryPanel.attributes to contain 'refreshHz', got {:?}",
                panel.attributes
            ));
        }
        if !panel.actions.iter().any(|a| a.name == "updateTelemetry") {
            return Err(format!(
                "Expected TelemetryPanel.actions to contain 'updateTelemetry', got {:?}",
                panel.actions
            ));
        }
        if !panel
            .satisfied_requirements
            .iter()
            .any(|r| r == "REQ-UI-TELEM-01")
        {
            return Err(format!(
                "Expected TelemetryPanel.satisfied_requirements to contain 'REQ-UI-TELEM-01', got {:?}",
                panel.satisfied_requirements
            ));
        }
        if !panel
            .constraints
            .iter()
            .any(|c| c.expression.contains("sample >= 0.0"))
        {
            return Err(format!(
                "Expected TelemetryPanel.constraints to contain expression 'sample >= 0.0', got {:?}",
                panel.constraints
            ));
        }
        if panel.doc.as_deref() != Some("Primary telemetry panel widget.") {
            return Err(format!(
                "Expected TelemetryPanel.doc to be preserved, got {:?}",
                panel.doc
            ));
        }
        Ok(())
    }

    /// Verifies that `parse_rust_source_ast` and `parse_rust_impl_header` accurately classify methods inside multi-line `impl` blocks, `impl<F: Fn(...) -> Ret>` arrow bounds, and `where` clauses containing HRTB `for<'a>` bounds (`#558`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_558_multiline_impl_headers_and_fn_arrow_bounds`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Multi-line `impl` and `unsafe impl` headers accumulate across lines until `{` is reached before `parse_rust_impl_header` resolves `active_impl_owner`.
    /// - Return-type arrows `->` inside `Fn(...) -> ...` or `fn(...) -> ...` generic bounds do not prematurely decrement `<...>` generic depth.
    /// - Top-level `where` clauses (including `for<'a>` higher-ranked trait bounds) are stripped prior to splitting on top-level `" for "`.
    ///
    /// ## 3. Preconditions
    /// - In-memory Rust source fixture containing multi-line `impl` blocks, `Fn(u32) -> bool` bounds in `impl<...>`, and trait `impl` headers with `where for<'a>` bounds.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when all methods inside those `impl` blocks are classified as `SourceItemKind::Method` with `owner_name == Some("TelemetryProcessor")` and lowered onto `TelemetryProcessor`'s `PartDef.actions`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(L) over source lines and header length L.
    /// - Space Complexity: O(I) for extracted AST items and lowered package definitions.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` with diagnostic details on classification or owner mismatch.
    /// - State: Stateless in-memory test.
    /// - Action: Zero panics, unwraps, or expectations.
    /// - Resource: Bounded stack and heap fixture strings.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-PARSE-RUST-IMPL-HEADER]
    #[test]
    fn test_issue_558_multiline_impl_headers_and_fn_arrow_bounds() -> Result<(), String> {
        let src = concat!(
            "pub struct TelemetryProcessor<F> {\n",
            "    pub filter: F,\n",
            "}\n",
            "\n",
            "impl<F: Fn(u32) -> bool> TelemetryProcessor<F> {\n",
            "    pub fn eval_inline(&self, sample: u32) -> bool {\n",
            "        (self.filter)(sample)\n",
            "    }\n",
            "}\n",
            "\n",
            "impl<F>\n",
            "    TelemetryProcessor<F>\n",
            "where\n",
            "    F: Fn(u32) -> bool,\n",
            "    for<'a> &'a F: Send,\n",
            "{\n",
            "    pub fn eval_multiline(&self, sample: u32) -> bool {\n",
            "        (self.filter)(sample)\n",
            "    }\n",
            "}\n",
            "\n",
            "unsafe impl<F>\n",
            "    Send for TelemetryProcessor<F>\n",
            "where\n",
            "    for<'a> &'a F: Sync,\n",
            "{\n",
            "    fn verify_send(&self) {}\n",
            "}\n",
        );

        let ast = parse_rust_source_ast(src, "issue_558.rs")?;
        for method_name in ["eval_inline", "eval_multiline", "verify_send"] {
            let item = ast
                .iter()
                .find(|i| i.name == method_name)
                .ok_or_else(|| format!("Expected AST item '{method_name}' to be extracted"))?;
            if item.kind != SourceItemKind::Method {
                return Err(format!(
                    "Expected '{method_name}' to have kind Method, got {:?}",
                    item.kind
                ));
            }
            if item.owner_name.as_deref() != Some("TelemetryProcessor") {
                return Err(format!(
                    "Expected '{method_name}' to have owner_name Some(\"TelemetryProcessor\"), got {:?}",
                    item.owner_name
                ));
            }
        }

        let pkg = lower_source_ast_to_package(&ast, "TelemetryPkg");
        let proc_part = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "TelemetryProcessor")
            .ok_or_else(|| "Expected TelemetryProcessor PartDef".to_string())?;
        for method_name in ["eval_inline", "eval_multiline", "verify_send"] {
            if !proc_part.actions.iter().any(|a| a.name == method_name) {
                return Err(format!(
                    "Expected TelemetryProcessor.actions to contain '{method_name}', got {:?}",
                    proc_part.actions
                ));
            }
        }

        Ok(())
    }

    /// Verifies that `parse_dart_source_ast` and `lower_source_ast_to_package` extract all formal parameters and `assert(...)` invariants from multi-line Dart constructors, methods, `async` methods with next-line `{`, and top-level functions (`#561`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_561_multiline_dart_callable_signatures_and_invariants`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Multi-line Dart callable signatures (`const FlightControlWidget({\n...\n});`, `void engageHold(\n...\n)`, `Future<void> syncTelemetry(\n...\n) async\n{`, and `bool validateEnvelope(\n...\n)`) retain 100% of positional and named formal parameters.
    /// - Mixed positional + named parameter groups (`String callsign, { required double targetAltitudeMeters, int modeCode = 1 }`) expand inner named parameters cleanly.
    /// - `assert(...)` statements inside multi-line class methods (including next-line `{`) and top-level Dart functions are preserved in `SourceAstItem::invariants` and lowered into `ConstraintDef` nodes.
    ///
    /// ## 3. Preconditions
    /// - In-memory Dart source fixture containing multi-line class constructors, methods, `async` methods, and top-level functions with runtime `assert(...)` invariants.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when all formal parameters and assertion invariants are present in both the parsed `SourceAstItem` slice and the lowered `PackageDef`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(L) over source lines L.
    /// - Space Complexity: O(I + P) for extracted items I and parameters P.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` with descriptive mismatch diagnostics.
    /// - State: Stateless in-memory test.
    /// - Action: Zero panics, unwraps, or expectations.
    /// - Resource: Bounded stack and heap fixture strings.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
    #[test]
    fn test_issue_561_multiline_dart_callable_signatures_and_invariants() -> Result<(), String> {
        let dart_src = concat!(
            "class FlightControlWidget {\n",
            "  final String vehicleCallsign;\n",
            "  final double initialAltitudeMeters;\n",
            "\n",
            "  const FlightControlWidget({\n",
            "    super.key,\n",
            "    required this.vehicleCallsign,\n",
            "    required this.initialAltitudeMeters,\n",
            "  });\n",
            "\n",
            "  void engageHold(\n",
            "    String callsign,\n",
            "    {\n",
            "      required double targetAltitudeMeters,\n",
            "      int modeCode = 1,\n",
            "    }\n",
            "  ) {\n",
            "    assert(targetAltitudeMeters >= 0.0);\n",
            "  }\n",
            "\n",
            "  Future<void> syncTelemetry(\n",
            "    int sequenceNumber,\n",
            "  ) async\n",
            "  {\n",
            "    assert(sequenceNumber > 0);\n",
            "  }\n",
            "}\n",
            "\n",
            "bool validateEnvelope(\n",
            "  double speedMps,\n",
            "  double ceilingMeters,\n",
            ") {\n",
            "  assert(speedMps >= 0.0);\n",
            "  return speedMps <= ceilingMeters;\n",
            "}\n",
        );

        let ast = parse_dart_source_ast(dart_src, "flight_control_widget.dart")?;

        let ctor = ast
            .iter()
            .find(|i| i.kind == SourceItemKind::Constructor && i.name == "FlightControlWidget")
            .ok_or_else(|| "Expected Constructor 'FlightControlWidget'".to_string())?;
        let ctor_params: Vec<&str> = ctor.params.iter().map(|p| p.name.as_str()).collect();
        if ctor_params != vec!["key", "vehicleCallsign", "initialAltitudeMeters"] {
            return Err(format!(
                "Expected FlightControlWidget constructor params [\"key\", \"vehicleCallsign\", \"initialAltitudeMeters\"], got {:?}",
                ctor_params
            ));
        }

        let engage = ast
            .iter()
            .find(|i| i.kind == SourceItemKind::Method && i.name == "engageHold")
            .ok_or_else(|| "Expected Method 'engageHold'".to_string())?;
        let engage_params: Vec<&str> = engage.params.iter().map(|p| p.name.as_str()).collect();
        if engage_params != vec!["callsign", "targetAltitudeMeters", "modeCode"] {
            return Err(format!(
                "Expected engageHold params [\"callsign\", \"targetAltitudeMeters\", \"modeCode\"], got {:?}",
                engage_params
            ));
        }
        if !engage
            .invariants
            .iter()
            .any(|inv| inv == "targetAltitudeMeters >= 0.0")
        {
            return Err(format!(
                "Expected engageHold.invariants to contain 'targetAltitudeMeters >= 0.0', got {:?}",
                engage.invariants
            ));
        }

        let sync_fn = ast
            .iter()
            .find(|i| i.kind == SourceItemKind::Method && i.name == "syncTelemetry")
            .ok_or_else(|| "Expected Method 'syncTelemetry'".to_string())?;
        let sync_params: Vec<&str> = sync_fn.params.iter().map(|p| p.name.as_str()).collect();
        if sync_params != vec!["sequenceNumber"] {
            return Err(format!(
                "Expected syncTelemetry params [\"sequenceNumber\"], got {:?}",
                sync_params
            ));
        }
        if !sync_fn
            .invariants
            .iter()
            .any(|inv| inv == "sequenceNumber > 0")
        {
            return Err(format!(
                "Expected syncTelemetry.invariants to contain 'sequenceNumber > 0', got {:?}",
                sync_fn.invariants
            ));
        }

        let top_fn = ast
            .iter()
            .find(|i| i.kind == SourceItemKind::Function && i.name == "validateEnvelope")
            .ok_or_else(|| "Expected Function 'validateEnvelope'".to_string())?;
        let top_params: Vec<&str> = top_fn.params.iter().map(|p| p.name.as_str()).collect();
        if top_params != vec!["speedMps", "ceilingMeters"] {
            return Err(format!(
                "Expected validateEnvelope params [\"speedMps\", \"ceilingMeters\"], got {:?}",
                top_params
            ));
        }
        if !top_fn.invariants.iter().any(|inv| inv == "speedMps >= 0.0") {
            return Err(format!(
                "Expected validateEnvelope.invariants to contain 'speedMps >= 0.0', got {:?}",
                top_fn.invariants
            ));
        }

        let pkg = lower_source_ast_to_package(&ast, "FlightControlPkg");
        let part = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "FlightControlWidget")
            .ok_or_else(|| "Expected PartDef 'FlightControlWidget'".to_string())?;
        for expected_expr in ["targetAltitudeMeters >= 0.0", "sequenceNumber > 0"] {
            if !part
                .constraints
                .iter()
                .any(|c| c.expression.contains(expected_expr))
            {
                return Err(format!(
                    "Expected FlightControlWidget.constraints to contain '{expected_expr}', got {:?}",
                    part.constraints
                ));
            }
        }
        if !pkg
            .constraint_defs
            .iter()
            .any(|c| c.expression.contains("speedMps >= 0.0"))
        {
            return Err(format!(
                "Expected pkg.constraint_defs to contain 'speedMps >= 0.0', got {:?}",
                pkg.constraint_defs
            ));
        }

        Ok(())
    }

    /// Verifies generic container type unwrapping, Dart `this.`/`super.` field parameter resolution, literal field inference, and `<Pkg>Module` fallback synthesis (`#518`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_518_generic_unwrapping_dart_field_params_and_module_fallback`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `normalize_type_for_sysml` recursively unwraps single-argument and primary-payload generic containers (`Option<T>`, `Result<T, E>`, `Vec<T>`, `Future<T>`, `Stream<T>`, `ValueChanged<T>`, `Arc<Mutex<T>>`) to their inner SysML v2 type while normalizing `Future<void>` and `Result<(), E>` to `"String"`.
    /// - `parse_dart_source_ast` and `lower_source_ast_to_package` infer untyped field literal types (`bool`, `int`, `double`), resolve `this.<field>` and `super.<param>` against declared class/superclass fields without hardcoding `"Key"`, and support `<Type> this.<field>` / `<Type> super.<param>`.
    /// - `lower_source_ast_to_package` synthesizes `<Package>Module` (never `<Package>Controller`) when only top-level functions are present.
    ///
    /// ## 3. Preconditions
    /// - In-memory Rust and Dart fixtures with abstract domain-neutral identifiers.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when generic unwrapping, Dart constructor/field type resolution, and `<Package>Module` fallback all match expected SysML v2 types.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over fixture input size.
    /// - Space Complexity: O(I) for parsed AST items and lowered package definitions.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` with descriptive diagnostic messages on mismatch.
    /// - State: Stateless in-memory unit test.
    /// - Action: Zero panics, unwraps, or expectations.
    /// - Resource: Bounded stack and heap allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-NORMALIZE-TYPE-SYSML], [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG], [REQ-FEAT-42-LOWER-AST-TO-PACKAGE]
    #[test]
    fn test_issue_518_generic_unwrapping_dart_field_params_and_module_fallback(
    ) -> Result<(), String> {
        for (raw, expected) in [
            ("Option<u32>", "Integer"),
            ("Result<f64, String>", "Real"),
            ("Vec<SampleRecord>", "SampleRecord"),
            ("Future<bool>", "Boolean"),
            ("Stream<int>", "Integer"),
            ("ValueChanged<double>", "Real"),
            ("Arc<Mutex<NodeState>>", "NodeState"),
            ("Future<void>", "String"),
            ("Result<(), String>", "String"),
        ] {
            let actual = normalize_type_for_sysml(raw);
            if actual != expected {
                return Err(format!(
                    "normalize_type_for_sysml({raw:?}) = {actual:?}, expected {expected:?}"
                ));
            }
        }
        let fn_only = parse_rust_source_ast(
            "pub fn compute_metric(x: Option<i64>) -> Result<f64, String> { Ok(1.0) }\n",
            "mod.rs",
        )?;
        let fn_pkg = lower_source_ast_to_package(&fn_only, "SignalPkg");
        if fn_pkg.part_defs.first().map(|p| p.name.as_str()) != Some("SignalPkgModule") {
            return Err(format!(
                "Expected fallback PartDef 'SignalPkgModule', got {:?}",
                fn_pkg.part_defs
            ));
        }
        let dart_src = concat!(
            "class BaseNode {\n  final int nodeId;\n  const BaseNode({required this.nodeId});\n}\n",
            "class ChildNode extends BaseNode {\n  final double scaleFactor;\n  final maxRetries = 3;\n  var isReady = true;\n",
            "  const ChildNode({required super.nodeId, super.untypedToken, required this.scaleFactor, this.maxRetries = 3, this.isReady = true, int this.extraCode = 0});\n}\n",
        );
        let dart_pkg =
            lower_source_ast_to_package(&parse_dart_source_ast(dart_src, "nodes.dart")?, "NodePkg");
        let child = dart_pkg
            .part_defs
            .iter()
            .find(|p| p.name == "ChildNode")
            .ok_or("Missing ChildNode")?;
        let ctor = child
            .actions
            .iter()
            .find(|a| a.name == "ChildNode")
            .ok_or("Missing ChildNode ctor")?;
        let got: Vec<(&str, &str)> = ctor
            .in_params
            .iter()
            .map(|p| (p.name.as_str(), p.type_name.as_str()))
            .collect();
        let expected_params = vec![
            ("nodeId", "Integer"),
            ("untypedToken", "String"),
            ("scaleFactor", "Real"),
            ("maxRetries", "Integer"),
            ("isReady", "Boolean"),
            ("extraCode", "Integer"),
        ];
        if got != expected_params {
            return Err(format!(
                "Expected ChildNode ctor params {expected_params:?}, got {got:?}"
            ));
        }
        Ok(())
    }

    /// Verifies `REQ-0605` `AC-01..AC-02` unified lexical scanner types (`InterpolationFrame`, `RustDartLexState`, `ScopedItemScannerState`), `scan_line_with_unified_lex_state`, `extract_rust_impl_header_prefix`, `parse_rust_impl_header`, `strip_line_comment_preserving_strings` on nested Dart string interpolations, and `parse_dart_source_ast` multi-line annotation doc preservation (`#529`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_issue_529_part_a_req_0605_unified_lex_state_and_dart_interp`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `InterpolationFrame`, `RustDartLexState`, and `ScopedItemScannerState` are publicly exported with deterministic state transitions in `scan_line_with_unified_lex_state`.
    /// - `extract_rust_impl_header_prefix` and `parse_rust_impl_header` extract multi-line and generic Rust `impl` headers cleanly.
    /// - `strip_line_comment_preserving_strings` preserves `"http://"` inside nested `${map["http://"]}` string interpolations while stripping trailing `// comment`.
    /// - `parse_dart_source_ast` preserves `/// Realises: [REQ-0605]` doc comments across multi-line `@pragma(\n  'vm:prefer-inline',\n)` annotations and inline `/* ... */` comments.
    ///
    /// ## 3. Preconditions
    /// - Invoked under `cargo test -p deap-core --lib` with in-memory Rust and Dart lexical fixtures.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` when all unified lexical state transitions, `impl` header extractions, comment strippers, and Dart AST doc bindings match expected outputs.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over the test fixture byte length N.
    /// - Space Complexity: O(N) for parsed AST items and sanitized line buffers.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>` with descriptive diagnostic messages on any mismatch.
    /// - State: Isolated local test state.
    /// - Action: Zero panics, unwraps, expectations, or raw slice indexing.
    /// - Resource: Bounded stack and heap allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0605], [REQ-SOURCE-AST-STRIP-LINE-COMMENT], [REQ-FEAT-42-PARSE-DART-AST]
    #[test]
    fn test_issue_529_part_a_req_0605_unified_lex_state_and_dart_interp() -> Result<(), String> {
        let frame = InterpolationFrame {
            quote_byte: b'"',
            is_triple: false,
            is_raw: false,
            brace_depth: 1,
        };
        if frame.quote_byte != b'"' || frame.is_triple || frame.is_raw || frame.brace_depth != 1 {
            return Err(format!("Unexpected InterpolationFrame fields: {frame:?}"));
        }

        let scoped = ScopedItemScannerState::default();
        if scoped.mod_tests_brace_depth.is_some()
            || scoped.cfg_test_block_brace_depth.is_some()
            || scoped.cfg_test_pending
            || scoped.dart_annotation_paren_depth != 0
        {
            return Err(format!(
                "Unexpected default ScopedItemScannerState: {scoped:?}"
            ));
        }

        let mut rust_state = RustDartLexState {
            language: SourceLanguage::Rust,
            ..RustDartLexState::default()
        };
        let scanned_rust = scan_line_with_unified_lex_state(
            "pub fn check<'lifetime_a>(s: &'lifetime_a str) -> char { '/' } // comment",
            &mut rust_state,
        );
        if rust_state.block_comment_depth != 0
            || rust_state.raw_hash_count.is_some()
            || rust_state.active_string.is_some()
        {
            return Err(format!(
                "Expected clean RustDartLexState, got {rust_state:?}"
            ));
        }
        if !scanned_rust.contains("pub fn check<'lifetime_a>") || scanned_rust.contains("comment") {
            return Err(format!("Unexpected scanned_rust output: {scanned_rust:?}"));
        }

        let mut dart_state = RustDartLexState {
            language: SourceLanguage::Dart,
            ..RustDartLexState::default()
        };
        let scanned_dart = scan_line_with_unified_lex_state(
            "final s = \"url: ${map[\"http://\"]}\"; // trailing",
            &mut dart_state,
        );
        if dart_state.active_string.is_some() || !dart_state.interp_stack.is_empty() {
            return Err(format!(
                "Expected Dart interpolation stack to close cleanly, got {dart_state:?}"
            ));
        }
        if scanned_dart.contains("trailing") || !scanned_dart.contains("final s =") {
            return Err(format!("Unexpected scanned_dart output: {scanned_dart:?}"));
        }

        let impl_code =
            "impl<T: Clone> TraitBound for TargetComponent<T> where T: Send { fn run(&self) {} }";
        let prefix = extract_rust_impl_header_prefix(impl_code).ok_or_else(|| {
            "Expected Some(prefix) from extract_rust_impl_header_prefix".to_string()
        })?;
        if !prefix.starts_with("impl<T: Clone>") || !prefix.ends_with("where T: Send") {
            return Err(format!("Unexpected impl header prefix: {prefix:?}"));
        }
        let target = parse_rust_impl_header(impl_code)
            .ok_or_else(|| "Expected Some(target) from parse_rust_impl_header".to_string())?;
        if target != "TargetComponent" {
            return Err(format!(
                "Expected TargetComponent from parse_rust_impl_header, got {target:?}"
            ));
        }

        let interp_line = "final u = \"url: ${map[\"http://\"]}\"; // comment";
        let stripped = strip_line_comment_preserving_strings(interp_line);
        if stripped != "final u = \"url: ${map[\"http://\"]}\"; " {
            return Err(format!(
                "Expected strip_line_comment_preserving_strings to preserve nested interpolation quote, got {stripped:?}"
            ));
        }

        let dart_src = concat!(
            "/// Summary of annotated service.\n",
            "/// Realises: [REQ-0605]\n",
            "@pragma(\n",
            "  'vm:prefer-inline',\n",
            ")\n",
            "/* inline block comment */\n",
            "class AnnotatedService {\n",
            "  /// Summary of execute action.\n",
            "  /// Realises: [REQ-0605]\n",
            "  @pragma(\n",
            "    'vm:prefer-inline',\n",
            "  )\n",
            "  void executeAction() {\n",
            "    final msg = \"url: ${map[\"http://\"]}\"; // keep brace depth intact\n",
            "  }\n",
            "}\n",
        );
        let dart_items = parse_dart_source_ast(dart_src, "annotated_service.dart")?;
        let cls = dart_items
            .iter()
            .find(|i| i.kind == SourceItemKind::Class && i.name == "AnnotatedService")
            .ok_or_else(|| "Missing AnnotatedService class".to_string())?;
        if cls.satisfied_requirements != vec!["REQ-0605".to_string()] {
            return Err(format!(
                "Expected AnnotatedService satisfied_requirements == [\"REQ-0605\"], got {:?}",
                cls.satisfied_requirements
            ));
        }
        let method = dart_items
            .iter()
            .find(|i| i.kind == SourceItemKind::Method && i.name == "executeAction")
            .ok_or_else(|| "Missing executeAction method".to_string())?;
        if method.satisfied_requirements != vec!["REQ-0605".to_string()] {
            return Err(format!(
                "Expected executeAction satisfied_requirements == [\"REQ-0605\"], got {:?}",
                method.satisfied_requirements
            ));
        }

        Ok(())
    }

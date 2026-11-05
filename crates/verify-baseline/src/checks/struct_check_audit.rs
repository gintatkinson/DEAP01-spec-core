//! Check 34 source purity auditor (`audit_source_purity`), `mod tests` scope tracker, and production line checks.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
//! Contract: Audits Rust source strings for regex, static string-slice arrays, hollow stubs, fail-open patterns, and Rustdoc.
//! Bounds & Faults: Single-pass O(L) line analysis; fails closed with sorted, deduplicated violations.

use super::struct_check_clippy::{scan_clippy_allow_tokens, RequiredClippyForbidLint};
use super::struct_check_mask::{
    audit_tier_a_module_docblocks, contains_direct_range_index,
    mask_comments_and_literals_preserving_lines,
};
use super::struct_check_purity::{
    any_split_token, contains_bounded_split_token, enforces_strict_io_arms, has_split_token,
    BINDING_STR_SLICE_TOKENS, DIRECT_LOWER_SLICE_TOKENS, FAIL_OPEN_FILTER_MAP_TOKENS,
    FORBIDDEN_REGEX_TOKENS, LOWER_VAR_INDEX_TOKENS, UNCONDITIONAL_STR_SLICE_TOKENS,
};

/// Brace-depth state tracker for `mod tests` blocks during Check 34 line-by-line source auditing.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
#[derive(Default)]
struct ModTestsScope {
    active: bool,
    brace_depth: usize,
    entered_block: bool,
    has_test: bool,
}

impl ModTestsScope {
    /// Resets and activates `mod tests` scope tracking upon encountering a `mod tests` declaration.
    /// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
    fn enter(&mut self) {
        self.active = true;
        self.brace_depth = 0;
        self.entered_block = false;
        self.has_test = false;
    }
}

/// Outcome of inspecting one masked line inside an active `mod tests` block.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
enum ModTestsStep<'a> {
    Consumed,
    TrailingProduction(&'a str),
}

/// Audits a single Rust source string for compiler purity violations (a)-(f).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ECSS self-hosting compiler purity requirements, and DEAP Rust
/// Coding Standard v2.0 Section 4. Mechanically audits source lines for banned regex, static
/// string-slice arrays, hollow gate stubs, fail-open error swallowing, cross-string byte-slicing
/// hazards, tiered Tier A / Tier B/C Rustdoc completeness, and `C-PURITY-03` test anti-gaming invariants.
///
/// ## 2. Invariants & Mathematical Properties
/// - Banned constructs are matched via static split-token tables with zero runtime string concatenation.
/// - Non-doc comments and string/char literals are masked before token inspection.
/// - `mod tests` scope tracks brace depth `{` / `}` so closed blocks do not exempt trailing code.
/// - In audited modules, every item and structured module header must satisfy its normative Tier A
///   (7-section Rustdoc or 4-Block Contract) or Tier B/C (summary + `Realises: [...]`) contract.
///
/// ## 3. Preconditions
/// `source` must be valid UTF-8 Rust source code.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if source passes all purity checks.
/// Returns `Err(Vec<String>)` containing all detected violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is line count.
/// Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure non-panicking string analysis.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY].
pub fn audit_source_purity(
    source: &str,
    file_rel: &str,
    check_rustdoc: bool,
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    collect_forbidden_clippy_allows(source, file_rel, &mut violations);

    let masked_lines = mask_comments_and_literals_preserving_lines(source);
    if check_rustdoc {
        if let Err(doc_errs) = super::docstring_gate::audit_rust_source_docstrings(source, file_rel)
        {
            violations.extend(doc_errs);
        }
        audit_tier_a_module_docblocks(&masked_lines, file_rel, &mut violations);
    }

    let mut scope = ModTestsScope::default();
    let strict_io = enforces_strict_io_arms(file_rel);

    for (idx, masked_line) in masked_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = masked_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//!") || trimmed.starts_with("///") {
            continue;
        }

        if !scope.active && contains_bounded_split_token(trimmed, "mod", " tests") {
            scope.enter();
        }

        let code = if scope.active {
            match advance_mod_tests_line(&mut scope, trimmed, file_rel, line_no, &mut violations) {
                ModTestsStep::Consumed => continue,
                ModTestsStep::TrailingProduction(trailing) => trailing,
            }
        } else {
            trimmed
        };

        let line_ctx = LinePurityContext {
            file_rel,
            line_no,
            check_rustdoc,
            strict_io,
        };
        audit_production_code_line(code, line_ctx, &mut violations);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Appends violations for any `#[allow(clippy::...)]` suppressing a `RequiredClippyForbidLint`.
/// Realises: [REQ-CHECK-34-CLIPPY-ALLOW-TOKEN-SCAN]
fn collect_forbidden_clippy_allows(source: &str, file_rel: &str, violations: &mut Vec<String>) {
    for clippy_viol in scan_clippy_allow_tokens(source, file_rel) {
        let is_forbidden = RequiredClippyForbidLint::ALL
            .iter()
            .any(|lint| clippy_viol.attribute.contains(lint.as_str()));
        if is_forbidden {
            violations.push(format!(
                "{}:{}: forbidden clippy allow attribute detected: {}",
                clippy_viol.file, clippy_viol.line, clippy_viol.attribute
            ));
        }
    }
}

/// Audits one masked line inside `mod tests`, updating brace depth and returning any trailing production slice.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn advance_mod_tests_line<'a>(
    scope: &mut ModTestsScope,
    code: &'a str,
    file_rel: &str,
    line_no: usize,
    violations: &mut Vec<String>,
) -> ModTestsStep<'a> {
    if code.contains("#[test]") || code.contains("fn test_") {
        scope.has_test = true;
    }
    if any_split_token(code, FORBIDDEN_REGEX_TOKENS) {
        violations.push(format!("{}:{}: regex usage detected", file_rel, line_no));
    }
    if has_split_token(code, "#[ig", "nore") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 ignored test attribute detected",
            file_rel, line_no
        ));
    }
    if has_split_token(code, "assert!", "(true)") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 tautological assertion detected",
            file_rel, line_no
        ));
    }
    if code.contains(".exists()") && code.contains("return;") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 vacuous conditional early-return in test detected",
            file_rel, line_no
        ));
    }
    if has_split_token(code, "temp_dir()", ".join(\"\")") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 unisolated literal temp_dir().join(...) in test detected",
            file_rel, line_no
        ));
    }

    let mut closed_at_byte: Option<usize> = None;
    let mut byte_offset: usize = 0;
    for ch in code.chars() {
        let next_byte = byte_offset.saturating_add(ch.len_utf8());
        if ch == '{' {
            scope.brace_depth = scope.brace_depth.saturating_add(1);
            scope.entered_block = true;
        } else if ch == '}' {
            scope.brace_depth = scope.brace_depth.saturating_sub(1);
            if scope.entered_block && scope.brace_depth == 0 {
                closed_at_byte = Some(next_byte);
                break;
            }
        }
        byte_offset = next_byte;
    }

    if closed_at_byte.is_some() {
        if !scope.has_test {
            violations.push(format!(
                "{}:{}: C-PURITY-03 empty or decoy mod tests block detected",
                file_rel, line_no
            ));
        }
        scope.active = false;
        if let Some(after_idx) = closed_at_byte {
            let trailing = code.get(after_idx..).unwrap_or("").trim();
            if !trailing.is_empty() {
                return ModTestsStep::TrailingProduction(trailing);
            }
        }
    } else if !scope.entered_block && code.contains(';') {
        scope.active = false;
    }
    ModTestsStep::Consumed
}

/// Context carrying per-file Check 34 flags when auditing a single masked production line.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
#[derive(Clone, Copy)]
struct LinePurityContext<'a> {
    file_rel: &'a str,
    line_no: usize,
    check_rustdoc: bool,
    strict_io: bool,
}

/// Audits one masked production code line against Check 34 rules (a)-(e).
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn audit_production_code_line(
    code: &str,
    ctx: LinePurityContext<'_>,
    violations: &mut Vec<String>,
) {
    if any_split_token(code, FORBIDDEN_REGEX_TOKENS) {
        violations.push(format!(
            "{}:{}: regex usage detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_forbidden_static_str_slice(code) {
        violations.push(format!(
            "{}:{}: static string-slice array detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_split_token(code, "let", " _ = ") {
        violations.push(format!(
            "{}:{}: hollow gate stub detected (`let _ = `)",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_fail_open_error_swallowing(code, ctx.strict_io) {
        violations.push(format!(
            "{}:{}: fail-open error swallowing detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_cross_string_utf8_slice_hazard(code) {
        violations.push(format!(
            "{}:{}: cross-string UTF-8 byte-slice hazard detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if ctx.check_rustdoc && contains_direct_range_index(code) {
        violations.push(format!(
            "{}:{}: direct panicking range-slice indexing detected (use .get(..))",
            ctx.file_rel, ctx.line_no
        ));
    }
}

/// Detects banned static string-slice array or iterator types on one masked production line.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn has_forbidden_static_str_slice(code: &str) -> bool {
    if any_split_token(code, UNCONDITIONAL_STR_SLICE_TOKENS) {
        return true;
    }
    let is_binding_or_const =
        code.contains('=') || code.contains("const ") || code.contains("static ");
    is_binding_or_const && any_split_token(code, BINDING_STR_SLICE_TOKENS)
}

/// Detects fail-open error swallowing patterns on one masked production line.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn has_fail_open_error_swallowing(code: &str, strict_io: bool) -> bool {
    if any_split_token(code, FAIL_OPEN_FILTER_MAP_TOKENS) {
        return true;
    }
    let has_dot_ok = has_split_token(code, ".ok", "()");
    let has_read_str = has_split_token(code, "read_to", "_string");
    if has_dot_ok && (has_split_token(code, ".filter", "_map(") || has_read_str) {
        return true;
    }
    if !strict_io {
        return false;
    }
    if has_split_token(code, "Err(_)", " => continue") {
        return true;
    }
    let has_read_io = has_read_str || has_split_token(code, "read_", "dir(");
    has_split_token(code, "if let ", "Ok(") && has_read_io
}

/// Detects cross-string UTF-8 byte-slice indexing on lowercase-converted buffers.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn has_cross_string_utf8_slice_hazard(code: &str) -> bool {
    if any_split_token(code, DIRECT_LOWER_SLICE_TOKENS) {
        return true;
    }
    any_split_token(code, LOWER_VAR_INDEX_TOKENS) && has_split_token(code, ".", ".")
}

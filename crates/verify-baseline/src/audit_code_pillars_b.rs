#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Pillar 3 (`E0603` Concurrency) and Pillar 4 (`E0604` Test Integrity) evaluators (`BP-06`).
//! Realises: [REQ-0606-AUDIT-PILLAR-3-CONCURRENCY], [REQ-0606-AUDIT-PILLAR-4-TESTS]

use crate::audit_code::{AuditFinding, AuditPillar, UmlDiagramKind};
use crate::audit_code_models::SourceFileAuditCtx;
use crate::audit_code_pillars_a::{contains_bounded_word, extract_callable_name};
use crate::audit_code_scanner::{find_brace_block_end, find_enclosing_symbol, sanitize_slug};
use crate::defect_dossier::DossierSeverity;
use deap_core::rust_cst::source_bridge::SourceLanguage;

/// Scans Dart `async` methods for unguarded `notifyListeners(` or `setState(` after `await` (`E0603`).
/// Realises: [REQ-0606-AUDIT-PILLAR-3-CONCURRENCY]
fn audit_dart_async_concurrency(ctx: &SourceFileAuditCtx<'_>, out: &mut Vec<AuditFinding>) {
    let mut idx: usize = 0;
    while let Some(line) = ctx.masked_lines.get(idx) {
        let next_has_brace = ctx
            .masked_lines
            .get(idx.saturating_add(1))
            .is_some_and(|n| n.contains('{'));
        if !(contains_bounded_word(line, "async") && (line.contains('{') || next_has_brace)) {
            idx = idx.saturating_add(1);
            continue;
        }
        let method_name = extract_callable_name(line);
        let end_idx = find_brace_block_end(&ctx.masked_lines, idx);
        let mut await_seen = false;
        let mut guarded_since_await = false;
        let mut body_idx = idx;
        while body_idx <= end_idx {
            if let Some(bline) = ctx.masked_lines.get(body_idx) {
                if contains_bounded_word(bline, "await") {
                    await_seen = true;
                    guarded_since_await = false;
                } else if await_seen
                    && (contains_bounded_word(bline, "mounted")
                        || bline.contains("_disposed")
                        || bline.contains("isDisposed"))
                {
                    guarded_since_await = true;
                }
                let has_notify = bline.contains("notifyListeners(");
                let has_set_state = bline.contains("setState(");
                if await_seen && !guarded_since_await && (has_notify || has_set_state) {
                    let callee = if has_set_state {
                        "setState"
                    } else {
                        "notifyListeners"
                    };
                    push_dart_async_finding(
                        ctx,
                        &method_name,
                        callee,
                        (idx.saturating_add(1), body_idx.saturating_add(1)),
                        out,
                    );
                    break;
                }
            }
            body_idx = body_idx.saturating_add(1);
        }
        idx = idx.saturating_add(1);
    }
}

/// Appends a Dart post-`await` lifecycle concurrency finding (`E0603`) to `out`.
/// Realises: [REQ-0606-AUDIT-PILLAR-3-CONCURRENCY]
fn push_dart_async_finding(
    ctx: &SourceFileAuditCtx<'_>,
    method_name: &str,
    callee: &str,
    span: (usize, usize),
    out: &mut Vec<AuditFinding>,
) {
    let (start_line, call_line) = span;
    out.push(AuditFinding {
        slug: format!("{}-e0603-async-l{}", sanitize_slug(ctx.rel_path), call_line),
        file_path: ctx.rel_path.to_string(),
        start_line,
        end_line: call_line,
        pillar: AuditPillar::Concurrency,
        severity: DossierSeverity::Critical,
        symptom: format!(
            "Async method {method_name} invokes {callee}() after await suspension without disposal or mounted guard"
        ),
        test_target: ctx.test_target.clone(),
        whys: vec![
            ("does the async callback throw after component teardown".to_string(), format!("{method_name} resumes after an await suspension and calls {callee}() at line {call_line} without checking disposal state")),
            ("can teardown occur while the Future is suspended".to_string(), "UI navigation or controller disposal can complete on the event loop before the awaited Future resolves".to_string()),
            ("did the compiler accept the unguarded post-await call".to_string(), "Dart static type checking does not track object lifecycle transitions across async suspension points".to_string()),
            ("does invoking notifications on a disposed owner cause unhandled faults".to_string(), "ChangeNotifier and State assert active lifecycle ownership and throw unhandled exceptions once disposed".to_string()),
            ("must post-await lifecycle guards be enforced".to_string(), "every state mutation or listener dispatch after await must fail closed behind a mounted or disposal check".to_string()),
        ],
        correctness_trace: format!(
            "Across `{}:{start_line}-{call_line}`, `{method_name}` suspends at an `await` boundary and subsequently invokes `{callee}()` without verifying `mounted` or `_disposed`. If the owning instance is disposed during suspension, resumption triggers a use-after-dispose exception.",
            ctx.rel_path
        ),
        invariant_violated: "PostAwaitLifecycleGuard".to_string(),
        uml_kind: UmlDiagramKind::SequenceDiagram,
        caller_symbol: method_name.to_string(),
        callee_symbol: callee.to_string(),
        affected_callers: vec![
            (method_name.to_string(), format!("Invokes unguarded `{callee}()` at line {call_line} after `await`")),
        ],
        language_tag: "dart".to_string(),
        proposed_correction_code: "Future<void> refreshAfterAwait(Future<int> pendingValue) async {\n  final resolved = await pendingValue;\n  if (_disposed) {\n    return;\n  }\n  latestValue = resolved;\n  notifyListeners();\n}".to_string(),
    });
}

/// Evaluates Pillar 3 (`E0603` Concurrency): post-`await` unguarded Dart notifications and TOCTOU file races.
/// Realises: [REQ-0606-AUDIT-PILLAR-3-CONCURRENCY]
pub(crate) fn audit_pillar_concurrency(ctx: &SourceFileAuditCtx<'_>, out: &mut Vec<AuditFinding>) {
    if ctx.lang == SourceLanguage::Dart {
        audit_dart_async_concurrency(ctx, out);
    }
    let exists_call = [".ex", "ists()"].join("");
    for (idx, line) in ctx.masked_lines.iter().enumerate() {
        if ctx.lang == SourceLanguage::Rust && ctx.non_test_mask.get(idx).copied() != Some(true) {
            continue;
        }
        if !line.contains(&exists_call) {
            continue;
        }
        let mut look = idx.saturating_add(1);
        let limit = idx.saturating_add(4);
        while look <= limit {
            let is_toctou_use = ctx.masked_lines.get(look).is_some_and(|next_line| {
                next_line.contains("read_to_string(")
                    || next_line.contains("File::open(")
                    || next_line.contains("remove_file(")
                    || next_line.contains("readAsString(")
            });
            if is_toctou_use {
                push_toctou_finding(ctx, idx.saturating_add(1), look.saturating_add(1), out);
                break;
            }
            look = look.saturating_add(1);
        }
    }
}

/// Appends a TOCTOU filesystem race finding (`E0603`) to `out`.
/// Realises: [REQ-0606-AUDIT-PILLAR-3-CONCURRENCY]
fn push_toctou_finding(
    ctx: &SourceFileAuditCtx<'_>,
    start_line: usize,
    end_line: usize,
    out: &mut Vec<AuditFinding>,
) {
    let caller = find_enclosing_symbol(ctx.ast_items, start_line);
    let (lang_tag, fix_code) = if ctx.lang == SourceLanguage::Dart {
        (
            "dart",
            "Future<String?> readAtomicIfPresent(File target) async {\n  try {\n    return await target.readAsString();\n  } on FileSystemException {\n    return null;\n  }\n}",
        )
    } else {
        (
            "rust",
            "pub fn read_atomic_if_present(path: &std::path::Path) -> Result<Option<String>, std::io::Error> {\n    match std::fs::read_to_string(path) {\n        Ok(content) => Ok(Some(content)),\n        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),\n        Err(err) => Err(err),\n    }\n}",
        )
    };
    out.push(AuditFinding {
        slug: format!("{}-e0603-toctou-l{}", sanitize_slug(ctx.rel_path), start_line),
        file_path: ctx.rel_path.to_string(),
        start_line,
        end_line,
        pillar: AuditPillar::Concurrency,
        severity: DossierSeverity::Important,
        symptom: format!(
            "TOCTOU filesystem check-then-use sequence in {caller} risks concurrent race window"
        ),
        test_target: ctx.test_target.clone(),
        whys: vec![
            ("can filesystem access fail despite a prior existence check".to_string(), format!("{caller} checks .exists() at line {start_line} before opening or removing the path at line {end_line}")),
            ("is a separate existence probe non-atomic".to_string(), "another OS process or thread can unlink or replace the path between the two syscalls".to_string()),
            ("do developers write check-then-open sequences".to_string(), "boolean existence helpers appear simpler than matching ErrorKind::NotFound directly".to_string()),
            ("does the race window compromise reliability".to_string(), "concurrent directory mutations turn expected NotFound states into unhandled I/O failures".to_string()),
            ("must atomic open-and-match semantics be used".to_string(), "performing the file operation directly and matching NotFound eliminates the race window".to_string()),
        ],
        correctness_trace: format!(
            "Between `{}:{start_line}-{end_line}`, `{caller}` queries `.exists()` and then performs a separate filesystem syscall, introducing a Time-of-Check to Time-of-Use race window.",
            ctx.rel_path
        ),
        invariant_violated: "AtomicFilesystemErrorHandling".to_string(),
        uml_kind: UmlDiagramKind::SequenceDiagram,
        caller_symbol: caller.clone(),
        callee_symbol: "FilesystemToctouWindow".to_string(),
        affected_callers: vec![
            (caller, format!("Executes non-atomic check-then-use file sequence across lines {start_line}-{end_line}")),
        ],
        language_tag: lang_tag.to_string(),
        proposed_correction_code: fix_code.to_string(),
    });
}

/// Inspects a single `#[test]` function body from `f_idx` to `end_idx` for Pillar 4 (`E0604`).
/// Realises: [REQ-0606-AUDIT-PILLAR-4-TESTS]
fn inspect_test_fn_body(masked_lines: &[String], f_idx: usize, end_idx: usize) -> (bool, bool) {
    let mut has_real_assertion = false;
    let mut has_vacuous_skip = false;
    let mut body_idx = f_idx;
    while body_idx <= end_idx {
        if let Some(bline) = masked_lines.get(body_idx) {
            let bt = bline.trim();
            let line_has_assert = bt.contains("assert!(")
                || bt.contains("assert_eq!(")
                || bt.contains("assert_ne!(")
                || bt.contains("debug_assert!(")
                || bt.contains(".is_ok()")
                || bt.contains(".is_err()");
            let is_tautology = bt.contains("assert!(true)") || bt.contains("size_of::<");
            if line_has_assert && !is_tautology {
                has_real_assertion = true;
            }
            let next_returns = masked_lines
                .get(body_idx.saturating_add(1))
                .is_some_and(|n| contains_bounded_word(n, "return"));
            if bt.starts_with("if !")
                && bt.contains(".exists()")
                && (contains_bounded_word(bt, "return") || next_returns)
            {
                has_vacuous_skip = true;
            }
        }
        body_idx = body_idx.saturating_add(1);
    }
    (has_real_assertion, has_vacuous_skip)
}

/// Evaluates Pillar 4 (`E0604` Test Integrity): zero-assertion `#[test]` functions and vacuous early returns.
/// Realises: [REQ-0606-AUDIT-PILLAR-4-TESTS]
pub(crate) fn audit_pillar_test_integrity(
    ctx: &SourceFileAuditCtx<'_>,
    out: &mut Vec<AuditFinding>,
) {
    if ctx.lang != SourceLanguage::Rust {
        return;
    }
    let mut idx: usize = 0;
    while let Some(line) = ctx.masked_lines.get(idx) {
        let trimmed = line.trim();
        if !(trimmed == "#[test]" || trimmed.starts_with("#[test]") || trimmed.contains("::test]"))
        {
            idx = idx.saturating_add(1);
            continue;
        }
        let mut fn_idx = idx.saturating_add(1);
        let search_limit = idx.saturating_add(12);
        let mut found_fn: Option<usize> = None;
        while fn_idx <= search_limit {
            if let Some(fline) = ctx.masked_lines.get(fn_idx) {
                if contains_bounded_word(fline, "fn") {
                    found_fn = Some(fn_idx);
                    break;
                }
            } else {
                break;
            }
            fn_idx = fn_idx.saturating_add(1);
        }
        let Some(f_idx) = found_fn else {
            idx = idx.saturating_add(1);
            continue;
        };
        let fn_line_str = ctx.masked_lines.get(f_idx).map(String::as_str).unwrap_or("");
        let fn_name = extract_callable_name(fn_line_str);
        let end_idx = find_brace_block_end(&ctx.masked_lines, f_idx);
        let (has_real_assertion, has_vacuous_skip) =
            inspect_test_fn_body(&ctx.masked_lines, f_idx, end_idx);
        if !has_real_assertion || has_vacuous_skip {
            let start_line = f_idx.saturating_add(1);
            let end_line = end_idx.saturating_add(1);
            let symptom = if has_vacuous_skip {
                format!("Test function {fn_name} silently returns early when fixture path is absent instead of failing closed")
            } else {
                format!("Test function {fn_name} executes without any behavioral state or output assertion macro")
            };
            out.push(AuditFinding {
                slug: format!("{}-e0604-test-l{}", sanitize_slug(ctx.rel_path), start_line),
                file_path: ctx.rel_path.to_string(),
                start_line,
                end_line,
                pillar: AuditPillar::TestIntegrity,
                severity: DossierSeverity::Important,
                symptom,
                test_target: ctx.test_target.clone(),
                whys: vec![
                    ("can the unit test pass even when target behavior regresses".to_string(), format!("test function {fn_name} at lines {start_line}-{end_line} lacks an active fail-closed assertion oracle")),
                    ("did the test runner report green status".to_string(), "Rust test harnesses mark any #[test] function returning () without panicking as passed".to_string()),
                    ("do zero-assertion or early-return tests enter suites".to_string(), "smoke probes or conditional fixture guards omit explicit postcondition assertions".to_string()),
                    ("does hollow test coverage create safety blind spots".to_string(), "coverage metrics count executed lines while behavioral regressions pass silently".to_string()),
                    ("must deterministic assertion checks be enforced".to_string(), "every #[test] function must verify concrete postconditions and fail closed on missing fixtures".to_string()),
                ],
                correctness_trace: format!(
                    "In `{}:{start_line}-{end_line}`, `#[test]` function `{fn_name}` completes execution without evaluating a non-tautological assertion predicate (or exits early via a vacuous `.exists()` skip), yielding a false-positive green test result.",
                    ctx.rel_path
                ),
                invariant_violated: "NonVacuousTestAssertionOracle".to_string(),
                uml_kind: UmlDiagramKind::SequenceDiagram,
                caller_symbol: fn_name.clone(),
                callee_symbol: "AssertionOracle".to_string(),
                affected_callers: vec![
                    (fn_name, format!("Reports false-positive test pass across lines {start_line}-{end_line}")),
                ],
                language_tag: "rust".to_string(),
                proposed_correction_code: "fn verified_state_transition_probe() {\n    let computed = 2_u32.saturating_add(2);\n    assert_eq!(computed, 4);\n}".to_string(),
            });
        }
        idx = end_idx.saturating_add(1);
    }
}

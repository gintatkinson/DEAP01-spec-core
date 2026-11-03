#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Pillar 1 (`E0601` Memory Safety) and Pillar 2 (`E0602` Resource Lifecycle) evaluators (`BP-06`).
//! Realises: [REQ-0606-AUDIT-PILLAR-1-MEMORY], [REQ-0606-AUDIT-PILLAR-2-LIFECYCLE]

use crate::audit_code::{AuditFinding, AuditPillar, UmlDiagramKind};
use crate::audit_code_models::SourceFileAuditCtx;
use crate::audit_code_scanner::{find_brace_block_end, find_enclosing_symbol, sanitize_slug};
use crate::defect_dossier::DossierSeverity;
use deap_core::rust_cst::source_bridge::SourceLanguage;

/// Detects unchecked `expr[idx]` or `expr[a..b]` bracket indexing on a masked Rust production line.
/// Realises: [REQ-0606-DETECT-BRACKET-INDEX]
pub(crate) fn has_unchecked_bracket_index(masked_line: &str) -> bool {
    let trimmed = masked_line.trim_start();
    if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
        return false;
    }
    let bytes = masked_line.as_bytes();
    for (pos, byte) in bytes.iter().enumerate() {
        if *byte == b'[' && pos > 0 {
            if let Some(prev) = bytes.get(pos.saturating_sub(1)).copied() {
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' {
                    return true;
                }
            }
        }
    }
    false
}

/// Detects unchecked narrowing numeric `as <primitive>` casts on a masked Rust production line.
/// Realises: [REQ-0606-DETECT-NARROWING-CAST]
pub(crate) fn detect_narrowing_as_cast(masked_line: &str) -> Option<&'static str> {
    let targets = ["u8", "u16", "u32", "usize", "i8", "i16", "i32", "isize"];
    for (idx, _) in masked_line.match_indices(" as ") {
        let Some(after_as) = masked_line.get(idx.saturating_add(4)..).map(str::trim_start) else {
            continue;
        };
        for &ty in &targets {
            if let Some(rem) = after_as.strip_prefix(ty) {
                let boundary_ok = match rem.as_bytes().first().copied() {
                    Some(b) => !(b.is_ascii_alphanumeric() || b == b'_'),
                    None => true,
                };
                if boundary_ok {
                    return Some(ty);
                }
            }
        }
    }
    None
}

/// Inspects a single masked line for Pillar 1 (`E0601`) memory or arithmetic hazards.
/// Realises: [REQ-0606-AUDIT-PILLAR-1-MEMORY]
fn detect_memory_safety_violation(
    line: &str,
    lang: SourceLanguage,
    caller: &str,
) -> Option<(&'static str, String, &'static str, &'static str)> {
    if lang == SourceLanguage::Rust && has_unchecked_bracket_index(line) {
        Some((
            "idx",
            format!("Unchecked slice or bracket indexing in {caller} risks out-of-bounds runtime abort"),
            "SliceBoundsCheckedAccess",
            "SliceIndexer",
        ))
    } else if lang == SourceLanguage::Rust {
        if let Some(ty) = detect_narrowing_as_cast(line) {
            Some((
                "cast",
                format!("Unchecked numeric narrowing cast to {ty} in {caller} silently truncates upper bits"),
                "LosslessCheckedNumericConversion",
                "PrimitiveAsCast",
            ))
        } else if line.contains("*const ") || line.contains("*mut ") || line.contains("from_ptr(") {
            Some((
                "ptr",
                format!("Raw pointer dereference or foreign pointer alias in {caller} lacks lifetime bound"),
                "BoundedPointerLifetimeSafety",
                "RawPointerHandle",
            ))
        } else {
            None
        }
    } else if line.contains("Pointer<") || line.contains("fromAddress(") {
        Some((
            "ffi",
            format!("Foreign pointer handle in {caller} lacks deterministic finalizer attachment"),
            "FinalizerBackedForeignHandle",
            "ForeignPointer",
        ))
    } else {
        None
    }
}

/// Evaluates Pillar 1 (`E0601` Memory Safety): unchecked indexing, narrowing `as` casts, and raw pointers.
/// Realises: [REQ-0606-AUDIT-PILLAR-1-MEMORY]
pub(crate) fn audit_pillar_memory_safety(
    ctx: &SourceFileAuditCtx<'_>,
    out: &mut Vec<AuditFinding>,
) {
    let (lang_tag, fix_code) = if ctx.lang == SourceLanguage::Dart {
        (
            "dart",
            "int readBoundedElement(List<int> buffer, int index) {\n  if (index < 0 || index >= buffer.length) {\n    return 0;\n  }\n  return buffer.elementAt(index);\n}",
        )
    } else {
        (
            "rust",
            "pub fn checked_slice_and_narrow_sample(buf: &[u8], wide: u64) -> Option<u8> {\n    let first = buf.first().copied()?;\n    let narrow = u8::try_from(wide).ok()?;\n    Some(first.wrapping_add(narrow))\n}",
        )
    };
    for (idx, line) in ctx.masked_lines.iter().enumerate() {
        if ctx.lang == SourceLanguage::Rust && ctx.non_test_mask.get(idx).copied() != Some(true) {
            continue;
        }
        let line_no = idx.saturating_add(1);
        let caller = find_enclosing_symbol(ctx.ast_items, line_no);
        let Some((tag, symptom, inv, callee)) =
            detect_memory_safety_violation(line, ctx.lang, &caller)
        else {
            continue;
        };
        out.push(AuditFinding {
            slug: format!("{}-e0601-{}-l{}", sanitize_slug(ctx.rel_path), tag, line_no),
            file_path: ctx.rel_path.to_string(),
            start_line: line_no,
            end_line: line_no,
            pillar: AuditPillar::MemorySafety,
            severity: DossierSeverity::Critical,
            symptom,
            test_target: ctx.test_target.clone(),
            whys: vec![
                ("does the operation risk memory or arithmetic corruption".to_string(), format!("function {caller} executes an unguarded memory index, cast, or pointer operation at line {line_no}")),
                ("was the unguarded operation permitted to compile".to_string(), "primitive indexing and as-cast operators bypass fallible Result or Option return contracts".to_string()),
                ("can caller-supplied inputs trigger the boundary fault".to_string(), "upstream callers can supply empty buffers or values exceeding the narrower integer range".to_string()),
                ("was this missed prior to static AST verification".to_string(), "type-checking alone accepts unchecked indexing and lossy primitive casts without range proofs".to_string()),
                ("is deterministic static enforcement required".to_string(), "safety-critical code must use .get(..) and TryFrom conversions to fail closed without panicking".to_string()),
            ],
            correctness_trace: format!(
                "At `{}:{line_no}-{line_no}`, `{caller}` performs an unguarded memory bounds or numeric conversion step via `{callee}`. When runtime inputs exceed buffer length or target bit width, execution either aborts or propagates silently truncated state.",
                ctx.rel_path
            ),
            invariant_violated: inv.to_string(),
            uml_kind: UmlDiagramKind::SequenceDiagram,
            caller_symbol: caller.clone(),
            callee_symbol: callee.to_string(),
            affected_callers: vec![
                (caller, format!("Directly executes unguarded `{callee}` operation at line {line_no}")),
            ],
            language_tag: lang_tag.to_string(),
            proposed_correction_code: fix_code.to_string(),
        });
    }
}

/// Checks whether `line` contains `word` bounded by non-identifier ASCII characters.
/// Realises: [REQ-0606-CONTAINS-BOUNDED-WORD]
pub(crate) fn contains_bounded_word(line: &str, word: &str) -> bool {
    let bytes = line.as_bytes();
    for (idx, _) in line.match_indices(word) {
        let before_ok = idx == 0
            || match bytes.get(idx.saturating_sub(1)).copied() {
                Some(b) => !(b.is_ascii_alphanumeric() || b == b'_'),
                None => true,
            };
        let end_idx = idx.saturating_add(word.len());
        let after_ok = match bytes.get(end_idx).copied() {
            Some(b) => !(b.is_ascii_alphanumeric() || b == b'_'),
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Extracts the identifier immediately preceding `'('` on a function or method signature line.
/// Realises: [REQ-0606-EXTRACT-CALLABLE-NAME]
pub(crate) fn extract_callable_name(masked_line: &str) -> String {
    let before_paren = match masked_line.split_once('(') {
        Some((prefix, _)) => prefix.trim(),
        None => masked_line.trim(),
    };
    let candidate = before_paren
        .split_whitespace()
        .last()
        .unwrap_or("AsyncMethod");
    let cleaned: String = candidate
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if cleaned.is_empty() {
        "AsyncMethod".to_string()
    } else {
        cleaned
    }
}

/// Scans Dart `ChangeNotifier` and `State<T>` classes for undisposed managed handles (`E0602`).
/// Realises: [REQ-0606-AUDIT-PILLAR-2-LIFECYCLE]
fn audit_dart_resource_lifecycle(ctx: &SourceFileAuditCtx<'_>, out: &mut Vec<AuditFinding>) {
    let managed_types = [
        "StreamSubscription",
        "AnimationController",
        "TextEditingController",
        "FocusNode",
        "Timer",
    ];
    let mut idx: usize = 0;
    while let Some(line) = ctx.masked_lines.get(idx) {
        if !(contains_bounded_word(line, "class")
            && (line.contains("ChangeNotifier") || line.contains("State<")))
        {
            idx = idx.saturating_add(1);
            continue;
        }
        let class_name = line
            .split_once("class ")
            .and_then(|(_, rest)| rest.split_whitespace().next())
            .map(|s| {
                s.chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "StatefulComponent".to_string());
        let end_idx = find_brace_block_end(&ctx.masked_lines, idx);
        let mut found_res: Option<(&str, usize)> = None;
        let mut has_dispose_decl = false;
        let mut has_cleanup_call = false;
        let mut body_idx = idx;
        while body_idx <= end_idx {
            if let Some(bline) = ctx.masked_lines.get(body_idx) {
                if found_res.is_none() {
                    for &mty in &managed_types {
                        if contains_bounded_word(bline, mty) {
                            found_res = Some((mty, body_idx.saturating_add(1)));
                            break;
                        }
                    }
                }
                has_dispose_decl |= bline.contains("void dispose(");
                has_cleanup_call |= bline.contains(".cancel(")
                    || (bline.contains(".dispose(") && !bline.contains("super.dispose("));
            }
            body_idx = body_idx.saturating_add(1);
        }
        if let Some((res_ty, field_line)) = found_res {
            if !(has_dispose_decl && has_cleanup_call) {
                push_dart_dispose_finding(
                    ctx,
                    &class_name,
                    res_ty,
                    (field_line, end_idx.saturating_add(1)),
                    out,
                );
            }
        }
        idx = end_idx.saturating_add(1);
    }
}

/// Appends an `E0602` undisposed Dart resource finding to `out`.
/// Realises: [REQ-0606-AUDIT-PILLAR-2-LIFECYCLE]
fn push_dart_dispose_finding(
    ctx: &SourceFileAuditCtx<'_>,
    class_name: &str,
    res_ty: &str,
    span: (usize, usize),
    out: &mut Vec<AuditFinding>,
) {
    let (field_line, end_line) = span;
    out.push(AuditFinding {
        slug: format!("{}-e0602-dispose-l{}", sanitize_slug(ctx.rel_path), field_line),
        file_path: ctx.rel_path.to_string(),
        start_line: field_line,
        end_line,
        pillar: AuditPillar::ResourceLifecycle,
        severity: DossierSeverity::Critical,
        symptom: format!(
            "Class {class_name} holds {res_ty} handle without cancelling or disposing it in dispose()"
        ),
        test_target: ctx.test_target.clone(),
        whys: vec![
            ("does the component leak active subscriptions or controllers".to_string(), format!("{class_name} retains a {res_ty} field declared at line {field_line} without overriding dispose()")),
            ("does omitting dispose() cause runtime faults".to_string(), "active stream callbacks continue firing after widget or notifier teardown".to_string()),
            ("was the missing lifecycle hook undetected at compile time".to_string(), "Dart ChangeNotifier and State base classes provide a no-op base dispose() implementation".to_string()),
            ("does the leaked handle corrupt downstream application state".to_string(), "orphaned listeners retain object graphs in memory and invoke notifications on defunct owners".to_string()),
            ("must deterministic lifecycle cleanup be enforced".to_string(), "every owned subscription, timer, or controller requires deterministic cancellation inside dispose()".to_string()),
        ],
        correctness_trace: format!(
            "In `{}:{field_line}-{end_line}`, `{class_name}` allocates or stores `{res_ty}` without a matching `void dispose()` teardown routine. When `{class_name}` leaves the active lifecycle scope, the underlying handle remains registered and leaks callbacks and memory.",
            ctx.rel_path
        ),
        invariant_violated: "DeterministicResourceDisposal".to_string(),
        uml_kind: UmlDiagramKind::ClassDiagram,
        caller_symbol: class_name.to_string(),
        callee_symbol: res_ty.to_string(),
        affected_callers: vec![
            (class_name.to_string(), format!("Retains undisposed `{res_ty}` across lines {field_line}-{end_line}")),
        ],
        language_tag: "dart".to_string(),
        proposed_correction_code: "@override\nvoid dispose() {\n  _subscription?.cancel();\n  _subscription = null;\n  super.dispose();\n}".to_string(),
    });
}

/// Evaluates Pillar 2 (`E0602` Resource Lifecycle): undisposed Dart controllers/subscriptions and `/tmp/` writes.
/// Realises: [REQ-0606-AUDIT-PILLAR-2-LIFECYCLE]
pub(crate) fn audit_pillar_resource_lifecycle(
    ctx: &SourceFileAuditCtx<'_>,
    out: &mut Vec<AuditFinding>,
) {
    if ctx.lang == SourceLanguage::Dart {
        audit_dart_resource_lifecycle(ctx, out);
    }
    let shared_tmp_prefix = ["\"/", "tmp/"].join("");
    for (idx, line) in ctx.comment_stripped_lines.iter().enumerate() {
        if line.contains(&shared_tmp_prefix)
            && (line.contains("write") || line.contains("File") || line.contains("create"))
        {
            let line_no = idx.saturating_add(1);
            let caller = find_enclosing_symbol(ctx.ast_items, line_no);
            let (lang_tag, fix_code) = if ctx.lang == SourceLanguage::Dart {
                (
                    "dart",
                    "Future<Directory> createScopedScratchDir() async {\n  return Directory.systemTemp.createTemp('scoped_audit_');\n}",
                )
            } else {
                (
                    "rust",
                    "pub fn create_unique_scratch_path(label: &str) -> std::path::PathBuf {\n    std::env::temp_dir().join(format!(\"scoped_{label}_{}\", std::process::id()))\n}",
                )
            };
            out.push(AuditFinding {
                slug: format!("{}-e0602-tmp-l{}", sanitize_slug(ctx.rel_path), line_no),
                file_path: ctx.rel_path.to_string(),
                start_line: line_no,
                end_line: line_no,
                pillar: AuditPillar::ResourceLifecycle,
                severity: DossierSeverity::Important,
                symptom: format!("Hardcoded shared /tmp/ filesystem write in {caller} risks cross-process collision"),
                test_target: ctx.test_target.clone(),
                whys: vec![
                    ("can concurrent test or tool processes collide on disk".to_string(), format!("{caller} writes directly to a fixed /tmp/ path at line {line_no}")),
                    ("is a fixed /tmp/ path non-hermetic".to_string(), "shared global filenames lack process isolation and deterministic RAII teardown".to_string()),
                    ("can residual files poison subsequent runs".to_string(), "early return or failure leaves stale artifacts in the shared directory".to_string()),
                    ("does host portability suffer".to_string(), "hardcoded /tmp/ literals ignore platform-configured temporary directory roots".to_string()),
                    ("must scoped RAII directories be mandated".to_string(), "process-unique scoped directories guarantee collision-free hermetic cleanup".to_string()),
                ],
                correctness_trace: format!(
                    "At `{}:{line_no}-{line_no}`, `{caller}` targets a hardcoded `/tmp/` path instead of an isolated RAII workspace directory, risking concurrent write races and leaked temporary files.",
                    ctx.rel_path
                ),
                invariant_violated: "HermeticRaiiTemporaryIsolation".to_string(),
                uml_kind: UmlDiagramKind::SequenceDiagram,
                caller_symbol: caller.clone(),
                callee_symbol: "SharedTmpFilesystem".to_string(),
                affected_callers: vec![
                    (caller, format!("Writes to un-isolated shared temporary path at line {line_no}")),
                ],
                language_tag: lang_tag.to_string(),
                proposed_correction_code: fix_code.to_string(),
            });
        }
    }
}

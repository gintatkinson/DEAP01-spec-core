#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Track-1 Lossless Rust CST Codemod & NDJSON CLI Mode Handlers (`TD-11`, `TD-15`, `TD-18`, `TD-19`).
//! Safety & Traceability: Realises: [REQ-0608], [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]
//! Contract: Executes `--list-rust-items`, `--extract-rust-module`, `--compact-tier-bc`, `--apply-machine-fixes`, and `--message-format json` via `deap_core::rust_cst`.
//! Bounds & Faults: Operates in O(N) time and O(N) space per source file; fails closed via typed `DriverError` on missing files or invalid item lists.

use crate::cli::MessageFormat;
use crate::docstring_cst::compact_tier_bc_tree;
use crate::driver::DriverError;
use crate::driver_mode_handlers::handle_inject_docstrings;
use crate::salsa_db::write_atomic;
use deap_core::diagnostics::{DiagnosticEmitError, StructuredDiagnostic};
use deap_core::rust_cst::codemod::{
    list_rust_cst_items, CodemodError, RustCstInventoryError,
};
use deap_core::rust_cst::{
    apply_machine_applicable_fixes, compact_tier_bc_docstrings, emit_ndjson_line,
    extract_rust_module, list_rust_items, normalize_lf, rewrite_blueprint_line_refs,
    NdjsonDiagnostic, NdjsonSpan, ParsedRustFile,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Formats a structured `NdjsonDiagnostic` line using `deap_core::rust_cst::emit_ndjson_line`.
/// Safety & Traceability: Realises: [REQ-0608], [DO-330-TQL-4-CLI]
/// Contract: Serializes a diagnostic code, level, message, and target file span into a single NDJSON line.
/// Bounds & Faults: O(M) time and space in message length M; never panics.
pub fn format_driver_ndjson_line(
    code: &str,
    level: &str,
    message: &str,
    file_path: &str,
    fix_applicable: bool,
) -> String {
    let applicability = fix_applicable.then(|| "MachineApplicable".to_string());
    let span = NdjsonSpan {
        file_name: file_path.to_string(),
        byte_start: 0,
        byte_end: 0,
        line_start: 1,
        line_end: 1,
        column_start: 1,
        column_end: 1,
        suggested_replacement: None,
        suggestion_applicability: applicability,
    };
    let diag = NdjsonDiagnostic {
        code: code.to_string(),
        level: level.to_string(),
        message: message.to_string(),
        spans: vec![span],
        rendered: None,
    };
    emit_ndjson_line(&diag).unwrap_or_else(|_| format!("{{\"code\":\"{code}\",\"level\":\"{level}\"}}"))
}

/// Counts 1-indexed line number at `byte_offset` in `src`.
/// Realises: [REQ-0608]
fn line_at_offset(src: &str, byte_offset: usize) -> usize {
    1 + src.get(..byte_offset).unwrap_or("").bytes().filter(|&b| b == b'\n').count()
}

/// Emits a deterministic top-level Rust CST item inventory to `out` (`TD-15`).
/// Safety & Traceability: Realises: [REQ-0608], [DO-330-TQL-4-CLI]
/// Contract: Parses `target_path` via `list_rust_cst_items` and writes items ordered by `(byte_start, byte_end, name)` as NDJSON or human lines.
/// Bounds & Faults: O(N log K) time and O(N) space; fails closed via `RustCstInventoryError` on missing files, syntax errors, or stream I/O errors.
pub fn emit_rust_item_inventory(
    target_path: &Path,
    message_format_json: bool,
    out: &mut dyn Write,
) -> Result<(), RustCstInventoryError> {
    if target_path.as_os_str().is_empty() {
        return Err(RustCstInventoryError::EmptySourceOrPath);
    }
    let path_str = target_path.display().to_string();
    let src = fs::read_to_string(target_path)
        .map_err(|e| RustCstInventoryError::IoError(format!("{path_str}: {e}")))?;
    let inv = list_rust_cst_items(&path_str, &src)?;
    let mut ordered: Vec<_> = inv.items.into_values().collect();
    ordered.sort_by(|a, b| {
        (a.span.byte_start, a.span.byte_end, &a.name)
            .cmp(&(b.span.byte_start, b.span.byte_end, &b.name))
    });
    for item in ordered {
        let start_line = line_at_offset(&src, item.span.byte_start);
        let end_line = line_at_offset(&src, item.span.byte_end);
        let res = if message_format_json {
            let payload = serde_json::json!({
                "name": item.name, "kind": item.kind, "is_pub": item.is_pub,
                "doc_tier": item.doc_tier, "span": item.span,
                "byte_start": item.span.byte_start, "byte_end": item.span.byte_end,
                "start_line": start_line, "end_line": end_line,
            });
            writeln!(out, "{payload}")
        } else {
            writeln!(out, "{:?} {} {}..={}", item.kind, item.name, start_line, end_line)
        };
        res.map_err(|e| RustCstInventoryError::IoError(format!("{path_str}: {e}")))?;
    }
    Ok(())
}

/// Emits a `StructuredDiagnostic` as a single-line NDJSON record with zero ANSI escapes (`TD-18`).
/// Safety & Traceability: Realises: [REQ-0608], [DO-330-TQL-4-CLI]
/// Contract: Serializes `diag` to single-line JSON stripped of `\r`, `\n`, and `\x1b` control sequences and writes a `\n`-terminated line to `out`.
/// Bounds & Faults: O(M) time and space in serialized byte length `M`; maps serialization and stream errors to `DiagnosticEmitError`.
pub fn emit_codemod_ndjson_diagnostic(
    diag: &StructuredDiagnostic,
    out: &mut dyn Write,
) -> Result<(), DiagnosticEmitError> {
    let raw = serde_json::to_string(diag)
        .map_err(|e| DiagnosticEmitError::SerializationError(e.to_string()))?;
    let clean = raw.replace(['\r', '\n', '\x1b'], "").replace("\\u001b", "");
    writeln!(out, "{clean}").map_err(|e| DiagnosticEmitError::IoError(e.to_string()))
}

/// Rewrites markdown blueprint references in `designs_dir` from `old_rel` to `new_rel` with LF line endings.
/// Realises: [REQ-0608]
fn rewrite_designs_dir_refs(
    designs_dir: &Path,
    old_rel: &str,
    new_rel: &str,
) -> Result<Vec<String>, String> {
    if !designs_dir.is_dir() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(designs_dir)
        .map_err(|e| format!("Error reading designs directory '{}': {e}", designs_dir.display()))?;
    let mut updated = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let Ok(md) = fs::read_to_string(&path) else { continue };
        let rewritten = normalize_lf(&rewrite_blueprint_line_refs(&md, old_rel, new_rel));
        if rewritten != md {
            write_atomic(&path, &rewritten)
                .map_err(|e| format!("Error updating blueprint file '{}': {e}", path.display()))?;
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                updated.push(format!("docs/designs/{fname}"));
            }
        }
    }
    updated.sort();
    Ok(updated)
}

/// Atomically extracts named top-level Rust CST items from `from_rel` into `to_rel` and updates blueprint references (`TD-18`).
/// Safety & Traceability: Realises: [REQ-0608], [DO-330-TQL-4-CLI]
/// Contract: Validates all requested `item_names` before mutation, writes LF-normalized submodule and parent files atomically, and updates `docs/designs/*.md`.
/// Bounds & Faults: O(N + B) time across source size `N` and blueprint bytes `B`; fails closed via `CodemodError` with zero partial writes on unknown items.
pub fn extract_rust_module_atomic(
    workspace_root: &Path,
    from_rel: &str,
    to_rel: &str,
    item_names: &BTreeSet<String>,
) -> Result<Vec<String>, CodemodError> {
    if workspace_root.as_os_str().is_empty()
        || from_rel.trim().is_empty()
        || to_rel.trim().is_empty()
        || item_names.is_empty()
        || from_rel.contains("..")
        || to_rel.contains("..")
    {
        return Err(CodemodError::EmptyInput);
    }
    let from_path = workspace_root.join(from_rel);
    let src = fs::read_to_string(&from_path).map_err(|e| CodemodError::IoError(e.to_string()))?;
    let parsed = ParsedRustFile::parse_rust_source(&src);
    let existing: BTreeSet<String> = list_rust_items(&parsed).into_iter().map(|i| i.name).collect();
    for name in item_names {
        if !existing.contains(name) {
            return Err(CodemodError::ItemNotFound(name.clone()));
        }
    }
    let to_path = workspace_root.join(to_rel);
    let submod_name = to_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("extracted_submodule");
    let item_refs: Vec<&str> = item_names.iter().map(String::as_str).collect();
    let plan = extract_rust_module(&src, submod_name, &item_refs)
        .map_err(CodemodError::PostSpliceSyntaxError)?;
    write_atomic(&to_path, &normalize_lf(&plan.new_submodule_source))
        .map_err(|e| CodemodError::IoError(e.to_string()))?;
    write_atomic(&from_path, &normalize_lf(&plan.updated_parent_source))
        .map_err(|e| CodemodError::IoError(e.to_string()))?;
    let (old_norm, new_norm) = (from_rel.replace('\\', "/"), to_rel.replace('\\', "/"));
    let mut modified = BTreeSet::from([old_norm.clone(), new_norm.clone()]);
    let updated_bps = rewrite_designs_dir_refs(&workspace_root.join("docs/designs"), &old_norm, &new_norm)
        .map_err(CodemodError::IoError)?;
    modified.extend(updated_bps);
    Ok(modified.into_iter().collect())
}

/// Executes `--list-rust-items <FILE>` (`CommandMode::ListRustItems`, `TD-11`).
/// Realises: [REQ-0608], [DO-330-TQL-4-CLI]
pub(crate) fn handle_list_rust_items(
    target_file: &Path,
    message_format: MessageFormat,
) -> Result<i32, DriverError> {
    if !target_file.is_file() {
        return Err(DriverError::Resolution(format!(
            "Error: Rust source file does not exist: {}",
            target_file.display()
        )));
    }
    let src = fs::read_to_string(target_file).map_err(|e| {
        DriverError::Io(format!("Error reading Rust source file '{}': {e}", target_file.display()))
    })?;
    let parsed = ParsedRustFile::parse_rust_source(&src);
    let items = list_rust_items(&parsed);
    for item in &items {
        let line = serde_json::json!({
            "name": item.name, "kind": format!("{:?}", item.kind), "is_pub": item.is_pub,
            "start_line": item.start_line, "end_line": item.end_line,
            "fn_exec_lines": item.fn_exec_lines, "fn_param_count": item.fn_param_count,
        });
        println!("{line}");
    }
    if message_format == MessageFormat::Json {
        let summary = format_driver_ndjson_line(
            "CST-LIST-ITEMS",
            "info",
            &format!("Listed {} Rust CST item(s)", items.len()),
            &target_file.to_string_lossy(),
            false,
        );
        println!("{summary}");
    }
    Ok(0)
}

/// Executes `--extract-rust-module <SRC> --items <CSV> --dest <DEST>` (`CommandMode::ExtractRustModule`, `TD-15`).
/// Realises: [REQ-0608], [DO-330-TQL-4-CLI]
pub(crate) fn handle_extract_rust_module(
    source_file: &Path,
    dest_file: &Path,
    items_csv: &str,
    dry_run: bool,
) -> Result<i32, DriverError> {
    if !source_file.is_file() {
        return Err(DriverError::Resolution(format!(
            "Error: Source Rust file does not exist: {}",
            source_file.display()
        )));
    }
    let src = fs::read_to_string(source_file).map_err(|e| {
        DriverError::Io(format!("Error reading source Rust file '{}': {e}", source_file.display()))
    })?;
    let submod_name = dest_file
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("extracted_submodule");
    let item_names: Vec<&str> = items_csv.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if item_names.is_empty() {
        return Err(DriverError::Cli(
            "Error: --extract-rust-module requires at least one item in --items <CSV>".to_string(),
        ));
    }
    let plan = extract_rust_module(&src, submod_name, &item_names)
        .map_err(|e| DriverError::Codegen(1, format!("Error extracting Rust module: {e}")))?;
    if !dry_run {
        write_atomic(dest_file, &normalize_lf(&plan.new_submodule_source)).map_err(|e| {
            DriverError::Io(format!("Error writing extracted submodule '{}': {e}", dest_file.display()))
        })?;
        write_atomic(source_file, &normalize_lf(&plan.updated_parent_source)).map_err(|e| {
            DriverError::Io(format!("Error updating parent Rust module '{}': {e}", source_file.display()))
        })?;
        let (old_rel, new_rel) = (
            source_file.to_string_lossy().replace('\\', "/"),
            dest_file.to_string_lossy().replace('\\', "/"),
        );
        let _ = rewrite_designs_dir_refs(Path::new("docs/designs"), &old_rel, &new_rel)
            .map_err(DriverError::Io)?;
    }
    let prefix = if dry_run { "[Extract Rust Module] [DRY RUN] Would extract" } else { "[Extract Rust Module] Extracted" };
    println!("{prefix} {} item(s) from '{}' to '{}'", item_names.len(), source_file.display(), dest_file.display());
    Ok(0)
}

/// Executes `--compact-tier-bc <PATH>` (`CommandMode::CompactTierBc`, `TD-18`).
/// Realises: [REQ-0608], [DO-330-TQL-4-CLI]
pub(crate) fn handle_compact_tier_bc(
    target_path: &Path,
    dry_run: bool,
) -> Result<i32, DriverError> {
    if !target_path.exists() {
        return Err(DriverError::Resolution(format!(
            "Error: Target path does not exist: {}",
            target_path.display()
        )));
    }
    if dry_run {
        if target_path.is_file() {
            let src = fs::read_to_string(target_path).map_err(|e| {
                DriverError::Io(format!("Error reading '{}': {e}", target_path.display()))
            })?;
            let _compacted = compact_tier_bc_docstrings(&src);
        }
        println!("[Compact Tier B/C] [DRY RUN] Would compact Tier B/C docstrings in '{}'", target_path.display());
        return Ok(0);
    }
    let modified = compact_tier_bc_tree(target_path).map_err(DriverError::Io)?;
    println!("[Compact Tier B/C] Compacted Tier B/C docstrings in {modified} file(s) under '{}'", target_path.display());
    Ok(0)
}

/// Executes `--apply-machine-fixes` (`CommandMode::ApplyMachineFixes`, `TD-19`).
/// Realises: [REQ-0608], [DO-330-TQL-4-CLI]
pub(crate) fn handle_apply_machine_fixes(
    target_path: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<i32, DriverError> {
    if !target_path.exists() {
        return Err(DriverError::Resolution(format!(
            "Error: Target path does not exist: {}",
            target_path.display()
        )));
    }
    if target_path.is_file() && target_path.extension().and_then(|s| s.to_str()) == Some("rs") {
        let original = fs::read_to_string(target_path)
            .map_err(|e| DriverError::Io(format!("Error reading '{}': {e}", target_path.display())))?;
        let code = handle_inject_docstrings(target_path, schema, dry_run, force)?;
        if !dry_run {
            let updated = fs::read_to_string(target_path)
                .map_err(|e| DriverError::Io(format!("Error reading '{}': {e}", target_path.display())))?;
            if updated != original {
                let span = NdjsonSpan {
                    file_name: target_path.to_string_lossy().to_string(),
                    byte_start: 0,
                    byte_end: original.len(),
                    line_start: 1,
                    line_end: original.lines().count().max(1),
                    column_start: 1,
                    column_end: 1,
                    suggested_replacement: Some(updated),
                    suggestion_applicability: Some("MachineApplicable".to_string()),
                };
                let diag = NdjsonDiagnostic {
                    code: "CST-DOC-FIX".to_string(),
                    level: "warning".to_string(),
                    message: "Apply machine-applicable docstring synthesis".to_string(),
                    spans: vec![span],
                    rendered: None,
                };
                let fixed = apply_machine_applicable_fixes(&original, &[diag]).map_err(DriverError::Io)?;
                write_atomic(target_path, &normalize_lf(&fixed))
                    .map_err(|e| DriverError::Io(format!("Error writing '{}': {e}", target_path.display())))?;
            }
        }
        return Ok(code);
    }
    handle_inject_docstrings(target_path, schema, dry_run, force)
}

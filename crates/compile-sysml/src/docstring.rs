#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Tiered v2.0 DO-178C safety docstring synthesizer, Tier B/C compactor, and atomic source injector (`FEAT-42` / `BP-05`).
//! Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-SYNTHESIZER], [DO-330-TQL-4-DOCSTRING-SYNTHESIZER], [REQ-0501], [REQ-0605]
//! Contract: Synthesizes per-item unique 7-section safety docstrings on Rust Tier A and Dart declarations, compacts Tier B/C items, and writes updates atomically.
//! Bounds & Faults: O(N) time and space in total source lines `N`; fails closed on syntax, I/O, or idempotency violations.

pub use crate::docstring_compliance_check::is_compliant_docblock;
pub use crate::docstring_cst;
pub use crate::docstring_dart_codemod::inject_dart_file_docstrings;
pub use crate::docstring_rust_codemod::inject_rust_file_docstrings;
pub use crate::docstring_schema_bind::{
    collect_all_parts, resolve_schema_context_strict, MAX_AST_DEPTH,
};

use crate::docstring_cst::{
    collect_injectable_source_files, ingest_source_directory, resolve_source_package_name,
    sanitize_sysml_identifier, single_char_literal_byte_len, SourceLanguage,
};
use crate::semantic::digest::{generate_digest, write_atomic, write_digest_atomic};
use crate::semantic::serializer::to_sysml;
use deap_core::sysml_ast::PackageDef;
use std::fs;
use std::path::{Path, PathBuf};

/// Process exit code contract for `compile-sysml --inject-safety-docstrings` (`FEAT-42` Section 2.2).
/// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-EXIT-CODE], [REQ-0605]
/// Contract: Maps `Clean -> 0`, `Error -> 1`, and `IdempotencyOrConflict -> 2`.
/// Bounds & Faults: O(1) time and space; total `code(self) -> i32` projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocstringExitCode {
    Clean,
    Error,
    IdempotencyOrConflict,
}

impl DocstringExitCode {
    /// Converts `self` to its integer process exit code (`0`, `1`, or `2`).
    /// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-EXIT-CODE-INT]
    /// Contract: Bijective mapping `{Clean -> 0, Error -> 1, IdempotencyOrConflict -> 2}`.
    /// Bounds & Faults: O(1) time and space; infallible.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Clean => 0,
            Self::Error => 1,
            Self::IdempotencyOrConflict => 2,
        }
    }
}

/// Typed algebraic error classification for safety docstring synthesis and injection (`FEAT-42` Section 2.2, refs `#512`).
/// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-ERROR-ENUM], [REQ-0605]
/// Contract: Partitions failures into `Io` (`1`), `Parse` (`1`), and `UnforcedConflict` (`2`).
/// Bounds & Faults: O(M) space in message byte length `M`; zero string-prefix heuristics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocstringError {
    Io(String),
    Parse(String),
    UnforcedConflict(String),
}

impl DocstringError {
    /// Maps this `DocstringError` variant deterministically to its normative `DocstringExitCode`.
    /// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-ERROR-EXIT-CODE]
    /// Contract: Maps `Io(_) | Parse(_) -> Error` (`1`) and `UnforcedConflict(_) -> IdempotencyOrConflict` (`2`).
    /// Bounds & Faults: O(1) time and space; infallible.
    #[must_use]
    pub const fn exit_code(&self) -> DocstringExitCode {
        match self {
            Self::Io(_) | Self::Parse(_) => DocstringExitCode::Error,
            Self::UnforcedConflict(_) => DocstringExitCode::IdempotencyOrConflict,
        }
    }

    /// Returns the human-readable diagnostic message carried by this `DocstringError`.
    /// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-ERROR-AS-STR]
    /// Contract: Borrows the inner `String` slice without allocation.
    /// Bounds & Faults: O(1) time and space; infallible.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Io(msg) | Self::Parse(msg) | Self::UnforcedConflict(msg) => msg.as_str(),
        }
    }

    /// Consumes this `DocstringError` and returns its owned diagnostic message `String`.
    /// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-ERROR-INTO-MESSAGE]
    /// Contract: Moves the inner `String` out of `self` with zero cloning.
    /// Bounds & Faults: O(1) time and space; infallible.
    #[must_use]
    pub fn into_message(self) -> String {
        match self {
            Self::Io(msg) | Self::Parse(msg) | Self::UnforcedConflict(msg) => msg,
        }
    }
}

impl std::ops::Deref for DocstringError {
    type Target = str;

    /// Dereferences `DocstringError` to its underlying `&str` diagnostic message slice.
    /// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-DEREF]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl std::fmt::Display for DocstringError {
    /// Formats the diagnostic message of `DocstringError` into `f`.
    /// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-DISPLAY]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::error::Error for DocstringError {}

impl From<DocstringError> for String {
    /// Converts a `DocstringError` into its owned `String` diagnostic message.
    /// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-INTO-STRING]
    fn from(err: DocstringError) -> Self {
        err.into_message()
    }
}

/// Configuration options for `run_docstring_injection`.
/// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTION-OPTIONS]
/// Contract: Bundles `target_path`, `dry_run`, `force`, and optional `schema_model`.
/// Bounds & Faults: O(1) field access; bounded by optional `PackageDef` AST size.
#[derive(Debug, Clone)]
pub struct DocstringInjectionOptions {
    pub target_path: PathBuf,
    pub dry_run: bool,
    pub force: bool,
    pub schema_model: Option<PackageDef>,
}

/// Summary report returned after running safety docstring synthesis across a target path.
/// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTION-REPORT]
/// Contract: Records `files_scanned`, `files_modified`, `items_enriched`, and `modified_files`.
/// Bounds & Faults: O(F) space in modified file count `F`; saturating counter arithmetic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocstringInjectionReport {
    pub files_scanned: usize,
    pub files_modified: usize,
    pub items_enriched: usize,
    pub modified_files: Vec<PathBuf>,
}

/// Executes `--extract-source <DIR>` reverse synchronization probe into canonical SysML v2 and digest.
/// Safety & Traceability: Realises: [REQ-FEAT-42-RUN-EXTRACT-SOURCE]
/// Contract: Extracts canonical `PackageDef` from `source_dir`, serializes SysML v2 text, and writes model + digest atomically when `!dry_run`.
/// Bounds & Faults: O(N log N) time in source byte size `N`; returns `Err((DocstringExitCode::Error, String))` on failure.
pub fn run_extract_source(
    source_dir: &Path,
    out_path: &Path,
    digest_path: &Path,
    dry_run: bool,
) -> Result<PackageDef, (DocstringExitCode, String)> {
    let pkg_name = resolve_source_package_name(source_dir, None);
    let pkg = ingest_source_directory(source_dir, SourceLanguage::Auto, &pkg_name)
        .map_err(|e| (DocstringExitCode::Error, e.into_string()))?;
    let sysml_text = to_sysml(&pkg);
    if !dry_run {
        write_atomic(out_path, &sysml_text).map_err(|e| {
            (
                DocstringExitCode::Error,
                format!("Failed to write extracted SysML to '{}': {e}", out_path.display()),
            )
        })?;
        let digest = generate_digest(&pkg, &sysml_text);
        write_digest_atomic(digest_path, &digest).map_err(|e| {
            (
                DocstringExitCode::Error,
                format!("Failed to write schema digest to '{}': {e}", digest_path.display()),
            )
        })?;
    }
    Ok(pkg)
}

/// Executes `--inject-safety-docstrings <DIR>` across all eligible `.rs` and `.dart` files under `opts.target_path`.
/// Safety & Traceability: Realises: [REQ-FEAT-42-RUN-DOCSTRING-INJECTION]
/// Contract: Walks `opts.target_path`, synthesizes tiered safety docstrings, verifies second-pass idempotency in memory, and writes atomically.
/// Bounds & Faults: O(N) time and O(L + F) space; returns `Err((DocstringExitCode, String))` on syntax, I/O, or idempotency failure.
pub fn run_docstring_injection(
    opts: &DocstringInjectionOptions,
) -> Result<DocstringInjectionReport, (DocstringExitCode, String)> {
    let map_inject_err =
        |e: DocstringError| -> (DocstringExitCode, String) { (e.exit_code(), e.into_message()) };
    if !opts.target_path.exists() {
        return Err(map_inject_err(DocstringError::Io(format!(
            "Target path '{}' does not exist",
            opts.target_path.display()
        ))));
    }
    let files = collect_injectable_source_files(&opts.target_path)
        .map_err(DocstringError::Io)
        .map_err(map_inject_err)?;
    let mut report = DocstringInjectionReport {
        files_scanned: files.len(),
        ..Default::default()
    };

    for file_path in files {
        let original = fs::read_to_string(&file_path)
            .map_err(|e| DocstringError::Io(format!("Failed to read '{}': {e}", file_path.display())))
            .map_err(map_inject_err)?;
        let file_rel = file_path.display().to_string();
        let lang = SourceLanguage::detect_from_extension(&file_path).unwrap_or(SourceLanguage::Rust);
        let schema_ref = opts.schema_model.as_ref();
        let (enriched, count) = match lang {
            SourceLanguage::Rust | SourceLanguage::Auto => {
                inject_rust_file_docstrings(&original, &file_rel, schema_ref, opts.force)
                    .map_err(map_inject_err)?
            }
            SourceLanguage::Dart => {
                inject_dart_file_docstrings(&original, &file_rel, schema_ref, opts.force)
                    .map_err(map_inject_err)?
            }
        };
        let (second_pass, second_count) = match lang {
            SourceLanguage::Rust | SourceLanguage::Auto => {
                inject_rust_file_docstrings(&enriched, &file_rel, schema_ref, false)
                    .map_err(map_inject_err)?
            }
            SourceLanguage::Dart => {
                inject_dart_file_docstrings(&enriched, &file_rel, schema_ref, false)
                    .map_err(map_inject_err)?
            }
        };
        if second_pass != enriched || second_count != 0 {
            return Err((
                DocstringExitCode::IdempotencyOrConflict,
                format!(
                    "Idempotency invariant violation in '{file_rel}': second pass modified {second_count} item(s)"
                ),
            ));
        }
        if enriched != original {
            if !opts.dry_run {
                write_atomic(&file_path, &enriched)
                    .map_err(|e| DocstringError::Io(format!("Failed to write '{}': {e}", file_path.display())))
                    .map_err(map_inject_err)?;
            }
            report.files_modified = report.files_modified.saturating_add(1);
            report.items_enriched = report.items_enriched.saturating_add(count);
            report.modified_files.push(file_path);
        }
    }
    Ok(report)
}

/// Probes whether `slice` starts with a single-character literal via `single_char_literal_byte_len`.
/// Safety & Traceability: Realises: [REQ-FEAT-42-SINGLE-CHAR-PROBE]
/// Contract: Delegates to `single_char_literal_byte_len(slice)`.
/// Bounds & Faults: O(1) time and space; returns `None` when `slice` is not a char literal.
#[must_use]
pub fn probe_single_char_literal(slice: &str) -> Option<usize> {
    single_char_literal_byte_len(slice)
}

/// Compacts Tier-B/C docblocks under `root_dir` while skipping `skip_generated_rel` (`TD-19`).
/// Safety & Traceability: Realises: [REQ-0501], [REQ-0608], [SC-01]
/// Contract: Walks `.rs` files under `root_dir` (excluding `skip_generated_rel`), compacts Tier-B/C docblocks to 2-line form while preserving Tier-A 7-section docblocks, and writes LF-normalized files.
/// Bounds & Faults: O(F * N) time across `F` files; returns `CodemodError::EmptyInput` on empty inputs or `CodemodError::IoError` on missing/unreadable paths.
pub fn compact_tier_bc_tree(
    root_dir: &Path,
    skip_generated_rel: &str,
) -> Result<usize, deap_core::rust_cst::codemod::CodemodError> {
    use deap_core::rust_cst::codemod::CodemodError;
    let skip_clean = skip_generated_rel.trim().replace('\\', "/");
    if root_dir.as_os_str().is_empty() || skip_clean.is_empty() || skip_clean.contains("..") {
        return Err(CodemodError::EmptyInput);
    }
    docstring_cst::compact_tier_bc_with_skip(root_dir, Some(&skip_clean))
        .map_err(CodemodError::IoError)
}

/// Derives an uppercase requirement tag prefix from a file path (`file_rel`).
/// Realises: [REQ-FEAT-42-DERIVE-MODULE-REQ-TAG]
pub(crate) fn derive_module_req_tag(file_rel: &str) -> String {
    let path = Path::new(file_rel);
    let raw_components: Vec<String> = path
        .components()
        .filter_map(|comp| {
            let trimmed = comp.as_os_str().to_string_lossy().trim().to_string();
            (!matches!(trimmed.as_str(), "" | "." | ".." | "/" | "\\")).then_some(trimmed)
        })
        .collect();

    let anchor_start = raw_components
        .iter()
        .rposition(|c| matches!(c.as_str(), "crates" | "app_flutter"))
        .or_else(|| raw_components.iter().rposition(|c| matches!(c.as_str(), "src" | "lib" | "tests" | "bin")))
        .unwrap_or(0);

    let scoped_slice = raw_components.get(anchor_start..).unwrap_or(&raw_components);
    let mut parts = Vec::new();
    for comp in scoped_slice {
        let stem = comp
            .strip_suffix(".rs")
            .or_else(|| comp.strip_suffix(".dart"))
            .unwrap_or(comp.as_str());
        if matches!(stem, "" | "." | ".." | "crates" | "src" | "app_flutter" | "lib" | "tests" | "bin") {
            continue;
        }
        let clean = sanitize_req_segment(stem);
        if !clean.is_empty() {
            parts.push(clean);
        }
    }
    if parts.len() > 2 {
        if let Some(tail) = parts.get(parts.len().saturating_sub(2)..) {
            return tail.join("-");
        }
    }
    if parts.is_empty() { "MODULE".to_string() } else { parts.join("-") }
}

/// Sanitizes an identifier into an uppercase hyphenated requirement ID segment.
/// Realises: [REQ-FEAT-42-SANITIZE-REQ-SEGMENT]
pub(crate) fn sanitize_req_segment(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_hyphen = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_hyphen = false;
        } else if !prev_hyphen && !out.is_empty() {
            out.push('-');
            prev_hyphen = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        sanitize_sysml_identifier(raw, "ITEM").to_ascii_uppercase()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
#[path = "docstring_unit_tests.rs"]
mod tests;

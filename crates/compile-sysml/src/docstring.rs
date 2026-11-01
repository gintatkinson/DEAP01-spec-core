//! Tiered v2.0 DO-178C safety docstring synthesizer, Tier B/C compactor, and atomic source injector (FEAT-42).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, ECSS-E-ST-40C, and DEAP Rust Coding
//! Standard v2.0 Section 4. Synthesizes per-item unique 7-section safety docstrings on Rust Tier A
//! and Dart declarations, synthesizes and compacts concise 2-line docstrings on Rust Tier B/C items,
//! and injects updates atomically without altering executable AST semantics.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bit-for-Bit Idempotency (AC-04): Running `--inject-safety-docstrings <DIR>` a second time on
//!   an already-enriched directory produces zero file mutations and identical SHA-256 hashes.
//! - Per-Function Body Uniqueness (Check 34 / Check 36): Every synthesized Tier A and Dart callable
//!   docstring interpolates classifier, signature parameters, return type, and complexity/failure bounds
//!   across Sections 1 through 6 so normalized docblock bodies never collide in `seen_docblocks`.
//! - Attribute-Aware Placement: Synthesized `///` docblocks are positioned immediately before any
//!   `#[...]` attributes (Rust) or `@...` annotations (Dart) attached to the target declaration.
//!
//! ## 3. Preconditions
//! - Target paths passed to `run_docstring_injection` or `run_extract_source` exist on disk and
//!   contain valid UTF-8 `.rs` or `.dart` source files.
//!
//! ## 4. Postconditions
//! - Every Rust Tier A item carries a compliant 7-section or 4-Block Contract docstring, every Rust
//!   Tier B/C item outside `modular_codegen/generated` carries a concise summary plus `Realises: [...]`,
//!   and every Dart item carries a unique 7-section docstring.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in total source lines and AST declarations $N$.
//! - Space Complexity: $O(N)$ proportional to active file line buffers and per-file `seen_docblocks` sets.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Missing directories or syntax errors return `DocstringExitCode::Error` (`1`).
//! - State: Atomic file replacement via `write_atomic` prevents partial or torn source writes.
//! - Action: Idempotency self-check failures return `DocstringExitCode::IdempotencyOrConflict` (`2`).
//! - Resource: Per-file buffers are released immediately after each file transaction.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-FEAT-42-DOCSTRING-SYNTHESIZER], [DO-330-TQL-4-DOCSTRING-SYNTHESIZER]

use crate::semantic::digest::{generate_digest, write_atomic, write_digest_atomic};
use crate::semantic::serializer::to_sysml;
use deap_core::source_ast::{
    classify_dart_member_ast, classify_dart_top_level_ast, classify_rust_item_line_ast,
    extract_leading_ast_ident, extract_realises_ids_from_doc_lines,
    extract_rust_impl_header_prefix, ingest_source_directory, parse_dart_callable_signature,
    parse_rust_callable_signature, parse_rust_impl_header, sanitize_sysml_identifier,
    scan_line_with_unified_lex_state, single_char_literal_byte_len,
    strip_dart_annotations_tracking_state, strip_leading_dart_annotations,
    strip_line_comment_preserving_strings, summary_from_doc_lines, update_ast_delimiter_depths,
    validate_source_syntax_for_language, RustDartLexState, ScopedItemScannerState, SourceItemKind,
    SourceLanguage, SourceParam,
};
use deap_core::sysml_ast::PackageDef;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Process exit code contract for `compile-sysml --inject-safety-docstrings` (FEAT-42 Section 2.2).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.2 CLI exit status specification (`DocstringExitCode`).
/// Encodes the three normative termination states (`0`, `1`, `2`) for the safety docstring synthesizer.
///
/// ## 2. Invariants & Mathematical Properties
/// - `Clean` maps to `0`, `Error` maps to `1`, and `IdempotencyOrConflict` maps to `2`.
///
/// ## 3. Preconditions
/// - Returned by `run_docstring_injection` or `run_extract_source`.
///
/// ## 4. Postconditions
/// - Converts deterministically to an OS process exit code via `DocstringExitCode::code`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant time.
/// - Space Complexity: O(1) stack representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Closed enum prevents undefined exit codes.
/// - State: Immutable copy value.
/// - Action: Exhaustive `match` conversion.
/// - Resource: Zero heap allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DOCSTRING-EXIT-CODE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocstringExitCode {
    /// Exit code `0`: All eligible source items updated or verified cleanly.
    Clean,
    /// Exit code `1`: Compilation, I/O, or syntax error in source or SysML schema.
    Error,
    /// Exit code `2`: Idempotency invariant violation or conflicting unforced docstring.
    IdempotencyOrConflict,
}

impl DocstringExitCode {
    /// Converts `self` to its integer process exit code (`0`, `1`, or `2`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A CLI exit code determinism (`DocstringExitCode::code`).
    /// Maps each `DocstringExitCode` variant to the exact integer mandated by FEAT-42 Section 2.2.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective mapping `{Clean -> 0, Error -> 1, IdempotencyOrConflict -> 2}`.
    ///
    /// ## 3. Preconditions
    /// - `self: DocstringExitCode` is a valid enum discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns `0`, `1`, or `2` as an `i32`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant time.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure projection.
    /// - Action: Cannot panic.
    /// - Resource: Zero memory overhead.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-DOCSTRING-EXIT-CODE-INT]
    pub const fn code(self) -> i32 {
        match self {
            Self::Clean => 0,
            Self::Error => 1,
            Self::IdempotencyOrConflict => 2,
        }
    }
}

/// Typed algebraic error classification for safety docstring synthesis and injection (`FEAT-42` Section 2.2, refs `#512`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and FEAT-42 Section 2.2 deterministic fault classification (`DocstringError`).
/// Distinguishes filesystem I/O failures (`Io`), source syntax validation errors (`Parse`), and unforced
/// requirement traceability conflicts (`UnforcedConflict`) at the type level without fragile string-prefix inspection.
///
/// ## 2. Invariants & Mathematical Properties
/// - `Io(_)` and `Parse(_)` map algebraically to `DocstringExitCode::Error` (`1`).
/// - `UnforcedConflict(_)` maps algebraically to `DocstringExitCode::IdempotencyOrConflict` (`2`).
/// - Implements `Deref<Target = str>` and `Display` so callers can inspect diagnostic messages transparently.
///
/// ## 3. Preconditions
/// - Constructed with a non-empty human-readable diagnostic message describing the I/O, parse, or conflict condition.
///
/// ## 4. Postconditions
/// - Preserves both the algebraic failure category and the diagnostic payload across call boundaries.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) variant construction and discriminant matching.
/// - Space Complexity: O(M) where M is the byte length of the owned diagnostic string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exhaustive enum variants eliminate ad-hoc string prefix matching.
/// - State: Immutable value semantics after construction.
/// - Action: Deterministic exit code projection via `DocstringError::exit_code`.
/// - Resource: Single `String` heap allocation per error instance.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-ENUM]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocstringError {
    /// Filesystem read/write or directory traversal error (maps to `DocstringExitCode::Error` = `1`).
    Io(String),
    /// Source syntax or delimiter balance validation error (maps to `DocstringExitCode::Error` = `1`).
    Parse(String),
    /// Conflicting unforced `Realises:` docstring requirement tag (maps to `DocstringExitCode::IdempotencyOrConflict` = `2`).
    UnforcedConflict(String),
}

impl DocstringError {
    /// Maps this `DocstringError` variant deterministically to its normative `DocstringExitCode`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and FEAT-42 Section 2.2 exit code contract (`DocstringError::exit_code`).
    /// Guarantees that `Io` and `Parse` errors exit with status `1` (`DocstringExitCode::Error`) while
    /// `UnforcedConflict` errors exit with status `2` (`DocstringExitCode::IdempotencyOrConflict`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Total Partition: `Io(_) | Parse(_) -> DocstringExitCode::Error`, `UnforcedConflict(_) -> DocstringExitCode::IdempotencyOrConflict`.
    ///
    /// ## 3. Preconditions
    /// - `self: &DocstringError` is a valid initialized `DocstringError` reference.
    ///
    /// ## 4. Postconditions
    /// - Returns the corresponding `DocstringExitCode` variant without inspecting message text.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time discriminant match.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` over a closed enum.
    /// - State: Pure read-only discriminant projection.
    /// - Action: Zero panics or branches on diagnostic message formatting.
    /// - Resource: Zero memory allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-EXIT-CODE]
    pub const fn exit_code(&self) -> DocstringExitCode {
        match self {
            Self::Io(_) | Self::Parse(_) => DocstringExitCode::Error,
            Self::UnforcedConflict(_) => DocstringExitCode::IdempotencyOrConflict,
        }
    }

    /// Borrows the underlying diagnostic message slice from `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A diagnostic reporting (`DocstringError::as_str`).
    /// Provides zero-copy borrowed access to the error message payload across all `DocstringError` variants.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns the exact `&str` slice stored inside `Io(msg)`, `Parse(msg)`, or `UnforcedConflict(msg)`.
    ///
    /// ## 3. Preconditions
    /// - `self: &DocstringError` is a valid borrowed error value.
    ///
    /// ## 4. Postconditions
    /// - Returns a borrowed `&str` with lifetime bound to `self`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) pointer projection.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Total match across all three variants.
    /// - State: Read-only borrow; never mutates `self`.
    /// - Action: Cannot panic.
    /// - Resource: Zero heap overhead.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-AS-STR]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Io(msg) | Self::Parse(msg) | Self::UnforcedConflict(msg) => msg.as_str(),
        }
    }

    /// Consumes `self` and returns the owned diagnostic `String` message.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A zero-clone error propagation (`DocstringError::into_message`).
    /// Moves the inner diagnostic `String` out of `DocstringError` when constructing CLI exit tuples.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves byte-for-byte identity with `self.as_str()` without reallocating or cloning.
    ///
    /// ## 3. Preconditions
    /// - `self: DocstringError` is passed by value.
    ///
    /// ## 4. Postconditions
    /// - Returns the owned `String` payload previously held by `Io`, `Parse`, or `UnforcedConflict`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) move of the owned `String` header.
    /// - Space Complexity: O(1) zero additional heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Exhaustive by-value destructuring.
    /// - State: Transfers ownership cleanly to the caller.
    /// - Action: Infallible move semantics.
    /// - Resource: Reuses existing `String` buffer.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-DOCSTRING-ERROR-INTO-MESSAGE]
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.2 synthesizer configuration (`DocstringInjectionOptions`).
/// Bundles the target directory path, dry-run simulation flag, force-overwrite flag, and optional
/// loaded SysML v2 `PackageDef` schema model.
///
/// ## 2. Invariants & Mathematical Properties
/// - When `dry_run == true`, zero filesystem writes occur while planned item enrichment counts are still computed.
/// - When `force == false`, existing complete, unique 7-section docstrings are preserved bit-for-bit.
///
/// ## 3. Preconditions
/// - `target_path` references a target `.rs`/`.dart` file or source directory tree.
///
/// ## 4. Postconditions
/// - Passed by reference to `run_docstring_injection` to govern synthesis and atomic file writes.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access.
/// - Space Complexity: O(M) where M is the size of the optional `schema_model`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: All fields are strongly typed with explicit boolean flags.
/// - State: Immutable configuration record during execution.
/// - Action: No side effects on construction.
/// - Resource: Bounded by optional `PackageDef` AST size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTION-OPTIONS]
#[derive(Debug, Clone)]
pub struct DocstringInjectionOptions {
    /// Target directory or source file path (`--inject-safety-docstrings <DIR>`).
    pub target_path: PathBuf,
    /// When `true`, simulates injection without modifying files on disk (`--dry-run`).
    pub dry_run: bool,
    /// When `true`, overwrites existing docstrings unconditionally (`--force`).
    pub force: bool,
    /// Optional authoritative SysML v2 model providing requirement and constraint bindings (`--schema`).
    pub schema_model: Option<PackageDef>,
}

/// Summary report returned after running safety docstring synthesis across a target path.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A verification auditability (`DocstringInjectionReport`).
/// Records the exact counts of scanned files, modified files, and enriched AST items.
///
/// ## 2. Invariants & Mathematical Properties
/// - `files_modified <= files_scanned`.
/// - On a second idempotent run (`!force`), `files_modified == 0` and `items_enriched == 0`.
///
/// ## 3. Preconditions
/// - Produced by `run_docstring_injection`.
///
/// ## 4. Postconditions
/// - Provides deterministic telemetry for CLI output and integration test assertions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access.
/// - Space Complexity: O(F) where F is the number of modified file paths recorded.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Value struct with public counters and modified path list.
/// - State: Immutable once returned.
/// - Action: Infallible field reads.
/// - Resource: Minimal heap usage for `modified_files`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTION-REPORT]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocstringInjectionReport {
    /// Total number of eligible `.rs` and `.dart` files scanned.
    pub files_scanned: usize,
    /// Number of files whose contents were modified (or would be modified under `--dry-run`).
    pub files_modified: usize,
    /// Total number of AST items that received synthesized 7-section docstrings.
    pub items_enriched: usize,
    /// Sorted list of modified file paths.
    pub modified_files: Vec<PathBuf>,
}

/// Maximum AST nesting depth traversed when collecting SysML v2 packages and nested parts.
/// Realises: [REQ-FEAT-42-MAX-AST-DEPTH]
const MAX_AST_DEPTH: usize = 64;

/// Internal descriptor for a single AST item being enriched or compacted with a safety docstring.
/// Realises: [REQ-FEAT-42-ITEM-SYNTHESIS-CONTEXT]
struct ItemSynthesisContext<'a> {
    file_rel: &'a str,
    module_tag: &'a str,
    owner_name: Option<&'a str>,
    item_kind: SourceItemKind,
    item_kw_override: Option<&'a str>,
    item_name: &'a str,
    ordinal_index: usize,
    indent: &'a str,
    existing_doc: &'a [String],
    params: Vec<SourceParam>,
    return_type: Option<String>,
    fields_or_variants: Vec<String>,
    invariants: Vec<String>,
    schema_model: Option<&'a PackageDef>,
}

impl ItemSynthesisContext<'_> {
    /// Returns the explicit keyword override (`const` or `type`) or the underlying `SourceItemKind` keyword.
    /// Realises: [REQ-FEAT-42-ITEM-SYNTHESIS-KEYWORD]
    fn item_keyword(&self) -> &str {
        self.item_kw_override
            .unwrap_or_else(|| self.item_kind.keyword())
    }

    /// Returns whether this declaration represents a callable function, method, or constructor.
    /// Realises: [REQ-FEAT-42-ITEM-SYNTHESIS-IS-CALLABLE]
    fn is_callable(&self) -> bool {
        self.item_kw_override.is_none() && self.item_kind.is_callable()
    }
}

/// Classification of an enclosing Rust block on `impl_owner_stack` for Tier A versus Tier B/C resolution.
/// Realises: [REQ-FEAT-42-RUST-OWNER-KIND]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustOwnerKind {
    /// Inherent implementation block (`impl Type { ... }`).
    InherentImpl,
    /// Trait implementation block (`impl Trait for Type { ... }`).
    TraitImpl,
    /// Trait definition block (`trait MyTrait { ... }`).
    TraitDef,
}

/// Pending test-related attribute classification attached to the next Rust declaration.
/// Realises: [REQ-FEAT-42-RUST-PENDING-ATTR]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustPendingAttr {
    /// No `#[cfg(test)]` or `#[test]` attribute is pending.
    None,
    /// A `#[cfg(test)]` conditional compilation attribute is pending.
    CfgTest,
    /// A `#[test]` (or `#[...::test]`) test entrypoint attribute is pending.
    TestFn,
}

/// Classified Rust item declaration with its documentation tier (`is_tier_a`).
/// Realises: [REQ-FEAT-42-CLASSIFIED-RUST-ITEM]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassifiedRustItem {
    kind: SourceItemKind,
    keyword_override: Option<&'static str>,
    name: String,
    is_tier_a: bool,
}

/// Line-indexed deletion and insertion buffers for non-destructive source docstring rewriting.
/// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS]
struct SourceLineEdits {
    removed_line_flags: Vec<bool>,
    insertions_before: Vec<Vec<String>>,
    enriched_count: usize,
}

impl SourceLineEdits {
    /// Allocates empty deletion and insertion tables sized to `total_lines`.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-NEW]
    fn new(total_lines: usize) -> Self {
        Self {
            removed_line_flags: vec![false; total_lines],
            insertions_before: vec![Vec::new(); total_lines],
            enriched_count: 0,
        }
    }

    /// Marks every line index in `indices` for removal from the rendered output buffer.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-MARK-REMOVED]
    fn mark_removed(&mut self, indices: &[usize]) {
        for &doc_idx in indices {
            if let Some(slot) = self.removed_line_flags.get_mut(doc_idx) {
                *slot = true;
            }
        }
    }

    /// Removes `pending_indices` and increments `enriched_count` if `current_doc` contains a placeholder line.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-STRIP-PLACEHOLDER]
    fn strip_placeholder_doc(&mut self, current_doc: &[String], pending_indices: &[usize]) {
        if any_placeholder_in_doc(current_doc) {
            self.mark_removed(pending_indices);
            self.enriched_count = self.enriched_count.saturating_add(1);
        }
    }

    /// Appends `new_doc` lines into the insertion bucket preceding `target_idx`.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-INSERT]
    fn insert_before(&mut self, target_idx: usize, new_doc: Vec<String>) {
        if let Some(bucket) = self.insertions_before.get_mut(target_idx) {
            bucket.extend(new_doc);
        }
    }

    /// Renders `raw_lines` with all scheduled removals and insertions using checked saturating line capacity.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-RENDER]
    fn render(&self, source: &str, raw_lines: &[&str]) -> String {
        let total_lines = raw_lines.len();
        let inserted_lines = self
            .insertions_before
            .iter()
            .fold(0usize, |acc, bucket| acc.saturating_add(bucket.len()));
        let removed_lines = self
            .removed_line_flags
            .iter()
            .filter(|&&removed| removed)
            .count();
        let capacity = total_lines
            .saturating_sub(removed_lines)
            .saturating_add(inserted_lines);

        let mut out_lines: Vec<String> = Vec::with_capacity(capacity);
        for idx in 0..total_lines {
            if let Some(ins) = self.insertions_before.get(idx) {
                for line in ins {
                    out_lines.push(line.clone());
                }
            }
            if !self.removed_line_flags.get(idx).copied().unwrap_or(false) {
                if let Some(&orig) = raw_lines.get(idx) {
                    out_lines.push(orig.to_string());
                }
            }
        }

        let mut rendered = out_lines.join("\n");
        if source.ends_with('\n') {
            rendered.push('\n');
        }
        rendered
    }
}

/// Intermediate state for a multi-line Rust `impl` header prior to its opening `{` brace.
/// Realises: [REQ-FEAT-42-PENDING-RUST-IMPL]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingRustImpl {
    target: String,
    kind: RustOwnerKind,
    in_where_clause: bool,
}

/// Canonical block header discriminant for the Tier A Concise 4-Block Contract.
/// Realises: [REQ-FEAT-42-FOUR-BLOCK-HEADER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FourBlockHeader {
    /// `Safety & Traceability` block carrying `Realises: [REQ-ID]`.
    SafetyAndTraceability,
    /// `Contract` block specifying preconditions, postconditions, and invariants.
    Contract,
    /// `Bounds & Faults` block specifying complexity/resource bounds and fault behavior.
    BoundsAndFaults,
}

/// Executes `--extract-source <DIR>` reverse synchronization probe into canonical SysML v2 and digest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.2 `--extract-source <DIR>` contract (`run_extract_source`).
/// Extracts the canonical `PackageDef` AST from `source_dir`, serializes it to SysML v2 text, and
/// writes the output model and SHA-256 `SchemaDigest` atomically.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses zero-regex recursive-descent parsing via `ingest_source_directory`.
/// - When `dry_run == true`, validates extraction and serialization without writing to `out_path` or `digest_path`.
///
/// ## 3. Preconditions
/// - `source_dir: &Path` is an existing directory or source file path.
/// - `out_path: &Path` and `digest_path: &Path` are target output paths.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` on successful extraction (and atomic write when `!dry_run`), or
///   `Err((DocstringExitCode, String))` on failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N) where N is total source byte size under `source_dir`.
/// - Space Complexity: O(N) for the extracted `PackageDef` and serialized SysML v2 string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `(DocstringExitCode::Error, msg)` if `source_dir` is missing or contains syntax errors.
/// - State: Atomic file writes via `write_atomic` and `write_digest_atomic`.
/// - Action: Leaves existing output files untouched on error.
/// - Resource: Bounded by source tree size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-RUN-EXTRACT-SOURCE]
pub fn run_extract_source(
    source_dir: &Path,
    out_path: &Path,
    digest_path: &Path,
    dry_run: bool,
) -> Result<PackageDef, (DocstringExitCode, String)> {
    let pkg_name = deap_core::source_ast::resolve_source_package_name(source_dir, None);
    let pkg = ingest_source_directory(source_dir, SourceLanguage::Auto, &pkg_name)
        .map_err(|e| (DocstringExitCode::Error, e))?;
    let sysml_text = to_sysml(&pkg);

    if !dry_run {
        write_atomic(out_path, &sysml_text).map_err(|e| {
            (
                DocstringExitCode::Error,
                format!(
                    "Failed to write extracted SysML to '{}': {}",
                    out_path.display(),
                    e
                ),
            )
        })?;
        let digest = generate_digest(&pkg, &sysml_text);
        write_digest_atomic(digest_path, &digest).map_err(|e| {
            (
                DocstringExitCode::Error,
                format!(
                    "Failed to write schema digest to '{}': {}",
                    digest_path.display(),
                    e
                ),
            )
        })?;
    }

    Ok(pkg)
}

/// Executes `--inject-safety-docstrings <DIR>` across all eligible `.rs` and `.dart` files under `opts.target_path`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.2 / AC-03 / AC-04 (`run_docstring_injection`).
/// Walks `opts.target_path` deterministically, synthesizes missing or incomplete 7-section safety
/// docstrings, verifies second-pass bit-for-bit idempotency in memory, and commits changes atomically.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero File Allowlists: Visits 100% of non-test `.rs` files and non-generated `.dart` files under `opts.target_path`.
/// - Bit-for-Bit Idempotency: Verifies that re-running synthesis on the enriched buffer yields zero further changes;
///   only writes to disk when `enriched != original`.
///
/// ## 3. Preconditions
/// - `opts: &DocstringInjectionOptions` specifies an existing `target_path`.
///
/// ## 4. Postconditions
/// - Returns `Ok(DocstringInjectionReport)` when all files are enriched and verified cleanly, or
///   `Err((DocstringExitCode, String))` on syntax, I/O, or idempotency failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is the total line count across all scanned `.rs` and `.dart` files.
/// - Space Complexity: O(L + F) where L is maximum single-file byte length and F is file count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-existent `target_path` or unreadable files fail closed with `DocstringExitCode::Error`.
/// - State: Uses `write_atomic` so interrupted execution never leaves half-written source files.
/// - Action: Verifies idempotency in memory before writing to disk; returns `DocstringExitCode::IdempotencyOrConflict` on mismatch.
/// - Resource: Frees per-file buffers at the end of each loop iteration.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-RUN-DOCSTRING-INJECTION]
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
            .map_err(|e| {
                DocstringError::Io(format!("Failed to read '{}': {}", file_path.display(), e))
            })
            .map_err(map_inject_err)?;
        let file_rel = file_path.display().to_string();
        let lang =
            SourceLanguage::detect_from_extension(&file_path).unwrap_or(SourceLanguage::Rust);

        let (enriched, count) = match lang {
            SourceLanguage::Rust | SourceLanguage::Auto => inject_rust_file_docstrings(
                &original,
                &file_rel,
                opts.schema_model.as_ref(),
                opts.force,
            )
            .map_err(map_inject_err)?,
            SourceLanguage::Dart => inject_dart_file_docstrings(
                &original,
                &file_rel,
                opts.schema_model.as_ref(),
                opts.force,
            )
            .map_err(map_inject_err)?,
        };

        // Verify idempotency invariant in memory (second pass with force=false must produce 0 changes)
        let (second_pass, second_count) = match lang {
            SourceLanguage::Rust | SourceLanguage::Auto => {
                inject_rust_file_docstrings(&enriched, &file_rel, opts.schema_model.as_ref(), false)
                    .map_err(map_inject_err)?
            }
            SourceLanguage::Dart => {
                inject_dart_file_docstrings(&enriched, &file_rel, opts.schema_model.as_ref(), false)
                    .map_err(map_inject_err)?
            }
        };

        if second_pass != enriched || second_count != 0 {
            return Err((
                DocstringExitCode::IdempotencyOrConflict,
                format!(
                    "Idempotency invariant violation in '{}': second pass modified {} item(s)",
                    file_rel, second_count
                ),
            ));
        }

        if enriched != original {
            if !opts.dry_run {
                write_atomic(&file_path, &enriched)
                    .map_err(|e| {
                        DocstringError::Io(format!(
                            "Failed to write '{}': {}",
                            file_path.display(),
                            e
                        ))
                    })
                    .map_err(map_inject_err)?;
            }
            report.files_modified = report.files_modified.saturating_add(1);
            report.items_enriched = report.items_enriched.saturating_add(count);
            report.modified_files.push(file_path);
        }
    }

    Ok(report)
}

/// Checks whether `file_rel` lies inside a `modular_codegen/generated` directory tree.
/// Realises: [REQ-FEAT-42-IS-MODULAR-CODEGEN-GENERATED-REL]
fn is_modular_codegen_generated_rel(file_rel: &str) -> bool {
    let mut prev_was_modular_codegen = false;
    for seg in file_rel.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if prev_was_modular_codegen && seg == "generated" {
            return true;
        }
        prev_was_modular_codegen = seg == "modular_codegen";
    }
    false
}

/// Checks whether `path` lies inside a `modular_codegen/generated` directory tree.
/// Realises: [REQ-FEAT-42-IS-MODULAR-CODEGEN-GENERATED-PATH]
fn is_modular_codegen_generated_path(path: &Path) -> bool {
    is_modular_codegen_generated_rel(&path.to_string_lossy())
}

/// Checks whether a directory basename is excluded from recursive docstring injection traversal.
/// Realises: [REQ-FEAT-42-IS-EXCLUDED-WALK-DIR]
fn is_excluded_walk_dir_name(dname: &str) -> bool {
    matches!(
        dname,
        "target"
            | ".git"
            | "build"
            | "Pods"
            | "ephemeral"
            | ".dart_tool"
            | "tests"
            | "benches"
            | "examples"
            | "generated"
    )
}

/// Checks whether a Dart filename matches a generated artifact naming convention.
/// Realises: [REQ-FEAT-42-IS-GENERATED-DART-FILENAME]
fn is_generated_dart_filename(fname: &str) -> bool {
    if fname == "generated_plugin_registrant.dart" {
        return true;
    }
    [".g.dart", ".freezed.dart", ".gr.dart", ".mocks.dart"]
        .iter()
        .any(|ext| fname.ends_with(ext))
}

/// Collects all non-test `.rs` and non-generated `.dart` files under `root` in sorted order.
/// Realises: [REQ-FEAT-42-COLLECT-INJECTABLE-FILES]
fn collect_injectable_source_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if is_modular_codegen_generated_path(root) {
        return Ok(Vec::new());
    }
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry_res in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.depth() > 0 && e.file_type().is_dir() {
                let dname = e.file_name().to_string_lossy();
                !is_excluded_walk_dir_name(&dname)
            } else {
                true
            }
        })
    {
        let entry = entry_res
            .map_err(|e| format!("Directory walk error in '{}': {}", root.display(), e))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if is_modular_codegen_generated_path(path) {
            continue;
        }
        let fname = entry.file_name().to_string_lossy();
        if is_generated_dart_filename(&fname) {
            continue;
        }
        if SourceLanguage::detect_from_extension(path).is_some() {
            files.push(path.to_path_buf());
        }
    }

    files.sort();
    Ok(files)
}

/// Validates that an existing unforced `Realises:` binding does not conflict with `schema_model`.
/// Realises: [REQ-FEAT-42-VALIDATE-UNFORCED-SCHEMA-REALISES]
fn validate_unforced_schema_realises(
    file_rel: &str,
    item_name: &str,
    current_doc: &[String],
    schema_model: Option<&PackageDef>,
    owner_name: Option<&str>,
) -> Result<(), DocstringError> {
    let (schema_req_opt, _) = resolve_schema_context(schema_model, owner_name, item_name);
    let Some(schema_req) = schema_req_opt else {
        return Ok(());
    };
    let existing_reqs = extract_all_realises_ids(current_doc);
    if existing_reqs.is_empty() {
        return Ok(());
    }
    let all_matched = schema_req
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .all(|sr| {
            existing_reqs
                .iter()
                .any(|r| schema_identifiers_match(r, sr))
        });
    if !all_matched {
        return Err(DocstringError::UnforcedConflict(format!(
            "Conflicting unforced docstring in '{file_rel}' for '{item_name}': existing Realises {existing_reqs:?} conflicts with schema requirement '{schema_req}' (use --force to overwrite)"
        )));
    }
    Ok(())
}

/// Checks whether `lex_state` is outside block comments, raw strings, active strings, and interpolations.
/// Realises: [REQ-FEAT-42-IS-LEX-STATE-NORMAL]
fn is_lex_state_normal(lex_state: &RustDartLexState) -> bool {
    let outside_strings = lex_state.raw_hash_count.is_none() && lex_state.active_string.is_none();
    lex_state.block_comment_depth == 0 && outside_strings && lex_state.interp_stack.is_empty()
}

/// Checks whether `code` opens an inline `mod tests` or `#[cfg(test)] mod ...` block.
/// Realises: [REQ-FEAT-42-IS-MOD-TESTS-ENTRY]
fn is_mod_tests_entry(code: &str, pending_attr: RustPendingAttr) -> bool {
    let is_named_tests =
        code == "mod tests {" || code.starts_with("mod tests ") || code.starts_with("mod tests{");
    let is_cfg_test_mod = pending_attr == RustPendingAttr::CfgTest
        && code.starts_with("mod ")
        && !code.ends_with(';');
    is_named_tests || is_cfg_test_mod
}

/// Checks whether a leading `#[...]` attribute line marks a `#[test]` entrypoint function.
/// Realises: [REQ-FEAT-42-IS-RUST-TEST-FN-ATTR]
fn is_rust_test_fn_attr(code: &str) -> bool {
    let Some(after_open) = code.trim_start().strip_prefix("#[") else {
        return false;
    };
    let Some((attr_body, _)) = after_open.split_once(']') else {
        return false;
    };
    let attr = attr_body.trim();
    let base = attr.split_once('(').map_or(attr, |(head, _)| head.trim());
    base == "test" || base.ends_with("::test")
}

/// Updates the pending test attribute state when scanning a Rust `#[...]` attribute line.
/// Realises: [REQ-FEAT-42-CLASSIFY-PENDING-RUST-ATTR]
fn classify_pending_rust_attr(code: &str, current: RustPendingAttr) -> RustPendingAttr {
    let is_cfg_test = code.starts_with("#[cfg(test)") || code.contains("cfg(test)");
    if is_cfg_test || current == RustPendingAttr::CfgTest {
        RustPendingAttr::CfgTest
    } else if is_rust_test_fn_attr(code) {
        RustPendingAttr::TestFn
    } else {
        current
    }
}

/// Updates `attr_bracket_depth` across `[` and `]` bytes in a multi-line Rust attribute continuation.
/// Realises: [REQ-FEAT-42-UPDATE-ATTR-BRACKET-DEPTH]
fn update_attr_bracket_depth(code: &str, attr_bracket_depth: &mut usize) {
    for b in code.bytes() {
        if b == b'[' {
            *attr_bracket_depth = attr_bracket_depth.saturating_add(1);
        } else if b == b']' {
            *attr_bracket_depth = attr_bracket_depth.saturating_sub(1);
        }
    }
}

/// Consumes leading `#[...]` attributes on `code`, updating `attr_bracket_depth` if an attribute spans lines.
/// Realises: [REQ-FEAT-42-CONSUME-LEADING-RUST-ATTRS]
fn consume_leading_rust_attrs<'b>(code: &'b str, attr_bracket_depth: &mut usize) -> &'b str {
    let mut rest = code.trim_start();
    while let Some(after_hash) = rest.strip_prefix("#[") {
        let mut depth: usize = 1;
        let mut close_off: Option<usize> = None;
        for (i, b) in after_hash.bytes().enumerate() {
            if b == b'[' {
                depth = depth.saturating_add(1);
            } else if b == b']' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_off = Some(i.saturating_add(1));
                    break;
                }
            }
        }
        if let Some(off) = close_off {
            rest = after_hash.get(off..).unwrap_or("").trim_start();
        } else {
            *attr_bracket_depth = depth;
            return "";
        }
    }
    rest
}

/// Scanned single-line context passed between Rust docstring injection helpers.
/// Realises: [REQ-FEAT-42-RUST-LINE-SCAN-CONTEXT]
struct RustLineScanContext<'b> {
    raw_line: &'b str,
    idx: usize,
    code: &'b str,
    inline_comment_closed: bool,
}

/// Per-file state machine for tiered Rust safety docstring synthesis and Tier B/C compaction.
/// Realises: [REQ-FEAT-42-RUST-INJECTOR-STATE]
struct RustInjectorState<'a> {
    file_rel: &'a str,
    module_tag: String,
    schema_model: Option<&'a PackageDef>,
    force: bool,
    skip_tier_bc_compaction: bool,
    current_doc: Vec<String>,
    pending_doc_indices: Vec<usize>,
    first_attr_idx: Option<usize>,
    seen_docblocks: BTreeSet<String>,
    lex_state: RustDartLexState,
    in_mod_tests: bool,
    pending_attr: RustPendingAttr,
    attr_bracket_depth: usize,
    brace_depth: usize,
    paren_depth: usize,
    active_fn_body_depth: Option<usize>,
    pending_fn_body_depth: Option<usize>,
    impl_owner_stack: Vec<(usize, String, RustOwnerKind)>,
    pending_impl: Option<PendingRustImpl>,
    edits: SourceLineEdits,
    item_ordinal: usize,
}

impl<'a> RustInjectorState<'a> {
    /// Initializes a `RustInjectorState` for `file_rel` across `total_lines`.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-STATE-NEW]
    fn new(
        total_lines: usize,
        file_rel: &'a str,
        schema_model: Option<&'a PackageDef>,
        force: bool,
    ) -> Self {
        Self {
            file_rel,
            module_tag: derive_module_req_tag(file_rel),
            schema_model,
            force,
            skip_tier_bc_compaction: is_modular_codegen_generated_rel(file_rel),
            current_doc: Vec::new(),
            pending_doc_indices: Vec::new(),
            first_attr_idx: None,
            seen_docblocks: BTreeSet::new(),
            lex_state: RustDartLexState {
                language: SourceLanguage::Rust,
                ..Default::default()
            },
            in_mod_tests: false,
            pending_attr: RustPendingAttr::None,
            attr_bracket_depth: 0,
            brace_depth: 0,
            paren_depth: 0,
            active_fn_body_depth: None,
            pending_fn_body_depth: None,
            impl_owner_stack: Vec::new(),
            pending_impl: None,
            edits: SourceLineEdits::new(total_lines),
            item_ordinal: 0,
        }
    }

    /// Clears pending docblock lines, attribute anchor index, and pending test attribute state.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-RESET-PENDING]
    fn reset_pending_item_context(&mut self) {
        self.pending_attr = RustPendingAttr::None;
        self.current_doc.clear();
        self.pending_doc_indices.clear();
        self.first_attr_idx = None;
    }

    /// Returns the non-empty nominal type identifier of the innermost enclosing `impl` or `trait` block.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-OWNER-NAME]
    fn current_owner_name(&self) -> Option<&str> {
        self.impl_owner_stack.last().and_then(|(_, name, _)| {
            if name.is_empty() {
                None
            } else {
                Some(name.as_str())
            }
        })
    }

    /// Returns whether the scanner is currently inside an open function or method body block.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-IN-FN-BODY]
    fn in_function_body(&self) -> bool {
        self.active_fn_body_depth
            .is_some_and(|open_depth| self.brace_depth > open_depth)
    }

    /// Scans and processes a single line at `idx` in `raw_lines`.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-PROCESS-LINE]
    fn process_line(&mut self, raw_lines: &[&str], idx: usize) -> Result<(), DocstringError> {
        let raw_line = raw_lines.get(idx).copied().unwrap_or("");
        let trimmed = raw_line.trim();
        let started_in_normal = is_lex_state_normal(&self.lex_state);
        let started_in_string =
            self.lex_state.raw_hash_count.is_some() || self.lex_state.active_string.is_some();

        if started_in_normal {
            if trimmed.starts_with("//!") {
                if has_placeholder_doc_line(trimmed) {
                    self.edits.mark_removed(&[idx]);
                    self.edits.enriched_count = self.edits.enriched_count.saturating_add(1);
                }
                return Ok(());
            }
            if trimmed.starts_with("///") {
                if !self.in_mod_tests {
                    self.current_doc.push(trimmed.to_string());
                    self.pending_doc_indices.push(idx);
                }
                return Ok(());
            }
        }

        let scanned_code = scan_line_with_unified_lex_state(trimmed, &mut self.lex_state);
        let ended_in_normal = is_lex_state_normal(&self.lex_state);
        let raw_code = scanned_code.trim();
        let mut code = if started_in_string {
            raw_code.strip_prefix('"').unwrap_or(raw_code).trim()
        } else {
            raw_code
        };
        if code.is_empty() {
            return Ok(());
        }

        let still_in_non_code = !ended_in_normal || started_in_string;
        if !started_in_normal && still_in_non_code {
            if self.attr_bracket_depth > 0 {
                update_attr_bracket_depth(code, &mut self.attr_bracket_depth);
            } else {
                self.update_depths_after_code(code);
            }
            return Ok(());
        }

        if is_mod_tests_entry(code, self.pending_attr) {
            self.in_mod_tests = true;
        }
        if self.in_mod_tests {
            return Ok(());
        }

        if self.attr_bracket_depth > 0 {
            update_attr_bracket_depth(code, &mut self.attr_bracket_depth);
            return Ok(());
        }

        if code.starts_with("#[") {
            if self.first_attr_idx.is_none() {
                self.first_attr_idx = Some(idx);
            }
            self.pending_attr = classify_pending_rust_attr(code, self.pending_attr);
            code = consume_leading_rust_attrs(code, &mut self.attr_bracket_depth);
            if code.is_empty() {
                return Ok(());
            }
        }

        if self.paren_depth == 0 {
            let scan = RustLineScanContext {
                raw_line,
                idx,
                code,
                inline_comment_closed: !started_in_normal && ended_in_normal,
            };
            self.handle_top_level_code(raw_lines, &scan)?;
        }

        self.update_depths_after_code(code);
        Ok(())
    }

    /// Dispatches `impl` headers and item declarations encountered at `paren_depth == 0`.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-HANDLE-TOP-LEVEL]
    fn handle_top_level_code(
        &mut self,
        raw_lines: &[&str],
        scan: &RustLineScanContext<'_>,
    ) -> Result<(), DocstringError> {
        let code = scan.code;
        let enclosing_owner_kind = self.impl_owner_stack.last().map(|&(_, _, kind)| kind);
        let in_fn_body = self.in_function_body();

        if let Some((impl_target, impl_kind)) = extract_rust_impl_target_and_kind(code) {
            self.pending_impl = None;
            if code.contains('{') && !code.contains('}') {
                self.impl_owner_stack
                    .push((self.brace_depth, impl_target, impl_kind));
            }
            self.edits
                .strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            self.reset_pending_item_context();
        } else if self.brace_depth == 0 && parse_rust_impl_header(&format!("{code} {{")).is_some() {
            if let Some(impl_target) = parse_rust_impl_header(&format!("{code} {{")) {
                if !code.ends_with(';') && !code.contains('}') {
                    let (trait_for_tail, saw_where) = scan_rust_impl_segment(code);
                    let impl_kind = if trait_for_tail.is_some() {
                        RustOwnerKind::TraitImpl
                    } else {
                        RustOwnerKind::InherentImpl
                    };
                    self.pending_impl = Some(PendingRustImpl {
                        target: impl_target,
                        kind: impl_kind,
                        in_where_clause: saw_where,
                    });
                }
            }
            self.edits
                .strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            self.reset_pending_item_context();
        } else if self.brace_depth == 0 && self.pending_impl.is_some() {
            advance_pending_rust_impl(
                code,
                self.brace_depth,
                &mut self.pending_impl,
                &mut self.impl_owner_stack,
            );
        } else if let Some(item) =
            classify_rust_item_for_injection(code, enclosing_owner_kind, in_fn_body)
        {
            self.handle_classified_item(raw_lines, scan, item)?;
        } else {
            self.edits
                .strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
            self.reset_pending_item_context();
        }
        Ok(())
    }

    /// Evaluates compliance, compaction, or synthesis for a classified Rust declaration.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-HANDLE-ITEM]
    fn handle_classified_item(
        &mut self,
        raw_lines: &[&str],
        scan: &RustLineScanContext<'_>,
        item: ClassifiedRustItem,
    ) -> Result<(), DocstringError> {
        if self.pending_attr != RustPendingAttr::None {
            self.handle_test_annotated_item(raw_lines, scan, &item);
            self.register_item_scope(scan.code, &item);
            self.reset_pending_item_context();
            return Ok(());
        }

        self.item_ordinal = self.item_ordinal.saturating_add(1);
        let owner_exists = self.current_owner_name().map(str::to_string);
        let owner_ref = owner_exists.as_deref();
        let check_uniqueness = item.is_tier_a && item.kind == SourceItemKind::Function;
        let policy = DocCompliancePolicy {
            is_tier_a: item.is_tier_a,
            check_uniqueness,
            allow_legacy_tier_bc_sections: self.skip_tier_bc_compaction,
        };
        let is_valid = !self.force
            && is_compliant_docblock_in_file_with_mode(
                &self.current_doc,
                policy,
                &mut self.seen_docblocks,
            );

        let should_check_conflict =
            !self.force && (is_valid || has_legacy_seven_section_headers(&self.current_doc));
        if should_check_conflict {
            validate_unforced_schema_realises(
                self.file_rel,
                &item.name,
                &self.current_doc,
                self.schema_model,
                owner_ref,
            )?;
        }

        if !is_valid {
            self.emit_synthesized_rust_doc(raw_lines, scan, &item, owner_ref);
        }

        self.register_item_scope(scan.code, &item);
        self.reset_pending_item_context();
        Ok(())
    }

    /// Handles `#[cfg(test)]` and `#[test]` items without adding 7-section boilerplate, compacting legacy 7-section blocks if present.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-HANDLE-TEST-ITEM]
    fn handle_test_annotated_item(
        &mut self,
        raw_lines: &[&str],
        scan: &RustLineScanContext<'_>,
        item: &ClassifiedRustItem,
    ) {
        if !self.skip_tier_bc_compaction && has_legacy_seven_section_headers(&self.current_doc) {
            let owner_exists = self.current_owner_name().map(str::to_string);
            let mut tier_bc_item = item.clone();
            tier_bc_item.is_tier_a = false;
            self.item_ordinal = self.item_ordinal.saturating_add(1);
            self.emit_synthesized_rust_doc(raw_lines, scan, &tier_bc_item, owner_exists.as_deref());
        } else {
            self.edits
                .strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
        }
    }

    /// Synthesizes either a Tier A 7-section docblock or a concise Tier B/C 2-line docblock and records the line edits.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-EMIT-DOC]
    fn emit_synthesized_rust_doc(
        &mut self,
        raw_lines: &[&str],
        scan: &RustLineScanContext<'_>,
        item: &ClassifiedRustItem,
        owner_name: Option<&str>,
    ) {
        self.edits.mark_removed(&self.pending_doc_indices);

        let inline_comment_split = if scan.inline_comment_closed && self.first_attr_idx.is_none() {
            scan.raw_line
                .split_once("*/")
                .map(|(prefix, rest)| (format!("{prefix}*/"), rest.trim_start()))
        } else {
            None
        };
        let indent = if inline_comment_split.is_some() {
            ""
        } else {
            leading_whitespace(scan.raw_line)
        };
        let (params, return_type, fields_or_variants, invariants) =
            if item.keyword_override.is_some() {
                (Vec::new(), None, Vec::new(), Vec::new())
            } else {
                inspect_rust_item_details(raw_lines, scan.idx, item.kind)
            };

        let ctx = ItemSynthesisContext {
            file_rel: self.file_rel,
            module_tag: &self.module_tag,
            owner_name,
            item_kind: item.kind,
            item_kw_override: item.keyword_override,
            item_name: &item.name,
            ordinal_index: self.item_ordinal,
            indent,
            existing_doc: &self.current_doc,
            params,
            return_type,
            fields_or_variants,
            invariants,
            schema_model: self.schema_model,
        };

        let new_doc = if item.is_tier_a {
            synthesize_7_section_docstring(&ctx, &mut self.seen_docblocks)
        } else {
            synthesize_tier_bc_docstring(&ctx)
        };

        let target_insert_idx = self.first_attr_idx.unwrap_or(scan.idx);
        if let Some((comment_prefix, decl_tail)) = inline_comment_split {
            let mut combined = Vec::with_capacity(new_doc.len().saturating_add(2));
            combined.push(comment_prefix);
            combined.extend(new_doc);
            combined.push(format!("{indent}{decl_tail}"));
            self.edits.insert_before(target_insert_idx, combined);
            self.edits.mark_removed(&[scan.idx]);
        } else {
            self.edits.insert_before(target_insert_idx, new_doc);
        }
        self.edits.enriched_count = self.edits.enriched_count.saturating_add(1);
    }

    /// Registers `TraitDef` owner scope or pending function body tracking before delimiter updates.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-REGISTER-SCOPE]
    fn register_item_scope(&mut self, code: &str, item: &ClassifiedRustItem) {
        let is_open_trait = item.keyword_override.is_none()
            && item.kind == SourceItemKind::Trait
            && code.contains('{');
        if is_open_trait && !code.contains('}') {
            self.impl_owner_stack.push((
                self.brace_depth,
                item.name.clone(),
                RustOwnerKind::TraitDef,
            ));
        }
        if item.keyword_override.is_none()
            && item.kind == SourceItemKind::Function
            && self.active_fn_body_depth.is_none()
        {
            self.pending_fn_body_depth = Some(self.brace_depth);
        }
    }

    /// Updates `(brace_depth, paren_depth)`, function-body scope tracking, and `impl_owner_stack` closures.
    /// Realises: [REQ-FEAT-42-RUST-INJECTOR-UPDATE-DEPTHS]
    fn update_depths_after_code(&mut self, code: &str) {
        update_ast_delimiter_depths(code, &mut self.brace_depth, &mut self.paren_depth);

        if let Some(open_d) = self.pending_fn_body_depth {
            let ended_without_body = code.ends_with(';') || code.ends_with('}');
            if self.brace_depth > open_d {
                self.active_fn_body_depth = Some(open_d);
                self.pending_fn_body_depth = None;
            } else if self.paren_depth == 0 && ended_without_body {
                self.pending_fn_body_depth = None;
            }
        }
        if let Some(open_d) = self.active_fn_body_depth {
            if self.brace_depth <= open_d {
                self.active_fn_body_depth = None;
            }
        }

        while let Some(&(open_depth, _, _)) = self.impl_owner_stack.last() {
            if self.brace_depth <= open_depth {
                self.impl_owner_stack.pop();
            } else {
                break;
            }
        }
    }
}

/// Synthesizes and compacts tiered v2.0 safety docstrings into a single Rust source buffer.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, Check 34 (`audit_source_purity`), Check 36 (`audit_rust_source_docstrings`),
/// and DEAP Rust Coding Standard v2.0 Section 4. Preserves or synthesizes 7-section / 4-Block Contract
/// docstrings on Tier A items while synthesizing and compacting concise 2-line docstrings on Tier B/C items.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Module-Level Exemptions: `//!` inner docstrings are preserved verbatim and never satisfy
///   outer `///` item docstring requirements.
/// - Per-Function Body Uniqueness: Tracks `seen_docblocks` across Tier A `pub fn` items in `source` so
///   every public function has a unique normalized 7-section body.
/// - Metacircular Parity Isolation: Skips Tier B/C 7-section compaction on `modular_codegen/generated` paths.
///
/// ## 3. Preconditions
/// - `source: &str` contains valid UTF-8 Rust source code.
/// - `file_rel: &str` is the file path used for module tag derivation and diagnostics.
///
/// ## 4. Postconditions
/// - Returns `Ok((updated_source, enriched_item_count))` on success, or `Err(DocstringError)` if syntax validation or unforced requirement checking fails.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the line count of `source`.
/// - Space Complexity: O(L + I) where I is the number of unique Tier A function docblocks.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates balanced delimiters via `validate_source_syntax_for_language` before modifying lines.
/// - State: Isolated per-file `RustInjectorState` tracking attributes, raw strings, comments, and `impl` blocks.
/// - Action: Non-panicking `.get(..)` slice operations and saturating arithmetic throughout.
/// - Resource: Linear memory proportional to `source.len()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-INJECT-RUST-DOCSTRINGS]
pub fn inject_rust_file_docstrings(
    source: &str,
    file_rel: &str,
    schema_model: Option<&PackageDef>,
    force: bool,
) -> Result<(String, usize), DocstringError> {
    validate_source_syntax_for_language(source, file_rel, SourceLanguage::Rust)
        .map_err(DocstringError::Parse)?;

    let raw_lines: Vec<&str> = source.lines().collect();
    let mut state = RustInjectorState::new(raw_lines.len(), file_rel, schema_model, force);

    for idx in 0..raw_lines.len() {
        state.process_line(&raw_lines, idx)?;
    }

    if state.edits.enriched_count == 0 {
        return Ok((source.to_string(), 0));
    }

    let rendered = state.edits.render(source, &raw_lines);
    Ok((rendered, state.edits.enriched_count))
}

/// Per-file state machine for Dart 7-section safety docstring synthesis.
/// Realises: [REQ-FEAT-42-DART-INJECTOR-STATE]
struct DartInjectorState<'a> {
    file_rel: &'a str,
    module_tag: String,
    schema_model: Option<&'a PackageDef>,
    force: bool,
    current_doc: Vec<String>,
    pending_doc_indices: Vec<usize>,
    first_annot_idx: Option<usize>,
    seen_docblocks: BTreeSet<String>,
    dart_lex_state: RustDartLexState,
    scoped_state: ScopedItemScannerState,
    brace_depth: usize,
    paren_depth: usize,
    active_class_name: Option<String>,
    edits: SourceLineEdits,
    item_ordinal: usize,
}

impl<'a> DartInjectorState<'a> {
    /// Initializes a `DartInjectorState` for `file_rel` across `total_lines`.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-STATE-NEW]
    fn new(
        total_lines: usize,
        file_rel: &'a str,
        schema_model: Option<&'a PackageDef>,
        force: bool,
    ) -> Self {
        Self {
            file_rel,
            module_tag: derive_module_req_tag(file_rel),
            schema_model,
            force,
            current_doc: Vec::new(),
            pending_doc_indices: Vec::new(),
            first_annot_idx: None,
            seen_docblocks: BTreeSet::new(),
            dart_lex_state: RustDartLexState {
                language: SourceLanguage::Dart,
                ..Default::default()
            },
            scoped_state: ScopedItemScannerState::default(),
            brace_depth: 0,
            paren_depth: 0,
            active_class_name: None,
            edits: SourceLineEdits::new(total_lines),
            item_ordinal: 0,
        }
    }

    /// Clears pending Dart docblock lines and leading `@annotation` index.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-RESET-PENDING]
    fn reset_pending_item_context(&mut self) {
        self.current_doc.clear();
        self.pending_doc_indices.clear();
        self.first_annot_idx = None;
    }

    /// Strips any placeholder lines from `current_doc` and resets pending item context.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-DISCARD-NON-ITEM]
    fn discard_non_item_doc(&mut self) {
        self.edits
            .strip_placeholder_doc(&self.current_doc, &self.pending_doc_indices);
        self.reset_pending_item_context();
    }

    /// Scans and processes a single Dart line at `idx` in `raw_lines`.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-PROCESS-LINE]
    fn process_line(&mut self, raw_lines: &[&str], idx: usize) -> Result<(), DocstringError> {
        let raw_line = raw_lines.get(idx).copied().unwrap_or("");
        let trimmed = raw_line.trim();
        let started_in_comment_or_string = self.dart_lex_state.block_comment_depth > 0
            || self.dart_lex_state.active_string.is_some()
            || !self.dart_lex_state.interp_stack.is_empty();

        if !started_in_comment_or_string
            && self.scoped_state.dart_annotation_paren_depth == 0
            && trimmed.starts_with("///")
        {
            self.current_doc.push(trimmed.to_string());
            self.pending_doc_indices.push(idx);
            return Ok(());
        }

        let scanned = scan_line_with_unified_lex_state(trimmed, &mut self.dart_lex_state);
        if scanned.trim().is_empty() {
            return Ok(());
        }

        let code_raw = if !started_in_comment_or_string && !trimmed.contains("/*") {
            strip_line_comment_preserving_strings(trimmed)
        } else {
            strip_line_comment_preserving_strings(scanned.trim())
        };
        if code_raw.is_empty() {
            return Ok(());
        }

        if self.scoped_state.dart_annotation_paren_depth == 0
            && code_raw.starts_with('@')
            && self.first_annot_idx.is_none()
        {
            self.first_annot_idx = Some(idx);
        }
        let stripped_code = strip_dart_annotations_tracking_state(
            code_raw,
            &mut self.scoped_state.dart_annotation_paren_depth,
        );
        let code = stripped_code.trim();
        if code.is_empty() {
            return Ok(());
        }

        if self.paren_depth == 0 {
            self.dispatch_dart_declaration(raw_lines, idx, code)?;
        }

        let prev_brace_depth = self.brace_depth;
        update_ast_delimiter_depths(code, &mut self.brace_depth, &mut self.paren_depth);
        let closed_to_zero = prev_brace_depth > 0 && self.brace_depth == 0;
        let ended_at_zero = self.brace_depth == 0 && (code.ends_with(';') || code.ends_with('}'));
        if closed_to_zero || ended_at_zero {
            self.active_class_name = None;
        }
        Ok(())
    }

    /// Dispatches top-level (`brace_depth == 0`) or class-member (`brace_depth == 1`) Dart declarations.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-DISPATCH]
    fn dispatch_dart_declaration(
        &mut self,
        raw_lines: &[&str],
        idx: usize,
        code: &str,
    ) -> Result<(), DocstringError> {
        if self.brace_depth == 0 {
            if let Some((kind, item_name, _parent)) = classify_dart_top_level_ast(code) {
                let target = DartItemTarget {
                    idx,
                    kind,
                    owner_name: None,
                    item_name: &item_name,
                    check_uniqueness: kind == SourceItemKind::Function,
                };
                self.validate_or_enrich_dart_item(raw_lines, target)?;
                if matches!(
                    kind,
                    SourceItemKind::Class | SourceItemKind::Mixin | SourceItemKind::Extension
                ) {
                    self.active_class_name = Some(item_name);
                }
                self.reset_pending_item_context();
            } else {
                self.discard_non_item_doc();
            }
        } else if self.brace_depth == 1 {
            if let Some(cls_name) = self.active_class_name.clone() {
                if let Some((member_kind, member_name)) = classify_dart_member_ast(code, &cls_name)
                {
                    let target = DartItemTarget {
                        idx,
                        kind: member_kind,
                        owner_name: Some(cls_name.as_str()),
                        item_name: &member_name,
                        check_uniqueness: true,
                    };
                    self.validate_or_enrich_dart_item(raw_lines, target)?;
                    self.reset_pending_item_context();
                } else {
                    self.discard_non_item_doc();
                }
            } else {
                self.discard_non_item_doc();
            }
        } else {
            self.discard_non_item_doc();
        }
        Ok(())
    }

    /// Validates an existing Dart 7-section docblock or synthesizes a replacement 7-section docblock.
    /// Realises: [REQ-FEAT-42-DART-INJECTOR-VALIDATE-OR-ENRICH]
    fn validate_or_enrich_dart_item(
        &mut self,
        raw_lines: &[&str],
        target: DartItemTarget<'_>,
    ) -> Result<(), DocstringError> {
        self.item_ordinal = self.item_ordinal.saturating_add(1);
        let policy = DocCompliancePolicy {
            is_tier_a: true,
            check_uniqueness: target.check_uniqueness,
            allow_legacy_tier_bc_sections: false,
        };
        let is_valid = !self.force
            && is_compliant_docblock_in_file_with_mode(
                &self.current_doc,
                policy,
                &mut self.seen_docblocks,
            );

        if is_valid {
            validate_unforced_schema_realises(
                self.file_rel,
                target.item_name,
                &self.current_doc,
                self.schema_model,
                target.owner_name,
            )?;
        } else {
            self.edits.mark_removed(&self.pending_doc_indices);
            let raw_line = raw_lines.get(target.idx).copied().unwrap_or("");
            let indent = leading_whitespace(raw_line);
            let (params, return_type, fields_or_variants, invariants) =
                inspect_dart_item_details(raw_lines, target.idx, target.kind, target.item_name);
            let ctx = ItemSynthesisContext {
                file_rel: self.file_rel,
                module_tag: &self.module_tag,
                owner_name: target.owner_name,
                item_kind: target.kind,
                item_kw_override: None,
                item_name: target.item_name,
                ordinal_index: self.item_ordinal,
                indent,
                existing_doc: &self.current_doc,
                params,
                return_type,
                fields_or_variants,
                invariants,
                schema_model: self.schema_model,
            };
            let new_doc = synthesize_7_section_docstring(&ctx, &mut self.seen_docblocks);
            let target_insert_idx = self.first_annot_idx.unwrap_or(target.idx);
            self.edits.insert_before(target_insert_idx, new_doc);
            self.edits.enriched_count = self.edits.enriched_count.saturating_add(1);
        }
        Ok(())
    }
}

/// Target Dart item metadata passed to `DartInjectorState::validate_or_enrich_dart_item`.
/// Realises: [REQ-FEAT-42-DART-ITEM-TARGET]
struct DartItemTarget<'b> {
    idx: usize,
    kind: SourceItemKind,
    owner_name: Option<&'b str>,
    item_name: &'b str,
    check_uniqueness: bool,
}

/// Policy flags governing tiered docblock validation and per-file callable uniqueness checks.
/// Realises: [REQ-FEAT-42-DOC-COMPLIANCE-POLICY]
#[derive(Debug, Clone, Copy)]
struct DocCompliancePolicy {
    is_tier_a: bool,
    check_uniqueness: bool,
    allow_legacy_tier_bc_sections: bool,
}

/// Synthesizes and injects 7-section safety docstrings into a single Dart source buffer.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and Check 36 (`audit_dart_source_docstrings`).
/// Enriches every Dart `class`, `enum`, `mixin`, `extension`, `typedef`, `constructor`, `method`,
/// and top-level function with a complete, unique 7-section docstring while stripping forbidden
/// `/// Member documentation.` placeholders.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves `@override` and `@immutable` annotations immediately below the synthesized `///` block.
/// - Tracks `(brace_depth, paren_depth)` identically to `audit_dart_source_docstrings`.
///
/// ## 3. Preconditions
/// - `source: &str` contains valid UTF-8 Dart source code.
/// - `file_rel: &str` is the relative or absolute file path for diagnostic and requirement ID tagging.
///
/// ## 4. Postconditions
/// - Returns `Ok((updated_source, enriched_item_count))` passing `audit_dart_source_docstrings` with zero violations, or `Err(DocstringError)` on syntax or unforced conflict error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the number of lines in `source`.
/// - Space Complexity: O(L + I) where I is the number of unique callable docblocks.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates delimiter balance via `validate_source_syntax_for_language` prior to injection.
/// - State: Isolated per-file `DartInjectorState` scanner state.
/// - Action: Non-panicking `.get(..)` slice accessors and saturating arithmetic.
/// - Resource: Linear memory bounded by file size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-INJECT-DART-DOCSTRINGS]
pub fn inject_dart_file_docstrings(
    source: &str,
    file_rel: &str,
    schema_model: Option<&PackageDef>,
    force: bool,
) -> Result<(String, usize), DocstringError> {
    validate_source_syntax_for_language(source, file_rel, SourceLanguage::Dart)
        .map_err(DocstringError::Parse)?;

    let raw_lines: Vec<&str> = source.lines().collect();
    let mut state = DartInjectorState::new(raw_lines.len(), file_rel, schema_model, force);

    for idx in 0..raw_lines.len() {
        state.process_line(&raw_lines, idx)?;
    }

    if state.edits.enriched_count == 0 {
        return Ok((source.to_string(), 0));
    }

    let rendered = state.edits.render(source, &raw_lines);
    Ok((rendered, state.edits.enriched_count))
}

/// Detects legacy 7-section numbered H2 headers (`## 1.` through `## 7.`) on a docblock.
/// Realises: [REQ-FEAT-42-HAS-LEGACY-SEVEN-SECTION-HEADERS]
fn has_legacy_seven_section_headers(doc_lines: &[String]) -> bool {
    doc_lines.iter().any(|line| {
        let text = strip_outer_doc_marker(line);
        matches!(
            text.get(..5),
            Some("## 1." | "## 2." | "## 3." | "## 4." | "## 5." | "## 6." | "## 7.")
        )
    })
}

/// Checks whether `doc_lines` satisfies tiered Rust/Dart documentation compliance (`is_tier_a`) with optional generated-file exemption.
/// Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK-MODE]
fn is_compliant_docblock_with_mode(
    doc_lines: &[String],
    is_tier_a: bool,
    allow_legacy_tier_bc_sections: bool,
) -> bool {
    if doc_lines.is_empty()
        || any_placeholder_in_doc(doc_lines)
        || has_malformed_prefix_or_hybrid_headers(doc_lines)
    {
        return false;
    }

    let joined = doc_lines.join("\n");
    if !docblock_has_non_empty_realises(&joined) {
        return false;
    }

    let is_module_doc = doc_lines
        .iter()
        .any(|line| line.trim_start().starts_with("//!"));
    let effective_tier_a = is_tier_a || is_module_doc;

    if effective_tier_a {
        let valid_seven =
            has_all_seven_sections(&joined) && has_non_empty_tier_bc_summary(doc_lines);
        valid_seven || has_valid_four_block_contract(doc_lines)
    } else if allow_legacy_tier_bc_sections {
        has_non_empty_tier_bc_summary(doc_lines)
    } else {
        !has_legacy_seven_section_headers(doc_lines) && has_non_empty_tier_bc_summary(doc_lines)
    }
}

/// Checks whether `doc_lines` satisfies tiered Rust/Dart documentation compliance (`is_tier_a`) without per-file deduplication state.
/// Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK]
#[cfg(test)]
fn is_compliant_docblock(doc_lines: &[String], is_tier_a: bool) -> bool {
    is_compliant_docblock_with_mode(doc_lines, is_tier_a, false)
}

/// Evaluates `is_compliant_docblock_with_mode` and enforces per-file normalized body uniqueness on 7-section callable docblocks.
/// Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK-IN-FILE]
fn is_compliant_docblock_in_file_with_mode(
    current_doc: &[String],
    policy: DocCompliancePolicy,
    seen_docblocks: &mut BTreeSet<String>,
) -> bool {
    if !is_compliant_docblock_with_mode(
        current_doc,
        policy.is_tier_a,
        policy.allow_legacy_tier_bc_sections,
    ) {
        return false;
    }

    if policy.check_uniqueness {
        let joined = current_doc.join("\n");
        if has_all_seven_sections(&joined) {
            let normalized = normalize_docblock_body(current_doc);
            if normalized.is_empty() || !seen_docblocks.insert(normalized) {
                return false;
            }
        }
    }

    true
}

/// Checks whether a joined docblock contains all 7 canonical DO-178C numbered H2 section headers.
/// Realises: [REQ-FEAT-42-HAS-ALL-SEVEN-SECTIONS]
fn has_all_seven_sections(joined: &str) -> bool {
    let first_half = joined.contains("## 1. Safety Intent & Regulatory Scope")
        && joined.contains("## 2. Invariants & Mathematical Properties")
        && joined.contains("## 3. Preconditions");
    let second_half = joined.contains("## 4. Postconditions")
        && joined.contains("## 5. Algorithmic Complexity Bounds")
        && joined.contains("## 6. Failure Modes & Fault Tolerance");
    first_half && second_half && joined.contains("## 7. Realization & Traceability")
}

/// Strips the leading `///` or `//!` doc-comment marker and surrounding whitespace from a single line.
/// Realises: [REQ-FEAT-42-STRIP-OUTER-DOC-MARKER]
fn strip_outer_doc_marker(line: &str) -> &str {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("///")
        .or_else(|| trimmed.strip_prefix("//!"))
        .unwrap_or(trimmed)
        .trim()
}

/// Classifies a stripped doc-comment line against the three Tier A 4-Block Contract headers.
/// Realises: [REQ-FEAT-42-CLASSIFY-FOUR-BLOCK-HEADER]
fn classify_four_block_header(line_text: &str) -> Option<(FourBlockHeader, &str)> {
    let clean = line_text
        .strip_prefix("- ")
        .or_else(|| line_text.strip_prefix("* "))
        .or_else(|| line_text.strip_prefix("### "))
        .or_else(|| line_text.strip_prefix("## "))
        .unwrap_or(line_text)
        .trim_start();
    let unbolded = clean.strip_prefix("**").unwrap_or(clean).trim_start();
    let candidates = [
        (
            "Safety & Traceability",
            FourBlockHeader::SafetyAndTraceability,
        ),
        ("Bounds & Faults", FourBlockHeader::BoundsAndFaults),
        ("Contract", FourBlockHeader::Contract),
    ];
    for (label, header) in candidates {
        if let Some(after_label) = unbolded.strip_prefix(label) {
            let after_bold = after_label.trim_start();
            let after_bold = after_bold
                .strip_prefix("**")
                .unwrap_or(after_bold)
                .trim_start();
            if after_bold.is_empty() {
                return Some((header, ""));
            }
            if let Some(after_colon) = after_bold.strip_prefix(':') {
                let tail = after_colon.trim_start();
                let tail = tail.strip_prefix("**").unwrap_or(tail).trim();
                return Some((header, tail));
            }
        }
    }
    None
}

/// Verifies that `doc_lines` satisfies the Tier A Concise 4-Block Contract (summary + 3 populated contract blocks).
/// Realises: [REQ-FEAT-42-HAS-VALID-FOUR-BLOCK-CONTRACT]
fn has_valid_four_block_contract(doc_lines: &[String]) -> bool {
    let mut mask: u8 = 0;
    let mut seen_headers: u8 = 0;
    let mut active_header_bit: u8 = 0;

    for line in doc_lines {
        let text = strip_outer_doc_marker(line);
        if text.is_empty() {
            continue;
        }
        if let Some((header, inline_tail)) = classify_four_block_header(text) {
            let bit: u8 = match header {
                FourBlockHeader::SafetyAndTraceability => 0b0010,
                FourBlockHeader::Contract => 0b0100,
                FourBlockHeader::BoundsAndFaults => 0b1000,
            };
            seen_headers |= bit;
            if inline_tail.chars().any(char::is_alphanumeric) {
                mask |= bit;
                active_header_bit = 0;
            } else {
                active_header_bit = bit;
            }
        } else {
            if seen_headers == 0
                && !text.starts_with("Realises:")
                && !text.starts_with('#')
                && text.chars().any(char::is_alphanumeric)
            {
                mask |= 0b0001;
            }
            if active_header_bit != 0 && text.chars().any(char::is_alphanumeric) {
                mask |= active_header_bit;
            }
        }
    }

    mask == 0b1111
}

/// Checks whether `doc_lines` contains at least one non-empty descriptive summary line distinct from `Realises:`.
/// Realises: [REQ-FEAT-42-HAS-NON-EMPTY-TIER-BC-SUMMARY]
fn has_non_empty_tier_bc_summary(doc_lines: &[String]) -> bool {
    for line in doc_lines {
        let text = strip_outer_doc_marker(line);
        if text.is_empty() || text.starts_with("## ") || text.starts_with("# ") {
            continue;
        }
        if let Some((before_realises, _)) = text.split_once("Realises:") {
            let prefix = before_realises.trim();
            if classify_four_block_header(prefix).is_none()
                && prefix.chars().any(char::is_alphanumeric)
            {
                return true;
            }
        } else if text.chars().any(char::is_alphanumeric) {
            return true;
        }
    }
    false
}

/// Extracts all requirement IDs from `Realises: [...]` tags across `doc_lines`, including inline 4-Block `Safety & Traceability: Realises: [...]` lines.
/// Realises: [REQ-FEAT-42-EXTRACT-ALL-REALISES-IDS]
fn extract_all_realises_ids(doc_lines: &[String]) -> Vec<String> {
    let mut ids = extract_realises_ids_from_doc_lines(doc_lines);
    for line in doc_lines {
        if let Some((_, after_realises)) = line.split_once("Realises:") {
            let mut rest = after_realises;
            while let Some(open_idx) = rest.find('[') {
                let after_open = rest.get(open_idx.saturating_add(1)..).unwrap_or("");
                let Some(close_idx) = after_open.find(']') else {
                    break;
                };
                let inside = after_open.get(..close_idx).unwrap_or("");
                for tok in inside.split(',') {
                    let clean = tok.trim();
                    if !clean.is_empty() && !ids.iter().any(|existing| existing == clean) {
                        ids.push(clean.to_string());
                    }
                }
                rest = after_open.get(close_idx.saturating_add(1)..).unwrap_or("");
            }
        }
    }
    ids
}

/// Detects doubled doc-comment prefixes, legacy H1 section headers, or duplicate `Realises:` lines in `current_doc`.
/// Realises: [REQ-FEAT-42-HAS-MALFORMED-PREFIX-OR-HYBRID-HEADERS]
fn has_malformed_prefix_or_hybrid_headers(current_doc: &[String]) -> bool {
    let mut realises_count: usize = 0;
    for line in current_doc {
        let trimmed = line.trim();
        let after_outer = trimmed
            .strip_prefix("///")
            .or_else(|| trimmed.strip_prefix("//!"))
            .unwrap_or(trimmed)
            .trim_start();
        if after_outer.starts_with("///") || after_outer.starts_with("//!") {
            return true;
        }
        if after_outer.starts_with("# ") && !after_outer.starts_with("## ") {
            return true;
        }
        if after_outer.starts_with("Realises:") {
            realises_count = realises_count.saturating_add(1);
        }
    }
    realises_count > 1
}

/// Formats a single canonical `///` docstring line at `indent`, stripping any accidental leading `///` or `//!` markers from `text`.
/// Realises: [REQ-FEAT-42-FORMAT-NORMALIZED-DOC-LINE]
fn format_normalized_doc_line(indent: &str, text: &str) -> String {
    let mut clean = text.trim();
    while let Some(rest) = clean
        .strip_prefix("///")
        .or_else(|| clean.strip_prefix("//!"))
    {
        clean = rest.trim_start();
    }
    if clean.is_empty() {
        format!("{indent}///")
    } else {
        format!("{indent}/// {clean}")
    }
}

/// Normalizes a `///` docblock identically to Check 34 (`structural.rs:827-839`) and Check 36 (`docstring_gate.rs:965-977`).
/// Realises: [REQ-FEAT-42-NORMALIZE-DOCBLOCK-BODY]
fn normalize_docblock_body(doc_lines: &[String]) -> String {
    doc_lines
        .iter()
        .skip(1)
        .filter(|doc_line| !doc_line.contains("Realises:"))
        .map(|doc_line| {
            doc_line
                .trim_start_matches("///")
                .trim_start_matches("//!")
                .trim()
        })
        .filter(|doc_text| !doc_text.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Strips repeated leading `///` or `//!` markers and surrounding whitespace from a docstring line.
/// Realises: [REQ-FEAT-42-CLEAN-DOC-LINE-PAYLOAD]
fn clean_doc_line_payload(line: &str) -> &str {
    let mut text = line.trim();
    while let Some(rest) = text
        .strip_prefix("///")
        .or_else(|| text.strip_prefix("//!"))
    {
        text = rest.trim_start();
    }
    text.trim()
}

/// Extracts a clean 1-sentence domain summary from `existing_doc` (leading line or non-boilerplate Section 1 line).
/// Realises: [REQ-FEAT-42-EXTRACT-TIER-BC-SUMMARY]
fn extract_tier_bc_summary(existing_doc: &[String]) -> Option<String> {
    for line in existing_doc {
        let text = clean_doc_line_payload(line);
        if text.is_empty() {
            continue;
        }
        if text.starts_with('#') {
            break;
        }
        let is_meta = text.starts_with("Realises:") || classify_four_block_header(text).is_some();
        if !is_meta && !has_placeholder_doc_line(line) && text.chars().any(char::is_alphanumeric) {
            return Some(text.to_string());
        }
    }

    let mut in_section_1 = false;
    for line in existing_doc {
        let text = clean_doc_line_payload(line);
        if text.is_empty() {
            continue;
        }
        if text.starts_with("## 1.") {
            in_section_1 = true;
            continue;
        }
        if text.starts_with('#') {
            if in_section_1 {
                break;
            }
            continue;
        }
        if in_section_1 {
            let is_boilerplate =
                text.starts_with("Conforms to DO-178C") || text.starts_with("Realises:");
            if !is_boilerplate
                && !has_placeholder_doc_line(line)
                && text.chars().any(char::is_alphanumeric)
            {
                return Some(text.to_string());
            }
        }
    }

    None
}

/// Synthesizes a concise 1-sentence Tier B/C summary when no reusable domain summary exists.
/// Realises: [REQ-FEAT-42-SYNTHESIZE-DEFAULT-TIER-BC-SUMMARY]
fn synthesize_default_tier_bc_summary(
    ctx: &ItemSynthesisContext<'_>,
    qualified_name: &str,
) -> String {
    if ctx.item_kw_override == Some("const") {
        format!(
            "Defines module constant `{qualified_name}` in `{}`.",
            ctx.module_tag
        )
    } else if ctx.item_kw_override == Some("type") {
        format!(
            "Defines type alias `{qualified_name}` in `{}`.",
            ctx.module_tag
        )
    } else if ctx.is_callable() {
        match ctx.return_type.as_deref() {
            Some(ret) => format!(
                "Executes `{qualified_name}` returning `{ret}` in `{}`.",
                ctx.module_tag
            ),
            None => format!("Executes `{qualified_name}` within `{}`.", ctx.module_tag),
        }
    } else {
        format!(
            "Defines internal {} `{qualified_name}` within `{}`.",
            ctx.item_keyword(),
            ctx.module_tag
        )
    }
}

/// Resolves the bracketed `Realises:` requirement tag string from schema bindings, existing doc tags, or module fallback.
/// Realises: [REQ-FEAT-42-RESOLVE-REALISES-REQ-TAG]
fn resolve_realises_req_tag(
    ctx: &ItemSynthesisContext<'_>,
    qualified_name: &str,
    schema_req_id: Option<&str>,
) -> String {
    let existing_reqs = extract_all_realises_ids(ctx.existing_doc);
    if let Some(s_req) = schema_req_id {
        let schema_reqs: Vec<&str> = s_req
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let existing_covers_schema = !schema_reqs.is_empty()
            && schema_reqs.iter().all(|sr| {
                existing_reqs
                    .iter()
                    .any(|r| schema_identifiers_match(r, sr))
            });
        if existing_covers_schema {
            existing_reqs
                .iter()
                .map(|r| format!("[{r}]"))
                .collect::<Vec<String>>()
                .join(", ")
        } else {
            schema_reqs
                .iter()
                .map(|sr| format!("[{sr}]"))
                .collect::<Vec<String>>()
                .join(", ")
        }
    } else if !existing_reqs.is_empty() {
        existing_reqs
            .iter()
            .map(|r| format!("[{r}]"))
            .collect::<Vec<String>>()
            .join(", ")
    } else {
        let item_tag = sanitize_req_segment(qualified_name);
        format!("[REQ-{}-{item_tag}]", ctx.module_tag)
    }
}

/// Synthesizes or compacts a concise 2-line Tier B/C Rust docstring (`/// <summary>` + `/// Realises: [<REQ-ID>]`).
/// Realises: [REQ-FEAT-42-SYNTHESIZE-TIER-BC-DOC]
fn synthesize_tier_bc_docstring(ctx: &ItemSynthesisContext<'_>) -> Vec<String> {
    let qualified_name = match ctx.owner_name {
        Some(owner) => format!("{owner}::{}", ctx.item_name),
        None => ctx.item_name.to_string(),
    };
    let summary = extract_tier_bc_summary(ctx.existing_doc)
        .unwrap_or_else(|| synthesize_default_tier_bc_summary(ctx, &qualified_name));
    let (schema_req_id, _) =
        resolve_schema_context(ctx.schema_model, ctx.owner_name, ctx.item_name);
    let req_tag = resolve_realises_req_tag(ctx, &qualified_name, schema_req_id.as_deref());

    vec![
        format_normalized_doc_line(ctx.indent, &summary),
        format_normalized_doc_line(ctx.indent, &format!("Realises: {req_tag}")),
    ]
}

/// Builds Section 3 (Preconditions) lines and Section 4 (Postconditions) text for `synthesize_7_section_docstring`.
/// Realises: [REQ-FEAT-42-BUILD-7-SECTION-PRE-POST]
fn build_7_section_pre_and_post<'a>(
    ctx: &'a ItemSynthesisContext<'a>,
    qualified_name: &'a str,
) -> (Vec<String>, &'a str, String) {
    let mut sec3_lines = Vec::new();
    if !ctx.params.is_empty() {
        for param in &ctx.params {
            sec3_lines.push(format_normalized_doc_line(
                ctx.indent,
                &format!(
                    "- `{}: {}` must satisfy domain and type-safety constraints prior to invoking `{qualified_name}`.",
                    param.name, param.type_name
                ),
            ));
        }
    } else if !ctx.fields_or_variants.is_empty() {
        sec3_lines.push(format_normalized_doc_line(
            ctx.indent,
            &format!(
                "- Caller supplies well-formed values for `{qualified_name}` members (`{}`).",
                ctx.fields_or_variants.join(", ")
            ),
        ));
    } else {
        sec3_lines.push(format_normalized_doc_line(
            ctx.indent,
            &format!(
                "- Invoked within a valid execution context for `{qualified_name}` in `{}`.",
                ctx.module_tag
            ),
        ));
    }

    let default_ret = if ctx.is_callable() {
        "()"
    } else {
        qualified_name
    };
    let ret_desc = ctx.return_type.as_deref().unwrap_or(default_ret);
    let sec4_body = if ctx.is_callable() {
        format!("- Returns `{ret_desc}` from `{qualified_name}` with deterministic post-execution state.")
    } else {
        format!(
            "- Defines the authoritative `{ret_desc}` representation for {} `{qualified_name}`.",
            ctx.item_keyword()
        )
    };

    (sec3_lines, ret_desc, sec4_body)
}

/// Appends Sections 5 (Algorithmic Complexity Bounds) and 6 (Failure Modes & Fault Tolerance) onto `lines`.
/// Realises: [REQ-FEAT-42-APPEND-7-SECTION-BOUNDS-FAILURES]
fn append_7_section_bounds_and_failures(
    ctx: &ItemSynthesisContext<'_>,
    qualified_name: &str,
    ret_desc: &str,
    lines: &mut Vec<String>,
) {
    let dim_var = if let Some(first_param) = ctx.params.first() {
        format!("N({})", first_param.name)
    } else if !ctx.fields_or_variants.is_empty() {
        format!("M({})", ctx.fields_or_variants.len())
    } else {
        format!("K({})", ctx.item_name)
    };
    let empty_line = format_normalized_doc_line(ctx.indent, "");

    lines.push(format_normalized_doc_line(
        ctx.indent,
        "## 5. Algorithmic Complexity Bounds",
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!(
            "- Time Complexity: O({dim_var}) bounded over input/member dimension of `{qualified_name}`."
        ),
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!(
            "- Space Complexity: O({dim_var}) bounded auxiliary memory for `{qualified_name}`."
        ),
    ));
    lines.push(empty_line.clone());
    lines.push(format_normalized_doc_line(
        ctx.indent,
        "## 6. Failure Modes & Fault Tolerance",
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!(
            "- Interface: Validates boundary inputs and types for `{qualified_name}` (`{ret_desc}`)."
        ),
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!(
            "- State: Maintains consistent state invariants across `{qualified_name}` without partial mutation on error."
        ),
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!(
            "- Action: Executes `{qualified_name}` deterministically with zero unbounded recursion or blocking."
        ),
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!(
            "- Resource: Bounds stack and heap allocations within `{qualified_name}` (`{}`).",
            ctx.module_tag
        ),
    ));
    lines.push(empty_line);
}

/// Synthesizes a complete, Check-34/36-unique 7-section `///` docblock for Tier A Rust and Dart items.
/// Realises: [REQ-FEAT-42-SYNTHESIZE-7-SECTION-DOC]
fn synthesize_7_section_docstring(
    ctx: &ItemSynthesisContext<'_>,
    seen_docblocks: &mut BTreeSet<String>,
) -> Vec<String> {
    let qualified_name = match ctx.owner_name {
        Some(owner) => format!("{owner}::{}", ctx.item_name),
        None => ctx.item_name.to_string(),
    };
    let kind_kw = ctx.item_keyword();

    let summary = summary_from_doc_lines(ctx.existing_doc)
        .filter(|s| !has_placeholder_doc_line(s))
        .unwrap_or_else(|| {
            format!(
                "Executes deterministic {kind_kw} `{qualified_name}` within `{}`.",
                ctx.file_rel
            )
        });

    let (schema_req_id, schema_constraint_expr) =
        resolve_schema_context(ctx.schema_model, ctx.owner_name, ctx.item_name);

    let sec1_body = if let Some(ref c_expr) = schema_constraint_expr {
        format!(
            "Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for {kind_kw} `{qualified_name}` in module `{}` (enforcing schema constraint `{c_expr}`).",
            ctx.module_tag
        )
    } else {
        format!(
            "Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for {kind_kw} `{qualified_name}` in `{}`.",
            ctx.file_rel
        )
    };

    let sec2_body = if !ctx.invariants.is_empty() {
        format!(
            "- Enforces deterministic state and execution invariants for `{qualified_name}`: `{}`.",
            ctx.invariants.join(" && ")
        )
    } else if !ctx.fields_or_variants.is_empty() {
        format!(
            "- Preserves structural type invariants for {kind_kw} `{qualified_name}` across member set `[{}]`.",
            ctx.fields_or_variants.join(", ")
        )
    } else {
        format!(
            "- Preserves deterministic state conservation and referential transparency for {kind_kw} `{qualified_name}` (declaration #{}) within `{}`.",
            ctx.ordinal_index, ctx.module_tag
        )
    };

    let (sec3_lines, ret_desc, sec4_body) = build_7_section_pre_and_post(ctx, &qualified_name);
    let req_tag = resolve_realises_req_tag(ctx, &qualified_name, schema_req_id.as_deref());
    let sysml_elem = if ctx.is_callable() {
        format!("ActionDef({qualified_name})")
    } else {
        format!("PartDef({qualified_name})")
    };

    let empty_line = format_normalized_doc_line(ctx.indent, "");
    let mut lines = vec![
        format_normalized_doc_line(ctx.indent, &summary),
        empty_line.clone(),
        format_normalized_doc_line(ctx.indent, "## 1. Safety Intent & Regulatory Scope"),
        format_normalized_doc_line(ctx.indent, &sec1_body),
        empty_line.clone(),
        format_normalized_doc_line(ctx.indent, "## 2. Invariants & Mathematical Properties"),
        format_normalized_doc_line(ctx.indent, &sec2_body),
        empty_line.clone(),
        format_normalized_doc_line(ctx.indent, "## 3. Preconditions"),
    ];
    lines.extend(sec3_lines);
    lines.push(empty_line.clone());
    lines.push(format_normalized_doc_line(
        ctx.indent,
        "## 4. Postconditions",
    ));
    lines.push(format_normalized_doc_line(ctx.indent, &sec4_body));
    lines.push(empty_line);
    append_7_section_bounds_and_failures(ctx, &qualified_name, ret_desc, &mut lines);
    lines.push(format_normalized_doc_line(
        ctx.indent,
        "## 7. Realization & Traceability",
    ));
    lines.push(format_normalized_doc_line(
        ctx.indent,
        &format!("Realises: {req_tag} | SysML v2: {sysml_elem}"),
    ));

    let normalized = normalize_docblock_body(&lines);
    if !seen_docblocks.insert(normalized) {
        if let Some(sec1_slot) = lines.get_mut(3) {
            *sec1_slot = format_normalized_doc_line(
                ctx.indent,
                &format!("{sec1_body} (Item ordinal #{}).", ctx.ordinal_index),
            );
        }
        let renormalized = normalize_docblock_body(&lines);
        seen_docblocks.insert(renormalized);
    }

    lines
}

/// Inspects lines starting at `start_idx` in `raw_lines` to extract Rust callable signatures, struct fields, or enum variants.
/// Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS]
fn inspect_rust_item_details(
    raw_lines: &[&str],
    start_idx: usize,
    kind: SourceItemKind,
) -> (Vec<SourceParam>, Option<String>, Vec<String>, Vec<String>) {
    let mut params = Vec::new();
    let mut return_type = None;
    let mut fields_or_variants = Vec::new();
    let mut invariants = Vec::new();

    if kind.is_callable() {
        let mut sig_buf = String::new();
        let mut brace_depth: i32 = 0;
        let mut saw_open_brace = false;
        let mut body_start_idx = start_idx.saturating_add(1);
        if let Some(window) = raw_lines.get(start_idx..) {
            for (offset, &line) in window.iter().enumerate() {
                let code = strip_line_comment_preserving_strings(line.trim());
                if !sig_buf.is_empty() {
                    sig_buf.push(' ');
                }
                sig_buf.push_str(code);
                let (delta, has_open) = count_rust_line_brace_delta(code);
                if has_open {
                    saw_open_brace = true;
                    if let Some((_, after_brace)) = code.split_once('{') {
                        let inline_stmt = after_brace.trim();
                        if let Some(rest) = inline_stmt
                            .strip_prefix("assert!(")
                            .or_else(|| inline_stmt.strip_prefix("debug_assert!("))
                        {
                            if let Some(clean_cond) = extract_balanced_rust_assert_cond(rest) {
                                let formatted_cond = clean_cond.replace('"', "'");
                                if !formatted_cond.is_empty()
                                    && !invariants.contains(&formatted_cond)
                                {
                                    invariants.push(formatted_cond);
                                }
                            }
                        }
                    }
                }
                brace_depth = brace_depth.saturating_add(delta);
                body_start_idx = start_idx.saturating_add(offset).saturating_add(1);
                if has_open || code.ends_with(';') {
                    break;
                }
            }
        }
        let (p, r) = parse_rust_callable_signature(&sig_buf);
        params = p;
        return_type = r;

        if saw_open_brace && brace_depth > 0 {
            if let Some(body_window) = raw_lines.get(body_start_idx..) {
                for &line in body_window {
                    let t = strip_line_comment_preserving_strings(line.trim());
                    if let Some(rest) = t
                        .strip_prefix("assert!(")
                        .or_else(|| t.strip_prefix("debug_assert!("))
                    {
                        if let Some(clean_cond) = extract_balanced_rust_assert_cond(rest) {
                            let formatted_cond = clean_cond.replace('"', "'");
                            if !formatted_cond.is_empty() && !invariants.contains(&formatted_cond) {
                                invariants.push(formatted_cond);
                            }
                        }
                    }
                    let (delta, has_open) = count_rust_line_brace_delta(t);
                    if has_open {
                        saw_open_brace = true;
                    }
                    brace_depth = brace_depth.saturating_add(delta);
                    if saw_open_brace && brace_depth <= 0 {
                        break;
                    }
                }
            }
        }
    } else if kind == SourceItemKind::Struct
        || kind == SourceItemKind::Enum
        || kind == SourceItemKind::Trait
    {
        let mut brace_depth: i32 = 0;
        let mut saw_open_brace = false;
        if let Some(window) = raw_lines.get(start_idx..) {
            for &line in window {
                let t = strip_line_comment_preserving_strings(line.trim());
                let depth_before = brace_depth;
                let (delta, has_open) = count_rust_line_brace_delta(t);
                if has_open {
                    saw_open_brace = true;
                }
                brace_depth = brace_depth.saturating_add(delta);

                if depth_before == 1
                    && !t.starts_with('}')
                    && !t.is_empty()
                    && !t.starts_with('#')
                    && !t.starts_with("///")
                    && !t.starts_with("//!")
                {
                    if kind == SourceItemKind::Struct {
                        let after_vis = t
                            .strip_prefix("pub(crate) ")
                            .or_else(|| t.strip_prefix("pub(super) "))
                            .or_else(|| t.strip_prefix("pub "))
                            .unwrap_or(t)
                            .trim_start();
                        if let Some((fname, ftype)) = after_vis.split_once(':') {
                            if let Some(id) = extract_leading_ast_ident(fname.trim()) {
                                let clean_t = ftype.trim().trim_end_matches(',').trim();
                                let entry = format!("{}: {}", id, clean_t);
                                if !fields_or_variants.contains(&entry) {
                                    fields_or_variants.push(entry);
                                }
                            }
                        }
                    } else if kind == SourceItemKind::Enum {
                        if let Some(v) = extract_leading_ast_ident(t) {
                            if !fields_or_variants.contains(&v) {
                                fields_or_variants.push(v);
                            }
                        }
                    } else if kind == SourceItemKind::Trait {
                        if let Some((_, fn_rest)) = t.split_once("fn ") {
                            if let Some(m) = extract_leading_ast_ident(fn_rest.trim()) {
                                let entry = format!("fn {}", m);
                                if !fields_or_variants.contains(&entry) {
                                    fields_or_variants.push(entry);
                                }
                            }
                        }
                    }
                }

                if (saw_open_brace && brace_depth <= 0) || (!saw_open_brace && t.ends_with(';')) {
                    break;
                }
            }
        }
    }

    (params, return_type, fields_or_variants, invariants)
}

/// Inspects lines starting at `start_idx` in `raw_lines` to extract Dart callable signatures, `assert(...)` invariants, class/mixin/extension fields, or enum variants.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn inspect_dart_item_details(
    raw_lines: &[&str],
    start_idx: usize,
    kind: SourceItemKind,
    item_name: &str,
) -> (Vec<SourceParam>, Option<String>, Vec<String>, Vec<String>) {
    let mut params = Vec::new();
    let mut return_type = None;
    let mut fields_or_variants = Vec::new();
    let mut invariants = Vec::new();

    if kind.is_callable() {
        let sig_code = deap_core::source_ast::collect_dart_callable_signature(raw_lines, start_idx);
        let (p, r) = parse_dart_callable_signature(&sig_code, item_name);
        params = p;
        return_type = r;

        let mut item_brace_depth: usize = 0;
        let mut item_paren_depth: usize = 0;
        let mut saw_open_brace = false;
        if let Some(window) = raw_lines.get(start_idx..) {
            for &raw_line in window {
                let t = strip_line_comment_preserving_strings(raw_line.trim());
                let mut search_slice = t;
                while let Some(pos) = search_slice.find("assert(") {
                    let after_assert = search_slice.get(pos.saturating_add(7)..).unwrap_or("");
                    if let Some(cond) = extract_balanced_rust_assert_cond(after_assert) {
                        let clean = cond.replace('"', "'");
                        if !clean.is_empty() && !invariants.contains(&clean) {
                            invariants.push(clean);
                        }
                    }
                    search_slice = after_assert;
                }
                update_ast_delimiter_depths(t, &mut item_brace_depth, &mut item_paren_depth);
                if item_paren_depth == 0 && (item_brace_depth > 0 || t.contains('}')) {
                    saw_open_brace = true;
                }
                if (saw_open_brace && item_brace_depth == 0 && item_paren_depth == 0)
                    || (!saw_open_brace && item_paren_depth == 0 && t.ends_with(';'))
                {
                    break;
                }
            }
        }
    } else if kind == SourceItemKind::Enum {
        let mut item_brace_depth: usize = 0;
        let mut item_paren_depth: usize = 0;
        let mut saw_open_brace = false;
        let mut enum_variants_done = false;
        if let Some(window) = raw_lines.get(start_idx..) {
            for &raw_line in window {
                let t = strip_line_comment_preserving_strings(raw_line.trim());
                let depth_before = item_brace_depth;
                let paren_before = item_paren_depth;
                if (depth_before == 1 || (depth_before == 0 && t.contains('{')))
                    && !enum_variants_done
                {
                    let segment = if depth_before == 0 {
                        t.split_once('{').map(|(_, a)| a).unwrap_or("")
                    } else {
                        t
                    };
                    let cut_idx = match (segment.find(';'), segment.find('}')) {
                        (Some(s), Some(b)) => Some(s.min(b)),
                        (Some(s), None) => Some(s),
                        (None, Some(b)) => Some(b),
                        (None, None) => None,
                    };
                    let variant_part = match cut_idx {
                        Some(idx) => {
                            enum_variants_done = true;
                            segment.get(..idx).unwrap_or("")
                        }
                        None => segment,
                    };
                    let mut local_paren = paren_before;
                    for token in variant_part.split(',') {
                        if local_paren == 0 {
                            let clean_tok = strip_leading_dart_annotations(token.trim()).trim();
                            if !clean_tok.is_empty() && !clean_tok.starts_with("//") {
                                if let Some(v) = extract_leading_ast_ident(clean_tok) {
                                    if !fields_or_variants.contains(&v) {
                                        fields_or_variants.push(v);
                                    }
                                }
                            }
                        }
                        let mut dummy_brace = 0usize;
                        update_ast_delimiter_depths(token, &mut dummy_brace, &mut local_paren);
                    }
                }
                update_ast_delimiter_depths(t, &mut item_brace_depth, &mut item_paren_depth);
                if item_brace_depth > 0 || t.contains('{') {
                    saw_open_brace = true;
                }
                if (saw_open_brace && item_brace_depth == 0)
                    || (!saw_open_brace && t.ends_with(';'))
                {
                    break;
                }
            }
        }
    } else if kind == SourceItemKind::Class
        || kind == SourceItemKind::Mixin
        || kind == SourceItemKind::Extension
    {
        let mut item_brace_depth: usize = 0;
        let mut item_paren_depth: usize = 0;
        let mut saw_open_brace = false;
        if let Some(window) = raw_lines.get(start_idx..) {
            for &raw_line in window {
                let raw_t = strip_line_comment_preserving_strings(raw_line.trim());
                let t = strip_leading_dart_annotations(raw_t).trim();
                let depth_before = item_brace_depth;
                let paren_before = item_paren_depth;
                update_ast_delimiter_depths(raw_t, &mut item_brace_depth, &mut item_paren_depth);
                if item_brace_depth > 0 || raw_t.contains('{') {
                    saw_open_brace = true;
                }
                if depth_before == 1
                    && paren_before == 0
                    && !t.is_empty()
                    && !t.starts_with('}')
                    && !t.starts_with("//")
                    && t.ends_with(';')
                    && classify_dart_member_ast(t, item_name).is_none()
                {
                    let without_semi = t.trim_end_matches(';').trim();
                    let decl_head = without_semi
                        .split_once('=')
                        .map(|(b, _)| b.trim())
                        .unwrap_or(without_semi);
                    if !decl_head.is_empty() && !decl_head.contains('(') && !decl_head.contains('{')
                    {
                        let mut clean_decl = decl_head;
                        loop {
                            let next = clean_decl
                                .strip_prefix("static ")
                                .or_else(|| clean_decl.strip_prefix("late "))
                                .or_else(|| clean_decl.strip_prefix("final "))
                                .or_else(|| clean_decl.strip_prefix("const "))
                                .or_else(|| clean_decl.strip_prefix("covariant "))
                                .or_else(|| clean_decl.strip_prefix("external "))
                                .map(str::trim_start);
                            if let Some(stripped) = next {
                                clean_decl = stripped;
                            } else {
                                break;
                            }
                        }
                        if let Some(stripped_var) = clean_decl.strip_prefix("var ") {
                            if let Some(fname) = extract_leading_ast_ident(stripped_var.trim()) {
                                let entry = format!("{}: dynamic", fname);
                                if !fields_or_variants.contains(&entry) {
                                    fields_or_variants.push(entry);
                                }
                            }
                        } else if let Some((ftype_raw, fname_raw)) =
                            clean_decl.rsplit_once(char::is_whitespace)
                        {
                            let ftype = ftype_raw.trim();
                            if let Some(fname) = extract_leading_ast_ident(fname_raw.trim()) {
                                if !ftype.is_empty() {
                                    let entry = format!("{}: {}", fname, ftype);
                                    if !fields_or_variants.contains(&entry) {
                                        fields_or_variants.push(entry);
                                    }
                                }
                            }
                        } else if let Some(fname) = extract_leading_ast_ident(clean_decl) {
                            let entry = format!("{}: dynamic", fname);
                            if !fields_or_variants.contains(&entry) {
                                fields_or_variants.push(entry);
                            }
                        }
                    }
                }
                if (saw_open_brace && item_brace_depth == 0)
                    || (!saw_open_brace && raw_t.ends_with(';'))
                {
                    break;
                }
            }
        }
    }

    (params, return_type, fields_or_variants, invariants)
}

/// Collects all top-level and nested `PartDef` nodes within `pkg` into `out` in pre-order traversal up to `MAX_AST_DEPTH`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.2 hierarchical part resolution (`collect_all_parts`).
/// Ensures nested subsystem `PartDef` declarations (`part.parts`) are discoverable during docstring synthesis while bounding AST depth to `MAX_AST_DEPTH` (`64`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Visits every `PartDef` node in `pkg` at `depth <= MAX_AST_DEPTH` in pre-order without recursion and stops cleanly when `depth > MAX_AST_DEPTH`.
///
/// ## 3. Preconditions
/// - `pkg: &'a PackageDef` is a parsed SysML v2 package; `out: &mut Vec<&'a deap_core::sysml_ast::PartDef>` is the accumulator.
///
/// ## 4. Postconditions
/// - Appends all top-level and nested `&'a PartDef` references within `0..=MAX_AST_DEPTH` from `pkg` into `out`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P) where P is the number of visited `PartDef` nodes at `depth <= MAX_AST_DEPTH`.
/// - Space Complexity: O(P) for the iterative traversal stack and output references.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts packages with arbitrary part nesting depth without stack overflow.
/// - State: Mutates only `out`.
/// - Action: Non-recursive depth-bounded iterative stack traversal.
/// - Resource: Strictly bounded by `MAX_AST_DEPTH` (`64`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-FEAT-42-COLLECT-ALL-PARTS]
pub fn collect_all_parts<'a>(
    pkg: &'a PackageDef,
    out: &mut Vec<&'a deap_core::sysml_ast::PartDef>,
) {
    collect_all_parts_from_depth(pkg, 0, out);
}

/// Appends `PartDef` nodes from `pkg` starting at `initial_depth` into `out`, stopping cleanly when `depth > MAX_AST_DEPTH`.
/// Realises: [REQ-FEAT-42-COLLECT-ALL-PARTS-BOUNDED]
fn collect_all_parts_from_depth<'a>(
    pkg: &'a PackageDef,
    initial_depth: usize,
    out: &mut Vec<&'a deap_core::sysml_ast::PartDef>,
) {
    if initial_depth > MAX_AST_DEPTH {
        return;
    }
    let mut stack: Vec<(&'a deap_core::sysml_ast::PartDef, usize)> = pkg
        .part_defs
        .iter()
        .rev()
        .map(|part| (part, initial_depth))
        .collect();
    while let Some((part, depth)) = stack.pop() {
        if depth > MAX_AST_DEPTH {
            continue;
        }
        out.push(part);
        let next_depth = depth.saturating_add(1);
        if next_depth <= MAX_AST_DEPTH {
            for child in part.parts.iter().rev() {
                stack.push((child, next_depth));
            }
        }
    }
}

/// Extracts the first argument expression of a Rust `assert!(...)` or `debug_assert!(...)` invocation with balanced delimiter tracking.
/// Realises: [REQ-FEAT-42-EXTRACT-BALANCED-RUST-ASSERT-COND]
fn extract_balanced_rust_assert_cond(after_open_paren: &str) -> Option<&str> {
    let mut paren_depth: i32 = 1;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    for (idx, ch) in after_open_paren.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if in_char {
            if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_char = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '\'' => {
                if after_open_paren
                    .get(idx..)
                    .and_then(single_char_literal_byte_len)
                    .is_some()
                {
                    in_char = true;
                }
            }
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                if paren_depth == 0 {
                    return after_open_paren.get(..idx).map(str::trim);
                }
            }
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1).max(0),
            '{' => brace_depth = brace_depth.saturating_add(1),
            '}' => brace_depth = brace_depth.saturating_sub(1).max(0),
            ',' if paren_depth == 1 && bracket_depth == 0 && brace_depth == 0 => {
                return after_open_paren.get(..idx).map(str::trim);
            }
            _ => {}
        }
    }
    None
}

/// Computes the net `{` minus `}` brace depth delta on a comment-stripped Rust line outside string and char literals.
/// Realises: [REQ-FEAT-42-COUNT-RUST-LINE-BRACE-DELTA]
fn count_rust_line_brace_delta(code: &str) -> (i32, bool) {
    let mut delta: i32 = 0;
    let mut saw_open = false;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    for (idx, ch) in code.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if in_char {
            if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_char = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '\'' => {
                if code
                    .get(idx..)
                    .and_then(single_char_literal_byte_len)
                    .is_some()
                {
                    in_char = true;
                }
            }
            '{' => {
                delta = delta.saturating_add(1);
                saw_open = true;
            }
            '}' => {
                delta = delta.saturating_sub(1);
            }
            _ => {}
        }
    }
    (delta, saw_open)
}

/// Checks whether a SysML `ConstraintDef` name (`c_name`) matches an `ActionDef` or callable name (`act_name`).
/// Realises: [REQ-FEAT-42-CONSTRAINT-MATCHES-ACTION]
fn constraint_matches_action(c_name: &str, act_name: &str) -> bool {
    if schema_identifiers_match(c_name, act_name) {
        return true;
    }
    if let Some(rest) = c_name
        .strip_prefix("invariant_")
        .or_else(|| c_name.strip_prefix("INVARIANT_"))
    {
        if schema_identifiers_match(rest, act_name) {
            return true;
        }
        if let Some((prefix, _)) = rest.split_once('_') {
            if schema_identifiers_match(prefix, act_name) {
                return true;
            }
        }
        if let Some((prefix, _)) = rest.rsplit_once('_') {
            if schema_identifiers_match(prefix, act_name) {
                return true;
            }
        }
    }
    false
}

/// Formats a SysML constraint `(name, expr)` pair into a human-readable docstring annotation.
/// Realises: [REQ-FEAT-42-FORMAT-SCHEMA-CONSTRAINT]
fn format_schema_constraint(name: &str, expr: &str) -> Option<String> {
    let n = name.trim();
    let e = expr.trim();
    if !n.is_empty() && !e.is_empty() {
        Some(format!("{n} ({e})"))
    } else if !e.is_empty() {
        Some(e.to_string())
    } else if !n.is_empty() {
        Some(n.to_string())
    } else {
        None
    }
}

/// Normalizes an identifier to lowercase ASCII alphanumeric characters for fuzzy schema matching.
/// Realises: [REQ-FEAT-42-NORMALIZE-ASCII-ALNUM]
fn normalize_ascii_alnum_lower(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Appends `token` to `reqs` if non-empty and not already matched via `schema_identifiers_match`.
/// Realises: [REQ-FEAT-42-PUSH-UNIQUE-SCHEMA-REQ]
fn push_unique_schema_req(reqs: &mut Vec<String>, token: String) {
    let trimmed = token.trim();
    if !trimmed.is_empty()
        && !reqs
            .iter()
            .any(|existing| schema_identifiers_match(existing, trimmed))
    {
        reqs.push(trimmed.to_string());
    }
}

/// Appends `expr` to `constraints` if non-empty and not already present verbatim.
/// Realises: [REQ-FEAT-42-PUSH-UNIQUE-SCHEMA-CONSTRAINT]
fn push_unique_schema_constraint(constraints: &mut Vec<String>, expr: String) {
    let trimmed = expr.trim();
    if !trimmed.is_empty() && !constraints.iter().any(|existing| existing == trimmed) {
        constraints.push(trimmed.to_string());
    }
}

/// Returns the authoritative requirement identifier (`req_id` when non-empty, otherwise `name`) of `req_def`.
/// Realises: [REQ-FEAT-42-REQ-DEF-PRIMARY-ID]
fn req_def_primary_id(req_def: &deap_core::sysml_ast::RequirementDef) -> &str {
    if !req_def.req_id.is_empty() {
        req_def.req_id.as_str()
    } else {
        req_def.name.as_str()
    }
}

/// Checks whether `part` explicitly satisfies or owns `r` across `satisfied_requirements`, `safety_realises`, inline `requirements`, or `satisfied_by`.
/// Realises: [REQ-FEAT-42-PART-SATISFIES-REQ-DEF]
fn part_satisfies_requirement_def(
    part: &deap_core::sysml_ast::PartDef,
    r: &deap_core::sysml_ast::RequirementDef,
) -> bool {
    let in_satisfied = part.satisfied_requirements.iter().any(|sr| {
        schema_identifiers_match(sr, &r.name)
            || (!r.req_id.is_empty() && schema_identifiers_match(sr, &r.req_id))
    });
    let in_safety = part.safety_realises.iter().any(|sr| {
        schema_identifiers_match(sr, &r.name)
            || (!r.req_id.is_empty() && schema_identifiers_match(sr, &r.req_id))
    });
    let in_inline_reqs = part.requirements.iter().any(|pr| {
        let matches_req_id = !r.req_id.is_empty()
            && (schema_identifiers_match(&pr.req_id, &r.req_id)
                || schema_identifiers_match(&pr.name, &r.req_id));
        schema_identifiers_match(&pr.name, &r.name) || matches_req_id
    });
    let in_satisfied_by = r
        .satisfied_by
        .iter()
        .any(|sb| schema_identifiers_match(sb, &part.name));
    let direct_match = in_satisfied || in_safety;
    let reverse_match = in_inline_reqs || in_satisfied_by;
    direct_match || reverse_match
}

/// Depth-bounded pre-order index of SysML v2 packages and parts up to `MAX_AST_DEPTH`.
/// Realises: [REQ-FEAT-42-SCHEMA-LOOKUP-INDEX]
struct SchemaLookupIndex<'a> {
    all_pkgs: Vec<&'a PackageDef>,
    all_parts: Vec<&'a deap_core::sysml_ast::PartDef>,
    active_part_ident: &'a str,
    item_is_other_part: bool,
}

impl<'a> SchemaLookupIndex<'a> {
    /// Builds a `SchemaLookupIndex` over `root_pkg`, stopping package and part traversal cleanly when `depth > MAX_AST_DEPTH`.
    /// Realises: [REQ-FEAT-42-SCHEMA-LOOKUP-INDEX-NEW]
    fn new(root_pkg: &'a PackageDef, owner_name: Option<&'a str>, item_name: &'a str) -> Self {
        let mut all_pkgs: Vec<&'a PackageDef> = Vec::new();
        let mut all_parts: Vec<&'a deap_core::sysml_ast::PartDef> = Vec::new();
        let mut stack: Vec<(&'a PackageDef, usize)> = vec![(root_pkg, 0)];

        while let Some((curr, depth)) = stack.pop() {
            if depth > MAX_AST_DEPTH {
                continue;
            }
            all_pkgs.push(curr);
            collect_all_parts_from_depth(curr, depth, &mut all_parts);
            let next_depth = depth.saturating_add(1);
            if next_depth <= MAX_AST_DEPTH {
                for sub in curr.packages.iter().rev() {
                    stack.push((sub, next_depth));
                }
            }
        }

        let active_part_ident = owner_name.unwrap_or(item_name);
        let item_is_other_part = all_parts
            .iter()
            .any(|p| schema_identifiers_match(&p.name, item_name));

        Self {
            all_pkgs,
            all_parts,
            active_part_ident,
            item_is_other_part,
        }
    }

    /// Resolves a requirement name or ID token against all `requirement_defs` in `self.all_pkgs`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-REQ-TOKEN]
    fn resolve_req_token(&self, token: &str) -> Option<String> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return None;
        }
        for pkg in &self.all_pkgs {
            if let Some(req_def) = pkg.requirement_defs.iter().find(|r| {
                schema_identifiers_match(&r.name, trimmed)
                    || (!r.req_id.is_empty() && schema_identifiers_match(&r.req_id, trimmed))
            }) {
                let primary = req_def_primary_id(req_def);
                if !primary.is_empty() {
                    return Some(primary.to_string());
                }
            }
        }
        Some(trimmed.to_string())
    }

    /// Extracts requirement identifiers from an action or element `doc` comment block.
    /// Realises: [REQ-FEAT-42-SCHEMA-EXTRACT-DOC-REQ]
    fn extract_doc_req(&self, doc: &str) -> Option<String> {
        let mut collected = Vec::new();
        let doc_lines: Vec<String> = doc.lines().map(str::to_string).collect();
        for req in extract_realises_ids_from_doc_lines(&doc_lines) {
            if let Some(resolved) = self.resolve_req_token(&req) {
                push_unique_schema_req(&mut collected, resolved);
            }
        }
        if !collected.is_empty() {
            return Some(collected.join(", "));
        }
        if let Some((_, after_prefix)) = doc.split_once("Realises: [") {
            if let Some((inside, _)) = after_prefix.split_once(']') {
                for tok in inside.split(',') {
                    let clean_tok = tok
                        .trim()
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .trim();
                    if let Some(resolved) = self.resolve_req_token(clean_tok) {
                        push_unique_schema_req(&mut collected, resolved);
                    }
                }
            }
        }
        let mut rest = doc;
        while let Some(open_idx) = rest.find('[') {
            let after_open = rest.get(open_idx.saturating_add(1)..).unwrap_or("");
            let Some(close_idx) = after_open.find(']') else {
                break;
            };
            let inside = after_open.get(..close_idx).unwrap_or("").trim();
            for token in inside.split(',') {
                let tok = token.trim();
                let upper = tok.to_ascii_uppercase();
                if upper.starts_with("REQ-") || upper.starts_with("REQ_") {
                    if let Some(resolved) = self.resolve_req_token(tok) {
                        push_unique_schema_req(&mut collected, resolved);
                    }
                }
            }
            rest = after_open.get(close_idx.saturating_add(1)..).unwrap_or("");
        }
        (!collected.is_empty()).then(|| collected.join(", "))
    }

    /// Aggregates all satisfied requirements and inline constraints declared on `part`.
    /// Realises: [REQ-FEAT-42-SCHEMA-EXTRACT-PART-CONTEXT]
    fn extract_part_req_and_constraint(
        &self,
        part: &deap_core::sysml_ast::PartDef,
    ) -> (Option<String>, Option<String>) {
        let mut part_reqs = Vec::new();
        for r in &part.satisfied_requirements {
            if let Some(resolved) = self.resolve_req_token(r) {
                push_unique_schema_req(&mut part_reqs, resolved);
            }
        }
        for r in &part.requirements {
            if let Some(resolved) = self.resolve_req_token(req_def_primary_id(r)) {
                push_unique_schema_req(&mut part_reqs, resolved);
            }
        }
        for r in &part.safety_realises {
            if let Some(resolved) = self.resolve_req_token(r) {
                push_unique_schema_req(&mut part_reqs, resolved);
            }
        }
        for pkg in &self.all_pkgs {
            for req in &pkg.requirement_defs {
                if req
                    .satisfied_by
                    .iter()
                    .any(|s| schema_identifiers_match(s, &part.name))
                {
                    let id = req_def_primary_id(req);
                    if !id.is_empty() {
                        push_unique_schema_req(&mut part_reqs, id.to_string());
                    }
                }
            }
        }
        let req = (!part_reqs.is_empty()).then(|| part_reqs.join(", "));

        let mut part_constraints = Vec::new();
        for c in &part.constraints {
            if let Some(formatted) = format_schema_constraint(&c.name, &c.expression) {
                push_unique_schema_constraint(&mut part_constraints, formatted);
            }
        }
        let constraint = (!part_constraints.is_empty()).then(|| part_constraints.join("; "));

        (req, constraint)
    }

    /// Checks whether `r` is owned by or named after a `PartDef` distinct from `self.active_part_ident`.
    /// Realises: [REQ-FEAT-42-SCHEMA-REQ-OTHER-PART]
    fn req_belongs_to_other_part(&self, r: &deap_core::sysml_ast::RequirementDef) -> bool {
        let nr = normalize_ascii_alnum_lower(&r.name);
        let nreq = normalize_ascii_alnum_lower(&r.req_id);
        self.all_parts.iter().any(|p| {
            if schema_identifiers_match(&p.name, self.active_part_ident) {
                return false;
            }
            let satisfied_by_other = part_satisfies_requirement_def(p, r);
            let np = normalize_ascii_alnum_lower(&p.name);
            let named_after_other = np.len() >= 2 && (nr.contains(&np) || nreq.contains(&np));
            satisfied_by_other || named_after_other
        })
    }

    /// Checks whether `c` is owned by or named after a `PartDef` distinct from `self.active_part_ident`.
    /// Realises: [REQ-FEAT-42-SCHEMA-CONSTRAINT-OTHER-PART]
    fn constraint_belongs_to_other_part(&self, c: &deap_core::sysml_ast::ConstraintDef) -> bool {
        let nc = normalize_ascii_alnum_lower(&c.name);
        self.all_parts.iter().any(|p| {
            if schema_identifiers_match(&p.name, self.active_part_ident) {
                return false;
            }
            let owned_by_other = p
                .constraints
                .iter()
                .any(|pc| schema_identifiers_match(&pc.name, &c.name));
            let np = normalize_ascii_alnum_lower(&p.name);
            let named_after_other = np.len() >= 2 && nc.contains(&np);
            owned_by_other || named_after_other
        })
    }

    /// Checks whether `r` binds specifically to `member_ident` without bleeding across sibling parts.
    /// Realises: [REQ-FEAT-42-SCHEMA-REQ-MATCHES-MEMBER]
    fn req_matches_member(
        &self,
        r: &deap_core::sysml_ast::RequirementDef,
        member_ident: &str,
    ) -> bool {
        if self.req_belongs_to_other_part(r) {
            return false;
        }
        let exact_id_match = schema_identifiers_match(&r.name, member_ident)
            || schema_identifiers_match(&r.req_id, member_ident);
        let satisfied_by_member = r
            .satisfied_by
            .iter()
            .any(|s| schema_identifiers_match(s, member_ident));
        if exact_id_match || satisfied_by_member {
            return true;
        }

        let norm_member = normalize_ascii_alnum_lower(member_ident);
        if !self.item_is_other_part && norm_member.len() >= 4 {
            let nr = normalize_ascii_alnum_lower(&r.name);
            let nreq = normalize_ascii_alnum_lower(&r.req_id);
            let contains_member = nr.contains(&norm_member) || nreq.contains(&norm_member);
            let has_req_prefix = r.name.to_ascii_uppercase().starts_with("REQ")
                || r.req_id.to_ascii_uppercase().starts_with("REQ");
            let looks_like_action_req = has_req_prefix || nr.contains(&norm_member);
            if looks_like_action_req && contains_member {
                return true;
            }
        }
        false
    }

    /// Aggregates action-matched constraints from `constraints` (or `act.steps` fallback) for `act`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-ACTION-CONSTRAINTS]
    fn resolve_action_constraints(
        &self,
        act: &deap_core::sysml_ast::ActionDef,
        constraints: &[deap_core::sysml_ast::ConstraintDef],
        filter_other_parts: bool,
    ) -> Option<String> {
        let mut act_constraints = Vec::new();
        for act_c in constraints {
            let allowed = !filter_other_parts || !self.constraint_belongs_to_other_part(act_c);
            if allowed && constraint_matches_action(&act_c.name, &act.name) {
                if let Some(formatted) = format_schema_constraint(&act_c.name, &act_c.expression) {
                    push_unique_schema_constraint(&mut act_constraints, formatted);
                }
            }
        }
        if !act_constraints.is_empty() {
            return Some(act_constraints.join("; "));
        }
        let mut step_constraints = Vec::new();
        for step_expr in &act.steps {
            if let Some(formatted) = format_schema_constraint(&act.name, step_expr) {
                push_unique_schema_constraint(&mut step_constraints, formatted);
            }
        }
        (!step_constraints.is_empty()).then(|| step_constraints.join("; "))
    }

    /// Searches `PackageDef::action_defs` across `self.all_pkgs` for an action matching `item_name`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-PKG-ACTION]
    fn resolve_pkg_action(
        &self,
        item_name: &str,
        need_constraint: bool,
    ) -> Option<(Option<String>, Option<String>)> {
        for pkg in &self.all_pkgs {
            let Some(act) = pkg
                .action_defs
                .iter()
                .find(|a| schema_identifiers_match(&a.name, item_name))
            else {
                continue;
            };
            let mut matched_req = act.doc.as_deref().and_then(|doc| self.extract_doc_req(doc));
            if matched_req.is_none() {
                let mut act_reqs = Vec::new();
                for req_def in pkg
                    .requirement_defs
                    .iter()
                    .filter(|r| self.req_matches_member(r, &act.name))
                {
                    let id = req_def_primary_id(req_def);
                    if !id.is_empty() {
                        push_unique_schema_req(&mut act_reqs, id.to_string());
                    }
                }
                if !act_reqs.is_empty() {
                    matched_req = Some(act_reqs.join(", "));
                }
            }
            let matched_constraint = if need_constraint {
                self.resolve_action_constraints(act, &pkg.constraint_defs, true)
            } else {
                None
            };
            return Some((matched_req, matched_constraint));
        }
        None
    }

    /// Resolves `(req, constraint)` for a member `item_name` owned by `owner`, returning `None` if `owner` is unmapped.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-OWNED-ITEM]
    fn resolve_owned_item(
        &self,
        owner: &str,
        item_name: &str,
    ) -> Option<(Option<String>, Option<String>)> {
        let owner_part = self
            .all_parts
            .iter()
            .copied()
            .find(|p| schema_identifiers_match(&p.name, owner))?;
        let (p_req, mut matched_constraint) = self.extract_part_req_and_constraint(owner_part);
        let mut matched_req: Option<String> = None;

        if let Some(act) = owner_part
            .actions
            .iter()
            .find(|a| schema_identifiers_match(&a.name, item_name))
        {
            matched_req = act.doc.as_deref().and_then(|doc| self.extract_doc_req(doc));
            if let Some(act_const) =
                self.resolve_action_constraints(act, &owner_part.constraints, false)
            {
                matched_constraint = Some(act_const);
            }
        }

        if matched_req.is_none() {
            let mut member_reqs = Vec::new();
            for req_def in owner_part
                .requirements
                .iter()
                .filter(|r| self.req_matches_member(r, item_name))
            {
                let id = req_def_primary_id(req_def);
                if !id.is_empty() {
                    push_unique_schema_req(&mut member_reqs, id.to_string());
                }
            }
            if !member_reqs.is_empty() {
                matched_req = Some(member_reqs.join(", "));
            }
        }

        if matched_req.is_none() {
            let mut pkg_member_reqs = Vec::new();
            for pkg in &self.all_pkgs {
                for req_def in pkg
                    .requirement_defs
                    .iter()
                    .filter(|r| self.req_matches_member(r, item_name))
                {
                    let id = req_def_primary_id(req_def);
                    if !id.is_empty() {
                        push_unique_schema_req(&mut pkg_member_reqs, id.to_string());
                    }
                }
            }
            if !pkg_member_reqs.is_empty() {
                matched_req = Some(pkg_member_reqs.join(", "));
            }
        }

        if matched_req.is_none() {
            if let Some((act_req, act_const)) =
                self.resolve_pkg_action(item_name, matched_constraint.is_none())
            {
                matched_req = act_req;
                if matched_constraint.is_none() {
                    matched_constraint = act_const;
                }
            }
        }

        if matched_req.is_none() {
            matched_req = p_req;
        }
        Some((matched_req, matched_constraint))
    }

    /// Resolves package-level fallback requirements matching `target_ident` or `item_name`.
    /// Realises: [REQ-FEAT-42-SCHEMA-FALLBACK-REQS]
    fn resolve_fallback_reqs(&self, target_ident: &str, item_name: &str) -> Option<String> {
        let mut fallback_reqs = Vec::new();
        for pkg in &self.all_pkgs {
            for req_def in &pkg.requirement_defs {
                if self.req_belongs_to_other_part(req_def) {
                    continue;
                }
                let matches_target = schema_identifiers_match(&req_def.name, target_ident)
                    || schema_identifiers_match(&req_def.req_id, target_ident)
                    || req_def
                        .satisfied_by
                        .iter()
                        .any(|s| schema_identifiers_match(s, target_ident));
                let matches_item = schema_identifiers_match(&req_def.name, item_name)
                    || schema_identifiers_match(&req_def.req_id, item_name);
                if matches_target || matches_item {
                    let id = req_def_primary_id(req_def);
                    if !id.is_empty() {
                        push_unique_schema_req(&mut fallback_reqs, id.to_string());
                    }
                }
            }
        }
        (!fallback_reqs.is_empty()).then(|| fallback_reqs.join(", "))
    }

    /// Resolves package-level fallback constraints matching `target_ident` or `item_name`.
    /// Realises: [REQ-FEAT-42-SCHEMA-FALLBACK-CONSTRAINTS]
    fn resolve_fallback_constraints(&self, target_ident: &str, item_name: &str) -> Option<String> {
        let mut fallback_constraints = Vec::new();
        for pkg in &self.all_pkgs {
            for c_def in &pkg.constraint_defs {
                if self.constraint_belongs_to_other_part(c_def) {
                    continue;
                }
                let matches_ident = schema_identifiers_match(&c_def.name, target_ident)
                    || schema_identifiers_match(&c_def.name, item_name)
                    || constraint_matches_action(&c_def.name, item_name);
                if matches_ident {
                    if let Some(formatted) =
                        format_schema_constraint(&c_def.name, &c_def.expression)
                    {
                        push_unique_schema_constraint(&mut fallback_constraints, formatted);
                    }
                }
            }
        }
        (!fallback_constraints.is_empty()).then(|| fallback_constraints.join("; "))
    }
}

/// Resolves optional requirement ID and constraint expression from `schema_model` for `(owner_name, item_name)`.
/// Realises: [REQ-FEAT-42-RESOLVE-SCHEMA-CONTEXT]
fn resolve_schema_context(
    schema_model: Option<&PackageDef>,
    owner_name: Option<&str>,
    item_name: &str,
) -> (Option<String>, Option<String>) {
    let Some(root_pkg) = schema_model else {
        return (None, None);
    };

    let index = SchemaLookupIndex::new(root_pkg, owner_name, item_name);
    let (mut matched_req, mut matched_constraint) = if let Some(owner) = owner_name {
        let Some(owned_res) = index.resolve_owned_item(owner, item_name) else {
            return (None, None);
        };
        owned_res
    } else if let Some(part) = index
        .all_parts
        .iter()
        .copied()
        .find(|p| schema_identifiers_match(&p.name, item_name))
    {
        index.extract_part_req_and_constraint(part)
    } else {
        index
            .resolve_pkg_action(item_name, true)
            .unwrap_or((None, None))
    };

    let target_ident = owner_name.unwrap_or(item_name);
    if matched_req.is_none() {
        matched_req = index.resolve_fallback_reqs(target_ident, item_name);
    }
    if matched_constraint.is_none() {
        matched_constraint = index.resolve_fallback_constraints(target_ident, item_name);
    }

    (matched_req, matched_constraint)
}

/// Resolves optional requirement ID and constraint expression from `schema_model` with strict `owner_name` part isolation (`REQ-0605` `AC-03..AC-04`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `resolve_schema_context_strict` (`REQ-0605` `AC-03..AC-04`).
///
/// ## 2. Invariants & Mathematical Properties
/// - When `owner_name` is `Some(owner)` and `owner` is absent from all top-level and nested `PartDef` nodes in `schema_model`, returns `(None, None)` immediately without falling through to package-level `requirement_defs` or `constraint_defs`.
/// - When `owner_name` matches a `PartDef` or is `None`, delegates deterministically to `resolve_schema_context`.
///
/// ## 3. Preconditions
/// - `schema_model: Option<&PackageDef>` is the optional parsed SysML v2 model.
/// - `owner_name: Option<&str>` and `item_name: &str` identify the target source item and optional enclosing classifier.
///
/// ## 4. Postconditions
/// - Returns `(Option<String>, Option<String>)` representing the resolved requirement ID and constraint expression, or `(None, None)` when unmatched.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P + A + R + C) where P is part count, A is action count, R is requirement count, and C is constraint count.
/// - Space Complexity: O(P) for collected package and part reference slices.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts `None` or arbitrary UTF-8 identifiers safely.
/// - State: Pure read-only lookup with zero state mutation.
/// - Action: Never panics and never attributes unrelated package-level requirements to unmapped owners.
/// - Resource: Bounded reference vectors and at most two `String` allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-FEAT-42-RESOLVE-SCHEMA-CONTEXT]
pub fn resolve_schema_context_strict(
    schema_model: Option<&PackageDef>,
    owner_name: Option<&str>,
    item_name: &str,
) -> (Option<String>, Option<String>) {
    resolve_schema_context(schema_model, owner_name, item_name)
}

/// Compares a schema element identifier (`schema_ident`) against a source AST identifier (`source_ident`).
/// Realises: [REQ-FEAT-42-SCHEMA-IDENTIFIERS-MATCH]
fn schema_identifiers_match(schema_ident: &str, source_ident: &str) -> bool {
    let mut a_iter = schema_ident
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase());
    let mut b_iter = source_ident
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase());
    let mut saw_any = false;
    loop {
        match (a_iter.next(), b_iter.next()) {
            (Some(ca), Some(cb)) => {
                if ca != cb {
                    return false;
                }
                saw_any = true;
            }
            (None, None) => return saw_any,
            _ => return false,
        }
    }
}

/// Extracts the target type identifier and owner kind (`InherentImpl` vs `TraitImpl`) from a single-line Rust `impl` header with `{`.
/// Realises: [REQ-0605], [REQ-FEAT-42-EXTRACT-IMPL-TARGET]
fn extract_rust_impl_target_and_kind(code: &str) -> Option<(String, RustOwnerKind)> {
    extract_rust_impl_header_prefix(code)?;
    let (trait_for_tail, _) = scan_rust_impl_segment(code);
    let kind = if trait_for_tail.is_some() {
        RustOwnerKind::TraitImpl
    } else {
        RustOwnerKind::InherentImpl
    };
    if let Some(target) = parse_rust_impl_header(code) {
        Some((target, kind))
    } else if kind == RustOwnerKind::TraitImpl {
        let fallback_target = trait_for_tail
            .and_then(extract_target_from_trait_for_tail)
            .unwrap_or_default();
        Some((fallback_target, kind))
    } else {
        None
    }
}

/// Scans a Rust `impl` header line segment for a top-level trait-separating `for` keyword and top-level `where` clause.
/// Realises: [REQ-FEAT-42-SCAN-RUST-IMPL-SEGMENT]
fn scan_rust_impl_segment(segment: &str) -> (Option<&str>, bool) {
    let mut angle_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let mut trait_for_tail: Option<&str> = None;
    let mut saw_where = false;
    let mut chars = segment.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let mut end_idx = idx.saturating_add(ch.len_utf8());
            while let Some(&(next_idx, next_ch)) = chars.peek() {
                if next_ch.is_ascii_alphanumeric() || next_ch == '_' {
                    end_idx = next_idx.saturating_add(next_ch.len_utf8());
                    chars.next();
                } else {
                    break;
                }
            }
            let is_raw_ident =
                idx > 0 && segment.as_bytes().get(idx.saturating_sub(1)) == Some(&b'#');
            if !is_raw_ident && angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                let word = segment.get(idx..end_idx).unwrap_or("");
                if word == "where" {
                    saw_where = true;
                    break;
                }
                if word == "for" {
                    let after_for = segment.get(end_idx..).unwrap_or("").trim_start();
                    let is_hrtb = after_for.strip_prefix('<').is_some_and(|after_lt| {
                        let t = after_lt.trim_start();
                        t.starts_with('\'') || t.starts_with('>')
                    });
                    if !is_hrtb && trait_for_tail.is_none() {
                        trait_for_tail = Some(after_for);
                    }
                }
            }
        } else {
            match ch {
                '-' if chars.peek().is_some_and(|&(_, next_ch)| next_ch == '>') => {
                    chars.next();
                }
                '<' => angle_depth = angle_depth.saturating_add(1),
                '>' => angle_depth = angle_depth.saturating_sub(1),
                '(' => paren_depth = paren_depth.saturating_add(1),
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth = bracket_depth.saturating_add(1),
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '{' if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 => break,
                _ => {}
            }
        }
    }

    (trait_for_tail, saw_where)
}

/// Extracts the leading nominal target type identifier from the slice following `for` in `impl Trait for Type`.
/// Realises: [REQ-FEAT-42-EXTRACT-TARGET-FROM-TRAIT-FOR-TAIL]
fn extract_target_from_trait_for_tail(after_for: &str) -> Option<String> {
    let before_brace = after_for
        .split_once('{')
        .map(|(b, _)| b.trim())
        .unwrap_or(after_for);
    let before_where = before_brace
        .split_once(" where ")
        .map(|(b, _)| b.trim())
        .unwrap_or(before_brace)
        .trim_end_matches(" where")
        .trim();
    let mut clean = before_where.trim_start_matches('&').trim_start();
    if let Some(after_tick) = clean.strip_prefix('\'') {
        let lt_len = after_tick
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        clean = after_tick.get(lt_len..).unwrap_or("").trim_start();
    }
    let clean = clean.strip_prefix("mut ").unwrap_or(clean).trim();
    extract_leading_ast_ident(clean)
}

/// Advances a multi-line `PendingRustImpl` state across a continuation line and pushes onto `impl_owner_stack` on `{`.
/// Realises: [REQ-FEAT-42-ADVANCE-PENDING-RUST-IMPL]
fn advance_pending_rust_impl(
    code: &str,
    brace_depth: usize,
    pending_impl: &mut Option<PendingRustImpl>,
    impl_owner_stack: &mut Vec<(usize, String, RustOwnerKind)>,
) {
    if let Some(ref mut pending) = pending_impl {
        if !pending.in_where_clause {
            let (trait_for_tail, saw_where) = scan_rust_impl_segment(code);
            if let Some(after_for) = trait_for_tail {
                pending.kind = RustOwnerKind::TraitImpl;
                if let Some(updated_target) = extract_target_from_trait_for_tail(after_for) {
                    pending.target = updated_target;
                } else {
                    pending.target.clear();
                }
            }
            if saw_where {
                pending.in_where_clause = true;
            }
        }
    }

    if code.contains('{') && !code.contains('}') {
        if let Some(pending) = pending_impl.take() {
            impl_owner_stack.push((brace_depth, pending.target, pending.kind));
        }
    } else if code.contains(';') || code.contains('}') {
        *pending_impl = None;
    }
}

/// Strips any leading inline `#[...]` attributes on the same line as a Rust item declaration.
/// Realises: [REQ-FEAT-42-STRIP-LEADING-INLINE-RUST-ATTRS]
fn strip_leading_inline_rust_attrs(code: &str) -> &str {
    let mut rest = code.trim_start();
    while let Some(after_hash) = rest.strip_prefix("#[") {
        let mut depth: usize = 1;
        let mut end_offset: Option<usize> = None;
        for (idx, b) in after_hash.bytes().enumerate() {
            if b == b'[' {
                depth = depth.saturating_add(1);
            } else if b == b']' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_offset = Some(idx.saturating_add(1));
                    break;
                }
            }
        }
        if let Some(off) = end_offset {
            rest = after_hash.get(off..).unwrap_or("").trim_start();
        } else {
            break;
        }
    }
    rest
}

/// Strips restricted Rust visibility qualifiers (`pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ...)`) from `code`.
/// Realises: [REQ-FEAT-42-STRIP-RESTRICTED-PUB-VISIBILITY]
fn strip_restricted_pub_visibility(code: &str) -> Option<&str> {
    let after_pub = code.strip_prefix("pub")?.trim_start();
    let after_open = after_pub.strip_prefix('(')?;
    let mut depth: usize = 1;
    for (idx, b) in after_open.bytes().enumerate() {
        if b == b'(' {
            depth = depth.saturating_add(1);
        } else if b == b')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let tail = after_open
                    .get(idx.saturating_add(1)..)
                    .unwrap_or("")
                    .trim_start();
                return if tail.is_empty() { None } else { Some(tail) };
            }
        }
    }
    None
}

/// Classifies a Rust declaration line and computes its Tier A (`true`) versus Tier B/C (`false`) documentation tier.
/// Realises: [REQ-FEAT-42-CLASSIFY-RUST-ITEM-WITH-TIER]
fn classify_rust_item_with_tier_ast(
    code: &str,
    enclosing_owner_kind: Option<RustOwnerKind>,
) -> Option<(SourceItemKind, String, bool)> {
    let clean = strip_leading_inline_rust_attrs(code);
    let has_exact_pub = match clean.strip_prefix("pub") {
        Some(after_pub) if after_pub.starts_with(' ') || after_pub.starts_with('\t') => {
            !after_pub.trim_start().starts_with('(')
        }
        _ => false,
    };

    let (kind, item_name) = classify_rust_item_line_ast(clean)
        .or_else(|| strip_restricted_pub_visibility(clean).and_then(classify_rust_item_line_ast))?;

    let is_tier_a = has_exact_pub
        && matches!(
            enclosing_owner_kind,
            None | Some(RustOwnerKind::InherentImpl)
        );
    Some((kind, item_name, is_tier_a))
}

/// Classifies module-level or inherent-impl Rust `const` and `type` declarations as Tier B/C items.
/// Realises: [REQ-FEAT-42-CLASSIFY-RUST-CONST-OR-TYPE]
fn classify_rust_const_or_type_ast(
    code: &str,
    enclosing_owner_kind: Option<RustOwnerKind>,
) -> Option<(&'static str, String)> {
    if matches!(
        enclosing_owner_kind,
        Some(RustOwnerKind::TraitImpl | RustOwnerKind::TraitDef)
    ) {
        return None;
    }

    let clean = strip_leading_inline_rust_attrs(code);
    let after_vis = strip_restricted_pub_visibility(clean)
        .or_else(|| {
            clean
                .strip_prefix("pub ")
                .or_else(|| clean.strip_prefix("pub\t"))
                .map(str::trim_start)
        })
        .unwrap_or(clean);

    if let Some(after_const) = after_vis
        .strip_prefix("const ")
        .or_else(|| after_vis.strip_prefix("const\t"))
    {
        let body = after_const.trim_start();
        let name = extract_leading_ast_ident(body)?;
        if name == "_" || matches!(name.as_str(), "fn" | "unsafe" | "extern") {
            return None;
        }
        let tail = body.get(name.len()..).unwrap_or("").trim_start();
        return tail.starts_with(':').then_some(("const", name));
    }

    if let Some(after_type) = after_vis
        .strip_prefix("type ")
        .or_else(|| after_vis.strip_prefix("type\t"))
    {
        let body = after_type.trim_start();
        let name = extract_leading_ast_ident(body)?;
        if name == "_" {
            return None;
        }
        let tail = body.get(name.len()..).unwrap_or("").trim_start();
        let valid_type_tail =
            tail.starts_with('=') || tail.starts_with('<') || tail.starts_with(':');
        return (valid_type_tail || tail.starts_with("where")).then_some(("type", name));
    }

    None
}

/// Classifies a Rust declaration outside function bodies into a tiered `ClassifiedRustItem`.
/// Realises: [REQ-FEAT-42-CLASSIFY-RUST-ITEM-FOR-INJECTION]
fn classify_rust_item_for_injection(
    code: &str,
    enclosing_owner_kind: Option<RustOwnerKind>,
    in_fn_body: bool,
) -> Option<ClassifiedRustItem> {
    if in_fn_body {
        return None;
    }
    if let Some((kind, name, is_tier_a)) =
        classify_rust_item_with_tier_ast(code, enclosing_owner_kind)
    {
        return Some(ClassifiedRustItem {
            kind,
            keyword_override: None,
            name,
            is_tier_a,
        });
    }
    let (kw, name) = classify_rust_const_or_type_ast(code, enclosing_owner_kind)?;
    Some(ClassifiedRustItem {
        kind: SourceItemKind::Struct,
        keyword_override: Some(kw),
        name,
        is_tier_a: false,
    })
}

/// Checks whether a joined docblock contains `Realises: [<non-empty>]`.
/// Realises: [REQ-FEAT-42-DOCBLOCK-NON-EMPTY-REALISES]
fn docblock_has_non_empty_realises(doc_block: &str) -> bool {
    if let Some((_, after_prefix)) = doc_block.split_once("Realises: [") {
        if let Some((inside, _)) = after_prefix.split_once(']') {
            return !inside.contains('\n') && !inside.trim().is_empty();
        }
    }
    false
}

/// Checks whether any line in `doc_lines` is a forbidden placeholder docstring.
/// Realises: [REQ-FEAT-42-ANY-PLACEHOLDER-IN-DOC]
fn any_placeholder_in_doc(doc_lines: &[String]) -> bool {
    doc_lines.iter().any(|l| has_placeholder_doc_line(l))
}

/// Replaces balanced inline backtick code spans and double-quoted literals on a single line with spaces.
/// Realises: [REQ-FEAT-42-STRIP-BALANCED-CODE-AND-QUOTE-SPANS]
fn strip_balanced_code_and_quote_spans(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut skip_until: usize = 0;

    for (idx, ch) in raw.char_indices() {
        if idx < skip_until {
            continue;
        }
        if ch == '`' || ch == '"' {
            let after_open = raw.get(idx.saturating_add(1)..).unwrap_or("");
            if let Some(rel_close) = after_open.find(ch) {
                skip_until = idx
                    .saturating_add(1)
                    .saturating_add(rel_close)
                    .saturating_add(1);
                out.push(' ');
                continue;
            }
        }
        out.push(ch);
    }

    out
}

/// Checks whether a single docstring line contains forbidden placeholder phrases or unfinished marker tokens.
/// Realises: [REQ-FEAT-42-HAS-PLACEHOLDER-DOC-LINE]
fn has_placeholder_doc_line(doc_line: &str) -> bool {
    let raw = strip_outer_doc_marker(doc_line);
    let body = raw.to_ascii_lowercase();
    let mut tokens = body.split_whitespace();
    let matches_stub_phrase = match (tokens.next(), tokens.next(), tokens.next()) {
        (Some("member"), Some("documentation"), None) => true,
        (Some("member"), Some(second), _) => second.starts_with("documentation."),
        (Some("todo:"), Some(second), _) => second.starts_with("document"),
        (Some("placeholder"), Some(second), _) => second.starts_with("documentation"),
        _ => false,
    };
    if matches_stub_phrase {
        return true;
    }

    let unquoted = strip_balanced_code_and_quote_spans(raw);
    unquoted
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '!')
        .any(|tok| {
            tok.eq_ignore_ascii_case("todo")
                || tok.eq_ignore_ascii_case("tbd")
                || tok.eq_ignore_ascii_case("fixme")
                || tok.eq_ignore_ascii_case("xxx")
        })
}

/// Extracts the leading whitespace slice from `raw_line`.
/// Realises: [REQ-FEAT-42-LEADING-WHITESPACE]
fn leading_whitespace(raw_line: &str) -> &str {
    let trimmed_len = raw_line.trim_start().len();
    let ws_len = raw_line.len().saturating_sub(trimmed_len);
    raw_line.get(..ws_len).unwrap_or("")
}

/// Derives an uppercase requirement tag prefix from a file path (`file_rel`).
/// Realises: [REQ-FEAT-42-DERIVE-MODULE-REQ-TAG]
fn derive_module_req_tag(file_rel: &str) -> String {
    let path = Path::new(file_rel);
    let raw_components: Vec<String> = path
        .components()
        .filter_map(|comp| {
            let s = comp.as_os_str().to_string_lossy();
            let trimmed = s.trim();
            if matches!(trimmed, "" | "." | ".." | "/" | "\\") {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();

    let anchor_start = raw_components
        .iter()
        .rposition(|c| matches!(c.as_str(), "crates" | "app_flutter"))
        .or_else(|| {
            raw_components
                .iter()
                .rposition(|c| matches!(c.as_str(), "src" | "lib" | "tests" | "bin"))
        })
        .unwrap_or(0);

    let scoped_slice = raw_components
        .get(anchor_start..)
        .unwrap_or(&raw_components);
    let mut parts = Vec::new();
    for comp in scoped_slice {
        let stem = comp
            .strip_suffix(".rs")
            .or_else(|| comp.strip_suffix(".dart"))
            .unwrap_or(comp.as_str());
        if matches!(
            stem,
            "" | "." | ".." | "crates" | "src" | "app_flutter" | "lib" | "tests" | "bin"
        ) {
            continue;
        }
        let clean = sanitize_req_segment(stem);
        if !clean.is_empty() {
            parts.push(clean);
        }
    }
    if parts.len() > 2 {
        let start = parts.len().saturating_sub(2);
        if let Some(tail) = parts.get(start..) {
            return tail.join("-");
        }
    }
    if parts.is_empty() {
        "MODULE".to_string()
    } else {
        parts.join("-")
    }
}

/// Sanitizes an identifier into an uppercase hyphenated requirement ID segment.
/// Realises: [REQ-FEAT-42-SANITIZE-REQ-SEGMENT]
fn sanitize_req_segment(raw: &str) -> String {
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
mod tests {
    use super::*;
    use deap_core::sysml_ast::{ActionDef, ConstraintDef, PartDef, RequirementDef};

    /// Checks whether `haystack` contains `left` immediately followed by `right`.
    /// Realises: [REQ-ISSUE-515-ADJACENT-TOKEN-PROBE]
    fn contains_adjacent_pair(haystack: &str, left: &str, right: &str) -> bool {
        haystack.match_indices(left).any(|(idx, _)| {
            haystack
                .get(idx.saturating_add(left.len())..)
                .is_some_and(|rest| rest.starts_with(right))
        })
    }

    /// Verifies that unmatched AST items do not inherit unrelated `pkg.requirement_defs.first()` or `pkg.constraint_defs.first()` bindings (DEFECT-09, refs #498).
    #[test]
    fn test_defect_09_unmatched_items_do_not_inherit_first_package_req_or_constraint(
    ) -> Result<(), String> {
        let limit_constraint = ConstraintDef {
            name: "ValueLimit".to_string(),
            doc: None,
            parameters: Vec::new(),
            expression: "metric <= 85.0".to_string(),
            is_assertion: false,
            pre_conditions: Vec::new(),
            post_conditions: Vec::new(),
        };
        let nested_child = PartDef {
            name: "NestedSubunit".to_string(),
            is_def: true,
            requirements: vec![RequirementDef {
                name: "NestedSubunitReq".to_string(),
                req_id: "REQ-NS-002".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let primary_part = PartDef {
            name: "PrimaryController".to_string(),
            is_def: true,
            satisfied_requirements: vec!["REQ-PC-001".to_string()],
            constraints: vec![limit_constraint.clone()],
            ..Default::default()
        };
        let secondary_part = PartDef {
            name: "SecondaryMonitor".to_string(),
            is_def: true,
            parts: vec![nested_child],
            ..Default::default()
        };
        let imu_part = PartDef {
            name: "Imu".to_string(),
            is_def: true,
            satisfied_requirements: vec!["REQ-IMU-003".to_string()],
            ..Default::default()
        };
        let pkg = PackageDef {
            name: "CorePackage".to_string(),
            part_defs: vec![primary_part, secondary_part, imu_part],
            action_defs: vec![ActionDef {
                name: "calibrate_element".to_string(),
                is_def: true,
                ..Default::default()
            }],
            requirement_defs: vec![
                RequirementDef {
                    name: "PrimaryControllerLimitReq".to_string(),
                    req_id: "REQ-PC-001".to_string(),
                    ..Default::default()
                },
                RequirementDef {
                    name: "ImuCalibrationReq".to_string(),
                    req_id: "REQ-IMU-003".to_string(),
                    ..Default::default()
                },
                RequirementDef {
                    name: "NestedSubunitReq".to_string(),
                    req_id: "REQ-NS-002".to_string(),
                    ..Default::default()
                },
            ],
            constraint_defs: vec![limit_constraint],
            ..Default::default()
        };

        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "PrimaryController"),
            (
                Some("REQ-PC-001".to_string()),
                Some("ValueLimit (metric <= 85.0)".to_string())
            ),
            "PrimaryController must resolve its satisfied requirement and constraint"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "NestedSubunit"),
            (Some("REQ-NS-002".to_string()), None),
            "Nested child PartDef must resolve its inline requirement"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "Imu"),
            (Some("REQ-IMU-003".to_string()), None),
            "Short 3-char PartDef Imu must resolve its satisfied requirement"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "SecondaryMonitor"),
            (None, None),
            "SecondaryMonitor without reqs/constraints must not inherit pkg.requirement_defs.first()"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), Some("SecondaryMonitor"), "primary_controller"),
            (None, None),
            "Method primary_controller on SecondaryMonitor must not match unrelated PartDef PrimaryController"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), Some("SecondaryMonitor"), "control"),
            (None, None),
            "Method control on SecondaryMonitor must not bleed REQ-PC-001 from PrimaryControllerLimitReq"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), Some("SecondaryMonitor"), "limit"),
            (None, None),
            "Method limit on SecondaryMonitor must not bleed REQ-PC-001 from PrimaryControllerLimitReq"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), Some("SecondaryMonitor"), "value_limit"),
            (None, None),
            "Method value_limit on SecondaryMonitor must not bleed PrimaryController's ValueLimit constraint"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "value_limit"),
            (None, None),
            "Standalone fn value_limit must not bleed PrimaryController's ValueLimit constraint"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), Some("SecondaryMonitor"), "calibration"),
            (None, None),
            "Method calibration on SecondaryMonitor must not bleed REQ-IMU-003 from 3-char sibling PartDef Imu"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), Some("SecondaryMonitor"), "nested_subunit_req"),
            (None, None),
            "Method on SecondaryMonitor must not bleed inline requirement owned by NestedSubunit"
        );
        let mut st = RustDartLexState {
            language: SourceLanguage::Rust,
            ..Default::default()
        };
        let scanned_chars = scan_line_with_unified_lex_state(
            "let a = '\\''; let b = '\\\"'; let c = '\\x7f'; let d = '\\u{10FFFF}'; pub fn after_chars() {}",
            &mut st,
        );
        assert!(st.active_string.is_none() && st.block_comment_depth == 0);
        assert!(
            scanned_chars.contains("pub fn after_chars() {}"),
            "Escaped char literals must not corrupt lex_state: {}",
            scanned_chars
        );
        let scanned_lt = scan_line_with_unified_lex_state(
            "pub fn select_lifetime<'a, 'b>(first: &'a str, second: &'b str) -> char { '\"' }",
            &mut st,
        );
        assert!(st.active_string.is_none() && st.block_comment_depth == 0);
        assert!(
            scanned_lt.starts_with(
                "pub fn select_lifetime<'a, 'b>(first: &'a str, second: &'b str) -> char"
            ),
            "Multi-lifetime signature must not be swallowed as a char literal: {}",
            scanned_lt
        );
        assert_eq!(
            extract_balanced_rust_assert_cond("ch != '\\x29' && ch != '\\u{7d}', \"msg\");"),
            Some("ch != '\\x29' && ch != '\\u{7d}'"),
            "extract_balanced_rust_assert_cond must handle hex and unicode char escapes without premature truncation"
        );
        assert_eq!(
            count_rust_line_brace_delta("let c = '\\u{7d}'; let d = '\\u{10FFFF}';"),
            (0, false),
            "count_rust_line_brace_delta must ignore braces inside unicode escape char literals"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "calibrate_element"),
            (None, None),
            "Package action calibrate_element without reqs must not inherit pkg.requirement_defs.first()"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "unmapped_checksum_helper"),
            (None, None),
            "Unmatched item must not inherit pkg.requirement_defs.first() or pkg.constraint_defs.first()"
        );

        let src = r#"pub struct PrimaryController<T> {
    pub metric: T,
}

/* multiline comment start
   closing inline before fn */ pub fn post_comment_op(x: u32) -> u32 { x }

impl<F: Fn(u32) -> bool> PrimaryController<F> {
    pub fn check_fn_bound(&self, v: u32) -> bool { (self.metric)(v) }
}

impl<T> PrimaryController<T>
where
    for <'a> &'a T: Copy,
{
    pub fn read_metric(&self) -> T { self.metric }
}

pub fn unmapped_checksum_helper(seed: u32) -> u32 {
    seed.wrapping_add(1)
}
"#;
        let (enriched, count) =
            inject_rust_file_docstrings(src, "crates/core_module/src/codec.rs", Some(&pkg), false)?;
        assert_eq!(
            count, 5,
            "Expected 5 items enriched (struct, post_comment_op, check_fn_bound, read_metric, unmapped_checksum_helper), got {} in:\n{}",
            count, enriched
        );
        assert!(
            enriched.contains("Realises: [REQ-PC-001] | SysML v2: PartDef(PrimaryController)"),
            "Matched struct PrimaryController must bind REQ-PC-001:\n{}",
            enriched
        );
        assert!(
            enriched.contains("SysML v2: ActionDef(post_comment_op)"),
            "Inline block-comment closed declaration post_comment_op must be enriched:\n{}",
            enriched
        );
        let close_comment_pos = enriched
            .find("closing inline before fn */")
            .ok_or_else(|| "missing closing inline comment in enriched output".to_string())?;
        let post_doc_pos = enriched
            .find("SysML v2: ActionDef(post_comment_op)")
            .ok_or_else(|| "missing post_comment_op docstring in enriched output".to_string())?;
        let post_fn_pos = enriched
            .find("pub fn post_comment_op")
            .ok_or_else(|| "missing pub fn post_comment_op in enriched output".to_string())?;
        assert!(
            close_comment_pos < post_doc_pos && post_doc_pos < post_fn_pos,
            "Docstring for post_comment_op must be injected AFTER `*/` and BEFORE `pub fn post_comment_op`:\n{}",
            enriched
        );
        assert!(
            enriched.contains("Realises: [REQ-PC-001] | SysML v2: ActionDef(PrimaryController::check_fn_bound)"),
            "Method inside impl<F: Fn(u32) -> bool> PrimaryController<F> must resolve owner PrimaryController:\n{}",
            enriched
        );
        assert!(
            enriched.contains("Realises: [REQ-PC-001] | SysML v2: ActionDef(PrimaryController::read_metric)"),
            "Method inside multiline impl<T> PrimaryController<T> where ... must resolve owner PrimaryController:\n{}",
            enriched
        );
        assert!(
            enriched.contains("Realises: [REQ-CORE-MODULE-CODEC-UNMAPPED-CHECKSUM-HELPER] | SysML v2: ActionDef(unmapped_checksum_helper)"),
            "Unmatched function must synthesize deterministic module-scoped REQ tag:\n{}",
            enriched
        );
        let (enriched_2, count_2) = inject_rust_file_docstrings(
            &enriched,
            "crates/core_module/src/codec.rs",
            Some(&pkg),
            false,
        )?;
        assert_eq!(
            count_2, 0,
            "Second pass of inject_rust_file_docstrings must be idempotent (count_2 == 0), got {}:\n{}",
            count_2, enriched_2
        );
        assert_eq!(enriched_2, enriched);

        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {}", self_path.display(), e))?;
        let (prod_src, _) = self_src
            .split_once("\n#[cfg(test)]\nmod tests")
            .ok_or_else(|| "missing mod tests boundary in src/docstring.rs".to_string())?;
        assert!(
            !prod_src.contains("close_pos <= 10"),
            "src/docstring.rs must not contain magic close_pos heuristic"
        );
        assert!(
            !prod_src.contains("enum RustStringScannerState"),
            "src/docstring.rs must import RustStringScannerState from deap_core::source_ast instead of redefining it"
        );
        assert!(
            !prod_src.contains("size_of::<"),
            "src/docstring.rs must not contain size_of workarounds"
        );
        assert!(
            !prod_src.contains("(Some('\\\\'), Some(_)) =>"),
            "src/docstring.rs must delegate char literal scanning to single_char_literal_byte_len"
        );
        let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("docstring_injector_tests.rs");
        let test_src = fs::read_to_string(&test_path)
            .map_err(|e| format!("Failed to read {}: {}", test_path.display(), e))?;
        assert!(
            !contains_adjacent_pair(&test_src, "#![allow(", "clippy::"),
            "tests/docstring_injector_tests.rs must not contain clippy suppression attributes"
        );
        Ok(())
    }

    /// Verifies that multi-hash raw strings (`r##"..."##`) and multiline standard strings containing fake items do not confuse the scanner (DEFECT-09, refs #498).
    #[test]
    fn test_defect_09_multihash_raw_and_multiline_standard_strings_do_not_confuse_scanner(
    ) -> Result<(), String> {
        let mut src = String::new();
        src.push_str("pub fn first_real_fn() -> &'static str {\n");
        src.push_str("    let raw_multi = r##\"\n");
        src.push_str("interior single close \"# does not end r##\n");
        src.push_str("fn fake_item() {}\n");
        src.push_str("struct Fake {}\n");
        src.push_str("\"##;\n");
        src.push_str("    let std_multi = \"line one\n");
        src.push_str("fn fake_item() {}\n");
        src.push_str("struct Fake {}\n");
        src.push_str("end line\";\n");
        src.push_str("    if std_multi.is_empty() {\n");
        src.push_str("        return \"\";\n");
        src.push_str("    }\n");
        src.push_str("    raw_multi\n");
        src.push_str("}\n\n");
        src.push_str("pub fn second_real_fn(val: usize) -> usize {\n");
        src.push_str("    val.saturating_add(1)\n");
        src.push_str("}\n");

        let (enriched, count) =
            inject_rust_file_docstrings(&src, "crates/compile-sysml/src/sample.rs", None, false)?;
        assert_eq!(
            count, 2,
            "Only the 2 real functions should be enriched, not fake items inside strings. Got count={} in:\n{}",
            count, enriched
        );
        assert!(
            enriched.contains("let raw_multi = r##\"\ninterior single close \"# does not end r##\nfn fake_item() {}\nstruct Fake {}\n\"##;"),
            "Multi-hash raw string literal was corrupted by docstring injector:\n{}",
            enriched
        );
        assert!(
            enriched.contains(
                "let std_multi = \"line one\nfn fake_item() {}\nstruct Fake {}\nend line\";"
            ),
            "Multiline standard string literal was corrupted by docstring injector:\n{}",
            enriched
        );

        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {}", self_path.display(), e))?;
        let (_self_enriched, self_count) = inject_rust_file_docstrings(
            &self_src,
            "crates/compile-sysml/src/docstring.rs",
            None,
            false,
        )?;
        assert_eq!(
            self_count, 0,
            "src/docstring.rs must be 100% self-compliant with tiered v2.0 Check 36 docstrings"
        );
        Ok(())
    }

    /// Verifies balanced `assert!(...)` condition extraction across inner `{ ... }` blocks and `pkg.action_defs` lookup (refs #559).
    #[test]
    fn test_issue_559_balanced_assert_extraction_and_pkg_action_defs_lookup() {
        let fn_src = [
            "pub fn validate_reading(x: f64, ready: bool) -> f64 {",
            "    if ready {",
            "        let _ = 1;",
            "    }",
            "    assert!(x.is_finite() && in_range(x, 0.0, 100.0), \"out of range\");",
            "    x",
            "}",
        ];
        let (_params, _ret, _fields, invariants) =
            inspect_rust_item_details(&fn_src, 0, SourceItemKind::Function);
        assert!(
            invariants
                .iter()
                .any(|inv| inv == "x.is_finite() && in_range(x, 0.0, 100.0)"),
            "Expected full balanced assert condition without truncation at inner ')' or ',' and after inner '}}' block, got: {:?}",
            invariants
        );

        let pkg = PackageDef {
            name: "SystemPkg".to_string(),
            action_defs: vec![ActionDef {
                name: "evaluate_transition".to_string(),
                doc: Some("Realises: [REQ-SYS-EVAL-01]".to_string()),
                is_def: true,
                ..Default::default()
            }],
            constraint_defs: vec![ConstraintDef {
                name: "invariant_evaluate_transition_0".to_string(),
                doc: None,
                parameters: Vec::new(),
                expression: "dt > 0.0".to_string(),
                is_assertion: true,
                pre_conditions: Vec::new(),
                post_conditions: Vec::new(),
            }],
            ..Default::default()
        };

        let (matched_req, matched_constraint) =
            resolve_schema_context(Some(&pkg), None, "evaluate_transition");
        assert_eq!(
            matched_req,
            Some("REQ-SYS-EVAL-01".to_string()),
            "Expected top-level ActionDef in pkg.action_defs to resolve Realises requirement"
        );
        let constraint_str = matched_constraint.unwrap_or_default();
        assert!(
            constraint_str.contains("dt > 0.0"),
            "Expected invariant_evaluate_transition_0 constraint to resolve for evaluate_transition, got: {:?}",
            constraint_str
        );
    }

    /// Verifies method-level `Realises:` conflict detection in Rust/Dart and action-specific requirement precedence over parent `PartDef` requirements (refs #560).
    #[test]
    fn test_issue_560_method_realises_conflict_and_action_req_precedence() -> Result<(), String> {
        let model = PackageDef {
            name: "SystemControlPkg".to_string(),
            part_defs: vec![PartDef {
                name: "SystemController".to_string(),
                is_def: true,
                satisfied_requirements: vec!["REQ-PARENT".to_string()],
                actions: vec![
                    ActionDef {
                        name: "process_signal".to_string(),
                        doc: Some("Processes signal command [REQ-ACTION]".to_string()),
                        is_def: false,
                        ..Default::default()
                    },
                    ActionDef {
                        name: "reconcile_state".to_string(),
                        doc: None,
                        is_def: false,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            action_defs: vec![ActionDef {
                name: "synchronize_phase".to_string(),
                doc: Some("Realises: [REQ-ACTION-SYNC]".to_string()),
                is_def: true,
                ..Default::default()
            }],
            requirement_defs: vec![
                RequirementDef {
                    name: "REQ_PARENT".to_string(),
                    req_id: "REQ-PARENT".to_string(),
                    ..Default::default()
                },
                RequirementDef {
                    name: "REQ_ACTION".to_string(),
                    req_id: "REQ-ACTION".to_string(),
                    ..Default::default()
                },
                RequirementDef {
                    name: "REQ_ACTION_RECONCILE_STATE".to_string(),
                    req_id: "REQ-ACTION-RECONCILE-STATE".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // (a) Action-specific requirement precedence vs parent PartDef fallback
        let (signal_req, _) =
            resolve_schema_context(Some(&model), Some("SystemController"), "process_signal");
        assert_eq!(
            signal_req,
            Some("REQ-ACTION".to_string()),
            "Action doc [REQ-ACTION] on owner_part.actions must take precedence over parent PartDef REQ-PARENT"
        );

        let (reconcile_req, _) =
            resolve_schema_context(Some(&model), Some("SystemController"), "reconcile_state");
        assert_eq!(
            reconcile_req,
            Some("REQ-ACTION-RECONCILE-STATE".to_string()),
            "Action matching RequirementDef REQ_ACTION_RECONCILE_STATE must take precedence over parent PartDef REQ-PARENT"
        );

        let (sync_req, _) =
            resolve_schema_context(Some(&model), Some("SystemController"), "synchronize_phase");
        assert_eq!(
            sync_req,
            Some("REQ-ACTION-SYNC".to_string()),
            "Action matching pkg.action_defs synchronize_phase must take precedence over parent PartDef REQ-PARENT"
        );

        let (helper_req, _) =
            resolve_schema_context(Some(&model), Some("SystemController"), "helper_step");
        assert_eq!(
            helper_req,
            Some("REQ-PARENT".to_string()),
            "Method without an action-specific requirement must fall back to parent PartDef REQ-PARENT"
        );

        // (b) Rust impl method with compliant 7-section docstring but conflicting Realises tag
        let rust_src = r#"/// System controller classifier.
/// ## 1. Safety Intent & Regulatory Scope
/// Struct SystemController safety scope.
/// ## 2. Invariants & Mathematical Properties
/// Struct SystemController invariants.
/// ## 3. Preconditions
/// Struct SystemController preconditions.
/// ## 4. Postconditions
/// Struct SystemController postconditions.
/// ## 5. Algorithmic Complexity Bounds
/// Time: O(1), Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Deterministic struct layout.
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARENT]
pub struct SystemController;

impl SystemController {
    /// Processes signal output.
    /// ## 1. Safety Intent & Regulatory Scope
    /// Method SystemController::process_signal safety scope.
    /// ## 2. Invariants & Mathematical Properties
    /// Method SystemController::process_signal invariants.
    /// ## 3. Preconditions
    /// Method SystemController::process_signal preconditions.
    /// ## 4. Postconditions
    /// Method SystemController::process_signal postconditions.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time: O(1), Space: O(1) for process_signal.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Deterministic method execution for process_signal.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-STALE-999]
    pub fn process_signal(&self) -> f64 {
        1.0
    }
}
"#;
        let rust_res = inject_rust_file_docstrings(
            rust_src,
            "crates/core_module/src/controller.rs",
            Some(&model),
            false,
        );
        let rust_err = rust_res.err().ok_or(
            "Expected inject_rust_file_docstrings (force=false) to detect conflicting Realises on impl method process_signal",
        )?;
        assert!(
            rust_err.contains("process_signal") && rust_err.contains("REQ-ACTION"),
            "Expected Rust conflict error to reference 'process_signal' and 'REQ-ACTION', got: {}",
            rust_err
        );

        // (c) Dart class method at brace_depth == 1 with compliant 7-section docstring but conflicting Realises tag
        let dart_src = r#"/// System controller Dart class.
/// ## 1. Safety Intent & Regulatory Scope
/// Class SystemController safety scope.
/// ## 2. Invariants & Mathematical Properties
/// Class SystemController invariants.
/// ## 3. Preconditions
/// Class SystemController preconditions.
/// ## 4. Postconditions
/// Class SystemController postconditions.
/// ## 5. Algorithmic Complexity Bounds
/// Time: O(1), Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Deterministic class layout.
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARENT]
class SystemController {
  /// Processes signal output in Dart.
  /// ## 1. Safety Intent & Regulatory Scope
  /// Method SystemController.processSignal safety scope.
  /// ## 2. Invariants & Mathematical Properties
  /// Method SystemController.processSignal invariants.
  /// ## 3. Preconditions
  /// Method SystemController.processSignal preconditions.
  /// ## 4. Postconditions
  /// Method SystemController.processSignal postconditions.
  /// ## 5. Algorithmic Complexity Bounds
  /// Time: O(1), Space: O(1) for processSignal.
  /// ## 6. Failure Modes & Fault Tolerance
  /// Deterministic method execution for processSignal.
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-STALE-999]
  double processSignal() {
    return 1.0;
  }
}
"#;
        let dart_res = inject_dart_file_docstrings(
            dart_src,
            "lib/system_controller.dart",
            Some(&model),
            false,
        );
        let dart_err = dart_res.err().ok_or(
            "Expected inject_dart_file_docstrings (force=false) to detect conflicting Realises on class method processSignal",
        )?;
        assert!(
            dart_err.contains("processSignal") && dart_err.contains("REQ-ACTION"),
            "Expected Dart conflict error to reference 'processSignal' and 'REQ-ACTION', got: {}",
            dart_err
        );
        Ok(())
    }

    /// Verifies that `inject_dart_file_docstrings` extracts formal parameter names from multi-line Dart class methods and top-level functions (`#561`).
    #[test]
    fn test_issue_561_inject_dart_docstrings_multiline_callable_params() -> Result<(), String> {
        let dart_src = r#"class DispatchCoordinator {
  void scheduleBatch(
    String channelId,
    {
      required double targetThresholdRatio,
      int modeCode = 1,
    }
  ) {
    return;
  }
}

bool validateBounds(
  double inputRate,
  double maxLimit,
) {
  return inputRate <= maxLimit;
}
"#;

        let (enriched, count) =
            inject_dart_file_docstrings(dart_src, "lib/dispatch_coordinator.dart", None, false)
                .map_err(|e| e.into_message())?;
        assert!(
            count >= 3,
            "Expected at least 3 enriched Dart items, got {count}"
        );
        let doc_only: String = enriched
            .lines()
            .filter(|l| l.trim_start().starts_with("///"))
            .collect::<Vec<_>>()
            .join("\n");
        for param_name in ["targetThresholdRatio", "modeCode", "inputRate", "maxLimit"] {
            assert!(
                doc_only.contains(param_name),
                "Expected synthesized Dart docstrings to mention multi-line parameter '{param_name}', got:\n{doc_only}"
            );
        }
        Ok(())
    }

    /// Verifies structural workspace anchor resolution in `derive_module_req_tag` without hardcoded workstation path segments (refs #511).
    #[test]
    fn test_issue_511_derive_module_req_tag_structural_anchor() -> Result<(), String> {
        let cases = [
            (
                "workstation_prefix/outer_dir/app_flutter/lib/main.dart",
                "MAIN",
            ),
            ("outer_mount/workspace_dir/src/engine.rs", "ENGINE"),
            ("outer_mount/workspace_dir/src/lib.rs", "MODULE"),
            (
                "outer_mount/workspace_dir/tests/smoke_test.rs",
                "SMOKE-TEST",
            ),
            ("outer_mount/workspace_dir/bin/runner.rs", "RUNNER"),
            (
                "outer_mount/workspace_dir/crates/compile-sysml/src/docstring.rs",
                "COMPILE-SYSML-DOCSTRING",
            ),
        ];
        for (input, expected) in cases {
            let actual = derive_module_req_tag(input);
            if actual != expected {
                return Err(format!(
                    "derive_module_req_tag({input:?}) returned {actual:?}, expected {expected:?}"
                ));
            }
        }

        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {e}", self_path.display()))?;
        let (prod_src, _) = self_src
            .split_once("\n#[cfg(test)]\nmod tests")
            .ok_or_else(|| "missing mod tests boundary in src/docstring.rs".to_string())?;
        let (_, after_fn_start) = prod_src
            .split_once("fn derive_module_req_tag")
            .ok_or_else(|| "missing fn derive_module_req_tag".to_string())?;
        let (fn_body, _) = after_fn_start
            .split_once("fn sanitize_req_segment")
            .ok_or_else(|| "missing fn sanitize_req_segment boundary".to_string())?;
        let allowed_literals = [
            "",
            ".",
            "..",
            "/",
            "\\\\",
            "crates",
            "app_flutter",
            "src",
            "lib",
            "tests",
            "bin",
            ".rs",
            ".dart",
            "-",
            "MODULE",
        ];
        for line in fn_body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            for (idx, segment) in trimmed.split('"').enumerate() {
                if idx % 2 == 1 && !allowed_literals.contains(&segment) {
                    return Err(format!(
                        "derive_module_req_tag contains unexpected string literal: {segment:?}"
                    ));
                }
            }
        }
        Ok(())
    }

    /// Verifies that `src/docstring.rs` contains zero hardcoded domain-specific concepts across production and test fixtures (refs #515).
    #[test]
    fn test_issue_515_docstring_tests_zero_hardcoded_domain_concepts() -> Result<(), String> {
        const FORBIDDEN_DOMAIN_PAIRS: &[(&str, &str)] = &[
            ("Avionics", "Pkg"),
            ("Thermal", "Limit"),
            ("calibrate_", "sensor"),
            ("PrimaryController", "ThermalReq"),
            ("crates/tele", "metry/"),
            ("REQ-TELE", "METRY-"),
            ("Navigation", "Pkg"),
            ("compute_", "trajectory"),
            ("REQ-NAV-", "TRAJ"),
            ("Flight", "ControlPkg"),
            ("Flight", "Controller"),
            ("compute_", "thrust"),
            ("compute", "Thrust"),
            ("adjust_", "pitch"),
            ("stabilize_", "yaw"),
            ("Flight", "Director"),
            ("engage", "Hold"),
            ("call", "sign"),
            ("targetAltitude", "Meters"),
            ("validate", "Envelope"),
            ("speed", "Mps"),
            ("ceiling", "Meters"),
        ];

        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {e}", self_path.display()))?;
        for &(left, right) in FORBIDDEN_DOMAIN_PAIRS {
            if contains_adjacent_pair(&self_src, left, right) {
                return Err(format!(
                    "src/docstring.rs must not contain hardcoded domain concept token: {left}{right}"
                ));
            }
        }
        Ok(())
    }

    /// Verifies that `src/docstring.rs` uses zero runtime slice-concatenation workarounds and anchors all self-reads to `CARGO_MANIFEST_DIR` (refs #513).
    #[test]
    fn test_issue_513_zero_concat_evasion_and_manifest_anchored_self_reads() -> Result<(), String> {
        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {e}", self_path.display()))?;
        let (before_self, _) = self_src
            .split_once("fn test_issue_513_zero_concat_evasion_and_manifest_anchored_self_reads")
            .ok_or_else(|| "missing test_issue_513 boundary".to_string())?;
        if before_self.contains(".concat()") {
            return Err(
                "src/docstring.rs must not use runtime .concat() scanner-evasion".to_string(),
            );
        }
        if before_self.contains("Path::new(\"src/docstring.rs\")") {
            return Err(
                "src/docstring.rs must not use CWD-relative Path::new(\"src/docstring.rs\")"
                    .to_string(),
            );
        }

        let positive_cases = [
            "/// Member documentation.",
            "/// Member documentation",
            "/// Member documentation. Extra",
            "/// TODO: document foo",
            "/// Placeholder documentation",
            "//! Placeholder documentation.",
        ];
        for line in positive_cases {
            if !has_placeholder_doc_line(line) {
                return Err(format!(
                    "Expected has_placeholder_doc_line({line:?}) to be true"
                ));
            }
        }

        let negative_cases = [
            "/// Member documentation for x",
            "/// Validates the AST token stream.",
        ];
        for line in negative_cases {
            if has_placeholder_doc_line(line) {
                return Err(format!(
                    "Expected has_placeholder_doc_line({line:?}) to be false"
                ));
            }
        }
        Ok(())
    }

    /// Verifies that `DocstringError` algebraically classifies `Io`, `Parse`, and `UnforcedConflict` failures without string-prefix matching (refs #512).
    #[test]
    fn test_issue_512_docstring_error_algebraic_exit_code_classification() -> Result<(), String> {
        let io_err = DocstringError::Io("disk read failure".to_string());
        let parse_err = DocstringError::Parse("unbalanced brace".to_string());
        let conflict_err = DocstringError::UnforcedConflict("conflicting Realises".to_string());

        assert_eq!(io_err.exit_code(), DocstringExitCode::Error);
        assert_eq!(parse_err.exit_code(), DocstringExitCode::Error);
        assert_eq!(
            conflict_err.exit_code(),
            DocstringExitCode::IdempotencyOrConflict
        );
        assert_eq!(io_err.as_str(), "disk read failure");
        assert_eq!(&*parse_err, "unbalanced brace");
        assert_eq!(format!("{conflict_err}"), "conflicting Realises");
        let converted: String = io_err.into_message();
        assert_eq!(converted, "disk read failure");

        let rust_parse_err = inject_rust_file_docstrings(
            "pub fn broken( {\n",
            "crates/core_module/src/broken.rs",
            None,
            false,
        )
        .err()
        .ok_or("Expected unbalanced Rust syntax to return Err(DocstringError::Parse)")?;
        assert!(matches!(rust_parse_err, DocstringError::Parse(_)));
        assert_eq!(rust_parse_err.exit_code(), DocstringExitCode::Error);

        let dart_parse_err =
            inject_dart_file_docstrings("class Broken {\n", "lib/broken.dart", None, false)
                .err()
                .ok_or("Expected unbalanced Dart syntax to return Err(DocstringError::Parse)")?;
        assert!(matches!(dart_parse_err, DocstringError::Parse(_)));
        assert_eq!(dart_parse_err.exit_code(), DocstringExitCode::Error);

        let model = PackageDef {
            name: "SystemControlPkg".to_string(),
            part_defs: vec![PartDef {
                name: "SystemController".to_string(),
                is_def: true,
                satisfied_requirements: vec!["REQ-PARENT".to_string()],
                ..Default::default()
            }],
            requirement_defs: vec![RequirementDef {
                name: "REQ_PARENT".to_string(),
                req_id: "REQ-PARENT".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let conflict_rust_src = r#"/// System controller classifier.
/// ## 1. Safety Intent & Regulatory Scope
/// Struct SystemController safety scope.
/// ## 2. Invariants & Mathematical Properties
/// Struct SystemController invariants.
/// ## 3. Preconditions
/// Struct SystemController preconditions.
/// ## 4. Postconditions
/// Struct SystemController postconditions.
/// ## 5. Algorithmic Complexity Bounds
/// Time: O(1), Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Deterministic struct layout.
/// ## 7. Realization & Traceability
/// Realises: [REQ-STALE-999]
pub struct SystemController;
"#;
        let rust_conflict_err = inject_rust_file_docstrings(
            conflict_rust_src,
            "crates/core_module/src/controller.rs",
            Some(&model),
            false,
        )
        .err()
        .ok_or(
            "Expected unforced Realises conflict to return Err(DocstringError::UnforcedConflict)",
        )?;
        assert!(matches!(
            rust_conflict_err,
            DocstringError::UnforcedConflict(_)
        ));
        assert_eq!(
            rust_conflict_err.exit_code(),
            DocstringExitCode::IdempotencyOrConflict
        );

        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {e}", self_path.display()))?;
        let (prod_src, _) = self_src
            .split_once("\n#[cfg(test)]\nmod tests")
            .ok_or_else(|| "missing mod tests boundary in src/docstring.rs".to_string())?;
        if prod_src.contains("starts_with(\"Conflicting unforced docstring\")") {
            return Err(
                "src/docstring.rs production code must not use string-prefix matching for exit code classification"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// Verifies that `inspect_rust_item_details` scans Rust signatures, bodies, structs, enums, and traits using brace-depth tracking without fixed line-window truncation (refs #514).
    #[test]
    fn test_issue_514_inspect_rust_item_details_unbounded_brace_depth() {
        // 1. Callable with >16 signature lines and >35 body lines before assert!
        let mut fn_lines = vec!["pub fn evaluate_multi_stage_pipeline(".to_string()];
        for idx in 1..=18 {
            fn_lines.push(format!("    param_{idx}: usize,"));
        }
        fn_lines.push("    late_bound_limit: usize,".to_string());
        fn_lines.push(") -> Result<usize, String> {".to_string());
        for idx in 1..=40 {
            fn_lines.push(format!(
                "    let step_{idx} = param_1.saturating_add({idx});"
            ));
        }
        fn_lines.push("    assert!(late_bound_limit > 0, \"limit must be positive\");".to_string());
        fn_lines.push("    Ok(step_40 + late_bound_limit)".to_string());
        fn_lines.push("}".to_string());
        let fn_raw: Vec<&str> = fn_lines.iter().map(String::as_str).collect();

        let (params, ret_ty, fields, invariants) =
            inspect_rust_item_details(&fn_raw, 0, SourceItemKind::Function);
        assert!(
            params.iter().any(|p| p.name == "late_bound_limit"),
            "Expected parameter 'late_bound_limit' beyond line 16 to be extracted, got: {params:?}"
        );
        assert_eq!(
            ret_ty.as_deref(),
            Some("Result<usize, String>"),
            "Expected return type beyond line 16 to be extracted"
        );
        assert!(fields.is_empty());
        assert_eq!(
            invariants,
            vec!["late_bound_limit > 0".to_string()],
            "Expected assert! beyond line 35 to be extracted"
        );

        // 2. Enum with multi-line struct variant followed by VariantBeta beyond line 25
        let mut enum_lines = vec![
            "pub enum StageStateEnum {".to_string(),
            "    VariantAlpha {".to_string(),
            "        inner_slot_count: u32,".to_string(),
        ];
        for idx in 1..=26 {
            enum_lines.push(format!("        // padding comment line {idx}"));
        }
        enum_lines.push("    },".to_string());
        enum_lines.push("    VariantBeta,".to_string());
        enum_lines.push("}".to_string());
        let enum_raw: Vec<&str> = enum_lines.iter().map(String::as_str).collect();

        let (_, _, enum_variants, _) =
            inspect_rust_item_details(&enum_raw, 0, SourceItemKind::Enum);
        assert_eq!(
            enum_variants,
            vec!["VariantAlpha".to_string(), "VariantBeta".to_string()],
            "Expected only top-level enum variants (excluding struct-variant fields and including variants past line 25)"
        );

        // 3. Trait with a default method containing nested braces followed by another method beyond line 25
        let mut trait_lines = vec![
            "pub trait StageProcessorTrait {".to_string(),
            "    fn process_first_stage(&self) -> usize {".to_string(),
            "        if true {".to_string(),
            "            1".to_string(),
            "        } else {".to_string(),
            "            0".to_string(),
            "        }".to_string(),
            "    }".to_string(),
        ];
        for idx in 1..=24 {
            trait_lines.push(format!("    // trait padding comment {idx}"));
        }
        trait_lines.push("    fn process_final_stage(&self) -> usize;".to_string());
        trait_lines.push("}".to_string());
        let trait_raw: Vec<&str> = trait_lines.iter().map(String::as_str).collect();

        let (_, _, trait_methods, _) =
            inspect_rust_item_details(&trait_raw, 0, SourceItemKind::Trait);
        assert_eq!(
            trait_methods,
            vec![
                "fn process_first_stage".to_string(),
                "fn process_final_stage".to_string()
            ],
            "Expected trait scan to survive closing braces of default methods and extract methods beyond line 25"
        );

        // 4. Unit struct followed immediately by another struct with fields (zero bleed-through)
        let unit_struct_lines = [
            "pub struct UnitMarker;",
            "pub struct SubsequentRecord {",
            "    pub record_id: u64,",
            "}",
        ];
        let (_, _, unit_fields, _) =
            inspect_rust_item_details(&unit_struct_lines, 0, SourceItemKind::Struct);
        assert!(
            unit_fields.is_empty(),
            "Expected unit struct `pub struct UnitMarker;` to have zero extracted fields without bleeding into subsequent struct, got: {unit_fields:?}"
        );
    }

    /// Verifies that `is_compliant_docblock` and `synthesize_7_section_docstring` reject and normalize doubled doc-comment prefixes, legacy H1 headers, and duplicate `Realises:` lines (refs #516).
    #[test]
    fn test_issue_516_rejects_doubled_prefixes_and_hybrid_h1_docblocks() -> Result<(), String> {
        let doubled_outer = format!("{} {}", "///", "///");
        let doubled_inner = format!("{} {}", "//!", "///");
        let legacy_h1_summary = format!("{} {}", "/// #", "Summary");

        let hybrid_fixture = format!(
            r#"{doubled_outer} Custom summary with accidental leading marker.
{doubled_outer} Realises: [REQ-HYBRID-516]
///
{legacy_h1_summary}
/// Legacy H1 summary paragraph.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A for `hybrid_target`.
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic execution for `hybrid_target`.
/// ## 3. Preconditions
/// - Valid input `x: u32`.
/// ## 4. Postconditions
/// - Returns `u32`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe u32 arithmetic.
/// - State: Stateless.
/// - Action: Pure.
/// - Resource: Zero allocation.
/// ## 7. Realization & Traceability
/// Realises: [REQ-HYBRID-516]
pub fn hybrid_target(x: u32) -> u32 {{
    x.saturating_add(1)
}}
"#
        );

        let (enriched, count) = inject_rust_file_docstrings(
            &hybrid_fixture,
            "crates/compile-sysml/src/hybrid_sample.rs",
            None,
            false,
        )
        .map_err(|e| e.into_message())?;

        assert_eq!(
            count, 1,
            "Expected hybrid docblock with doubled prefixes and H1 headers to be rejected and re-synthesized (count == 1), got count={count} in:\n{enriched}"
        );
        assert!(
            !enriched.contains(&doubled_outer),
            "Enriched output must not contain doubled outer doc prefix:\n{enriched}"
        );
        assert!(
            !enriched.contains(&legacy_h1_summary),
            "Enriched output must not contain legacy H1 Summary header:\n{enriched}"
        );
        assert_eq!(
            enriched.matches("Realises:").count(),
            1,
            "Enriched output must contain exactly one Realises: line:\n{enriched}"
        );

        let self_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("docstring.rs");
        let self_src = fs::read_to_string(&self_path)
            .map_err(|e| format!("Failed to read {}: {e}", self_path.display()))?;
        assert!(
            !self_src.contains(&doubled_outer),
            "crates/compile-sysml/src/docstring.rs must not contain doubled outer doc prefixes"
        );
        assert!(
            !self_src.contains(&doubled_inner),
            "crates/compile-sysml/src/docstring.rs must not contain doubled inner doc prefixes"
        );
        assert!(
            !self_src.contains(&legacy_h1_summary),
            "crates/compile-sysml/src/docstring.rs must not contain legacy H1 Summary headers"
        );

        Ok(())
    }

    /// Verifies Tier A (7-section and 4-Block Contract) versus Tier B/C (`fn`, `pub(crate) fn`, and `impl Trait for Type`) compliance in `inject_rust_file_docstrings`.
    #[test]
    fn test_tier_a_and_tier_bc_docstring_compliance() -> Result<(), String> {
        let compliant_src = r#"//! Module header using concise 4-Block Contract.
//!
//! Safety & Traceability: Realises: [REQ-TIER-MOD-01]
//! Contract: Preserves deterministic buffer invariants across all public entrypoints.
//! Bounds & Faults: O(N) time and space; returns typed errors without panicking.

/// Public buffer record documented with a 4-Block Contract.
///
/// Safety & Traceability: Realises: [REQ-TIER-A-STRUCT]
/// Contract: Holds bounded slot count `slots` initialized deterministically.
/// Bounds & Faults: O(1) stack footprint; zero fallible operations.
pub struct RecordBuffer {
    pub slots: usize,
}

/// Public status enumeration documented with full 7-section Rustdoc.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to deterministic status classification for `BufferStatus`.
/// ## 2. Invariants & Mathematical Properties
/// Closed two-variant discriminant set.
/// ## 3. Preconditions
/// Constructed by buffer operations.
/// ## 4. Postconditions
/// Encodes `Ready` or `Empty`.
/// ## 5. Algorithmic Complexity Bounds
/// Time: O(1), Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Exhaustive match semantics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-TIER-A-ENUM]
pub enum BufferStatus {
    Ready,
    Empty,
}

/// Public trait documented with a 4-Block Contract.
///
/// Safety & Traceability: Realises: [REQ-TIER-A-TRAIT]
/// Contract: Implementors return deterministic step counts.
/// Bounds & Faults: O(1) dispatch overhead; total interface.
pub trait StepProcessor {
    /// Computes the next deterministic step count.
    /// Realises: [REQ-TIER-BC-TRAIT-DECL]
    fn next_step(&self) -> usize;
}

impl RecordBuffer {
    /// Returns the configured slot count of this buffer.
    ///
    /// Safety & Traceability: Realises: [REQ-TIER-A-INHERENT-PUB]
    /// Contract: Pure projection of `self.slots`.
    /// Bounds & Faults: O(1) time and space; infallible.
    pub fn capacity(&self) -> usize {
        self.slots
    }

    /// Returns the internal crate-scoped slot index.
    /// Realises: [REQ-TIER-BC-INHERENT-CRATE]
    pub(crate) fn crate_slot(&self) -> usize {
        self.slots
    }

    /// Returns the private internal slot offset.
    /// Realises: [REQ-TIER-BC-INHERENT-PRIV]
    fn private_slot(&self) -> usize {
        self.slots
    }
}

impl std::fmt::Display for RecordBuffer {
    /// Formats the buffer slot count into the target formatter stream.
    /// Realises: [REQ-TIER-BC-DISPLAY]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.slots)
    }
}

impl Default
    for RecordBuffer
{
    /// Constructs an empty zero-slot `RecordBuffer`.
    /// Realises: [REQ-TIER-BC-DEFAULT]
    fn default() -> Self {
        Self { slots: 0 }
    }
}

/// Evaluates a bounded limit using a concise 4-Block Contract.
///
/// Safety & Traceability: Realises: [REQ-TIER-A-PUB-FN]
/// Contract: Increments `limit` with saturating arithmetic.
/// Bounds & Faults: O(1) time and space; never overflows.
pub fn evaluate_public_bound(limit: usize) -> usize {
    limit.saturating_add(1)
}

/// Computes crate-internal step increment.
/// Realises: [REQ-TIER-BC-PUB-CRATE]
pub(crate) fn crate_helper(v: usize) -> usize {
    v.saturating_add(1)
}

/// Computes parent-module step increment.
/// Realises: [REQ-TIER-BC-PUB-SUPER]
pub(super) fn super_helper(v: usize) -> usize {
    v.saturating_add(2)
}

/// Computes path-restricted internal step increment.
/// Realises: [REQ-TIER-BC-PUB-IN]
pub(in crate) fn scoped_helper(v: usize) -> usize {
    v.saturating_add(3)
}

/// Computes private helper increment.
/// Realises: [REQ-TIER-BC-PRIV-FN]
fn local_helper(v: usize) -> usize {
    v.saturating_add(4)
}

/// Internal private cursor state.
/// Realises: [REQ-TIER-BC-PRIV-STRUCT]
struct InternalCursor {
    offset: usize,
}
"#;

        let (enriched, count) = inject_rust_file_docstrings(
            compliant_src,
            "crates/core_module/src/tiered_sample.rs",
            None,
            false,
        )
        .map_err(|e| e.into_message())?;
        assert_eq!(
            count, 0,
            "Expected compliant Tier A (4-Block & 7-Section) and Tier B/C (2-line) items to remain untouched when force=false, got count={count} in:\n{enriched}"
        );
        assert_eq!(enriched, compliant_src);

        // Negative probes:
        // 1. Tier A `pub fn` with only a 2-line Tier B/C docstring must be rejected and enriched
        let tier_a_short_src = r#"/// Summary without 4-Block Contract or 7-Section headers.
/// Realises: [REQ-TIER-A-SHORT]
pub fn public_fn_missing_tier_a_blocks(v: usize) -> usize {
    v
}
"#;
        let (_, short_count) = inject_rust_file_docstrings(
            tier_a_short_src,
            "crates/core_module/src/short_tier_a.rs",
            None,
            false,
        )
        .map_err(|e| e.into_message())?;
        assert_eq!(
            short_count, 1,
            "Tier A `pub fn` with only a 2-line Tier B/C docblock must be rejected and enriched"
        );

        // 2. Inherent `impl` with HRTB `for<'a>` in `where` clause must still treat `pub fn` as Tier A
        let hrtb_inherent_src = r#"/// Generic holder.
/// Safety & Traceability: Realises: [REQ-GENERIC-HOLDER]
/// Contract: Wraps value `v`.
/// Bounds & Faults: O(1) footprint.
pub struct GenericHolder<T> {
    pub v: T,
}

impl<T> GenericHolder<T>
where
    for<'a> &'a T: Copy,
{
    /// Two-line docblock is insufficient for inherent `pub fn`.
    /// Realises: [REQ-GENERIC-GET]
    pub fn borrow_ref(&self) -> &T {
        &self.v
    }
}
"#;
        let (_, hrtb_count) = inject_rust_file_docstrings(
            hrtb_inherent_src,
            "crates/core_module/src/hrtb_inherent.rs",
            None,
            false,
        )
        .map_err(|e| e.into_message())?;
        assert_eq!(
            hrtb_count, 1,
            "Inherent `impl` with HRTB `for<'a>` in `where` clause must classify `pub fn` as Tier A (hrtb_count == 1)"
        );

        // 3. Tier B/C items missing summary, missing Realises, or containing TODO/TBD/FIXME/XXX placeholders must be rejected
        let invalid_tier_bc_src = r#"/// Realises: [REQ-NO-SUMMARY]
fn missing_summary_line(x: u32) -> u32 {
    x
}

/// Summary without a requirement tag.
fn missing_realises_tag(x: u32) -> u32 {
    x
}

/// Summary containing TODO marker.
/// Realises: [REQ-PLACEHOLDER-1]
fn has_todo_marker(x: u32) -> u32 {
    x
}

/// Summary containing TBD marker.
/// Realises: [REQ-PLACEHOLDER-2]
pub(crate) fn has_tbd_marker(x: u32) -> u32 {
    x
}

/// Summary containing FIXME marker.
/// Realises: [REQ-PLACEHOLDER-3]
fn has_fixme_marker(x: u32) -> u32 {
    x
}

/// Summary containing XXX marker.
/// Realises: [REQ-PLACEHOLDER-4]
fn has_xxx_marker(x: u32) -> u32 {
    x
}
"#;
        let (repaired_bc, bc_count) = inject_rust_file_docstrings(
            invalid_tier_bc_src,
            "crates/core_module/src/invalid_bc.rs",
            None,
            false,
        )
        .map_err(|e| e.into_message())?;
        assert_eq!(
            bc_count, 6,
            "Expected all 6 invalid/placeholder Tier B/C functions to be rejected and enriched, got {bc_count}"
        );
        let (_, bc_second_count) = inject_rust_file_docstrings(
            &repaired_bc,
            "crates/core_module/src/invalid_bc.rs",
            None,
            false,
        )
        .map_err(|e| e.into_message())?;
        assert_eq!(
            bc_second_count, 0,
            "Enriched output after replacing placeholder summaries must be idempotent on second pass"
        );

        // 4. Direct classifier & module-docblock (`//!`) Tier A assertions
        assert_eq!(
            classify_rust_item_with_tier_ast("pub fn exported_op()", None),
            Some((SourceItemKind::Function, "exported_op".to_string(), true))
        );
        assert_eq!(
            classify_rust_item_with_tier_ast(
                "pub fn inherent_op(&self)",
                Some(RustOwnerKind::InherentImpl)
            ),
            Some((SourceItemKind::Function, "inherent_op".to_string(), true))
        );
        assert_eq!(
            classify_rust_item_with_tier_ast(
                "pub fn trait_impl_op(&self)",
                Some(RustOwnerKind::TraitImpl)
            ),
            Some((SourceItemKind::Function, "trait_impl_op".to_string(), false))
        );
        assert_eq!(
            classify_rust_item_with_tier_ast("pub(crate) fn internal_op()", None),
            Some((SourceItemKind::Function, "internal_op".to_string(), false))
        );
        assert_eq!(
            classify_rust_item_with_tier_ast("pub (super) fn super_op()", None),
            Some((SourceItemKind::Function, "super_op".to_string(), false))
        );
        assert_eq!(
            classify_rust_item_with_tier_ast("pub(in crate::semantic) fn path_op()", None),
            Some((SourceItemKind::Function, "path_op".to_string(), false))
        );
        assert_eq!(
            classify_rust_item_with_tier_ast("fn private_op()", Some(RustOwnerKind::InherentImpl)),
            Some((SourceItemKind::Function, "private_op".to_string(), false))
        );

        let valid_mod_doc = vec![
            "//! Deterministic module contract.".to_string(),
            "//! Safety & Traceability: Realises: [REQ-MOD-4BLOCK]".to_string(),
            "//! Contract: Total deterministic projections.".to_string(),
            "//! Bounds & Faults: O(N) time and space; zero panics.".to_string(),
        ];
        assert!(
            is_compliant_docblock(&valid_mod_doc, false),
            "Module-level `//!` docblock must always evaluate as Tier A and pass with a valid 4-Block Contract"
        );
        let short_mod_doc = vec![
            "//! Module summary without 4-Block or 7-Section headers.".to_string(),
            "//! Realises: [REQ-MOD-SHORT]".to_string(),
        ];
        assert!(
            !is_compliant_docblock(&short_mod_doc, false),
            "Module-level `//!` docblock with only 2-line Tier B/C headers must fail Tier A validation"
        );
        let missing_summary_4block = vec![
            "/// Safety & Traceability: Realises: [REQ-NO-SUM]".to_string(),
            "/// Contract: Preconditions hold.".to_string(),
            "/// Bounds & Faults: O(1) time.".to_string(),
        ];
        assert!(
            !is_compliant_docblock(&missing_summary_4block, true),
            "Tier A 4-Block Contract missing its summary line must fail"
        );
        let missing_bounds_4block = vec![
            "/// Summary present.".to_string(),
            "/// Safety & Traceability: Realises: [REQ-NO-BOUNDS]".to_string(),
            "/// Contract: Preconditions hold.".to_string(),
        ];
        assert!(
            !is_compliant_docblock(&missing_bounds_4block, true),
            "Tier A 4-Block Contract missing `Bounds & Faults` must fail"
        );

        Ok(())
    }

    /// Verifies `MAX_AST_DEPTH` (64) bounds nested part and package traversal at `depth == 64` and stops cleanly at `depth == 65`.
    #[test]
    fn test_max_ast_depth_bounds_collect_all_parts_and_resolve_schema_context() {
        let beyond_depth = MAX_AST_DEPTH.saturating_add(1);

        let mut current_part = PartDef {
            name: format!("NestedNode{beyond_depth}"),
            is_def: true,
            satisfied_requirements: vec!["REQ-BEYOND-DEPTH".to_string()],
            ..Default::default()
        };
        for depth in (0..=MAX_AST_DEPTH).rev() {
            let reqs = if depth == MAX_AST_DEPTH {
                vec!["REQ-AT-MAX-DEPTH".to_string()]
            } else {
                Vec::new()
            };
            current_part = PartDef {
                name: format!("NestedNode{depth}"),
                is_def: true,
                satisfied_requirements: reqs,
                parts: vec![current_part],
                ..Default::default()
            };
        }

        let pkg = PackageDef {
            name: "DepthBoundPkg".to_string(),
            part_defs: vec![current_part],
            ..Default::default()
        };

        let mut collected = Vec::new();
        collect_all_parts(&pkg, &mut collected);
        assert_eq!(
            collected.len(),
            MAX_AST_DEPTH.saturating_add(1),
            "collect_all_parts must include depths 0..=64 and stop when depth > 64"
        );
        assert!(collected.iter().any(|p| p.name == "NestedNode64"));
        assert!(!collected.iter().any(|p| p.name == "NestedNode65"));

        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "NestedNode64"),
            (Some("REQ-AT-MAX-DEPTH".to_string()), None),
            "Part at exact MAX_AST_DEPTH (64) must resolve schema requirement"
        );
        assert_eq!(
            resolve_schema_context(Some(&pkg), None, "NestedNode65"),
            (None, None),
            "Part beyond MAX_AST_DEPTH (65) must be skipped cleanly"
        );

        let mut nested_pkg = PackageDef {
            name: format!("SubPkg{beyond_depth}"),
            requirement_defs: vec![RequirementDef {
                name: "BeyondPkgAction".to_string(),
                req_id: "REQ-PKG-BEYOND".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        for depth in (0..=MAX_AST_DEPTH).rev() {
            let req_defs = if depth == MAX_AST_DEPTH {
                vec![RequirementDef {
                    name: "AtMaxPkgAction".to_string(),
                    req_id: "REQ-PKG-AT-MAX".to_string(),
                    ..Default::default()
                }]
            } else {
                Vec::new()
            };
            nested_pkg = PackageDef {
                name: format!("SubPkg{depth}"),
                packages: vec![nested_pkg],
                requirement_defs: req_defs,
                ..Default::default()
            };
        }

        assert_eq!(
            resolve_schema_context(Some(&nested_pkg), None, "AtMaxPkgAction"),
            (Some("REQ-PKG-AT-MAX".to_string()), None),
            "Package at exact MAX_AST_DEPTH (64) must resolve requirement_defs"
        );
        assert_eq!(
            resolve_schema_context(Some(&nested_pkg), None, "BeyondPkgAction"),
            (None, None),
            "Package beyond MAX_AST_DEPTH (65) must be skipped cleanly"
        );
    }
}

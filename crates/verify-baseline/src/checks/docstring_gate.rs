//! Check 36: Per-item 7-section safety docstring completeness gate for Rust and Dart sources.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! verification and traceability standards. Mechanically enforces 100% per-item 7-section
//! safety docstring coverage and `Realises: [...]` requirement bindings across all non-test
//! Rust source files in `crates/` and all non-generated Dart source files in `app_flutter/lib/`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero Module-Level Exemptions: Inner module comments (`//!`) in `lib.rs` or any submodule
//!   never satisfy or exempt outer item docstrings (`///`) on `struct`, `enum`, `trait`, or `fn`.
//! - Zero File Allowlists: Recursively walks 100% of non-test `.rs` files under `crates/` and
//!   100% of non-generated `.dart` files under `app_flutter/lib/` (and root `lib/` for Flutter targets).
//! - Zero Placeholder Docstrings: Any occurrence of stub member-documentation or placeholder docstrings
//!   fails closed immediately.
//! - Per-Function Docblock Uniqueness: Normalized 7-section docstring bodies (excluding summary line
//!   and `Realises:` tag) must be unique across all functions/methods within each source file.
//!
//! ## 3. Preconditions
//! - `repo_root` references an accessible local workspace directory on disk.
//! - Source files under `crates/` and `app_flutter/lib/` contain UTF-8 text.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` if and only if every production item across all scanned Rust and Dart files
//!   carries a complete, non-placeholder, unique 7-section docstring with a non-empty `Realises: [...]` tag.
//! - Returns `Err(Vec<String>)` with sorted `file:line` diagnostic violations on any non-compliance.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(F \times L)$ where $F$ is the number of scanned source files and $L$ is the
//!   average line count per file.
//! - Space Complexity: $O(D + V)$ where $D$ is the per-file unique docblock set and $V$ is the violation count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Unreadable directories or non-UTF-8 source files fail closed with explicit I/O diagnostics.
//! - State: Per-file scanner state resets deterministically between files without shared mutable state.
//! - Action: Unbalanced delimiters or malformed `Realises: []` tags fail closed without panicking.
//! - Resource: Bounded line-by-line traversal using non-panicking `.get(..)` slice accessors.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-CHECK-36-DOCSTRING-GATE], [DO-330-TQL-4-DOCSTRING-GATE]

use deap_core::source_ast::{
    extract_dart_getter_ident, strip_leading_dart_annotations,
    strip_line_comment_preserving_strings,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Strongly typed enumeration of the 7 mandatory DO-178C safety docstring sections enforced by Check 36.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C structural documentation taxonomy controls.
/// Replaces static string-slice arrays with a closed, compile-time verified enumeration of the
/// seven canonical safety docstring headings required on every production Rust and Dart item.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijective mapping between the 7 variants in `DocstringSection::ALL` and the 7 exact Markdown
///   H2 headings `## 1. Safety Intent & Regulatory Scope` through `## 7. Realization & Traceability`.
/// - Total ordering follows the canonical section sequence `1..=7`.
///
/// ## 3. Preconditions
/// - Constructed via `DocstringSection::ALL` iteration or explicit enum variant selection.
///
/// ## 4. Postconditions
/// - Projects deterministically to the exact required Markdown H2 heading via `DocstringSection::header`
///   and to a concise diagnostic label via `DocstringSection::short_label`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant-time discriminant dispatch.
/// - Space Complexity: O(1) zero-allocation static representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Closed enum prevents invalid section indices at compile time.
/// - State: Copy-value enum with zero mutable state.
/// - Action: Exhaustive `match` arms prevent unhandled section variants.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocstringSection {
    /// Section 1: `## 1. Safety Intent & Regulatory Scope`.
    SafetyIntent,
    /// Section 2: `## 2. Invariants & Mathematical Properties`.
    Invariants,
    /// Section 3: `## 3. Preconditions`.
    Preconditions,
    /// Section 4: `## 4. Postconditions`.
    Postconditions,
    /// Section 5: `## 5. Algorithmic Complexity Bounds`.
    ComplexityBounds,
    /// Section 6: `## 6. Failure Modes & Fault Tolerance`.
    FailureModes,
    /// Section 7: `## 7. Realization & Traceability`.
    RealizationTraceability,
}

impl DocstringSection {
    /// Declaration-ordered catalog of all 7 mandatory safety docstring sections.
    pub const ALL: &'static [Self] = &[
        Self::SafetyIntent,
        Self::Invariants,
        Self::Preconditions,
        Self::Postconditions,
        Self::ComplexityBounds,
        Self::FailureModes,
        Self::RealizationTraceability,
    ];

    /// Projects a `DocstringSection` variant onto its exact required Markdown H2 heading string.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34 source purity controls within `DocstringSection`.
    /// Provides the authoritative heading literal for `DocstringSection::header` without static `&[&str]` arrays.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Injective mapping from each of the 7 `DocstringSection` variants to its unique `'static`
    ///   H2 heading string prefixed with `## 1.` through `## 7.`.
    ///
    /// ## 3. Preconditions
    /// - `self: DocstringSection` is a valid initialized enum discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns the exact non-empty `'static` Markdown H2 heading string required in item docblocks.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time branch lookup.
    /// - Space Complexity: O(1) static read-only string slice return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` signature accepting `self: DocstringSection`.
    /// - State: Pure function with zero state mutation.
    /// - Action: Cannot panic, time out, or recurse.
    /// - Resource: Zero dynamic memory allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-HEADER]
    pub const fn header(self) -> &'static str {
        match self {
            Self::SafetyIntent => "## 1. Safety Intent & Regulatory Scope",
            Self::Invariants => "## 2. Invariants & Mathematical Properties",
            Self::Preconditions => "## 3. Preconditions",
            Self::Postconditions => "## 4. Postconditions",
            Self::ComplexityBounds => "## 5. Algorithmic Complexity Bounds",
            Self::FailureModes => "## 6. Failure Modes & Fault Tolerance",
            Self::RealizationTraceability => "## 7. Realization & Traceability",
        }
    }

    /// Projects a `DocstringSection` variant onto its concise diagnostic summary label.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A diagnostic clarity requirements within `DocstringSection`.
    /// Formats human-readable missing-section names for Check 36 error messages via `short_label`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective mapping from each `DocstringSection` variant to its numbered short label `1..=7`.
    ///
    /// ## 3. Preconditions
    /// - `self: DocstringSection` is a valid initialized enum discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty `'static` diagnostic label identifying the missing section number and title.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time branch table lookup.
    /// - Space Complexity: O(1) static read-only string slice return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` accepting `self: DocstringSection`.
    /// - State: Pure projection with zero side effects.
    /// - Action: Exhaustive compile-time `match` coverage.
    /// - Resource: Zero heap or stack growth.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-LABEL]
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::SafetyIntent => "1. Safety Intent",
            Self::Invariants => "2. Invariants",
            Self::Preconditions => "3. Preconditions",
            Self::Postconditions => "4. Postconditions",
            Self::ComplexityBounds => "5. Algorithmic Complexity",
            Self::FailureModes => "6. Failure Modes",
            Self::RealizationTraceability => "7. Realization & Traceability",
        }
    }
}

/// Check 36: Enforces 100% per-item 7-section safety docstring coverage across Rust and Dart sources.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C traceability gates.
/// Walks 100% of non-test `.rs` files in `crates/` and 100% of non-generated `.dart` files in
/// `app_flutter/lib/` (and root `lib/` when `pubspec.yaml` is present), rejecting missing 7-section
/// docstrings, placeholder `/// Member documentation.` comments, and `//!` module-level exemptions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero File Allowlists: Every eligible `.rs` and `.dart` file in the workspace tree is audited.
/// - Zero Module-Level Exemptions: `//!` headers never exempt any item inside `lib.rs` or submodules.
/// - Deterministic Ordering: Any discovered violations are sorted and deduplicated before return.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` points to an existing workspace or application root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when zero docstring violations exist across all Rust and Dart source trees.
/// - Returns `Err(Vec<String>)` listing every `file:line` docstring completeness or placeholder violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F * L) where F is total `.rs` and `.dart` file count and L is average line count.
/// - Space Complexity: O(V) where V is total accumulated violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Filesystem traversal or file read errors push fail-closed diagnostics to `violations`.
/// - State: Stateless orchestration aggregating into a local `Vec<String>` buffer.
/// - Action: Evaluates both Rust (`crates/`) and Dart (`app_flutter/lib/`, `lib/`) trees unconditionally.
/// - Resource: Linear memory usage bounded by single-file source content and diagnostic count.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-DOCSTRING-COMPLETENESS]
pub fn check_docstring_completeness(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    let crates_dir = repo_root.join("crates");
    if crates_dir.is_dir() {
        collect_rust_docstring_violations(&crates_dir, repo_root, &mut violations);
    }

    let app_flutter_lib = repo_root.join("app_flutter").join("lib");
    if app_flutter_lib.is_dir() {
        collect_dart_docstring_violations(&app_flutter_lib, repo_root, &mut violations);
    }

    let root_lib = repo_root.join("lib");
    if repo_root.join("pubspec.yaml").is_file() && root_lib.is_dir() {
        collect_dart_docstring_violations(&root_lib, repo_root, &mut violations);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Alias entrypoint for Check 36 7-section safety docstring verification gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 baseline verification interface stability.
/// Exposes `check_7_section_docstring_gate` as a direct delegation to `check_docstring_completeness`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Referential equivalence: `check_7_section_docstring_gate(repo_root) == check_docstring_completeness(repo_root)`
///   for all workspace paths `repo_root`.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` is a valid filesystem path reference to the target repository root.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` on 100% per-item 7-section docstring compliance, or `Err(Vec<String>)` on violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F * L) delegated to `check_docstring_completeness`.
/// - Space Complexity: O(V) proportional to the returned violation vector.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates all underlying `Err(Vec<String>)` diagnostics without filtering or suppression.
/// - State: Zero local state mutation.
/// - Action: Direct synchronous delegation.
/// - Resource: Zero additional heap allocation beyond the delegated call.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-DOCSTRING-GATE-ALIAS]
pub fn check_7_section_docstring_gate(repo_root: &Path) -> Result<(), Vec<String>> {
    check_docstring_completeness(repo_root)
}

/// Recursively walks `crates_dir` and audits every non-test `.rs` file for per-item 7-section docstrings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C source verification completeness controls.
/// Ensures 100% of production Rust source files under `crates_dir` are inspected without file allowlists.
///
/// ## 2. Invariants & Mathematical Properties
/// - Excludes only build/VCS caches and dedicated integration test/bench/example directories via
///   `is_excluded_rust_dir`.
/// - Every discovered `.rs` file is read and audited via `audit_rust_source_docstrings`.
///
/// ## 3. Preconditions
/// - `crates_dir: &Path` is an existing directory (`repo_root/crates`).
/// - `repo_root: &Path` is the parent workspace root used to compute relative diagnostic paths.
/// - `violations: &mut Vec<String>` is a mutable accumulator for diagnostic messages.
///
/// ## 4. Postconditions
/// - Appends any I/O errors or per-item Rust docstring violations discovered under `crates_dir` into `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(R * L) where R is the count of `.rs` files under `crates_dir` and L is line count.
/// - Space Complexity: O(L + V) for the active file buffer and accumulated violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Directory walk errors and `fs::read_to_string` failures are recorded into `violations` (fail-closed).
/// - State: Mutates only the caller-supplied `violations` vector.
/// - Action: Continues scanning remaining files after recording a file violation so all defects are reported in one run.
/// - Resource: Releases each file's string buffer at the end of its loop iteration.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-COLLECT-RUST-DOCSTRINGS]
fn collect_rust_docstring_violations(
    crates_dir: &Path,
    repo_root: &Path,
    violations: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(crates_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let dir_name = e.file_name().to_string_lossy();
            !is_excluded_rust_dir(&dir_name)
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(val) => val,
            Err(err) => {
                violations.push(format!("Check 36 Rust directory walk I/O error: {}", err));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let is_rs = path.extension().map(|ext| ext == "rs").unwrap_or(false);
        if !is_rs {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Err(errs) = audit_rust_source_docstrings(&content, &rel) {
                    violations.extend(errs);
                }
            }
            Err(err) => {
                violations.push(format!("Check 36: Unable to read Rust file '{}': {}", rel, err));
            }
        }
    }
}

/// Recursively walks `dart_lib_dir` and audits every non-generated `.dart` file for 7-section docstrings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D downstream application verification controls.
/// Ensures 100% of hand-authored or transpired `.dart` files under `app_flutter/lib/` (or `lib/`)
/// carry complete 7-section docstrings and zero `/// Member documentation.` placeholders.
///
/// ## 2. Invariants & Mathematical Properties
/// - Excludes build/cache directories via `is_excluded_dart_dir` and codegen outputs via `is_generated_dart_file`.
/// - Audits every remaining `.dart` file via `audit_dart_source_docstrings`.
///
/// ## 3. Preconditions
/// - `dart_lib_dir: &Path` is an existing Dart `lib` directory.
/// - `repo_root: &Path` is the workspace root used for relative diagnostic path formatting.
/// - `violations: &mut Vec<String>` is a mutable vector collecting violation diagnostics.
///
/// ## 4. Postconditions
/// - Appends any I/O errors, placeholder docstring errors, or missing 7-section docstring errors to `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(D * L) where D is the number of `.dart` files and L is average line count.
/// - Space Complexity: O(L + V) for the current file buffer and violation entries.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Fails closed when `WalkDir` or `fs::read_to_string` encounters an unreadable file.
/// - State: Appends deterministically to `violations`.
/// - Action: Scans all non-generated `.dart` files in a single pass.
/// - Resource: Deallocates each file buffer immediately after auditing.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-COLLECT-DART-DOCSTRINGS]
fn collect_dart_docstring_violations(
    dart_lib_dir: &Path,
    repo_root: &Path,
    violations: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(dart_lib_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let dir_name = e.file_name().to_string_lossy();
            !is_excluded_dart_dir(&dir_name)
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(val) => val,
            Err(err) => {
                violations.push(format!("Check 36 Dart directory walk I/O error: {}", err));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();
        let is_dart = path.extension().map(|ext| ext == "dart").unwrap_or(false);
        if !is_dart || is_generated_dart_file(&file_name) {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Err(errs) = audit_dart_source_docstrings(&content, &rel) {
                    violations.extend(errs);
                }
            }
            Err(err) => {
                violations.push(format!("Check 36: Unable to read Dart file '{}': {}", rel, err));
            }
        }
    }
}

/// Determines whether a directory basename inside `crates/` should be excluded from production Rust scanning.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A configuration boundary rules for Check 36 (`is_excluded_rust_dir`).
/// Separates build caches and standalone integration test directories (`tests`, `benches`, `examples`)
/// from production crate sources (`crates/**/src/**/*.rs`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if and only if `dir_name` matches one of the 9 non-production directory basenames
///   (`target`, `.git`, `build`, `Pods`, `ephemeral`, `.dart_tool`, `tests`, `benches`, `examples`).
/// - Never excludes `src` or any production submodule directory.
///
/// ## 3. Preconditions
/// - `dir_name: &str` is the single-component directory basename from `WalkDir`.
///
/// ## 4. Postconditions
/// - Returns a deterministic boolean predicate value with zero side effects.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) bounded string equality comparisons on `dir_name`.
/// - Space Complexity: O(1) auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts any valid UTF-8 `&str` slice.
/// - State: Pure predicate function.
/// - Action: Infallible pattern matching.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-EXCLUDED-RUST-DIR]
fn is_excluded_rust_dir(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "target"
            | ".git"
            | "build"
            | "Pods"
            | "ephemeral"
            | ".dart_tool"
            | "tests"
            | "benches"
            | "examples"
    )
}

/// Determines whether a directory basename inside `app_flutter/lib` should be excluded from Dart scanning.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A workspace filtering rules for Check 36 (`is_excluded_dart_dir`).
/// Prevents build artifact caches and generated l10n/protobuf directories from polluting Dart source scans.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if and only if `dir_name` is a build, VCS, or code-generation directory
///   (`.git`, `.dart_tool`, `build`, `Pods`, `ephemeral`, `generated`).
/// - Never excludes application feature, domain, or component directories (`app`, `domain`, `core`, `components`).
///
/// ## 3. Preconditions
/// - `dir_name: &str` is a directory basename yielded during Dart source tree traversal.
///
/// ## 4. Postconditions
/// - Returns `true` when `dir_name` must be skipped, and `false` for all production Dart directories.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant-time pattern match over `dir_name`.
/// - Space Complexity: O(1) zero-allocation execution.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles empty or arbitrary UTF-8 directory names safely.
/// - State: Stateless boolean predicate.
/// - Action: Cannot panic or block.
/// - Resource: Zero dynamic memory usage.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-EXCLUDED-DART-DIR]
fn is_excluded_dart_dir(dir_name: &str) -> bool {
    matches!(
        dir_name,
        ".git" | ".dart_tool" | "build" | "Pods" | "ephemeral" | "generated"
    )
}

/// Determines whether a `.dart` filename corresponds to a tool-generated artifact.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A source boundary rules for Check 36 (`is_generated_dart_file`).
/// Excludes machine-generated `build_runner` and Flutter plugin registrant files while keeping
/// 100% of hand-written and DEAP-transpiled `.dart` files in scope.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` only for canonical Flutter/Dart codegen suffixes (`.g.dart`, `.freezed.dart`,
///   `.gr.dart`, `.mocks.dart`) and `generated_plugin_registrant.dart`.
///
/// ## 3. Preconditions
/// - `file_name: &str` is the basename of a `.dart` file.
///
/// ## 4. Postconditions
/// - Returns `true` if `file_name` is a generated artifact, or `false` if it must be audited by Check 36.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `file_name.len()` for suffix inspection.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking suffix checks on `file_name`.
/// - State: Pure function with no mutable state.
/// - Action: Deterministic suffix evaluation.
/// - Resource: O(1) stack frame.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-GENERATED-DART-FILE]
fn is_generated_dart_file(file_name: &str) -> bool {
    file_name == "generated_plugin_registrant.dart"
        || file_name.ends_with(".g.dart")
        || file_name.ends_with(".freezed.dart")
        || file_name.ends_with(".gr.dart")
        || file_name.ends_with(".mocks.dart")
}

/// Audits a single Rust source string for 100% per-item 7-section docstring completeness and uniqueness.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Check 34/36 compiler purity requirements.
/// Verifies that every production `struct`, `enum`, `trait`, and `fn` outside `#[cfg(test)]` and
/// `mod tests` carries its own outer `///` 7-section docstring and `Realises: [...]` tag, never
/// allowing `//!` module docstrings to exempt any item.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Module-Level Exemptions: `//!` lines are inspected for placeholder strings but are never
///   appended to `current_doc`, so module headers cannot satisfy item docstrings.
/// - Attribute Preservation: Outer `///` docstrings remain bound across single-line and multi-line
///   `#[...]` attributes until the target item declaration line is reached.
/// - Normalized Body Uniqueness: Every function (`kind == "fn"`) in `source` must have a distinct
///   normalized 7-section body in `seen_docblocks`.
///
/// ## 3. Preconditions
/// - `source: &str` contains valid UTF-8 Rust source text.
/// - `file_rel: &str` is the workspace-relative file path used in violation diagnostics.
///
/// ## 4. Postconditions
/// - Returns `Ok(documented_items)` with the count of verified production items when zero violations occur.
/// - Returns `Err(Vec<String>)` containing explicit `file_rel:line_no` diagnostics when any item is
///   undocumented, incomplete, placeholder, or duplicated.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the number of lines in `source`.
/// - Space Complexity: O(I + V) where I is the number of unique function docblocks and V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Pure string slice analysis over `source` and `file_rel`.
/// - State: Tracks multiline comments, raw strings, attribute brackets, and `#[cfg(test)]` state locally.
/// - Action: Never panics; uses `.get(..)` and safe prefix stripping throughout.
/// - Resource: Bounded memory proportional to active docstring lines and unique docblock hashes.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-AUDIT-RUST-DOCSTRINGS]
pub fn audit_rust_source_docstrings(source: &str, file_rel: &str) -> Result<usize, Vec<String>> {
    let mut violations = Vec::new();
    let mut current_doc: Vec<String> = Vec::new();
    let mut seen_docblocks: BTreeSet<String> = BTreeSet::new();
    let mut in_multiline_comment = false;
    let mut in_raw_string_hashes: Option<usize> = None;
    let mut in_multiline_string = false;
    let mut in_mod_tests = false;
    let mut cfg_test_pending = false;
    let mut attr_bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut documented_items: usize = 0;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();

        let started_in_comment_or_string =
            in_multiline_comment || in_raw_string_hashes.is_some() || in_multiline_string;

        if !started_in_comment_or_string {
            if trimmed.starts_with("//!") {
                if contains_placeholder_docstring(trimmed) {
                    violations.push(format!(
                        "{}:{}: placeholder module docstring '{}' detected",
                        file_rel, line_no, trimmed
                    ));
                }
                // Zero Module-Level Exemption Invariant: `//!` is a module/crate inner docstring
                // and MUST NEVER populate `current_doc` for subsequent structs, enums, traits, or fns.
                continue;
            }

            if trimmed.starts_with("///") {
                if !in_mod_tests {
                    if contains_placeholder_docstring(trimmed) {
                        violations.push(format!(
                            "{}:{}: placeholder docstring '{}' detected",
                            file_rel, line_no, trimmed
                        ));
                    }
                    current_doc.push(trimmed.to_string());
                }
                continue;
            }
        }

        let scanned = scan_rust_line_preserving_lexical_state(
            trimmed,
            &mut in_multiline_comment,
            &mut in_raw_string_hashes,
            &mut in_multiline_string,
        );
        let code = scanned.trim();

        if code.is_empty() {
            continue;
        }

        if code == "mod tests {"
            || code.starts_with("mod tests ")
            || code.starts_with("mod tests{")
            || (cfg_test_pending && code.starts_with("mod "))
        {
            in_mod_tests = true;
        }

        if in_mod_tests {
            continue;
        }

        if attr_bracket_depth > 0 {
            for b in code.bytes() {
                if b == b'[' {
                    attr_bracket_depth = attr_bracket_depth.saturating_add(1);
                } else if b == b']' {
                    attr_bracket_depth = attr_bracket_depth.saturating_sub(1);
                }
            }
            continue;
        }

        if code.starts_with("#[") {
            if code.starts_with("#[cfg(test)") || code.contains("cfg(test)") {
                cfg_test_pending = true;
            }
            for b in code.bytes() {
                if b == b'[' {
                    attr_bracket_depth = attr_bracket_depth.saturating_add(1);
                } else if b == b']' {
                    attr_bracket_depth = attr_bracket_depth.saturating_sub(1);
                }
            }
            continue;
        }

        if paren_depth == 0 {
            if let Some((kind, item_name)) = classify_rust_item_line(code) {
                if cfg_test_pending {
                    cfg_test_pending = false;
                    current_doc.clear();
                } else {
                    validate_item_docblock(
                        &current_doc,
                        file_rel,
                        line_no,
                        kind,
                        &item_name,
                        kind == "fn",
                        &mut seen_docblocks,
                        &mut violations,
                    );
                    documented_items = documented_items.saturating_add(1);
                    current_doc.clear();
                }
            } else {
                cfg_test_pending = false;
                current_doc.clear();
            }
        }

        update_line_delimiter_depths(code, &mut brace_depth, &mut paren_depth);
    }

    if violations.is_empty() {
        Ok(documented_items)
    } else {
        Err(violations)
    }
}

/// Advances Rust block comment, multi-hash raw string (`r"..."`, `r#"..."#`, `r##"..."##`),
/// and standard multiline string state across a single source line while returning the
/// non-string, non-comment code slice for item classification and delimiter tracking.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Check 34/36 lexical boundary soundness (refs #500).
/// Prevents declaration keywords (`fn`, `struct`), doc comments (`///`), or unbalanced delimiters
/// inside `r##"..."##` raw strings and multiline `"..."` strings from corrupting Check 36 verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - A raw string opened with `r` followed by $k \ge 0$ `#` bytes and `"` remains open in
///   `*in_raw_string_hashes = Some(k)` until a closing `"` immediately followed by $k$ `#` bytes is reached.
/// - A standard string opened with `"` remains open in `*in_multiline_string = true` across lines
///   until an unescaped `"` is reached.
/// - Character literals (`'c'` and `'\...'`) are consumed atomically so `'\"'` or `'{'` never
///   perturbs string state or delimiter depth.
///
/// ## 3. Preconditions
/// - `line: &str` is a valid UTF-8 source line slice.
/// - `in_block_comment`, `in_raw_string_hashes`, and `in_multiline_string` hold the active lexical
///   state carried over from the preceding line.
///
/// ## 4. Postconditions
/// - Updates `*in_block_comment`, `*in_raw_string_hashes`, and `*in_multiline_string` to their
///   exact end-of-line lexical states and returns a `String` containing only top-level code tokens.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass traversal where N is `line.len()`.
/// - Space Complexity: O(N) bounded by the output `String` capacity.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.get(..)` exclusively with zero panicking slice or index operations.
/// - State: Mutates only the three caller-supplied lexical state references.
/// - Action: Preserves UTF-8 character boundaries at every step via `len_utf8()`.
/// - Resource: Bounded single-line allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-SCAN-RUST-LEXICAL-STATE]
fn scan_rust_line_preserving_lexical_state(
    line: &str,
    in_block_comment: &mut bool,
    in_raw_string_hashes: &mut Option<usize>,
    in_multiline_string: &mut bool,
) -> String {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i: usize = 0;
    let mut out = String::with_capacity(len);

    while i < len {
        let step_utf8 = line
            .get(i..)
            .and_then(|s| s.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);

        if *in_block_comment {
            if bytes.get(i).copied() == Some(b'*')
                && bytes.get(i.saturating_add(1)).copied() == Some(b'/')
            {
                *in_block_comment = false;
                i = i.saturating_add(2);
            } else {
                i = i.saturating_add(step_utf8);
            }
            continue;
        }

        if let Some(req_hashes) = *in_raw_string_hashes {
            if bytes.get(i).copied() == Some(b'"') {
                let mut h: usize = 0;
                while h < req_hashes
                    && bytes
                        .get(i.saturating_add(1).saturating_add(h))
                        .copied()
                        == Some(b'#')
                {
                    h = h.saturating_add(1);
                }
                if h == req_hashes {
                    *in_raw_string_hashes = None;
                    i = i.saturating_add(1).saturating_add(req_hashes);
                    continue;
                }
            }
            i = i.saturating_add(step_utf8);
            continue;
        }

        if *in_multiline_string {
            if bytes.get(i).copied() == Some(b'\\') {
                let next_len = line
                    .get(i.saturating_add(1)..)
                    .and_then(|s| s.chars().next())
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                i = i.saturating_add(1).saturating_add(next_len);
                continue;
            }
            if bytes.get(i).copied() == Some(b'"') {
                *in_multiline_string = false;
            }
            i = i.saturating_add(step_utf8);
            continue;
        }

        if bytes.get(i).copied() == Some(b'/')
            && bytes.get(i.saturating_add(1)).copied() == Some(b'/')
        {
            break;
        }
        if bytes.get(i).copied() == Some(b'/')
            && bytes.get(i.saturating_add(1)).copied() == Some(b'*')
        {
            *in_block_comment = true;
            i = i.saturating_add(2);
            continue;
        }

        if bytes.get(i).copied() == Some(b'\'') {
            if bytes.get(i.saturating_add(1)).copied() == Some(b'\\') {
                let escaped_len = line
                    .get(i.saturating_add(2)..)
                    .and_then(|s| s.chars().next())
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                let mut scan_close = i.saturating_add(2).saturating_add(escaped_len);
                let limit = i.saturating_add(14).min(len);
                let mut closed_at: Option<usize> = None;
                while scan_close < limit {
                    if bytes.get(scan_close).copied() == Some(b'\'') {
                        closed_at = Some(scan_close);
                        break;
                    }
                    let c_len = line
                        .get(scan_close..)
                        .and_then(|s| s.chars().next())
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    scan_close = scan_close.saturating_add(c_len);
                }
                if let Some(close_idx) = closed_at {
                    i = close_idx.saturating_add(1);
                    continue;
                }
            } else if let Some(inner_ch) =
                line.get(i.saturating_add(1)..).and_then(|s| s.chars().next())
            {
                if inner_ch != '\'' && inner_ch != '\\' {
                    let close_pos = i.saturating_add(1).saturating_add(inner_ch.len_utf8());
                    if bytes.get(close_pos).copied() == Some(b'\'') {
                        i = close_pos.saturating_add(1);
                        continue;
                    }
                }
            }
        }

        if bytes.get(i).copied() == Some(b'r') {
            let prev_is_ident = if i == 0 {
                false
            } else {
                let prev = bytes.get(i.saturating_sub(1)).copied().unwrap_or(b' ');
                if prev == b'b' || prev == b'c' {
                    if i >= 2 {
                        let prev2 = bytes.get(i.saturating_sub(2)).copied().unwrap_or(b' ');
                        prev2.is_ascii_alphanumeric() || prev2 == b'_' || prev2 == b'\''
                    } else {
                        false
                    }
                } else {
                    prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'\''
                }
            };
            if !prev_is_ident {
                let mut h: usize = 0;
                while bytes
                    .get(i.saturating_add(1).saturating_add(h))
                    .copied()
                    == Some(b'#')
                {
                    h = h.saturating_add(1);
                }
                if bytes
                    .get(i.saturating_add(1).saturating_add(h))
                    .copied()
                    == Some(b'"')
                {
                    *in_raw_string_hashes = Some(h);
                    i = i.saturating_add(2).saturating_add(h);
                    continue;
                }
            }
        }

        if bytes.get(i).copied() == Some(b'"') {
            *in_multiline_string = true;
            i = i.saturating_add(1);
            continue;
        }

        if let Some(ch) = line.get(i..).and_then(|s| s.chars().next()) {
            out.push(ch);
            i = i.saturating_add(ch.len_utf8());
        } else {
            break;
        }
    }

    out
}

/// Discriminant tracking active Dart triple-quoted multiline string state across source lines.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D Dart lexical state tracking (`DartMultilineString`).
/// Distinguishes single-quoted (`'''...'''`) and double-quoted (`"""..."""`) Dart multiline strings
/// as well as raw (`r'''...'''` / `r"""..."""`) vs. escape-aware variants.
///
/// ## 2. Invariants & Mathematical Properties
/// - `TripleSingle { raw }` closes only on three consecutive `'` bytes (ignoring `\'` when `!raw`).
/// - `TripleDouble { raw }` closes only on three consecutive `"` bytes (ignoring `\"` when `!raw`).
///
/// ## 3. Preconditions
/// - Constructed when `scan_dart_line_preserving_lexical_state` encounters an unquoted `'''` or `"""` token.
///
/// ## 4. Postconditions
/// - Preserves exact delimiter and escape semantics across newline boundaries until closing `'''` or `"""`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) state inspection.
/// - Space Complexity: O(1) stack value.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Closed strongly typed enum with boolean `raw` flag.
/// - State: Copy value with zero heap references.
/// - Action: Exhaustive pattern matching in `scan_dart_line_preserving_lexical_state`.
/// - Resource: Zero dynamic allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-DART-MULTILINE-STRING-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DartMultilineString {
    /// Dart `'''...'''` or `r'''...'''` multiline string literal.
    TripleSingle {
        /// Whether the literal was opened with an `r` raw prefix.
        raw: bool,
    },
    /// Dart `"""..."""` or `r"""..."""` multiline string literal.
    TripleDouble {
        /// Whether the literal was opened with an `r` raw prefix.
        raw: bool,
    },
}

/// Advances Dart block comments and triple-quoted multiline strings (`'''...'''` / `"""..."""`)
/// across a single source line while preserving single-line code and string tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Tier 2 Flutter lexical boundary completeness (refs #500).
/// Prevents interior lines of Dart triple-quoted strings from triggering false-positive Check 36
/// item classifications, placeholder errors, or delimiter depth drift.
///
/// ## 2. Invariants & Mathematical Properties
/// - Characters inside `/* ... */` block comments and `'''...'''` / `"""..."""` multiline strings
///   are stripped from the returned code line.
/// - Single-line `'...'` and `"..."` literals are emitted intact so `update_line_delimiter_depths`
///   can evaluate `${...}` string interpolations accurately.
///
/// ## 3. Preconditions
/// - `line: &str` is a valid UTF-8 Dart source line.
/// - `in_block_comment` and `in_triple_string` hold the active lexical state from the previous line.
///
/// ## 4. Postconditions
/// - Updates `*in_block_comment` and `*in_triple_string` to their end-of-line states and returns
///   the non-comment, non-triple-string Dart code slice.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass scan over `line.len()`.
/// - Space Complexity: O(N) bounded by the output `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` byte and char accessors throughout.
/// - State: Mutates only `in_block_comment` and `in_triple_string`.
/// - Action: Maintains valid UTF-8 boundaries at every iteration via `len_utf8()`.
/// - Resource: Bounded single-line buffer allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-SCAN-DART-LEXICAL-STATE]
fn scan_dart_line_preserving_lexical_state(
    line: &str,
    in_block_comment: &mut bool,
    in_triple_string: &mut Option<DartMultilineString>,
) -> String {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i: usize = 0;
    let mut in_single_line_quote: Option<(u8, bool)> = None;
    let mut out = String::with_capacity(len);

    while i < len {
        let step_utf8 = line
            .get(i..)
            .and_then(|s| s.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);

        if *in_block_comment {
            if bytes.get(i).copied() == Some(b'*')
                && bytes.get(i.saturating_add(1)).copied() == Some(b'/')
            {
                *in_block_comment = false;
                i = i.saturating_add(2);
            } else {
                i = i.saturating_add(step_utf8);
            }
            continue;
        }

        if let Some(triple_kind) = *in_triple_string {
            let (q_byte, is_raw) = match triple_kind {
                DartMultilineString::TripleSingle { raw } => (b'\'', raw),
                DartMultilineString::TripleDouble { raw } => (b'"', raw),
            };
            if !is_raw && bytes.get(i).copied() == Some(b'\\') {
                let next_len = line
                    .get(i.saturating_add(1)..)
                    .and_then(|s| s.chars().next())
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                i = i.saturating_add(1).saturating_add(next_len);
                continue;
            }
            if bytes.get(i).copied() == Some(q_byte)
                && bytes.get(i.saturating_add(1)).copied() == Some(q_byte)
                && bytes.get(i.saturating_add(2)).copied() == Some(q_byte)
            {
                *in_triple_string = None;
                i = i.saturating_add(3);
                continue;
            }
            i = i.saturating_add(step_utf8);
            continue;
        }

        if let Some((q_byte, is_raw)) = in_single_line_quote {
            if !is_raw && bytes.get(i).copied() == Some(b'\\') {
                out.push('\\');
                if let Some(next_ch) = line
                    .get(i.saturating_add(1)..)
                    .and_then(|s| s.chars().next())
                {
                    out.push(next_ch);
                    i = i.saturating_add(1).saturating_add(next_ch.len_utf8());
                } else {
                    i = len;
                }
                continue;
            }
            if bytes.get(i).copied() == Some(q_byte) {
                in_single_line_quote = None;
            }
            if let Some(ch) = line.get(i..).and_then(|s| s.chars().next()) {
                out.push(ch);
                i = i.saturating_add(ch.len_utf8());
            } else {
                break;
            }
            continue;
        }

        if bytes.get(i).copied() == Some(b'/')
            && bytes.get(i.saturating_add(1)).copied() == Some(b'/')
        {
            break;
        }
        if bytes.get(i).copied() == Some(b'/')
            && bytes.get(i.saturating_add(1)).copied() == Some(b'*')
        {
            *in_block_comment = true;
            i = i.saturating_add(2);
            continue;
        }

        if bytes.get(i).copied() == Some(b'r') {
            let prev_is_ident = i > 0
                && bytes
                    .get(i.saturating_sub(1))
                    .copied()
                    .map(|b| b.is_ascii_alphanumeric() || b == b'_')
                    .unwrap_or(false);
            if !prev_is_ident {
                if bytes.get(i.saturating_add(1)).copied() == Some(b'\'')
                    && bytes.get(i.saturating_add(2)).copied() == Some(b'\'')
                    && bytes.get(i.saturating_add(3)).copied() == Some(b'\'')
                {
                    *in_triple_string = Some(DartMultilineString::TripleSingle { raw: true });
                    i = i.saturating_add(4);
                    continue;
                }
                if bytes.get(i.saturating_add(1)).copied() == Some(b'"')
                    && bytes.get(i.saturating_add(2)).copied() == Some(b'"')
                    && bytes.get(i.saturating_add(3)).copied() == Some(b'"')
                {
                    *in_triple_string = Some(DartMultilineString::TripleDouble { raw: true });
                    i = i.saturating_add(4);
                    continue;
                }
                if let Some(q) = bytes.get(i.saturating_add(1)).copied() {
                    if q == b'\'' || q == b'"' {
                        in_single_line_quote = Some((q, true));
                        out.push('r');
                        out.push(char::from(q));
                        i = i.saturating_add(2);
                        continue;
                    }
                }
            }
        }

        if bytes.get(i).copied() == Some(b'\'')
            && bytes.get(i.saturating_add(1)).copied() == Some(b'\'')
            && bytes.get(i.saturating_add(2)).copied() == Some(b'\'')
        {
            *in_triple_string = Some(DartMultilineString::TripleSingle { raw: false });
            i = i.saturating_add(3);
            continue;
        }
        if bytes.get(i).copied() == Some(b'"')
            && bytes.get(i.saturating_add(1)).copied() == Some(b'"')
            && bytes.get(i.saturating_add(2)).copied() == Some(b'"')
        {
            *in_triple_string = Some(DartMultilineString::TripleDouble { raw: false });
            i = i.saturating_add(3);
            continue;
        }

        if let Some(q) = bytes.get(i).copied() {
            if q == b'\'' || q == b'"' {
                in_single_line_quote = Some((q, false));
                out.push(char::from(q));
                i = i.saturating_add(1);
                continue;
            }
        }

        if let Some(ch) = line.get(i..).and_then(|s| s.chars().next()) {
            out.push(ch);
            i = i.saturating_add(ch.len_utf8());
        } else {
            break;
        }
    }

    out
}

/// Audits a single Dart source string for 100% per-item 7-section docstring completeness and zero placeholders.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and Tier 2 downstream Flutter verification rules.
/// Rejects `/// Member documentation.` placeholders and verifies that every Dart `class`, `enum`,
/// `mixin`, `extension`, `typedef`, constructor, top-level function, and class method carries a
/// complete 7-section `///` docstring and a non-empty `Realises: [...]` tag.
///
/// ## 2. Invariants & Mathematical Properties
/// - Brace and Parenthesis Depth Tracking: Tracks `(brace_depth, paren_depth)` so named parameter
///   blocks `({super.key})` inside `(...)` never corrupt class vs. method body brace depth.
/// - Annotation Preservation: Preserves `current_doc` across `@override` and `@immutable` annotations.
/// - Placeholder Rejection: Any `/// Member documentation.` line triggers an immediate violation even if
///   attached to a constructor or field.
///
/// ## 3. Preconditions
/// - `source: &str` contains valid UTF-8 Dart source code.
/// - `file_rel: &str` is the workspace-relative path of the `.dart` file.
///
/// ## 4. Postconditions
/// - Returns `Ok(documented_items)` when every eligible Dart declaration satisfies Check 36.
/// - Returns `Err(Vec<String>)` with explicit `file_rel:line_no` diagnostics on any violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the line count of `source`.
/// - Space Complexity: O(I + V) where I is the unique method docblock count and V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking line and byte scanning over `source`.
/// - State: Local state machine tracking `brace_depth`, `paren_depth`, and `active_class_name`.
/// - Action: Never panics on malformed Dart syntax; saturating arithmetic guards depth counters.
/// - Resource: Linear memory bounded by the active docblock and diagnostic list.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-AUDIT-DART-DOCSTRINGS]
pub fn audit_dart_source_docstrings(source: &str, file_rel: &str) -> Result<usize, Vec<String>> {
    let mut violations = Vec::new();
    let mut current_doc: Vec<String> = Vec::new();
    let mut seen_docblocks: BTreeSet<String> = BTreeSet::new();
    let mut in_multiline_comment = false;
    let mut in_triple_string: Option<DartMultilineString> = None;
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut active_class_name: Option<String> = None;
    let mut documented_items: usize = 0;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();

        let started_in_comment_or_triple = in_multiline_comment || in_triple_string.is_some();

        if !started_in_comment_or_triple && trimmed.starts_with("///") {
            if contains_placeholder_docstring(trimmed) {
                violations.push(format!(
                    "{}:{}: placeholder docstring '{}' detected",
                    file_rel, line_no, trimmed
                ));
            }
            current_doc.push(trimmed.to_string());
            continue;
        }

        let scanned = scan_dart_line_preserving_lexical_state(
            trimmed,
            &mut in_multiline_comment,
            &mut in_triple_string,
        );
        let code = strip_line_comment_preserving_strings(&scanned);

        if code.is_empty() {
            continue;
        }

        let stripped_code = strip_leading_dart_annotations(code);
        if stripped_code.trim().is_empty() {
            continue;
        }
        let code = stripped_code.trim();

        if paren_depth == 0 {
            if brace_depth == 0 {
                if let Some((kind, item_name)) = classify_dart_top_level_item(code) {
                    let check_uniqueness = kind == "fn";
                    validate_item_docblock(
                        &current_doc,
                        file_rel,
                        line_no,
                        kind,
                        &item_name,
                        check_uniqueness,
                        &mut seen_docblocks,
                        &mut violations,
                    );
                    documented_items = documented_items.saturating_add(1);
                    if kind == "class" || kind == "mixin" || kind == "extension" {
                        active_class_name = Some(item_name);
                    }
                    current_doc.clear();
                } else {
                    current_doc.clear();
                }
            } else if brace_depth == 1 {
                if let Some(ref class_name) = active_class_name {
                    if let Some((kind, member_name)) = classify_dart_member_item(code, class_name) {
                        validate_item_docblock(
                            &current_doc,
                            file_rel,
                            line_no,
                            kind,
                            &member_name,
                            true,
                            &mut seen_docblocks,
                            &mut violations,
                        );
                        documented_items = documented_items.saturating_add(1);
                        current_doc.clear();
                    } else {
                        current_doc.clear();
                    }
                } else {
                    current_doc.clear();
                }
            } else {
                current_doc.clear();
            }
        }

        let prev_brace_depth = brace_depth;
        update_line_delimiter_depths(code, &mut brace_depth, &mut paren_depth);
        if (prev_brace_depth > 0 && brace_depth == 0)
            || (brace_depth == 0 && (code.ends_with(';') || code.ends_with('}')))
        {
            active_class_name = None;
        }
    }

    if violations.is_empty() {
        Ok(documented_items)
    } else {
        Err(violations)
    }
}

/// Validates that `current_doc` contains all 7 canonical sections, a non-empty `Realises: [...]` tag,
/// and (for functions/methods) a unique normalized body within the file.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34/36 docstring completeness and anti-boilerplate rules.
/// Enforces that every audited `item_name` of `kind` at `file_rel:line_no` has all 7 `DocstringSection`
/// headings, a non-empty requirement realization tag, and non-duplicated content.
///
/// ## 2. Invariants & Mathematical Properties
/// - Completeness: Every variant in `DocstringSection::ALL` must appear in `current_doc`.
/// - Traceability: `has_non_empty_realises_tag` must hold on the joined docblock.
/// - Uniqueness: When `check_uniqueness` is `true`, the normalized body (skipping line 1 and `Realises:`)
///   must insert cleanly as a new entry in `seen_docblocks`.
///
/// ## 3. Preconditions
/// - `current_doc: &[String]` holds the contiguous `///` lines preceding the declaration at `line_no`.
/// - `file_rel`, `kind`, and `item_name` are non-empty diagnostic identifiers.
/// - `seen_docblocks` and `violations` are mutable per-file accumulators.
///
/// ## 4. Postconditions
/// - Appends descriptive diagnostic strings to `violations` if any section, traceability tag, or
///   uniqueness invariant is violated; leaves `violations` unchanged when compliant.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(D * log(S)) where D is the byte length of `current_doc` and S is `seen_docblocks.len()`.
/// - Space Complexity: O(D) for the joined and normalized docblock strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles empty `current_doc` cleanly by emitting a single missing-docstring diagnostic.
/// - State: Updates `seen_docblocks` and `violations` deterministically.
/// - Action: Pure string inspection with zero panicking indexing operations.
/// - Resource: Bounded by single-item docstring size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-VALIDATE-ITEM-DOCBLOCK]
#[allow(clippy::too_many_arguments)]
fn validate_item_docblock(
    current_doc: &[String],
    file_rel: &str,
    line_no: usize,
    kind: &str,
    item_name: &str,
    check_uniqueness: bool,
    seen_docblocks: &mut BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    if current_doc.is_empty() {
        violations.push(format!(
            "{}:{}: {} '{}' missing mandatory 7-section docstring",
            file_rel, line_no, kind, item_name
        ));
        return;
    }

    let doc_block = current_doc.join("\n");
    let mut missing = Vec::new();
    for section in DocstringSection::ALL {
        if !doc_block.contains(section.header()) {
            missing.push(section.short_label());
        }
    }

    if !missing.is_empty() {
        violations.push(format!(
            "{}:{}: {} '{}' missing mandatory 7-section docstring (missing: {})",
            file_rel,
            line_no,
            kind,
            item_name,
            missing.join(", ")
        ));
    }

    if !has_non_empty_realises_tag(&doc_block) {
        violations.push(format!(
            "{}:{}: {} '{}' missing non-empty 'Realises: [...]' traceability tag",
            file_rel, line_no, kind, item_name
        ));
    }

    if check_uniqueness {
        let normalized_doc = current_doc
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
            .join("\n");
        if !normalized_doc.is_empty() && !seen_docblocks.insert(normalized_doc) {
            violations.push(format!(
                "{}:{}: {} '{}' has duplicate/boilerplate 7-section docstring body",
                file_rel, line_no, kind, item_name
            ));
        }
    }
}

/// Checks whether a joined docblock contains a non-empty `Realises: [<ID>]` traceability binding.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A bidirectional requirement traceability rules (`has_non_empty_realises_tag`).
/// Prevents vacuous `Realises: []` tags from passing Check 36 verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if and only if `doc_block` contains `"Realises: ["` followed by a closing `']'`
///   whose enclosed trimmed substring is non-empty.
///
/// ## 3. Preconditions
/// - `doc_block: &str` is the newline-joined outer docstring of an audited item.
///
/// ## 4. Postconditions
/// - Returns `true` when at least one non-empty bracketed requirement ID follows `Realises: [`, else `false`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `doc_block.len()`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe substring splitting via `split_once` without byte-offset indexing.
/// - State: Pure predicate with zero side effects.
/// - Action: Returns `false` on missing prefix or unclosed bracket.
/// - Resource: Zero dynamic allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-NON-EMPTY-REALISES-TAG]
fn has_non_empty_realises_tag(doc_block: &str) -> bool {
    if let Some((_, after_prefix)) = doc_block.split_once("Realises: [") {
        if let Some((inside_brackets, _)) = after_prefix.split_once(']') {
            return !inside_brackets.trim().is_empty();
        }
    }
    false
}

/// Detects forbidden placeholder docstrings (such as stub member-documentation lines) on a comment line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A anti-stub governance (`contains_placeholder_docstring`).
/// Specifically catches the Tier 2 stub member-documentation defect observed in `app_flutter/lib/app/app.dart`
/// as well as generic placeholder doc comments in Rust and Dart.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips leading `///` or `//!` markers and whitespace, then returns `true` whenever the
///   normalized comment text matches known stub member-documentation or todo-documentation phrases.
///
/// ## 3. Preconditions
/// - `doc_line: &str` is a trimmed `///` or `//!` comment line.
///
/// ## 4. Postconditions
/// - Returns `true` if `doc_line` is a forbidden placeholder comment line, or `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is the byte length of `doc_line`.
/// - Space Complexity: O(N) for the ASCII-lowercased comparison string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts any UTF-8 `&str` line.
/// - State: Pure function with no shared state.
/// - Action: Deterministic normalized string comparison.
/// - Resource: Single short-lived lowercase string allocation bounded by line length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-PLACEHOLDER-DETECTION]
fn contains_placeholder_docstring(doc_line: &str) -> bool {
    let body = doc_line
        .trim_start_matches("///")
        .trim_start_matches("//!")
        .trim()
        .to_ascii_lowercase();
    let needle_member_dot = ["member ", "documentation."].concat();
    let needle_member = ["member ", "documentation"].concat();
    let needle_todo_doc = ["todo: ", "document"].concat();
    let needle_placeholder_doc = ["placeholder ", "documentation"].concat();

    body == needle_member_dot
        || body == needle_member
        || body.starts_with(&needle_member_dot)
        || body.starts_with(&needle_todo_doc)
        || body.starts_with(&needle_placeholder_doc)
}

/// Classifies a comment-stripped Rust source line as a `struct`, `enum`, `trait`, or `fn` declaration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A per-item Rust verification rules (`classify_rust_item_line`).
/// Identifies all production `pub struct`, `struct`, `pub enum`, `enum`, `pub trait`, `trait`,
/// `pub fn`, `pub const fn`, `pub async fn`, `pub(crate) fn`, and `fn` items without regex.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips optional visibility (`pub`, `pub(crate)`, `pub(super)`, `pub(in ...)`) and function
///   qualifiers (`const`, `async`) before matching `struct `, `enum `, `trait `, or `fn `.
/// - Returns `Some((kind, identifier))` if and only if a valid Rust identifier immediately follows the keyword.
///
/// ## 3. Preconditions
/// - `code: &str` is a trimmed, comment-stripped Rust source line outside `mod tests` and strings.
///
/// ## 4. Postconditions
/// - Returns `Some((kind, name))` where `kind` is `"struct"`, `"enum"`, `"trait"`, or `"fn"`,
///   or `None` if `code` is not an eligible Rust item declaration.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of the extracted item identifier string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking prefix stripping via `strip_prefix` and `split_once`.
/// - State: Stateless lexical classifier.
/// - Action: Returns `None` cleanly if no identifier follows the keyword.
/// - Resource: Allocates at most one short `String` for the item identifier.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-CLASSIFY-RUST-ITEM]
fn classify_rust_item_line(code: &str) -> Option<(&'static str, String)> {
    let after_vis = if let Some(rest) = code.strip_prefix("pub(crate) ") {
        rest.trim_start()
    } else if let Some(rest) = code.strip_prefix("pub(super) ") {
        rest.trim_start()
    } else if let Some(rest) = code.strip_prefix("pub(in ") {
        if let Some((_, after_paren)) = rest.split_once(") ") {
            after_paren.trim_start()
        } else {
            code
        }
    } else if let Some(rest) = code.strip_prefix("pub ") {
        rest.trim_start()
    } else {
        code
    };

    if let Some(rest) = after_vis.strip_prefix("struct ") {
        let name = extract_leading_identifier(rest.trim_start())?;
        return Some(("struct", name));
    }
    if let Some(rest) = after_vis.strip_prefix("enum ") {
        let name = extract_leading_identifier(rest.trim_start())?;
        return Some(("enum", name));
    }
    if let Some(rest) = after_vis.strip_prefix("trait ") {
        let name = extract_leading_identifier(rest.trim_start())?;
        return Some(("trait", name));
    }

    let after_qual = if let Some(rest) = after_vis.strip_prefix("const async ") {
        rest.trim_start()
    } else if let Some(rest) = after_vis.strip_prefix("async ") {
        rest.trim_start()
    } else if let Some(rest) = after_vis.strip_prefix("const ") {
        rest.trim_start()
    } else {
        after_vis
    };

    if let Some(rest) = after_qual.strip_prefix("fn ") {
        let name = extract_leading_identifier(rest.trim_start())?;
        return Some(("fn", name));
    }

    None
}

/// Classifies a top-level (`brace_depth == 0`) Dart line as a `class`, `enum`, `mixin`, `extension`, `typedef`, or `fn`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Tier 2 Flutter verification rules (`classify_dart_top_level_item`).
/// Detects all top-level Dart type and function declarations in `app_flutter/lib/**/*.dart` without regex.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips Dart 3 class modifiers (`abstract`, `base`, `final`, `interface`, `sealed`, `mixin`) before
///   matching `class `, `enum `, `mixin `, `extension `, or `typedef `.
/// - Identifies top-level functions (such as `void main()`) when a line contains `(` without preceding `=`
///   and is not an `import`, `export`, `part`, `library`, or top-level variable declaration.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Dart source line at `brace_depth == 0` and `paren_depth == 0`.
///
/// ## 4. Postconditions
/// - Returns `Some((kind, identifier))` for top-level Dart declarations, or `None` for directives/variables.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the extracted identifier length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe prefix stripping and token scanning without panicking slices.
/// - State: Pure function with zero side effects.
/// - Action: Bounded loop stripping at most 4 modifier keywords.
/// - Resource: Allocates only the returned identifier `String`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-CLASSIFY-DART-TOP-LEVEL]
fn classify_dart_top_level_item(code: &str) -> Option<(&'static str, String)> {
    if code.starts_with("import ")
        || code.starts_with("export ")
        || code.starts_with("part ")
        || code.starts_with("library ")
        || code.starts_with("final ")
        || code.starts_with("const ")
        || code.starts_with("var ")
        || code.starts_with("late ")
    {
        return None;
    }

    let mut rest = code;
    for _ in 0..4 {
        if let Some(after) = rest.strip_prefix("abstract ") {
            rest = after.trim_start();
        } else if let Some(after) = rest.strip_prefix("base ") {
            rest = after.trim_start();
        } else if let Some(after) = rest.strip_prefix("final ") {
            rest = after.trim_start();
        } else if let Some(after) = rest.strip_prefix("interface ") {
            rest = after.trim_start();
        } else if let Some(after) = rest.strip_prefix("sealed ") {
            rest = after.trim_start();
        } else {
            break;
        }
    }

    if let Some(after_mixin_class) = rest.strip_prefix("mixin class ") {
        let name = extract_leading_identifier(after_mixin_class.trim_start())?;
        return Some(("class", name));
    }
    if let Some(after_class) = rest.strip_prefix("class ") {
        let name = extract_leading_identifier(after_class.trim_start())?;
        return Some(("class", name));
    }
    if let Some(after_enum) = rest.strip_prefix("enum ") {
        let name = extract_leading_identifier(after_enum.trim_start())?;
        return Some(("enum", name));
    }
    if let Some(after_mixin) = rest.strip_prefix("mixin ") {
        let name = extract_leading_identifier(after_mixin.trim_start())?;
        return Some(("mixin", name));
    }
    if let Some(after_ext_type) = rest.strip_prefix("extension type ") {
        let name = extract_leading_identifier(after_ext_type.trim_start())?;
        return Some(("extension", name));
    }
    if let Some(after_ext) = rest.strip_prefix("extension ") {
        let name = extract_leading_identifier(after_ext.trim_start())
            .unwrap_or_else(|| "anonymous_extension".to_string());
        return Some(("extension", name));
    }
    if let Some(after_typedef) = rest.strip_prefix("typedef ") {
        let name = extract_leading_identifier(after_typedef.trim_start())?;
        return Some(("typedef", name));
    }

    if let Some(fn_name) = extract_dart_callable_name(code) {
        return Some(("fn", fn_name));
    }

    if let Some(getter_name) = extract_dart_getter_ident(code) {
        return Some(("fn", getter_name));
    }

    None
}

/// Classifies a class-body (`brace_depth == 1`) Dart line as a constructor or method declaration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Tier 2 Flutter defect remediation (`classify_dart_member_item`).
/// Specifically identifies constructors such as `const MyApp({super.key});`, named/factory constructors,
/// and methods such as `Widget build(BuildContext context)` inside `class_name` while ignoring fields.
///
/// ## 2. Invariants & Mathematical Properties
/// - Constructors match `const <class_name>(...)`, `<class_name>(...)`, `<class_name>.<named>(...)`,
///   and `factory <class_name>...`.
/// - Field declarations (`final ...;`, `var ...;`, `late ...;`, `static const ...;`, or assignments
///   where `=` precedes `(`) return `None`.
/// - Member methods return `Some(("method", method_name))`.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Dart line at `brace_depth == 1` and `paren_depth == 0`.
/// - `class_name: &str` is the identifier of the enclosing Dart `class`, `mixin`, or `extension`.
///
/// ## 4. Postconditions
/// - Returns `Some(("constructor", name))` or `Some(("method", name))` for callable class members,
///   or `None` for field declarations and closing braces.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of the extracted constructor or method name.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking string prefix and delimiter checks.
/// - State: Pure function of `(code, class_name)`.
/// - Action: Distinguishes `const ClassName(` constructors from `static const` fields deterministically.
/// - Resource: Allocates at most one `String` for the member name.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-CLASSIFY-DART-MEMBER]
fn classify_dart_member_item(code: &str, class_name: &str) -> Option<(&'static str, String)> {
    if code.starts_with("final ")
        || code.starts_with("var ")
        || code.starts_with("late ")
        || code.starts_with("static final ")
        || code.starts_with("static const ")
        || code.starts_with("static var ")
        || code.starts_with("static late ")
    {
        return None;
    }

    if let Some(after_factory) = code.strip_prefix("factory ") {
        let trimmed_after = after_factory.trim_start();
        if let Some((cls, rest)) = trimmed_after.split_once('.') {
            if let Some(named) = extract_leading_identifier(rest) {
                return Some(("constructor", format!("{}.{}", cls, named)));
            }
        }
        let cname = extract_leading_identifier(trimmed_after)?;
        return Some(("constructor", cname));
    }

    let after_const = if let Some(rest) = code.strip_prefix("const ") {
        rest.trim_start()
    } else {
        code
    };

    if let Some(after_cls) = after_const.strip_prefix(class_name) {
        let trimmed_after = after_cls.trim_start();
        if trimmed_after.starts_with('(') {
            return Some(("constructor", class_name.to_string()));
        }
        if let Some(after_dot) = trimmed_after.strip_prefix('.') {
            if let Some(named_ctor) = extract_leading_identifier(after_dot) {
                return Some(("constructor", format!("{}.{}", class_name, named_ctor)));
            }
        }
    }

    if code.starts_with("const ") {
        return None;
    }

    if let Some(getter_name) = extract_dart_getter_ident(code) {
        return Some(("method", getter_name));
    }

    let after_mods = code
        .strip_prefix("static ")
        .unwrap_or(code)
        .trim_start();

    if let Some(method_name) = extract_dart_callable_name(after_mods) {
        if method_name != "if"
            && method_name != "for"
            && method_name != "while"
            && method_name != "switch"
            && method_name != "return"
            && method_name != "assert"
            && method_name != "super"
        {
            return Some(("method", method_name));
        }
    }

    None
}

/// Extracts the leading ASCII identifier (`[A-Za-z0-9_]+`) from `input`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 zero-regex lexical scanning (`extract_leading_identifier`).
/// Parses Rust and Dart type/function identifiers deterministically without regular expressions or
/// panicking byte slices.
///
/// ## 2. Invariants & Mathematical Properties
/// - Consumes the maximal leading prefix of characters satisfying `c.is_ascii_alphanumeric() || c == '_'`.
/// - Returns `None` if the first character is not an ASCII alphabetic character or `'_'`.
///
/// ## 3. Preconditions
/// - `input: &str` is a trimmed source slice positioned immediately after a declaration keyword.
///
/// ## 4. Postconditions
/// - Returns `Some(String)` containing the non-empty identifier, or `None` if `input` does not start
///   with a valid identifier character.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K) where K is the character length of the leading identifier.
/// - Space Complexity: O(K) for the returned `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Iterates via `.chars()` with zero byte-index slicing.
/// - State: Pure function with no mutable external state.
/// - Action: Terminates at the first non-identifier character.
/// - Resource: Bounded by single identifier token length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-EXTRACT-IDENTIFIER]
fn extract_leading_identifier(input: &str) -> Option<String> {
    let mut chars = input.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    let mut ident = String::new();
    ident.push(first);
    for ch in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident.push(ch);
        } else {
            break;
        }
    }
    Some(ident)
}

/// Extracts the function or method name preceding `(` in a Dart callable declaration line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 zero-regex Dart parsing (`extract_dart_callable_name`).
/// Identifies Dart function and method declarations such as `Widget build(BuildContext context)` or
/// `Future<void> initialize() async` while rejecting field assignments where `=` precedes `(`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Requires `'('` to appear in `code`, and requires any `'='` (not part of `=>`) to appear only after `'('`.
/// - Requires at least one return-type token and whitespace before the callable identifier preceding `'('`.
///
/// ## 3. Preconditions
/// - `code: &str` is a trimmed, modifier-stripped Dart declaration line at `brace_depth <= 1`.
///
/// ## 4. Postconditions
/// - Returns `Some(callable_name)` when `code` has `<ReturnType> <callable_name>(...)` syntax, or `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of `callable_name`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `split_once('(')` and character iterators without panicking range indexing.
/// - State: Pure lexical helper.
/// - Action: Rejects field initializers and bare calls without return types.
/// - Resource: Allocates only the extracted method name `String`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-EXTRACT-DART-CALLABLE]
fn extract_dart_callable_name(code: &str) -> Option<String> {
    let (before_paren, _) = code.split_once('(')?;
    if before_paren.contains('=') || before_paren.contains(';') {
        return None;
    }
    let trimmed_before = before_paren.trim();
    // Strip optional generic type parameter suffix `<T>` on generic functions/methods
    let without_trailing_generics = if trimmed_before.ends_with('>') {
        let mut depth: usize = 0;
        let mut cut_byte_idx: Option<usize> = None;
        for (byte_idx, ch) in trimmed_before.char_indices().rev() {
            if ch == '>' {
                depth = depth.saturating_add(1);
            } else if ch == '<' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    cut_byte_idx = Some(byte_idx);
                    break;
                }
            }
        }
        if let Some(idx) = cut_byte_idx {
            trimmed_before.get(..idx).unwrap_or(trimmed_before).trim_end()
        } else {
            trimmed_before
        }
    } else {
        trimmed_before
    };

    // Must have a return type token followed by whitespace and the callable identifier
    let mut rev_chars = without_trailing_generics.chars().rev();
    let mut ident_rev = String::new();
    for ch in &mut rev_chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident_rev.push(ch);
        } else if ch.is_whitespace() {
            break;
        } else {
            return None;
        }
    }

    let remaining_return_type: String = rev_chars.collect();
    if ident_rev.is_empty() || remaining_return_type.trim().is_empty() {
        return None;
    }

    let ident: String = ident_rev.chars().rev().collect();
    let first = ident.chars().next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    Some(ident)
}

/// Updates `brace_depth` and `paren_depth` across a single comment-stripped code line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic delimiter tracking (`update_line_delimiter_depths`).
/// Delegates to `update_ast_delimiter_depths` to track delimiters across single/double quotes,
/// raw strings, nested `${...}` interpolations, lifetimes, and unicode character literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - `paren_depth` increments on unquoted `'('` and saturatingly decrements on unquoted `')'`.
/// - `brace_depth` increments on unquoted `'{'` and saturatingly decrements on unquoted `'}'` if and
///   only if `*paren_depth == 0`.
/// - Characters inside string literals, raw string literals, or character literals are ignored.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped source line.
/// - `brace_depth: &mut usize` and `paren_depth: &mut usize` hold the current nesting depths.
///
/// ## 4. Postconditions
/// - Mutates `*brace_depth` and `*paren_depth` in place to reflect the net delimiter transitions on `code`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass byte scan over `code.len()`.
/// - Space Complexity: O(1) auxiliary state.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking byte lookups and saturating counter arithmetic.
/// - State: Updates only the two caller-provided depth references.
/// - Action: Skips escaped characters inside string and character literals safely.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-DELIMITER-DEPTH-TRACKER]
fn update_line_delimiter_depths(code: &str, brace_depth: &mut usize, paren_depth: &mut usize) {
    deap_core::source_ast::update_ast_delimiter_depths(code, brace_depth, paren_depth);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies delimiter depth tracking across Dart raw strings, interpolations, and UTF-8 char literals.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A delimiter tracking verification for `update_line_delimiter_depths`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Braces inside `'${map['{']}'` and `"${map["{"]}"` never corrupt outer `brace_depth`.
    ///
    /// ## 3. Preconditions
    /// - `brace_depth` and `paren_depth` start at `0`.
    ///
    /// ## 4. Postconditions
    /// - `brace_depth` increments to `1` and `paren_depth` returns to `0`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over the input line length.
    /// - Space Complexity: O(1) stack state.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Direct invocation of `update_line_delimiter_depths`.
    /// - State: Mutates local test counters only.
    /// - Action: Asserts exact depth values.
    /// - Resource: Zero heap allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-36-DELIMITER-DEPTH-TEST]
    #[test]
    fn test_update_line_delimiter_depths_dart_interpolation_raw_and_utf8_char() {
        let mut brace_depth: usize = 0;
        let mut paren_depth: usize = 0;

        update_line_delimiter_depths(
            "final sep = r'\\'; final raw = r\"\\\"; final msg = '${map['{']}'; final msg2 = \"${map[\"{\"]}\"; if (ch == '↑') {",
            &mut brace_depth,
            &mut paren_depth,
        );
        assert_eq!(
            brace_depth, 1,
            "brace_depth must increment to 1 on trailing '{{' without being corrupted by raw strings, '${{...}}' interpolations, or UTF-8 char literals"
        );
        assert_eq!(paren_depth, 0);
    }

    /// Verifies that double slashes inside string literals do not truncate lines during Check 36 scanning.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A lexical comment-stripping verification for `audit_rust_source_docstrings`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `//` inside `"https://example.com//path"` never strips closing quotes or braces on the same line.
    ///
    /// ## 3. Preconditions
    /// - In-memory Rust source fixtures contain URL string literals with `//`.
    ///
    /// ## 4. Postconditions
    /// - Undocumented follow-up functions are flagged with `Err`, and documented follow-up functions return `Ok(2)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(L) over the fixture lines.
    /// - Space Complexity: O(1) bounded fixture strings.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Direct `audit_rust_source_docstrings` evaluation.
    /// - State: Isolated per-call state.
    /// - Action: Verifies both negative and positive outcomes without `.unwrap()` or `.expect()`.
    /// - Resource: Zero filesystem side effects.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-36-URL-DOUBLE-SLASH-TEST]
    #[test]
    fn test_audit_rust_source_docstrings_preserves_double_slash_in_string_literal() {
        let rust_with_missing_fn = r#"/// Documented function with URL string literal containing double slashes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates URL prefix handling.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves delimiter balance across URL literals.
///
/// ## 3. Preconditions
/// - `endpoint` is a valid UTF-8 slice.
///
/// ## 4. Postconditions
/// - Returns true when matching.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total comparison with zero panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-TEST-URL-01]
pub fn check_endpoint(endpoint: &str) -> bool {
    let formatted = format!("https://example.com//path");
    if endpoint == formatted {
        return true;
    }
    false
}

pub fn missing_docs() {}
"#;
        match audit_rust_source_docstrings(rust_with_missing_fn, "crates/demo/src/lib.rs") {
            Ok(count) => panic!(
                "undocumented function after URL string literal must be flagged, got Ok({})",
                count
            ),
            Err(err) => {
                assert!(
                    err.iter().any(|v| v.contains("missing_docs")),
                    "Expected violation for missing_docs, got: {:?}",
                    err
                );
            }
        }

        let rust_all_documented = r#"/// Documented function with URL string literal containing double slashes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates URL prefix handling.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves delimiter balance across URL literals.
///
/// ## 3. Preconditions
/// - `endpoint` is a valid UTF-8 slice.
///
/// ## 4. Postconditions
/// - Returns true when matching.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total comparison with zero panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-TEST-URL-01]
pub fn check_endpoint(endpoint: &str) -> bool {
    let formatted = format!("https://example.com//path");
    if endpoint == formatted {
        return true;
    }
    false
}

/// Documented second function following URL string literal function.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Secondary operation after URL literal function.
///
/// ## 2. Invariants & Mathematical Properties
/// - Executes only after `check_endpoint` delimiter scope closes cleanly.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Completes deterministically.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible unit return.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-TEST-URL-02]
pub fn documented_followup() {}
"#;
        assert_eq!(
            audit_rust_source_docstrings(rust_all_documented, "crates/demo/src/lib.rs"),
            Ok(2)
        );
    }

    /// Verifies that Check 36 ignores `struct`, `fn`, `///`, and unbalanced delimiters inside `r##"..."##` and multiline strings.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 lexical boundary completeness for `audit_rust_source_docstrings` (refs #500).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Multi-hash raw strings (`r##"..."##`) containing single-hash `"#` substrings remain open until `"##` is reached.
    /// - Standard multiline string literals (`"..."`) spanning lines never trigger item classification or delimiter depth drift.
    ///
    /// ## 3. Preconditions
    /// - In-memory Rust source fixture contains `r##"..."##` and multiline `"..."` literals with embedded `pub fn` and `pub struct` tokens.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(2)` for the two real documented functions and returns `Err` when a subsequent real function is undocumented.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(L) over the fixture line count.
    /// - Space Complexity: O(1) bounded test fixture state.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Invokes `audit_rust_source_docstrings` directly with deterministic UTF-8 input.
    /// - State: Isolated per-invocation scanner state.
    /// - Action: Asserts both positive pass and negative fail-closed detection after multiline strings.
    /// - Resource: Zero filesystem side effects.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-36-MULTIHASH-RAW-STRING-TEST]
    #[test]
    fn test_audit_rust_source_docstrings_ignores_multihash_raw_and_multiline_strings() {
        let rust_source = concat!(
            "/// Primary host function containing multi-hash raw string and multiline standard string.\n",
            "///\n",
            "/// ## 1. Safety Intent & Regulatory Scope\n",
            "/// Exercises multi-hash raw string lexical scanning.\n",
            "///\n",
            "/// ## 2. Invariants & Mathematical Properties\n",
            "/// - Interior declarations inside string literals are inert.\n",
            "///\n",
            "/// ## 3. Preconditions\n",
            "/// - None.\n",
            "///\n",
            "/// ## 4. Postconditions\n",
            "/// - Returns static slice.\n",
            "///\n",
            "/// ## 5. Algorithmic Complexity Bounds\n",
            "/// - Time Complexity: O(1).\n",
            "/// - Space Complexity: O(1).\n",
            "///\n",
            "/// ## 6. Failure Modes & Fault Tolerance\n",
            "/// - Infallible static literal return.\n",
            "///\n",
            "/// ## 7. Realization & Traceability\n",
            "/// Realises: [REQ-TEST-RAW-01]\n",
            "pub fn host_with_raw_and_multiline() -> &'static str {\n",
            "    let raw_fixture = r##\"\n",
            "/// Member documentation.\n",
            "pub struct FakeRawStruct {\n",
            "    let inner_quote_hash = \"#;\n",
            "    pub fn fake_raw_fn(\n",
            "\"##;\n",
            "    let multiline_std = \"first line\n",
            "/// Member documentation.\n",
            "pub fn fake_std_multiline_fn(\n",
            "last line\";\n",
            "    if multiline_std.is_empty() {\n",
            "        return raw_fixture;\n",
            "    }\n",
            "    raw_fixture\n",
            "}\n\n",
            "/// Follow-up documented function after multi-hash raw string and multiline string.\n",
            "///\n",
            "/// ## 1. Safety Intent & Regulatory Scope\n",
            "/// Verifies delimiter depth recovery after multiline string literals.\n",
            "///\n",
            "/// ## 2. Invariants & Mathematical Properties\n",
            "/// - Delimiter depths `brace_depth` and `paren_depth` are zero at declaration entry.\n",
            "///\n",
            "/// ## 3. Preconditions\n",
            "/// - Prior function scope closed cleanly.\n",
            "///\n",
            "/// ## 4. Postconditions\n",
            "/// - Returns true.\n",
            "///\n",
            "/// ## 5. Algorithmic Complexity Bounds\n",
            "/// - Time Complexity: O(1).\n",
            "/// - Space Complexity: O(1).\n",
            "///\n",
            "/// ## 6. Failure Modes & Fault Tolerance\n",
            "/// - Pure boolean constant.\n",
            "///\n",
            "/// ## 7. Realization & Traceability\n",
            "/// Realises: [REQ-TEST-RAW-02]\n",
            "pub fn documented_after_raw() -> bool {\n",
            "    true\n",
            "}\n"
        );

        assert_eq!(
            audit_rust_source_docstrings(rust_source, "crates/demo/src/lib.rs"),
            Ok(2)
        );

        let mut rust_with_undocumented = String::from(rust_source);
        rust_with_undocumented.push_str("\npub fn undocumented_after_raw_string() {}\n");
        match audit_rust_source_docstrings(&rust_with_undocumented, "crates/demo/src/lib.rs") {
            Ok(count) => panic!("expected Err for undocumented_after_raw_string, got Ok({})", count),
            Err(errs) => {
                assert!(
                    errs.iter().any(|v| v.contains("undocumented_after_raw_string")),
                    "expected violation for undocumented_after_raw_string, got: {:?}",
                    errs
                );
            }
        }
    }

    /// Verifies that Check 36 ignores Dart declarations and placeholders inside triple-quoted multiline strings.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ISO 26262 ASIL D Dart lexical boundary completeness for `audit_dart_source_docstrings` (refs #500).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Triple-quoted Dart strings (`'''...'''` and `"""..."""`) never trigger item classification or placeholder errors on interior lines.
    ///
    /// ## 3. Preconditions
    /// - In-memory Dart source fixture contains triple-quoted strings with embedded `class`, `void`, and placeholder doc lines.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(2)` when all real Dart declarations carry valid 7-section docstrings.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(L) over the Dart fixture line count.
    /// - Space Complexity: O(1) bounded fixture allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Invokes `audit_dart_source_docstrings` directly with deterministic UTF-8 input.
    /// - State: Isolated per-invocation scanner state.
    /// - Action: Verifies zero false positives on triple-quoted string interiors.
    /// - Resource: Zero filesystem side effects.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-36-DART-TRIPLE-STRING-TEST]
    #[test]
    fn test_audit_dart_source_docstrings_ignores_triple_quoted_strings() {
        let dart_source = concat!(
            "/// Primary Dart function containing triple-quoted multiline string literals.\n",
            "///\n",
            "/// ## 1. Safety Intent & Regulatory Scope\n",
            "/// Validates Dart triple-quoted string scanning.\n",
            "///\n",
            "/// ## 2. Invariants & Mathematical Properties\n",
            "/// - Preserves brace and parenthesis depth across multiline strings.\n",
            "///\n",
            "/// ## 3. Preconditions\n",
            "/// - None.\n",
            "///\n",
            "/// ## 4. Postconditions\n",
            "/// - Returns non-empty string.\n",
            "///\n",
            "/// ## 5. Algorithmic Complexity Bounds\n",
            "/// - Time Complexity: O(1).\n",
            "/// - Space Complexity: O(1).\n",
            "///\n",
            "/// ## 6. Failure Modes & Fault Tolerance\n",
            "/// - Infallible constant string return.\n",
            "///\n",
            "/// ## 7. Realization & Traceability\n",
            "/// Realises: [REQ-TEST-DART-TRIPLE-01]\n",
            "String buildSnippet() {\n",
            "  const snippet = '''\n",
            "/// Member documentation.\n",
            "class FakeClassInTripleSingle {\n",
            "  void fakeMethod(\n",
            "''';\n",
            "  const snippet2 = \"\"\"\n",
            "/// Member documentation.\n",
            "void fakeTopLevelFn(\n",
            "\"\"\";\n",
            "  return snippet + snippet2;\n",
            "}\n\n",
            "/// Secondary Dart function after triple-quoted multiline strings.\n",
            "///\n",
            "/// ## 1. Safety Intent & Regulatory Scope\n",
            "/// Confirms clean delimiter state after triple-quoted strings.\n",
            "///\n",
            "/// ## 2. Invariants & Mathematical Properties\n",
            "/// - Evaluated at top-level depth (`brace_depth == 0`, `paren_depth == 0`).\n",
            "///\n",
            "/// ## 3. Preconditions\n",
            "/// - `buildSnippet` body closed cleanly.\n",
            "///\n",
            "/// ## 4. Postconditions\n",
            "/// - Completes deterministically.\n",
            "///\n",
            "/// ## 5. Algorithmic Complexity Bounds\n",
            "/// - Time Complexity: O(1).\n",
            "/// - Space Complexity: O(1).\n",
            "///\n",
            "/// ## 6. Failure Modes & Fault Tolerance\n",
            "/// - No-op execution.\n",
            "///\n",
            "/// ## 7. Realization & Traceability\n",
            "/// Realises: [REQ-TEST-DART-TRIPLE-02]\n",
            "void verifyAfterTripleQuote() {}\n"
        );

        assert_eq!(
            audit_dart_source_docstrings(dart_source, "app_flutter/lib/demo.dart"),
            Ok(2)
        );
    }
}

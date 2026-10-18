//! Source-code AST translator lowering Rust and Dart source trees into canonical SysML v2 `PackageDef` models.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! architecture extraction requirements (FEAT-42 Section 2.1 and Section 3.1). Bridges source
//! code trees (`--source <DIR> --format code --language <auto|dart|rust>`) into canonical
//! SysML v2 `PackageDef` AST representations without regular expressions.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero-Regex Delegation: Delegates lexical delimiter verification, AST extraction, and M2
//!   metamodel lowering to `deap_core::source_ast`.
//! - Deterministic Output: Multi-file directory translation sorts paths lexicographically before
//!   lowering into a single `PackageDef`.
//!
//! ## 3. Preconditions
//! - Input source buffers are valid UTF-8 strings or accessible filesystem paths containing
//!   `.rs` and/or `.dart` files.
//!
//! ## 4. Postconditions
//! - Returns `Ok(PackageDef)` containing lowered `PartDef`, `AttributeDef`, `PortDef`, `ActionDef`,
//!   `StateDef`, `ConstraintDef`, and `RequirementDef` nodes, or `Err(String)` on syntax/I/O error.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N)$ where $N$ is total source byte size across discovered files.
//! - Space Complexity: $O(N)$ for extracted `SourceAstItem` records and lowered `PackageDef` nodes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Unrecognized syntax, unclosed braces, or unreadable files return `Err(String)`.
//! - State: Stateless translator record parameterized only by `SourceLanguage`.
//! - Action: Fails closed on the first malformed source file without partial output writes.
//! - Resource: Linear memory proportional to parsed AST node count.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-FEAT-42-CODE-TRANSLATOR], [DO-330-TQL-4-CODE-TRANSLATOR]

use deap_core::source_ast::{
    ingest_source_directory, lower_source_ast_to_package, parse_dart_source_ast,
    parse_rust_source_ast, SourceAstItem, SourceLanguage,
};
use deap_core::sysml_ast::PackageDef;
use std::fs;
use std::path::{Path, PathBuf};

/// Deterministic translator converting Rust and Dart source code into canonical SysML v2 `PackageDef` models.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.1 source-to-SysML ingestion (`CodeTranslator`).
/// Encapsulates the active `SourceLanguage` mode (`Auto`, `Rust`, `Dart`) for single-buffer,
/// file-list, and directory-tree source AST translation.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure value configuration: `language` determines whether `.rs`, `.dart`, or both are parsed.
/// - Zero mutable internal state across translation invocations.
///
/// ## 3. Preconditions
/// - Constructed via `CodeTranslator::new()` or `CodeTranslator::with_language(language)`.
///
/// ## 4. Postconditions
/// - Exposes deterministic `translate`, `translate_files`, and `translate_directory` methods.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) construction.
/// - Space Complexity: O(1) stack footprint.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed `SourceLanguage` field prevents invalid language state.
/// - State: Immutable after construction.
/// - Action: Infallible instantiation.
/// - Resource: Zero heap allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-STRUCT]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeTranslator {
    /// Selected source language parser mode (`Auto`, `Rust`, or `Dart`).
    pub language: SourceLanguage,
}

impl Default for CodeTranslator {
    /// Constructs a default `CodeTranslator` initialized with `SourceLanguage::Auto`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A deterministic default configuration for `CodeTranslator::default`.
    /// Ensures `--language auto` is the baseline when no explicit language override is supplied.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Always returns `CodeTranslator { language: SourceLanguage::Auto }`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `CodeTranslator` in `Auto` language detection mode.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant time.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Standard `Default::default` trait implementation.
    /// - State: Pure constant construction.
    /// - Action: Cannot fail or panic.
    /// - Resource: Zero dynamic memory usage.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-DEFAULT]
    fn default() -> Self {
        Self::new()
    }
}

impl CodeTranslator {
    /// Creates a new `CodeTranslator` in `SourceLanguage::Auto` mode.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A constructor determinism (`CodeTranslator::new`).
    /// Provides a `const fn` constructor for automatic `.rs` and `.dart` source AST ingestion.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `self.language` to `SourceLanguage::Auto`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns `CodeTranslator { language: SourceLanguage::Auto }`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) compile-time or runtime constant initialization.
    /// - Space Complexity: O(1) stack value.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Parameterless `const fn`.
    /// - State: Zero mutable state.
    /// - Action: Infallible.
    /// - Resource: Zero heap allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-NEW]
    pub const fn new() -> Self {
        Self {
            language: SourceLanguage::Auto,
        }
    }

    /// Creates a `CodeTranslator` bound to an explicit `SourceLanguage` (`Auto`, `Rust`, or `Dart`).
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and FEAT-42 Section 2.1 `--language` contract (`with_language`).
    /// Restricts AST extraction to the caller-specified source language grammar.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves `self.language == language` identically.
    ///
    /// ## 3. Preconditions
    /// - `language: SourceLanguage` is a validated language selector.
    ///
    /// ## 4. Postconditions
    /// - Returns a `CodeTranslator` configured with `language`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant time.
    /// - Space Complexity: O(1) stack value.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts `SourceLanguage` by value.
    /// - State: Pure constructor.
    /// - Action: Infallible.
    /// - Resource: Zero heap allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-WITH-LANG]
    pub const fn with_language(language: SourceLanguage) -> Self {
        Self { language }
    }

    /// Translates a single in-memory source string (`content`) into a canonical SysML v2 `PackageDef`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and FEAT-42 AC-01/AC-02/AC-05 single-file translation (`CodeTranslator::translate`).
    /// Determines whether `content` is Rust or Dart from `self.language` or `file_name`, verifies
    /// delimiter balance, extracts `SourceAstItem` nodes, and lowers them to `PackageDef`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Halts closed with `Err(String)` containing `line:col` coordinates if `content` has unbalanced
    ///   braces, parentheses, brackets, or string/comment literals.
    ///
    /// ## 3. Preconditions
    /// - `content: &str` is valid UTF-8 Rust or Dart source text.
    /// - `file_name: &str` is the source filename or package identifier.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(PackageDef)` on clean parse and lowering, or `Err(String)` on syntax error.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is `content.len()`.
    /// - Space Complexity: O(N) proportional to extracted AST items and lowered `PackageDef` elements.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates syntax/delimiter errors directly from `parse_rust_source_ast` or `parse_dart_source_ast`.
    /// - State: Pure transformation with zero side effects.
    /// - Action: Never panics on malformed source text.
    /// - Resource: Linear memory bounded by `content.len()`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-BUFFER]
    pub fn translate(&self, content: &str, file_name: &str) -> Result<PackageDef, String> {
        let resolved_lang = match self.language {
            SourceLanguage::Auto => SourceLanguage::detect_from_extension(Path::new(file_name))
                .unwrap_or_else(|| {
                    if content.contains("extends StatefulWidget")
                        || content.contains("extends StatelessWidget")
                        || content.contains("Widget build(")
                        || content.contains("import 'package:")
                    {
                        SourceLanguage::Dart
                    } else {
                        SourceLanguage::Rust
                    }
                }),
            explicit => explicit,
        };

        let items = match resolved_lang {
            SourceLanguage::Rust | SourceLanguage::Auto => {
                parse_rust_source_ast(content, file_name)?
            }
            SourceLanguage::Dart => parse_dart_source_ast(content, file_name)?,
        };

        let pkg_name = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);

        Ok(lower_source_ast_to_package(&items, pkg_name))
    }

    /// Translates an explicit slice of `.rs` and/or `.dart` file paths into a consolidated `PackageDef`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A batch source translation (`CodeTranslator::translate_files`).
    /// Reads and parses each file in `file_paths` in deterministic order and lowers the combined
    /// `SourceAstItem` stream into a single SysML v2 `PackageDef`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - All files in `file_paths` must pass syntax verification before the lowered `PackageDef` is returned.
    /// - Associates `impl` blocks and `State<Widget>` classes across files within the same package.
    ///
    /// ## 3. Preconditions
    /// - `file_paths: &[PathBuf]` is a non-empty slice of readable `.rs` or `.dart` file paths.
    /// - `package_name: &str` is the target SysML v2 package identifier.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(PackageDef)` aggregating all parsed source items, or `Err(String)` on any I/O or syntax failure.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is the sum of byte lengths across `file_paths`.
    /// - Space Complexity: O(N) for accumulated `SourceAstItem` and `PackageDef` structures.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Err(String)` if `file_paths` is empty or any file cannot be read or parsed.
    /// - State: Local vector accumulation with zero global state.
    /// - Action: Fails closed immediately on the first defective file.
    /// - Resource: Drops per-file text buffers after each iteration.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-FILES]
    pub fn translate_files(
        &self,
        file_paths: &[PathBuf],
        package_name: &str,
    ) -> Result<PackageDef, String> {
        if file_paths.is_empty() {
            return Err("No source files provided to CodeTranslator::translate_files".to_string());
        }
        let mut all_items: Vec<SourceAstItem> = Vec::new();
        for path in file_paths {
            let content = fs::read_to_string(path).map_err(|e| {
                format!("Failed to read source file '{}': {}", path.display(), e)
            })?;
            let label = path.display().to_string();
            let file_lang = match self.language {
                SourceLanguage::Auto => {
                    SourceLanguage::detect_from_extension(path).unwrap_or(SourceLanguage::Rust)
                }
                explicit => explicit,
            };
            let mut items = match file_lang {
                SourceLanguage::Rust | SourceLanguage::Auto => {
                    parse_rust_source_ast(&content, &label)?
                }
                SourceLanguage::Dart => parse_dart_source_ast(&content, &label)?,
            };
            all_items.append(&mut items);
        }
        Ok(lower_source_ast_to_package(&all_items, package_name))
    }

    /// Recursively discovers and translates all eligible source files under `source_dir` into a `PackageDef`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and FEAT-42 Section 2.1 `--source <DIR>` ingestion (`translate_directory`).
    /// Delegates directory traversal, syntax verification, and M2 metamodel lowering to `ingest_source_directory`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Identical directory trees and `self.language` settings always produce bit-for-bit identical `PackageDef` trees.
    ///
    /// ## 3. Preconditions
    /// - `source_dir: &Path` points to an existing source file or directory on disk.
    /// - `package_name: &str` is the desired root `PackageDef` name.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(PackageDef)` when all discovered source files parse cleanly, or `Err(String)` on error.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N log N) over total directory file count and byte length N.
    /// - Space Complexity: O(N) for the lowered `PackageDef`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates missing-directory, empty-directory, and syntax errors via `Result::Err`.
    /// - State: Stateless delegation.
    /// - Action: Halts closed without modifying disk state.
    /// - Resource: Bounded by source tree size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-DIR]
    pub fn translate_directory(
        &self,
        source_dir: &Path,
        package_name: &str,
    ) -> Result<PackageDef, String> {
        ingest_source_directory(source_dir, self.language, package_name)
    }
}
